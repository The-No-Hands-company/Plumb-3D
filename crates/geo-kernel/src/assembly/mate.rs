//! Kinematic CAD Mates and Mate Connectors
//!
//! Provides the mathematical foundations for multi-body assemblies:
//! - [`MateConnector`]: 3D reference coordinate frame (origin, Z-axis normal, X-axis alignment).
//! - [`MateType`]: Coincident, Concentric, Revolute, Prismatic, Cylindrical, Gear, Fixed.
//! - [`KinematicMate`]: Pairwise kinematic constraint coupling two bodies.

use core_math::{Mat4, Quat, Transform, Vec3};
use serde::{Deserialize, Serialize};

/// 3D Reference coordinate frame attached to a solid body face, edge, or vertex.
/// The normal vector acts as the primary Z-axis (mating axis),
/// and the tangent vector acts as the secondary X-axis (orientation/alignment).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MateConnector {
    /// Origin position in the body's local space
    pub origin: Vec3,
    /// Normal axis (Z-axis, e.g. face normal or cylinder revolution axis)
    pub normal: Vec3,
    /// Tangent axis (X-axis, orthogonal to normal)
    pub tangent: Vec3,
}

impl Default for MateConnector {
    fn default() -> Self {
        Self {
            origin: Vec3::ZERO,
            normal: Vec3::Z,
            tangent: Vec3::X,
        }
    }
}

impl MateConnector {
    /// Create a new mate connector with explicit origin, normal, and tangent
    pub fn new(origin: Vec3, normal: Vec3, tangent: Vec3) -> Self {
        let n = if normal.length_squared() > 1e-6 {
            normal.normalize()
        } else {
            Vec3::Z
        };
        // Ensure tangent is strictly perpendicular to normal
        let mut t = tangent - n * n.dot(tangent);
        if t.length_squared() < 1e-6 {
            let arbitrary = if n.x.abs() < 0.9 { Vec3::X } else { Vec3::Y };
            t = (arbitrary - n * n.dot(arbitrary)).normalize();
        } else {
            t = t.normalize();
        }

        Self {
            origin,
            normal: n,
            tangent: t,
        }
    }

    /// Construct a mate connector from an origin point and normal vector
    pub fn from_origin_and_normal(origin: Vec3, normal: Vec3) -> Self {
        let n = if normal.length_squared() > 1e-6 {
            normal.normalize()
        } else {
            Vec3::Z
        };
        let arbitrary = if n.x.abs() < 0.9 { Vec3::X } else { Vec3::Y };
        let tangent = (arbitrary - n * n.dot(arbitrary)).normalize();
        Self {
            origin,
            normal: n,
            tangent,
        }
    }

    /// Compute the binormal (Y-axis = Z x X)
    #[inline]
    pub fn binormal(&self) -> Vec3 {
        self.normal.cross(self.tangent).normalize()
    }

    /// Rotation quaternion transforming default basis (X, Y, Z) to (tangent, binormal, normal)
    pub fn rotation(&self) -> Quat {
        let rot_mat = Mat4::from_cols(
            self.tangent.extend(0.0),
            self.binormal().extend(0.0),
            self.normal.extend(0.0),
            Vec3::ZERO.extend(1.0),
        );
        Quat::from_mat4(&rot_mat)
    }

    /// Convert connector to a local Transform
    pub fn to_transform(&self) -> Transform {
        Transform {
            translation: self.origin,
            rotation: self.rotation(),
            scale: Vec3::ONE,
        }
    }

    /// Transform this connector by a parent body transform
    pub fn transform_by(&self, transform: &Transform) -> Self {
        let world_origin = transform.transform_point(self.origin);
        let world_normal = (transform.rotation * self.normal).normalize();
        let world_tangent = (transform.rotation * self.tangent).normalize();
        Self {
            origin: world_origin,
            normal: world_normal,
            tangent: world_tangent,
        }
    }
}

/// Relative orientation alignment between two mate connectors
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum MateAlignment {
    /// Normals point in the exact same direction (coplanar / flush)
    Aligned,
    /// Normals point against each other (face-to-face contact)
    #[default]
    AntiAligned,
}

/// CAD Kinematic Mate joint types
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum MateType {
    /// Planar or point coincident mate: locks translation along normal axis.
    /// Allows 2 translational DOFs on the plane and 1 rotational DOF around normal.
    Coincident,
    /// Concentric mate: aligns two cylindrical axes.
    /// Allows 1 rotational DOF around axis and 1 translational DOF along axis unless axial lock is enabled.
    Concentric { axial_lock: bool },
    /// Revolute hinge joint: locks all translations, allows 1 rotational DOF around the connector normal.
    Revolute {
        limits: Option<(f32, f32)>, // (min_rad, max_rad)
    },
    /// Prismatic slider joint: locks all rotations, allows 1 translational DOF along the connector tangent (or normal).
    Prismatic {
        limits: Option<(f32, f32)>, // (min_dist, max_dist)
    },
    /// Cylindrical joint: allows simultaneous 1 rotational DOF around axis and 1 translational DOF along axis.
    Cylindrical {
        angle_limits: Option<(f32, f32)>,
        linear_limits: Option<(f32, f32)>,
    },
    /// Gear / Coupler joint: couples rotation of driver body to driven body by a ratio
    Gear { ratio: f32 },
    /// Fixed mate: grounds or rigidly welds the second body to the first body (0 DOFs).
    Fixed,
}

/// A Kinematic Mate constraining two bodies in an assembly
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KinematicMate {
    pub id: usize,
    pub name: String,
    pub first_body: usize,
    pub first_connector: MateConnector,
    pub second_body: usize,
    pub second_connector: MateConnector,
    pub mate_type: MateType,
    pub alignment: MateAlignment,
    /// Linear offset distance along the mating normal
    pub offset: f32,
    /// Current actuated parameter (angle in radians for revolute, distance for prismatic)
    pub current_param: f32,
    pub enabled: bool,
}

impl KinematicMate {
    /// Construct a new kinematic mate between two bodies
    pub fn new(
        id: usize,
        name: impl Into<String>,
        first_body: usize,
        first_connector: MateConnector,
        second_body: usize,
        second_connector: MateConnector,
        mate_type: MateType,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            first_body,
            first_connector,
            second_body,
            second_connector,
            mate_type,
            alignment: MateAlignment::AntiAligned,
            offset: 0.0,
            current_param: 0.0,
            enabled: true,
        }
    }

    /// Returns the number of unconstrained degrees of freedom (DOFs) permitted by this mate
    pub fn unconstrained_dof(&self) -> (u8, u8) {
        // Returns (translation_dof, rotation_dof)
        match self.mate_type {
            MateType::Fixed => (0, 0),
            MateType::Revolute { .. } => (0, 1),
            MateType::Prismatic { .. } => (1, 0),
            MateType::Cylindrical { .. } => (1, 1),
            MateType::Concentric { axial_lock: true } => (0, 1),
            MateType::Concentric { axial_lock: false } => (1, 1),
            MateType::Coincident => (2, 1),
            MateType::Gear { .. } => (0, 1),
        }
    }

    /// Clamp current parameter within limits if defined
    pub fn clamp_param(&mut self) {
        match self.mate_type {
            MateType::Revolute { limits: Some((min, max)) } => {
                self.current_param = self.current_param.clamp(min, max);
            }
            MateType::Prismatic { limits: Some((min, max)) } => {
                self.current_param = self.current_param.clamp(min, max);
            }
            MateType::Cylindrical { linear_limits: Some((min, max)), .. } => {
                self.current_param = self.current_param.clamp(min, max);
            }
            _ => {}
        }
    }
}

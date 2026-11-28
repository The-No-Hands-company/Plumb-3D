//! Multi-View Orthographic Projection Cameras and Sheet Transformations
//!
//! Provides First-Angle (ISO) and Third-Angle (ANSI) orthographic projection
//! engines for generating engineering drawing views from 3D solid models.

use core_math::{Mat4, Vec2, Vec3};
use serde::{Deserialize, Serialize};

/// Standard engineering drawing views
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ViewType {
    /// Front elevation view (looking along -Y in Z-up or along -Z in Y-up)
    Front,
    /// Top plan view (looking down from +Y)
    Top,
    /// Right end view (looking from +X)
    Right,
    /// Left end view (looking from -X)
    Left,
    /// Bottom view (looking up from -Y)
    Bottom,
    /// Rear view (looking from -Z)
    Rear,
    /// Axonometric 30° / 30° Isometric pictorial projection
    Isometric,
}

/// Drawing projection standard
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ProjectionStandard {
    /// ISO First-Angle Projection (Europe & International standard):
    /// - Front view is central
    /// - Top view placed below Front
    /// - Right view placed to the left of Front
    #[default]
    FirstAngle,
    /// ANSI/ASME Third-Angle Projection (North America standard):
    /// - Front view is central
    /// - Top view placed directly above Front
    /// - Right view placed directly to the right of Front
    ThirdAngle,
}

/// An orthographic camera for projecting 3D model geometry onto 2D drawing sheet views
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ViewCamera {
    pub view_type: ViewType,
    /// Scale factor relating 3D model units (mm) to 2D paper sheet units (mm), e.g. 1.0 = 1:1, 0.5 = 1:2
    pub scale: f32,
    /// 2D center position on the paper sheet (mm from bottom-left origin)
    pub sheet_center: Vec2,
    /// 3D model center focus point
    pub target: Vec3,
}

impl ViewCamera {
    pub fn new(view_type: ViewType, sheet_center: Vec2, scale: f32, target: Vec3) -> Self {
        Self {
            view_type,
            scale: scale.max(0.001),
            sheet_center,
            target,
        }
    }

    /// Primary view direction vector (camera sightline in world space)
    pub fn view_direction(&self) -> Vec3 {
        match self.view_type {
            ViewType::Front => Vec3::new(0.0, 0.0, -1.0),
            ViewType::Top => Vec3::new(0.0, -1.0, 0.0),
            ViewType::Right => Vec3::new(-1.0, 0.0, 0.0),
            ViewType::Left => Vec3::new(1.0, 0.0, 0.0),
            ViewType::Bottom => Vec3::new(0.0, 1.0, 0.0),
            ViewType::Rear => Vec3::new(0.0, 0.0, 1.0),
            ViewType::Isometric => Vec3::new(-1.0, -1.0, -1.0).normalize(),
        }
    }

    /// View up vector in world space
    pub fn up_direction(&self) -> Vec3 {
        match self.view_type {
            ViewType::Front | ViewType::Right | ViewType::Left | ViewType::Rear => Vec3::Y,
            ViewType::Top => Vec3::new(0.0, 0.0, -1.0),
            ViewType::Bottom => Vec3::new(0.0, 0.0, 1.0),
            ViewType::Isometric => Vec3::new(-1.0, 2.0, -1.0).normalize(),
        }
    }

    /// View right vector (X-axis of the 2D projected view)
    pub fn right_direction(&self) -> Vec3 {
        self.view_direction().cross(self.up_direction()).normalize()
    }

    /// Projects a 3D world coordinate to 2D paper sheet coordinates (x, y) and view depth (z)
    pub fn project_point(&self, p: Vec3) -> (Vec2, f32) {
        let v = p - self.target;
        let right = self.right_direction();
        let up = self.up_direction();
        let forward = self.view_direction();

        let x_local = v.dot(right) * self.scale;
        let y_local = v.dot(up) * self.scale;
        let depth = v.dot(forward); // Larger depth means further along view direction away from eye

        let sheet_pos = self.sheet_center + Vec2::new(x_local, y_local);
        (sheet_pos, depth)
    }

    /// View-projection matrix transformation
    pub fn to_mat4(&self) -> Mat4 {
        let eye = self.target - self.view_direction() * 1000.0;
        Mat4::look_at_rh(eye, self.target, self.up_direction())
    }
}

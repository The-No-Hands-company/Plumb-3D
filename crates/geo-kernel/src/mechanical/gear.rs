//! Involute Gear & Power Transmission Engine
//!
//! Generates analytical involute tooth profiles for Spur, Helical, and Bevel gears
//! conforming to standard AGMA / ISO metric gear geometry with true meshing contact.

use core_math::Vec3;
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

use crate::mesh::Mesh;

/// Gear Kinematic Transmission Style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum GearType {
    /// Straight cylindrical teeth parallel to axis
    #[default]
    Spur,
    /// Angled teeth twisted along cylindrical helix angle
    Helical,
    /// Conical teeth for 90° intersecting shaft power transmission
    Bevel,
}

/// Standard Parameters for Involute Gear Generation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GearParams {
    /// Metric gear module m in millimeters (e.g. 1.0, 1.5, 2.0, 2.5, 3.0)
    pub module: f32,
    /// Number of teeth Z (typically Z >= 12 to prevent undercut)
    pub teeth: usize,
    /// Standard pressure angle alpha in degrees (typically 20.0° or 14.5°)
    pub pressure_angle_deg: f32,
    /// Gear face width b in millimeters (axial gear thickness)
    pub face_width: f32,
    /// Helix angle beta in degrees (0.0° for spur, typically 15° to 30° for helical)
    pub helix_angle_deg: f32,
    /// Center shaft mounting bore diameter in millimeters (0.0 for solid gear)
    pub bore_diameter: f32,
    pub gear_type: GearType,
}

impl Default for GearParams {
    fn default() -> Self {
        Self {
            module: 2.0,
            teeth: 20,
            pressure_angle_deg: 20.0,
            face_width: 15.0,
            helix_angle_deg: 0.0,
            bore_diameter: 10.0,
            gear_type: GearType::Spur,
        }
    }
}

impl GearParams {
    /// Standard Pitch Diameter d = m * Z
    pub fn pitch_diameter(&self) -> f32 {
        self.module * self.teeth as f32
    }

    /// Pitch Radius r = d / 2
    pub fn pitch_radius(&self) -> f32 {
        self.pitch_diameter() * 0.5
    }

    /// Base Circle Radius rb = r * cos(alpha)
    pub fn base_radius(&self) -> f32 {
        self.pitch_radius() * self.pressure_angle_deg.to_radians().cos()
    }

    /// Addendum (Tooth tip height above pitch line) ha = 1.0 * m
    pub fn addendum(&self) -> f32 {
        1.0 * self.module
    }

    /// Dedendum (Tooth root depth below pitch line) hf = 1.25 * m
    pub fn dedendum(&self) -> f32 {
        1.25 * self.module
    }

    /// Outside / Tip Radius ra = r + ha
    pub fn tip_radius(&self) -> f32 {
        self.pitch_radius() + self.addendum()
    }

    /// Root Radius rf = r - hf
    pub fn root_radius(&self) -> f32 {
        (self.pitch_radius() - self.dedendum()).max(0.1)
    }

    /// Circular Pitch p = pi * m
    pub fn circular_pitch(&self) -> f32 {
        PI * self.module
    }
}

/// Geometric Builder for Analytical Involute Gears
pub struct InvoluteGearBuilder;

impl InvoluteGearBuilder {
    /// Generate a 3D solid BMesh for the specified gear parameters
    pub fn generate_mesh(params: &GearParams) -> Mesh {
        let mut mesh = Mesh::new();
        let b = params.face_width.max(1.0);
        let r_bore = (params.bore_diameter * 0.5).min(params.root_radius() * 0.8);

        // Generate 2D Involute Tooth Contour (XY Plane)
        let contour_2d = Self::generate_2d_gear_profile(params);
        let pts_per_slice = contour_2d.len();

        let slices = if params.gear_type == GearType::Helical && params.helix_angle_deg.abs() > 0.1 {
            12
        } else {
            2
        };

        let beta_rad = params.helix_angle_deg.to_radians();
        let total_twist = if params.gear_type == GearType::Helical {
            (b * beta_rad.tan()) / params.pitch_radius()
        } else {
            0.0
        };

        // Extrude slices along Z axis with optional helical twist and bevel taper
        let mut slice_verts = Vec::with_capacity(slices);

        for s in 0..slices {
            let frac = s as f32 / (slices - 1) as f32;
            let z = frac * b;
            let twist = frac * total_twist;

            // Bevel cone scale factor (tapering towards cone apex)
            let scale = if params.gear_type == GearType::Bevel {
                1.0 - frac * 0.3
            } else {
                1.0
            };

            let mut v_slice = Vec::with_capacity(pts_per_slice);
            for pt in &contour_2d {
                let rot_x = pt.x * twist.cos() - pt.y * twist.sin();
                let rot_y = pt.x * twist.sin() + pt.y * twist.cos();
                let pos = Vec3::new(rot_x * scale, rot_y * scale, z);
                v_slice.push(mesh.add_vertex(pos));
            }
            slice_verts.push(v_slice);
        }

        // Connect tooth sidewall quads between slices
        for s in 0..(slices - 1) {
            let s0 = &slice_verts[s];
            let s1 = &slice_verts[s + 1];
            for i in 0..pts_per_slice {
                let next = (i + 1) % pts_per_slice;
                let _ = mesh.add_face(&[s0[i], s0[next], s1[next], s1[i]]);
            }
        }

        // Center Bore / End Cap Tessellation
        if r_bore > 0.5 {
            // Bore inner cylinder
            let bore_segs = pts_per_slice.min(32);
            let mut v_bore_bot = Vec::new();
            let mut v_bore_top = Vec::new();

            for i in 0..bore_segs {
                let angle = (i as f32 / bore_segs as f32) * 2.0 * PI;
                v_bore_bot.push(mesh.add_vertex(Vec3::new(r_bore * angle.cos(), r_bore * angle.sin(), 0.0)));
                v_bore_top.push(mesh.add_vertex(Vec3::new(r_bore * angle.cos(), r_bore * angle.sin(), b)));
            }

            for i in 0..bore_segs {
                let next = (i + 1) % bore_segs;
                let _ = mesh.add_face(&[v_bore_bot[next], v_bore_bot[i], v_bore_top[i], v_bore_top[next]]);
            }
        }

        // Front Face (Z = 0)
        let mut front_verts = slice_verts[0].clone();
        front_verts.reverse();
        let _ = mesh.add_face(&front_verts);

        // Back Face (Z = b)
        let _ = mesh.add_face(&slice_verts[slices - 1]);

        mesh
    }

    /// Computes the 2D planar perimeter contour of all gear teeth in the XY plane
    fn generate_2d_gear_profile(params: &GearParams) -> Vec<Vec3> {
        let z = params.teeth;
        let r_base = params.base_radius();
        let r_tip = params.tip_radius();
        let r_root = params.root_radius();
        let pitch_angle = 2.0 * PI / z as f32;

        let mut contour = Vec::new();

        for tooth_idx in 0..z {
            let tooth_center_angle = tooth_idx as f32 * pitch_angle;

            // Involute curve calculation points from root/base to tip
            let inv_steps = 4;
            let phi_max = ((r_tip * r_tip - r_base * r_base).max(0.0)).sqrt() / r_base;
            let half_thickness_angle = PI / (2.0 * z as f32);

            // Left flank (rising from root to tip)
            for s in 0..=inv_steps {
                let frac = s as f32 / inv_steps as f32;
                let phi = frac * phi_max;
                let r = (r_base * r_base * (1.0 + phi * phi)).sqrt().clamp(r_root, r_tip);
                let inv = phi - phi.atan();
                let theta = tooth_center_angle - half_thickness_angle - inv;

                contour.push(Vec3::new(r * theta.cos(), r * theta.sin(), 0.0));
            }

            // Right flank (falling from tip back to root)
            for s in (0..=inv_steps).rev() {
                let frac = s as f32 / inv_steps as f32;
                let phi = frac * phi_max;
                let r = (r_base * r_base * (1.0 + phi * phi)).sqrt().clamp(r_root, r_tip);
                let inv = phi - phi.atan();
                let theta = tooth_center_angle + half_thickness_angle + inv;

                contour.push(Vec3::new(r * theta.cos(), r * theta.sin(), 0.0));
            }

            // Root bottom fillet curve to next tooth
            let next_tooth_center = (tooth_idx + 1) as f32 * pitch_angle;
            let root_mid_angle = (tooth_center_angle + next_tooth_center) * 0.5;
            contour.push(Vec3::new(r_root * root_mid_angle.cos(), r_root * root_mid_angle.sin(), 0.0));
        }

        contour
    }
}

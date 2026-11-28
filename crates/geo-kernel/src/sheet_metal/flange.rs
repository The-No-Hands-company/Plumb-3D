//! Sheet Metal Base Flange & Edge Flange Generators
//!
//! Generates solid 3D sheet metal geometry with uniform thickness,
//! cylindrical bend transitions, and edge flanges.

use core_math::{Quat, Vec3};
use serde::{Deserialize, Serialize};

use super::bend::{BendDirection, SheetBend};
use super::params::SheetMetalParams;
use crate::mesh::Mesh;

/// Parameters for creating a planar sheet metal base plate
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BaseFlangeSettings {
    pub width: f32,
    pub length: f32,
    pub params: SheetMetalParams,
}

impl Default for BaseFlangeSettings {
    fn default() -> Self {
        Self {
            width: 100.0,
            length: 100.0,
            params: SheetMetalParams::default(),
        }
    }
}

/// Parameters for extruding an angled edge flange along a sheet boundary
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EdgeFlangeSettings {
    /// Flange length in millimeters extending from bend line
    pub length: f32,
    /// Bend angle in degrees (typically 90.0°)
    pub angle_deg: f32,
    /// Inner bend radius (if None, uses params default)
    pub bend_radius: Option<f32>,
    pub direction: BendDirection,
}

impl Default for EdgeFlangeSettings {
    fn default() -> Self {
        Self {
            length: 30.0,
            angle_deg: 90.0,
            bend_radius: None,
            direction: BendDirection::Up,
        }
    }
}

pub struct SheetMetalBuilder;

impl SheetMetalBuilder {
    /// Generate a 3D solid base flange plate with uniform thickness
    pub fn create_base_flange(settings: &BaseFlangeSettings) -> Mesh {
        let mut mesh = Mesh::new();
        let hw = settings.width * 0.5;
        let hl = settings.length * 0.5;
        let t = settings.params.thickness;

        // Bottom 4 vertices (Z = 0)
        let v0 = mesh.add_vertex(Vec3::new(-hw, -hl, 0.0));
        let v1 = mesh.add_vertex(Vec3::new(hw, -hl, 0.0));
        let v2 = mesh.add_vertex(Vec3::new(hw, hl, 0.0));
        let v3 = mesh.add_vertex(Vec3::new(-hw, hl, 0.0));

        // Top 4 vertices (Z = T)
        let v4 = mesh.add_vertex(Vec3::new(-hw, -hl, t));
        let v5 = mesh.add_vertex(Vec3::new(hw, -hl, t));
        let v6 = mesh.add_vertex(Vec3::new(hw, hl, t));
        let v7 = mesh.add_vertex(Vec3::new(-hw, hl, t));

        // Bottom face (-Z)
        let _ = mesh.add_face(&[v0, v3, v2, v1]);
        // Top face (+Z)
        let _ = mesh.add_face(&[v4, v5, v6, v7]);
        // Front face (-Y)
        let _ = mesh.add_face(&[v0, v1, v5, v4]);
        // Back face (+Y)
        let _ = mesh.add_face(&[v2, v3, v7, v6]);
        // Right face (+X)
        let _ = mesh.add_face(&[v1, v2, v6, v5]);
        // Left face (-X)
        let _ = mesh.add_face(&[v3, v0, v4, v7]);

        mesh
    }

    /// Add an angled edge flange with cylindrical bend transition along a sheet side
    pub fn add_edge_flange(
        base_mesh: &Mesh,
        settings: &EdgeFlangeSettings,
        params: &SheetMetalParams,
        flange_edge_x: f32, // X coordinate of the bending edge
        sheet_length: f32,  // Y width of the flange along the edge
    ) -> (Mesh, SheetBend) {
        let mut mesh = base_mesh.clone();
        let r_inner = settings.bend_radius.unwrap_or(params.bend_radius);
        let t = params.thickness;
        let hl = sheet_length * 0.5;
        let angle_rad = settings.angle_deg.to_radians();
        let segments = 6; // Arc tessellation steps

        // Axis along the Y edge
        let axis_start = Vec3::new(flange_edge_x, -hl, 0.0);
        let axis_end = Vec3::new(flange_edge_x, hl, 0.0);

        let bend = SheetBend::new(
            0,
            "EdgeFlangeBend",
            settings.angle_deg,
            r_inner,
            settings.direction,
            axis_start,
            axis_end,
            params,
        );

        // Generate cylindrical bend profile vertices
        let mut arc_pts_inner = Vec::new();
        let mut arc_pts_outer = Vec::new();

        // Center of bend cylinder
        let center_z = if settings.direction == BendDirection::Up {
            r_inner
        } else {
            -r_inner
        };

        for i in 0..=segments {
            let frac = i as f32 / segments as f32;
            let theta = frac * angle_rad;

            // Inner surface arc
            let (dx_in, dz_in) = (r_inner * theta.sin(), -r_inner * (1.0 - theta.cos()));
            // Outer surface arc
            let (dx_out, dz_out) = ((r_inner + t) * theta.sin(), -(r_inner + t) * (1.0 - theta.cos()));

            arc_pts_inner.push((flange_edge_x + dx_in, center_z + dz_in));
            arc_pts_outer.push((flange_edge_x + dx_out, center_z + dz_out + t));
        }

        // Add planar flange extension from end of arc
        let last_in = *arc_pts_inner.last().unwrap();
        let last_out = *arc_pts_outer.last().unwrap();
        let flange_rot = Quat::from_rotation_y(angle_rad);
        let flange_dir = flange_rot * Vec3::Z;

        let ext_in = Vec3::new(last_in.0, 0.0, last_in.1) + flange_dir * settings.length;
        let ext_out = Vec3::new(last_out.0, 0.0, last_out.1) + flange_dir * settings.length;

        // Build 3D flange quad faces
        let v_in_0 = mesh.add_vertex(Vec3::new(last_in.0, -hl, last_in.1));
        let v_in_1 = mesh.add_vertex(Vec3::new(last_in.0, hl, last_in.1));
        let v_ext_1 = mesh.add_vertex(Vec3::new(ext_in.x, hl, ext_in.z));
        let v_ext_0 = mesh.add_vertex(Vec3::new(ext_in.x, -hl, ext_in.z));
        let _ = mesh.add_face(&[v_in_0, v_in_1, v_ext_1, v_ext_0]);

        let v_out_0 = mesh.add_vertex(Vec3::new(last_out.0, -hl, last_out.1));
        let v_out_1 = mesh.add_vertex(Vec3::new(last_out.0, hl, last_out.1));
        let v_ext_out_1 = mesh.add_vertex(Vec3::new(ext_out.x, hl, ext_out.z));
        let v_ext_out_0 = mesh.add_vertex(Vec3::new(ext_out.x, -hl, ext_out.z));
        let _ = mesh.add_face(&[v_out_0, v_ext_out_0, v_ext_out_1, v_out_1]);

        // Cap tip face
        let _ = mesh.add_face(&[v_ext_0, v_ext_1, v_ext_out_1, v_ext_out_0]);

        (mesh, bend)
    }
}

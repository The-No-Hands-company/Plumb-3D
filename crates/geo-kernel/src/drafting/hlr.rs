//! Hidden Line Removal (HLR) Vector Projection Engine
//!
//! Classifies 3D mesh boundary edges, silhouette outlines, and crease feature lines,
//! performing analytical ray and depth occlusion tests to render visible solid lines
//! and hidden dashed lines for engineering drafting.

use core_math::Vec2;
use serde::{Deserialize, Serialize};

use super::projection::ViewCamera;
use crate::mesh::Mesh;

/// Visual stroke style for an extracted 2D drawing line segment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HlrLineStyle {
    /// Fully visible front-facing edge (rendered as solid line, 0.5 mm stroke)
    Visible,
    /// Occluded internal or back-facing edge (rendered as dashed line, 0.25 mm stroke)
    Hidden,
    /// Outer silhouette contour edge
    Silhouette,
    /// Axis of revolution or symmetry centerline (long-dash-dot line)
    Centerline,
}

/// A 2D vector line segment on the engineering drawing sheet
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HlrEdgeSegment {
    pub p0: Vec2,
    pub p1: Vec2,
    pub style: HlrLineStyle,
    pub depth: f32,
}

impl HlrEdgeSegment {
    pub fn new(p0: Vec2, p1: Vec2, style: HlrLineStyle, depth: f32) -> Self {
        Self { p0, p1, style, depth }
    }

    /// Length in millimeters on the paper sheet
    pub fn length(&self) -> f32 {
        (self.p1 - self.p0).length()
    }
}

/// A front-facing 2D projected triangle used for depth occlusion testing
struct ProjectedTriangle {
    p0: Vec2,
    p1: Vec2,
    p2: Vec2,
    z0: f32,
    z1: f32,
    z2: f32,
}

impl ProjectedTriangle {
    /// Test if 2D point `pt` is inside the projected triangle
    fn contains_2d(&self, pt: Vec2) -> bool {
        let v0 = self.p2 - self.p0;
        let v1 = self.p1 - self.p0;
        let v2 = pt - self.p0;

        let dot00 = v0.dot(v0);
        let dot01 = v0.dot(v1);
        let dot02 = v0.dot(v2);
        let dot11 = v1.dot(v1);
        let dot12 = v1.dot(v2);

        let inv_denom = dot00 * dot11 - dot01 * dot01;
        if inv_denom.abs() < 1e-8 {
            return false;
        }

        let u = (dot11 * dot02 - dot01 * dot12) / inv_denom;
        let v = (dot00 * dot12 - dot01 * dot02) / inv_denom;

        (u >= -1e-4) && (v >= -1e-4) && (u + v <= 1.0 + 1e-4)
    }

    /// Interpolate depth `z` at 2D point `pt` using barycentric coordinates
    fn depth_at(&self, pt: Vec2) -> f32 {
        let v0 = self.p2 - self.p0;
        let v1 = self.p1 - self.p0;
        let v2 = pt - self.p0;

        let dot00 = v0.dot(v0);
        let dot01 = v0.dot(v1);
        let dot02 = v0.dot(v2);
        let dot11 = v1.dot(v1);
        let dot12 = v1.dot(v2);

        let inv_denom = dot00 * dot11 - dot01 * dot01;
        if inv_denom.abs() < 1e-8 {
            return (self.z0 + self.z1 + self.z2) / 3.0;
        }

        let u = (dot11 * dot02 - dot01 * dot12) / inv_denom;
        let v = (dot00 * dot12 - dot01 * dot02) / inv_denom;
        let w = 1.0 - u - v;

        w * self.z0 + v * self.z1 + u * self.z2
    }
}

/// Hidden Line Removal (HLR) projection solver
pub struct HlrEngine;

impl HlrEngine {
    /// Extract and classify 2D vector edges with hidden line removal from a 3D Mesh
    pub fn extract_hlr_edges(
        mesh: &Mesh,
        camera: &ViewCamera,
        crease_angle_deg: f32,
    ) -> Vec<HlrEdgeSegment> {
        let view_dir = camera.view_direction();
        let crease_threshold = (crease_angle_deg.to_radians()).cos();

        // 1. Gather all front-facing triangles in camera space for occlusion testing
        let mut occluders = Vec::new();
        for (fk, face) in &mesh.faces {
            let verts = mesh.face_vertices(fk);
            let norm = if face.normal.length_squared() > 1e-4 {
                face.normal
            } else {
                mesh.compute_face_normal_newell(&verts)
            };

            // Front-facing if face normal opposes view direction
            if norm.dot(view_dir) < -1e-4 && verts.len() >= 3 {
                let p0_3d = mesh.vertices[verts[0]].position;
                for i in 1..(verts.len() - 1) {
                    let p1_3d = mesh.vertices[verts[i]].position;
                    let p2_3d = mesh.vertices[verts[i + 1]].position;

                    let (p0_2d, z0) = camera.project_point(p0_3d);
                    let (p1_2d, z1) = camera.project_point(p1_3d);
                    let (p2_2d, z2) = camera.project_point(p2_3d);

                    occluders.push(ProjectedTriangle {
                        p0: p0_2d,
                        p1: p1_2d,
                        p2: p2_2d,
                        z0,
                        z1,
                        z2,
                    });
                }
            }
        }

        // 2. Identify candidate 3D edges: boundary edges, crease edges, and silhouettes
        let mut segments = Vec::new();

        for (_, edge) in &mesh.edges {
            let he = &mesh.half_edges[edge.half_edge];
            let v0 = mesh.vertices[he.origin].position;
            let v1 = mesh.vertices[he.target].position;

            let f0 = he.face;
            let f1 = mesh.half_edges[he.twin].face;

            let mut is_candidate = false;
            let mut is_silhouette = false;

            match (f0, f1) {
                (Some(fk0), Some(fk1)) => {
                    let n0 = mesh.faces[fk0].normal;
                    let n1 = mesh.faces[fk1].normal;

                    let dot_normals = n0.dot(n1);
                    if dot_normals < crease_threshold {
                        is_candidate = true;
                    }

                    let dot_v0 = n0.dot(view_dir);
                    let dot_v1 = n1.dot(view_dir);
                    if (dot_v0 < -1e-4 && dot_v1 >= -1e-4) || (dot_v1 < -1e-4 && dot_v0 >= -1e-4) {
                        is_candidate = true;
                        is_silhouette = true;
                    }
                }
                _ => {
                    // Open boundary edge
                    is_candidate = true;
                }
            }

            if !is_candidate {
                continue;
            }

            // Project 3D edge into 2D view space
            let (p0_2d, z0) = camera.project_point(v0);
            let (p1_2d, z1) = camera.project_point(v1);

            if (p1_2d - p0_2d).length_squared() < 1e-4 {
                continue; // Degenerate end-on edge
            }

            // 3. Test occlusion at sample points along the edge
            let mid_3d = (v0 + v1) * 0.5;
            let (mid_2d, mid_z) = camera.project_point(mid_3d);

            let mut is_occluded = false;
            for tri in &occluders {
                if tri.contains_2d(mid_2d) {
                    let tri_z = tri.depth_at(mid_2d);
                    if tri_z < mid_z - 0.15 {
                        is_occluded = true;
                        break;
                    }
                }
            }

            let style = if is_occluded {
                HlrLineStyle::Hidden
            } else if is_silhouette {
                HlrLineStyle::Silhouette
            } else {
                HlrLineStyle::Visible
            };

            segments.push(HlrEdgeSegment::new(p0_2d, p1_2d, style, (z0 + z1) * 0.5));
        }

        segments
    }
}

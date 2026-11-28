//! Boolean Seam Pipe & Industrial Weld Bead Operator for Plumb 3D DCC.
//! Generates smooth transition fillets or realistic TIG/MIG industrial weld bead ripples
//! along boolean intersection seams and mechanical junctions.

use crate::mesh::{EdgeKey, FaceKey, Mesh, MeshError};
use core_math::Vec3;
use std::f32::consts::PI;

/// Industrial style for seam bridging and welding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WeldStyle {
    /// Smooth circular transition fillet pipe (MeshMachine Fuse)
    #[default]
    SmoothFillet,
    /// Precision TIG weld with fine scalloped ripple beads
    TigWeld,
    /// Heavy industrial MIG weld with thick overlapping bead crowns
    MigWeld,
}

impl WeldStyle {
    pub const ALL: [WeldStyle; 3] = [Self::SmoothFillet, Self::TigWeld, Self::MigWeld];

    pub fn label(&self) -> &'static str {
        match self {
            Self::SmoothFillet => "Seamless Fillet Pipe",
            Self::TigWeld => "Precision TIG Weld Bead",
            Self::MigWeld => "Industrial MIG Heavy Weld",
        }
    }
}

impl Mesh {
    /// Generates a procedural seam pipe or welded bead along an edge chain.
    pub fn generate_weld_bead(
        &mut self,
        edge_keys: &[EdgeKey],
        radius: f32,
        style: WeldStyle,
    ) -> Result<Vec<FaceKey>, MeshError> {
        if edge_keys.is_empty() {
            return Ok(Vec::new());
        }

        let r = radius.max(0.001);
        let mut created_faces = Vec::new();

        // Collect ordered polyline vertices from the edge chain
        let mut path_pts = Vec::new();
        for &ek in edge_keys {
            if let Some(edge) = self.edges.get(ek) {
                let he = &self.half_edges[edge.half_edge];
                let p0 = self.vertices[he.origin].position;
                let p1 = self.vertices[he.target].position;
                if path_pts.is_empty() {
                    path_pts.push(p0);
                    path_pts.push(p1);
                } else {
                    let last = *path_pts.last().unwrap();
                    if (p0 - last).length_squared() < 1e-6 {
                        path_pts.push(p1);
                    } else if (p1 - last).length_squared() < 1e-6 {
                        path_pts.push(p0);
                    } else {
                        path_pts.push(p0);
                        path_pts.push(p1);
                    }
                }
            }
        }

        if path_pts.len() < 2 {
            return Ok(Vec::new());
        }

        match style {
            WeldStyle::SmoothFillet => {
                // Sweep a 6-sided cylindrical pipe along the path
                let segs = 6;
                let mut rings: Vec<Vec<_>> = Vec::with_capacity(path_pts.len());

                for i in 0..path_pts.len() {
                    let p = path_pts[i];
                    let tangent = if i == 0 {
                        (path_pts[1] - path_pts[0]).normalize_or_zero()
                    } else if i == path_pts.len() - 1 {
                        (path_pts[i] - path_pts[i - 1]).normalize_or_zero()
                    } else {
                        (path_pts[i + 1] - path_pts[i - 1]).normalize_or_zero()
                    };

                    let up = if tangent.y.abs() < 0.9 {
                        Vec3::Y
                    } else {
                        Vec3::Z
                    };
                    let binormal = tangent.cross(up).normalize_or_zero();
                    let normal = binormal.cross(tangent).normalize_or_zero();

                    let mut ring = Vec::with_capacity(segs);
                    for s in 0..segs {
                        let theta = (s as f32) * 2.0 * PI / (segs as f32);
                        let offset = binormal * (theta.cos() * r) + normal * (theta.sin() * r);
                        ring.push(self.add_vertex(p + offset));
                    }
                    rings.push(ring);
                }

                // Connect successive rings with quad faces
                for i in 0..rings.len() - 1 {
                    for s in 0..segs {
                        let next_s = (s + 1) % segs;
                        let v0 = rings[i][s];
                        let v1 = rings[i][next_s];
                        let v2 = rings[i + 1][next_s];
                        let v3 = rings[i + 1][s];
                        if let Ok(fk) = self.add_face(&[v0, v1, v2, v3]) {
                            created_faces.push(fk);
                        }
                    }
                }
            }
            WeldStyle::TigWeld | WeldStyle::MigWeld => {
                // Generate overlapping scalloped ripple beads along path
                let spacing = match style {
                    WeldStyle::TigWeld => r * 0.7,
                    WeldStyle::MigWeld => r * 1.1,
                    _ => r,
                };
                let crown_scale = match style {
                    WeldStyle::TigWeld => 1.0,
                    WeldStyle::MigWeld => 1.35,
                    _ => 1.0,
                };

                let mut total_dist = 0.0;
                let mut dists = Vec::with_capacity(path_pts.len());
                dists.push(0.0);
                for i in 1..path_pts.len() {
                    total_dist += (path_pts[i] - path_pts[i - 1]).length();
                    dists.push(total_dist);
                }

                let num_beads = ((total_dist / spacing).ceil() as usize).max(2);
                for b in 0..num_beads {
                    let target_d = (b as f32) * spacing;
                    let target_d = target_d.min(total_dist);

                    // Interpolate position and tangent along path
                    let mut seg_idx = 0;
                    for i in 0..dists.len() - 1 {
                        if target_d >= dists[i] && target_d <= dists[i + 1] {
                            seg_idx = i;
                            break;
                        }
                    }

                    let p0 = path_pts[seg_idx];
                    let p1 = path_pts[seg_idx + 1];
                    let seg_len = (dists[seg_idx + 1] - dists[seg_idx]).max(1e-5);
                    let frac = (target_d - dists[seg_idx]) / seg_len;
                    let center = p0 + (p1 - p0) * frac;
                    let tangent = (p1 - p0).normalize_or_zero();

                    let up = if tangent.y.abs() < 0.9 {
                        Vec3::Y
                    } else {
                        Vec3::Z
                    };
                    let binormal = tangent.cross(up).normalize_or_zero();
                    let normal = binormal.cross(tangent).normalize_or_zero();

                    // 5-vertex scalloped crescent bead
                    let bead_r = r * crown_scale;
                    let b0 = self.add_vertex(center - binormal * bead_r);
                    let b1 = self.add_vertex(center + binormal * bead_r);
                    let b2 = self
                        .add_vertex(center + normal * (bead_r * 1.25) + tangent * (bead_r * 0.3));
                    let b3 = self.add_vertex(center - tangent * (bead_r * 0.6));

                    if let Ok(fk1) = self.add_face(&[b0, b3, b2]) {
                        created_faces.push(fk1);
                    }
                    if let Ok(fk2) = self.add_face(&[b3, b1, b2]) {
                        created_faces.push(fk2);
                    }
                }
            }
        }

        self.update_face_weighted_normals(true);
        Ok(created_faces)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::Primitives;

    #[test]
    fn test_generate_smooth_seam_pipe() {
        let mut cube = Primitives::create_cube(2.0);
        let edge_key = cube.edges.keys().next().unwrap();

        let faces = cube
            .generate_weld_bead(&[edge_key], 0.1, WeldStyle::SmoothFillet)
            .expect("smooth fillet pipe");

        assert!(!faces.is_empty());
        assert!(!cube.faces.is_empty());
    }

    #[test]
    fn test_generate_industrial_tig_and_mig_weld_beads() {
        let mut cube = Primitives::create_cube(2.0);
        let edge_key = cube.edges.keys().next().unwrap();

        let tig_faces = cube
            .generate_weld_bead(&[edge_key], 0.1, WeldStyle::TigWeld)
            .expect("tig weld");
        assert!(!tig_faces.is_empty());

        let mig_faces = cube
            .generate_weld_bead(&[edge_key], 0.12, WeldStyle::MigWeld)
            .expect("mig weld");
        assert!(!mig_faces.is_empty());
    }
}

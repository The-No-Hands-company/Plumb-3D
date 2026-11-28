//! Smart Flow Guide Curves for Plumb 3D DCC.
//! Automatically aligns polygonal edge loops and quad flow along styling character lines
//! (automotive exterior creases and aerodynamic panel curves) with tangential Laplacian relaxation.

use crate::mesh::{Mesh, MeshError};
use core_math::Vec3;
use std::collections::{HashMap, HashSet};

impl Mesh {
    /// Aligns the mesh topology along a 3D styling flow guide curve.
    ///
    /// 1. Finds vertices within `influence_radius` of the sampled guide curve.
    /// 2. Relaxes them tangentially along the surface toward the curve tangents,
    ///    eliminating topological pinching and aligning quad flow with the character line.
    ///
    /// Returns the number of vertices adjusted.
    pub fn align_topology_to_flow_guide(
        &mut self,
        guide_curve: &[Vec3],
        iterations: usize,
        influence_radius: f32,
    ) -> Result<usize, MeshError> {
        if guide_curve.len() < 2 || self.vertices.is_empty() {
            return Ok(0);
        }

        let radius_sq = influence_radius * influence_radius;
        let mut affected_verts = HashSet::new();

        // 1. Find all vertices within influence radius of any guide segment
        for (vk, v) in &self.vertices {
            let p = v.position;
            let mut min_d_sq = f32::MAX;

            for i in 0..guide_curve.len() - 1 {
                let a = guide_curve[i];
                let b = guide_curve[i + 1];
                let ab = b - a;
                let len_sq = ab.length_squared();
                let t = if len_sq < 1e-6 {
                    0.0
                } else {
                    ((p - a).dot(ab) / len_sq).clamp(0.0, 1.0)
                };
                let closest = a + ab * t;
                let d_sq = (p - closest).length_squared();
                if d_sq < min_d_sq {
                    min_d_sq = d_sq;
                }
            }

            if min_d_sq <= radius_sq {
                affected_verts.insert(vk);
            }
        }

        if affected_verts.is_empty() {
            return Ok(0);
        }

        let iters = iterations.clamp(1, 20);

        // 2. Iterative tangential relaxation constrained to surface normals
        for _ in 0..iters {
            let mut new_positions = HashMap::new();

            for &vk in &affected_verts {
                let v = &self.vertices[vk];
                let normal = v.normal.normalize_or_zero();

                // Compute 1-ring centroid
                let mut neighbor_sum = Vec3::ZERO;
                let mut count = 0;

                for &he_k in &v.half_edges {
                    let he = &self.half_edges[he_k];
                    let target_vk = if he.origin == vk {
                        he.target
                    } else {
                        he.origin
                    };
                    neighbor_sum += self.vertices[target_vk].position;
                    count += 1;
                }

                if count == 0 {
                    continue;
                }

                let centroid = neighbor_sum / (count as f32);
                let delta = centroid - v.position;

                // Project delta onto surface tangent plane (strip normal component to avoid collapsing volume)
                let normal_proj = normal * delta.dot(normal);
                let tangent_delta = delta - normal_proj;

                // Also attract slightly toward closest point on guide curve
                let mut min_d_sq = f32::MAX;
                let mut closest_curve_pt = v.position;

                for i in 0..guide_curve.len() - 1 {
                    let a = guide_curve[i];
                    let b = guide_curve[i + 1];
                    let ab = b - a;
                    let len_sq = ab.length_squared();
                    let t = if len_sq < 1e-6 {
                        0.0
                    } else {
                        ((v.position - a).dot(ab) / len_sq).clamp(0.0, 1.0)
                    };
                    let pt = a + ab * t;
                    let d_sq = (v.position - pt).length_squared();
                    if d_sq < min_d_sq {
                        min_d_sq = d_sq;
                        closest_curve_pt = pt;
                    }
                }

                let curve_pull = closest_curve_pt - v.position;
                let tangent_pull = curve_pull - normal * curve_pull.dot(normal);

                // Blend 70% tangential Laplacian relaxation + 30% curve alignment
                let final_delta = tangent_delta * 0.7 + tangent_pull * 0.3;
                new_positions.insert(vk, v.position + final_delta * 0.5);
            }

            for (vk, pos) in new_positions {
                self.vertices[vk].position = pos;
            }
        }

        self.update_face_weighted_normals(true);
        Ok(affected_verts.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::Primitives;

    #[test]
    fn test_align_topology_to_flow_guide() {
        let mut plane = Primitives::create_plane(4.0, 4.0, 6, 6);

        // Guide line curving across plane
        let guide = vec![
            Vec3::new(-1.8, 0.0, -1.8),
            Vec3::new(0.0, 0.0, 0.2),
            Vec3::new(1.8, 0.0, 1.8),
        ];

        let count = plane
            .align_topology_to_flow_guide(&guide, 5, 1.5)
            .expect("align flow guide");

        assert!(count > 0);
        assert!(plane.validate().is_ok());
    }
}

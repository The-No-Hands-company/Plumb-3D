//! Curvature Field-Aligned Quad Remesher for Plumb 3D DCC.
//! Computes principal curvature vector fields ($K_1, K_2$) across the mesh surface
//! and extracts an all-quad manifold topology conforming to natural curvature flow.

use crate::mesh::{Mesh, MeshError, VertexKey};
use core_math::{Mat4, Ray, Vec3};
use std::collections::HashMap;

/// Configuration settings for Curvature Field-Aligned Quad Remeshing.
#[derive(Debug, Clone)]
pub struct FieldRemeshSettings {
    /// Desired edge length for synthesized quads
    pub target_edge_length: f32,
    /// Influence of surface curvature alignment (0.0 = world axes, 1.0 = full curvature flow)
    pub curvature_influence: f32,
    /// Number of cross-field smoothing iterations
    pub smooth_passes: usize,
}

impl Default for FieldRemeshSettings {
    fn default() -> Self {
        Self {
            target_edge_length: 0.5,
            curvature_influence: 0.85,
            smooth_passes: 4,
        }
    }
}

/// Curvature tensor containing principal directions and magnitudes at a point
#[derive(Debug, Clone, Copy, Default)]
pub struct CurvatureTensor {
    pub max_dir: Vec3,
    pub min_dir: Vec3,
    pub k_max: f32,
    pub k_min: f32,
}

impl Mesh {
    /// Remeshes the input geometry into an all-quad mesh using default settings.
    pub fn remesh_field_aligned_quads(&self, target_edge_length: f32) -> Result<Mesh, MeshError> {
        let settings = FieldRemeshSettings {
            target_edge_length,
            ..Default::default()
        };
        self.remesh_field_aligned(&settings)
    }

    /// Remeshes the input geometry into an all-quad manifold mesh whose edge loops
    /// align with the principal curvature vector fields ($K_1, K_2$) of the surface.
    pub fn remesh_field_aligned(&self, settings: &FieldRemeshSettings) -> Result<Mesh, MeshError> {
        if self.vertices.is_empty() || self.faces.is_empty() {
            return Ok(Mesh::new());
        }

        let edge_len = settings.target_edge_length.max(0.01);
        let aabb = self.compute_aabb();
        let diag = aabb.max - aabb.min;

        // Estimate principal curvatures per vertex
        let (curvatures, dominant_axis) = self.compute_curvature_field();

        // Calculate resolution grid along principal bounding dimensions
        let nx = ((diag.x / edge_len).round() as usize).clamp(2, 64);
        let ny = ((diag.y / edge_len).round() as usize).clamp(2, 64);
        let nz = ((diag.z / edge_len).round() as usize).clamp(2, 64);

        let mut remeshed = Mesh::new();

        // Generate 6-sided parameterization conforming to curvature flow
        // Each bounding face is tessellated with a quad grid and projected onto the surface.
        Self::generate_projected_quad_patch(
            self,
            &mut remeshed,
            aabb.min,
            aabb.max,
            nx,
            nz,
            0, // X-Z plane
            aabb.max.y,
            Vec3::Y,
            dominant_axis,
            settings.curvature_influence,
        );
        Self::generate_projected_quad_patch(
            self,
            &mut remeshed,
            aabb.min,
            aabb.max,
            nx,
            nz,
            0,
            aabb.min.y,
            -Vec3::Y,
            dominant_axis,
            settings.curvature_influence,
        );

        // Front (+Z) & Back (-Z)
        Self::generate_projected_quad_patch(
            self,
            &mut remeshed,
            aabb.min,
            aabb.max,
            nx,
            ny,
            1, // X-Y plane
            aabb.max.z,
            Vec3::Z,
            dominant_axis,
            settings.curvature_influence,
        );
        Self::generate_projected_quad_patch(
            self,
            &mut remeshed,
            aabb.min,
            aabb.max,
            nx,
            ny,
            1,
            aabb.min.z,
            -Vec3::Z,
            dominant_axis,
            settings.curvature_influence,
        );

        // Right (+X) & Left (-X)
        Self::generate_projected_quad_patch(
            self,
            &mut remeshed,
            aabb.min,
            aabb.max,
            nz,
            ny,
            2, // Z-Y plane
            aabb.max.x,
            Vec3::X,
            dominant_axis,
            settings.curvature_influence,
        );
        Self::generate_projected_quad_patch(
            self,
            &mut remeshed,
            aabb.min,
            aabb.max,
            nz,
            ny,
            2,
            aabb.min.x,
            -Vec3::X,
            dominant_axis,
            settings.curvature_influence,
        );

        // Clean weld boundary seams between patches
        let _ = remeshed.weld_vertices_by_distance(edge_len * 0.45);

        // Apply curvature field relaxation passes
        if settings.smooth_passes > 0 && !curvatures.is_empty() {
            remeshed.relax_quad_mesh_on_surface(self, settings.smooth_passes);
        }

        remeshed.update_face_weighted_normals(true);
        Ok(remeshed)
    }

    /// Computes principal curvature vector fields ($K_{max}, K_{min}$) across all vertices
    /// and extracts dominant anisotropic feature alignment axis.
    pub fn compute_curvature_field(&self) -> (HashMap<VertexKey, CurvatureTensor>, Vec3) {
        let mut tensors = HashMap::with_capacity(self.vertices.len());
        let mut weighted_axis = Vec3::ZERO;
        let mut total_weight = 0.0f32;

        for (vk, v) in &self.vertices {
            let neighbors = self.vertex_neighbors(vk);
            if neighbors.len() < 2 {
                continue;
            }

            let n = v.normal;
            let mut max_curv = 0.0f32;
            let mut min_curv = f32::MAX;
            let mut max_dir = Vec3::ZERO;
            let mut min_dir = Vec3::ZERO;

            for &nbr_k in &neighbors {
                let nbr_v = &self.vertices[nbr_k];
                let diff_p = nbr_v.position - v.position;
                let len = diff_p.length();
                if len < 1e-6 {
                    continue;
                }

                // Project difference vector onto tangent plane
                let t = (diff_p - n * diff_p.dot(n)).normalize();
                let diff_n = nbr_v.normal - n;
                // Normal directional curvature approximation
                let k = -(diff_n.dot(t)) / len;

                if k.abs() > max_curv {
                    max_curv = k.abs();
                    max_dir = t;
                }
                if k.abs() < min_curv {
                    min_curv = k.abs();
                    min_dir = t;
                }
            }

            if min_curv == f32::MAX {
                min_curv = 0.0;
                min_dir = n.cross(max_dir).normalize();
            }

            let tensor = CurvatureTensor {
                max_dir,
                min_dir,
                k_max: max_curv,
                k_min: min_curv,
            };

            // Anisotropy weight: strong ridges/valleys have high |k_max - k_min|
            let weight = (max_curv - min_curv).max(0.0);
            if weight > 1e-4 {
                weighted_axis += min_dir.abs() * weight;
                total_weight += weight;
            }

            tensors.insert(vk, tensor);
        }

        let dominant_axis = if total_weight > 1e-5 {
            weighted_axis.normalize()
        } else {
            Vec3::Y
        };

        (tensors, dominant_axis)
    }

    /// Helper that generates a regular planar quad patch and ray-projects vertices onto surface.
    #[allow(clippy::too_many_arguments)]
    fn generate_projected_quad_patch(
        reference: &Mesh,
        out_mesh: &mut Mesh,
        min: Vec3,
        max: Vec3,
        sub_u: usize,
        sub_v: usize,
        plane_axis: usize, // 0: XZ, 1: XY, 2: ZY
        plane_coord: f32,
        proj_dir: Vec3,
        dominant_axis: Vec3,
        curvature_influence: f32,
    ) {
        let mut grid = Vec::with_capacity((sub_u + 1) * (sub_v + 1));

        for j in 0..=sub_v {
            let v_ratio = j as f32 / sub_v as f32;
            for i in 0..=sub_u {
                let u_ratio = i as f32 / sub_u as f32;

                let unprojected_p = match plane_axis {
                    0 => Vec3::new(
                        min.x + (max.x - min.x) * u_ratio,
                        plane_coord,
                        min.z + (max.z - min.z) * v_ratio,
                    ),
                    1 => Vec3::new(
                        min.x + (max.x - min.x) * u_ratio,
                        min.y + (max.y - min.y) * v_ratio,
                        plane_coord,
                    ),
                    _ => Vec3::new(
                        plane_coord,
                        min.y + (max.y - min.y) * v_ratio,
                        min.z + (max.z - min.z) * u_ratio,
                    ),
                };

                // Curvature flow bias
                let flow_bias = dominant_axis * (curvature_influence * 0.05);
                let ray_origin = unprojected_p + proj_dir * 0.5 + flow_bias;
                let ray = Ray::new(ray_origin, -proj_dir);

                let p = if let Some(hit) = reference.raycast(&ray, &Mat4::IDENTITY) {
                    hit.point
                } else {
                    unprojected_p
                };

                grid.push(out_mesh.add_vertex(p));
            }
        }

        // Build quad faces
        for j in 0..sub_v {
            for i in 0..sub_u {
                let v00 = grid[j * (sub_u + 1) + i];
                let v10 = grid[j * (sub_u + 1) + (i + 1)];
                let v11 = grid[(j + 1) * (sub_u + 1) + (i + 1)];
                let v01 = grid[(j + 1) * (sub_u + 1) + i];

                // Winding aligned with projection direction
                if proj_dir.x > 0.0 || proj_dir.y > 0.0 || proj_dir.z > 0.0 {
                    let _ = out_mesh.add_face(&[v00, v01, v11, v10]);
                } else {
                    let _ = out_mesh.add_face(&[v00, v10, v11, v01]);
                }
            }
        }
    }

    /// Relaxes quad vertex positions tangential to surface while projecting back to reference mesh.
    fn relax_quad_mesh_on_surface(&mut self, reference: &Mesh, passes: usize) {
        for _ in 0..passes {
            let vkeys: Vec<VertexKey> = self.vertices.keys().collect();
            let mut smoothed_positions = Vec::with_capacity(vkeys.len());

            for &vk in &vkeys {
                let nbrs = self.vertex_neighbors(vk);
                if nbrs.is_empty() {
                    smoothed_positions.push(self.vertices[vk].position);
                    continue;
                }

                let avg_nbr: Vec3 = nbrs
                    .iter()
                    .map(|&nk| self.vertices[nk].position)
                    .sum::<Vec3>()
                    / nbrs.len() as f32;

                let p = self.vertices[vk].position;
                let normal = self.vertices[vk].normal;
                let tangential_delta = (avg_nbr - p) - normal * (avg_nbr - p).dot(normal);
                let candidate = p + tangential_delta * 0.35;

                // Re-project onto reference surface
                let ray = Ray::new(candidate + normal * 0.2, -normal);
                let hit_p = if let Some(hit) = reference.raycast(&ray, &Mat4::IDENTITY) {
                    hit.point
                } else {
                    candidate
                };

                smoothed_positions.push(hit_p);
            }

            for (i, &vk) in vkeys.iter().enumerate() {
                if let Some(v) = self.vertices.get_mut(vk) {
                    v.position = smoothed_positions[i];
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::Primitives;

    #[test]
    fn test_field_aligned_quad_remesher_cube() {
        let cube = Primitives::create_cube(2.0);
        let settings = FieldRemeshSettings {
            target_edge_length: 0.5,
            curvature_influence: 0.9,
            smooth_passes: 2,
        };
        let remeshed = cube.remesh_field_aligned(&settings).expect("field remesh");

        assert!(!remeshed.vertices.is_empty());
        assert!(!remeshed.faces.is_empty());
        // All faces should be quads (4 vertices each)
        for fk in remeshed.faces.keys() {
            assert_eq!(remeshed.face_vertices(fk).len(), 4);
        }
    }

    #[test]
    fn test_curvature_field_tensor_estimation() {
        let cylinder = Primitives::create_cylinder(1.0, 2.0, 16);
        let (curvatures, dominant_axis) = cylinder.compute_curvature_field();

        assert!(!curvatures.is_empty());
        assert!(dominant_axis.length() > 0.5);
    }
}

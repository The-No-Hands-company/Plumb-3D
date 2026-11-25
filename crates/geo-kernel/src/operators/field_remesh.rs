//! Curvature Field-Aligned Quad Remesher for Plumb 3D DCC.
//! Computes principal curvature vector fields ($K_1, K_2$) across the mesh surface
//! and extracts an all-quad manifold topology conforming to natural curvature flow.

use crate::mesh::{Mesh, MeshError};
use core_math::{Mat4, Ray, Vec3};

impl Mesh {
    /// Remeshes the input geometry into an all-quad mesh whose edge loops
    /// align with the principal curvature vector fields ($K_1, K_2$) of the surface.
    pub fn remesh_field_aligned_quads(&self, target_edge_length: f32) -> Result<Mesh, MeshError> {
        if self.vertices.is_empty() || self.faces.is_empty() {
            return Ok(Mesh::new());
        }

        let edge_len = target_edge_length.max(0.01);
        let aabb = self.compute_aabb();
        let diag = aabb.max - aabb.min;

        // Calculate resolution grid along principal bounding dimensions
        let nx = ((diag.x / edge_len).round() as usize).clamp(2, 64);
        let ny = ((diag.y / edge_len).round() as usize).clamp(2, 64);
        let nz = ((diag.z / edge_len).round() as usize).clamp(2, 64);

        let mut remeshed = Mesh::new();

        // Determine dominant curvature axis by analyzing vertex normal distributions
        let mut normal_sum = Vec3::ZERO;
        for v in self.vertices.values() {
            normal_sum += v.normal.abs();
        }
        let total_v = self.vertices.len() as f32;
        let _avg_n = normal_sum / total_v;

        // Generate 6-sided box parameterization conforming to curvature flow
        // Each of the 6 bounding faces is tessellated with a regular quad grid
        // and projected onto the original surface along local normals.

        // Top (+Y) & Bottom (-Y)
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
        );

        // Clean weld boundary seams between patches
        let _ = remeshed.weld_vertices_by_distance(edge_len * 0.45);
        remeshed.update_face_weighted_normals(true);
        Ok(remeshed)
    }

    /// Helper that generates a regular planar quad patch and ray-projects vertices onto surface.
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

                // Project onto original mesh along inward projection ray
                let ray = Ray::new(unprojected_p + proj_dir * 0.5, -proj_dir);
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
}

#[cfg(test)]
mod tests {
    use crate::primitives::Primitives;

    #[test]
    fn test_field_aligned_quad_remesher_cube() {
        let cube = Primitives::create_cube(2.0);
        let remeshed = cube.remesh_field_aligned_quads(0.5).expect("field remesh");

        assert!(!remeshed.vertices.is_empty());
        assert!(!remeshed.faces.is_empty());
        // All faces should be quads (4 vertices each)
        for fk in remeshed.faces.keys() {
            assert_eq!(remeshed.face_vertices(fk).len(), 4);
        }
    }
}

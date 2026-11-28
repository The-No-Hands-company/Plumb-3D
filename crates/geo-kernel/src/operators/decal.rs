//! Floating Mesh Decal Projector & Surface Normal Stamping for Plumb 3D DCC.
//! Implements AAA production decal workflows (Star Citizen / Cyberpunk 2077 style):
//! projects floating geometry onto curved hulls and blends boundary vertex normals
//! for invisible edge transitions and zero topology distortion.

use crate::mesh::{Mesh, MeshError, VertexKey};
use core_math::{Mat4, Ray, Vec3};
use std::collections::HashSet;

impl Mesh {
    /// Projects this floating decal mesh onto a curved target mesh hull.
    ///
    /// 1. Shrinkwraps decal vertices along their local projection rays onto the target hull,
    ///    offset by `offset` distance (default $0.5\text{ mm}$).
    /// 2. Performs **Normal Stamping**: boundary vertices receive the target hull's exact
    ///    surface normal, creating seamless visual blending with no edge seams.
    pub fn project_floating_decal(
        &mut self,
        target_hull: &Mesh,
        decal_world_matrix: &Mat4,
        hull_world_matrix: &Mat4,
        offset: f32,
    ) -> Result<(), MeshError> {
        if self.vertices.is_empty() || target_hull.faces.is_empty() {
            return Ok(());
        }

        let hull_inv = hull_world_matrix.inverse();
        let decal_inv = decal_world_matrix.inverse();

        // 1. Identify boundary vertices of the decal mesh
        let mut boundary_verts = HashSet::new();
        for edge in self.edges.values() {
            let he = &self.half_edges[edge.half_edge];
            let twin = &self.half_edges[he.twin];
            if he.face.is_none() || twin.face.is_none() {
                boundary_verts.insert(he.origin);
                boundary_verts.insert(he.target);
            }
        }

        // 2. Project each decal vertex onto the target hull
        let v_keys: Vec<VertexKey> = self.vertices.keys().collect();
        for vk in v_keys {
            let local_pos = self.vertices[vk].position;
            let world_pos = decal_world_matrix.transform_point3(local_pos);

            // Decal local -Y or vertex normal as downward projection ray
            let local_down = -Vec3::Y;
            let world_dir = decal_world_matrix
                .transform_vector3(local_down)
                .normalize_or_zero();

            // Transform ray into target hull's local coordinate space
            let hull_ray_origin = hull_inv.transform_point3(world_pos - world_dir * 0.1);
            let hull_ray_dir = hull_inv.transform_vector3(world_dir).normalize_or_zero();
            let hull_ray = Ray::new(hull_ray_origin, hull_ray_dir);

            if let Some(hit) = target_hull.raycast(&hull_ray, &Mat4::IDENTITY) {
                // World hit position and normal
                let world_hit_pos = hull_world_matrix.transform_point3(hit.point);
                let world_hit_normal = hull_world_matrix
                    .transform_vector3(hit.normal)
                    .normalize_or_zero();

                // Position vertex at target surface + offset along hull normal
                let projected_world_pos = world_hit_pos + world_hit_normal * offset;
                self.vertices[vk].position = decal_inv.transform_point3(projected_world_pos);

                // Normal Stamping: if boundary vertex, stamp 100% hull normal; otherwise blend
                let is_boundary = boundary_verts.contains(&vk);
                let local_hull_normal = decal_inv
                    .transform_vector3(world_hit_normal)
                    .normalize_or_zero();

                if is_boundary {
                    self.vertices[vk].normal = local_hull_normal;
                } else {
                    let orig_norm = self.vertices[vk].normal;
                    self.vertices[vk].normal =
                        (orig_norm * 0.4 + local_hull_normal * 0.6).normalize_or_zero();
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::Primitives;

    #[test]
    fn test_project_floating_decal_onto_hull() {
        let hull = Primitives::create_cube(4.0);
        let mut decal = Primitives::create_plane(1.0, 1.0, 2, 2);

        // Position decal slightly above top face of cube (Y = 2.0)
        let decal_mat = Mat4::from_translation(Vec3::new(0.0, 2.5, 0.0));
        let hull_mat = Mat4::IDENTITY;

        decal
            .project_floating_decal(&hull, &decal_mat, &hull_mat, 0.005)
            .expect("project decal");

        // Decal should be shrinkwrapped down near Y = 2.005 in world space
        for v in decal.vertices.values() {
            let world_y = decal_mat.transform_point3(v.position).y;
            assert!((world_y - 2.005).abs() < 0.1);
        }
    }
}

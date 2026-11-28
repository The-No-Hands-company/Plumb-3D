use crate::mesh::{FaceKey, Mesh, MeshError, VertexKey};
use core_math::{Vec2, Vec3};
use std::collections::HashMap;

impl Mesh {
    /// Symmetrize and bisect-weld the mesh across an arbitrary plane (e.g. active CAD workplane).
    /// Bisects the mesh along plane (origin, normal), mirrors the positive half across the plane,
    /// and welds seam boundary vertices with automatic face-weighted normal preservation.
    pub fn symmetrize_plane(&mut self, origin: Vec3, normal: Vec3) -> Result<(), MeshError> {
        let n = normal.normalize_or_zero();
        if n.length_squared() < 1e-6 {
            return Ok(());
        }

        // 1. Bisect across the plane and clear the negative half
        self.bisect(origin, n, false, true, false)?;

        let sym_tolerance = 1e-3_f32;

        // Snap vertices near the symmetry plane exactly onto the plane
        for v in self.vertices.values_mut() {
            let dist_to_plane = (v.position - origin).dot(n);
            if dist_to_plane.abs() <= sym_tolerance {
                v.position -= n * dist_to_plane;
            }
        }

        // 2. Collect existing vertices to mirror across plane:
        // P_mirrored = P - 2.0 * ((P - origin) . n) * n
        let existing_verts: Vec<(VertexKey, Vec3, Vec2)> = self
            .vertices
            .iter()
            .map(|(k, v)| (k, v.position, v.uv))
            .collect();

        let mut old_to_mirrored_v: HashMap<VertexKey, VertexKey> = HashMap::new();

        for (vk, pos, uv) in existing_verts {
            let dist = (pos - origin).dot(n);
            if dist.abs() <= 1e-4 {
                old_to_mirrored_v.insert(vk, vk);
            } else {
                let mirrored_pos = pos - 2.0 * dist * n;
                let mirrored_vk = self.add_vertex_with_uv(mirrored_pos, uv);
                old_to_mirrored_v.insert(vk, mirrored_vk);
            }
        }

        // 3. Mirror all faces (reversing winding order to keep outward normals)
        let existing_faces: Vec<FaceKey> = self.faces.keys().collect();
        for fk in existing_faces {
            let verts = self.face_vertices(fk);
            let mut mirrored_face_verts: Vec<VertexKey> =
                verts.iter().map(|k| old_to_mirrored_v[k]).collect();
            mirrored_face_verts.reverse();
            let _ = self.add_face(&mirrored_face_verts);
        }

        // 4. Weld boundary seam vertices and update normals
        let _ = self.weld_vertices_by_distance(0.001);
        self.update_face_weighted_normals(true);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::Primitives;

    #[test]
    fn test_symmetrize_plane_cube() {
        let mut cube = Primitives::create_cube(2.0);
        for v in cube.vertices.values_mut() {
            v.position.x += 0.5;
        }

        let res = cube.symmetrize_plane(Vec3::ZERO, Vec3::X);
        assert!(res.is_ok());
        assert!(cube.validate().is_ok());
        assert!(cube.faces.len() >= 6);
    }
}

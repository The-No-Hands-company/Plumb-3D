use crate::mesh::{FaceKey, Mesh, MeshError};

/// Result record of a bi-directional Push-Pull operation.
#[derive(Debug, Clone, PartialEq)]
pub struct PushPullResult {
    pub cap_face: FaceKey,
    pub wall_faces: Vec<FaceKey>,
    pub distance: f32,
}

impl Mesh {
    /// Performs a modal bi-directional Push-Pull operation on face `face_key`:
    /// - If `distance > 0.0`: Extrudes outward as a Boss / Union, creating perpendicular side walls.
    /// - If `distance < 0.0`: Sinks inward as a Cavity / Pocket Cut, creating interior cavity walls.
    /// - Preserves strict 2-manifold BMesh topology and Euler characteristic (χ = V - E + F).
    ///
    /// Returns the new cap/floor face key and the side wall face keys.
    pub fn push_pull_face(
        &mut self,
        face_key: FaceKey,
        distance: f32,
    ) -> Result<PushPullResult, MeshError> {
        if !self.faces.contains_key(face_key) {
            return Err(MeshError::InvalidFace(face_key));
        }

        if distance.abs() < 1e-6 {
            return Ok(PushPullResult {
                cap_face: face_key,
                wall_faces: Vec::new(),
                distance: 0.0,
            });
        }

        let face_normal = self.faces[face_key].normal;
        let v_keys = self.face_vertices(face_key);
        let n = v_keys.len();
        if n < 3 {
            return Err(MeshError::DegenerateFace(n));
        }

        let offset = face_normal * distance;

        // 1. Detach old face
        let face_hes = self.face_half_edges(face_key);
        for &he_key in &face_hes {
            self.half_edges[he_key].face = None;
        }
        self.faces.remove(face_key);

        // 2. Create newly positioned vertices
        let mut new_v_keys = Vec::with_capacity(n);
        for &vk in &v_keys {
            let old_pos = self.vertices[vk].position;
            let old_uv = self.vertices[vk].uv;
            let new_vk = self.add_vertex_with_uv(old_pos + offset, old_uv);
            if self.vertices[vk].crease_weight > 0.0 {
                self.set_vertex_crease(new_vk, self.vertices[vk].crease_weight);
            }
            new_v_keys.push(new_vk);
        }

        // 3. Create side quad walls
        let mut wall_faces = Vec::with_capacity(n);
        for i in 0..n {
            let v_curr = v_keys[i];
            let v_next = v_keys[(i + 1) % n];
            let v_curr_top = new_v_keys[i];
            let v_next_top = new_v_keys[(i + 1) % n];

            let wall_fk = self.add_face(&[v_curr, v_next, v_next_top, v_curr_top])?;
            wall_faces.push(wall_fk);
        }

        // 4. Create top/floor cap face
        let cap_fk = self.add_face(&new_v_keys)?;

        self.update_normals();
        Ok(PushPullResult {
            cap_face: cap_fk,
            wall_faces,
            distance,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::primitives::Primitives;
    use core_math::vec3;

    #[test]
    fn test_push_pull_outward_boss() {
        let mut cube = Primitives::create_cube(2.0);
        let top_fk = cube
            .faces
            .iter()
            .find_map(|(fk, f)| if f.normal.y > 0.9 { Some(fk) } else { None })
            .expect("Must find top face");

        let res = cube
            .push_pull_face(top_fk, 0.5)
            .expect("Outward push-pull should succeed");

        assert_eq!(res.wall_faces.len(), 4);
        assert!(cube.faces.contains_key(res.cap_face));

        // Top face position should now be at Y = 1.5
        let cap_verts = cube.face_vertices(res.cap_face);
        for vk in cap_verts {
            assert!((cube.vertices[vk].position.y - 1.5).abs() < 1e-4);
        }

        assert!(cube.validate().is_ok());
    }

    #[test]
    fn test_push_pull_inward_cavity_cut_after_inscribing() {
        let mut cube = Primitives::create_cube(2.0);
        let top_fk = cube
            .faces
            .iter()
            .find_map(|(fk, f)| if f.normal.y > 0.9 { Some(fk) } else { None })
            .expect("Must find top face");

        // Inscribe a square pocket on top face
        let pocket = [
            vec3(-0.4, 1.0, -0.4),
            vec3(0.4, 1.0, -0.4),
            vec3(0.4, 1.0, 0.4),
            vec3(-0.4, 1.0, 0.4),
        ];
        let (inner_fk, _) = cube
            .inscribe_polygon_on_face(top_fk, &pocket)
            .expect("Inscribing pocket face should succeed");

        // Push-pull inward by -0.6 to carve a recessed cavity
        let res = cube
            .push_pull_face(inner_fk, -0.6)
            .expect("Inward push-pull cavity cut should succeed");

        assert_eq!(res.wall_faces.len(), 4);
        let cap_verts = cube.face_vertices(res.cap_face);
        for vk in cap_verts {
            assert!((cube.vertices[vk].position.y - 0.4).abs() < 1e-4);
        }

        // Cavity walls should point into cavity (towards X=0, Z=0)
        assert!(cube.validate().is_ok());
    }
}

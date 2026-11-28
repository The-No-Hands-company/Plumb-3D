use crate::mesh::{FaceKey, Mesh, MeshError};
use core_math::Vec3;

impl Mesh {
    /// Inscribes a 2D/3D polygon profile onto an existing face `target_face`,
    /// bisecting the face into an interior island face and surrounding perimeter skirt faces.
    ///
    /// Preserves strict 2-manifold BMesh invariants and Euler characteristic (χ = V - E + F).
    /// Returns `(inner_face, Vec<skirt_faces>)`.
    pub fn inscribe_polygon_on_face(
        &mut self,
        target_face: FaceKey,
        polygon_points: &[Vec3],
    ) -> Result<(FaceKey, Vec<FaceKey>), MeshError> {
        let face = self
            .faces
            .get(target_face)
            .ok_or(MeshError::InvalidFace(target_face))?;

        let m = polygon_points.len();
        if m < 3 {
            return Err(MeshError::DegenerateFace(m));
        }

        let outer_vks = self.face_vertices(target_face);
        let n = outer_vks.len();
        if n < 3 {
            return Err(MeshError::DegenerateFace(n));
        }

        let face_normal = face.normal;
        let v0_pos = self.vertices[outer_vks[0]].position;

        // 1. Project polygon points onto the face plane for exact coplanarity
        let mut proj_points = Vec::with_capacity(m);
        let mut poly_center = Vec3::ZERO;
        for &p in polygon_points {
            let dist = (p - v0_pos).dot(face_normal);
            let projected = p - face_normal * dist;
            proj_points.push(projected);
            poly_center += projected;
        }
        poly_center /= m as f32;

        // 2. Ensure polygon vertices match the outer face normal orientation (counter-clockwise)
        let edge1 = proj_points[1] - proj_points[0];
        let edge2 = proj_points[2] - proj_points[0];
        let poly_normal = edge1.cross(edge2);
        if poly_normal.dot(face_normal) < 0.0 {
            proj_points.reverse();
        }

        // 3. Add inner vertices to mesh
        let mut inner_vks = Vec::with_capacity(m);
        for &p in &proj_points {
            let vk = self.add_vertex(p);
            inner_vks.push(vk);
        }

        // 4. Detach and remove the original target face
        let face_hes = self.face_half_edges(target_face);
        for &he_key in &face_hes {
            self.half_edges[he_key].face = None;
        }
        self.faces.remove(target_face);

        // 5. Create the newly inscribed inner face
        let inner_face_key = self.add_face(&inner_vks)?;

        // 6. Bridge the outer loop and inner loop to form the annulus skirt faces
        // Find inner vertex closest to outer_vks[0] to establish initial seam
        let mut best_start_j = 0;
        let mut best_dist_sq = f32::INFINITY;
        for (j, &ivk) in inner_vks.iter().enumerate() {
            let d_sq = (self.vertices[ivk].position - v0_pos).length_squared();
            if d_sq < best_dist_sq {
                best_dist_sq = d_sq;
                best_start_j = j;
            }
        }

        let mut skirt_faces = Vec::new();
        let mut curr_i = 0;
        let mut curr_j = best_start_j;
        let mut steps_i = 0;
        let mut steps_j = 0;

        while steps_i < n || steps_j < m {
            let next_i = (curr_i + 1) % n;
            let next_j = (curr_j + 1) % m;

            let vi = outer_vks[curr_i];
            let v_next_i = outer_vks[next_i];
            let pj = inner_vks[curr_j];
            let p_next_j = inner_vks[next_j];

            let can_advance_i = steps_i < n;
            let can_advance_j = steps_j < m;

            let advance_i = if can_advance_i && can_advance_j {
                let diag_i_sq = (self.vertices[v_next_i].position - self.vertices[pj].position)
                    .length_squared();
                let diag_j_sq = (self.vertices[vi].position - self.vertices[p_next_j].position)
                    .length_squared();
                diag_i_sq <= diag_j_sq
            } else {
                can_advance_i
            };

            if advance_i {
                // Form triangle [vi, v_next_i, pj]
                let fk = self.add_face(&[vi, v_next_i, pj])?;
                skirt_faces.push(fk);
                curr_i = next_i;
                steps_i += 1;
            } else {
                // Form triangle [vi, p_next_j, pj]
                let fk = self.add_face(&[vi, p_next_j, pj])?;
                skirt_faces.push(fk);
                curr_j = next_j;
                steps_j += 1;
            }
        }

        self.update_normals();
        Ok((inner_face_key, skirt_faces))
    }
}

#[cfg(test)]
mod tests {
    use crate::primitives::Primitives;
    use core_math::vec3;

    #[test]
    fn test_inscribe_quad_on_cube_face() {
        let mut cube = Primitives::create_cube(2.0);
        let initial_verts = cube.vertices.len();
        let initial_faces = cube.faces.len();

        // Find top face (normal pointing along +Y)
        let top_fk = cube
            .faces
            .iter()
            .find_map(|(fk, f)| if f.normal.y > 0.9 { Some(fk) } else { None })
            .expect("Must find top face on cube");

        // Define a smaller centered quad profile on top face (Y = 1.0)
        let profile = [
            vec3(-0.5, 1.0, -0.5),
            vec3(0.5, 1.0, -0.5),
            vec3(0.5, 1.0, 0.5),
            vec3(-0.5, 1.0, 0.5),
        ];

        let (inner_fk, skirt_fks) = cube
            .inscribe_polygon_on_face(top_fk, &profile)
            .expect("Inscribing quad profile on face should succeed");

        assert!(cube.faces.contains_key(inner_fk));
        assert!(!skirt_fks.is_empty());
        assert_eq!(cube.vertices.len(), initial_verts + 4);
        assert_eq!(cube.faces.len(), initial_faces - 1 + 1 + skirt_fks.len());

        // Validate 2-manifold BMesh invariants
        assert!(cube.validate().is_ok());
    }

    #[test]
    fn test_inscribe_octagon_circle_on_cube_face() {
        let mut cube = Primitives::create_cube(2.0);

        let top_fk = cube
            .faces
            .iter()
            .find_map(|(fk, f)| if f.normal.y > 0.9 { Some(fk) } else { None })
            .expect("Must find top face on cube");

        // Generate 8-point circular profile on Y = 1.0 plane
        let radius = 0.6_f32;
        let mut circle_pts = Vec::new();
        for i in 0..8 {
            let theta = i as f32 * std::f32::consts::TAU / 8.0;
            circle_pts.push(vec3(theta.cos() * radius, 1.0, theta.sin() * radius));
        }

        let (inner_fk, skirt_fks) = cube
            .inscribe_polygon_on_face(top_fk, &circle_pts)
            .expect("Inscribing circular profile on face should succeed");

        assert!(cube.faces.contains_key(inner_fk));
        assert_eq!(skirt_fks.len(), 4 + 8); // 4 outer + 8 inner = 12 skirt triangles
        assert!(cube.validate().is_ok());
    }
}

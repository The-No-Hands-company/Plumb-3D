use crate::mesh::{EdgeKey, FaceKey, Mesh, MeshError, VertexKey};

impl Mesh {
    /// Splits a quad face into two sub-quads by inserting a cut between opposite edges at ratio `t` (0.01..0.99).
    pub fn split_quad_face(
        &mut self,
        face_key: FaceKey,
        edge_index: usize,
        t: f32,
    ) -> Result<(FaceKey, FaceKey), MeshError> {
        let v_keys = self.face_vertices(face_key);
        if v_keys.len() != 4 {
            return Err(MeshError::InvariantFailed(
                "Split quad face requires a 4-vertex quad".into(),
            ));
        }

        let i0 = edge_index % 4;
        let i1 = (i0 + 1) % 4;
        let i2 = (i0 + 2) % 4;
        let i3 = (i0 + 3) % 4;

        let v0 = v_keys[i0];
        let v1 = v_keys[i1];
        let v2 = v_keys[i2];
        let v3 = v_keys[i3];

        let p0 = self.vertices[v0].position;
        let p1 = self.vertices[v1].position;
        let p2 = self.vertices[v2].position;
        let p3 = self.vertices[v3].position;

        let uv0 = self.vertices[v0].uv;
        let uv1 = self.vertices[v1].uv;
        let uv2 = self.vertices[v2].uv;
        let uv3 = self.vertices[v3].uv;

        let ratio = t.clamp(0.01, 0.99);

        // New split vertices on opposite edges
        let m0_pos = p0 + (p1 - p0) * ratio;
        let m0_uv = uv0 + (uv1 - uv0) * ratio;
        let m1_pos = p3 + (p2 - p3) * ratio;
        let m1_uv = uv3 + (uv2 - uv3) * ratio;

        let m0 = self.add_vertex_with_uv(m0_pos, m0_uv);
        let m1 = self.add_vertex_with_uv(m1_pos, m1_uv);

        let mat_id = self.faces.get(face_key).map(|f| f.material_id).unwrap_or(0);

        // Detach old face
        let face_hes = self.face_half_edges(face_key);
        for &he_key in &face_hes {
            self.half_edges[he_key].face = None;
        }
        self.faces.remove(face_key);

        // Add the two new sub-quads
        let f_a = self.add_face(&[v0, m0, m1, v3])?;
        let f_b = self.add_face(&[m0, v1, v2, m1])?;
        let _ = self.set_face_material(f_a, mat_id);
        let _ = self.set_face_material(f_b, mat_id);

        self.update_normals();
        Ok((f_a, f_b))
    }

    /// Connect two non-adjacent vertices on a polygon face, splitting the face into two sub-faces.
    /// Equivalent to Maya's Split Polygon / Multi-Cut tool and Blender's Vertex Connect (J key).
    pub fn connect_vertices(
        &mut self,
        face_key: FaceKey,
        v0: VertexKey,
        v1: VertexKey,
    ) -> Result<(FaceKey, FaceKey), MeshError> {
        let v_keys = self.face_vertices(face_key);
        let n = v_keys.len();
        if n < 4 {
            return Err(MeshError::InvariantFailed(
                "Cannot split a face with fewer than 4 vertices".into(),
            ));
        }

        let idx0 = v_keys
            .iter()
            .position(|&k| k == v0)
            .ok_or(MeshError::InvalidVertex(v0))?;
        let idx1 = v_keys
            .iter()
            .position(|&k| k == v1)
            .ok_or(MeshError::InvalidVertex(v1))?;

        if idx0 == idx1 {
            return Err(MeshError::InvariantFailed(
                "Cannot connect a vertex to itself".into(),
            ));
        }

        if (idx0 + 1) % n == idx1 || (idx1 + 1) % n == idx0 {
            return Err(MeshError::InvariantFailed(
                "Vertices are already adjacent on face".into(),
            ));
        }

        let (i_start, i_end) = if idx0 < idx1 {
            (idx0, idx1)
        } else {
            (idx1, idx0)
        };

        // Sub-loop 1: from i_start to i_end inclusive
        let mut loop1 = Vec::new();
        for i in i_start..=i_end {
            loop1.push(v_keys[i]);
        }

        // Sub-loop 2: from i_end to i_start inclusive (wrapping around)
        let mut loop2 = Vec::new();
        let mut curr = i_end;
        while curr != i_start {
            loop2.push(v_keys[curr]);
            curr = (curr + 1) % n;
        }
        loop2.push(v_keys[i_start]);

        // 1. Detach old face
        let old_mat_id = self.faces.get(face_key).map(|f| f.material_id).unwrap_or(0);
        let face_hes = self.face_half_edges(face_key);
        for &he_key in &face_hes {
            self.half_edges[he_key].face = None;
        }
        self.faces.remove(face_key);

        // 2. Add both sub-faces
        let f_a = self.add_face(&loop1)?;
        let f_b = self.add_face(&loop2)?;
        self.faces[f_a].material_id = old_mat_id;
        self.faces[f_b].material_id = old_mat_id;

        self.update_normals();
        Ok((f_a, f_b))
    }

    /// Subdivide an edge into two edges by inserting a new vertex at parametric factor `t` (0.01..0.99).
    /// Updates all incident polygon faces while maintaining 2-manifold half-edge topology.
    pub fn split_edge(&mut self, edge_key: EdgeKey, t: f32) -> Result<VertexKey, MeshError> {
        let edge = self
            .edges
            .get(edge_key)
            .ok_or(MeshError::InvariantFailed("Edge not found".into()))?;
        let h1_key = edge.half_edge;
        let h1 = self.half_edges[h1_key].clone();
        let h2_key = h1.twin;
        let h2 = self.half_edges[h2_key].clone();

        let v0 = h1.origin;
        let v1 = h1.target;

        let p0 = self.vertices[v0].position;
        let p1 = self.vertices[v1].position;
        let uv0 = self.vertices[v0].uv;
        let uv1 = self.vertices[v1].uv;

        let factor = t.clamp(0.01, 0.99);
        let new_pos = p0 + (p1 - p0) * factor;
        let new_uv = uv0 + (uv1 - uv0) * factor;
        let new_vk = self.add_vertex_with_uv(new_pos, new_uv);

        let mut face_updates = Vec::new();

        if let Some(f1_key) = h1.face {
            let mut verts = self.face_vertices(f1_key);
            if let Some(pos) = verts.iter().position(|&v| v == v0) {
                let next_idx = (pos + 1) % verts.len();
                if verts[next_idx] == v1 {
                    verts.insert(next_idx, new_vk);
                    face_updates.push((f1_key, verts));
                }
            }
        }

        if let Some(f2_key) = h2.face {
            let mut verts = self.face_vertices(f2_key);
            if let Some(pos) = verts.iter().position(|&v| v == v1) {
                let next_idx = (pos + 1) % verts.len();
                if verts[next_idx] == v0 {
                    verts.insert(next_idx, new_vk);
                    face_updates.push((f2_key, verts));
                }
            }
        }

        // Detach old faces
        for &(fk, _) in &face_updates {
            for he_key in self.face_half_edges(fk) {
                self.half_edges[he_key].face = None;
            }
            self.faces.remove(fk);
        }

        // Remove old half-edges from vertex outgoing lists
        self.vertices[v0].half_edges.retain(|he| *he != h1_key);
        self.vertices[v1].half_edges.retain(|he| *he != h2_key);
        self.half_edges.remove(h1_key);
        self.half_edges.remove(h2_key);
        self.edges.remove(edge_key);

        // Re-add faces with inserted vertex
        for (_, verts) in face_updates {
            self.add_face(&verts)?;
        }

        self.update_normals();
        Ok(new_vk)
    }
}

use crate::mesh::{EdgeKey, FaceKey, HalfEdgeKey, Mesh, MeshError, VertexKey};
use core_math::Vec3;
use std::collections::HashSet;


impl Mesh {
    pub fn split_quad_face(
        &mut self,
        face_key: FaceKey,
        edge_index: usize,
        t: f32,
    ) -> Result<(FaceKey, FaceKey), MeshError> {
        let v_keys = self.face_vertices(face_key);
        if v_keys.len() != 4 {
            return Err(MeshError::InvariantFailed("Split quad face requires a 4-vertex quad".into()));
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

        let (i_start, i_end) = if idx0 < idx1 { (idx0, idx1) } else { (idx1, idx0) };

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

    /// Subdivide an edge into two edges by inserting a new vertex at parametric factor `t` (0.0..1.0).
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

    /// Insert an Edge Loop across an aligned quad strip / ring intersecting the selected edge at ratio `t`.
    /// Traverses quad neighbors in both directions until hitting an open boundary, a non-quad face, or looping.
    /// Equivalent to Blender's Loop Cut (Ctrl+R) and Maya's Insert Edge Loop tool.
    pub fn loop_cut(&mut self, edge_key: EdgeKey, t: f32) -> Result<Vec<FaceKey>, MeshError> {
        let edge = self
            .edges
            .get(edge_key)
            .ok_or(MeshError::InvariantFailed("Edge not found".into()))?;
        let h1_key = edge.half_edge;
        let h2_key = self.half_edges[h1_key].twin;

        let mut faces_to_split: Vec<(FaceKey, EdgeKey)> = Vec::new();
        let mut visited_faces = HashSet::new();

        // Helper closure to explore one direction along the quad ring
        let explore_direction = |start_he: HalfEdgeKey,
                                     faces_to_split: &mut Vec<(FaceKey, EdgeKey)>,
                                     visited_faces: &mut HashSet<FaceKey>| {
            let mut curr_he = start_he;
            loop {
                let f_key = match self.half_edges[curr_he].face {
                    Some(fk) => fk,
                    None => break,
                };

                if visited_faces.contains(&f_key) {
                    break;
                }

                let verts = self.face_vertices(f_key);
                if verts.len() != 4 {
                    // Non-quad face terminates the edge loop ring
                    break;
                }

                visited_faces.insert(f_key);
                faces_to_split.push((f_key, self.half_edges[curr_he].edge));

                // Opposite edge in quad: advance 2 steps in face loop
                let next1 = self.half_edges[curr_he].next;
                let opp_he = self.half_edges[next1].next;
                let twin_he = self.half_edges[opp_he].twin;
                curr_he = twin_he;
            }
        };

        explore_direction(h1_key, &mut faces_to_split, &mut visited_faces);
        explore_direction(h2_key, &mut faces_to_split, &mut visited_faces);

        let mut new_faces = Vec::new();
        for (f_key, e_key) in faces_to_split {
            if !self.faces.contains_key(f_key) {
                continue;
            }
            // Find edge index in face
            let hes = self.face_half_edges(f_key);
            let mut edge_idx = 0;
            for (i, &he) in hes.iter().enumerate() {
                if self.half_edges[he].edge == e_key {
                    edge_idx = i;
                    break;
                }
            }
            let (f1, f2) = self.split_quad_face(f_key, edge_idx, t)?;
            new_faces.push(f1);
            new_faces.push(f2);
        }

        Ok(new_faces)
    }

    /// Computes the 3D cut line segments for a loop cut across adjacent quads without modifying geometry.
    pub fn get_loop_cut_preview(&self, edge_key: EdgeKey, t: f32) -> Vec<(Vec3, Vec3)> {
        let edge = match self.edges.get(edge_key) {
            Some(e) => e,
            None => return Vec::new(),
        };
        let h1_key = edge.half_edge;
        let h2_key = self.half_edges[h1_key].twin;

        let mut faces_to_split: Vec<(FaceKey, EdgeKey)> = Vec::new();
        let mut visited_faces = HashSet::new();

        let mut explore_direction = |start_he: HalfEdgeKey| {
            let mut curr_he = start_he;
            loop {
                let f_key = match self.half_edges[curr_he].face {
                    Some(fk) => fk,
                    None => break,
                };

                if visited_faces.contains(&f_key) {
                    break;
                }

                let verts = self.face_vertices(f_key);
                if verts.len() != 4 {
                    break;
                }

                visited_faces.insert(f_key);
                faces_to_split.push((f_key, self.half_edges[curr_he].edge));

                let next1 = self.half_edges[curr_he].next;
                let opp_he = self.half_edges[next1].next;
                let twin_he = self.half_edges[opp_he].twin;
                curr_he = twin_he;
            }
        };

        explore_direction(h1_key);
        explore_direction(h2_key);

        let ratio = t.clamp(0.01, 0.99);
        let mut lines = Vec::new();

        for (f_key, e_key) in faces_to_split {
            let hes = self.face_half_edges(f_key);
            let mut edge_idx = 0;
            for (i, &he) in hes.iter().enumerate() {
                if self.half_edges[he].edge == e_key {
                    edge_idx = i;
                    break;
                }
            }
            let v_keys = self.face_vertices(f_key);
            if v_keys.len() == 4 {
                let i0 = edge_idx % 4;
                let i1 = (i0 + 1) % 4;
                let i2 = (i0 + 2) % 4;
                let i3 = (i0 + 3) % 4;

                let p0 = self.vertices[v_keys[i0]].position;
                let p1 = self.vertices[v_keys[i1]].position;
                let p2 = self.vertices[v_keys[i2]].position;
                let p3 = self.vertices[v_keys[i3]].position;

                let m0 = p0 + (p1 - p0) * ratio;
                let m1 = p3 + (p2 - p3) * ratio;
                lines.push((m0, m1));
            }
        }

        lines
    }

    /// Bevel / Chamfer an edge, expanding it into a new quad separating the adjacent faces.

    pub fn poke_face(&mut self, face_key: FaceKey, offset: f32) -> Result<Vec<FaceKey>, MeshError> {
        let v_keys = self.face_vertices(face_key);
        let n = v_keys.len();
        if n < 3 {
            return Err(MeshError::DegenerateFace(n));
        }

        let normal = self.faces.get(face_key).map(|f| f.normal).unwrap_or(Vec3::Y);

        // Compute face centroid and UV
        let mut center_pos = Vec3::ZERO;
        let mut center_uv = core_math::Vec2::ZERO;
        for &vk in &v_keys {
            center_pos += self.vertices[vk].position;
            center_uv += self.vertices[vk].uv;
        }
        center_pos /= n as f32;
        center_uv /= n as f32;
        center_pos += normal * offset;

        // Detach old face
        let face_hes = self.face_half_edges(face_key);
        for &he in &face_hes {
            self.half_edges[he].face = None;
        }
        self.faces.remove(face_key);

        // Add center vertex
        let center_vk = self.add_vertex_with_uv(center_pos, center_uv);

        // Add triangular fan faces
        let mut created_faces = Vec::with_capacity(n);
        for i in 0..n {
            let v_curr = v_keys[i];
            let v_next = v_keys[(i + 1) % n];
            let fk = self.add_face(&[v_curr, v_next, center_vk])?;
            created_faces.push(fk);
        }

        self.update_normals();
        Ok(created_faces)
    }

    /// Dissolve an edge shared by two faces, merging them into a single polygon (Blender Dissolve Edge).
    pub fn dissolve_edge(&mut self, edge_key: EdgeKey) -> Result<FaceKey, MeshError> {
        let edge = self.edges.get(edge_key).ok_or(MeshError::InvariantFailed("Invalid edge".into()))?;
        let he0_key = edge.half_edge;
        let he0 = &self.half_edges[he0_key];
        let he1_key = he0.twin;
        let he1 = &self.half_edges[he1_key];

        let f0 = match he0.face {
            Some(f) => f,
            None => return Err(MeshError::InvariantFailed("Cannot dissolve boundary edge".into())),
        };
        let f1 = match he1.face {
            Some(f) => f,
            None => return Err(MeshError::InvariantFailed("Cannot dissolve boundary edge".into())),
        };

        if f0 == f1 {
            return Err(MeshError::InvariantFailed("Edge belongs to the same face twice".into()));
        }

        let (v0, v1) = (he0.origin, he0.target);

        let verts_f0 = self.face_vertices(f0);
        let verts_f1 = self.face_vertices(f1);

        let idx0_f0 = verts_f0.iter().position(|&v| v == v0).ok_or(MeshError::InvalidVertex(v0))?;
        let len0 = verts_f0.len();

        let idx1_f1 = verts_f1.iter().position(|&v| v == v1).ok_or(MeshError::InvalidVertex(v1))?;
        let len1 = verts_f1.len();

        let mut combined = Vec::with_capacity(len0 + len1 - 2);

        for offset in 1..=len0 {
            let i = (idx0_f0 + offset) % len0;
            combined.push(verts_f0[i]);
            if verts_f0[i] == v0 {
                break;
            }
        }

        for offset in 1..len1 {
            let i = (idx1_f1 + offset) % len1;
            if verts_f1[i] == v1 {
                break;
            }
            if verts_f1[i] != v0 {
                combined.push(verts_f1[i]);
            }
        }

        if combined.len() < 3 {
            return Err(MeshError::DegenerateFace(combined.len()));
        }

        let mut unique = HashSet::new();
        for &vk in &combined {
            if !unique.insert(vk) {
                return Err(MeshError::DuplicateVertexInFace(vk));
            }
        }

        // Rebuild clean topology without the dissolved edge or orphaned half-edges
        let mut new_mesh = Mesh::new();
        let mut old_to_new_v = std::collections::HashMap::new();
        for (vk, v) in &self.vertices {
            let new_vk = new_mesh.add_vertex_with_uv(v.position, v.uv);
            old_to_new_v.insert(vk, new_vk);
        }

        // Add all existing faces except f0 and f1
        for (fk, _) in &self.faces {
            if fk != f0 && fk != f1 {
                let f_verts = self.face_vertices(fk);
                let new_f_verts: Vec<VertexKey> = f_verts.iter().map(|k| old_to_new_v[k]).collect();
                new_mesh.add_face(&new_f_verts)?;
            }
        }

        // Add the merged face
        let new_combined: Vec<VertexKey> = combined.iter().map(|k| old_to_new_v[k]).collect();
        let merged_fk = new_mesh.add_face(&new_combined)?;

        // Copy vertex groups
        for vg in &self.vertex_groups {
            let idx = new_mesh.add_vertex_group(&vg.name);
            for (&vk, &w) in &vg.weights {
                if let Some(&new_vk) = old_to_new_v.get(&vk) {
                    new_mesh.vertex_groups[idx].set_weight(new_vk, w);
                }
            }
        }

        new_mesh.update_normals();
        *self = new_mesh;
        Ok(merged_fk)
    }

    /// Joins another mesh into this mesh, transforming the other mesh's vertices by `transform`.
    /// Merges vertices, edges, faces, and vertex groups into a unified 2-manifold BMesh.

    pub fn rip_vertices(&mut self, vertices: &[VertexKey]) -> Result<Vec<VertexKey>, MeshError> {
        if vertices.is_empty() {
            return Ok(Vec::new());
        }

        let mut new_vertices = Vec::new();
        // Collect existing faces with their ordered vertex keys and material IDs
        let mut face_data: Vec<(FaceKey, Vec<VertexKey>, u32)> = Vec::new();
        for (fk, f) in &self.faces {
            face_data.push((fk, self.face_vertices(fk), f.material_id));
        }

        // For each selected vertex, identify incident faces and partition them
        for &vk in vertices {
            if !self.vertices.contains_key(vk) {
                continue;
            }

            // Find all faces containing vk
            let incident_faces: Vec<usize> = face_data
                .iter()
                .enumerate()
                .filter(|(_, (_, verts, _))| verts.contains(&vk))
                .map(|(i, _)| i)
                .collect();

            if incident_faces.len() < 2 {
                // Cannot rip a vertex with fewer than 2 incident faces
                continue;
            }

            // Duplicate the vertex
            let orig = &self.vertices[vk];
            let new_vk = self.add_vertex_with_uv_and_color(orig.position, orig.uv, orig.color);
            new_vertices.push(new_vk);

            // Copy vertex group weights from orig to new_vk
            for vg in &mut self.vertex_groups {
                if let Some(&w) = vg.weights.get(&vk) {
                    vg.set_weight(new_vk, w);
                }
            }

            // Partition incident faces: assign half to new_vk
            let split_count = incident_faces.len() / 2;
            for &idx in &incident_faces[..split_count] {
                for v in &mut face_data[idx].1 {
                    if *v == vk {
                        *v = new_vk;
                    }
                }
            }
        }

        if new_vertices.is_empty() {
            return Ok(Vec::new());
        }

        // Rebuild mesh faces with updated vertex indices
        self.faces.clear();
        self.edges.clear();
        self.half_edges.clear();
        for v in self.vertices.values_mut() {
            v.half_edges.clear();
        }

        for (_, verts, mat_id) in face_data {
            if verts.len() >= 3 {
                if let Ok(fk) = self.add_face(&verts) {
                    self.faces[fk].material_id = mat_id;
                }
            }
        }

        self.update_normals();
        Ok(new_vertices)
    }
}

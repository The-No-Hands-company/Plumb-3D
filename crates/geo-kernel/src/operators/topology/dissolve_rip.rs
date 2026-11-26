use crate::mesh::{EdgeKey, FaceKey, Mesh, MeshError, VertexKey};
use core_math::Vec3;
use std::collections::HashSet;

impl Mesh {
    /// Pokes a polygon face outward by creating a central apex vertex connected to all boundary vertices by a triangle fan.
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

    /// Rips / separates vertices along adjacent faces, creating open seams (Blender Rip operator V key).
    pub fn rip_vertices(&mut self, vertices: &[VertexKey]) -> Result<Vec<VertexKey>, MeshError> {
        if vertices.is_empty() {
            return Ok(Vec::new());
        }

        let mut new_vertices = Vec::new();
        let mut face_data: Vec<(FaceKey, Vec<VertexKey>, u32)> = Vec::new();
        for (fk, f) in &self.faces {
            face_data.push((fk, self.face_vertices(fk), f.material_id));
        }

        for &vk in vertices {
            if !self.vertices.contains_key(vk) {
                continue;
            }

            let incident_faces: Vec<usize> = face_data
                .iter()
                .enumerate()
                .filter(|(_, (_, verts, _))| verts.contains(&vk))
                .map(|(i, _)| i)
                .collect();

            if incident_faces.len() < 2 {
                continue;
            }

            // Duplicate the vertex
            let orig = &self.vertices[vk];
            let new_vk = self.add_vertex_with_uv_and_color(orig.position, orig.uv, orig.color);
            new_vertices.push(new_vk);

            for vg in &mut self.vertex_groups {
                if let Some(&w) = vg.weights.get(&vk) {
                    vg.set_weight(new_vk, w);
                }
            }

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

    /// Dissolve a contiguous set of adjacent faces into a single merged polygon (Blender Dissolve Faces).
    pub fn dissolve_faces(&mut self, faces: &[FaceKey]) -> Result<FaceKey, MeshError> {
        if faces.is_empty() {
            return Err(MeshError::InvariantFailed("No faces to dissolve".into()));
        }
        if faces.len() == 1 {
            return Ok(faces[0]);
        }

        let face_set: HashSet<FaceKey> = faces.iter().copied().collect();

        // 1. Collect all directed boundary edges (origin -> target) of the face selection
        let mut next_vertex: std::collections::HashMap<VertexKey, VertexKey> = std::collections::HashMap::new();
        let mut boundary_origins = Vec::new();

        for &fk in faces {
            for &he_key in &self.face_half_edges(fk) {
                let he = &self.half_edges[he_key];
                let twin = &self.half_edges[he.twin];
                let is_internal = twin.face.is_some_and(|tfk| face_set.contains(&tfk));
                if !is_internal {
                    next_vertex.insert(he.origin, he.target);
                    boundary_origins.push(he.origin);
                }
            }
        }

        if boundary_origins.is_empty() {
            return Err(MeshError::InvariantFailed("No boundary edges found for face set".into()));
        }

        // 2. Trace the outer boundary loop
        let start_vk = boundary_origins[0];
        let mut loop_verts = Vec::new();
        let mut current_vk = start_vk;

        for _ in 0..next_vertex.len() {
            loop_verts.push(current_vk);
            if let Some(&nxt) = next_vertex.get(&current_vk) {
                if nxt == start_vk {
                    break;
                }
                current_vk = nxt;
            } else {
                return Err(MeshError::InvariantFailed("Discontinuous boundary loop in dissolve_faces".into()));
            }
        }

        if loop_verts.len() < 3 {
            return Err(MeshError::DegenerateFace(loop_verts.len()));
        }

        // 3. Rebuild clean topology: keep all non-dissolved faces and add the merged boundary face
        let mut new_mesh = Mesh::new();
        let mut old_to_new_v = std::collections::HashMap::new();

        for (vk, v) in &self.vertices {
            let new_vk = new_mesh.add_vertex_with_uv_and_color(v.position, v.uv, v.color);
            old_to_new_v.insert(vk, new_vk);
        }

        // Re-add non-dissolved faces
        for (fk, f) in &self.faces {
            if !face_set.contains(&fk) {
                let f_verts = self.face_vertices(fk);
                let new_f_verts: Vec<VertexKey> = f_verts.iter().map(|k| old_to_new_v[k]).collect();
                if let Ok(new_fk) = new_mesh.add_face(&new_f_verts) {
                    new_mesh.faces[new_fk].material_id = f.material_id;
                }
            }
        }

        // Add the merged boundary face
        let new_loop_verts: Vec<VertexKey> = loop_verts.iter().map(|k| old_to_new_v[k]).collect();
        let merged_fk = new_mesh.add_face(&new_loop_verts)?;

        // Inherit material from the first dissolved face
        if let Some(first_fk) = faces.first() {
            if let Some(f) = self.faces.get(*first_fk) {
                new_mesh.faces[merged_fk].material_id = f.material_id;
            }
        }

        // Copy vertex groups
        for vg in &self.vertex_groups {
            let idx = new_mesh.add_vertex_group(&vg.name);
            for (&vk, &w) in &vg.weights {
                if let Some(&new_vk) = old_to_new_v.get(&vk) {
                    new_mesh.vertex_groups[idx].set_weight(new_vk, w);
                }
            }
        }

        // Remove orphaned isolated vertices that have no half-edges
        let orphaned: Vec<VertexKey> = new_mesh
            .vertices
            .iter()
            .filter(|(_, v)| v.half_edges.is_empty())
            .map(|(k, _)| k)
            .collect();
        for ovk in orphaned {
            new_mesh.vertices.remove(ovk);
        }

        new_mesh.update_normals();
        *self = new_mesh;
        Ok(merged_fk)
    }
}

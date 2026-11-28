use crate::mesh::{FaceKey, Mesh, MeshError, VertexKey};
use core_math::Mat4;
use std::collections::{HashMap, HashSet};

impl Mesh {
    pub fn join(&mut self, other: &Mesh, transform: Mat4) -> Result<(), MeshError> {
        let mut other_to_self_v = std::collections::HashMap::new();
        for (other_vk, other_v) in &other.vertices {
            let world_pos = transform.transform_point3(other_v.position);
            let self_vk = self.add_vertex_with_uv(world_pos, other_v.uv);
            if other_v.crease_weight > 0.0 {
                self.set_vertex_crease(self_vk, other_v.crease_weight);
            }
            other_to_self_v.insert(other_vk, self_vk);
        }

        for (other_fk, _) in &other.faces {
            let other_verts = other.face_vertices(other_fk);
            let self_verts: Vec<VertexKey> =
                other_verts.iter().map(|k| other_to_self_v[k]).collect();
            self.add_face(&self_verts)?;
        }

        // Transfer edge creases & sharpness
        for (_, other_edge) in &other.edges {
            if other_edge.crease_weight > 0.0 || other_edge.is_sharp {
                let he = &other.half_edges[other_edge.half_edge];
                if let (Some(&v0), Some(&v1)) = (
                    other_to_self_v.get(&he.origin),
                    other_to_self_v.get(&he.target),
                ) {
                    if let Some(self_ek) = self.find_edge(v0, v1) {
                        self.set_edge_crease(self_ek, other_edge.crease_weight);
                        self.set_edge_sharp(self_ek, other_edge.is_sharp);
                    }
                }
            }
        }

        for other_vg in &other.vertex_groups {
            let self_vg_idx = if let Some(existing_idx) = self
                .vertex_groups
                .iter()
                .position(|vg| vg.name == other_vg.name)
            {
                existing_idx
            } else {
                self.add_vertex_group(&other_vg.name)
            };

            for (&other_vk, &w) in &other_vg.weights {
                if let Some(&self_vk) = other_to_self_v.get(&other_vk) {
                    self.vertex_groups[self_vg_idx].set_weight(self_vk, w);
                }
            }
        }

        self.update_normals();
        Ok(())
    }

    /// Separates a set of faces out of this mesh into a brand new Mesh (Blender `P` -> Selection / Maya Extract).
    /// Removes the extracted faces from `self` and rebuilds clean 2-manifold topology on both meshes.
    pub fn separate_faces(&mut self, faces_to_extract: &[FaceKey]) -> Result<Mesh, MeshError> {
        if faces_to_extract.is_empty() {
            return Ok(Mesh::new());
        }

        let extract_set: HashSet<FaceKey> = faces_to_extract.iter().copied().collect();

        // 1. Build extracted mesh
        let mut extracted_mesh = Mesh::new();
        let mut self_to_extracted_v = std::collections::HashMap::new();

        for &fk in faces_to_extract {
            if self.faces.contains_key(fk) {
                for vk in self.face_vertices(fk) {
                    if let std::collections::hash_map::Entry::Vacant(e) =
                        self_to_extracted_v.entry(vk)
                    {
                        let v = &self.vertices[vk];
                        let new_vk = extracted_mesh.add_vertex_with_uv(v.position, v.uv);
                        e.insert(new_vk);
                    }
                }
            }
        }

        for &fk in faces_to_extract {
            if self.faces.contains_key(fk) {
                let verts = self.face_vertices(fk);
                let new_verts: Vec<VertexKey> =
                    verts.iter().map(|k| self_to_extracted_v[k]).collect();
                extracted_mesh.add_face(&new_verts)?;
            }
        }

        // Copy vertex groups to extracted mesh
        for vg in &self.vertex_groups {
            let mut has_any = false;
            for &vk in self_to_extracted_v.keys() {
                if vg.weights.contains_key(&vk) {
                    has_any = true;
                    break;
                }
            }
            if has_any {
                let ext_vg_idx = extracted_mesh.add_vertex_group(&vg.name);
                for (&old_vk, &new_vk) in &self_to_extracted_v {
                    if let Some(&w) = vg.weights.get(&old_vk) {
                        extracted_mesh.vertex_groups[ext_vg_idx].set_weight(new_vk, w);
                    }
                }
            }
        }
        extracted_mesh.update_normals();

        // 2. Rebuild self without the extracted faces
        let mut remaining_mesh = Mesh::new();
        let mut old_to_new_v = std::collections::HashMap::new();

        // Only copy vertices that belong to remaining faces
        for (fk, _) in &self.faces {
            if !extract_set.contains(&fk) {
                for vk in self.face_vertices(fk) {
                    if let std::collections::hash_map::Entry::Vacant(e) = old_to_new_v.entry(vk) {
                        let v = &self.vertices[vk];
                        let new_vk = remaining_mesh.add_vertex_with_uv(v.position, v.uv);
                        e.insert(new_vk);
                    }
                }
            }
        }

        for (fk, _) in &self.faces {
            if !extract_set.contains(&fk) {
                let verts = self.face_vertices(fk);
                let new_verts: Vec<VertexKey> = verts.iter().map(|k| old_to_new_v[k]).collect();
                remaining_mesh.add_face(&new_verts)?;
            }
        }

        // Copy vertex groups to remaining mesh
        for vg in &self.vertex_groups {
            let mut has_any = false;
            for &vk in old_to_new_v.keys() {
                if vg.weights.contains_key(&vk) {
                    has_any = true;
                    break;
                }
            }
            if has_any {
                let rem_vg_idx = remaining_mesh.add_vertex_group(&vg.name);
                for (&old_vk, &new_vk) in &old_to_new_v {
                    if let Some(&w) = vg.weights.get(&old_vk) {
                        remaining_mesh.vertex_groups[rem_vg_idx].set_weight(new_vk, w);
                    }
                }
            }
        }
        remaining_mesh.update_normals();

        *self = remaining_mesh;
        Ok(extracted_mesh)
    }

    /// Reverses the winding order of the specified faces (or all faces if empty), flipping their surface normals.

    pub fn separate_loose_parts(&self) -> Result<Vec<Mesh>, MeshError> {
        if self.faces.is_empty() {
            return Ok(Vec::new());
        }

        // Map vertices to all incident faces
        let mut vert_to_faces: HashMap<VertexKey, Vec<FaceKey>> = HashMap::new();
        for fk in self.faces.keys() {
            for vk in self.face_vertices(fk) {
                vert_to_faces.entry(vk).or_default().push(fk);
            }
        }

        let mut unvisited_faces: HashSet<FaceKey> = self.faces.keys().collect();
        let mut face_components: Vec<Vec<FaceKey>> = Vec::new();

        while let Some(&seed_face) = unvisited_faces.iter().next() {
            unvisited_faces.remove(&seed_face);
            let mut comp = vec![seed_face];
            let mut queue = std::collections::VecDeque::new();
            queue.push_back(seed_face);

            while let Some(curr_fk) = queue.pop_front() {
                for vk in self.face_vertices(curr_fk) {
                    if let Some(adj_faces) = vert_to_faces.get(&vk) {
                        for &neighbor_fk in adj_faces {
                            if unvisited_faces.remove(&neighbor_fk) {
                                comp.push(neighbor_fk);
                                queue.push_back(neighbor_fk);
                            }
                        }
                    }
                }
            }
            face_components.push(comp);
        }

        if face_components.len() <= 1 {
            return Ok(vec![self.clone()]);
        }

        let mut result = Vec::with_capacity(face_components.len());

        for comp in face_components {
            let mut sub_mesh = Mesh::new();
            let mut old_to_new_v = HashMap::new();

            for &fk in &comp {
                for vk in self.face_vertices(fk) {
                    old_to_new_v.entry(vk).or_insert_with(|| {
                        let v = &self.vertices[vk];
                        let new_vk = sub_mesh.add_vertex_with_uv(v.position, v.uv);
                        new_vk
                    });
                }
            }

            for &fk in &comp {
                let verts = self.face_vertices(fk);
                let new_verts: Vec<VertexKey> = verts.iter().map(|k| old_to_new_v[k]).collect();
                sub_mesh.add_face(&new_verts)?;
            }

            // Transfer vertex groups
            for vg in &self.vertex_groups {
                let mut has_any = false;
                for &vk in old_to_new_v.keys() {
                    if vg.weights.contains_key(&vk) {
                        has_any = true;
                        break;
                    }
                }
                if has_any {
                    let new_vg_idx = sub_mesh.add_vertex_group(&vg.name);
                    for (&old_vk, &new_vk) in &old_to_new_v {
                        if let Some(&w) = vg.weights.get(&old_vk) {
                            sub_mesh.vertex_groups[new_vg_idx].set_weight(new_vk, w);
                        }
                    }
                }
            }

            // Transfer vertex creases
            for (&old_vk, &new_vk) in &old_to_new_v {
                if let Some(old_v) = self.vertices.get(old_vk) {
                    if old_v.crease_weight > 0.0 {
                        sub_mesh.set_vertex_crease(new_vk, old_v.crease_weight);
                    }
                }
            }

            // Transfer edge creases & sharpness
            let new_to_old_v: HashMap<VertexKey, VertexKey> =
                old_to_new_v.iter().map(|(&o, &n)| (n, o)).collect();
            let mut edge_updates = Vec::new();
            for (ek, edge) in &sub_mesh.edges {
                let he = &sub_mesh.half_edges[edge.half_edge];
                let v0_new = he.origin;
                let v1_new = he.target;
                if let (Some(&v0_old), Some(&v1_old)) =
                    (new_to_old_v.get(&v0_new), new_to_old_v.get(&v1_new))
                {
                    if let Some(old_ek) = self.find_edge(v0_old, v1_old) {
                        if let Some(old_edge) = self.edges.get(old_ek) {
                            edge_updates.push((ek, old_edge.crease_weight, old_edge.is_sharp));
                        }
                    }
                }
            }
            for (ek, crease, sharp) in edge_updates {
                sub_mesh.set_edge_crease(ek, crease);
                sub_mesh.set_edge_sharp(ek, sharp);
            }

            sub_mesh.update_normals();
            result.push(sub_mesh);
        }

        Ok(result)
    }

    /// Partitions the mesh into multiple meshes grouped by material slot ID (Blender `P` -> By Material).
    /// Returns a list of `(material_id, Mesh)` pairs, one for each unique material used by faces.
    pub fn separate_by_material(&self) -> Result<Vec<(u32, Mesh)>, MeshError> {
        if self.faces.is_empty() {
            return Ok(Vec::new());
        }

        let mat_ids = self.unique_material_ids();
        if mat_ids.len() <= 1 {
            return Ok(vec![(mat_ids.first().copied().unwrap_or(0), self.clone())]);
        }

        let mut result = Vec::with_capacity(mat_ids.len());

        for mat_id in mat_ids {
            let faces = self.faces_with_material(mat_id);
            if faces.is_empty() {
                continue;
            }

            let mut sub_mesh = Mesh::new();
            let mut old_to_new_v = HashMap::new();

            for &fk in &faces {
                for vk in self.face_vertices(fk) {
                    old_to_new_v.entry(vk).or_insert_with(|| {
                        let v = &self.vertices[vk];
                        let new_vk = sub_mesh.add_vertex_with_uv(v.position, v.uv);
                        new_vk
                    });
                }
            }

            for &fk in &faces {
                let verts = self.face_vertices(fk);
                let new_verts: Vec<VertexKey> = verts.iter().map(|k| old_to_new_v[k]).collect();
                let new_fk = sub_mesh.add_face(&new_verts)?;
                let _ = sub_mesh.set_face_material(new_fk, mat_id);
            }

            // Transfer vertex groups
            for vg in &self.vertex_groups {
                let mut has_any = false;
                for &vk in old_to_new_v.keys() {
                    if vg.weights.contains_key(&vk) {
                        has_any = true;
                        break;
                    }
                }
                if has_any {
                    let new_vg_idx = sub_mesh.add_vertex_group(&vg.name);
                    for (&old_vk, &new_vk) in &old_to_new_v {
                        if let Some(&w) = vg.weights.get(&old_vk) {
                            sub_mesh.vertex_groups[new_vg_idx].set_weight(new_vk, w);
                        }
                    }
                }
            }

            // Transfer vertex creases
            for (&old_vk, &new_vk) in &old_to_new_v {
                if let Some(old_v) = self.vertices.get(old_vk) {
                    if old_v.crease_weight > 0.0 {
                        sub_mesh.set_vertex_crease(new_vk, old_v.crease_weight);
                    }
                }
            }

            // Transfer edge creases & sharpness
            let new_to_old_v: HashMap<VertexKey, VertexKey> =
                old_to_new_v.iter().map(|(&o, &n)| (n, o)).collect();
            let mut edge_updates = Vec::new();
            for (ek, edge) in &sub_mesh.edges {
                let he = &sub_mesh.half_edges[edge.half_edge];
                let v0_new = he.origin;
                let v1_new = he.target;
                if let (Some(&v0_old), Some(&v1_old)) =
                    (new_to_old_v.get(&v0_new), new_to_old_v.get(&v1_new))
                {
                    if let Some(old_ek) = self.find_edge(v0_old, v1_old) {
                        if let Some(old_edge) = self.edges.get(old_ek) {
                            edge_updates.push((ek, old_edge.crease_weight, old_edge.is_sharp));
                        }
                    }
                }
            }
            for (ek, crease, sharp) in edge_updates {
                sub_mesh.set_edge_crease(ek, crease);
                sub_mesh.set_edge_sharp(ek, sharp);
            }

            sub_mesh.update_normals();
            result.push((mat_id, sub_mesh));
        }

        Ok(result)
    }
}

use crate::mesh::{FaceKey, Mesh, MeshError, VertexKey};
use core_math::Vec3;
use std::collections::{HashMap, HashSet};

impl Mesh {
    pub fn weld_vertices_by_distance(&self, threshold: f32) -> Result<Mesh, MeshError> {
        let sq_threshold = threshold * threshold;
        let v_keys: Vec<VertexKey> = self.vertices.keys().collect();
        let n = v_keys.len();

        // Disjoint-set / Union-Find for clustering
        let mut parent: Vec<usize> = (0..n).collect();
        fn find(parent: &mut [usize], i: usize) -> usize {
            if parent[i] == i {
                i
            } else {
                let p = parent[i];
                parent[i] = find(parent, p);
                parent[i]
            }
        }
        fn union(parent: &mut [usize], i: usize, j: usize) {
            let root_i = find(parent, i);
            let root_j = find(parent, j);
            if root_i != root_j {
                parent[root_i] = root_j;
            }
        }

        // Pairwise distance test (for DCC meshes within interactive limits)
        for i in 0..n {
            let p_i = self.vertices[v_keys[i]].position;
            for j in (i + 1)..n {
                let p_j = self.vertices[v_keys[j]].position;
                if (p_i - p_j).length_squared() <= sq_threshold {
                    union(&mut parent, i, j);
                }
            }
        }

        // Group into clusters and compute centroids
        let mut cluster_members: std::collections::HashMap<usize, Vec<usize>> = std::collections::HashMap::new();
        for i in 0..n {
            let root = find(&mut parent, i);
            cluster_members.entry(root).or_default().push(i);
        }

        let mut welded = Mesh::new();
        // Map original VertexKey -> new VertexKey in welded mesh
        let mut old_to_new_v: std::collections::HashMap<VertexKey, VertexKey> = std::collections::HashMap::new();

        for members in cluster_members.values() {
            let mut sum_pos = Vec3::ZERO;
            let mut sum_uv = core_math::Vec2::ZERO;
            for &idx in members {
                let vk = v_keys[idx];
                sum_pos += self.vertices[vk].position;
                sum_uv += self.vertices[vk].uv;
            }
            let count = members.len() as f32;
            let centroid_pos = sum_pos / count;
            let centroid_uv = sum_uv / count;

            let new_vk = welded.add_vertex_with_uv(centroid_pos, centroid_uv);
            for &idx in members {
                old_to_new_v.insert(v_keys[idx], new_vk);
            }
        }

        // Transfer vertex groups with mapped vertex weights
        for vg in &self.vertex_groups {
            let new_vg_idx = welded.add_vertex_group(&vg.name);
            for (&old_vk, &w) in &vg.weights {
                if let Some(&new_vk) = old_to_new_v.get(&old_vk) {
                    let existing_w = welded.vertex_groups[new_vg_idx].get_weight(new_vk);
                    welded.vertex_groups[new_vg_idx].set_weight(new_vk, existing_w.max(w));
                }
            }
        }

        // Rebuild faces with remapped vertices, removing duplicate consecutive vertices
        for (fk, _) in &self.faces {
            let original_verts = self.face_vertices(fk);
            let mut mapped_verts = Vec::with_capacity(original_verts.len());
            for vk in original_verts {
                let new_v = old_to_new_v[&vk];
                if mapped_verts.is_empty() || *mapped_verts.last().unwrap() != new_v {
                    mapped_verts.push(new_v);
                }
            }
            // Also check first and last to prevent boundary duplicate
            if mapped_verts.len() > 1 && mapped_verts.first() == mapped_verts.last() {
                mapped_verts.pop();
            }

            // Only retain valid polygons (at least 3 unique vertices)
            if mapped_verts.len() >= 3 {
                // Check if all vertices are distinct
                let mut distinct = true;
                for i in 0..mapped_verts.len() {
                    for j in (i + 1)..mapped_verts.len() {
                        if mapped_verts[i] == mapped_verts[j] {
                            distinct = false;
                            break;
                        }
                    }
                    if !distinct {
                        break;
                    }
                }
                if distinct {
                    let _ = welded.add_face(&mapped_verts);
                }
            }
        }

        welded.update_normals();
        Ok(welded)
    }

    /// Bridge two equal-length edge loops with bridging quad faces (Blender Bridge Edge Loops).

    pub fn merge_vertices_at_center(&mut self, vertices_to_merge: &[VertexKey]) -> Result<Option<VertexKey>, MeshError> {
        let merge_set: HashSet<VertexKey> = vertices_to_merge.iter().copied().collect();
        if merge_set.len() < 2 {
            return Ok(vertices_to_merge.first().copied());
        }

        // 1. Calculate centroid position and UV
        let mut centroid_pos = Vec3::ZERO;
        let mut centroid_uv = core_math::Vec2::ZERO;
        let mut valid_count = 0.0_f32;

        for &vk in &merge_set {
            if let Some(v) = self.vertices.get(vk) {
                centroid_pos += v.position;
                centroid_uv += v.uv;
                valid_count += 1.0;
            }
        }

        if valid_count < 2.0 {
            return Ok(vertices_to_merge.first().copied());
        }

        centroid_pos /= valid_count;
        centroid_uv /= valid_count;

        let mut new_mesh = Mesh::new();
        let mut old_to_new_v: HashMap<VertexKey, VertexKey> = HashMap::new();

        // Add the single merged vertex
        let merged_vk = new_mesh.add_vertex_with_uv(centroid_pos, centroid_uv);
        for &vk in &merge_set {
            old_to_new_v.insert(vk, merged_vk);
        }

        // Copy all other vertices
        for (vk, v) in &self.vertices {
            if !merge_set.contains(&vk) {
                let n_vk = new_mesh.add_vertex_with_uv(v.position, v.uv);
                old_to_new_v.insert(vk, n_vk);
            }
        }

        // Transfer vertex groups with mapped vertex weights
        for vg in &self.vertex_groups {
            let new_vg_idx = new_mesh.add_vertex_group(&vg.name);
            for (&old_vk, &w) in &vg.weights {
                if let Some(&new_vk) = old_to_new_v.get(&old_vk) {
                    let existing_w = new_mesh.vertex_groups[new_vg_idx].get_weight(new_vk);
                    new_mesh.vertex_groups[new_vg_idx].set_weight(new_vk, existing_w.max(w));
                }
            }
        }

        // Rebuild faces with remapped vertices, removing duplicate consecutive vertices
        for (fk, _) in &self.faces {
            let original_verts = self.face_vertices(fk);
            let mut mapped_verts = Vec::with_capacity(original_verts.len());
            for vk in original_verts {
                let new_v = old_to_new_v[&vk];
                if mapped_verts.is_empty() || *mapped_verts.last().unwrap() != new_v {
                    mapped_verts.push(new_v);
                }
            }
            if mapped_verts.len() > 1 && mapped_verts.first() == mapped_verts.last() {
                mapped_verts.pop();
            }

            // Only retain valid polygons (at least 3 unique vertices)
            if mapped_verts.len() >= 3 {
                let mut distinct = true;
                for i in 0..mapped_verts.len() {
                    for j in (i + 1)..mapped_verts.len() {
                        if mapped_verts[i] == mapped_verts[j] {
                            distinct = false;
                            break;
                        }
                    }
                    if !distinct {
                        break;
                    }
                }
                if distinct {
                    let _ = new_mesh.add_face(&mapped_verts);
                }
            }
        }

        new_mesh.update_normals();
        *self = new_mesh;
        Ok(Some(merged_vk))
    }

    /// Slides vertices of selected edges along adjacent non-selected rail edges by `factor` in [-1.0, 1.0].
    /// Does not alter mesh topology.

    pub fn clean_degenerate_geometry(&mut self, min_area: f32, min_edge_len: f32) -> Result<usize, MeshError> {
        let mut cleaned_count = 0;

        // 1. Weld overlapping vertices
        let welded = self.weld_vertices_by_distance(min_edge_len)?;
        let count = self.vertex_count().saturating_sub(welded.vertex_count());
        *self = welded;
        cleaned_count += count;

        // 2. Identify degenerate faces (area < min_area or < 3 unique vertices)
        let mut degenerate_faces = HashSet::new();
        for fk in self.faces.keys() {
            let vks = self.face_vertices(fk);
            let mut unique_vks = HashSet::new();
            for vk in &vks {
                unique_vks.insert(*vk);
            }
            if unique_vks.len() < 3 {
                degenerate_faces.insert(fk);
            } else if self.face_area(fk) < min_area {
                degenerate_faces.insert(fk);
            }
        }

        if !degenerate_faces.is_empty() {
            cleaned_count += degenerate_faces.len();
            let valid_faces: Vec<FaceKey> = self.faces.keys()
                .filter(|k| !degenerate_faces.contains(k))
                .collect();

            let mut rebuilt = Mesh::new();
            let mut old_to_new_v = HashMap::new();

            for &fk in &valid_faces {
                for vk in self.face_vertices(fk) {
                    if let std::collections::hash_map::Entry::Vacant(e) = old_to_new_v.entry(vk) {
                        let v = &self.vertices[vk];
                        let new_vk = rebuilt.add_vertex_with_uv(v.position, v.uv);
                        e.insert(new_vk);
                    }
                }
            }

            for &fk in &valid_faces {
                let verts = self.face_vertices(fk);
                let new_verts: Vec<VertexKey> = verts.iter().map(|k| old_to_new_v[k]).collect();
                rebuilt.add_face(&new_verts)?;
            }

            // Transfer vertex groups
            for vg in &self.vertex_groups {
                let new_vg_idx = rebuilt.add_vertex_group(&vg.name);
                for (&old_vk, &new_vk) in &old_to_new_v {
                    if let Some(&w) = vg.weights.get(&old_vk) {
                        rebuilt.vertex_groups[new_vg_idx].set_weight(new_vk, w);
                    }
                }
            }

            rebuilt.update_normals();
            *self = rebuilt;
        }

        self.update_normals();
        Ok(cleaned_count)
    }
}

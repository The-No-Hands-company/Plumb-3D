use std::collections::HashSet;

use super::types::{EdgeKey, FaceKey, Mesh, VertexKey};

impl Mesh {
    /// Blender / Maya style Edge Loop Selection.
    /// Traverses quad faces picking opposite edges until boundaries or loop closure.
    pub fn select_edge_loop(&self, start_edge: EdgeKey) -> Vec<EdgeKey> {
        if !self.edges.contains_key(start_edge) {
            return Vec::new();
        }

        let mut loop_edges = Vec::new();
        let mut visited = HashSet::new();
        loop_edges.push(start_edge);
        visited.insert(start_edge);

        let edge = &self.edges[start_edge];
        let he0 = edge.half_edge;
        let he0_twin = self.half_edges[he0].twin;

        // Traverse in direction of half-edge
        let mut curr_he = Some(he0);
        while let Some(he_key) = curr_he {
            curr_he = None;
            let he = &self.half_edges[he_key];
            if let Some(fk) = he.face {
                let f_hes = self.face_half_edges(fk);
                if f_hes.len() == 4 {
                    // Quad face: opposite half-edge is 2 steps away
                    if let Some(idx) = f_hes.iter().position(|&k| k == he_key) {
                        let opp_he_key = f_hes[(idx + 2) % 4];
                        let opp_edge = self.half_edges[opp_he_key].edge;

                        if visited.insert(opp_edge) {
                            loop_edges.push(opp_edge);
                            let opp_twin = self.half_edges[opp_he_key].twin;
                            if self.half_edges[opp_twin].face.is_some() {
                                curr_he = Some(opp_twin);
                            }
                        }
                    }
                }
            }
        }

        // Traverse in direction of twin
        let mut curr_twin = Some(he0_twin);
        while let Some(twin_key) = curr_twin {
            curr_twin = None;
            let he = &self.half_edges[twin_key];
            if let Some(fk) = he.face {
                let f_hes = self.face_half_edges(fk);
                if f_hes.len() == 4 {
                    if let Some(idx) = f_hes.iter().position(|&k| k == twin_key) {
                        let opp_he_key = f_hes[(idx + 2) % 4];
                        let opp_edge = self.half_edges[opp_he_key].edge;

                        if visited.insert(opp_edge) {
                            loop_edges.push(opp_edge);
                            let opp_twin = self.half_edges[opp_he_key].twin;
                            if self.half_edges[opp_twin].face.is_some() {
                                curr_twin = Some(opp_twin);
                            }
                        }
                    }
                }
            }
        }

        loop_edges
    }

    /// Blender / Maya style Edge Ring Selection.
    /// Traverses parallel edges across adjacent quad faces.
    pub fn select_edge_ring(&self, start_edge: EdgeKey) -> Vec<EdgeKey> {
        if !self.edges.contains_key(start_edge) {
            return Vec::new();
        }

        let mut ring_edges = Vec::new();
        let mut visited = HashSet::new();
        ring_edges.push(start_edge);
        visited.insert(start_edge);

        let edge = &self.edges[start_edge];
        let he0 = edge.half_edge;
        let he0_twin = self.half_edges[he0].twin;

        // Traverse through face 1
        let mut curr_he = Some(he0);
        while let Some(he_key) = curr_he {
            curr_he = None;
            let he = &self.half_edges[he_key];
            if let Some(fk) = he.face {
                let f_hes = self.face_half_edges(fk);
                if f_hes.len() == 4 {
                    if let Some(idx) = f_hes.iter().position(|&k| k == he_key) {
                        let opp_he_key = f_hes[(idx + 2) % 4];
                        let opp_edge = self.half_edges[opp_he_key].edge;

                        if visited.insert(opp_edge) {
                            ring_edges.push(opp_edge);
                            let opp_twin = self.half_edges[opp_he_key].twin;
                            if self.half_edges[opp_twin].face.is_some() {
                                curr_he = Some(opp_twin);
                            }
                        }
                    }
                }
            }
        }

        // Traverse through face 2 (twin side)
        let mut curr_twin = Some(he0_twin);
        while let Some(twin_key) = curr_twin {
            curr_twin = None;
            let he = &self.half_edges[twin_key];
            if let Some(fk) = he.face {
                let f_hes = self.face_half_edges(fk);
                if f_hes.len() == 4 {
                    if let Some(idx) = f_hes.iter().position(|&k| k == twin_key) {
                        let opp_he_key = f_hes[(idx + 2) % 4];
                        let opp_edge = self.half_edges[opp_he_key].edge;

                        if visited.insert(opp_edge) {
                            ring_edges.push(opp_edge);
                            let opp_twin = self.half_edges[opp_he_key].twin;
                            if self.half_edges[opp_twin].face.is_some() {
                                curr_twin = Some(opp_twin);
                            }
                        }
                    }
                }
            }
        }

        ring_edges
    }

    /// Blender / Maya style Face Loop / Ring Selection.
    /// Traverses connected quad faces stepping across opposite edges until loop closure or boundary.
    pub fn select_face_loop(&self, start_face: FaceKey) -> Vec<FaceKey> {
        if !self.faces.contains_key(start_face) {
            return Vec::new();
        }
        let f_hes = self.face_half_edges(start_face);
        if f_hes.len() != 4 {
            return vec![start_face];
        }

        let mut loop_faces = Vec::new();
        let mut visited = HashSet::new();
        loop_faces.push(start_face);
        visited.insert(start_face);

        let neighbor_is_quad = |he: super::types::HalfEdgeKey| -> bool {
            let twin = self.half_edges[he].twin;
            if let Some(f) = self.half_edges[twin].face {
                self.face_half_edges(f).len() == 4
            } else {
                false
            }
        };

        let pair_0_2 = (neighbor_is_quad(f_hes[0]) as u8) + (neighbor_is_quad(f_hes[2]) as u8);
        let pair_1_3 = (neighbor_is_quad(f_hes[1]) as u8) + (neighbor_is_quad(f_hes[3]) as u8);
        let directions = if pair_1_3 >= pair_0_2 {
            [f_hes[1], f_hes[3]]
        } else {
            [f_hes[0], f_hes[2]]
        };

        for start_he in directions {
            let mut curr_he = start_he;
            loop {
                let twin = self.half_edges[curr_he].twin;
                let next_face = match self.half_edges[twin].face {
                    Some(f) => f,
                    None => break,
                };
                let next_hes = self.face_half_edges(next_face);
                if next_hes.len() != 4 || visited.contains(&next_face) {
                    break;
                }
                visited.insert(next_face);
                loop_faces.push(next_face);

                if let Some(idx) = next_hes.iter().position(|&h| h == twin) {
                    curr_he = next_hes[(idx + 2) % 4];
                } else {
                    break;
                }
            }
        }

        loop_faces
    }

    /// Expands selected vertices to include all direct 1-ring neighbors.
    pub fn grow_vertex_selection(&self, selected: &HashSet<VertexKey>) -> HashSet<VertexKey> {
        let mut grown = selected.clone();
        for &vk in selected {
            for neighbor in self.vertex_neighbors(vk) {
                grown.insert(neighbor);
            }
        }
        grown
    }

    /// Shrinks selected vertices, retaining only interior vertices whose neighbors are all selected.
    pub fn shrink_vertex_selection(&self, selected: &HashSet<VertexKey>) -> HashSet<VertexKey> {
        let mut shrunk = HashSet::new();
        for &vk in selected {
            let neighbors = self.vertex_neighbors(vk);
            if !neighbors.is_empty() && neighbors.iter().all(|n| selected.contains(n)) {
                shrunk.insert(vk);
            }
        }
        shrunk
    }

    /// Expands selected faces to include any face sharing at least one vertex with a selected face.
    pub fn grow_face_selection(&self, selected: &HashSet<FaceKey>) -> HashSet<FaceKey> {
        let mut grown = selected.clone();
        for &fk in selected {
            for vk in self.face_vertices(fk) {
                for neighbor_face in self.vertex_faces(vk) {
                    grown.insert(neighbor_face);
                }
            }
        }
        grown
    }

    /// Shrinks selected faces, retaining only faces whose neighbor faces are all selected.
    pub fn shrink_face_selection(&self, selected: &HashSet<FaceKey>) -> HashSet<FaceKey> {
        let mut shrunk = HashSet::new();
        for &fk in selected {
            let verts = self.face_vertices(fk);
            let mut all_selected = true;
            for vk in verts {
                for neighbor_face in self.vertex_faces(vk) {
                    if !selected.contains(&neighbor_face) {
                        all_selected = false;
                        break;
                    }
                }
                if !all_selected {
                    break;
                }
            }
            if all_selected {
                shrunk.insert(fk);
            }
        }
        shrunk
    }
}

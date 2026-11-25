use std::collections::{HashSet, VecDeque};

use super::types::{EdgeKey, FaceKey, HalfEdgeKey, Mesh, VertexKey};

/// Comprehensive 2-manifold topological diagnostic report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MeshManifoldReport {
    /// True if the mesh has zero boundary edges and zero non-manifold edges.
    pub is_watertight: bool,
    /// True if every edge is shared by at most 2 faces.
    pub is_2_manifold: bool,
    /// Number of open boundary edges (edges with exactly 1 incident face).
    pub boundary_edge_count: usize,
    /// Number of non-manifold edges (edges with 0 or >2 incident faces).
    pub non_manifold_edges: usize,
    /// Euler-Poincaré characteristic: χ = V - E + F. For closed sphere/cube, χ = 2.
    pub euler_characteristic: i32,
    /// Topological genus (number of holes/handles through the solid): g = (2 - χ) / 2.
    pub genus: i32,
    /// Number of isolated vertices with no incident edges.
    pub isolated_vertices: usize,
}

impl Mesh {
    /// Evaluates 2-manifold topological integrity, boundary hole count, Euler characteristic, and genus.
    pub fn check_manifold(&self) -> MeshManifoldReport {
        let mut boundary_edge_count = 0;
        let mut non_manifold_edges = 0;

        for edge in self.edges.values() {
            let he = &self.half_edges[edge.half_edge];
            let twin = &self.half_edges[he.twin];
            let f1 = he.face.is_some();
            let f2 = twin.face.is_some();

            if f1 ^ f2 {
                boundary_edge_count += 1;
            } else if !f1 && !f2 {
                non_manifold_edges += 1;
            }
        }

        let isolated_vertices = self.vertices.values().filter(|v| v.half_edges.is_empty()).count();
        let v = self.vertices.len() as i32;
        let e = self.edges.len() as i32;
        let f = self.faces.len() as i32;
        let euler = v - e + f;
        let genus = (2 - euler) / 2;

        MeshManifoldReport {
            is_watertight: boundary_edge_count == 0 && non_manifold_edges == 0,
            is_2_manifold: non_manifold_edges == 0,
            boundary_edge_count,
            non_manifold_edges,
            euler_characteristic: euler,
            genus: genus.max(0),
            isolated_vertices,
        }
    }

    /// True CAD / DCC Half-Edge Edge Loop Selection (Blender / Maya / 3ds Max standard).
    /// Traverses through vertices along continuous edge chains in both directions.
    /// At interior 4-valent vertices, steps straight across to the opposite edge (180°).
    /// At open mesh boundaries, follows along boundary loops.
    /// Terminates at poles (valence != 4) and non-quad junctions.
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

        // Trace forward from he0 (towards target)
        self.trace_edge_loop_half(he0, &mut visited, &mut loop_edges);
        // Trace backward from he0_twin (towards origin)
        self.trace_edge_loop_half(he0_twin, &mut visited, &mut loop_edges);

        loop_edges
    }

    fn trace_edge_loop_half(&self, start_he: HalfEdgeKey, visited: &mut HashSet<EdgeKey>, out_edges: &mut Vec<EdgeKey>) {
        let mut incoming = start_he;

        loop {
            let target_v = self.half_edges[incoming].target;
            let is_boundary = self.half_edges[incoming].face.is_none()
                || self.half_edges[self.half_edges[incoming].twin].face.is_none();

            if is_boundary {
                // Follow boundary edge at target_v if another boundary edge meets here
                let mut next_boundary_he = None;
                if let Some(v) = self.vertices.get(target_v) {
                    for &out_he in &v.half_edges {
                        let out_edge = self.half_edges[out_he].edge;
                        if out_edge != self.half_edges[incoming].edge {
                            let out_is_boundary = self.half_edges[out_he].face.is_none()
                                || self.half_edges[self.half_edges[out_he].twin].face.is_none();
                            if out_is_boundary {
                                next_boundary_he = Some(out_he);
                                break;
                            }
                        }
                    }
                }

                if let Some(next_he) = next_boundary_he {
                    let next_edge = self.half_edges[next_he].edge;
                    if visited.insert(next_edge) {
                        out_edges.push(next_edge);
                        incoming = next_he;
                        continue;
                    }
                }
                break;
            }

            // Interior vertex traversal: check cyclic fan of outgoing half-edges around target_v
            let h_back = self.half_edges[incoming].twin;
            let mut fan = Vec::new();
            let mut curr = h_back;

            for _ in 0..16 {
                fan.push(curr);
                let twin = self.half_edges[curr].twin;
                let next = self.half_edges[twin].next;
                if next == h_back || !self.half_edges.contains_key(next) {
                    break;
                }
                curr = next;
            }

            // Exactly 4 incident edges at quad junction: opposite edge is index 2
            if fan.len() == 4 {
                let opp_he = fan[2];
                let opp_edge = self.half_edges[opp_he].edge;
                if visited.insert(opp_edge) {
                    out_edges.push(opp_edge);
                    incoming = opp_he;
                    continue;
                }
            }

            break;
        }
    }

    /// Blender / Maya style Edge Ring Selection.
    /// Traverses parallel edges across adjacent quad faces in both directions.
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

        self.trace_edge_ring_half(he0, &mut visited, &mut ring_edges);
        self.trace_edge_ring_half(he0_twin, &mut visited, &mut ring_edges);

        ring_edges
    }

    fn trace_edge_ring_half(&self, start_he: HalfEdgeKey, visited: &mut HashSet<EdgeKey>, out_edges: &mut Vec<EdgeKey>) {
        let mut curr_he = Some(start_he);

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
                            out_edges.push(opp_edge);
                            let opp_twin = self.half_edges[opp_he_key].twin;
                            if self.half_edges[opp_twin].face.is_some() {
                                curr_he = Some(opp_twin);
                            }
                        }
                    }
                }
            }
        }
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

        let neighbor_is_quad = |he: HalfEdgeKey| -> bool {
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

    /// CAD Precision & Hard-Surface Coplanar Face Selection (Blender "Select Linked Flat Faces").
    /// Flood-fills across shared edges of adjacent faces whose normal angular deviation
    /// is within `tolerance_deg` degrees.
    pub fn select_coplanar_faces(&self, start_face: FaceKey, tolerance_deg: f32) -> Vec<FaceKey> {
        if !self.faces.contains_key(start_face) {
            return Vec::new();
        }
        let start_normal = self.faces[start_face].normal;
        let cos_thresh = tolerance_deg.to_radians().cos();
        let mut result = Vec::new();
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        queue.push_back(start_face);
        visited.insert(start_face);

        while let Some(curr_fk) = queue.pop_front() {
            result.push(curr_fk);
            for he_key in self.face_half_edges(curr_fk) {
                let twin_key = self.half_edges[he_key].twin;
                if let Some(adj_fk) = self.half_edges[twin_key].face {
                    if visited.insert(adj_fk) {
                        if let Some(adj_f) = self.faces.get(adj_fk) {
                            if adj_f.normal.dot(start_normal) >= cos_thresh - 1e-4 {
                                queue.push_back(adj_fk);
                            }
                        }
                    }
                }
            }
        }
        result
    }

    /// Selects all topologically connected faces (entire mesh island / continuous shell).
    pub fn select_linked(&self, start_face: FaceKey) -> Vec<FaceKey> {
        if !self.faces.contains_key(start_face) {
            return Vec::new();
        }
        let mut result = Vec::new();
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        queue.push_back(start_face);
        visited.insert(start_face);

        while let Some(curr_fk) = queue.pop_front() {
            result.push(curr_fk);
            for vk in self.face_vertices(curr_fk) {
                for adj_fk in self.vertex_faces(vk) {
                    if visited.insert(adj_fk) {
                        queue.push_back(adj_fk);
                    }
                }
            }
        }
        result
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

    /// Expands selected edges to include all edges incident to the endpoint vertices.
    pub fn grow_edge_selection(&self, selected: &HashSet<EdgeKey>) -> HashSet<EdgeKey> {
        let mut grown = selected.clone();
        for &ek in selected {
            if let Some(edge) = self.edges.get(ek) {
                let he = &self.half_edges[edge.half_edge];
                for &vk in &[he.origin, he.target] {
                    if let Some(v) = self.vertices.get(vk) {
                        for &out_he in &v.half_edges {
                            grown.insert(self.half_edges[out_he].edge);
                        }
                    }
                }
            }
        }
        grown
    }

    /// Shrinks selected edges, retaining only interior edges where all neighboring edges
    /// at both endpoint vertices are also selected.
    pub fn shrink_edge_selection(&self, selected: &HashSet<EdgeKey>) -> HashSet<EdgeKey> {
        let mut shrunk = HashSet::new();
        for &ek in selected {
            if let Some(edge) = self.edges.get(ek) {
                let he = &self.half_edges[edge.half_edge];
                let mut is_interior = true;
                for &vk in &[he.origin, he.target] {
                    if let Some(v) = self.vertices.get(vk) {
                        for &out_he in &v.half_edges {
                            let adj_ek = self.half_edges[out_he].edge;
                            if !selected.contains(&adj_ek) {
                                is_interior = false;
                                break;
                            }
                        }
                    }
                    if !is_interior {
                        break;
                    }
                }
                if is_interior {
                    shrunk.insert(ek);
                }
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

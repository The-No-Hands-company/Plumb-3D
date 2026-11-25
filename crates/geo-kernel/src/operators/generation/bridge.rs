use crate::mesh::{EdgeKey, FaceKey, Mesh, MeshError, VertexKey};
use std::collections::{HashMap, HashSet};

impl Mesh {
    /// Bridges two closed or open vertex loops with quad faces.
    pub fn bridge_edge_loops(
        &mut self,
        loop_a: &[VertexKey],
        loop_b: &[VertexKey],
    ) -> Result<Vec<FaceKey>, MeshError> {
        self.bridge_edge_loops_advanced(loop_a, loop_b, 0, 0)
    }

    /// Advanced bridge operator supporting rotational twist and intermediate subdivision cuts.
    pub fn bridge_edge_loops_advanced(
        &mut self,
        loop_a: &[VertexKey],
        loop_b: &[VertexKey],
        twist: i32,
        cuts: usize,
    ) -> Result<Vec<FaceKey>, MeshError> {
        let n = loop_a.len();
        if n < 3 || loop_b.len() != n {
            return Err(MeshError::InvariantFailed(
                "Bridging requires two loops of equal length >= 3".into(),
            ));
        }

        // Apply twist to loop_b alignment
        let twisted_b: Vec<VertexKey> = (0..n)
            .map(|i| {
                let shifted = (i as isize + twist as isize).rem_euclid(n as isize) as usize;
                loop_b[shifted]
            })
            .collect();

        // Build intermediate rings if cuts > 0
        let mut rings: Vec<Vec<VertexKey>> = Vec::with_capacity(cuts + 2);
        rings.push(loop_a.to_vec());

        if cuts > 0 {
            for s in 1..=cuts {
                let t = s as f32 / (cuts + 1) as f32;
                let mut intermediate_ring = Vec::with_capacity(n);
                for i in 0..n {
                    let va = loop_a[i];
                    let vb = twisted_b[i];
                    let pa = self.vertices[va].position;
                    let pb = self.vertices[vb].position;
                    let uva = self.vertices[va].uv;
                    let uvb = self.vertices[vb].uv;

                    let pos = pa * (1.0 - t) + pb * t;
                    let uv = uva * (1.0 - t) + uvb * t;
                    let new_vk = self.add_vertex_with_uv(pos, uv);
                    intermediate_ring.push(new_vk);
                }
                rings.push(intermediate_ring);
            }
        }

        rings.push(twisted_b);

        let mut created_faces = Vec::new();
        for r in 0..rings.len() - 1 {
            let r_curr = &rings[r];
            let r_next = &rings[r + 1];

            for i in 0..n {
                let a_curr = r_curr[i];
                let a_next = r_curr[(i + 1) % n];
                let b_next = r_next[(i + 1) % n];
                let b_curr = r_next[i];

                let fk = self.add_face(&[a_curr, a_next, b_next, b_curr])?;
                created_faces.push(fk);
            }
        }

        self.update_normals();
        Ok(created_faces)
    }

    /// Extracts continuous ordered vertex cycles or chains from a selection of edges.
    pub fn extract_loops_from_edges(&self, edges: &[EdgeKey]) -> Vec<Vec<VertexKey>> {
        if edges.is_empty() {
            return Vec::new();
        }

        // Build adjacency from the specified edges
        let mut adj: HashMap<VertexKey, Vec<VertexKey>> = HashMap::new();
        let mut edge_pairs: HashSet<(VertexKey, VertexKey)> = HashSet::new();

        for &ek in edges {
            if let Some(edge) = self.edges.get(ek) {
                let he = &self.half_edges[edge.half_edge];
                let twin = &self.half_edges[he.twin];
                let v0 = twin.origin;
                let v1 = he.origin;
                adj.entry(v0).or_default().push(v1);
                adj.entry(v1).or_default().push(v0);
                edge_pairs.insert((v0.min(v1), v0.max(v1)));
            }
        }

        let mut visited_edges: HashSet<(VertexKey, VertexKey)> = HashSet::new();
        let mut loops: Vec<Vec<VertexKey>> = Vec::new();

        // 1. First, trace any open chains (vertices with degree 1 in selection)
        let endpoints: Vec<VertexKey> = adj
            .iter()
            .filter(|(_, neighbors)| neighbors.len() == 1)
            .map(|(&v, _)| v)
            .collect();

        let mut visited_chain_starts: HashSet<VertexKey> = HashSet::new();

        for ep in endpoints {
            if visited_chain_starts.contains(&ep) {
                continue;
            }

            let mut chain = Vec::new();
            let mut curr = ep;
            let mut prev: Option<VertexKey> = None;
            chain.push(curr);
            visited_chain_starts.insert(curr);

            loop {
                let neighbors = adj.get(&curr).cloned().unwrap_or_default();
                let next = neighbors.into_iter().find(|&n| {
                    if Some(n) == prev {
                        return false;
                    }
                    let e_pair = (curr.min(n), curr.max(n));
                    !visited_edges.contains(&e_pair)
                });

                match next {
                    Some(n) => {
                        let e_pair = (curr.min(n), curr.max(n));
                        visited_edges.insert(e_pair);
                        prev = Some(curr);
                        curr = n;
                        chain.push(curr);
                        if adj.get(&curr).map_or(0, |x| x.len()) == 1 {
                            visited_chain_starts.insert(curr);
                            break;
                        }
                    }
                    None => break,
                }
            }

            if chain.len() >= 2 {
                loops.push(chain);
            }
        }

        // 2. Trace closed cycles
        for (&start_v, neighbors) in &adj {
            for &n in neighbors {
                let e_pair = (start_v.min(n), start_v.max(n));
                if visited_edges.contains(&e_pair) {
                    continue;
                }

                let mut cycle = Vec::new();
                let mut curr = start_v;
                let mut next_v = n;
                cycle.push(curr);
                visited_edges.insert(e_pair);

                loop {
                    curr = next_v;
                    if curr == start_v {
                        break;
                    }
                    cycle.push(curr);

                    let cand_neighbors = adj.get(&curr).cloned().unwrap_or_default();
                    let step = cand_neighbors.into_iter().find(|&candidate| {
                        let step_pair = (curr.min(candidate), curr.max(candidate));
                        !visited_edges.contains(&step_pair)
                    });

                    match step {
                        Some(s) => {
                            let step_pair = (curr.min(s), curr.max(s));
                            visited_edges.insert(step_pair);
                            next_v = s;
                        }
                        None => break,
                    }
                }

                if cycle.len() >= 3 {
                    loops.push(cycle);
                }
            }
        }

        loops
    }

    /// Automatically bridges two detected edge loops from selected edges.
    pub fn bridge_selected_edges(
        &mut self,
        selected_edges: &[EdgeKey],
        twist: i32,
        cuts: usize,
    ) -> Result<Vec<FaceKey>, MeshError> {
        let loops = self.extract_loops_from_edges(selected_edges);
        if loops.len() != 2 {
            return Err(MeshError::InvariantFailed(format!(
                "Bridge requires exactly 2 loops from selected edges, found {}",
                loops.len()
            )));
        }

        self.bridge_edge_loops_advanced(&loops[0], &loops[1], twist, cuts)
    }
}

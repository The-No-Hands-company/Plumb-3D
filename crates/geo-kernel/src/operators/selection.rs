use crate::mesh::{EdgeKey, Mesh, VertexKey};
use std::collections::{HashMap, HashSet};

use std::collections::BinaryHeap;

impl Mesh {
    pub fn is_edge_boundary(&self, edge_key: EdgeKey) -> bool {
        if let Some(edge) = self.edges.get(edge_key) {
            let he = &self.half_edges[edge.half_edge];
            let twin = &self.half_edges[he.twin];
            he.face.is_none() ^ twin.face.is_none()
        } else {
            false
        }
    }

    /// Detects all open boundary loops (holes) in the mesh.
    /// Each loop is an ordered cycle of vertex keys oriented such that
    /// creating a cap polygon with this ordering faces outward.
    pub fn boundary_loops(&self) -> Vec<Vec<VertexKey>> {
        let mut next_v: HashMap<VertexKey, VertexKey> = HashMap::new();

        for edge in self.edges.values() {
            let he = &self.half_edges[edge.half_edge];
            let twin = &self.half_edges[he.twin];
            if he.face.is_some() && twin.face.is_none() {
                next_v.insert(he.origin, he.target);
            } else if he.face.is_none() && twin.face.is_some() {
                next_v.insert(twin.origin, twin.target);
            }
        }

        let mut visited: HashSet<VertexKey> = HashSet::new();
        let mut loops: Vec<Vec<VertexKey>> = Vec::new();

        for &start_vk in next_v.keys() {
            if visited.contains(&start_vk) {
                continue;
            }

            let mut current_loop = Vec::new();
            let mut curr = start_vk;

            while !visited.contains(&curr) {
                visited.insert(curr);
                current_loop.push(curr);
                if let Some(&next) = next_v.get(&curr) {
                    if next == start_vk {
                        break;
                    }
                    curr = next;
                } else {
                    break;
                }
            }

            if current_loop.len() >= 3 {
                current_loop.reverse();
                loops.push(current_loop);
            }
        }

        loops
    }

    /// Caps all open boundary holes in the mesh (3ds Max `Cap Holes` / Blender `F` Fill).
    /// Generates outward-facing cap polygons, optionally triangulating them.

    pub fn shortest_path(&self, start: VertexKey, end: VertexKey) -> Option<Vec<VertexKey>> {
        if start == end {
            return Some(vec![start]);
        }
        if !self.vertices.contains_key(start) || !self.vertices.contains_key(end) {
            return None;
        }

        use std::cmp::Ordering;

        #[derive(Copy, Clone, PartialEq)]
        struct State {
            cost: f32,
            vertex: VertexKey,
        }

        impl Eq for State {}

        impl Ord for State {
            fn cmp(&self, other: &Self) -> Ordering {
                other.cost.partial_cmp(&self.cost).unwrap_or(Ordering::Equal)
            }
        }

        impl PartialOrd for State {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                Some(self.cmp(other))
            }
        }

        let mut dist: HashMap<VertexKey, f32> = HashMap::new();
        let mut prev: HashMap<VertexKey, VertexKey> = HashMap::new();
        let mut heap: BinaryHeap<State> = BinaryHeap::new();

        dist.insert(start, 0.0);
        heap.push(State { cost: 0.0, vertex: start });

        while let Some(State { cost, vertex }) = heap.pop() {
            if vertex == end {
                let mut path = Vec::new();
                let mut curr = end;
                path.push(curr);
                while let Some(&p) = prev.get(&curr) {
                    path.push(p);
                    curr = p;
                    if curr == start {
                        break;
                    }
                }
                path.reverse();
                return Some(path);
            }

            if let Some(&best) = dist.get(&vertex) {
                if cost > best + 1e-5 {
                    continue;
                }
            }

            let curr_pos = self.vertices[vertex].position;

            for &he_key in &self.vertices[vertex].half_edges {
                let he = &self.half_edges[he_key];
                let neighbor = if he.origin == vertex { he.target } else { he.origin };
                if let Some(n_vert) = self.vertices.get(neighbor) {
                    let d = (n_vert.position - curr_pos).length();
                    let next_cost = cost + d;
                    let is_better = match dist.get(&neighbor) {
                        Some(&cur_d) => next_cost < cur_d,
                        None => true,
                    };
                    if is_better {
                        dist.insert(neighbor, next_cost);
                        prev.insert(neighbor, vertex);
                        heap.push(State { cost: next_cost, vertex: neighbor });
                    }
                }
            }
        }

        None
    }
}

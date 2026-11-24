use crate::mesh::{Mesh, MeshError};
use crate::modifiers::Modifier;
use core_math::{Vec2, Vec3};
use std::collections::BinaryHeap;
use std::cmp::Ordering;

/// Symmetric 4x4 matrix for Garland-Heckbert Quadric Error Metrics
#[derive(Debug, Clone, Copy, Default)]
pub struct SymmQuadric {
    // Upper-triangular elements:
    // [0] = a00, [1] = a01, [2] = a02, [3] = a03
    // [4] = a11, [5] = a12, [6] = a13
    // [7] = a22, [8] = a23
    // [9] = a33
    pub data: [f32; 10],
}

impl SymmQuadric {
    pub fn zero() -> Self {
        Self { data: [0.0; 10] }
    }

    pub fn from_plane(a: f32, b: f32, c: f32, d: f32) -> Self {
        Self {
            data: [
                a * a, a * b, a * c, a * d,
                b * b, b * c, b * d,
                c * c, c * d,
                d * d,
            ],
        }
    }

    pub fn add(&self, other: &SymmQuadric) -> Self {
        let mut res = [0.0; 10];
        for i in 0..10 {
            res[i] = self.data[i] + other.data[i];
        }
        Self { data: res }
    }

    /// Evaluate error v^T * Q * v where v = [x, y, z, 1]
    pub fn evaluate(&self, p: Vec3) -> f32 {
        let x = p.x;
        let y = p.y;
        let z = p.z;
        let d = &self.data;

        (d[0] * x * x + 2.0 * d[1] * x * y + 2.0 * d[2] * x * z + 2.0 * d[3] * x
            + d[4] * y * y + 2.0 * d[5] * y * z + 2.0 * d[6] * y
            + d[7] * z * z + 2.0 * d[8] * z
            + d[9]).max(0.0)
    }
}

#[derive(Debug, Clone, Copy)]
struct CandidateCollapse {
    cost: f32,
    v0: usize,
    v1: usize,
    target_pos: Vec3,
    target_uv: Vec2,
}

impl PartialEq for CandidateCollapse {
    fn eq(&self, other: &Self) -> bool {
        self.cost == other.cost
    }
}

impl Eq for CandidateCollapse {}

impl PartialOrd for CandidateCollapse {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // Min-heap ordering (smallest cost has highest priority)
        other.cost.partial_cmp(&self.cost)
    }
}

impl Ord for CandidateCollapse {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Equal)
    }
}

/// Simplify a polygonal mesh using Quadric Error Metric (QEM) edge collapses.
/// `ratio` is in (0.0, 1.0], where 1.0 preserves original complexity and 0.5 targets 50% faces.
pub fn decimate_mesh(input: &Mesh, ratio: f32) -> Result<Mesh, MeshError> {
    if ratio >= 1.0 || input.face_count() == 0 {
        return Ok(input.clone());
    }

    let clamped_ratio = ratio.clamp(0.01, 1.0);

    // 1. Extract input geometry into simple indexed triangles
    let mut positions: Vec<Vec3> = Vec::with_capacity(input.vertex_count());
    let mut uvs: Vec<Vec2> = Vec::with_capacity(input.vertex_count());
    let mut vk_to_idx = std::collections::HashMap::new();

    for (vk, v) in &input.vertices {
        let idx = positions.len();
        positions.push(v.position);
        uvs.push(v.uv);
        vk_to_idx.insert(vk, idx);
    }

    let mut triangles: Vec<[usize; 3]> = Vec::new();
    for (fk, _face) in &input.faces {
        let loop_verts = input.face_vertices(fk);
        if loop_verts.len() < 3 {
            continue;
        }
        // Fan triangulation for polygons
        let v0 = vk_to_idx[&loop_verts[0]];
        for i in 1..(loop_verts.len() - 1) {
            let v1 = vk_to_idx[&loop_verts[i]];
            let v2 = vk_to_idx[&loop_verts[i + 1]];
            triangles.push([v0, v1, v2]);
        }
    }

    let target_triangle_count = ((input.face_count() as f32) * clamped_ratio).max(4.0) as usize;
    if triangles.len() <= target_triangle_count {
        return Ok(input.clone());
    }

    // 2. Compute initial quadrics for each vertex
    let mut quadrics = vec![SymmQuadric::zero(); positions.len()];
    for tri in &triangles {
        let p0 = positions[tri[0]];
        let p1 = positions[tri[1]];
        let p2 = positions[tri[2]];
        let normal = (p1 - p0).cross(p2 - p0);
        let len = normal.length();
        if len > 1e-6 {
            let n = normal / len;
            let d = -n.dot(p0);
            let q = SymmQuadric::from_plane(n.x, n.y, n.z, d);
            quadrics[tri[0]] = quadrics[tri[0]].add(&q);
            quadrics[tri[1]] = quadrics[tri[1]].add(&q);
            quadrics[tri[2]] = quadrics[tri[2]].add(&q);
        }
    }

    // Track active vertices and redirection for collapsed vertices (Disjoint-Set / Union-Find)
    let mut parent: Vec<usize> = (0..positions.len()).collect();

    fn find(parent: &mut [usize], mut i: usize) -> usize {
        while i != parent[i] {
            parent[i] = parent[parent[i]];
            i = parent[i];
        }
        i
    }

    // Helper: evaluate collapse candidate between two vertices
    let eval_edge = |u: usize, v: usize, quadrics: &[SymmQuadric], positions: &[Vec3], uvs: &[Vec2]| -> CandidateCollapse {
        let q = quadrics[u].add(&quadrics[v]);
        let p0 = positions[u];
        let p1 = positions[v];
        let p_mid = (p0 + p1) * 0.5;

        let err0 = q.evaluate(p0);
        let err1 = q.evaluate(p1);
        let err_mid = q.evaluate(p_mid);

        let (best_pos, best_uv, cost) = if err_mid <= err0 && err_mid <= err1 {
            (p_mid, (uvs[u] + uvs[v]) * 0.5, err_mid)
        } else if err0 <= err1 {
            (p0, uvs[u], err0)
        } else {
            (p1, uvs[v], err1)
        };

        CandidateCollapse {
            cost,
            v0: u,
            v1: v,
            target_pos: best_pos,
            target_uv: best_uv,
        }
    };

    // 3. Build priority queue of candidate collapses
    let mut heap = BinaryHeap::new();
    let mut seen_edges = std::collections::HashSet::new();

    for tri in &triangles {
        let edges = [(tri[0], tri[1]), (tri[1], tri[2]), (tri[2], tri[0])];
        for (a, b) in edges {
            let pair = if a < b { (a, b) } else { (b, a) };
            if seen_edges.insert(pair) {
                heap.push(eval_edge(pair.0, pair.1, &quadrics, &positions, &uvs));
            }
        }
    }

    let mut current_triangle_count = triangles.len();

    // 4. Iteratively collapse edges
    while current_triangle_count > target_triangle_count && !heap.is_empty() {
        let cand = heap.pop().unwrap();
        let root0 = find(&mut parent, cand.v0);
        let root1 = find(&mut parent, cand.v1);

        if root0 == root1 {
            continue;
        }

        // Check for normal inversion / triangle flipping
        let mut causes_flip = false;
        for tri in &triangles {
            let t0 = find(&mut parent, tri[0]);
            let t1 = find(&mut parent, tri[1]);
            let t2 = find(&mut parent, tri[2]);

            // Skip degenerate triangles that will be removed by this collapse
            let is_incident = (t0 == root0 || t0 == root1)
                || (t1 == root0 || t1 == root1)
                || (t2 == root0 || t2 == root1);

            if !is_incident {
                continue;
            }

            let new_t0 = if t0 == root1 { root0 } else { t0 };
            let new_t1 = if t1 == root1 { root0 } else { t1 };
            let new_t2 = if t2 == root1 { root0 } else { t2 };

            if new_t0 == new_t1 || new_t1 == new_t2 || new_t2 == new_t0 {
                continue; // degenerate after collapse, fine
            }

            // Old normal vs new normal
            let p0 = positions[t0];
            let p1 = positions[t1];
            let p2 = positions[t2];
            let old_norm = (p1 - p0).cross(p2 - p0);

            let np0 = if t0 == root0 || t0 == root1 { cand.target_pos } else { positions[t0] };
            let np1 = if t1 == root0 || t1 == root1 { cand.target_pos } else { positions[t1] };
            let np2 = if t2 == root0 || t2 == root1 { cand.target_pos } else { positions[t2] };
            let new_norm = (np1 - np0).cross(np2 - np0);

            if old_norm.length() > 1e-6 && new_norm.length() > 1e-6 {
                if old_norm.dot(new_norm) <= 0.0 {
                    causes_flip = true;
                    break;
                }
            }
        }

        if causes_flip {
            continue;
        }

        // Count how many triangles collapse
        let mut removed = 0;
        for tri in &triangles {
            let t0 = find(&mut parent, tri[0]);
            let t1 = find(&mut parent, tri[1]);
            let t2 = find(&mut parent, tri[2]);
            let has_r0 = t0 == root0 || t1 == root0 || t2 == root0;
            let has_r1 = t0 == root1 || t1 == root1 || t2 == root1;
            if has_r0 && has_r1 {
                removed += 1;
            }
        }

        // Perform collapse: merge root1 into root0
        parent[root1] = root0;
        positions[root0] = cand.target_pos;
        uvs[root0] = cand.target_uv;
        quadrics[root0] = quadrics[root0].add(&quadrics[root1]);

        current_triangle_count = current_triangle_count.saturating_sub(removed);
    }

    // 5. Reconstruct simplified Half-Edge Mesh
    let mut out_mesh = Mesh::new();
    let mut old_to_new = std::collections::HashMap::new();

    // Collect surviving triangles
    let mut surviving_triangles = Vec::new();
    for tri in &triangles {
        let t0 = find(&mut parent, tri[0]);
        let t1 = find(&mut parent, tri[1]);
        let t2 = find(&mut parent, tri[2]);

        if t0 != t1 && t1 != t2 && t2 != t0 {
            surviving_triangles.push([t0, t1, t2]);
        }
    }

    // Dedup duplicate or opposing duplicate faces
    let mut face_set = std::collections::HashSet::new();
    for tri in surviving_triangles {
        let mut sorted = tri;
        sorted.sort_unstable();
        if face_set.insert(sorted) {
            let vk0 = *old_to_new.entry(tri[0]).or_insert_with(|| {
                out_mesh.add_vertex_with_uv(positions[tri[0]], uvs[tri[0]])
            });
            let vk1 = *old_to_new.entry(tri[1]).or_insert_with(|| {
                out_mesh.add_vertex_with_uv(positions[tri[1]], uvs[tri[1]])
            });
            let vk2 = *old_to_new.entry(tri[2]).or_insert_with(|| {
                out_mesh.add_vertex_with_uv(positions[tri[2]], uvs[tri[2]])
            });

            let _ = out_mesh.add_face(&[vk0, vk1, vk2]);
        }
    }

    out_mesh.update_normals();
    Ok(out_mesh)
}

/// 3ds Max / Blender / Maya style Decimate (Simplify) modifier
#[derive(Debug, Clone)]
pub struct DecimateModifier {
    pub ratio: f32,
    pub enabled: bool,
}

impl DecimateModifier {
    pub fn new(ratio: f32) -> Self {
        Self {
            ratio,
            enabled: true,
        }
    }
}

impl Modifier for DecimateModifier {
    fn name(&self) -> &str {
        "Decimate"
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn apply(&self, input: &Mesh) -> Result<Mesh, MeshError> {
        decimate_mesh(input, self.ratio)
    }

    fn as_decimate_mut(&mut self) -> Option<&mut DecimateModifier> {
        Some(self)
    }
}

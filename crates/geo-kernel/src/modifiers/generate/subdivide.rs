use crate::mesh::{Mesh, MeshError};
use crate::modifiers::stack::Modifier;
use core_math::Vec3;
use std::collections::HashMap;

/// Subdivision Surface modifier (Loop / Catmull-Clark subdivision for quads and triangles)
#[derive(Debug, Clone)]
pub struct SubdivideModifier {
    pub iterations: u32,
    pub enabled: bool,
}

impl SubdivideModifier {
    pub fn new(iterations: u32) -> Self {
        Self {
            iterations,
            enabled: true,
        }
    }

    fn subdivide_once(mesh: &Mesh) -> Result<Mesh, MeshError> {
        let mut sub = Mesh::new();

        // 1. Copy original vertices to new mesh and record mapping
        let mut old_to_new_v = HashMap::new();
        for (vk, v) in &mesh.vertices {
            let new_vk = sub.add_vertex_with_uv(v.position, v.uv);
            old_to_new_v.insert(vk, new_vk);
        }

        // 2. Compute edge midpoints
        // Map (v_min, v_max) -> new midpoint VertexKey
        let mut edge_midpoints = HashMap::new();
        for (_, edge) in &mesh.edges {
            let he = &mesh.half_edges[edge.half_edge];
            let v0 = he.origin;
            let v1 = he.target;
            let pair = if v0 < v1 { (v0, v1) } else { (v1, v0) };

            edge_midpoints.entry(pair).or_insert_with(|| {
                let p0 = mesh.vertices[v0].position;
                let p1 = mesh.vertices[v1].position;
                let uv0 = mesh.vertices[v0].uv;
                let uv1 = mesh.vertices[v1].uv;

                let mid_pos = (p0 + p1) * 0.5;
                let mid_uv = (uv0 + uv1) * 0.5;
                let mid_vk = sub.add_vertex_with_uv(mid_pos, mid_uv);
                mid_vk
            });
        }

        // 3. For each face:
        //    - Compute face centroid vertex
        //    - Create a quad for each corner
        for (fk, _) in &mesh.faces {
            let v_keys = mesh.face_vertices(fk);
            let n = v_keys.len();
            if n < 3 {
                continue;
            }

            // Face center position & UV
            let mut center_pos = Vec3::ZERO;
            let mut center_uv = core_math::Vec2::ZERO;
            for &vk in &v_keys {
                center_pos += mesh.vertices[vk].position;
                center_uv += mesh.vertices[vk].uv;
            }
            center_pos /= n as f32;
            center_uv /= n as f32;

            let face_center_vk = sub.add_vertex_with_uv(center_pos, center_uv);

            // Subdivide into n quads:
            // For corner i: [old_v, edge_mid_next, face_center, edge_mid_prev]
            for i in 0..n {
                let v_curr = v_keys[i];
                let v_prev = v_keys[(i + n - 1) % n];
                let v_next = v_keys[(i + 1) % n];

                let pair_next = if v_curr < v_next { (v_curr, v_next) } else { (v_next, v_curr) };
                let pair_prev = if v_prev < v_curr { (v_prev, v_curr) } else { (v_curr, v_prev) };

                let new_curr = old_to_new_v[&v_curr];
                let mid_next = edge_midpoints[&pair_next];
                let mid_prev = edge_midpoints[&pair_prev];

                sub.add_face(&[new_curr, mid_next, face_center_vk, mid_prev])?;
            }
        }

        sub.update_normals();
        Ok(sub)
    }
}

impl Modifier for SubdivideModifier {
    fn name(&self) -> &str {
        "Subdivide"
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn apply(&self, input: &Mesh) -> Result<Mesh, MeshError> {
        let mut result = input.clone();
        for _ in 0..self.iterations {
            result = Self::subdivide_once(&result)?;
        }
        Ok(result)
    }
}

use crate::mesh::{Mesh, MeshError};
use core_math::{vec2, Vec3};
use std::collections::HashMap;

impl Mesh {
    /// Procedural Skin Envelope Generator (Blender Skin Modifier / ZBrush ZSpheres).
    /// Constructs an envelope of quad sleeves around arbitrary wireframe edge armatures.
    pub fn generate_skin_envelope(&self, default_radius: f32, smooth_iterations: usize) -> Result<Mesh, MeshError> {
        if self.edges.is_empty() {
            return Ok(self.clone());
        }

        let radius = default_radius.max(0.01);
        let mut out_mesh = Mesh::new();

        // 1. Calculate vertex degrees to identify leaf tips vs branching joints
        let mut degree: HashMap<crate::mesh::VertexKey, usize> = HashMap::new();
        for edge in self.edges.values() {
            let he = &self.half_edges[edge.half_edge];
            *degree.entry(he.origin).or_default() += 1;
            *degree.entry(he.target).or_default() += 1;
        }

        // 2. Generate quad sleeves along each edge
        for edge in self.edges.values() {
            let he = &self.half_edges[edge.half_edge];
            let p0 = self.vertices[he.origin].position;
            let p1 = self.vertices[he.target].position;
            let dir = p1 - p0;
            let len = dir.length();
            if len < 1e-4 {
                continue;
            }
            let d = dir / len;

            // Compute orthogonal frame (U, V) perpendicular to edge direction
            let u = if d.y.abs() < 0.9 {
                d.cross(Vec3::Y).normalize()
            } else {
                d.cross(Vec3::Z).normalize()
            };
            let v = d.cross(u).normalize();

            // 4 offset vectors around the tube
            let offsets = [
                (u + v) * 0.707 * radius,
                (-u + v) * 0.707 * radius,
                (-u - v) * 0.707 * radius,
                (u - v) * 0.707 * radius,
            ];

            // Add ring vertices at p0
            let r0: Vec<_> = offsets
                .iter()
                .enumerate()
                .map(|(idx, &off)| {
                    out_mesh.add_vertex_with_uv(p0 + off, vec2(idx as f32 / 4.0, 0.0))
                })
                .collect();

            // Add ring vertices at p1
            let r1: Vec<_> = offsets
                .iter()
                .enumerate()
                .map(|(idx, &off)| {
                    out_mesh.add_vertex_with_uv(p1 + off, vec2(idx as f32 / 4.0, 1.0))
                })
                .collect();

            // 4 Quad sleeve faces connecting r0 and r1
            for i in 0..4 {
                let next_i = (i + 1) % 4;
                let _ = out_mesh.add_face(&[r0[i], r0[next_i], r1[next_i], r1[i]]);
            }

            // End caps if endpoints are leaves (degree <= 1)
            if degree.get(&he.origin).copied().unwrap_or(0) <= 1 {
                let _ = out_mesh.add_face(&[r0[3], r0[2], r0[1], r0[0]]);
            }
            if degree.get(&he.target).copied().unwrap_or(0) <= 1 {
                let _ = out_mesh.add_face(&[r1[0], r1[1], r1[2], r1[3]]);
            }
        }

        if out_mesh.vertices.is_empty() {
            return Ok(self.clone());
        }

        // 3. Weld coincident ring vertices at branching junctions
        let welded = out_mesh.weld_vertices_by_distance(radius * 0.65).unwrap_or(out_mesh);

        // 4. Optional Laplacian smoothing pass
        let mut final_mesh = welded;
        if smooth_iterations > 0 {
            let all_verts: Vec<crate::mesh::VertexKey> = final_mesh.vertices.keys().collect();
            let _ = final_mesh.relax_vertices(&all_verts, smooth_iterations, 0.25);
        }

        final_mesh.update_normals();
        Ok(final_mesh)
    }
}

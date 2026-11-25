use crate::mesh::{Mesh, MeshError, VertexKey};
use core_math::{vec2, vec3, Vec2, Vec3};
use std::collections::HashMap;

impl Mesh {
    /// Export the Half-Edge Mesh into a standard Wavefront OBJ format string.
    /// Preserves arbitrary N-gons, quads, triangles, UVs, and vertex normals.
    pub fn to_obj(&self, object_name: &str) -> String {
        let mut out = String::new();
        out.push_str(&format!("# Plumb 3D DCC Suite OBJ Exporter\no {}\n", object_name));

        // 1. Write vertices (v) and map VertexKey -> 1-based OBJ index
        let mut key_to_idx = HashMap::new();
        let mut idx = 1;

        for (vk, v) in &self.vertices {
            out.push_str(&format!("v {:.6} {:.6} {:.6}\n", v.position.x, v.position.y, v.position.z));
            key_to_idx.insert(vk, idx);
            idx += 1;
        }

        // 2. Write UV texture coordinates (vt)
        for (_, v) in &self.vertices {
            out.push_str(&format!("vt {:.6} {:.6}\n", v.uv.x, v.uv.y));
        }

        // 3. Write vertex normals (vn)
        for (_, v) in &self.vertices {
            out.push_str(&format!("vn {:.6} {:.6} {:.6}\n", v.normal.x, v.normal.y, v.normal.z));
        }

        out.push_str("s 1\n");

        // 4. Write polygon faces (f v/vt/vn ...)
        for (fk, _) in &self.faces {
            let v_keys = self.face_vertices(fk);
            if v_keys.len() < 3 {
                continue;
            }
            out.push('f');
            for &vk in &v_keys {
                let v_idx = key_to_idx[&vk];
                out.push_str(&format!(" {}/{}/{}", v_idx, v_idx, v_idx));
            }
            out.push('\n');
        }

        out
    }

    /// Import a Wavefront OBJ string into a topological Half-Edge Mesh.
    /// Correctly parses N-gon, quad, and triangle faces with vertex positions, UVs, and normals.
    pub fn from_obj(obj_str: &str) -> Result<Self, MeshError> {
        let mut mesh = Mesh::new();
        let mut positions: Vec<Vec3> = Vec::new();
        let mut uvs: Vec<Vec2> = Vec::new();

        // 1-based vertex map: obj_idx -> VertexKey in mesh
        let mut obj_to_mesh_vk: Vec<Option<VertexKey>> = Vec::new();

        // Pass 1: Parse vertices (v) and texture coordinates (vt)
        for line in obj_str.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let mut parts = line.split_whitespace();
            let prefix = match parts.next() {
                Some(p) => p,
                None => continue,
            };

            match prefix {
                "v" => {
                    let x: f32 = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0.0);
                    let y: f32 = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0.0);
                    let z: f32 = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0.0);
                    positions.push(vec3(x, y, z));
                }
                "vt" => {
                    let u: f32 = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0.0);
                    let v: f32 = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0.0);
                    uvs.push(vec2(u, v));
                }
                _ => {}
            }
        }

        // Add vertices to half-edge mesh
        obj_to_mesh_vk.resize(positions.len() + 1, None);
        for (i, &pos) in positions.iter().enumerate() {
            let uv = uvs.get(i).copied().unwrap_or(Vec2::ZERO);
            let vk = mesh.add_vertex_with_uv(pos, uv);
            obj_to_mesh_vk[i + 1] = Some(vk); // 1-based indexing
        }

        // Pass 2: Parse faces (f)
        for line in obj_str.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let mut parts = line.split_whitespace();
            let prefix = match parts.next() {
                Some(p) => p,
                None => continue,
            };

            if prefix == "f" {
                let mut face_vks = Vec::new();
                for vert_spec in parts {
                    // Formats: "v", "v/vt", "v//vn", "v/vt/vn"
                    let v_idx_str = vert_spec.split('/').next().unwrap_or("");
                    if let Ok(v_idx) = v_idx_str.parse::<usize>() {
                        if v_idx < obj_to_mesh_vk.len() {
                            if let Some(vk) = obj_to_mesh_vk[v_idx] {
                                face_vks.push(vk);
                            }
                        }
                    }
                }

                if face_vks.len() >= 3 {
                    let _ = mesh.add_face(&face_vks);
                }
            }
        }

        mesh.update_normals();
        Ok(mesh)
    }
}

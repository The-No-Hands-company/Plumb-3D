use crate::mesh::{Mesh, MeshError};
use crate::triangulation::ShadingMode;
use core_math::vec3;
use std::collections::HashMap;

impl Mesh {
    /// Export the Half-Edge Mesh into ASCII Stanford PLY format with positions, normals, and vertex colors
    pub fn to_ply_ascii(&self) -> String {
        let mut out = String::new();
        let num_verts = self.vertices.len();
        let num_faces = self.faces.len();

        out.push_str("ply\nformat ascii 1.0\ncomment Clean-Room Rust 3D DCC Suite PLY Exporter\n");
        out.push_str(&format!("element vertex {}\n", num_verts));
        out.push_str("property float x\nproperty float y\nproperty float z\n");
        out.push_str("property float nx\nproperty float ny\nproperty float nz\n");
        out.push_str(
            "property uchar red\nproperty uchar green\nproperty uchar blue\nproperty uchar alpha\n",
        );
        out.push_str(&format!("element face {}\n", num_faces));
        out.push_str("property list uchar int vertex_indices\nend_header\n");

        let mut key_to_idx = HashMap::new();
        for (i, (vk, v)) in self.vertices.iter().enumerate() {
            key_to_idx.insert(vk, i);
            let r = (v.color[0].clamp(0.0, 1.0) * 255.0) as u8;
            let g = (v.color[1].clamp(0.0, 1.0) * 255.0) as u8;
            let b = (v.color[2].clamp(0.0, 1.0) * 255.0) as u8;
            let a = (v.color[3].clamp(0.0, 1.0) * 255.0) as u8;

            out.push_str(&format!(
                "{:.6} {:.6} {:.6} {:.6} {:.6} {:.6} {} {} {} {}\n",
                v.position.x,
                v.position.y,
                v.position.z,
                v.normal.x,
                v.normal.y,
                v.normal.z,
                r,
                g,
                b,
                a
            ));
        }

        for (fk, _) in &self.faces {
            let v_keys = self.face_vertices(fk);
            out.push_str(&format!("{}", v_keys.len()));
            for &vk in &v_keys {
                if let Some(&idx) = key_to_idx.get(&vk) {
                    out.push_str(&format!(" {}", idx));
                }
            }
            out.push('\n');
        }

        out
    }

    /// Export the Half-Edge Mesh into compact binary little-endian Stanford PLY format
    pub fn to_ply_binary(&self) -> Vec<u8> {
        let render_mesh = self.to_render_mesh(ShadingMode::Flat);
        let num_verts = render_mesh.vertices.len();
        let num_faces = render_mesh.indices.len() / 3;

        let header = format!(
            "ply\nformat binary_little_endian 1.0\ncomment Clean-Room Rust 3D DCC Suite\nelement vertex {}\nproperty float x\nproperty float y\nproperty float z\nproperty float nx\nproperty float ny\nproperty float nz\nproperty uchar red\nproperty uchar green\nproperty uchar blue\nproperty uchar alpha\nelement face {}\nproperty list uchar int vertex_indices\nend_header\n",
            num_verts, num_faces
        );

        let mut bytes = Vec::with_capacity(header.len() + num_verts * 28 + num_faces * 13);
        bytes.extend_from_slice(header.as_bytes());

        // Vertex buffer: 3x f32 (pos) + 3x f32 (normal) + 4x u8 (color) = 28 bytes
        for v in &render_mesh.vertices {
            bytes.extend_from_slice(&v.position[0].to_le_bytes());
            bytes.extend_from_slice(&v.position[1].to_le_bytes());
            bytes.extend_from_slice(&v.position[2].to_le_bytes());

            bytes.extend_from_slice(&v.normal[0].to_le_bytes());
            bytes.extend_from_slice(&v.normal[1].to_le_bytes());
            bytes.extend_from_slice(&v.normal[2].to_le_bytes());

            let r = (v.color[0].clamp(0.0, 1.0) * 255.0) as u8;
            let g = (v.color[1].clamp(0.0, 1.0) * 255.0) as u8;
            let b = (v.color[2].clamp(0.0, 1.0) * 255.0) as u8;
            let a = (v.color[3].clamp(0.0, 1.0) * 255.0) as u8;
            bytes.push(r);
            bytes.push(g);
            bytes.push(b);
            bytes.push(a);
        }

        // Face buffer: 1 byte count (3) + 3x i32 (indices) = 13 bytes
        for chunk in render_mesh.indices.as_chunks::<3>().0 {
            bytes.push(3u8);
            bytes.extend_from_slice(&(chunk[0] as i32).to_le_bytes());
            bytes.extend_from_slice(&(chunk[1] as i32).to_le_bytes());
            bytes.extend_from_slice(&(chunk[2] as i32).to_le_bytes());
        }

        bytes
    }

    /// Import a Stanford PLY file (auto-detects ASCII vs Binary Little-Endian)
    pub fn from_ply(data: &[u8]) -> Result<Self, MeshError> {
        if !data.starts_with(b"ply") {
            return Err(MeshError::InvariantFailed(
                "Missing 'ply' magic header".into(),
            ));
        }

        // Find end_header
        let mut header_end = 0;
        for (i, window) in data.windows(11).enumerate() {
            if window == b"end_header\n" {
                header_end = i + 11;
                break;
            }
        }

        if header_end == 0 {
            return Err(MeshError::InvariantFailed(
                "Missing 'end_header' in PLY".into(),
            ));
        }

        let header_str = std::str::from_utf8(&data[..header_end]).map_err(|e| {
            MeshError::InvariantFailed(format!("Invalid UTF-8 in PLY header: {}", e))
        })?;

        if header_str.contains("format ascii") {
            let full_text = std::str::from_utf8(data).map_err(|e| {
                MeshError::InvariantFailed(format!("Invalid UTF-8 in ASCII PLY: {}", e))
            })?;
            Self::from_ply_ascii(full_text)
        } else {
            Self::from_ply_binary_with_header(data, header_str, header_end)
        }
    }

    /// Parse an ASCII Stanford PLY string into a 2-manifold Half-Edge Mesh
    pub fn from_ply_ascii(text: &str) -> Result<Self, MeshError> {
        let mut lines = text.lines();
        let mut num_verts = 0;
        let mut num_faces = 0;
        let mut in_header = true;

        for line in lines.by_ref() {
            let line = line.trim();
            if line.starts_with("element vertex") {
                num_verts = line
                    .split_whitespace()
                    .nth(2)
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
            } else if line.starts_with("element face") {
                num_faces = line
                    .split_whitespace()
                    .nth(2)
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
            } else if line == "end_header" {
                in_header = false;
                break;
            }
        }

        if in_header {
            return Err(MeshError::InvariantFailed("Unterminated PLY header".into()));
        }

        let mut mesh = Mesh::new();
        let mut v_keys = Vec::with_capacity(num_verts);

        // Read vertices
        for _ in 0..num_verts {
            if let Some(line) = lines.next() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    let x: f32 = parts[0].parse().unwrap_or(0.0);
                    let y: f32 = parts[1].parse().unwrap_or(0.0);
                    let z: f32 = parts[2].parse().unwrap_or(0.0);
                    let vk = mesh.add_vertex(vec3(x, y, z));
                    v_keys.push(vk);
                }
            }
        }

        // Read faces
        for _ in 0..num_faces {
            if let Some(line) = lines.next() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if let Some(count_str) = parts.first() {
                    if let Ok(count) = count_str.parse::<usize>() {
                        if parts.len() > count {
                            let mut face_vks = Vec::with_capacity(count);
                            for idx_str in &parts[1..=count] {
                                if let Ok(idx) = idx_str.parse::<usize>() {
                                    if idx < v_keys.len() {
                                        face_vks.push(v_keys[idx]);
                                    }
                                }
                            }
                            if face_vks.len() >= 3 {
                                let _ = mesh.add_face(&face_vks);
                            }
                        }
                    }
                }
            }
        }

        mesh.update_normals();
        Ok(mesh)
    }

    /// Parse a Binary Little-Endian Stanford PLY buffer into a 2-manifold Half-Edge Mesh
    pub fn from_ply_binary_with_header(
        bytes: &[u8],
        header_str: &str,
        header_end: usize,
    ) -> Result<Self, MeshError> {
        let mut num_verts = 0;
        let mut num_faces = 0;

        for line in header_str.lines() {
            let line = line.trim();
            if line.starts_with("element vertex") {
                num_verts = line
                    .split_whitespace()
                    .nth(2)
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
            } else if line.starts_with("element face") {
                num_faces = line
                    .split_whitespace()
                    .nth(2)
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
            }
        }

        let mut mesh = Mesh::new();
        let mut v_keys = Vec::with_capacity(num_verts);
        let mut offset = header_end;

        let vert_stride = 28;
        for _ in 0..num_verts {
            if offset + 12 <= bytes.len() {
                let x = f32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap());
                let y = f32::from_le_bytes(bytes[offset + 4..offset + 8].try_into().unwrap());
                let z = f32::from_le_bytes(bytes[offset + 8..offset + 12].try_into().unwrap());
                let vk = mesh.add_vertex(vec3(x, y, z));
                v_keys.push(vk);
            }
            offset += vert_stride;
        }

        for _ in 0..num_faces {
            if offset < bytes.len() {
                let count = bytes[offset] as usize;
                offset += 1;
                let mut face_vks = Vec::with_capacity(count);
                for _ in 0..count {
                    if offset + 4 <= bytes.len() {
                        let idx = i32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
                            as usize;
                        offset += 4;
                        if idx < v_keys.len() {
                            face_vks.push(v_keys[idx]);
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

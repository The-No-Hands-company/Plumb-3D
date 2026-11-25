use crate::mesh::{Mesh, MeshError};
use crate::triangulation::ShadingMode;
use core_math::{vec3, Vec3};

impl Mesh {
    /// Export the Half-Edge Mesh into ASCII STL format (standard for 3D printing and CAD)
    pub fn to_stl_ascii(&self, solid_name: &str) -> String {
        let render_mesh = self.to_render_mesh(ShadingMode::Flat);
        let mut out = String::new();
        out.push_str(&format!("solid {}\n", solid_name));

        for chunk in render_mesh.indices.as_chunks::<3>().0 {
            let v0 = &render_mesh.vertices[chunk[0] as usize];
            let v1 = &render_mesh.vertices[chunk[1] as usize];
            let v2 = &render_mesh.vertices[chunk[2] as usize];

            let p0 = Vec3::from_slice(&v0.position);
            let p1 = Vec3::from_slice(&v1.position);
            let p2 = Vec3::from_slice(&v2.position);

            let normal = ((p1 - p0).cross(p2 - p0)).normalize_or_zero();

            out.push_str(&format!(
                "  facet normal {:.6} {:.6} {:.6}\n    outer loop\n      vertex {:.6} {:.6} {:.6}\n      vertex {:.6} {:.6} {:.6}\n      vertex {:.6} {:.6} {:.6}\n    endloop\n  endfacet\n",
                normal.x, normal.y, normal.z,
                p0.x, p0.y, p0.z,
                p1.x, p1.y, p1.z,
                p2.x, p2.y, p2.z,
            ));
        }

        out.push_str(&format!("endsolid {}\n", solid_name));
        out
    }

    /// Export the Half-Edge Mesh into compact binary STL format
    pub fn to_stl_binary(&self) -> Vec<u8> {
        let render_mesh = self.to_render_mesh(ShadingMode::Flat);
        let tri_count = (render_mesh.indices.len() / 3) as u32;

        let total_size = 80 + 4 + (tri_count as usize) * 50;
        let mut bytes = Vec::with_capacity(total_size);

        // 80-byte header
        let header = b"Clean-Room Rust 3D DCC Suite Binary STL Exporter";
        let mut header_buf = [0u8; 80];
        header_buf[..header.len()].copy_from_slice(header);
        bytes.extend_from_slice(&header_buf);

        // 4-byte triangle count
        bytes.extend_from_slice(&tri_count.to_le_bytes());

        // Triangles
        for chunk in render_mesh.indices.as_chunks::<3>().0 {
            let v0 = &render_mesh.vertices[chunk[0] as usize];
            let v1 = &render_mesh.vertices[chunk[1] as usize];
            let v2 = &render_mesh.vertices[chunk[2] as usize];

            let p0 = Vec3::from_slice(&v0.position);
            let p1 = Vec3::from_slice(&v1.position);
            let p2 = Vec3::from_slice(&v2.position);

            let normal = ((p1 - p0).cross(p2 - p0)).normalize_or_zero();

            // Normal vector (12 bytes)
            bytes.extend_from_slice(&normal.x.to_le_bytes());
            bytes.extend_from_slice(&normal.y.to_le_bytes());
            bytes.extend_from_slice(&normal.z.to_le_bytes());

            // Vertex 1 (12 bytes)
            bytes.extend_from_slice(&p0.x.to_le_bytes());
            bytes.extend_from_slice(&p0.y.to_le_bytes());
            bytes.extend_from_slice(&p0.z.to_le_bytes());

            // Vertex 2 (12 bytes)
            bytes.extend_from_slice(&p1.x.to_le_bytes());
            bytes.extend_from_slice(&p1.y.to_le_bytes());
            bytes.extend_from_slice(&p1.z.to_le_bytes());

            // Vertex 3 (12 bytes)
            bytes.extend_from_slice(&p2.x.to_le_bytes());
            bytes.extend_from_slice(&p2.y.to_le_bytes());
            bytes.extend_from_slice(&p2.z.to_le_bytes());

            // 2-byte attribute byte count (standard: 0)
            bytes.extend_from_slice(&0u16.to_le_bytes());
        }

        bytes
    }

    /// Import an STL file (auto-detects ASCII vs Binary)
    pub fn from_stl(data: &[u8]) -> Result<Self, MeshError> {
        // ASCII STL files begin with "solid"
        if data.len() >= 5 && &data[..5] == b"solid" {
            if let Ok(text) = std::str::from_utf8(data) {
                if text.contains("facet") && text.contains("endfacet") {
                    return Self::from_stl_ascii(text);
                }
            }
        }

        Self::from_stl_binary(data)
    }

    /// Parse an ASCII STL string into a 2-manifold Half-Edge Mesh
    pub fn from_stl_ascii(text: &str) -> Result<Self, MeshError> {
        let mut mesh = Mesh::new();
        let mut current_verts = Vec::with_capacity(3);

        for line in text.lines() {
            let line = line.trim();
            if line.starts_with("vertex") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    let x: f32 = parts[1].parse().unwrap_or(0.0);
                    let y: f32 = parts[2].parse().unwrap_or(0.0);
                    let z: f32 = parts[3].parse().unwrap_or(0.0);
                    let vk = mesh.add_vertex(vec3(x, y, z));
                    current_verts.push(vk);

                    if current_verts.len() == 3 {
                        let _ = mesh.add_face(&current_verts);
                        current_verts.clear();
                    }
                }
            }
        }

        mesh.update_normals();
        Ok(mesh)
    }

    /// Parse a Binary STL byte buffer into a 2-manifold Half-Edge Mesh
    pub fn from_stl_binary(bytes: &[u8]) -> Result<Self, MeshError> {
        if bytes.len() < 84 {
            return Err(MeshError::InvariantFailed("Binary STL buffer too short (< 84 bytes)".into()));
        }

        let num_triangles = u32::from_le_bytes(bytes[80..84].try_into().unwrap()) as usize;
        let expected_size = 84 + num_triangles * 50;
        if bytes.len() < expected_size {
            return Err(MeshError::InvariantFailed(format!(
                "Incomplete binary STL: expected {} bytes, got {}",
                expected_size,
                bytes.len()
            )));
        }

        let mut mesh = Mesh::new();
        let mut offset = 84;

        for _ in 0..num_triangles {
            // Skip 12-byte normal vector at offset..offset+12
            offset += 12;

            let p0_x = f32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap());
            let p0_y = f32::from_le_bytes(bytes[offset + 4..offset + 8].try_into().unwrap());
            let p0_z = f32::from_le_bytes(bytes[offset + 8..offset + 12].try_into().unwrap());
            offset += 12;

            let p1_x = f32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap());
            let p1_y = f32::from_le_bytes(bytes[offset + 4..offset + 8].try_into().unwrap());
            let p1_z = f32::from_le_bytes(bytes[offset + 8..offset + 12].try_into().unwrap());
            offset += 12;

            let p2_x = f32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap());
            let p2_y = f32::from_le_bytes(bytes[offset + 4..offset + 8].try_into().unwrap());
            let p2_z = f32::from_le_bytes(bytes[offset + 8..offset + 12].try_into().unwrap());
            offset += 12;

            // Skip 2-byte attribute byte count
            offset += 2;

            let v0 = mesh.add_vertex(vec3(p0_x, p0_y, p0_z));
            let v1 = mesh.add_vertex(vec3(p1_x, p1_y, p1_z));
            let v2 = mesh.add_vertex(vec3(p2_x, p2_y, p2_z));
            let _ = mesh.add_face(&[v0, v1, v2]);
        }

        mesh.update_normals();
        Ok(mesh)
    }
}

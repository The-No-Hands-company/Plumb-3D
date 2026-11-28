use crate::io::gltf::parse_gltf_payload;
use crate::mesh::{Mesh, MeshError};

impl Mesh {
    /// Export the Half-Edge Mesh into standard binary glTF 2.0 format (.glb)
    pub fn to_glb(&self, object_name: &str) -> Vec<u8> {
        self.to_glb_with_material(object_name, [0.8, 0.8, 0.8, 1.0], 0.0, 0.5)
    }

    /// Export the Half-Edge Mesh into standard binary glTF 2.0 format (.glb)
    /// with embedded PBR metallic-roughness material parameters.
    pub fn to_glb_with_material(
        &self,
        object_name: &str,
        base_color: [f32; 4],
        metallic: f32,
        roughness: f32,
    ) -> Vec<u8> {
        let render_mesh = self.to_render_mesh(crate::triangulation::ShadingMode::Smooth);
        let vert_count = render_mesh.vertices.len();
        let index_count = render_mesh.indices.len();

        let mut min_pos = [f32::INFINITY; 3];
        let mut max_pos = [f32::NEG_INFINITY; 3];

        let mut pos_bytes = Vec::with_capacity(vert_count * 12);
        let mut norm_bytes = Vec::with_capacity(vert_count * 12);
        let mut uv_bytes = Vec::with_capacity(vert_count * 8);

        for v in &render_mesh.vertices {
            for i in 0..3 {
                min_pos[i] = min_pos[i].min(v.position[i]);
                max_pos[i] = max_pos[i].max(v.position[i]);
            }
            pos_bytes.extend_from_slice(bytemuck::bytes_of(&v.position));
            norm_bytes.extend_from_slice(bytemuck::bytes_of(&v.normal));
            uv_bytes.extend_from_slice(bytemuck::bytes_of(&v.uv));
        }

        if vert_count == 0 {
            min_pos = [0.0; 3];
            max_pos = [0.0; 3];
        }

        let mut index_bytes = Vec::with_capacity(index_count * 4);
        for &idx in &render_mesh.indices {
            index_bytes.extend_from_slice(bytemuck::bytes_of(&idx));
        }

        // Combine binary buffers (all 4-byte aligned)
        let mut bin_bytes = Vec::new();

        let pos_offset = 0;
        let pos_len = pos_bytes.len();
        bin_bytes.extend_from_slice(&pos_bytes);

        let norm_offset = bin_bytes.len();
        let norm_len = norm_bytes.len();
        bin_bytes.extend_from_slice(&norm_bytes);

        let uv_offset = bin_bytes.len();
        let uv_len = uv_bytes.len();
        bin_bytes.extend_from_slice(&uv_bytes);

        let index_offset = bin_bytes.len();
        let index_len = index_bytes.len();
        bin_bytes.extend_from_slice(&index_bytes);

        let total_bin_bytes = bin_bytes.len();

        let json_str = format!(
            r#"{{
  "asset": {{
    "version": "2.0",
    "generator": "Plumb 3D DCC Suite"
  }},
  "scene": 0,
  "scenes": [
    {{
      "nodes": [0]
    }}
  ],
  "nodes": [
    {{
      "name": "{name}",
      "mesh": 0
    }}
  ],
  "materials": [
    {{
      "name": "{name}_PBR",
      "pbrMetallicRoughness": {{
        "baseColorFactor": [{bc0:.4}, {bc1:.4}, {bc2:.4}, {bc3:.4}],
        "metallicFactor": {metallic:.4},
        "roughnessFactor": {roughness:.4}
      }},
      "doubleSided": true
    }}
  ],
  "meshes": [
    {{
      "name": "{name}",
      "primitives": [
        {{
          "attributes": {{
            "POSITION": 0,
            "NORMAL": 1,
            "TEXCOORD_0": 2
          }},
          "indices": 3,
          "material": 0,
          "mode": 4
        }}
      ]
    }}
  ],
  "accessors": [
    {{
      "bufferView": 0,
      "byteOffset": 0,
      "componentType": 5126,
      "count": {vert_count},
      "type": "VEC3",
      "min": [{min_x:.6}, {min_y:.6}, {min_z:.6}],
      "max": [{max_x:.6}, {max_y:.6}, {max_z:.6}]
    }},
    {{
      "bufferView": 1,
      "byteOffset": 0,
      "componentType": 5126,
      "count": {vert_count},
      "type": "VEC3"
    }},
    {{
      "bufferView": 2,
      "byteOffset": 0,
      "componentType": 5126,
      "count": {vert_count},
      "type": "VEC2"
    }},
    {{
      "bufferView": 3,
      "byteOffset": 0,
      "componentType": 5125,
      "count": {index_count},
      "type": "SCALAR"
    }}
  ],
  "bufferViews": [
    {{
      "buffer": 0,
      "byteOffset": {pos_offset},
      "byteLength": {pos_len},
      "target": 34962
    }},
    {{
      "buffer": 0,
      "byteOffset": {norm_offset},
      "byteLength": {norm_len},
      "target": 34962
    }},
    {{
      "buffer": 0,
      "byteOffset": {uv_offset},
      "byteLength": {uv_len},
      "target": 34962
    }},
    {{
      "buffer": 0,
      "byteOffset": {index_offset},
      "byteLength": {index_len},
      "target": 34963
    }}
  ],
  "buffers": [
    {{
      "byteLength": {total_bin_bytes}
    }}
  ]
}}"#,
            name = object_name,
            bc0 = base_color[0],
            bc1 = base_color[1],
            bc2 = base_color[2],
            bc3 = base_color[3],
            metallic = metallic.clamp(0.0, 1.0),
            roughness = roughness.clamp(0.0, 1.0),
            vert_count = vert_count,
            index_count = index_count,
            min_x = min_pos[0],
            min_y = min_pos[1],
            min_z = min_pos[2],
            max_x = max_pos[0],
            max_y = max_pos[1],
            max_z = max_pos[2],
            pos_offset = pos_offset,
            pos_len = pos_len,
            norm_offset = norm_offset,
            norm_len = norm_len,
            uv_offset = uv_offset,
            uv_len = uv_len,
            index_offset = index_offset,
            index_len = index_len,
            total_bin_bytes = total_bin_bytes,
        );

        // Pad JSON chunk to 4-byte boundary with ASCII space 0x20
        let json_bytes = json_str.as_bytes();
        let json_pad = (4 - (json_bytes.len() % 4)) % 4;
        let mut json_chunk = Vec::with_capacity(json_bytes.len() + json_pad);
        json_chunk.extend_from_slice(json_bytes);
        json_chunk.resize(json_bytes.len() + json_pad, b' ');

        // Pad BIN chunk to 4-byte boundary with 0x00
        let bin_pad = (4 - (bin_bytes.len() % 4)) % 4;
        let mut bin_chunk = Vec::with_capacity(bin_bytes.len() + bin_pad);
        bin_chunk.extend_from_slice(&bin_bytes);
        bin_chunk.resize(bin_bytes.len() + bin_pad, 0x00);

        // Total GLB file size: 12 (header) + 8 + json_chunk.len() + 8 + bin_chunk.len()
        let total_file_size = 12 + 8 + json_chunk.len() + 8 + bin_chunk.len();
        let mut out = Vec::with_capacity(total_file_size);

        // 1. Header (12 bytes)
        out.extend_from_slice(b"glTF");
        out.extend_from_slice(&2u32.to_le_bytes());
        out.extend_from_slice(&(total_file_size as u32).to_le_bytes());

        // 2. Chunk 0: JSON (8 bytes header + data)
        out.extend_from_slice(&(json_chunk.len() as u32).to_le_bytes());
        out.extend_from_slice(b"JSON");
        out.extend_from_slice(&json_chunk);

        // 3. Chunk 1: BIN (8 bytes header + data)
        out.extend_from_slice(&(bin_chunk.len() as u32).to_le_bytes());
        out.extend_from_slice(b"BIN\0");
        out.extend_from_slice(&bin_chunk);

        out
    }

    /// Import a binary glTF 2.0 (.glb) container into a 2-manifold Half-Edge Mesh.
    pub fn from_glb(glb_bytes: &[u8]) -> Result<Self, MeshError> {
        if glb_bytes.len() < 12 {
            return Err(MeshError::InvariantFailed(
                "GLB binary buffer too short for header".into(),
            ));
        }

        let magic = u32::from_le_bytes(glb_bytes[0..4].try_into().unwrap());
        if magic != 0x4654_6C67 {
            return Err(MeshError::InvariantFailed(
                "Invalid glTF 2.0 magic number in GLB header".into(),
            ));
        }

        let version = u32::from_le_bytes(glb_bytes[4..8].try_into().unwrap());
        if version != 2 {
            return Err(MeshError::InvariantFailed(format!(
                "Unsupported glTF version: {}",
                version
            )));
        }

        let total_length = u32::from_le_bytes(glb_bytes[8..12].try_into().unwrap()) as usize;
        if glb_bytes.len() < total_length {
            return Err(MeshError::InvariantFailed(
                "Truncated glTF 2.0 binary file".into(),
            ));
        }

        // Chunk 0: JSON Chunk
        if glb_bytes.len() < 20 {
            return Err(MeshError::InvariantFailed(
                "Missing JSON chunk header in GLB".into(),
            ));
        }

        let json_chunk_len = u32::from_le_bytes(glb_bytes[12..16].try_into().unwrap()) as usize;
        let json_chunk_type = u32::from_le_bytes(glb_bytes[16..20].try_into().unwrap());
        if json_chunk_type != 0x4E4F_534A {
            return Err(MeshError::InvariantFailed(
                "First chunk in GLB container must be JSON".into(),
            ));
        }

        let json_end = 20 + json_chunk_len;
        if glb_bytes.len() < json_end {
            return Err(MeshError::InvariantFailed(
                "Truncated JSON chunk in GLB".into(),
            ));
        }

        let json_str = std::str::from_utf8(&glb_bytes[20..json_end]).map_err(|e| {
            MeshError::InvariantFailed(format!("Invalid UTF-8 in glTF JSON chunk: {}", e))
        })?;
        let root: serde_json::Value = serde_json::from_str(json_str)
            .map_err(|e| MeshError::InvariantFailed(format!("Invalid glTF JSON: {}", e)))?;

        // Chunk 1: BIN Chunk (optional per glTF spec, but required for mesh geometry)
        let bin_bytes = if glb_bytes.len() >= json_end + 8 {
            let bin_chunk_len =
                u32::from_le_bytes(glb_bytes[json_end..json_end + 4].try_into().unwrap()) as usize;
            let bin_chunk_type =
                u32::from_le_bytes(glb_bytes[json_end + 4..json_end + 8].try_into().unwrap());
            if bin_chunk_type == 0x004E_4942 {
                let bin_start = json_end + 8;
                let bin_end = (bin_start + bin_chunk_len).min(glb_bytes.len());
                &glb_bytes[bin_start..bin_end]
            } else {
                &[]
            }
        } else {
            &[]
        };

        parse_gltf_payload(&root, bin_bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glb_cube_export_and_import_roundtrip() {
        let cube = crate::primitives::Primitives::create_cube(2.0);
        let orig_verts = cube.vertices.len();
        let orig_faces = cube.faces.len();

        let glb_bytes = cube.to_glb_with_material("HeroCube", [0.9, 0.1, 0.1, 1.0], 0.8, 0.2);

        // Verify GLB magic header
        assert_eq!(&glb_bytes[0..4], b"glTF");
        let version = u32::from_le_bytes(glb_bytes[4..8].try_into().unwrap());
        assert_eq!(version, 2);
        let total_len = u32::from_le_bytes(glb_bytes[8..12].try_into().unwrap()) as usize;
        assert_eq!(total_len, glb_bytes.len());

        // Import back
        let imported = Mesh::from_glb(&glb_bytes).expect("Failed to import GLB");
        assert_eq!(imported.vertices.len(), orig_verts);
        assert_eq!(imported.faces.len(), orig_faces * 2); // triangulated faces
    }

    #[test]
    fn test_glb_malformed_buffer_error_handling() {
        let empty_bytes = vec![0u8; 8];
        assert!(Mesh::from_glb(&empty_bytes).is_err());

        let invalid_magic = vec![0u8; 32];
        assert!(Mesh::from_glb(&invalid_magic).is_err());
    }
}

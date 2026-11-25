use crate::mesh::{Mesh, MeshError};
use core_math::vec3;

impl Mesh {
    /// Export the Half-Edge Mesh into standard glTF 2.0 JSON format (.gltf)
    /// with self-contained base64 binary buffer (POSITION, NORMAL, TEXCOORD_0, INDICES).
    pub fn to_gltf(&self, object_name: &str) -> String {
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

        // Combine buffers with 4-byte alignment
        let mut combined_buffer = Vec::new();

        let pos_offset = 0;
        let pos_len = pos_bytes.len();
        combined_buffer.extend_from_slice(&pos_bytes);

        let norm_offset = combined_buffer.len();
        let norm_len = norm_bytes.len();
        combined_buffer.extend_from_slice(&norm_bytes);

        let uv_offset = combined_buffer.len();
        let uv_len = uv_bytes.len();
        combined_buffer.extend_from_slice(&uv_bytes);

        let index_offset = combined_buffer.len();
        let index_len = index_bytes.len();
        combined_buffer.extend_from_slice(&index_bytes);

        let total_bytes = combined_buffer.len();
        let base64_uri = format!("data:application/octet-stream;base64,{}", base64_encode(&combined_buffer));

        format!(
            r#"{{
  "asset": {{
    "version": "2.0",
    "generator": "Clean-Room Rust 3D DCC Suite"
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
      "byteLength": {total_bytes},
      "uri": "{base64_uri}"
    }}
  ]
}}"#,
            name = object_name,
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
            total_bytes = total_bytes,
            base64_uri = base64_uri
        )
    }

    /// Import a glTF 2.0 JSON string (.gltf) into a 2-manifold Half-Edge Mesh.
    /// Supports embedded base64 data URIs, POSITION, NORMAL, TEXCOORD_0, and indices.
    pub fn from_gltf(gltf_str: &str) -> Result<Self, MeshError> {
        let root: serde_json::Value = serde_json::from_str(gltf_str)
            .map_err(|e| MeshError::InvariantFailed(format!("Invalid glTF JSON: {}", e)))?;

        // 1. Get buffer data
        let buffers = root["buffers"]
            .as_array()
            .ok_or_else(|| MeshError::InvariantFailed("Missing glTF buffers array".into()))?;
        if buffers.is_empty() {
            return Err(MeshError::InvariantFailed("glTF has no buffers".into()));
        }

        let uri = buffers[0]["uri"].as_str().unwrap_or("");
        let buffer_bytes = if uri.starts_with("data:") {
            let base64_str = uri
                .split(',')
                .nth(1)
                .ok_or_else(|| MeshError::InvariantFailed("Invalid data URI in glTF".into()))?;
            base64_decode(base64_str)
                .map_err(|e| MeshError::InvariantFailed(format!("Base64 decode error: {}", e)))?
        } else {
            return Err(MeshError::InvariantFailed(
                "External glTF buffer URIs not supported, use embedded data URI".into(),
            ));
        };

        let buffer_views = root["bufferViews"]
            .as_array()
            .ok_or_else(|| MeshError::InvariantFailed("Missing bufferViews".into()))?;
        let accessors = root["accessors"]
            .as_array()
            .ok_or_else(|| MeshError::InvariantFailed("Missing accessors".into()))?;

        // Find primary mesh primitive
        let meshes = root["meshes"]
            .as_array()
            .ok_or_else(|| MeshError::InvariantFailed("Missing meshes".into()))?;
        if meshes.is_empty() {
            return Err(MeshError::InvariantFailed("No meshes in glTF".into()));
        }
        let primitives = meshes[0]["primitives"]
            .as_array()
            .ok_or_else(|| MeshError::InvariantFailed("Missing primitives".into()))?;
        if primitives.is_empty() {
            return Err(MeshError::InvariantFailed("No primitives in glTF".into()));
        }
        let prim = &primitives[0];

        let pos_acc_idx = prim["attributes"]["POSITION"]
            .as_u64()
            .ok_or_else(|| MeshError::InvariantFailed("Missing POSITION attribute in glTF".into()))?
            as usize;
        let pos_acc = &accessors[pos_acc_idx];
        let pos_count = pos_acc["count"].as_u64().unwrap_or(0) as usize;
        let pos_bv_idx = pos_acc["bufferView"].as_u64().unwrap_or(0) as usize;
        let pos_bv = &buffer_views[pos_bv_idx];
        let pos_bv_offset = pos_bv["byteOffset"].as_u64().unwrap_or(0) as usize;
        let pos_acc_offset = pos_acc["byteOffset"].as_u64().unwrap_or(0) as usize;
        let pos_start = pos_bv_offset + pos_acc_offset;

        // Read positions
        let mut positions = Vec::with_capacity(pos_count);
        for i in 0..pos_count {
            let offset = pos_start + i * 12;
            if offset + 12 <= buffer_bytes.len() {
                let x = f32::from_le_bytes(buffer_bytes[offset..offset + 4].try_into().unwrap());
                let y = f32::from_le_bytes(buffer_bytes[offset + 4..offset + 8].try_into().unwrap());
                let z = f32::from_le_bytes(buffer_bytes[offset + 8..offset + 12].try_into().unwrap());
                positions.push(vec3(x, y, z));
            }
        }

        // Read UVs if present
        let mut uvs = vec![core_math::Vec2::ZERO; pos_count];
        if let Some(uv_acc_idx) = prim["attributes"]["TEXCOORD_0"].as_u64() {
            let uv_acc = &accessors[uv_acc_idx as usize];
            let uv_bv_idx = uv_acc["bufferView"].as_u64().unwrap_or(0) as usize;
            let uv_bv = &buffer_views[uv_bv_idx];
            let uv_start = uv_bv["byteOffset"].as_u64().unwrap_or(0) as usize
                + uv_acc["byteOffset"].as_u64().unwrap_or(0) as usize;
            let count = uv_acc["count"].as_u64().unwrap_or(0) as usize;
            for i in 0..count.min(pos_count) {
                let offset = uv_start + i * 8;
                if offset + 8 <= buffer_bytes.len() {
                    let u = f32::from_le_bytes(buffer_bytes[offset..offset + 4].try_into().unwrap());
                    let v = f32::from_le_bytes(buffer_bytes[offset + 4..offset + 8].try_into().unwrap());
                    uvs[i] = core_math::vec2(u, v);
                }
            }
        }

        // Construct mesh vertices
        let mut mesh = Mesh::new();
        let mut v_keys = Vec::with_capacity(positions.len());
        for (i, &pos) in positions.iter().enumerate() {
            let vk = mesh.add_vertex_with_uv(pos, uvs[i]);
            v_keys.push(vk);
        }

        // Read indices
        if let Some(idx_acc_val) = prim["indices"].as_u64() {
            let idx_acc = &accessors[idx_acc_val as usize];
            let comp_type = idx_acc["componentType"].as_u64().unwrap_or(5125);
            let idx_count = idx_acc["count"].as_u64().unwrap_or(0) as usize;
            let idx_bv_idx = idx_acc["bufferView"].as_u64().unwrap_or(0) as usize;
            let idx_bv = &buffer_views[idx_bv_idx];
            let idx_start = idx_bv["byteOffset"].as_u64().unwrap_or(0) as usize
                + idx_acc["byteOffset"].as_u64().unwrap_or(0) as usize;

            let mut indices = Vec::with_capacity(idx_count);
            for i in 0..idx_count {
                if comp_type == 5123 {
                    let offset = idx_start + i * 2;
                    if offset + 2 <= buffer_bytes.len() {
                        let idx = u16::from_le_bytes(
                            buffer_bytes[offset..offset + 2].try_into().unwrap(),
                        ) as usize;
                        indices.push(idx);
                    }
                } else {
                    let offset = idx_start + i * 4;
                    if offset + 4 <= buffer_bytes.len() {
                        let idx = u32::from_le_bytes(
                            buffer_bytes[offset..offset + 4].try_into().unwrap(),
                        ) as usize;
                        indices.push(idx);
                    }
                }
            }

            for tri in indices.as_chunks::<3>().0 {
                let (i0, i1, i2) = (tri[0], tri[1], tri[2]);
                if i0 < v_keys.len() && i1 < v_keys.len() && i2 < v_keys.len() {
                    let _ = mesh.add_face(&[v_keys[i0], v_keys[i1], v_keys[i2]]);
                }
            }
        }

        mesh.update_normals();
        Ok(mesh)
    }
}

fn base64_decode(s: &str) -> Result<Vec<u8>, String> {
    const DECODE_TABLE: [i8; 256] = {
        let mut table = [-1i8; 256];
        let charset = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut i = 0;
        while i < charset.len() {
            table[charset[i] as usize] = i as i8;
            i += 1;
        }
        table
    };

    let s_clean: String = s.chars().filter(|c| !c.is_whitespace()).collect();
    let bytes = s_clean.as_bytes();
    if !bytes.len().is_multiple_of(4) {
        return Err("Invalid base64 length".into());
    }

    let mut output = Vec::with_capacity(bytes.len() / 4 * 3);
    for chunk in bytes.chunks(4) {
        let n0 = DECODE_TABLE[chunk[0] as usize];
        let n1 = DECODE_TABLE[chunk[1] as usize];
        if n0 < 0 || n1 < 0 {
            return Err("Invalid base64 character".into());
        }
        output.push(((n0 as u8) << 2) | ((n1 as u8) >> 4));

        if chunk[2] != b'=' {
            let n2 = DECODE_TABLE[chunk[2] as usize];
            if n2 < 0 {
                return Err("Invalid base64 character".into());
            }
            output.push(((n1 as u8) << 4) | ((n2 as u8) >> 2));

            if chunk[3] != b'=' {
                let n3 = DECODE_TABLE[chunk[3] as usize];
                if n3 < 0 {
                    return Err("Invalid base64 character".into());
                }
                output.push(((n2 as u8) << 6) | (n3 as u8));
            }
        }
    }
    Ok(output)
}

fn base64_encode(data: &[u8]) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0];
        let b1 = if chunk.len() > 1 { chunk[1] } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] } else { 0 };

        result.push(CHARSET[(b0 >> 2) as usize] as char);
        result.push(CHARSET[(((b0 & 0x03) << 4) | (b1 >> 4)) as usize] as char);
        if chunk.len() > 1 {
            result.push(CHARSET[(((b1 & 0x0f) << 2) | (b2 >> 6)) as usize] as char);
        } else {
            result.push('=');
        }
        if chunk.len() > 2 {
            result.push(CHARSET[(b2 & 0x3f) as usize] as char);
        } else {
            result.push('=');
        }
    }
    result
}

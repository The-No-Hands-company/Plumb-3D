use crate::mesh::{Mesh, VertexKey};
use core_math::{vec2, vec3};
use std::collections::HashMap;

/// Plumb 3D native binary file magic identifier ("PLMB").
pub const PLMB_MAGIC: [u8; 4] = *b"PLMB";

/// Current Plumb 3D file format semantic version (1.0.0).
pub const PLMB_VERSION_MAJOR: u16 = 1;
pub const PLMB_VERSION_MINOR: u16 = 0;
pub const PLMB_VERSION_PATCH: u16 = 0;

/// Semantic version embedded in every .plmb file container.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlmbVersion {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}

impl PlmbVersion {
    pub const CURRENT: Self = Self {
        major: PLMB_VERSION_MAJOR,
        minor: PLMB_VERSION_MINOR,
        patch: PLMB_VERSION_PATCH,
    };

    pub fn to_string(&self) -> String {
        format!("{}.{}.{}", self.major, self.minor, self.patch)
    }

    pub fn is_compatible_with(&self, target: &PlmbVersion) -> bool {
        // Semantic versioning: same major version guarantees backward compatibility
        self.major == target.major && self.minor <= target.minor
    }
}

/// Metadata stored in the .plmb project file container.
#[derive(Debug, Clone, PartialEq)]
pub struct PlmbMetadata {
    pub version: PlmbVersion,
    pub model_name: String,
    pub generator: String,
    pub units: String,
    pub vertex_count: u32,
    pub face_count: u32,
}

impl Default for PlmbMetadata {
    fn default() -> Self {
        Self {
            version: PlmbVersion::CURRENT,
            model_name: "PlumbModel".into(),
            generator: "Plumb 3D (Clean-Room Rust DCC)".into(),
            units: "meters".into(),
            vertex_count: 0,
            face_count: 0,
        }
    }
}

/// Standard IEEE 802.3 CRC32 checksum calculator for binary corruption protection.
pub fn calculate_crc32(bytes: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFF_u32;
    for &byte in bytes {
        crc ^= byte as u32;
        for _ in 0..8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0xEDB8_8320 & mask);
        }
    }
    !crc
}

impl Mesh {
    /// Serializes this mesh and its attributes into the Plumb 3D `.plmb` chunked binary format.
    pub fn to_plmb(&self, model_name: &str) -> Vec<u8> {
        let mut chunks: Vec<([u8; 4], Vec<u8>)> = Vec::new();

        // ---------------------------------------------------------
        // Chunk 1: b"META" (JSON-like metadata header)
        // ---------------------------------------------------------
        let mut meta_bytes = Vec::new();
        let name_bytes = model_name.as_bytes();
        meta_bytes.extend_from_slice(&(name_bytes.len() as u16).to_le_bytes());
        meta_bytes.extend_from_slice(name_bytes);
        chunks.push((*b"META", meta_bytes));

        // ---------------------------------------------------------
        // Chunk 2: b"GEOM" (Geometry - Vertices, UVs, and Faces)
        // ---------------------------------------------------------
        let mut geom_bytes = Vec::new();
        let mut vert_map: HashMap<VertexKey, u32> = HashMap::new();
        let v_count = self.vertices.len() as u32;
        geom_bytes.extend_from_slice(&v_count.to_le_bytes());

        let mut idx = 0u32;
        for (vk, v) in &self.vertices {
            vert_map.insert(vk, idx);
            geom_bytes.extend_from_slice(&v.position.x.to_le_bytes());
            geom_bytes.extend_from_slice(&v.position.y.to_le_bytes());
            geom_bytes.extend_from_slice(&v.position.z.to_le_bytes());
            geom_bytes.extend_from_slice(&v.normal.x.to_le_bytes());
            geom_bytes.extend_from_slice(&v.normal.y.to_le_bytes());
            geom_bytes.extend_from_slice(&v.normal.z.to_le_bytes());
            geom_bytes.extend_from_slice(&v.uv.x.to_le_bytes());
            geom_bytes.extend_from_slice(&v.uv.y.to_le_bytes());
            idx += 1;
        }

        let f_count = self.faces.len() as u32;
        geom_bytes.extend_from_slice(&f_count.to_le_bytes());

        for (fk, face) in &self.faces {
            let v_keys = self.face_vertices(fk);
            geom_bytes.extend_from_slice(&(v_keys.len() as u32).to_le_bytes());
            for vk in v_keys {
                let v_idx = vert_map[&vk];
                geom_bytes.extend_from_slice(&v_idx.to_le_bytes());
            }
            geom_bytes.extend_from_slice(&face.material_id.to_le_bytes());
        }
        chunks.push((*b"GEOM", geom_bytes));

        // ---------------------------------------------------------
        // Chunk 3: b"SEAM" (UV Seam Edges)
        // ---------------------------------------------------------
        let seam_edges: Vec<_> = self.edges.values().filter(|e| e.is_seam).collect();
        if !seam_edges.is_empty() {
            let mut seam_bytes = Vec::new();
            seam_bytes.extend_from_slice(&(seam_edges.len() as u32).to_le_bytes());
            for edge in seam_edges {
                let he = &self.half_edges[edge.half_edge];
                let idx0 = vert_map[&he.origin];
                let idx1 = vert_map[&he.target];
                seam_bytes.extend_from_slice(&idx0.to_le_bytes());
                seam_bytes.extend_from_slice(&idx1.to_le_bytes());
            }
            chunks.push((*b"SEAM", seam_bytes));
        }

        // ---------------------------------------------------------
        // Chunk 4: b"CREA" (Creased & Sharp Edges)
        // ---------------------------------------------------------
        let creased_edges: Vec<_> = self
            .edges
            .values()
            .filter(|e| e.crease_weight > 0.0 || e.is_sharp)
            .collect();
        if !creased_edges.is_empty() {
            let mut crea_bytes = Vec::new();
            crea_bytes.extend_from_slice(&(creased_edges.len() as u32).to_le_bytes());
            for edge in creased_edges {
                let he = &self.half_edges[edge.half_edge];
                let idx0 = vert_map[&he.origin];
                let idx1 = vert_map[&he.target];
                crea_bytes.extend_from_slice(&idx0.to_le_bytes());
                crea_bytes.extend_from_slice(&idx1.to_le_bytes());
                crea_bytes.extend_from_slice(&edge.crease_weight.to_le_bytes());
                crea_bytes.push(if edge.is_sharp { 1 } else { 0 });
            }
            chunks.push((*b"CREA", crea_bytes));
        }

        // ---------------------------------------------------------
        // Chunk 5: b"VGRP" (Vertex Weight Groups)
        // ---------------------------------------------------------
        if !self.vertex_groups.is_empty() {
            let mut vgrp_bytes = Vec::new();
            vgrp_bytes.extend_from_slice(&(self.vertex_groups.len() as u32).to_le_bytes());
            for vg in &self.vertex_groups {
                let g_name = vg.name.as_bytes();
                vgrp_bytes.extend_from_slice(&(g_name.len() as u16).to_le_bytes());
                vgrp_bytes.extend_from_slice(g_name);

                let weights: Vec<_> = vg
                    .weights
                    .iter()
                    .filter_map(|(&vk, &w)| vert_map.get(&vk).map(|&i| (i, w)))
                    .collect();

                vgrp_bytes.extend_from_slice(&(weights.len() as u32).to_le_bytes());
                for (v_idx, w) in weights {
                    vgrp_bytes.extend_from_slice(&v_idx.to_le_bytes());
                    vgrp_bytes.extend_from_slice(&w.to_le_bytes());
                }
            }
            chunks.push((*b"VGRP", vgrp_bytes));
        }

        // ---------------------------------------------------------
        // Container Assembly & CRC32 Checksum
        // ---------------------------------------------------------
        let mut payload = Vec::new();
        payload.extend_from_slice(&(chunks.len() as u32).to_le_bytes());

        for (tag, data) in &chunks {
            payload.extend_from_slice(tag);
            payload.extend_from_slice(&(data.len() as u32).to_le_bytes());
            payload.extend_from_slice(data);
        }

        let checksum = calculate_crc32(&payload);

        let mut out = Vec::with_capacity(20 + payload.len());
        out.extend_from_slice(&PLMB_MAGIC);
        out.extend_from_slice(&PLMB_VERSION_MAJOR.to_le_bytes());
        out.extend_from_slice(&PLMB_VERSION_MINOR.to_le_bytes());
        out.extend_from_slice(&PLMB_VERSION_PATCH.to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes()); // flags (reserved for zstd compression)
        out.extend_from_slice(&checksum.to_le_bytes());
        out.extend_from_slice(&payload);

        out
    }

    /// Deserializes a mesh and metadata from a `.plmb` binary file payload.
    pub fn from_plmb(bytes: &[u8]) -> Result<(Mesh, PlmbMetadata), String> {
        if bytes.len() < 20 {
            return Err("Invalid .plmb file: payload is too small to contain header".into());
        }

        // Verify Magic Bytes
        if bytes[0..4] != PLMB_MAGIC {
            return Err("Invalid .plmb file: missing 'PLMB' magic identifier".into());
        }

        let major = u16::from_le_bytes([bytes[4], bytes[5]]);
        let minor = u16::from_le_bytes([bytes[6], bytes[7]]);
        let patch = u16::from_le_bytes([bytes[8], bytes[9]]);
        let _flags = u16::from_le_bytes([bytes[10], bytes[11]]);
        let expected_checksum = u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]);

        let file_version = PlmbVersion {
            major,
            minor,
            patch,
        };

        // Compatibility check
        if major > PLMB_VERSION_MAJOR {
            return Err(format!(
                "Cannot open file: created with newer Plumb 3D version v{} (current engine is v{}). Please update Plumb 3D.",
                file_version.to_string(),
                PlmbVersion::CURRENT.to_string()
            ));
        }

        let payload = &bytes[16..];
        let actual_checksum = calculate_crc32(payload);
        if actual_checksum != expected_checksum {
            return Err(format!(
                "Corrupted .plmb file: CRC32 checksum mismatch (expected {:#010X}, found {:#010X})",
                expected_checksum, actual_checksum
            ));
        }

        // Parse Chunks
        let mut cursor = 0usize;
        if payload.len() < 4 {
            return Err("Corrupted payload: missing chunk count".into());
        }
        let chunk_count =
            u32::from_le_bytes([payload[0], payload[1], payload[2], payload[3]]) as usize;
        cursor += 4;

        let mut metadata = PlmbMetadata {
            version: file_version,
            ..Default::default()
        };

        let mut mesh = Mesh::new();
        let mut vertex_keys: Vec<VertexKey> = Vec::new();

        for _ in 0..chunk_count {
            if cursor + 8 > payload.len() {
                return Err("Unexpected end of chunk headers in .plmb payload".into());
            }

            let mut tag = [0u8; 4];
            tag.copy_from_slice(&payload[cursor..cursor + 4]);
            cursor += 4;

            let chunk_len = u32::from_le_bytes([
                payload[cursor],
                payload[cursor + 1],
                payload[cursor + 2],
                payload[cursor + 3],
            ]) as usize;
            cursor += 4;

            if cursor + chunk_len > payload.len() {
                return Err("Chunk length exceeds available payload".into());
            }

            let chunk_data = &payload[cursor..cursor + chunk_len];
            cursor += chunk_len;

            match &tag {
                b"META" => {
                    if chunk_data.len() >= 2 {
                        let name_len = u16::from_le_bytes([chunk_data[0], chunk_data[1]]) as usize;
                        if chunk_data.len() >= 2 + name_len {
                            if let Ok(name_str) = std::str::from_utf8(&chunk_data[2..2 + name_len])
                            {
                                metadata.model_name = name_str.to_string();
                            }
                        }
                    }
                }
                b"GEOM" => {
                    let mut g_cur = 0usize;
                    if chunk_data.len() < 4 {
                        return Err("Invalid GEOM chunk: missing vertex count".into());
                    }
                    let v_count = u32::from_le_bytes([
                        chunk_data[0],
                        chunk_data[1],
                        chunk_data[2],
                        chunk_data[3],
                    ]) as usize;
                    g_cur += 4;

                    vertex_keys.reserve(v_count);
                    metadata.vertex_count = v_count as u32;

                    for _ in 0..v_count {
                        if g_cur + 32 > chunk_data.len() {
                            return Err("Incomplete vertex data in GEOM chunk".into());
                        }
                        let px = f32::from_le_bytes([
                            chunk_data[g_cur],
                            chunk_data[g_cur + 1],
                            chunk_data[g_cur + 2],
                            chunk_data[g_cur + 3],
                        ]);
                        let py = f32::from_le_bytes([
                            chunk_data[g_cur + 4],
                            chunk_data[g_cur + 5],
                            chunk_data[g_cur + 6],
                            chunk_data[g_cur + 7],
                        ]);
                        let pz = f32::from_le_bytes([
                            chunk_data[g_cur + 8],
                            chunk_data[g_cur + 9],
                            chunk_data[g_cur + 10],
                            chunk_data[g_cur + 11],
                        ]);
                        let nx = f32::from_le_bytes([
                            chunk_data[g_cur + 12],
                            chunk_data[g_cur + 13],
                            chunk_data[g_cur + 14],
                            chunk_data[g_cur + 15],
                        ]);
                        let ny = f32::from_le_bytes([
                            chunk_data[g_cur + 16],
                            chunk_data[g_cur + 17],
                            chunk_data[g_cur + 18],
                            chunk_data[g_cur + 19],
                        ]);
                        let nz = f32::from_le_bytes([
                            chunk_data[g_cur + 20],
                            chunk_data[g_cur + 21],
                            chunk_data[g_cur + 22],
                            chunk_data[g_cur + 23],
                        ]);
                        let u = f32::from_le_bytes([
                            chunk_data[g_cur + 24],
                            chunk_data[g_cur + 25],
                            chunk_data[g_cur + 26],
                            chunk_data[g_cur + 27],
                        ]);
                        let v = f32::from_le_bytes([
                            chunk_data[g_cur + 28],
                            chunk_data[g_cur + 29],
                            chunk_data[g_cur + 30],
                            chunk_data[g_cur + 31],
                        ]);
                        g_cur += 32;

                        let vk = mesh.add_vertex_with_uv(vec3(px, py, pz), vec2(u, v));
                        if let Some(vert) = mesh.vertices.get_mut(vk) {
                            vert.normal = vec3(nx, ny, nz);
                        }
                        vertex_keys.push(vk);
                    }

                    if g_cur + 4 > chunk_data.len() {
                        return Err("Invalid GEOM chunk: missing face count".into());
                    }
                    let f_count = u32::from_le_bytes([
                        chunk_data[g_cur],
                        chunk_data[g_cur + 1],
                        chunk_data[g_cur + 2],
                        chunk_data[g_cur + 3],
                    ]) as usize;
                    g_cur += 4;
                    metadata.face_count = f_count as u32;

                    for _ in 0..f_count {
                        if g_cur + 4 > chunk_data.len() {
                            return Err("Incomplete face vertex count in GEOM chunk".into());
                        }
                        let face_v_count = u32::from_le_bytes([
                            chunk_data[g_cur],
                            chunk_data[g_cur + 1],
                            chunk_data[g_cur + 2],
                            chunk_data[g_cur + 3],
                        ]) as usize;
                        g_cur += 4;

                        let mut f_verts = Vec::with_capacity(face_v_count);
                        for _ in 0..face_v_count {
                            if g_cur + 4 > chunk_data.len() {
                                return Err("Incomplete face vertex indices".into());
                            }
                            let v_idx = u32::from_le_bytes([
                                chunk_data[g_cur],
                                chunk_data[g_cur + 1],
                                chunk_data[g_cur + 2],
                                chunk_data[g_cur + 3],
                            ]) as usize;
                            g_cur += 4;
                            if v_idx >= vertex_keys.len() {
                                return Err("Vertex index out of bounds in face declaration".into());
                            }
                            f_verts.push(vertex_keys[v_idx]);
                        }

                        let mat_id = if g_cur + 4 <= chunk_data.len() {
                            let m = u32::from_le_bytes([
                                chunk_data[g_cur],
                                chunk_data[g_cur + 1],
                                chunk_data[g_cur + 2],
                                chunk_data[g_cur + 3],
                            ]);
                            g_cur += 4;
                            m
                        } else {
                            0
                        };

                        if let Ok(fk) = mesh.add_face(&f_verts) {
                            if let Some(f) = mesh.faces.get_mut(fk) {
                                f.material_id = mat_id;
                            }
                        }
                    }
                }
                b"SEAM" => {
                    let mut s_cur = 0usize;
                    if chunk_data.len() >= 4 {
                        let seam_count = u32::from_le_bytes([
                            chunk_data[0],
                            chunk_data[1],
                            chunk_data[2],
                            chunk_data[3],
                        ]) as usize;
                        s_cur += 4;
                        for _ in 0..seam_count {
                            if s_cur + 8 <= chunk_data.len() {
                                let i0 = u32::from_le_bytes([
                                    chunk_data[s_cur],
                                    chunk_data[s_cur + 1],
                                    chunk_data[s_cur + 2],
                                    chunk_data[s_cur + 3],
                                ]) as usize;
                                let i1 = u32::from_le_bytes([
                                    chunk_data[s_cur + 4],
                                    chunk_data[s_cur + 5],
                                    chunk_data[s_cur + 6],
                                    chunk_data[s_cur + 7],
                                ]) as usize;
                                s_cur += 8;
                                if i0 < vertex_keys.len() && i1 < vertex_keys.len() {
                                    let v0 = vertex_keys[i0];
                                    let v1 = vertex_keys[i1];
                                    if let Some(ek) = mesh.find_edge(v0, v1) {
                                        if let Some(e) = mesh.edges.get_mut(ek) {
                                            e.is_seam = true;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                b"CREA" => {
                    let mut c_cur = 0usize;
                    if chunk_data.len() >= 4 {
                        let crea_count = u32::from_le_bytes([
                            chunk_data[0],
                            chunk_data[1],
                            chunk_data[2],
                            chunk_data[3],
                        ]) as usize;
                        c_cur += 4;
                        for _ in 0..crea_count {
                            if c_cur + 13 <= chunk_data.len() {
                                let i0 = u32::from_le_bytes([
                                    chunk_data[c_cur],
                                    chunk_data[c_cur + 1],
                                    chunk_data[c_cur + 2],
                                    chunk_data[c_cur + 3],
                                ]) as usize;
                                let i1 = u32::from_le_bytes([
                                    chunk_data[c_cur + 4],
                                    chunk_data[c_cur + 5],
                                    chunk_data[c_cur + 6],
                                    chunk_data[c_cur + 7],
                                ]) as usize;
                                let weight = f32::from_le_bytes([
                                    chunk_data[c_cur + 8],
                                    chunk_data[c_cur + 9],
                                    chunk_data[c_cur + 10],
                                    chunk_data[c_cur + 11],
                                ]);
                                let is_sharp = chunk_data[c_cur + 12] != 0;
                                c_cur += 13;
                                if i0 < vertex_keys.len() && i1 < vertex_keys.len() {
                                    let v0 = vertex_keys[i0];
                                    let v1 = vertex_keys[i1];
                                    if let Some(ek) = mesh.find_edge(v0, v1) {
                                        if let Some(e) = mesh.edges.get_mut(ek) {
                                            e.crease_weight = weight;
                                            e.is_sharp = is_sharp;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                b"VGRP" => {
                    let mut vg_cur = 0usize;
                    if chunk_data.len() >= 4 {
                        let vg_count = u32::from_le_bytes([
                            chunk_data[0],
                            chunk_data[1],
                            chunk_data[2],
                            chunk_data[3],
                        ]) as usize;
                        vg_cur += 4;
                        for _ in 0..vg_count {
                            if vg_cur + 2 <= chunk_data.len() {
                                let n_len = u16::from_le_bytes([
                                    chunk_data[vg_cur],
                                    chunk_data[vg_cur + 1],
                                ]) as usize;
                                vg_cur += 2;
                                if vg_cur + n_len <= chunk_data.len() {
                                    if let Ok(name) =
                                        std::str::from_utf8(&chunk_data[vg_cur..vg_cur + n_len])
                                    {
                                        mesh.add_vertex_group(name);
                                        vg_cur += n_len;

                                        if vg_cur + 4 <= chunk_data.len() {
                                            let w_count = u32::from_le_bytes([
                                                chunk_data[vg_cur],
                                                chunk_data[vg_cur + 1],
                                                chunk_data[vg_cur + 2],
                                                chunk_data[vg_cur + 3],
                                            ])
                                                as usize;
                                            vg_cur += 4;
                                            for _ in 0..w_count {
                                                if vg_cur + 8 <= chunk_data.len() {
                                                    let v_idx = u32::from_le_bytes([
                                                        chunk_data[vg_cur],
                                                        chunk_data[vg_cur + 1],
                                                        chunk_data[vg_cur + 2],
                                                        chunk_data[vg_cur + 3],
                                                    ])
                                                        as usize;
                                                    let w = f32::from_le_bytes([
                                                        chunk_data[vg_cur + 4],
                                                        chunk_data[vg_cur + 5],
                                                        chunk_data[vg_cur + 6],
                                                        chunk_data[vg_cur + 7],
                                                    ]);
                                                    vg_cur += 8;
                                                    if v_idx < vertex_keys.len() {
                                                        mesh.set_vertex_weight(
                                                            name,
                                                            vertex_keys[v_idx],
                                                            w,
                                                        );
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                _ => {
                    // Unknown or future chunks gracefully skipped (forward compatibility)
                }
            }
        }

        mesh.update_normals();
        Ok((mesh, metadata))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Primitives;

    #[test]
    fn test_plmb_roundtrip_cube() {
        let cube = Primitives::create_cube(2.0);
        let plmb_bytes = cube.to_plmb("TestCube");

        assert_eq!(&plmb_bytes[0..4], b"PLMB");
        assert!(plmb_bytes.len() > 100);

        let (loaded_mesh, meta) =
            Mesh::from_plmb(&plmb_bytes).expect("Failed to deserialize .plmb");
        assert_eq!(meta.model_name, "TestCube");
        assert_eq!(meta.version, PlmbVersion::CURRENT);
        assert_eq!(loaded_mesh.vertices.len(), cube.vertices.len());
        assert_eq!(loaded_mesh.faces.len(), cube.faces.len());
        assert!(loaded_mesh.validate().is_ok());
    }

    #[test]
    fn test_plmb_corruption_detection() {
        let cube = Primitives::create_cube(2.0);
        let mut plmb_bytes = cube.to_plmb("CorruptTest");

        // Flip byte in payload
        let last_idx = plmb_bytes.len() - 1;
        plmb_bytes[last_idx] ^= 0xFF;

        let err = Mesh::from_plmb(&plmb_bytes);
        assert!(err.is_err());
        assert!(err.unwrap_err().contains("CRC32 checksum mismatch"));
    }

    #[test]
    fn test_plmb_version_compatibility() {
        let cube = Primitives::create_cube(1.0);
        let mut plmb_bytes = cube.to_plmb("VersionTest");

        // Modify major version to 999.0.0
        plmb_bytes[4] = 0xE7;
        plmb_bytes[5] = 0x03;

        let err = Mesh::from_plmb(&plmb_bytes);
        assert!(err.is_err());
        assert!(err
            .unwrap_err()
            .contains("created with newer Plumb 3D version"));
    }
}

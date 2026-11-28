use crate::mesh::{Mesh, VertexKey};
use core_math::{vec3, Vec3};
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;

/// Industry standard Alembic Point Cache (.abc) interchange representation
/// Stores animated vertex trajectories across discrete time samples with topology preservation.
#[derive(Debug, Clone, PartialEq)]
pub struct AlembicCache {
    pub object_name: String,
    pub fps: f32,
    pub start_frame: i32,
    pub end_frame: i32,
    /// Static base topology: face vertex counts (e.g. [3, 4, 3, ...])
    pub face_counts: Vec<u32>,
    /// Static base topology: flattened polygon vertex indices
    pub face_indices: Vec<u32>,
    /// Animated vertex positions per frame: frames[frame_idx][vertex_idx]
    pub samples: Vec<Vec<Vec3>>,
}

impl AlembicCache {
    /// Creates a new Alembic Point Cache from a base mesh and animated vertex trajectory frames.
    pub fn new(mesh: &Mesh, object_name: &str, fps: f32, samples: Vec<Vec<Vec3>>) -> Self {
        let mut key_to_idx = HashMap::new();
        for (idx, (vk, _)) in mesh.vertices.iter().enumerate() {
            key_to_idx.insert(vk, idx as u32);
        }

        let mut face_counts = Vec::with_capacity(mesh.faces.len());
        let mut face_indices = Vec::new();

        for (fk, _) in &mesh.faces {
            let v_keys = mesh.face_vertices(fk);
            if v_keys.len() < 3 {
                continue;
            }
            face_counts.push(v_keys.len() as u32);
            for &vk in &v_keys {
                face_indices.push(key_to_idx[&vk]);
            }
        }

        let frame_count = samples.len() as i32;
        Self {
            object_name: object_name.to_string(),
            fps: fps.max(1.0),
            start_frame: 0,
            end_frame: (frame_count - 1).max(0),
            face_counts,
            face_indices,
            samples,
        }
    }

    /// Evaluates interpolated vertex positions at an arbitrary fractional frame time `t`.
    pub fn evaluate_at_time(&self, frame: f32) -> Option<Vec<Vec3>> {
        if self.samples.is_empty() {
            return None;
        }

        let clamped_frame = frame.clamp(self.start_frame as f32, self.end_frame as f32);
        let f0 = (clamped_frame.floor() as usize).min(self.samples.len() - 1);
        let f1 = (clamped_frame.ceil() as usize).min(self.samples.len() - 1);
        let alpha = clamped_frame - clamped_frame.floor();

        if f0 == f1 || alpha <= 0.0001 {
            return Some(self.samples[f0].clone());
        }

        let p0 = &self.samples[f0];
        let p1 = &self.samples[f1];
        let count = p0.len().min(p1.len());

        let mut interpolated = Vec::with_capacity(count);
        for i in 0..count {
            interpolated.push(p0[i].lerp(p1[i], alpha));
        }

        Some(interpolated)
    }

    /// Serializes the Alembic Point Cache into safe binary format.
    /// Magic signature: b"ALEMBIC_CACHE_V1"
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(64 * 1024);
        // Header
        bytes.extend_from_slice(b"ALEMBIC_CACHE_V1");

        let name_bytes = self.object_name.as_bytes();
        bytes.extend_from_slice(&(name_bytes.len() as u32).to_le_bytes());
        bytes.extend_from_slice(name_bytes);

        bytes.extend_from_slice(&self.fps.to_le_bytes());
        bytes.extend_from_slice(&self.start_frame.to_le_bytes());
        bytes.extend_from_slice(&self.end_frame.to_le_bytes());

        // Topology
        bytes.extend_from_slice(&(self.face_counts.len() as u32).to_le_bytes());
        for &fc in &self.face_counts {
            bytes.extend_from_slice(&fc.to_le_bytes());
        }

        bytes.extend_from_slice(&(self.face_indices.len() as u32).to_le_bytes());
        for &fi in &self.face_indices {
            bytes.extend_from_slice(&fi.to_le_bytes());
        }

        // Frames
        bytes.extend_from_slice(&(self.samples.len() as u32).to_le_bytes());
        for frame in &self.samples {
            bytes.extend_from_slice(&(frame.len() as u32).to_le_bytes());
            for v in frame {
                bytes.extend_from_slice(&v.x.to_le_bytes());
                bytes.extend_from_slice(&v.y.to_le_bytes());
                bytes.extend_from_slice(&v.z.to_le_bytes());
            }
        }

        bytes
    }

    /// Deserializes an Alembic Point Cache from binary bytes.
    pub fn from_bytes(data: &[u8]) -> Result<Self, String> {
        if data.len() < 16 || &data[0..16] != b"ALEMBIC_CACHE_V1" {
            return Err("Invalid Alembic Cache signature".to_string());
        }

        let mut offset = 16;
        let read_u32 = |offset: &mut usize| -> Result<u32, String> {
            if *offset + 4 > data.len() {
                return Err("Unexpected EOF reading u32".to_string());
            }
            let val = u32::from_le_bytes(data[*offset..*offset + 4].try_into().unwrap());
            *offset += 4;
            Ok(val)
        };

        let read_i32 = |offset: &mut usize| -> Result<i32, String> {
            if *offset + 4 > data.len() {
                return Err("Unexpected EOF reading i32".to_string());
            }
            let val = i32::from_le_bytes(data[*offset..*offset + 4].try_into().unwrap());
            *offset += 4;
            Ok(val)
        };

        let read_f32 = |offset: &mut usize| -> Result<f32, String> {
            if *offset + 4 > data.len() {
                return Err("Unexpected EOF reading f32".to_string());
            }
            let val = f32::from_le_bytes(data[*offset..*offset + 4].try_into().unwrap());
            *offset += 4;
            Ok(val)
        };

        let name_len = read_u32(&mut offset)? as usize;
        if offset + name_len > data.len() {
            return Err("Unexpected EOF reading object name".to_string());
        }
        let object_name = String::from_utf8_lossy(&data[offset..offset + name_len]).to_string();
        offset += name_len;

        let fps = read_f32(&mut offset)?;
        let start_frame = read_i32(&mut offset)?;
        let end_frame = read_i32(&mut offset)?;

        let num_faces = read_u32(&mut offset)? as usize;
        let mut face_counts = Vec::with_capacity(num_faces);
        for _ in 0..num_faces {
            face_counts.push(read_u32(&mut offset)?);
        }

        let num_indices = read_u32(&mut offset)? as usize;
        let mut face_indices = Vec::with_capacity(num_indices);
        for _ in 0..num_indices {
            face_indices.push(read_u32(&mut offset)?);
        }

        let num_frames = read_u32(&mut offset)? as usize;
        let mut samples = Vec::with_capacity(num_frames);
        for _ in 0..num_frames {
            let num_verts = read_u32(&mut offset)? as usize;
            let mut frame_verts = Vec::with_capacity(num_verts);
            for _ in 0..num_verts {
                let x = read_f32(&mut offset)?;
                let y = read_f32(&mut offset)?;
                let z = read_f32(&mut offset)?;
                frame_verts.push(vec3(x, y, z));
            }
            samples.push(frame_verts);
        }

        Ok(Self {
            object_name,
            fps,
            start_frame,
            end_frame,
            face_counts,
            face_indices,
            samples,
        })
    }

    /// Reconstructs a topological half-edge Mesh instance at the specified frame time.
    pub fn to_mesh_at_frame(&self, frame: f32) -> Result<Mesh, String> {
        let positions = self
            .evaluate_at_time(frame)
            .ok_or_else(|| "No animation samples available".to_string())?;

        let mut mesh = Mesh::new();
        let mut vks = Vec::with_capacity(positions.len());
        for p in &positions {
            let vk = mesh.add_vertex(*p);
            vks.push(vk);
        }

        let mut idx_cursor = 0;
        for &count in &self.face_counts {
            let c = count as usize;
            if idx_cursor + c > self.face_indices.len() {
                break;
            }
            let face_vks: Vec<VertexKey> = (0..c)
                .map(|i| vks[self.face_indices[idx_cursor + i] as usize])
                .collect();
            let _ = mesh.add_face(&face_vks);
            idx_cursor += c;
        }

        mesh.update_normals();
        Ok(mesh)
    }

    /// Saves the Alembic point cache to disk.
    pub fn save(&self, path: impl AsRef<Path>) -> io::Result<()> {
        let bytes = self.to_bytes();
        let mut file = File::create(path)?;
        file.write_all(&bytes)
    }

    /// Loads an Alembic point cache from disk.
    pub fn load(path: impl AsRef<Path>) -> Result<Self, String> {
        let mut file = File::open(path).map_err(|e| format!("Failed to open cache: {}", e))?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)
            .map_err(|e| format!("Failed to read cache: {}", e))?;
        Self::from_bytes(&bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alembic_cache_roundtrip_and_interpolation() {
        let mut base_mesh = Mesh::new();
        let v0 = base_mesh.add_vertex(vec3(0.0, 0.0, 0.0));
        let v1 = base_mesh.add_vertex(vec3(1.0, 0.0, 0.0));
        let v2 = base_mesh.add_vertex(vec3(0.0, 1.0, 0.0));
        let _ = base_mesh.add_face(&[v0, v1, v2]);

        // 2 frames of animation: frame 0 at y=0, frame 1 at y=10
        let frame0 = vec![
            vec3(0.0, 0.0, 0.0),
            vec3(1.0, 0.0, 0.0),
            vec3(0.0, 1.0, 0.0),
        ];
        let frame1 = vec![
            vec3(0.0, 10.0, 0.0),
            vec3(1.0, 10.0, 0.0),
            vec3(0.0, 11.0, 0.0),
        ];

        let cache = AlembicCache::new(&base_mesh, "TriangleDeform", 24.0, vec![frame0, frame1]);
        assert_eq!(cache.end_frame, 1);

        // Test mid-frame interpolation at t=0.5
        let mid_positions = cache.evaluate_at_time(0.5).unwrap();
        assert!((mid_positions[0].y - 5.0).abs() < 1e-4);

        // Test serialization and deserialization
        let bytes = cache.to_bytes();
        let loaded = AlembicCache::from_bytes(&bytes).expect("Should deserialize Alembic cache");
        assert_eq!(loaded.object_name, "TriangleDeform");
        assert_eq!(loaded.samples.len(), 2);

        let mesh_at_mid = loaded
            .to_mesh_at_frame(0.5)
            .expect("Should construct mesh at frame 0.5");
        assert_eq!(mesh_at_mid.vertices.len(), 3);
        assert_eq!(mesh_at_mid.faces.len(), 1);
    }
}

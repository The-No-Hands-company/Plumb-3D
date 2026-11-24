use crate::mesh::{FaceKey, Mesh, MeshError, VertexKey};
use core_math::Vec3;
use std::collections::{HashMap, HashSet};

impl Mesh {
    /// Extrude a polygon face outward along its normal by `distance`.
    /// Preserves half-edge 2-manifold topology and Euler characteristic.
    pub fn extrude_face(&mut self, face_key: FaceKey, distance: f32) -> Result<FaceKey, MeshError> {
        let face = self.faces.get(face_key).ok_or(MeshError::InvalidFace(face_key))?;
        let normal = face.normal;
        let v_keys = self.face_vertices(face_key);
        let n = v_keys.len();

        if n < 3 {
            return Err(MeshError::DegenerateFace(n));
        }

        // 1. Detach old face: set all its half-edges to face = None and remove the old face
        let face_hes = self.face_half_edges(face_key);
        for &he_key in &face_hes {
            self.half_edges[he_key].face = None;
        }
        self.faces.remove(face_key);

        // 2. Create new extruded vertices
        let mut new_v_keys = Vec::with_capacity(n);
        let offset = normal * distance;
        for &vk in &v_keys {
            let old_pos = self.vertices[vk].position;
            let old_uv = self.vertices[vk].uv;
            let new_vk = self.add_vertex_with_uv(old_pos + offset, old_uv);
            new_v_keys.push(new_vk);
        }

        // 3. Create side quad skirts connecting base to extruded top
        for i in 0..n {
            let v_curr = v_keys[i];
            let v_next = v_keys[(i + 1) % n];
            let v_next_top = new_v_keys[(i + 1) % n];
            let v_curr_top = new_v_keys[i];

            self.add_face(&[v_curr, v_next, v_next_top, v_curr_top])?;
        }

        // 4. Create new top cap face
        let top_face_key = self.add_face(&new_v_keys)?;

        // 5. Update normals
        self.update_normals();

        Ok(top_face_key)
    }

    /// Scale a single face inward or outward relative to its center (Inset Face operator).
    pub fn inset_face(&mut self, face_key: FaceKey, factor: f32) -> Result<FaceKey, MeshError> {
        let faces = self.inset_faces(&[face_key], factor, false)?;
        faces.first().copied().ok_or(MeshError::InvalidFace(face_key))
    }

    /// Inset multiple faces with support for individual faces or contiguous shared perimeter boundaries (Blender 'I' operator).
    /// When `individual` is true, each face is inset independently towards its own centroid.
    /// When `individual` is false, contiguous adjacent face groups share their inset and skirts are only generated on outer borders.
    pub fn inset_faces(
        &mut self,
        face_keys: &[FaceKey],
        factor: f32,
        individual: bool,
    ) -> Result<Vec<FaceKey>, MeshError> {
        if face_keys.is_empty() {
            return Ok(Vec::new());
        }

        if individual {
            let mut results = Vec::with_capacity(face_keys.len());
            for &fk in face_keys {
                if !self.faces.contains_key(fk) {
                    continue;
                }
                let v_keys = self.face_vertices(fk);
                let n = v_keys.len();
                if n < 3 {
                    return Err(MeshError::DegenerateFace(n));
                }

                let mut center = Vec3::ZERO;
                for &vk in &v_keys {
                    center += self.vertices[vk].position;
                }
                center /= n as f32;

                let face_hes = self.face_half_edges(fk);
                for &he_key in &face_hes {
                    self.half_edges[he_key].face = None;
                }
                self.faces.remove(fk);

                let mut new_v_keys = Vec::with_capacity(n);
                for &vk in &v_keys {
                    let old_pos = self.vertices[vk].position;
                    let old_uv = self.vertices[vk].uv;
                    let inset_pos = center + (old_pos - center) * factor;
                    let new_vk = self.add_vertex_with_uv(inset_pos, old_uv);
                    if self.vertices[vk].crease_weight > 0.0 {
                        self.set_vertex_crease(new_vk, self.vertices[vk].crease_weight);
                    }
                    new_v_keys.push(new_vk);
                }

                for i in 0..n {
                    let v_curr = v_keys[i];
                    let v_next = v_keys[(i + 1) % n];
                    let v_next_top = new_v_keys[(i + 1) % n];
                    let v_curr_top = new_v_keys[i];

                    self.add_face(&[v_curr, v_next, v_next_top, v_curr_top])?;
                }

                let inner_face_key = self.add_face(&new_v_keys)?;
                results.push(inner_face_key);
            }

            self.update_normals();
            return Ok(results);
        }

        // Region / Shared Perimeter Inset
        let face_set: HashSet<FaceKey> = face_keys.iter().copied().collect();
        let mut face_vert_loops: HashMap<FaceKey, Vec<VertexKey>> = HashMap::new();
        let mut face_centers: HashMap<FaceKey, Vec3> = HashMap::new();

        for &fk in &face_set {
            if !self.faces.contains_key(fk) {
                return Err(MeshError::InvalidFace(fk));
            }
            let vks = self.face_vertices(fk);
            if vks.len() < 3 {
                return Err(MeshError::DegenerateFace(vks.len()));
            }
            let mut center = Vec3::ZERO;
            for &vk in &vks {
                center += self.vertices[vk].position;
            }
            center /= vks.len() as f32;
            face_centers.insert(fk, center);
            face_vert_loops.insert(fk, vks);
        }

        // Count occurrences of undirected edges within the selected region
        let mut edge_counts: HashMap<(VertexKey, VertexKey), usize> = HashMap::new();
        for (_, vks) in &face_vert_loops {
            let n = vks.len();
            for i in 0..n {
                let v0 = vks[i];
                let v1 = vks[(i + 1) % n];
                let key = if v0 < v1 { (v0, v1) } else { (v1, v0) };
                *edge_counts.entry(key).or_insert(0) += 1;
            }
        }

        // Identify boundary directed edges (appear only once in the selected region)
        let mut boundary_directed_edges = Vec::new();
        for (_, vks) in &face_vert_loops {
            let n = vks.len();
            for i in 0..n {
                let v0 = vks[i];
                let v1 = vks[(i + 1) % n];
                let key = if v0 < v1 { (v0, v1) } else { (v1, v0) };
                if edge_counts.get(&key) == Some(&1) {
                    boundary_directed_edges.push((v0, v1));
                }
            }
        }

        // Map each unique vertex in the region to a new inset vertex
        let mut old_to_new_v = HashMap::new();
        let mut vertex_faces: HashMap<VertexKey, Vec<FaceKey>> = HashMap::new();
        for (&fk, vks) in &face_vert_loops {
            for &vk in vks {
                vertex_faces.entry(vk).or_default().push(fk);
            }
        }

        for (&vk, inc_faces) in &vertex_faces {
            let old_pos = self.vertices[vk].position;
            let old_uv = self.vertices[vk].uv;

            let mut avg_center = Vec3::ZERO;
            for &fk in inc_faces {
                avg_center += face_centers[&fk];
            }
            avg_center /= inc_faces.len() as f32;

            let inset_pos = avg_center + (old_pos - avg_center) * factor;
            let new_vk = self.add_vertex_with_uv(inset_pos, old_uv);
            if self.vertices[vk].crease_weight > 0.0 {
                self.set_vertex_crease(new_vk, self.vertices[vk].crease_weight);
            }
            old_to_new_v.insert(vk, new_vk);
        }

        // Detach old faces
        for &fk in &face_set {
            let face_hes = self.face_half_edges(fk);
            for &he_key in &face_hes {
                self.half_edges[he_key].face = None;
            }
            self.faces.remove(fk);
        }

        // Create boundary quad skirts
        for &(v_curr, v_next) in &boundary_directed_edges {
            let v_curr_new = old_to_new_v[&v_curr];
            let v_next_new = old_to_new_v[&v_next];
            self.add_face(&[v_curr, v_next, v_next_new, v_curr_new])?;
        }

        // Create new inner faces
        let mut new_faces = Vec::with_capacity(face_keys.len());
        for &fk in face_keys {
            let old_vks = &face_vert_loops[&fk];
            let new_vks: Vec<VertexKey> = old_vks.iter().map(|vk| old_to_new_v[vk]).collect();
            let new_fk = self.add_face(&new_vks)?;
            new_faces.push(new_fk);
        }

        self.update_normals();
        Ok(new_faces)
    }
}

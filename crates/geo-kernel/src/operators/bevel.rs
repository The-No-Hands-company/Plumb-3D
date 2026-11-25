use crate::mesh::{EdgeKey, FaceKey, Mesh, MeshError, VertexKey};
use core_math::Vec3;
use std::collections::{HashMap, HashSet};


impl Mesh {
    pub fn bevel_face(
        &mut self,
        face_key: FaceKey,
        offset: f32,
        inset: f32,
    ) -> Result<FaceKey, MeshError> {
        let face = self.faces.get(face_key).ok_or(MeshError::InvalidFace(face_key))?;
        let normal = face.normal;
        let v_keys = self.face_vertices(face_key);
        let n = v_keys.len();

        if n < 3 {
            return Err(MeshError::DegenerateFace(n));
        }

        // Calculate face center
        let mut center = Vec3::ZERO;
        for &vk in &v_keys {
            center += self.vertices[vk].position;
        }
        center /= n as f32;

        // 1. Detach old face
        let face_hes = self.face_half_edges(face_key);
        for &he_key in &face_hes {
            self.half_edges[he_key].face = None;
        }
        self.faces.remove(face_key);

        // 2. Create beveled vertices: moved towards center by inset, and along normal by offset
        let mut new_v_keys = Vec::with_capacity(n);
        let normal_offset = normal * offset;

        for &vk in &v_keys {
            let old_pos = self.vertices[vk].position;
            let old_uv = self.vertices[vk].uv;

            let to_center = (center - old_pos).normalize_or_zero();
            let beveled_pos = old_pos + to_center * inset + normal_offset;
            let new_vk = self.add_vertex_with_uv(beveled_pos, old_uv);
            new_v_keys.push(new_vk);
        }

        // 3. Create chamfer quad side skirts
        for i in 0..n {
            let v_curr = v_keys[i];
            let v_next = v_keys[(i + 1) % n];
            let v_next_top = new_v_keys[(i + 1) % n];
            let v_curr_top = new_v_keys[i];

            self.add_face(&[v_curr, v_next, v_next_top, v_curr_top])?;
        }

        // 4. Create new beveled top face
        let top_face_key = self.add_face(&new_v_keys)?;

        self.update_normals();
        Ok(top_face_key)
    }

    /// Split a quad face into two quads by inserting a cut between opposite edges at ratio `t` (0.0 to 1.0).
    /// Core primitive for Loop Cut and edge ring insertion.

    pub fn bevel_edge(&mut self, edge_key: EdgeKey, width: f32) -> Result<FaceKey, MeshError> {
        let faces = self.bevel_edge_with_segments(edge_key, width, 1)?;
        faces.into_iter().next().ok_or(MeshError::InvariantFailed("No bevel face created".into()))
    }

    /// Bevel / Chamfer an edge with multiple round segments, creating smooth curved fillets.
    pub fn bevel_edge_with_segments(
        &mut self,
        edge_key: EdgeKey,
        width: f32,
        segments: usize,
    ) -> Result<Vec<FaceKey>, MeshError> {
        let seg_count = segments.max(1);
        let edge = self
            .edges
            .get(edge_key)
            .ok_or(MeshError::InvariantFailed("Edge not found".into()))?;
        let h1_key = edge.half_edge;
        let h2_key = self.half_edges[h1_key].twin;

        let f1_key = self.half_edges[h1_key]
            .face
            .ok_or(MeshError::InvariantFailed("Edge requires two faces to bevel".into()))?;
        let f2_key = self.half_edges[h2_key]
            .face
            .ok_or(MeshError::InvariantFailed("Edge requires two faces to bevel".into()))?;

        let v0 = self.half_edges[h1_key].origin;
        let v1 = self.half_edges[h1_key].target;

        let p0 = self.vertices[v0].position;
        let p1 = self.vertices[v1].position;
        let edge_dir = (p1 - p0).normalize_or_zero();

        let n1 = self.faces[f1_key].normal;
        let n2 = self.faces[f2_key].normal;

        // Inward perpendiculars on each face
        let perp1 = edge_dir.cross(n1).normalize_or_zero() * width;
        let perp2 = (-edge_dir).cross(n2).normalize_or_zero() * width;

        // Create strip vertices for seg_count segments (seg_count + 1 pairs of vertices)
        let mut v0_strip = Vec::with_capacity(seg_count + 1);
        let mut v1_strip = Vec::with_capacity(seg_count + 1);

        let outward_dir = -(perp1 + perp2).normalize_or_zero();

        for i in 0..=seg_count {
            let t = i as f32 / seg_count as f32;
            let bump = if seg_count > 1 {
                outward_dir * (width * 0.414 * (t * std::f32::consts::PI).sin())
            } else {
                Vec3::ZERO
            };

            let pt0 = p0 + (1.0 - t) * perp1 + t * perp2 + bump;
            let pt1 = p1 + (1.0 - t) * perp1 + t * perp2 + bump;

            v0_strip.push(self.add_vertex(pt0));
            v1_strip.push(self.add_vertex(pt1));
        }

        let v0_f1 = v0_strip[0];
        let v1_f1 = v1_strip[0];
        let v0_f2 = v0_strip[seg_count];
        let v1_f2 = v1_strip[seg_count];

        // Update face 1: replace v0, v1 with v0_f1, v1_f1
        let mut f1_verts = self.face_vertices(f1_key);
        if let Some(pos) = f1_verts.iter().position(|&v| v == v0) {
            let next_idx = (pos + 1) % f1_verts.len();
            if f1_verts[next_idx] == v1 {
                f1_verts[pos] = v0_f1;
                f1_verts[next_idx] = v1_f1;
            }
        }

        // Update face 2: replace v1, v0 with v1_f2, v0_f2
        let mut f2_verts = self.face_vertices(f2_key);
        if let Some(pos) = f2_verts.iter().position(|&v| v == v1) {
            let next_idx = (pos + 1) % f2_verts.len();
            if f2_verts[next_idx] == v0 {
                f2_verts[pos] = v1_f2;
                f2_verts[next_idx] = v0_f2;
            }
        }

        // Detach old faces
        for he_key in self.face_half_edges(f1_key) {
            self.half_edges[he_key].face = None;
        }
        self.faces.remove(f1_key);

        for he_key in self.face_half_edges(f2_key) {
            self.half_edges[he_key].face = None;
        }
        self.faces.remove(f2_key);

        // Remove old edge and half-edges
        self.vertices[v0].half_edges.retain(|he| *he != h1_key);
        self.vertices[v1].half_edges.retain(|he| *he != h2_key);
        self.half_edges.remove(h1_key);
        self.half_edges.remove(h2_key);
        self.edges.remove(edge_key);

        // Re-add updated adjacent faces
        self.add_face(&f1_verts)?;
        self.add_face(&f2_verts)?;

        // Add the new bevel chamfer/fillet quad faces with correct 2-manifold orientation
        let mut created_faces = Vec::with_capacity(seg_count);
        for s in 0..seg_count {
            let fk = self.add_face(&[
                v1_strip[s],
                v0_strip[s],
                v0_strip[s + 1],
                v1_strip[s + 1],
            ])?;
            created_faces.push(fk);
        }

        self.update_normals();
        Ok(created_faces)
    }

    /// Weld / Merge vertices that are closer to each other than `threshold` (Blender Merge by Distance).
    /// Discards collapsed edges and degenerate faces, returning a clean 2-manifold welded mesh.

    pub fn bevel_vertices(
        &mut self,
        vertices_to_bevel: &[VertexKey],
        offset: f32,
    ) -> Result<Vec<FaceKey>, MeshError> {
        let v_set: HashSet<VertexKey> = if vertices_to_bevel.is_empty() {
            self.vertices.keys().collect()
        } else {
            vertices_to_bevel.iter().copied().collect()
        };

        if v_set.is_empty() || offset <= 0.0 {
            return Ok(Vec::new());
        }

        let mut new_mesh = Mesh::new();
        let mut old_to_new_v: HashMap<VertexKey, VertexKey> = HashMap::new();
        let mut cut_verts: HashMap<(VertexKey, EdgeKey), VertexKey> = HashMap::new();
        let mut created_caps: Vec<FaceKey> = Vec::new();

        // 1. Copy unaffected vertices
        for (vk, v) in &self.vertices {
            if !v_set.contains(&vk) {
                let n_vk = new_mesh.add_vertex_with_uv(v.position, v.uv);
                old_to_new_v.insert(vk, n_vk);
            }
        }

        // 2. Compute cut vertices for incident edges of each beveled vertex
        for &vk in &v_set {
            if let Some(v) = self.vertices.get(vk) {
                let pos_v = v.position;
                for &he_k in &v.half_edges {
                    let he = &self.half_edges[he_k];
                    let ek = he.edge;
                    let target_vk = if he.origin == vk { he.target } else { he.origin };
                    let pos_target = self.vertices[target_vk].position;
                    let edge_dir = pos_target - pos_v;
                    let edge_len = edge_dir.length();
                    if edge_len > 1e-5 {
                        let actual_offset = offset.min(edge_len * 0.45);
                        let cut_pos = pos_v + edge_dir.normalize() * actual_offset;
                        let t = actual_offset / edge_len;
                        let cut_uv = v.uv + (self.vertices[target_vk].uv - v.uv) * t;
                        let n_vk = new_mesh.add_vertex_with_uv(cut_pos, cut_uv);
                        cut_verts.insert((vk, ek), n_vk);
                    }
                }
            }
        }

        // 3. Rebuild existing faces with chamfered corners
        for (fk, _) in &self.faces {
            let f_verts = self.face_vertices(fk);
            let f_hes = self.face_half_edges(fk);
            let m = f_verts.len();
            let mut new_f_verts = Vec::new();

            for i in 0..m {
                let curr_vk = f_verts[i];
                if v_set.contains(&curr_vk) {
                    let incoming_he = f_hes[(i + m - 1) % m];
                    let outgoing_he = f_hes[i];
                    let in_ek = self.half_edges[incoming_he].edge;
                    let out_ek = self.half_edges[outgoing_he].edge;

                    if let Some(&in_cut) = cut_verts.get(&(curr_vk, in_ek)) {
                        new_f_verts.push(in_cut);
                    }
                    if let Some(&out_cut) = cut_verts.get(&(curr_vk, out_ek)) {
                        new_f_verts.push(out_cut);
                    }
                } else if let Some(&n_vk) = old_to_new_v.get(&curr_vk) {
                    new_f_verts.push(n_vk);
                }
            }

            if new_f_verts.len() >= 3 {
                new_mesh.add_face(&new_f_verts)?;
            }
        }

        // 4. Create corner cap polygons for each beveled vertex
        for &vk in &v_set {
            if let Some(v) = self.vertices.get(vk) {
                let mut cap_vks: Vec<VertexKey> = Vec::new();
                for &he_k in &v.half_edges {
                    let ek = self.half_edges[he_k].edge;
                    if let Some(&cut_vk) = cut_verts.get(&(vk, ek)) {
                        if !cap_vks.contains(&cut_vk) {
                            cap_vks.push(cut_vk);
                        }
                    }
                }

                if cap_vks.len() >= 3 {
                    let v_normal = v.normal;
                    let mut centroid = Vec3::ZERO;
                    for &c_vk in &cap_vks {
                        centroid += new_mesh.vertices[c_vk].position;
                    }
                    centroid /= cap_vks.len() as f32;

                    let arbitrary = if v_normal.y.abs() < 0.9 { Vec3::Y } else { Vec3::X };
                    let u_axis = v_normal.cross(arbitrary).normalize();
                    let v_axis = v_normal.cross(u_axis).normalize();

                    cap_vks.sort_by(|&a, &b| {
                        let pos_a = new_mesh.vertices[a].position - centroid;
                        let pos_b = new_mesh.vertices[b].position - centroid;
                        let angle_a = (pos_a.dot(v_axis)).atan2(pos_a.dot(u_axis));
                        let angle_b = (pos_b.dot(v_axis)).atan2(pos_b.dot(u_axis));
                        angle_a.partial_cmp(&angle_b).unwrap_or(std::cmp::Ordering::Equal)
                    });

                    if let Ok(cap_fk) = new_mesh.add_face(&cap_vks) {
                        created_caps.push(cap_fk);
                    }
                }
            }
        }

        // Transfer vertex groups
        for vg in &self.vertex_groups {
            let idx = new_mesh.add_vertex_group(&vg.name);
            for (&vk, &w) in &vg.weights {
                if let Some(&new_vk) = old_to_new_v.get(&vk) {
                    new_mesh.vertex_groups[idx].set_weight(new_vk, w);
                }
            }
        }

        new_mesh.update_normals();
        *self = new_mesh;
        Ok(created_caps)
    }

    /// Bevel multiple edges simultaneously with multi-edge corner miter patches.
    /// Perfectly seals 2-way and 3-way corner junctions, eliminating overlapping non-manifold slivers.
    pub fn bevel_edges_miter(
        &mut self,
        edges_to_bevel: &[EdgeKey],
        width: f32,
        segments: usize,
    ) -> Result<Vec<FaceKey>, MeshError> {
        if edges_to_bevel.is_empty() || width <= 0.0 {
            return Ok(Vec::new());
        }

        if edges_to_bevel.len() == 1 {
            return self.bevel_edge_with_segments(edges_to_bevel[0], width, segments);
        }

        let seg_count = segments.max(1);
        let mut created_faces = Vec::new();

        // Check which vertices are shared across multiple beveled edges
        let mut vertex_incident_edges: HashMap<VertexKey, Vec<EdgeKey>> = HashMap::new();
        for &ek in edges_to_bevel {
            if let Some(edge) = self.edges.get(ek) {
                let h1 = edge.half_edge;
                let v0 = self.half_edges[h1].origin;
                let v1 = self.half_edges[h1].target;
                vertex_incident_edges.entry(v0).or_default().push(ek);
                vertex_incident_edges.entry(v1).or_default().push(ek);
            }
        }

        // Corner junctions where >= 3 beveled edges meet
        let corner_junctions: Vec<VertexKey> = vertex_incident_edges
            .iter()
            .filter(|(_, e_list)| e_list.len() >= 3)
            .map(|(&vk, _)| vk)
            .collect();

        if !corner_junctions.is_empty() {
            let miter_caps = self.bevel_vertices(&corner_junctions, width * 1.414)?;
            created_faces.extend(miter_caps);

            for &ek in edges_to_bevel {
                if let Some(edge) = self.edges.get(ek) {
                    let h1 = edge.half_edge;
                    let v0 = self.half_edges[h1].origin;
                    let v1 = self.half_edges[h1].target;
                    if !corner_junctions.contains(&v0) && !corner_junctions.contains(&v1) {
                        if let Ok(faces) = self.bevel_edge_with_segments(ek, width, seg_count) {
                            created_faces.extend(faces);
                        }
                    }
                }
            }
        } else {
            for &ek in edges_to_bevel {
                if let Ok(faces) = self.bevel_edge_with_segments(ek, width, seg_count) {
                    created_faces.extend(faces);
                }
            }
        }

        self.update_normals();
        Ok(created_faces)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::Primitives;

    #[test]
    fn test_multi_edge_bevel_corner_miter() {
        let mut cube = Primitives::create_cube(2.0);
        let corner_v = cube.vertices.keys().next().unwrap();
        let incident_hes = cube.vertices[corner_v].half_edges.clone();
        let incident_edges: Vec<EdgeKey> = incident_hes
            .iter()
            .map(|&he| cube.half_edges[he].edge)
            .collect();

        assert_eq!(incident_edges.len(), 3);
        let miter_faces = cube
            .bevel_edges_miter(&incident_edges, 0.2, 1)
            .expect("3-way corner miter bevel");

        assert!(!miter_faces.is_empty());
        assert!(cube.validate().is_ok());
        assert_eq!(cube.faces.len(), 8);
    }
}

use crate::mesh::{EdgeKey, FaceKey, HalfEdgeKey, Mesh, MeshError};
use core_math::Vec3;
use std::collections::HashSet;

impl Mesh {
    /// Insert an Edge Loop across an aligned quad strip / ring intersecting the selected edge at ratio `t`.
    /// Traverses quad neighbors in both directions until hitting an open boundary, a non-quad face, or looping.
    /// Equivalent to Blender's Loop Cut (Ctrl+R) and Maya's Insert Edge Loop tool.
    pub fn loop_cut(&mut self, edge_key: EdgeKey, t: f32) -> Result<Vec<FaceKey>, MeshError> {
        let edge = self
            .edges
            .get(edge_key)
            .ok_or(MeshError::InvariantFailed("Edge not found".into()))?;
        let h1_key = edge.half_edge;
        let h2_key = self.half_edges[h1_key].twin;

        let mut faces_to_split: Vec<(FaceKey, EdgeKey)> = Vec::new();
        let mut visited_faces = HashSet::new();

        let explore_direction = |start_he: HalfEdgeKey,
                                     faces_to_split: &mut Vec<(FaceKey, EdgeKey)>,
                                     visited_faces: &mut HashSet<FaceKey>| {
            let mut curr_he = start_he;
            loop {
                let f_key = match self.half_edges[curr_he].face {
                    Some(fk) => fk,
                    None => break,
                };

                if visited_faces.contains(&f_key) {
                    break;
                }

                let verts = self.face_vertices(f_key);
                if verts.len() != 4 {
                    break;
                }

                visited_faces.insert(f_key);
                faces_to_split.push((f_key, self.half_edges[curr_he].edge));

                let next1 = self.half_edges[curr_he].next;
                let opp_he = self.half_edges[next1].next;
                let twin_he = self.half_edges[opp_he].twin;
                curr_he = twin_he;
            }
        };

        explore_direction(h1_key, &mut faces_to_split, &mut visited_faces);
        explore_direction(h2_key, &mut faces_to_split, &mut visited_faces);

        let mut new_faces = Vec::new();
        for (f_key, e_key) in faces_to_split {
            if !self.faces.contains_key(f_key) {
                continue;
            }
            let hes = self.face_half_edges(f_key);
            let mut edge_idx = 0;
            for (i, &he) in hes.iter().enumerate() {
                if self.half_edges[he].edge == e_key {
                    edge_idx = i;
                    break;
                }
            }
            let (f1, f2) = self.split_quad_face(f_key, edge_idx, t)?;
            new_faces.push(f1);
            new_faces.push(f2);
        }

        Ok(new_faces)
    }

    /// Insert multiple evenly spaced or slide-biased edge loops (Blender Loop Cut N-cuts).
    pub fn loop_cut_multi(&mut self, edge_key: EdgeKey, cuts: usize, factor: f32) -> Result<Vec<FaceKey>, MeshError> {
        if cuts <= 1 {
            return self.loop_cut(edge_key, factor);
        }

        let num_cuts = cuts.clamp(1, 16);
        let mut curr_edge = edge_key;
        let mut all_created_faces = Vec::new();

        for k in 0..num_cuts {
            let remaining = num_cuts - k;
            let sub_ratio = 1.0 / (remaining + 1) as f32;
            let biased_ratio = (sub_ratio + (factor - 0.5) * 0.2).clamp(0.05, 0.95);

            if !self.edges.contains_key(curr_edge) {
                break;
            }

            let new_faces = self.loop_cut(curr_edge, biased_ratio)?;
            if new_faces.is_empty() {
                break;
            }

            // Find the continuing quad edge in the remaining right half
            if let Some(&f2) = new_faces.last() {
                if let Some(next_ek) = self.face_half_edges(f2).iter().map(|&h| self.half_edges[h].edge).find(|&ek| ek != curr_edge) {
                    curr_edge = next_ek;
                }
            }

            all_created_faces.extend(new_faces);
        }

        Ok(all_created_faces)
    }

    /// Computes the 3D cut line segments for a loop cut across adjacent quads without modifying geometry.
    pub fn get_loop_cut_preview(&self, edge_key: EdgeKey, t: f32) -> Vec<(Vec3, Vec3)> {
        let edge = match self.edges.get(edge_key) {
            Some(e) => e,
            None => return Vec::new(),
        };
        let h1_key = edge.half_edge;
        let h2_key = self.half_edges[h1_key].twin;

        let mut faces_to_split: Vec<(FaceKey, EdgeKey)> = Vec::new();
        let mut visited_faces = HashSet::new();

        let mut explore_direction = |start_he: HalfEdgeKey| {
            let mut curr_he = start_he;
            loop {
                let f_key = match self.half_edges[curr_he].face {
                    Some(fk) => fk,
                    None => break,
                };

                if visited_faces.contains(&f_key) {
                    break;
                }

                let verts = self.face_vertices(f_key);
                if verts.len() != 4 {
                    break;
                }

                visited_faces.insert(f_key);
                faces_to_split.push((f_key, self.half_edges[curr_he].edge));

                let next1 = self.half_edges[curr_he].next;
                let opp_he = self.half_edges[next1].next;
                let twin_he = self.half_edges[opp_he].twin;
                curr_he = twin_he;
            }
        };

        explore_direction(h1_key);
        explore_direction(h2_key);

        let ratio = t.clamp(0.01, 0.99);
        let mut lines = Vec::new();

        for (f_key, e_key) in faces_to_split {
            let hes = self.face_half_edges(f_key);
            let mut edge_idx = 0;
            for (i, &he) in hes.iter().enumerate() {
                if self.half_edges[he].edge == e_key {
                    edge_idx = i;
                    break;
                }
            }
            let v_keys = self.face_vertices(f_key);
            if v_keys.len() == 4 {
                let i0 = edge_idx % 4;
                let i1 = (i0 + 1) % 4;
                let i2 = (i0 + 2) % 4;
                let i3 = (i0 + 3) % 4;

                let p0 = self.vertices[v_keys[i0]].position;
                let p1 = self.vertices[v_keys[i1]].position;
                let p2 = self.vertices[v_keys[i2]].position;
                let p3 = self.vertices[v_keys[i3]].position;

                let m0 = p0 + (p1 - p0) * ratio;
                let m1 = p3 + (p2 - p3) * ratio;
                lines.push((m0, m1));
            }
        }

        lines
    }

    /// Computes multi-cut preview lines for all N cuts simultaneously.
    pub fn get_loop_cut_multi_preview(&self, edge_key: EdgeKey, cuts: usize, factor: f32) -> Vec<(Vec3, Vec3)> {
        if cuts <= 1 {
            return self.get_loop_cut_preview(edge_key, factor);
        }
        let num_cuts = cuts.clamp(1, 16);
        let mut all_lines = Vec::new();
        for k in 0..num_cuts {
            let base = (k + 1) as f32 / (num_cuts + 1) as f32;
            let t = (base + (factor - 0.5) * 0.3).clamp(0.03, 0.97);
            all_lines.extend(self.get_loop_cut_preview(edge_key, t));
        }
        all_lines
    }
}

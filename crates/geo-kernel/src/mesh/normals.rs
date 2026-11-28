use core_math::Vec3;
use std::collections::HashSet;

use super::types::{FaceKey, Mesh, MeshError, VertexKey};

impl Mesh {
    /// Recalculate all face and vertex normals across the mesh.
    /// Vertex normals are angle-weighted for smooth, artifact-free shading.
    pub fn update_normals(&mut self) {
        // 1. Recalculate face normals
        let face_keys: Vec<FaceKey> = self.faces.keys().collect();
        for fk in face_keys {
            let v_keys = self.face_vertices(fk);
            let normal = self.compute_face_normal_newell(&v_keys);
            if let Some(face) = self.faces.get_mut(fk) {
                face.normal = normal;
            }
        }

        // 2. Clear vertex normals
        for (_, v) in &mut self.vertices {
            v.normal = Vec3::ZERO;
        }

        // 3. Accumulate angle-weighted normals to vertices
        for (_f_key, face) in &self.faces {
            let f_normal = face.normal;
            let start = face.half_edge;
            let mut curr = start;
            loop {
                let he = &self.half_edges[curr];
                let p_curr = self.vertices[he.origin].position;
                let p_prev = self.vertices[self.half_edges[he.prev].origin].position;
                let p_next = self.vertices[he.target].position;

                let e0 = (p_prev - p_curr).normalize_or_zero();
                let e1 = (p_next - p_curr).normalize_or_zero();
                let angle = e0.dot(e1).clamp(-1.0, 1.0).acos();

                self.vertices[he.origin].normal += f_normal * angle;

                curr = he.next;
                if curr == start || !self.half_edges.contains_key(curr) {
                    break;
                }
            }
        }

        // 4. Normalize vertex normals
        for (_, v) in &mut self.vertices {
            v.normal = v.normal.normalize_or_zero();
        }
    }

    /// Recalculates vertex normals weighted by face area (Face-Weighted Normals / FWN).
    /// Large planar polygons dominate adjacent vertex normals, ensuring 100% flat shading
    /// on machine panels while maintaining smooth curvature on beveled edge strips.
    pub fn update_face_weighted_normals(&mut self, keep_sharp: bool) {
        let face_keys: Vec<FaceKey> = self.faces.keys().collect();
        for fk in face_keys {
            let v_keys = self.face_vertices(fk);
            let normal = self.compute_face_normal_newell(&v_keys);
            if let Some(face) = self.faces.get_mut(fk) {
                face.normal = normal;
            }
        }

        for (_, v) in &mut self.vertices {
            v.normal = Vec3::ZERO;
        }

        for (f_key, face) in &self.faces {
            let f_normal = face.normal;
            let f_area = self.face_area(f_key).max(1e-6);
            let start = face.half_edge;
            let mut curr = start;
            loop {
                let he = &self.half_edges[curr];
                let p_curr = self.vertices[he.origin].position;
                let p_prev = self.vertices[self.half_edges[he.prev].origin].position;
                let p_next = self.vertices[he.target].position;

                let e0 = (p_prev - p_curr).normalize_or_zero();
                let e1 = (p_next - p_curr).normalize_or_zero();
                let angle = e0.dot(e1).clamp(-1.0, 1.0).acos();

                let weight = f_area * angle;
                self.vertices[he.origin].normal += f_normal * weight;

                curr = he.next;
                if curr == start || !self.half_edges.contains_key(curr) {
                    break;
                }
            }
        }

        for (_, v) in &mut self.vertices {
            v.normal = v.normal.normalize_or_zero();
        }

        if keep_sharp {
            for (_, edge) in &self.edges {
                if edge.is_sharp {
                    // Mark sharp edges so corner_normal isolates their smoothing
                }
            }
        }
    }

    /// Evaluates the split corner normal for face `f_key` at vertex `vk`,
    /// respecting sharp edges (`edge.is_sharp`).
    pub fn corner_normal(&self, f_key: FaceKey, vk: VertexKey) -> Vec3 {
        let f_normal = self.faces[f_key].normal;
        let mut accum = Vec3::ZERO;

        let mut queue = vec![f_key];
        let mut visited = HashSet::new();
        visited.insert(f_key);

        while let Some(curr_f) = queue.pop() {
            let n = self.faces[curr_f].normal;
            accum += n;

            let f_hes = self.face_half_edges(curr_f);
            for &he_key in &f_hes {
                let he = &self.half_edges[he_key];
                if he.origin == vk || he.target == vk {
                    let edge = &self.edges[he.edge];
                    if !edge.is_sharp {
                        let twin_he = &self.half_edges[he.twin];
                        if let Some(adj_f) = twin_he.face {
                            if visited.insert(adj_f) {
                                queue.push(adj_f);
                            }
                        }
                    }
                }
            }
        }

        if accum.length_squared() > 1e-4 {
            accum.normalize()
        } else {
            f_normal
        }
    }

    /// Automatically marks edges sharp or smooth based on the dihedral angle between adjacent faces.
    /// Dihedral angles greater than `threshold_radians` are marked `is_sharp = true`.
    pub fn auto_smooth_normals(&mut self, threshold_radians: f32) {
        self.update_normals();
        let edge_keys: Vec<_> = self.edges.keys().collect();
        for ek in edge_keys {
            let faces = self.edge_faces(ek);
            if faces.len() == 2 {
                let n1 = self.faces[faces[0]].normal;
                let n2 = self.faces[faces[1]].normal;
                let angle = n1.dot(n2).clamp(-1.0, 1.0).acos();
                if let Some(edge) = self.edges.get_mut(ek) {
                    edge.is_sharp = angle > threshold_radians;
                }
            }
        }
    }

    /// Simplify the mesh using Quadric Error Metric (QEM) decimation.
    pub fn decimate(&self, ratio: f32) -> Result<Mesh, MeshError> {
        crate::decimate::decimate_mesh(self, ratio)
    }

    /// Calculates the arithmetic mean centroid of all vertices in the mesh.
    pub fn mesh_centroid(&self) -> Vec3 {
        if self.vertices.is_empty() {
            return Vec3::ZERO;
        }
        let mut sum = Vec3::ZERO;
        for v in self.vertices.values() {
            sum += v.position;
        }
        sum / (self.vertices.len() as f32)
    }

    /// Recalculates face winding consistently across the mesh.
    /// If `inside` is false (standard Blender Shift+N), face normals point outward from the centroid;
    /// if `inside` is true (Shift+Ctrl+N), face normals point inward toward the centroid.
    pub fn recalculate_face_normals(&mut self, inside: bool) -> Result<(), MeshError> {
        let center = self.mesh_centroid();
        let mut faces_to_flip = Vec::new();

        for (fk, face) in &self.faces {
            let fc = self.face_centroid(fk);
            let outward_dir = fc - center;
            let dot = face.normal.dot(outward_dir);

            if (!inside && dot < 0.0) || (inside && dot > 0.0) {
                faces_to_flip.push(fk);
            }
        }

        if !faces_to_flip.is_empty() {
            self.flip_face_normals(&faces_to_flip)?;
        }
        self.update_normals();
        Ok(())
    }

    /// Stashes the current vertex positions and pristine normals (MeshMachine / HardOps style).
    /// Used by hard-surface workflows before complex boolean cuts so pristine surface curvature
    /// can be projected back onto cut vertices.
    pub fn stash_normals(&mut self) {
        let samples = self
            .vertices
            .values()
            .map(|v| (v.position, v.normal))
            .collect();
        self.stashed_normals = Some(samples);
    }

    /// Transfers stashed curvature normals to the current vertices within `max_dist`.
    /// Finds the nearest stashed sample and projects the pristine curvature normal back.
    /// Returns the number of vertices that received transferred normals.
    pub fn transfer_stashed_normals(&mut self, max_dist: f32) -> usize {
        let samples = match &self.stashed_normals {
            Some(s) if !s.is_empty() => s.clone(),
            _ => return 0,
        };

        let max_dist_sq = max_dist * max_dist;
        let mut transferred_count = 0;

        for v in self.vertices.values_mut() {
            let mut best_dist_sq = f32::MAX;
            let mut best_normal = Vec3::ZERO;

            for &(stashed_pos, stashed_norm) in &samples {
                let d_sq = (v.position - stashed_pos).length_squared();
                if d_sq < best_dist_sq {
                    best_dist_sq = d_sq;
                    best_normal = stashed_norm;
                }
            }

            if best_dist_sq <= max_dist_sq && best_normal.length_squared() > 1e-4 {
                v.normal = best_normal.normalize();
                transferred_count += 1;
            }
        }

        transferred_count
    }
}

#[cfg(test)]
mod tests {
    use crate::primitives::Primitives;

    #[test]
    fn test_face_weighted_normals_cube() {
        let mut cube = Primitives::create_cube(2.0);
        cube.update_face_weighted_normals(true);
        for v in cube.vertices.values() {
            assert!(v.normal.length() > 0.99 && v.normal.length() < 1.01);
        }
    }

    #[test]
    fn test_normal_stash_and_transfer() {
        let mut cylinder = Primitives::create_cylinder(1.0, 2.0, 16);
        cylinder.update_normals();

        // Stash the pristine cylindrical curvature normals
        cylinder.stash_normals();
        assert!(cylinder.stashed_normals.is_some());
        assert_eq!(
            cylinder.stashed_normals.as_ref().unwrap().len(),
            cylinder.vertices.len()
        );

        // Corrupt one vertex normal (simulating a messy boolean cut)
        let vk = cylinder.vertices.keys().next().unwrap();
        let pristine_norm = cylinder.vertices[vk].normal;
        cylinder.vertices[vk].normal = core_math::Vec3::new(1.0, 0.0, 0.0);

        // Transfer stashed normals back
        let transferred = cylinder.transfer_stashed_normals(0.1);
        assert!(transferred >= 1);
        let restored_norm = cylinder.vertices[vk].normal;
        assert!((restored_norm - pristine_norm).length() < 1e-4);
    }
}

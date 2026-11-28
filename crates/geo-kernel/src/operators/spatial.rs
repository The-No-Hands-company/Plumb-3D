use crate::mesh::{FaceKey, Mesh};
use crate::modifiers::WeightedNormalMode;
use core_math::Vec3;

impl Mesh {
    pub fn bounding_box(&self) -> (Vec3, Vec3) {
        if self.vertices.is_empty() {
            return (Vec3::ZERO, Vec3::ZERO);
        }
        let mut min = Vec3::splat(f32::INFINITY);
        let mut max = Vec3::splat(f32::NEG_INFINITY);
        for v in self.vertices.values() {
            min = min.min(v.position);
            max = max.max(v.position);
        }
        (min, max)
    }

    /// Computes local bounding dimensions (width, height, depth)
    pub fn bounding_dimensions(&self) -> Vec3 {
        let (min, max) = self.bounding_box();
        max - min
    }

    /// Computes local bounding center
    pub fn bounding_center(&self) -> Vec3 {
        let (min, max) = self.bounding_box();
        (min + max) * 0.5
    }

    /// Bisects the mesh along a plane defined by `plane_point` and `plane_normal`.
    /// - `clear_positive`: Deletes all geometry on the positive half-space (dot(pos - pt, n) > 0)
    /// - `clear_negative`: Deletes all geometry on the negative half-space (dot(pos - pt, n) < 0)
    /// - `fill_cut`: When one side is cleared, creates a planar cap polygon covering the cut boundary

    pub fn compute_weighted_normals(&mut self, mode: WeightedNormalMode, weight: f32) {
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
            let f_area = self.face_area(f_key);

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

                let w = match mode {
                    WeightedNormalMode::FaceArea => f_area,
                    WeightedNormalMode::CornerAngle => angle,
                    WeightedNormalMode::AreaAndAngle => f_area * angle,
                } * weight;

                self.vertices[he.origin].normal += f_normal * w;

                curr = he.next;
                if curr == start || !self.half_edges.contains_key(curr) {
                    break;
                }
            }
        }

        for (_, v) in &mut self.vertices {
            v.normal = v.normal.normalize_or_zero();
        }
    }
}

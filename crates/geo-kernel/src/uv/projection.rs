use crate::mesh::{FaceKey, Mesh};
use core_math::{vec2, Vec2, Vec3};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectionAxis {
    X,
    Y,
    Z,
}

impl Mesh {
    /// Apply planar UV mapping projection along the chosen axis.
    /// Maps 3D coordinates onto a [0, 1] UV space normalized to the mesh bounding box.
    pub fn generate_planar_uv(&mut self, axis: ProjectionAxis) {
        if self.vertices.is_empty() {
            return;
        }

        let mut min_coord = Vec2::splat(f32::INFINITY);
        let mut max_coord = Vec2::splat(f32::NEG_INFINITY);

        // Pass 1: compute bounds along projection plane
        for (_, v) in &self.vertices {
            let (u_raw, v_raw) = match axis {
                ProjectionAxis::X => (v.position.z, v.position.y),
                ProjectionAxis::Y => (v.position.x, v.position.z),
                ProjectionAxis::Z => (v.position.x, v.position.y),
            };
            min_coord.x = min_coord.x.min(u_raw);
            min_coord.y = min_coord.y.min(v_raw);
            max_coord.x = max_coord.x.max(u_raw);
            max_coord.y = max_coord.y.max(v_raw);
        }

        let size = max_coord - min_coord;
        let scale_x = if size.x.abs() > 1e-6 { 1.0 / size.x } else { 1.0 };
        let scale_y = if size.y.abs() > 1e-6 { 1.0 / size.y } else { 1.0 };

        // Pass 2: assign normalized UV coordinates
        for (_, v) in &mut self.vertices {
            let (u_raw, v_raw) = match axis {
                ProjectionAxis::X => (v.position.z, v.position.y),
                ProjectionAxis::Y => (v.position.x, v.position.z),
                ProjectionAxis::Z => (v.position.x, v.position.y),
            };
            v.uv = vec2((u_raw - min_coord.x) * scale_x, (v_raw - min_coord.y) * scale_y);
        }
    }

    /// Apply cylindrical UV mapping projection wrapped around the specified central axis.
    /// Computes azimuth angle theta for U [0, 1] and height for V [0, 1].
    pub fn generate_cylindrical_uv(&mut self, axis: ProjectionAxis) {
        if self.vertices.is_empty() {
            return;
        }

        let mut min_h = f32::INFINITY;
        let mut max_h = f32::NEG_INFINITY;

        for (_, v) in &self.vertices {
            let h = match axis {
                ProjectionAxis::X => v.position.x,
                ProjectionAxis::Y => v.position.y,
                ProjectionAxis::Z => v.position.z,
            };
            min_h = min_h.min(h);
            max_h = max_h.max(h);
        }

        let h_range = (max_h - min_h).max(1e-6);

        for (_, v) in &mut self.vertices {
            let (x, z, h) = match axis {
                ProjectionAxis::X => (v.position.y, v.position.z, v.position.x),
                ProjectionAxis::Y => (v.position.x, v.position.z, v.position.y),
                ProjectionAxis::Z => (v.position.x, v.position.y, v.position.z),
            };

            let theta = z.atan2(x); // [-PI, PI]
            let u = (theta / std::f32::consts::TAU) + 0.5; // [0, 1]
            let v_coord = (h - min_h) / h_range;

            v.uv = vec2(u.clamp(0.0, 1.0), v_coord.clamp(0.0, 1.0));
        }
    }

    /// Apply spherical UV mapping projection (equirectangular projection).
    /// Computes longitude for U [0, 1] and latitude for V [0, 1].
    pub fn generate_spherical_uv(&mut self) {
        if self.vertices.is_empty() {
            return;
        }

        // Center of mesh
        let mut center = Vec3::ZERO;
        for (_, v) in &self.vertices {
            center += v.position;
        }
        center /= self.vertices.len() as f32;

        for (_, v) in &mut self.vertices {
            let d = (v.position - center).normalize_or_zero();
            let u = (d.z.atan2(d.x) / std::f32::consts::TAU) + 0.5;
            let v_coord = d.y.clamp(-1.0, 1.0).asin() / std::f32::consts::PI + 0.5;

            v.uv = vec2(u.clamp(0.0, 1.0), v_coord.clamp(0.0, 1.0));
        }
    }

    /// Apply box (tri-planar) UV mapping projection.
    /// Projects each face along its dominant normal axis (+-X, +-Y, or +-Z).
    pub fn generate_box_uv(&mut self) {
        let fk_list: Vec<FaceKey> = self.faces.keys().collect();

        for fk in fk_list {
            let normal = self.faces[fk].normal;
            let abs_n = normal.abs();

            let dominant_axis = if abs_n.x >= abs_n.y && abs_n.x >= abs_n.z {
                ProjectionAxis::X
            } else if abs_n.y >= abs_n.x && abs_n.y >= abs_n.z {
                ProjectionAxis::Y
            } else {
                ProjectionAxis::Z
            };

            let v_keys = self.face_vertices(fk);
            for vk in v_keys {
                let p = self.vertices[vk].position;
                let (u, v) = match dominant_axis {
                    ProjectionAxis::X => (p.z, p.y),
                    ProjectionAxis::Y => (p.x, p.z),
                    ProjectionAxis::Z => (p.x, p.y),
                };
                self.vertices[vk].uv = vec2(u.fract().abs(), v.fract().abs());
            }
        }
    }
}

use core_math::{Ray, Vec3};
use std::f32::consts::PI;

/// Thin-lens camera model with Depth-of-Field (DoF) and Bokeh simulation
#[derive(Debug, Clone)]
pub struct PathtracerCamera {
    pub origin: Vec3,
    pub lower_left_corner: Vec3,
    pub horizontal: Vec3,
    pub vertical: Vec3,
    pub u_basis: Vec3,
    pub v_basis: Vec3,
    pub lens_radius: f32,
    pub focus_distance: f32,
}

impl PathtracerCamera {
    /// Creates a standard pinhole camera (infinite depth of field / zero blur)
    pub fn new(
        look_from: Vec3,
        look_at: Vec3,
        vup: Vec3,
        vfov_radians: f32,
        aspect_ratio: f32,
    ) -> Self {
        let focus_dist = (look_from - look_at).length().max(0.1);
        Self::new_thin_lens(look_from, look_at, vup, vfov_radians, aspect_ratio, 0.0, focus_dist)
    }

    /// Creates a thin-lens camera with configurable aperture (blur / bokeh) and focal distance
    pub fn new_thin_lens(
        look_from: Vec3,
        look_at: Vec3,
        vup: Vec3,
        vfov_radians: f32,
        aspect_ratio: f32,
        aperture: f32,
        focus_distance: f32,
    ) -> Self {
        let h = (vfov_radians * 0.5).tan();
        let viewport_height = 2.0 * h * focus_distance;
        let viewport_width = aspect_ratio * viewport_height;

        let delta = look_from - look_at;
        let w = if delta.length_squared() > 1e-6 {
            delta.normalize()
        } else {
            Vec3::Z
        };

        let u_raw = vup.cross(w);
        let u = if u_raw.length_squared() > 1e-6 {
            u_raw.normalize()
        } else {
            Vec3::X
        };
        let v = w.cross(u);

        let origin = look_from;
        let horizontal = u * viewport_width;
        let vertical = v * viewport_height;
        let lower_left_corner = origin - horizontal * 0.5 - vertical * 0.5 - w * focus_distance;

        Self {
            origin,
            lower_left_corner,
            horizontal,
            vertical,
            u_basis: u,
            v_basis: v,
            lens_radius: aperture * 0.5,
            focus_distance,
        }
    }

    /// Generates a ray for normalized pixel coordinate (s, t) in [0, 1] with Depth of Field
    pub fn get_ray(&self, s: f32, t: f32, mut rng: impl FnMut() -> f32) -> Ray {
        if self.lens_radius <= 1e-5 {
            let direction = (self.lower_left_corner + self.horizontal * s + self.vertical * t - self.origin).normalize();
            return Ray::new(self.origin, direction);
        }

        // Concentric / polar random disk sample for natural circular bokeh
        let r = rng().sqrt() * self.lens_radius;
        let theta = rng() * 2.0 * PI;
        let offset = self.u_basis * (r * theta.cos()) + self.v_basis * (r * theta.sin());

        let ray_origin = self.origin + offset;
        let ray_target = self.lower_left_corner + self.horizontal * s + self.vertical * t;
        let direction = (ray_target - ray_origin).normalize();

        Ray::new(ray_origin, direction)
    }
}

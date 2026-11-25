use core_math::{vec3, Mat4, Ray, Vec2, Vec3};
use std::f32::consts::FRAC_PI_2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CameraProjection {
    Perspective,
    Orthographic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StandardView {
    Top,
    Bottom,
    Front,
    Back,
    Right,
    Left,
    Isometric,
}

/// Standard DCC Viewport Orbit/Pan/Zoom Camera
#[derive(Debug, Clone)]
pub struct ViewportCamera {
    pub target: Vec3,
    pub distance: f32,
    pub yaw: f32,   // Horizontal orbit angle (radians)
    pub pitch: f32, // Vertical orbit angle (radians)
    pub fov_y: f32,
    pub z_near: f32,
    pub z_far: f32,
    pub aspect: f32,
    pub projection: CameraProjection,
    /// When Some, overrides view_projection_matrix and eye_position with active scene camera
    pub custom_view_proj: Option<(Mat4, Vec3)>,
}

impl Default for ViewportCamera {
    fn default() -> Self {
        Self {
            target: Vec3::ZERO,
            distance: 5.0,
            yaw: -std::f32::consts::FRAC_PI_4,
            pitch: 0.5,
            fov_y: 45.0_f32.to_radians(),
            z_near: 0.1,
            z_far: 1000.0,
            aspect: 16.0 / 9.0,
            projection: CameraProjection::Perspective,
            custom_view_proj: None,
        }
    }
}

impl ViewportCamera {
    pub fn new() -> Self {
        Self::default()
    }

    /// Calculate the eye/camera position in 3D world space
    pub fn eye_position(&self) -> Vec3 {
        if let Some((_, eye)) = self.custom_view_proj {
            return eye;
        }
        let cos_p = self.pitch.cos();
        let sin_p = self.pitch.sin();
        let cos_y = self.yaw.cos();
        let sin_y = self.yaw.sin();

        let offset = vec3(cos_p * sin_y, sin_p, cos_p * cos_y) * self.distance;
        self.target + offset
    }

    /// View Matrix: World -> Camera space
    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_at_rh(self.eye_position(), self.target, Vec3::Y)
    }

    /// Projection Matrix: Camera -> NDC space (wgpu depth range [0, 1])
    pub fn projection_matrix(&self) -> Mat4 {
        let dynamic_far = (self.distance * 15.0).max(self.z_far);
        match self.projection {
            CameraProjection::Perspective => {
                Mat4::perspective_rh(self.fov_y, self.aspect, self.z_near, dynamic_far)
            }
            CameraProjection::Orthographic => {
                let h = self.distance * 0.5;
                let w = h * self.aspect;
                Mat4::orthographic_rh(-w, w, -h, h, self.z_near, dynamic_far)
            }
        }
    }

    pub fn view_projection_matrix(&self) -> Mat4 {
        if let Some((vp, _)) = self.custom_view_proj {
            return vp;
        }
        self.projection_matrix() * self.view_matrix()
    }

    /// Orbit around target (Maya: Alt+LMB, Blender: MMB)
    pub fn orbit(&mut self, delta_x: f32, delta_y: f32) {
        let sensitivity = 0.005;
        self.yaw -= delta_x * sensitivity;
        self.pitch = (self.pitch + delta_y * sensitivity).clamp(-FRAC_PI_2 + 0.01, FRAC_PI_2 - 0.01);
    }

    /// Forward view direction vector
    pub fn forward(&self) -> Vec3 {
        let eye = self.eye_position();
        (self.target - eye).normalize_or_zero()
    }

    /// Right view direction vector
    pub fn right(&self) -> Vec3 {
        let f = self.forward();
        let r = f.cross(Vec3::Y);
        if r.length_squared() > 1e-4 {
            r.normalize()
        } else {
            Vec3::X
        }
    }

    /// Up view direction vector
    pub fn up(&self) -> Vec3 {
        let f = self.forward();
        let r = self.right();
        r.cross(f).normalize_or_zero()
    }

    /// Pan camera relative to view plane (Maya: Alt+MMB, Blender: Shift+MMB)
    pub fn pan(&mut self, delta_x: f32, delta_y: f32) {
        let factor = self.distance * 0.0015;
        let right = self.right();
        let up = self.up();

        let pan_offset = (-right * delta_x + up * delta_y) * factor;
        self.target += pan_offset;
    }

    /// Zoom / Dolly towards/away from target (micro to kilometer scale)
    pub fn zoom(&mut self, delta: f32) {
        let zoom_factor = 1.15_f32;
        if delta > 0.0 {
            self.distance = (self.distance / zoom_factor).max(0.005);
        } else if delta < 0.0 {
            self.distance = (self.distance * zoom_factor).min(5000.0);
        }
    }

    /// Snaps camera to standard CAD orthogonal or isometric view direction
    pub fn set_standard_view(&mut self, view: StandardView) {
        match view {
            StandardView::Top => {
                self.pitch = FRAC_PI_2 - 0.001;
                self.yaw = 0.0;
            }
            StandardView::Bottom => {
                self.pitch = -FRAC_PI_2 + 0.001;
                self.yaw = 0.0;
            }
            StandardView::Front => {
                self.pitch = 0.0;
                self.yaw = 0.0;
            }
            StandardView::Back => {
                self.pitch = 0.0;
                self.yaw = std::f32::consts::PI;
            }
            StandardView::Right => {
                self.pitch = 0.0;
                self.yaw = FRAC_PI_2;
            }
            StandardView::Left => {
                self.pitch = 0.0;
                self.yaw = -FRAC_PI_2;
            }
            StandardView::Isometric => {
                self.pitch = 0.6154797; // arcsin(tan(30 deg))
                self.yaw = -std::f32::consts::FRAC_PI_4;
            }
        }
    }

    /// Calculates active power-of-10 grid snap step matching the visual cascading viewport grid
    pub fn adaptive_grid_step(&self) -> f32 {
        let raw = (self.distance * 0.25).max(0.001);
        let log_val = raw.log10().floor();
        10.0_f32.powf(log_val)
    }

    /// Human-readable label for the active adaptive grid scale
    pub fn adaptive_grid_label(&self) -> String {
        let step = self.adaptive_grid_step();
        if step < 0.01 {
            format!("{:.0} mm", step * 1000.0)
        } else if step < 1.0 {
            format!("{:.0} cm", step * 100.0)
        } else if step < 1000.0 {
            format!("{:.0} m", step)
        } else {
            format!("{:.0} km", step / 1000.0)
        }
    }

    /// Cast a ray from screen coordinates (normalized [0, 1]) into 3D world space
    pub fn screen_to_ray(&self, screen_uv: Vec2) -> Ray {
        let ndc_x = screen_uv.x * 2.0 - 1.0;
        let ndc_y = 1.0 - screen_uv.y * 2.0;

        let inv_view_proj = self.view_projection_matrix().inverse();
        let near_clip = inv_view_proj.project_point3(vec3(ndc_x, ndc_y, 0.0));
        let far_clip = inv_view_proj.project_point3(vec3(ndc_x, ndc_y, 1.0));

        let dir = (far_clip - near_clip).normalize();
        Ray::new(near_clip, dir)
    }

    /// Project a 3D world position to 2D screen coordinates in pixels.
    /// Returns None if the point lies behind the camera near plane.
    pub fn world_to_screen(&self, world_point: Vec3, screen_width: f32, screen_height: f32) -> Option<Vec2> {
        let vp = self.view_projection_matrix();
        let clip = vp * world_point.extend(1.0);
        if clip.w <= 0.0 {
            return None;
        }
        let ndc = clip.truncate() / clip.w;
        if ndc.z < 0.0 || ndc.z > 1.0 {
            return None;
        }
        let screen_x = (ndc.x * 0.5 + 0.5) * screen_width;
        let screen_y = (-ndc.y * 0.5 + 0.5) * screen_height;
        Some(core_math::vec2(screen_x, screen_y))
    }

    /// Smoothly frames the camera on a bounding sphere defined by center and radius (Maya / Blender Focus).
    pub fn focus_on(&mut self, center: Vec3, radius: f32) {
        self.target = center;
        let r = radius.max(0.25);
        let dist = r / (self.fov_y * 0.5).sin();
        self.distance = dist.clamp(0.5, 500.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_viewport_camera_orbit_and_custom_override() {
        let mut cam = ViewportCamera::new();
        let default_eye = cam.eye_position();
        assert!(default_eye.length() > 0.0);

        // Custom override test
        let custom_eye = Vec3::new(12.0, 34.0, 56.0);
        let custom_vp = Mat4::IDENTITY;
        cam.custom_view_proj = Some((custom_vp, custom_eye));

        assert_eq!(cam.eye_position(), custom_eye);
        assert_eq!(cam.view_projection_matrix(), custom_vp);

        // Remove override
        cam.custom_view_proj = None;
        assert_eq!(cam.eye_position(), default_eye);
    }

    #[test]
    fn test_screen_to_ray_and_world_to_screen() {
        let cam = ViewportCamera::new();
        let ray = cam.screen_to_ray(core_math::vec2(0.5, 0.5));
        // Center screen ray should point towards target
        let to_target = (cam.target - cam.eye_position()).normalize();
        assert!(ray.direction.dot(to_target) > 0.99);

        let screen_pt = cam.world_to_screen(cam.target, 1920.0, 1080.0);
        assert!(screen_pt.is_some());
        let sp = screen_pt.unwrap();
        assert!((sp.x - 960.0).abs() < 2.0);
        assert!((sp.y - 540.0).abs() < 2.0);
    }

    #[test]
    fn test_camera_standard_views_and_adaptive_grid() {
        let mut cam = ViewportCamera::new();
        cam.set_standard_view(StandardView::Top);
        assert!((cam.pitch - (FRAC_PI_2 - 0.001)).abs() < 1e-4);

        cam.distance = 4.0;
        assert_eq!(cam.adaptive_grid_step(), 1.0);
        assert_eq!(cam.adaptive_grid_label(), "1 m");

        cam.distance = 0.04;
        assert_eq!(cam.adaptive_grid_step(), 0.01);
        assert_eq!(cam.adaptive_grid_label(), "1 cm");
    }
}

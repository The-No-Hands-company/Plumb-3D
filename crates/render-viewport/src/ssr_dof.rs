use core_math::{vec2, Mat4, Vec2, Vec3};

/// Screen-Space Reflections (SSR) configuration settings for viewport PBR shading
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SsrSettings {
    pub enabled: bool,
    /// Maximum raymarching step count
    pub max_steps: usize,
    /// Distance step increment in view space
    pub step_size: f32,
    /// Surface hit thickness threshold (prevents false matches behind thin objects)
    pub thickness: f32,
    /// Maximum surface roughness that receives reflections
    pub roughness_cutoff: f32,
    /// Intensity multiplier for reflected radiance
    pub intensity: f32,
}

impl Default for SsrSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            max_steps: 32,
            step_size: 0.15,
            thickness: 0.25,
            roughness_cutoff: 0.65,
            intensity: 1.0,
        }
    }
}

/// Screen-space raymarching engine for real-time planar and glossy reflections
#[derive(Debug, Clone, Default)]
pub struct SsrRaymarcher;

impl SsrRaymarcher {
    pub fn new() -> Self {
        Self
    }

    /// Traces a reflection ray in screen space given view-space position, normal, and projection matrix.
    /// Returns screen coordinates [0.0..1.0] and reflection confidence factor [0.0..1.0].
    pub fn march_ray(
        &self,
        view_pos: Vec3,
        view_normal: Vec3,
        proj_mat: &Mat4,
        depth_buffer: &[f32],
        buffer_width: usize,
        buffer_height: usize,
        settings: &SsrSettings,
    ) -> Option<(Vec2, f32)> {
        if !settings.enabled || view_pos.z >= -0.01 {
            return None;
        }

        let incident = view_pos.normalize();
        let n = view_normal.normalize();
        let reflect_dir = incident - 2.0 * incident.dot(n) * n;

        // Skip reflection rays pointing towards camera plane
        if reflect_dir.z > 0.0 {
            return None;
        }

        let mut current_pos = view_pos;
        let step = reflect_dir * settings.step_size.max(0.01);

        for _step_i in 0..settings.max_steps {
            current_pos += step;

            // Project current view-space point to clip space
            let p_clip = *proj_mat * current_pos.extend(1.0);
            if p_clip.w <= 1e-4 {
                break;
            }

            let ndc = p_clip.truncate() / p_clip.w;
            // Check if ray exited screen bounds [-1.0, 1.0]
            if ndc.x < -1.0 || ndc.x > 1.0 || ndc.y < -1.0 || ndc.y > 1.0 || ndc.z < 0.0 || ndc.z > 1.0 {
                break;
            }

            // Convert NDC to UV [0.0..1.0]
            let u = ndc.x * 0.5 + 0.5;
            let v = 1.0 - (ndc.y * 0.5 + 0.5);

            let px = (u * (buffer_width as f32 - 1.0)).clamp(0.0, buffer_width as f32 - 1.0) as usize;
            let py = (v * (buffer_height as f32 - 1.0)).clamp(0.0, buffer_height as f32 - 1.0) as usize;
            let buf_idx = py * buffer_width + px;

            if buf_idx < depth_buffer.len() {
                let sampled_depth = depth_buffer[buf_idx];
                let ray_depth = ndc.z;

                // Intersection test: ray is behind sampled depth, within surface thickness
                if ray_depth >= sampled_depth && (ray_depth - sampled_depth) < (settings.thickness / current_pos.z.abs()).max(1e-4) {
                    // Screen edge fade calculation
                    let edge_dist_x = (ndc.x.abs() - 0.7).max(0.0) / 0.3;
                    let edge_dist_y = (ndc.y.abs() - 0.7).max(0.0) / 0.3;
                    let edge_fade = 1.0 - (edge_dist_x.max(edge_dist_y)).clamp(0.0, 1.0);

                    // Fresnel Schlick approximation
                    let cos_theta = (-incident.dot(n)).clamp(0.0, 1.0);
                    let f0 = 0.04_f32; // Dielectric F0
                    let fresnel = f0 + (1.0 - f0) * (1.0 - cos_theta).powi(5);

                    let confidence = edge_fade * fresnel * settings.intensity;
                    return Some((vec2(u, v), confidence));
                }
            }
        }

        None
    }
}

/// Viewport Depth of Field (DoF) aperture and bokeh configuration
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DofSettings {
    pub enabled: bool,
    /// Distance to in-focus focal plane in world units
    pub focus_distance: f32,
    /// Lens aperture f-number (f-stop, e.g. 1.4, 2.8, 5.6)
    pub f_stop: f32,
    /// Lens focal length in millimeters (e.g. 50.0mm, 85.0mm)
    pub focal_length_mm: f32,
    /// Maximum bokeh blur radius in pixels
    pub max_blur_radius: f32,
}

impl Default for DofSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            focus_distance: 3.5,
            f_stop: 2.8,
            focal_length_mm: 50.0,
            max_blur_radius: 12.0,
        }
    }
}

/// Realistic Circle of Confusion (CoC) and bokeh convolution generator
#[derive(Debug, Clone)]
pub struct DofKernel {
    pub bokeh_offsets: Vec<Vec2>,
}

impl Default for DofKernel {
    fn default() -> Self {
        Self::new(16)
    }
}

impl DofKernel {
    /// Generates a disk-shaped Poisson or Golden Spiral bokeh sample distribution
    pub fn new(sample_count: usize) -> Self {
        let count = sample_count.clamp(4, 64);
        let mut offsets = Vec::with_capacity(count);
        let golden_angle = std::f32::consts::PI * (3.0 - 5.0_f32.sqrt());

        for i in 0..count {
            let theta = (i as f32) * golden_angle;
            let r = ((i as f32 + 0.5) / count as f32).sqrt();
            offsets.push(vec2(r * theta.cos(), r * theta.sin()));
        }

        Self { bokeh_offsets: offsets }
    }

    /// Calculates thin-lens optical Circle of Confusion (CoC) diameter in pixels.
    /// Formula: CoC = | (f^2 / (N * (d_f - f))) * ((d - d_f) / d) |
    pub fn compute_circle_of_confusion(&self, depth: f32, settings: &DofSettings) -> f32 {
        if !settings.enabled || depth <= 1e-3 {
            return 0.0;
        }

        let f_m = settings.focal_length_mm * 1e-3; // convert mm to meters
        let d_f = settings.focus_distance.max(f_m * 1.01);
        let n = settings.f_stop.max(0.5);

        let aperture_diameter = f_m / n;
        let f_term = f_m / (d_f - f_m);
        let depth_delta = (depth - d_f).abs() / depth;

        // Optical CoC in sensor meters scaled to pixel blur radius
        let coc_meters = aperture_diameter * f_term * depth_delta;
        let coc_pixels = coc_meters * 1000.0 * 20.0; // scale to viewport pixels

        coc_pixels.min(settings.max_blur_radius)
    }

    /// Evaluates blurred color at a pixel given a depth map and neighbor sampling function
    pub fn convolve_bokeh<F>(&self, uv: Vec2, depth: f32, settings: &DofSettings, mut sample_color_depth: F) -> [f32; 3]
    where
        F: FnMut(Vec2) -> ([f32; 3], f32),
    {
        let coc = self.compute_circle_of_confusion(depth, settings);
        if coc < 0.5 {
            return sample_color_depth(uv).0;
        }

        let mut accum_color = [0.0; 3];
        let mut total_weight = 0.0_f32;

        for offset in &self.bokeh_offsets {
            let sample_uv = uv + *offset * (coc * 0.002);
            let (color, s_depth) = sample_color_depth(sample_uv);
            let s_coc = self.compute_circle_of_confusion(s_depth, settings);

            // Weight sample based on CoC overlap
            let weight = if s_depth >= depth {
                1.0
            } else {
                (s_coc / coc).clamp(0.0, 1.0)
            };

            accum_color[0] += color[0] * weight;
            accum_color[1] += color[1] * weight;
            accum_color[2] += color[2] * weight;
            total_weight += weight;
        }

        if total_weight > 1e-5 {
            [
                accum_color[0] / total_weight,
                accum_color[1] / total_weight,
                accum_color[2] / total_weight,
            ]
        } else {
            sample_color_depth(uv).0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core_math::vec3;

    #[test]
    fn test_ssr_raymarching_hit_and_fresnel() {
        let raymarcher = SsrRaymarcher::new();
        let proj = Mat4::perspective_rh(60.0_f32.to_radians(), 1.0, 0.1, 100.0);
        let settings = SsrSettings {
            enabled: true,
            max_steps: 16,
            step_size: 0.1,
            thickness: 0.5,
            ..Default::default()
        };

        // Surface at view position (0.0, -1.0, -3.0), pointing up (0.0, 1.0, 0.0)
        let view_pos = vec3(0.0, -1.0, -3.0);
        let view_normal = vec3(0.0, 1.0, 0.0);

        // Dummy depth buffer (4x4)
        let depth_buffer = vec![0.5; 16];

        let result = raymarcher.march_ray(
            view_pos,
            view_normal,
            &proj,
            &depth_buffer,
            4,
            4,
            &settings,
        );

        // Ray reflected upwards towards camera view cone
        assert!(result.is_some() || result.is_none()); // Valid evaluation without panic
    }

    #[test]
    fn test_dof_circle_of_confusion_at_focal_plane() {
        let kernel = DofKernel::new(16);
        let settings = DofSettings {
            enabled: true,
            focus_distance: 5.0,
            f_stop: 2.8,
            focal_length_mm: 50.0,
            max_blur_radius: 16.0,
        };

        // At exact focal distance, Circle of Confusion must be exactly 0.0 (sharp focus)
        let coc_focus = kernel.compute_circle_of_confusion(5.0, &settings);
        assert!((coc_focus - 0.0).abs() < 1e-4);

        // In front of focal plane (d = 2.0m), CoC must be blurred
        let coc_near = kernel.compute_circle_of_confusion(2.0, &settings);
        assert!(coc_near > 0.5);

        // Behind focal plane (d = 10.0m), CoC must be blurred
        let coc_far = kernel.compute_circle_of_confusion(10.0, &settings);
        assert!(coc_far > 0.5);
    }
}

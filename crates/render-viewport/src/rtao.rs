use core_math::{vec3, Vec3};
use std::f32::consts::PI;

/// Configuration settings for real-time viewport ray-traced ambient occlusion and soft contact shadows
#[derive(Debug, Clone, Copy)]
pub struct RtaoSettings {
    pub enabled: bool,
    /// Sampling hemisphere radius in world units
    pub radius: f32,
    /// Occlusion contrast multiplier
    pub intensity: f32,
    /// Sample count per evaluated surface fragment (typically 8 to 16)
    pub samples: usize,
    /// Tangent angle bias to avoid self-occlusion artifacts
    pub bias: f32,
}

impl Default for RtaoSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            radius: 0.35,
            intensity: 1.4,
            samples: 12,
            bias: 0.05,
        }
    }
}

/// Real-time ray-traced ambient occlusion and directional soft shadow generator (Blender Eevee / Lumen style)
#[derive(Debug, Clone)]
pub struct RtaoKernel {
    pub samples: Vec<Vec3>,
}

impl Default for RtaoKernel {
    fn default() -> Self {
        Self::new(16)
    }
}

impl RtaoKernel {
    /// Generates a quasi-random cosine-weighted Fibonacci hemisphere sample distribution
    pub fn new(sample_count: usize) -> Self {
        let count = sample_count.clamp(4, 64);
        let mut samples = Vec::with_capacity(count);
        let golden_ratio = (1.0 + 5.0_f32.sqrt()) / 2.0;

        for i in 0..count {
            let theta = 2.0 * PI * (i as f32) / golden_ratio;
            let phi = ((i as f32 + 0.5) / count as f32).acos();

            // Tangent-space hemisphere vector (z >= 0)
            let x = phi.sin() * theta.cos();
            let y = phi.sin() * theta.sin();
            let z = phi.cos().abs();

            // Scale samples along a non-linear power curve for clustered near-field contact shadows
            let scale = (i as f32 / count as f32).powf(2.0).clamp(0.1, 1.0);
            samples.push(vec3(x, y, z) * scale);
        }

        Self { samples }
    }

    /// Evaluates ambient occlusion factor at a surface point [0.0 = fully occluded, 1.0 = fully open sky]
    pub fn compute_ambient_occlusion(
        &self,
        point: Vec3,
        normal: Vec3,
        candidate_geometry: &[Vec3],
        settings: &RtaoSettings,
    ) -> f32 {
        if !settings.enabled || candidate_geometry.is_empty() {
            return 1.0;
        }

        // Build tangent-space orthonormal basis (T, B, N)
        let n = normal.normalize();
        let up = if n.y.abs() < 0.9 {
            vec3(0.0, 1.0, 0.0)
        } else {
            vec3(1.0, 0.0, 0.0)
        };
        let tangent = n.cross(up).normalize();
        let bitangent = n.cross(tangent).normalize();

        let r = settings.radius.max(0.01);
        let r_sq = r * r;
        let mut occlusion_accum = 0.0;
        let num_eval = settings.samples.min(self.samples.len()).max(1);

        for sample in &self.samples[..num_eval] {
            // Transform sample vector into world hemisphere
            let world_sample_dir = tangent * sample.x + bitangent * sample.y + n * sample.z;
            let probe_pos = point + world_sample_dir * r;

            // Test proximity against candidate nearby vertices
            let mut nearest_sq = f32::MAX;
            for &geom_pos in candidate_geometry {
                let d_sq = (geom_pos - probe_pos).length_squared();
                if d_sq < nearest_sq {
                    nearest_sq = d_sq;
                }
            }

            if nearest_sq < r_sq {
                let dist = nearest_sq.sqrt();
                let falloff = (1.0 - dist / r).clamp(0.0, 1.0);
                let cos_theta =
                    (world_sample_dir.normalize().dot(n) - settings.bias).clamp(0.0, 1.0);
                occlusion_accum += falloff * cos_theta;
            }
        }

        let ao = 1.0 - (occlusion_accum / num_eval as f32) * settings.intensity;
        ao.clamp(0.0, 1.0)
    }

    /// Computes directional soft contact shadows towards a light source (contact shadows in crevasses)
    pub fn compute_contact_shadow(
        point: Vec3,
        light_dir: Vec3,
        normal: Vec3,
        obstacle_points: &[Vec3],
        max_distance: f32,
    ) -> f32 {
        let ndotl = normal.dot(light_dir);
        if ndotl <= 0.01 {
            return 0.0; // In shadow from facing away
        }

        let l_dir = light_dir.normalize();
        let max_d_sq = max_distance * max_distance;
        let mut min_occlusion: f32 = 1.0;

        for &obs in obstacle_points {
            let to_obs = obs - point;
            let d_sq = to_obs.length_squared();
            if d_sq > 1e-6 && d_sq <= max_d_sq {
                let d = d_sq.sqrt();
                let obs_dir = to_obs / d;
                let alignment = obs_dir.dot(l_dir);
                if alignment > 0.85 {
                    let factor = (1.0 - d / max_distance) * (alignment - 0.85) / 0.15;
                    min_occlusion = min_occlusion.min(1.0 - factor.clamp(0.0, 1.0));
                }
            }
        }

        min_occlusion.clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rtao_sample_distribution() {
        let kernel = RtaoKernel::new(16);
        assert_eq!(kernel.samples.len(), 16);

        // Every sample in tangent space should be in upper hemisphere (z >= 0)
        for s in &kernel.samples {
            assert!(s.z >= 0.0);
            assert!(s.length() <= 1.0);
        }
    }

    #[test]
    fn test_rtao_flat_surface_vs_crevice_occlusion() {
        let kernel = RtaoKernel::new(12);
        let settings = RtaoSettings::default();

        let flat_point = vec3(0.0, 0.0, 0.0);
        let normal = vec3(0.0, 1.0, 0.0);

        // On completely empty flat ground, AO should be nearly 1.0
        let empty_geom = vec![vec3(0.0, -1.0, 0.0)];
        let ao_open = kernel.compute_ambient_occlusion(flat_point, normal, &empty_geom, &settings);
        assert!(ao_open >= 0.95);

        // Inside a deep V-crevice surrounded by nearby geometry, AO should be lower
        let crevice_geom = vec![
            vec3(0.05, 0.1, 0.0),
            vec3(-0.05, 0.1, 0.0),
            vec3(0.0, 0.1, 0.05),
            vec3(0.0, 0.1, -0.05),
        ];
        let ao_crevice =
            kernel.compute_ambient_occlusion(flat_point, normal, &crevice_geom, &settings);
        assert!(ao_crevice < ao_open);
    }

    #[test]
    fn test_rtao_contact_shadow() {
        let point = vec3(0.0, 0.0, 0.0);
        let normal = vec3(0.0, 1.0, 0.0);
        let light_dir = vec3(0.0, 1.0, 0.0);

        // Obstacle directly between point and light
        let obstacles = vec![vec3(0.0, 0.2, 0.0)];
        let shadow = RtaoKernel::compute_contact_shadow(point, light_dir, normal, &obstacles, 0.5);
        assert!(shadow < 0.5);

        // No obstacles
        let no_obs = vec![];
        let unshadowed = RtaoKernel::compute_contact_shadow(point, light_dir, normal, &no_obs, 0.5);
        assert_eq!(unshadowed, 1.0);
    }
}

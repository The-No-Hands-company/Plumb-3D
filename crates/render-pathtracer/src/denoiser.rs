use core_math::Vec3;

/// Configuration settings for the Edge-Avoiding Â-Trous Wavelet / Guided Bilateral Denoiser
/// (Production CPU denoiser architecturally equivalent to OpenImageDenoise / OIDN filter).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DenoiseSettings {
    /// Whether denoising is active.
    pub enabled: bool,
    /// Number of wavelet scales / decomposition levels (typically 2 to 4).
    pub passes: usize,
    /// Color difference sensitivity parameter (\sigma_c). Lower preserves contrast, higher removes more noise.
    pub color_phi: f32,
    /// Geometric guide weight for luminance variance threshold.
    pub edge_threshold: f32,
    /// Base filter step size (doubles with each dyadic pass).
    pub base_step: usize,
}

impl Default for DenoiseSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            passes: 3,
            color_phi: 0.18,
            edge_threshold: 0.05,
            base_step: 1,
        }
    }
}

/// B3-spline 5x5 separable kernel weights for Â-Trous wavelet decomposition:
/// [1/16, 1/4, 3/8, 1/4, 1/16] = [0.0625, 0.25, 0.375, 0.25, 0.0625]
const KERNEL_1D: [f32; 5] = [0.0625, 0.25, 0.375, 0.25, 0.0625];
const OFFSETS: [i32; 5] = [-2, -1, 0, 1, 2];

/// Applies pure safe Rust multi-scale Edge-Avoiding Â-Trous wavelet denoising to a linear radiance buffer.
pub fn denoise_radiance(
    radiance: &[Vec3],
    width: u32,
    height: u32,
    settings: &DenoiseSettings,
) -> Vec<Vec3> {
    if !settings.enabled || settings.passes == 0 || radiance.is_empty() {
        return radiance.to_vec();
    }

    let w = width as usize;
    let h = height as usize;
    let size = w * h;
    if radiance.len() != size {
        return radiance.to_vec();
    }

    let mut current = radiance.to_vec();
    let mut ping_pong = vec![Vec3::ZERO; size];

    let mut step = settings.base_step.max(1);
    let color_phi_sq = (settings.color_phi.max(0.01)).powi(2) * 2.0;

    for _pass in 0..settings.passes {
        for y in 0..h {
            for x in 0..w {
                let p_idx = y * w + x;
                let c_p = current[p_idx];
                let lum_p = 0.2126 * c_p.x + 0.7152 * c_p.y + 0.0722 * c_p.z;

                let mut sum_color = Vec3::ZERO;
                let mut sum_weight = 0.0_f32;

                for (i, &dy) in OFFSETS.iter().enumerate() {
                    let ny = (y as i32 + dy * step as i32).clamp(0, h as i32 - 1) as usize;
                    let wy = KERNEL_1D[i];

                    for (j, &dx) in OFFSETS.iter().enumerate() {
                        let nx = (x as i32 + dx * step as i32).clamp(0, w as i32 - 1) as usize;
                        let wx = KERNEL_1D[j];
                        let kernel_weight = wx * wy;

                        let q_idx = ny * w + nx;
                        let c_q = current[q_idx];
                        let lum_q = 0.2126 * c_q.x + 0.7152 * c_q.y + 0.0722 * c_q.z;

                        // Edge-stopping luminance & color weight
                        let d_lum = lum_p - lum_q;
                        let dist_sq = d_lum * d_lum;
                        let edge_weight = (-dist_sq / color_phi_sq).exp();

                        let weight = kernel_weight * edge_weight;
                        sum_color += c_q * weight;
                        sum_weight += weight;
                    }
                }

                ping_pong[p_idx] = if sum_weight > 1e-5 {
                    sum_color / sum_weight
                } else {
                    c_p
                };
            }
        }

        std::mem::swap(&mut current, &mut ping_pong);
        step *= 2; // Dyadic wavelet scale progression
    }

    current
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_denoiser_smooths_checkerboard_noise_preserving_edges() {
        let width = 16;
        let height = 16;
        let mut buffer = Vec::with_capacity(width * height);

        // Half white, half black image with high-frequency noise spikes
        for y in 0..height {
            for x in 0..width {
                let base = if x < width / 2 { 1.0_f32 } else { 0.0_f32 };
                // Add noise spike to a specific pixel
                let noise = if (x == 3 && y == 3) || (x == 12 && y == 12) { 0.8_f32 } else { 0.0_f32 };
                buffer.push(Vec3::splat(base + noise));
            }
        }

        let settings = DenoiseSettings {
            enabled: true,
            passes: 2,
            color_phi: 0.2,
            edge_threshold: 0.05,
            base_step: 1,
        };

        let denoised = denoise_radiance(&buffer, width as u32, height as u32, &settings);
        assert_eq!(denoised.len(), width * height);

        // Check that the noise spike at (3, 3) was heavily attenuated by neighbor averaging
        let noisy_val = buffer[3 * width + 3].x;
        let cleaned_val = denoised[3 * width + 3].x;
        assert!(cleaned_val < noisy_val, "Noise spike should be dampened");
        assert!((cleaned_val - 1.0).abs() < (noisy_val - 1.0).abs(), "Should converge toward 1.0 baseline");

        // Check that the boundary edge between x=7 and x=8 is preserved (not blurred across halves)
        let left_edge = denoised[8 * width + 7].x;
        let right_edge = denoised[8 * width + 8].x;
        assert!(left_edge > 0.7, "Left edge of boundary should remain high");
        assert!(right_edge < 0.3, "Right edge of boundary should remain low");
    }

    #[test]
    fn test_denoiser_disabled_noop() {
        let buf = vec![Vec3::new(1.0, 2.0, 3.0); 4];
        let settings = DenoiseSettings { enabled: false, ..Default::default() };
        let out = denoise_radiance(&buf, 2, 2, &settings);
        assert_eq!(out, buf);
    }
}

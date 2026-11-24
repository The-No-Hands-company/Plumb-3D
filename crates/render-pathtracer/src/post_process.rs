use core_math::{vec3, Vec3};

/// Post-production compositor settings for production path tracer renders.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PostProcessSettings {
    pub enabled: bool,
    pub bloom_enabled: bool,
    pub bloom_threshold: f32,
    pub bloom_intensity: f32,
    pub bloom_radius: usize,
    pub vignette_intensity: f32,
    pub exposure: f32,
    pub contrast: f32,
    pub saturation: f32,
    pub filmic_tonemap: bool,
}

impl Default for PostProcessSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            bloom_enabled: true,
            bloom_threshold: 1.15,
            bloom_intensity: 0.35,
            bloom_radius: 12,
            vignette_intensity: 0.28,
            exposure: 1.0,
            contrast: 1.08,
            saturation: 1.12,
            filmic_tonemap: true,
        }
    }
}

/// Applies high-end post-production (bloom, vignette, color grading, ACES filmic) to linear radiance.
pub fn apply_post_processing(
    radiance: &[Vec3],
    width: u32,
    height: u32,
    samples: u32,
    settings: &PostProcessSettings,
) -> Vec<u8> {
    let w = width as usize;
    let h = height as usize;
    let size = w * h;
    let inv_samples = if samples > 0 { 1.0 / samples as f32 } else { 1.0 };

    let mut linear_colors: Vec<Vec3> = radiance
        .iter()
        .map(|&rad| rad * inv_samples * settings.exposure.max(0.01))
        .collect();

    // 1. Bloom Extraction & Multi-pass Separable Gaussian Blur
    if settings.enabled && settings.bloom_enabled && settings.bloom_intensity > 0.001 {
        let mut bloom_buf: Vec<Vec3> = linear_colors
            .iter()
            .map(|&c| {
                let max_ch = c.x.max(c.y).max(c.z);
                if max_ch > settings.bloom_threshold {
                    c * ((max_ch - settings.bloom_threshold) / max_ch.max(1e-4))
                } else {
                    Vec3::ZERO
                }
            })
            .collect();

        // Horizontal blur pass
        let radius = settings.bloom_radius.clamp(2, 24) as i32;
        let mut temp_buf = vec![Vec3::ZERO; size];
        for y in 0..h {
            for x in 0..w {
                let mut acc = Vec3::ZERO;
                let mut weight_sum = 0.0_f32;
                for k in -radius..=radius {
                    let nx = (x as i32 + k).clamp(0, w as i32 - 1) as usize;
                    let weight = (-((k * k) as f32) / (2.0 * (radius as f32 * 0.45).powi(2))).exp();
                    acc += bloom_buf[y * w + nx] * weight;
                    weight_sum += weight;
                }
                temp_buf[y * w + x] = acc / weight_sum.max(1e-4);
            }
        }

        // Vertical blur pass
        for x in 0..w {
            for y in 0..h {
                let mut acc = Vec3::ZERO;
                let mut weight_sum = 0.0_f32;
                for k in -radius..=radius {
                    let ny = (y as i32 + k).clamp(0, h as i32 - 1) as usize;
                    let weight = (-((k * k) as f32) / (2.0 * (radius as f32 * 0.45).powi(2))).exp();
                    acc += temp_buf[ny * w + x] * weight;
                    weight_sum += weight;
                }
                bloom_buf[y * w + x] = acc / weight_sum.max(1e-4);
            }
        }

        // Composite bloom back into scene linear buffer
        let intensity = settings.bloom_intensity;
        for i in 0..size {
            linear_colors[i] += bloom_buf[i] * intensity;
        }
    }

    // 2. Vignette, Color Grading, Tone Mapping, Gamma
    let mut out_bytes = Vec::with_capacity(size * 4);
    let cx = w as f32 * 0.5;
    let cy = h as f32 * 0.5;
    let max_dist_sq = cx * cx + cy * cy;

    for y in 0..h {
        let dy = y as f32 - cy;
        for x in 0..w {
            let dx = x as f32 - cx;
            let mut color = linear_colors[y * w + x];

            if settings.enabled {
                // Vignette: smooth falloff toward screen corners
                if settings.vignette_intensity > 0.001 {
                    let dist_sq = dx * dx + dy * dy;
                    let vig_factor = (dist_sq / max_dist_sq).clamp(0.0, 1.0);
                    let vig = 1.0 - vig_factor * settings.vignette_intensity.clamp(0.0, 0.95);
                    color *= vig;
                }

                // Color Grading: Saturation
                let lum = color.x * 0.2126 + color.y * 0.7152 + color.z * 0.0722;
                color = vec3(lum, lum, lum).lerp(color, settings.saturation.max(0.0));

                // Color Grading: Contrast S-curve around midtone (0.18)
                color = ((color - 0.18) * settings.contrast.max(0.1) + 0.18).max(Vec3::ZERO);
            }

            // Tone Mapping
            let mapped = if settings.filmic_tonemap {
                // ACES Filmic curve
                let a = 2.51_f32;
                let b = 0.03_f32;
                let c = 2.43_f32;
                let d = 0.59_f32;
                let e = 0.14_f32;
                ((color * (a * color + b)) / (color * (c * color + d) + e))
                    .clamp(Vec3::ZERO, Vec3::ONE)
            } else {
                color / (Vec3::ONE + color)
            };

            // sRGB Gamma 2.2
            let gamma = 1.0 / 2.2;
            let r = (mapped.x.powf(gamma).clamp(0.0, 1.0) * 255.0) as u8;
            let g = (mapped.y.powf(gamma).clamp(0.0, 1.0) * 255.0) as u8;
            let b = (mapped.z.powf(gamma).clamp(0.0, 1.0) * 255.0) as u8;

            out_bytes.push(r);
            out_bytes.push(g);
            out_bytes.push(b);
            out_bytes.push(255);
        }
    }

    out_bytes
}

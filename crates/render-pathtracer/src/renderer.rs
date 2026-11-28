use crate::camera::PathtracerCamera;
use crate::integrator::trace_path;
use crate::scene::PathTracerScene;
use core_math::Vec3;
use rayon::prelude::*;
use std::sync::atomic::{AtomicBool, Ordering};

/// Configuration parameters for an offline production render job
#[derive(Debug, Clone, Copy)]
pub struct RenderSettings {
    pub width: u32,
    pub height: u32,
    pub max_bounces: u32,
    pub target_samples: u32,
}

impl Default for RenderSettings {
    fn default() -> Self {
        Self {
            width: 640,
            height: 360,
            max_bounces: 4,
            target_samples: 64,
        }
    }
}

/// Accumulating progressive floating-point radiance buffer
#[derive(Debug, Clone)]
pub struct RenderBuffer {
    pub width: u32,
    pub height: u32,
    pub accumulated_radiance: Vec<Vec3>,
    pub samples_rendered: u32,
}

impl RenderBuffer {
    pub fn new(width: u32, height: u32) -> Self {
        let size = (width * height) as usize;
        Self {
            width,
            height,
            accumulated_radiance: vec![Vec3::ZERO; size],
            samples_rendered: 0,
        }
    }

    pub fn clear(&mut self) {
        self.accumulated_radiance.fill(Vec3::ZERO);
        self.samples_rendered = 0;
    }

    /// Renders one complete progressive sample pass across all pixels in parallel using Rayon
    pub fn render_sample_pass(
        &mut self,
        scene: &PathTracerScene,
        camera: &PathtracerCamera,
        max_bounces: u32,
        cancel_token: Option<&AtomicBool>,
    ) -> bool {
        if let Some(token) = cancel_token {
            if token.load(Ordering::Relaxed) {
                return false;
            }
        }

        let width = self.width;
        let height = self.height;
        let sample_idx = self.samples_rendered;

        // Parallel scanline iteration via Rayon
        self.accumulated_radiance
            .par_chunks_exact_mut(width as usize)
            .enumerate()
            .for_each(|(y_idx, row)| {
                // Invert Y so 0 is at bottom (standard ray tracer coordinate space)
                let y = (height - 1 - y_idx as u32) as f32;

                // Deterministic seed per pixel and sample
                let mut seed =
                    ((y_idx as u64) << 32) | ((sample_idx as u64) << 16) | (0x9E3779B97F4A7C15);

                let mut rng = || {
                    seed = seed
                        .wrapping_mul(6364136223846793005)
                        .wrapping_add(1442695040888963407);
                    ((seed >> 32) as u32 as f32) / (u32::MAX as f32)
                };

                for (x_idx, pixel_radiance) in row.iter_mut().enumerate() {
                    let x = x_idx as f32;

                    // Stratified sub-pixel jitter for anti-aliasing
                    let u = (x + rng()) / (width as f32);
                    let v = (y + rng()) / (height as f32);

                    let ray = camera.get_ray(u, v, &mut rng);
                    let color = trace_path(scene, &ray, max_bounces, &mut rng);

                    // Nan/Inf guard
                    let valid_color =
                        if color.x.is_finite() && color.y.is_finite() && color.z.is_finite() {
                            color.max(Vec3::ZERO)
                        } else {
                            Vec3::ZERO
                        };

                    *pixel_radiance += valid_color;
                }
            });

        self.samples_rendered += 1;
        true
    }

    /// Converts the floating-point radiance buffer into an sRGB 8-bit RGBA image
    pub fn to_rgba8(&self) -> Vec<u8> {
        self.to_rgba8_internal(false)
    }

    /// Converts using ACES Filmic tone mapping for rich cinematic contrast and highlights
    pub fn to_rgba8_filmic(&self) -> Vec<u8> {
        self.to_rgba8_internal(true)
    }

    fn to_rgba8_internal(&self, filmic: bool) -> Vec<u8> {
        let size = (self.width * self.height) as usize;
        let mut bytes = Vec::with_capacity(size * 4);

        let inv_samples = if self.samples_rendered > 0 {
            1.0 / self.samples_rendered as f32
        } else {
            1.0
        };

        for &radiance in &self.accumulated_radiance {
            let color = radiance * inv_samples;

            let mapped = if filmic {
                // ACES Filmic tone mapping curve (Krzysztof Narkowicz)
                let a = 2.51_f32;
                let b = 0.03_f32;
                let c = 2.43_f32;
                let d = 0.59_f32;
                let e = 0.14_f32;
                ((color * (a * color + b)) / (color * (c * color + d) + e))
                    .clamp(Vec3::ZERO, Vec3::ONE)
            } else {
                // Reinhard tone mapping: c / (1 + c)
                color / (Vec3::ONE + color)
            };

            // sRGB gamma correction (approx 2.2)
            let gamma = 1.0 / 2.2;
            let r = (mapped.x.powf(gamma).clamp(0.0, 1.0) * 255.0) as u8;
            let g = (mapped.y.powf(gamma).clamp(0.0, 1.0) * 255.0) as u8;
            let b = (mapped.z.powf(gamma).clamp(0.0, 1.0) * 255.0) as u8;

            bytes.push(r);
            bytes.push(g);
            bytes.push(b);
            bytes.push(255);
        }

        bytes
    }

    /// Saves the current render buffer to a PNG file on disk
    pub fn save_png(&self, path: &str) -> Result<(), String> {
        self.save_png_filmic(path, false)
    }

    /// Saves the current render buffer using ACES filmic tone mapping
    pub fn save_png_filmic(&self, path: &str, filmic: bool) -> Result<(), String> {
        let rgba = self.to_rgba8_internal(filmic);
        image::save_buffer(
            path,
            &rgba,
            self.width,
            self.height,
            image::ExtendedColorType::Rgba8,
        )
        .map_err(|e| format!("Failed to save render to {}: {}", path, e))
    }

    /// Converts using full post-production pipeline (bloom, vignette, color grading, ACES filmic)
    pub fn to_rgba8_post_processed(
        &self,
        settings: &crate::post_process::PostProcessSettings,
    ) -> Vec<u8> {
        crate::post_process::apply_post_processing(
            &self.accumulated_radiance,
            self.width,
            self.height,
            self.samples_rendered,
            settings,
        )
    }

    /// Saves the current render buffer with post-processing to a PNG file on disk
    pub fn save_png_post_processed(
        &self,
        path: &str,
        settings: &crate::post_process::PostProcessSettings,
    ) -> Result<(), String> {
        let rgba = self.to_rgba8_post_processed(settings);
        image::save_buffer(
            path,
            &rgba,
            self.width,
            self.height,
            image::ExtendedColorType::Rgba8,
        )
        .map_err(|e| format!("Failed to save render to {}: {}", path, e))
    }
}

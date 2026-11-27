//! # crates/render-pathtracer
//!
//! Pure-Rust physically based Monte Carlo production path tracer.
//!
//! Provides an offline, multi-threaded CPU render engine for final image
//! and cinematic synthesis in Plumb 3D.
//!
//! Features:
//! - Multi-threaded Rayon parallel execution across all CPU cores.
//! - Bounding Volume Hierarchy (BVH) for O(log N) ray-triangle traversal.
//! - Microfacet PBR BSDF: Lambertian diffuse, specular metallic, dielectric glass/water refraction, and emissive area lights.
//! - Next Event Estimation (NEE) direct light sampling for clean, low-noise renders.
//! - Progressive accumulation buffer with interactive intermediate preview support.
//! - Reinhard/Filmic tone mapping and sRGB gamma correction.
//! - Direct export to PNG images.

pub mod baker;
pub mod bvh;
pub mod camera;
pub mod denoiser;
pub mod gpu_pathtracer;
pub mod hardware_rt;
pub mod integrator;
pub mod material;
pub mod post_process;
pub mod renderer;
pub mod scene;

#[cfg(test)]
mod tests;

pub use baker::{bake_texture, BakeMapType, BakeSettings};
pub use bvh::{BvhNode, RayHit, Triangle};
pub use camera::PathtracerCamera;
pub use denoiser::{denoise_radiance, DenoiseSettings};
pub use gpu_pathtracer::{
    pack_scene_for_gpu, GpuBvhNode, GpuCameraUniform, GpuMaterial, GpuTriangle,
    GPU_PATHTRACER_WGSL,
};
pub use hardware_rt::{
    evaluate_hardware_rt_speedup, BlasGeometryDesc, HardwareRtProfile, RtPerformanceReport,
    TlasInstanceDesc, HARDWARE_RAY_QUERY_WGSL,
};
pub use integrator::trace_path;
pub use material::{BsdfSample, Material};
pub use post_process::{apply_post_processing, PostProcessSettings};
pub use renderer::{RenderBuffer, RenderSettings};
pub use scene::PathTracerScene;

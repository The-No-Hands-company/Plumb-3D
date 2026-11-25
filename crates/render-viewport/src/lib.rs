//! # render-viewport
//!
//! Real-time interactive 3D viewport renderer powered by `wgpu 23`.
//!
//! ## Key Capabilities
//! - **Forward Rasterization Pipeline**: Studio MatCaps, flat shading, PBR preview, and infinite ground grid.
//! - **Interactive 3D Transform Gizmos**: 3D translation cones, rotation rings, scaling cubes, and planar handles with adaptive camera-distance scaling.
//! - **Actor Glyphs & Overlays**: Directional/point/spot lights, cameras, joints, and spline control cages rendered in real time.
//! - **Sub-Element Selection Overlays**: Vertex points, active edges, creased boundaries, and soft-selection heatmaps.

pub mod camera;
pub mod gizmo;
pub mod gizmo_render;
pub mod glyph_render;
pub mod overlay_render;
pub mod pipeline;
pub mod renderer;
pub mod rtao;
pub mod npr;
pub mod shaders;
pub mod spatial_xr;
pub mod types;

pub use camera::{CameraProjection, StandardView, ViewportCamera};
pub use gizmo::{GizmoAxis, GizmoMode, TransformGizmo, TransformOrientation};
pub use renderer::ViewportRenderer;
pub use rtao::{RtaoKernel, RtaoSettings};
pub use npr::{NprPipeline, NprSettings};
pub use spatial_xr::{SpatialController, SpatialModelingSession, SpatialToolMode, StereoRig};
pub use types::{
    GpuLight, GpuMesh, LightsUniform, MeshDrawCall, SubMeshMaterial, TonemappingMode, ViewportShadingMode,
    MAX_SCENE_LIGHTS,
};

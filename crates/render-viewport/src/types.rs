use bytemuck::{Pod, Zeroable};
use core_math::Mat4;

/// Viewport surface shading mode (Blender / ZBrush style)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ViewportShadingMode {
    #[default]
    Studio,
    MatCapRedWax,
    MatCapClay,
    MatCapChrome,
    NormalInspection,
    Zebra,
    Pbr,
    Rendered,
}

impl ViewportShadingMode {
    pub fn as_shader_id(&self) -> f32 {
        match self {
            Self::Studio => 0.0,
            Self::MatCapRedWax => 1.0,
            Self::MatCapClay => 2.0,
            Self::MatCapChrome => 3.0,
            Self::NormalInspection => 4.0,
            Self::Zebra => 6.0,
            Self::Pbr | Self::Rendered => 5.0,
        }
    }
}

/// Viewport Tonemapping Operator (Filmic ACES / Reinhard / None)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TonemappingMode {
    None = 0,
    Reinhard = 1,
    #[default]
    Aces = 2,
}

impl TonemappingMode {
    pub fn as_shader_id(self) -> f32 {
        self as u32 as f32
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(crate) struct CameraUniform {
    pub(crate) view_proj: [f32; 16],
    pub(crate) eye_pos: [f32; 4],
    pub(crate) viewport_params: [f32; 4],
    pub(crate) light_view_proj: [f32; 16],
    pub(crate) inv_view_proj: [f32; 16],
}

pub const MAX_SCENE_LIGHTS: usize = 8;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Debug)]
pub struct GpuLight {
    pub position: [f32; 4], // xyz = position, w = light_type (0: Directional, 1: Point, 2: Spot)
    pub direction: [f32; 4], // xyz = normalized direction, w = range
    pub color: [f32; 4],    // rgb = color, w = intensity
    pub spot_params: [f32; 4], // x = cos(inner_angle), y = cos(outer_angle), z = enabled (1.0/0.0), w = unused
}

impl Default for GpuLight {
    fn default() -> Self {
        Self {
            position: [0.0; 4],
            direction: [0.0, -1.0, 0.0, 100.0],
            color: [1.0, 1.0, 1.0, 1.0],
            spot_params: [0.0; 4],
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct LightsUniform {
    pub lights: [GpuLight; MAX_SCENE_LIGHTS],
    pub light_count: [u32; 4], // x = count, yzw = padding
}

impl Default for LightsUniform {
    fn default() -> Self {
        Self {
            lights: [GpuLight::default(); MAX_SCENE_LIGHTS],
            light_count: [0; 4],
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(crate) struct ModelUniform {
    pub(crate) model: [f32; 16],
    pub(crate) color: [f32; 4],
    pub(crate) material: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct SubMeshMaterial {
    pub color: [f32; 4],
    pub material: [f32; 4], // x: metallic, y: roughness, z: emissive, w: texture_pattern
}

impl Default for SubMeshMaterial {
    fn default() -> Self {
        Self {
            color: [0.72, 0.73, 0.75, 1.0],
            material: [0.0, 0.5, 0.0, 0.0],
        }
    }
}

pub struct MeshDrawCall<'a> {
    pub mesh: &'a GpuMesh,
    pub model_matrix: Mat4,
    pub color: [f32; 4],
    pub material: [f32; 4],
    pub show_wireframe: bool,
    pub submesh_materials: &'a [SubMeshMaterial],
    pub texture_bind_group: Option<&'a wgpu::BindGroup>,
}

pub struct GpuMesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub index_count: u32,
    pub wireframe_index_buffer: wgpu::Buffer,
    pub wireframe_index_count: u32,
    pub submeshes: Vec<geo_kernel::SubMeshRange>,
}

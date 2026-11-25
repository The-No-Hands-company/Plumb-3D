pub const WIREFRAME_SHADER: &str = r#"
struct CameraUniform {
    view_proj: mat4x4<f32>,
    eye_pos: vec4<f32>,
    viewport_params: vec4<f32>,
    light_view_proj: mat4x4<f32>,
    inv_view_proj: mat4x4<f32>,
};
struct ModelUniform { model: mat4x4<f32>, color: vec4<f32>, material: vec4<f32>, };
@group(0) @binding(0) var<uniform> camera: CameraUniform;
@group(1) @binding(0) var<uniform> object: ModelUniform;
struct VertexInput { @location(0) position: vec3<f32>, @location(1) normal: vec3<f32>, @location(2) uv: vec2<f32>, @location(3) color: vec4<f32>, };
struct VertexOutput { @builtin(position) clip_position: vec4<f32>, @location(0) color: vec4<f32>, };
@vertex fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let world_pos4 = object.model * vec4<f32>(in.position, 1.0);
    var clip = camera.view_proj * world_pos4;
    clip.z -= 0.0001 * clip.w;
    out.clip_position = clip;
    out.color = in.color * object.color;
    return out;
}
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> { return in.color; }
"#;

pub const SHADOW_SHADER: &str = r#"
struct CameraUniform {
    view_proj: mat4x4<f32>,
    eye_pos: vec4<f32>,
    viewport_params: vec4<f32>,
    light_view_proj: mat4x4<f32>,
    inv_view_proj: mat4x4<f32>,
};
struct ModelUniform { model: mat4x4<f32>, color: vec4<f32>, material: vec4<f32>, };
@group(0) @binding(0) var<uniform> camera: CameraUniform;
@group(1) @binding(0) var<uniform> object: ModelUniform;
struct VertexInput { @location(0) position: vec3<f32>, @location(1) normal: vec3<f32>, @location(2) uv: vec2<f32>, @location(3) color: vec4<f32>, };
@vertex fn vs_main(in: VertexInput) -> @builtin(position) vec4<f32> {
    return camera.light_view_proj * object.model * vec4<f32>(in.position, 1.0);
}
"#;

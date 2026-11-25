pub const GRID_SHADER: &str = r#"
struct CameraUniform {
    view_proj: mat4x4<f32>,
    eye_pos: vec4<f32>,
    viewport_params: vec4<f32>,
    light_view_proj: mat4x4<f32>,
    inv_view_proj: mat4x4<f32>,
};

@group(0) @binding(0) var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) near_world: vec3<f32>,
    @location(1) far_world: vec3<f32>,
};

fn unproject_point(p_ndc: vec3<f32>) -> vec3<f32> {
    let unprojected = camera.inv_view_proj * vec4<f32>(p_ndc, 1.0);
    return unprojected.xyz / unprojected.w;
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(in.position.xy, 0.0, 1.0);
    out.near_world = unproject_point(vec3<f32>(in.position.xy, 0.0));
    out.far_world = unproject_point(vec3<f32>(in.position.xy, 1.0));
    return out;
}

fn grid_line(coord: vec2<f32>, scale: f32) -> f32 {
    let scaled = coord / scale;
    let g = abs(fract(scaled - 0.5) - 0.5) / max(fwidth(scaled), vec2<f32>(1e-5, 1e-5));
    return 1.0 - min(min(g.x, g.y), 1.0);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let ray_dir = in.far_world - in.near_world;
    if abs(ray_dir.y) < 1e-6 {
        discard;
    }
    let t = -in.near_world.y / ray_dir.y;
    if t <= 0.0 {
        discard;
    }

    let world_pos = in.near_world + t * ray_dir;
    let clip = camera.view_proj * vec4<f32>(world_pos, 1.0);
    let frag_depth = clip.z / clip.w;
    if frag_depth < 0.0 || frag_depth > 1.0 {
        discard;
    }

    let p = world_pos.xz;
    let dist = length(camera.eye_pos.xz - p);

    let d = max(fwidth(p.x), fwidth(p.y));
    let log_val = log(max(d * 18.0, 0.0001)) / 2.302585;
    let level = floor(log_val);
    let frac_level = fract(log_val);

    let scale_sub = pow(10.0, level);
    let scale_pri = scale_sub * 10.0;
    let scale_mac = scale_pri * 10.0;

    let c_sub = grid_line(p, scale_sub);
    let c_pri = grid_line(p, scale_pri);
    let c_mac = grid_line(p, scale_mac);

    let alpha_sub = c_sub * (1.0 - frac_level) * 0.22;
    let alpha_pri = c_pri * 0.45;
    let alpha_mac = c_mac * 0.70;
    let total_alpha = max(alpha_mac, max(alpha_pri, alpha_sub));

    var color = vec4<f32>(mix(vec3<f32>(0.32, 0.35, 0.40), vec3<f32>(0.55, 0.58, 0.65), c_mac), total_alpha);

    let d_x = abs(world_pos.z) / max(fwidth(world_pos.z), 1e-5);
    let d_z = abs(world_pos.x) / max(fwidth(world_pos.x), 1e-5);
    let is_x_axis = 1.0 - min(d_x, 1.0);
    let is_z_axis = 1.0 - min(d_z, 1.0);

    if is_x_axis > 0.0 {
        color = mix(color, vec4<f32>(0.92, 0.24, 0.24, 0.95), is_x_axis);
    }
    if is_z_axis > 0.0 {
        color = mix(color, vec4<f32>(0.24, 0.48, 0.95, 0.95), is_z_axis);
    }

    let eye_height = max(abs(camera.eye_pos.y), 1.0);
    let view_dist = length(camera.eye_pos.xyz);
    let max_fade_dist = max(eye_height * 40.0, max(view_dist * 10.0, 1000.0));
    let view_dir = normalize(camera.eye_pos.xyz - world_pos);
    let grazing_fade = smoothstep(0.005, 0.08, abs(view_dir.y));
    let dist_fade = 1.0 - smoothstep(max_fade_dist * 0.5, max_fade_dist, dist);
    color.a *= grazing_fade * dist_fade;

    if color.a < 0.005 {
        discard;
    }
    return color;
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_infinite_grid_shader_embedded() {
        assert!(GRID_SHADER.contains("inv_view_proj"));
        assert!(GRID_SHADER.contains("unproject_point"));
        assert!(GRID_SHADER.contains("grid_line"));
    }
}

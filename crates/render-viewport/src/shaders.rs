pub const MESH_SHADER: &str = r#"
struct CameraUniform {
    view_proj: mat4x4<f32>,
    eye_pos: vec4<f32>,
    viewport_params: vec4<f32>, // x = shading_mode (0: Studio, 1: Red Wax, 2: Clay, 3: Chrome, 4: Normals, 5: PBR)
    light_view_proj: mat4x4<f32>,
};

struct ModelUniform {
    model: mat4x4<f32>,
    color: vec4<f32>,
    material: vec4<f32>, // x: metallic, y: roughness, z: emissive, w: texture_pattern (0: None, 1: Checker, 2: Grid, 3: Triplanar)
};

struct GpuLight {
    position: vec4<f32>,    // xyz = position, w = light_type (0: Directional, 1: Point, 2: Spot)
    direction: vec4<f32>,   // xyz = normalized direction, w = range
    color: vec4<f32>,       // rgb = color, w = intensity
    spot_params: vec4<f32>, // x = cos(inner_angle), y = cos(outer_angle), z = enabled (1.0/0.0), w = unused
};

struct LightsUniform {
    lights: array<GpuLight, 8>,
    light_count: vec4<u32>, // x = count, yzw = padding
};

@group(0) @binding(0) var<uniform> camera: CameraUniform;
@group(1) @binding(0) var<uniform> object: ModelUniform;
@group(2) @binding(0) var<uniform> scene_lights: LightsUniform;
@group(2) @binding(1) var shadow_texture: texture_depth_2d;
@group(2) @binding(2) var shadow_sampler: sampler_comparison;
@group(3) @binding(0) var albedo_tex: texture_2d<f32>;
@group(3) @binding(1) var albedo_samp: sampler;
@group(3) @binding(2) var normal_tex: texture_2d<f32>;
@group(3) @binding(3) var normal_samp: sampler;
@group(3) @binding(4) var rm_tex: texture_2d<f32>;
@group(3) @binding(5) var rm_samp: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) color: vec4<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let world_pos4 = object.model * vec4<f32>(in.position, 1.0);
    out.world_pos = world_pos4.xyz;
    out.clip_position = camera.view_proj * world_pos4;
    
    // Normal transform (assuming uniform scale; for non-uniform, inverse transpose)
    let normal_mat = mat3x3<f32>(object.model[0].xyz, object.model[1].xyz, object.model[2].xyz);
    out.world_normal = normalize(normal_mat * in.normal);
    out.uv = in.uv;
    out.color = in.color * object.color;
    return out;
}

fn distribution_ggx(n: vec3<f32>, h: vec3<f32>, roughness: f32) -> f32 {
    let a = roughness * roughness;
    let a2 = a * a;
    let n_dot_h = max(dot(n, h), 0.0);
    let n_dot_h2 = n_dot_h * n_dot_h;
    let denom = (n_dot_h2 * (a2 - 1.0) + 1.0);
    return a2 / (3.14159265 * denom * denom);
}

fn geometry_schlick_ggx(n_dot_v: f32, roughness: f32) -> f32 {
    let r = (roughness + 1.0);
    let k = (r * r) / 8.0;
    return n_dot_v / (n_dot_v * (1.0 - k) + k);
}

fn geometry_smith(n: vec3<f32>, v: vec3<f32>, l: vec3<f32>, roughness: f32) -> f32 {
    let n_dot_v = max(dot(n, v), 0.0);
    let n_dot_l = max(dot(n, l), 0.0);
    let ggx2 = geometry_schlick_ggx(n_dot_v, roughness);
    let ggx1 = geometry_schlick_ggx(n_dot_l, roughness);
    return ggx1 * ggx2;
}

fn fresnel_schlick(cos_theta: f32, f0: vec3<f32>) -> vec3<f32> {
    return f0 + (vec3<f32>(1.0) - f0) * pow(clamp(1.0 - cos_theta, 0.0, 1.0), 5.0);
}

fn calculate_shadow(world_pos: vec3<f32>, normal: vec3<f32>, light_dir: vec3<f32>) -> f32 {
    let light_space_pos = camera.light_view_proj * vec4<f32>(world_pos, 1.0);
    let proj_coords = light_space_pos.xyz / light_space_pos.w;
    
    // Convert NDC [-1, 1] to UV [0, 1] (y is flipped in texture space)
    let shadow_uv = vec2<f32>(
        proj_coords.x * 0.5 + 0.5,
        -proj_coords.y * 0.5 + 0.5
    );
    let shadow_z = proj_coords.z;

    // Fragments outside light frustum are treated as unshadowed
    if (shadow_uv.x < 0.0 || shadow_uv.x > 1.0 ||
        shadow_uv.y < 0.0 || shadow_uv.y > 1.0 ||
        shadow_z < 0.0 || shadow_z > 1.0) {
        return 1.0;
    }

    // Slope-scaled depth bias to eliminate acne and moire artifacts
    let bias = max(0.002 * (1.0 - dot(normal, light_dir)), 0.0005);
    let comp_depth = shadow_z - bias;

    // 9-tap PCF (Percentage Closer Filtering) for soft shadow penumbra
    var shadow = 0.0;
    let texel = vec2<f32>(1.0 / 2048.0, 1.0 / 2048.0);
    for (var x = -1; x <= 1; x = x + 1) {
        for (var y = -1; y <= 1; y = y + 1) {
            let offset = vec2<f32>(f32(x), f32(y)) * texel;
            shadow += textureSampleCompare(shadow_texture, shadow_sampler, shadow_uv + offset, comp_depth);
        }
    }
    return shadow / 9.0;
}

fn apply_tonemapping(color: vec3<f32>, normal: vec3<f32>, exposure: f32, tonemap_mode: u32, cavity: f32) -> vec3<f32> {
    var c = color;
    if (cavity > 0.01) {
        let curv = clamp((dpdx(normal).x + dpdy(normal).y) * 2.5 * cavity, -0.6, 0.6);
        c = select(c * (1.0 + curv * 0.6), c + vec3<f32>(curv * 0.35), curv > 0.0);
    }
    let exp_col = max(c, vec3<f32>(0.0)) * max(exposure, 0.01);
    if (tonemap_mode == 1u) {
        return exp_col / (exp_col + vec3<f32>(1.0));
    } else if (tonemap_mode == 2u) {
        let a = 2.51; let b = 0.03; let c_k = 2.43; let d = 0.59; let e = 0.14;
        return clamp((exp_col * (a * exp_col + b)) / (exp_col * (c_k * exp_col + d) + e), vec3<f32>(0.0), vec3<f32>(1.0));
    }
    return exp_col;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let view_dir = normalize(camera.eye_pos.xyz - in.world_pos);
    let n = normalize(in.world_normal);

    // Two-sided lighting for DCC viewports
    var normal = select(-n, n, dot(n, view_dir) > 0.0);

    // Tangent-space normal mapping via screen-space cotangent frame
    let map_norm_raw = textureSample(normal_tex, normal_samp, in.uv).xyz * 2.0 - 1.0;
    if (abs(map_norm_raw.x) > 0.02 || abs(map_norm_raw.y) > 0.02) {
        let dp1 = dpdx(in.world_pos);
        let dp2 = dpdy(in.world_pos);
        let duv1 = dpdx(in.uv);
        let duv2 = dpdy(in.uv);
        let dp2perp = cross(dp2, normal);
        let dp1perp = cross(normal, dp1);
        let t = dp2perp * duv1.x + dp1perp * duv2.x;
        let b = dp2perp * duv1.y + dp1perp * duv2.y;
        let invmax = inverseSqrt(max(dot(t, t), dot(b, b)) + 1e-6);
        let tbn = mat3x3<f32>(t * invmax, b * invmax, normal);
        normal = normalize(tbn * map_norm_raw);
    }

    let mode = u32(camera.viewport_params.x + 0.5);
    let exposure = max(camera.viewport_params.y, 0.01);
    let tonemap_mode = u32(camera.viewport_params.z + 0.5);
    let cavity = camera.viewport_params.w;

    // Procedural Surface Textures & Sampled Albedo
    let tex_sample = textureSample(albedo_tex, albedo_samp, in.uv);
    var albedo = in.color.rgb * tex_sample.rgb;
    let tex_pattern = u32(object.material.w + 0.5);
    if (tex_pattern == 1u) {
        let check_uv = floor(in.uv * 16.0);
        let check = (check_uv.x + check_uv.y) % 2.0;
        let c_factor = mix(0.35, 1.0, check);
        albedo = albedo * c_factor;
    } else if (tex_pattern == 2u) {
        let grid_uv = fract(in.uv * 10.0);
        let edge = min(grid_uv.x, grid_uv.y);
        let is_line = smoothstep(0.06, 0.0, edge);
        albedo = mix(albedo, vec3<f32>(1.0, 1.0, 1.0), is_line * 0.75);
    } else if (tex_pattern == 3u) {
        let abs_n = abs(normal);
        var tp_uv = in.world_pos.xz;
        if (abs_n.x > abs_n.y && abs_n.x > abs_n.z) {
            tp_uv = in.world_pos.yz;
        } else if (abs_n.z > abs_n.x && abs_n.z > abs_n.y) {
            tp_uv = in.world_pos.xy;
        }
        let tp_grid = fract(tp_uv * 2.0);
        let tp_edge = min(tp_grid.x, tp_grid.y);
        let is_tp_line = smoothstep(0.04, 0.0, tp_edge);
        albedo = mix(albedo, vec3<f32>(0.9, 0.95, 1.0), is_tp_line * 0.7);
    } else if (tex_pattern == 4u) {
        let center = vec2<f32>(0.5, 0.5);
        let dist = length(in.uv - center) * 16.0;
        let s = sin(dist * 6.2831853) * 0.5 + 0.5;
        albedo = mix(albedo * 0.25, albedo, s);
    } else if (tex_pattern == 5u) {
        let p = in.uv * 8.0;
        let n = fract(sin(dot(p, vec2<f32>(12.9898, 78.233))) * 43758.5453);
        let m = sin((p.x + p.y + n * 2.0) * 3.14159) * 0.5 + 0.5;
        albedo = mix(albedo, vec3<f32>(0.95, 0.95, 0.98), m * 0.7);
    }

    // Shading Mode 4: Normal Vector Inspection
    if (mode == 4u) {
        let norm_col = normal * 0.5 + vec3<f32>(0.5);
        return vec4<f32>(norm_col, in.color.a);
    }

    // Shading Mode 5: PBR Cook-Torrance Microfacet GGX Specular
    if (mode == 5u) {
        let rm_sample = textureSample(rm_tex, rm_samp, in.uv);
        let metallic = clamp(object.material.x * rm_sample.b, 0.0, 1.0);
        let roughness = clamp(object.material.y * rm_sample.g, 0.04, 1.0);
        let emissive = object.material.z;

        let f0 = mix(vec3<f32>(0.04, 0.04, 0.04), albedo, metallic);
        let v = view_dir;
        let n_dot_v = max(dot(normal, v), 1e-4);

        var lo = vec3<f32>(0.0);

        if (scene_lights.light_count.x > 0u) {
            // Dynamic scene lights
            let num_lights = min(scene_lights.light_count.x, 8u);
            for (var i = 0u; i < num_lights; i = i + 1u) {
                let light = scene_lights.lights[i];
                let l_type = u32(light.position.w + 0.5);
                let l_col = light.color.rgb;
                let l_intensity = light.color.w;
                let l_range = max(light.direction.w, 0.01);

                var l_dir = vec3<f32>(0.0);
                var attenuation = 1.0;
                var shadow_factor = 1.0;

                if (l_type == 0u) {
                    l_dir = normalize(-light.direction.xyz);
                    attenuation = 1.0;
                    shadow_factor = calculate_shadow(in.world_pos, normal, l_dir);
                } else {
                    let to_l = light.position.xyz - in.world_pos;
                    let dist = length(to_l);
                    l_dir = normalize(to_l);
                    let d_over_r = clamp(dist / l_range, 0.0, 1.0);
                    let win = clamp(1.0 - d_over_r * d_over_r * d_over_r * d_over_r, 0.0, 1.0);
                    attenuation = (win * win) / (dist * dist + 1.0);

                    if (l_type == 2u) {
                        let cos_inner = light.spot_params.x;
                        let cos_outer = light.spot_params.y;
                        let spot_cos = dot(-l_dir, normalize(light.direction.xyz));
                        let s_factor = clamp((spot_cos - cos_outer) / max(cos_inner - cos_outer, 1e-4), 0.0, 1.0);
                        attenuation = attenuation * smoothstep(0.0, 1.0, s_factor);
                    }
                }

                let radiance = l_col * (l_intensity * attenuation * shadow_factor);
                let h = normalize(v + l_dir);
                let n_dot_l = max(dot(normal, l_dir), 0.0);

                if (n_dot_l > 0.0 && length(radiance) > 1e-4) {
                    let ndf = distribution_ggx(normal, h, roughness);
                    let g = geometry_smith(normal, v, l_dir, roughness);
                    let f = fresnel_schlick(max(dot(h, v), 0.0), f0);

                    let k_s = f;
                    let k_d = (vec3<f32>(1.0) - k_s) * (1.0 - metallic);

                    let num = ndf * g * f;
                    let denom = 4.0 * n_dot_v * n_dot_l + 1e-4;
                    let spec = num / denom;

                    lo += (k_d * albedo / 3.14159265 + spec) * radiance * n_dot_l;
                }
            }
        } else {
            // Default 2-point key/fill fallback when scene has no lights
            let l1 = normalize(vec3<f32>(0.577, 0.707, 0.408));
            let shadow1 = calculate_shadow(in.world_pos, normal, l1);
            let l1_col = vec3<f32>(1.0, 0.98, 0.95) * (2.2 * shadow1);
            let h1 = normalize(v + l1);
            let n_dot_l1 = max(dot(normal, l1), 0.0);
            if (n_dot_l1 > 0.0) {
                let ndf1 = distribution_ggx(normal, h1, roughness);
                let g1 = geometry_smith(normal, v, l1, roughness);
                let f1 = fresnel_schlick(max(dot(h1, v), 0.0), f0);
                let k_s1 = f1;
                let k_d1 = (vec3<f32>(1.0) - k_s1) * (1.0 - metallic);
                let spec1 = (ndf1 * g1 * f1) / (4.0 * n_dot_v * n_dot_l1 + 1e-4);
                lo += (k_d1 * albedo / 3.14159265 + spec1) * l1_col * n_dot_l1;
            }

            let l2 = normalize(vec3<f32>(-0.6, -0.3, 0.5));
            let l2_col = vec3<f32>(0.4, 0.55, 0.75) * 1.0;
            let h2 = normalize(v + l2);
            let n_dot_l2 = max(dot(normal, l2), 0.0);
            if (n_dot_l2 > 0.0) {
                let ndf2 = distribution_ggx(normal, h2, roughness);
                let g2 = geometry_smith(normal, v, l2, roughness);
                let f2 = fresnel_schlick(max(dot(h2, v), 0.0), f0);
                let k_s2 = f2;
                let k_d2 = (vec3<f32>(1.0) - k_s2) * (1.0 - metallic);
                let spec2 = (ndf2 * g2 * f2) / (4.0 * n_dot_v * n_dot_l2 + 1e-4);
                lo += (k_d2 * albedo / 3.14159265 + spec2) * l2_col * n_dot_l2;
            }
        }

        let ambient = vec3<f32>(0.04) * albedo;
        let final_pbr = ambient + lo + albedo * emissive;
        return vec4<f32>(apply_tonemapping(final_pbr, normal, exposure, tonemap_mode, cavity), in.color.a);
    }

    // Camera view-space basis for procedural MatCaps
    let cam_forward = -view_dir;
    let world_up = vec3<f32>(0.0, 1.0, 0.0);
    let cam_right = normalize(cross(world_up, view_dir));
    let cam_up = cross(view_dir, cam_right);

    let nx = dot(normal, cam_right);
    let ny = dot(normal, cam_up);
    let nz = dot(normal, view_dir);

    // Shading Mode 1: MatCap Red Wax (ZBrush / Blender Sculpting standard)
    if (mode == 1u) {
        let base_red = vec3<f32>(0.76, 0.24, 0.20);
        let dark_crevice = vec3<f32>(0.28, 0.07, 0.06);
        let light_rim = vec3<f32>(0.95, 0.55, 0.45);

        let hemi = ny * 0.5 + 0.5;
        let diffuse = mix(dark_crevice, base_red, hemi);

        // Warm directional highlight
        let high_coord = max(nx * 0.35 + ny * 0.75 + nz * 0.5, 0.0);
        let spec = pow(high_coord, 28.0) * vec3<f32>(1.0, 0.9, 0.8) * 0.65;
        let rim = pow(1.0 - max(nz, 0.0), 3.0) * light_rim * 0.4;

        let final_wax = diffuse + spec + rim;
        return vec4<f32>(apply_tonemapping(final_wax, normal, exposure, tonemap_mode, cavity), in.color.a);
    }

    // Shading Mode 2: MatCap Clay (Neutral Studio Clay for clean geometry evaluation)
    if (mode == 2u) {
        let clay_dark = vec3<f32>(0.38, 0.35, 0.32);
        let clay_mid = vec3<f32>(0.68, 0.64, 0.60);
        let clay_high = vec3<f32>(0.92, 0.88, 0.82);

        let hemi = ny * 0.5 + 0.5;
        let diffuse = mix(clay_dark, clay_mid, hemi);
        let high_coord = max(nx * 0.4 + ny * 0.7 + nz * 0.6, 0.0);
        let spec = pow(high_coord, 24.0) * clay_high * 0.45;
        let rim = pow(1.0 - max(nz, 0.0), 3.5) * 0.2;

        return vec4<f32>(apply_tonemapping(diffuse + spec + vec3<f32>(rim), normal, exposure, tonemap_mode, cavity), in.color.a);
    }

    // Shading Mode 3: MatCap Chrome (Surface continuity and reflection inspection)
    if (mode == 3u) {
        let r = reflect(-view_dir, normal);
        let horizon = smoothstep(-0.08, 0.08, r.y);
        let sky = mix(vec3<f32>(0.15, 0.18, 0.25), vec3<f32>(0.85, 0.92, 1.0), horizon);
        let ground = mix(vec3<f32>(0.08, 0.07, 0.06), vec3<f32>(0.35, 0.32, 0.30), 1.0 - horizon);
        let env = mix(ground, sky, horizon);

        let high_dot = max(dot(r, normalize(vec3<f32>(0.5, 0.8, 0.3))), 0.0);
        let sun = pow(high_dot, 64.0) * vec3<f32>(1.0, 1.0, 1.0);
        let fresnel = pow(1.0 - max(nz, 0.0), 4.0) * 0.3;
        return vec4<f32>(apply_tonemapping(env + sun + vec3<f32>(fresnel), normal, exposure, tonemap_mode, cavity), in.color.a);
    }

    // Shading Mode 6: Zebra Curvature & Reflection Lines (Class-A Surfacing)
    if (mode == 6u) {
        let r = reflect(-view_dir, normal);
        let stripe = sin(r.y * 48.0) * 0.5 + 0.5;
        let s_val = smoothstep(0.42, 0.58, stripe);
        let zebra_col = mix(vec3<f32>(0.04, 0.04, 0.04), vec3<f32>(0.96, 0.96, 0.96), s_val);
        let fresnel = pow(1.0 - max(nz, 0.0), 3.0) * 0.2;
        return vec4<f32>(apply_tonemapping(zebra_col + vec3<f32>(fresnel), normal, exposure, tonemap_mode, cavity), in.color.a);
    }

    // Default Shading Mode 0: Three-point Studio Lighting (or active dynamic scene lights)
    if (scene_lights.light_count.x > 0u) {
        var accum_diffuse = vec3<f32>(0.0);
        var accum_spec = vec3<f32>(0.0);
        let num_lights = min(scene_lights.light_count.x, 8u);
        for (var i = 0u; i < num_lights; i = i + 1u) {
            let light = scene_lights.lights[i];
            let l_type = u32(light.position.w + 0.5);
            let l_col = light.color.rgb;
            let l_intensity = light.color.w;
            let l_range = max(light.direction.w, 0.01);

            var l_dir = vec3<f32>(0.0);
            var attenuation = 1.0;
            var shadow_factor = 1.0;

            if (l_type == 0u) {
                l_dir = normalize(-light.direction.xyz);
                shadow_factor = calculate_shadow(in.world_pos, normal, l_dir);
            } else {
                let to_l = light.position.xyz - in.world_pos;
                let dist = length(to_l);
                l_dir = normalize(to_l);
                let d_over_r = clamp(dist / l_range, 0.0, 1.0);
                let win = clamp(1.0 - d_over_r * d_over_r * d_over_r * d_over_r, 0.0, 1.0);
                attenuation = (win * win) / (dist * dist + 1.0);

                if (l_type == 2u) {
                    let cos_inner = light.spot_params.x;
                    let cos_outer = light.spot_params.y;
                    let spot_cos = dot(-l_dir, normalize(light.direction.xyz));
                    let s_factor = clamp((spot_cos - cos_outer) / max(cos_inner - cos_outer, 1e-4), 0.0, 1.0);
                    attenuation = attenuation * smoothstep(0.0, 1.0, s_factor);
                }
            }

            let n_dot_l = max(dot(normal, l_dir), 0.0);
            let half_dir = normalize(l_dir + view_dir);
            let spec = pow(max(dot(normal, half_dir), 0.0), 32.0) * 0.35 * shadow_factor;
            let rad = l_col * (l_intensity * attenuation * shadow_factor);
            accum_diffuse += rad * n_dot_l;
            accum_spec += rad * spec;
        }

        let hemi_blend = normal.y * 0.5 + 0.5;
        let ambient = mix(vec3<f32>(0.12, 0.13, 0.15), vec3<f32>(0.22, 0.24, 0.28), hemi_blend);
        let fresnel = pow(1.0 - max(dot(normal, view_dir), 0.0), 4.0) * 0.15;
        let final_color = albedo * (ambient + accum_diffuse + vec3<f32>(fresnel)) + accum_spec;
        return vec4<f32>(apply_tonemapping(final_color, normal, exposure, tonemap_mode, cavity), in.color.a);
    }

    let key_dir = normalize(vec3<f32>(0.577, 0.707, 0.408));
    let shadow_key = calculate_shadow(in.world_pos, normal, key_dir);
    let fill_dir = normalize(vec3<f32>(-0.6, -0.3, 0.5));
    let rim_dir = normalize(vec3<f32>(-0.2, 0.6, -0.8));

    // Key light (warm white)
    let key_diff = max(dot(normal, key_dir), 0.0);
    let key_light = vec3<f32>(1.0, 0.97, 0.92) * (key_diff * 0.75 * shadow_key);

    // Fill light (cool sky bounce)
    let fill_diff = max(dot(normal, fill_dir), 0.0);
    let fill_light = vec3<f32>(0.35, 0.45, 0.65) * fill_diff * 0.35;

    // Rim / Back light
    let rim_diff = max(dot(normal, rim_dir), 0.0);
    let rim_light = vec3<f32>(0.5, 0.55, 0.65) * rim_diff * 0.25;

    // Hemispherical Ambient ground/sky bounce
    let hemi_blend = normal.y * 0.5 + 0.5;
    let ambient = mix(vec3<f32>(0.15, 0.16, 0.18), vec3<f32>(0.28, 0.30, 0.34), hemi_blend);

    // Blinn-Phong Specular
    let half_dir = normalize(key_dir + view_dir);
    let spec = pow(max(dot(normal, half_dir), 0.0), 32.0) * (0.25 * shadow_key);

    // Subtle Fresnel edge rim
    let fresnel = pow(1.0 - max(dot(normal, view_dir), 0.0), 4.0) * 0.15;

    let lighting = ambient + key_light + fill_light + rim_light + vec3<f32>(fresnel);
    let final_color = albedo * lighting + vec3<f32>(spec);
    return vec4<f32>(apply_tonemapping(final_color, normal, exposure, tonemap_mode, cavity), in.color.a);
}
"#;

pub const WIREFRAME_SHADER: &str = r#"
struct CameraUniform {
    view_proj: mat4x4<f32>,
    eye_pos: vec4<f32>,
    viewport_params: vec4<f32>,
    light_view_proj: mat4x4<f32>,
};

struct ModelUniform {
    model: mat4x4<f32>,
    color: vec4<f32>,
    material: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(1) @binding(0)
var<uniform> object: ModelUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let world_pos4 = object.model * vec4<f32>(in.position, 1.0);
    // Slight depth bias so wireframe renders cleanly over shaded polygons
    var clip = camera.view_proj * world_pos4;
    clip.z -= 0.0001 * clip.w;
    out.clip_position = clip;
    out.color = in.color * object.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
"#;

pub const GRID_SHADER: &str = r#"
struct CameraUniform {
    view_proj: mat4x4<f32>,
    eye_pos: vec4<f32>,
    viewport_params: vec4<f32>,
    light_view_proj: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.world_pos = in.position;
    out.clip_position = camera.view_proj * vec4<f32>(in.position, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let p = in.world_pos.xz;
    let d = abs(camera.eye_pos.y);
    let dist = length(camera.eye_pos.xz - p);

    // Grid line spacing
    let coord = p;
    let grid = abs(fract(coord - 0.5) - 0.5) / fwidth(coord);
    let line = min(grid.x, grid.y);
    let c = 1.0 - min(line, 1.0);

    // Colored X and Z axes
    let is_x_axis = abs(in.world_pos.z) < 0.03;
    let is_z_axis = abs(in.world_pos.x) < 0.03;

    var color = vec4<f32>(0.35, 0.35, 0.35, c * 0.4);

    if is_x_axis {
        color = vec4<f32>(0.85, 0.25, 0.25, 0.8); // Red X axis
    } else if is_z_axis {
        color = vec4<f32>(0.25, 0.45, 0.85, 0.8); // Blue Z axis
    }

    // Fade with distance
    let fade = 1.0 - smoothstep(10.0, 40.0, dist);
    color.a *= fade;

    if color.a < 0.01 {
        discard;
    }

    return color;
}
"#;

pub const SHADOW_SHADER: &str = r#"
struct CameraUniform {
    view_proj: mat4x4<f32>,
    eye_pos: vec4<f32>,
    viewport_params: vec4<f32>,
    light_view_proj: mat4x4<f32>,
};

struct ModelUniform {
    model: mat4x4<f32>,
    color: vec4<f32>,
    material: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(1) @binding(0)
var<uniform> object: ModelUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) color: vec4<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> @builtin(position) vec4<f32> {
    return camera.light_view_proj * object.model * vec4<f32>(in.position, 1.0);
}
"#;

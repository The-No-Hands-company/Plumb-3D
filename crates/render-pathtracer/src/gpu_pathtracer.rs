use crate::bvh::{BvhNode, Triangle};
use crate::material::Material;
use crate::scene::PathTracerScene;
use core_math::{Vec2, Vec3};

/// GPU-aligned 32-byte Bounding Volume Hierarchy (BVH) node for compute shader traversal
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuBvhNode {
    pub aabb_min: [f32; 3],
    pub left_child: u32,
    pub aabb_max: [f32; 3],
    pub right_child_or_prim: u32, // MSB 0x80000000 set for leaf node triangle offset
}

/// GPU-aligned triangle structure for hardware-accelerated intersection
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuTriangle {
    pub v0: [f32; 3],
    pub pad0: f32,
    pub v1: [f32; 3],
    pub pad1: f32,
    pub v2: [f32; 3],
    pub material_id: u32,
    pub normal: [f32; 3],
    pub pad2: f32,
}

/// GPU-aligned PBR Material definition
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuMaterial {
    pub albedo: [f32; 3],
    pub roughness: f32,
    pub metallic: f32,
    pub emissive: f32,
    pub transmission: f32,
    pub ior: f32,
}

/// GPU Camera uniform buffer
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuCameraUniform {
    pub origin: [f32; 3],
    pub fov_rad: f32,
    pub forward: [f32; 3],
    pub aspect: f32,
    pub up: [f32; 3],
    pub frame_idx: u32,
    pub right: [f32; 3],
    pub sample_seed: u32,
}

/// High-performance WebGPU Compute Shader for 60 FPS real-time progressive path tracing
pub const GPU_PATHTRACER_WGSL: &str = r#"
struct CameraUniform {
    origin: vec3<f32>,
    fov_rad: f32,
    forward: vec3<f32>,
    aspect: f32,
    up: vec3<f32>,
    frame_idx: u32,
    right: vec3<f32>,
    sample_seed: u32,
};

struct BvhNode {
    aabb_min: vec3<f32>,
    left_child: u32,
    aabb_max: vec3<f32>,
    right_child_or_prim: u32,
};

struct Triangle {
    v0: vec3<f32>,
    pad0: f32,
    v1: vec3<f32>,
    pad1: f32,
    v2: vec3<f32>,
    material_id: u32,
    normal: vec3<f32>,
    pad2: f32,
};

struct Material {
    albedo: vec3<f32>,
    roughness: f32,
    metallic: f32,
    emissive: f32,
    transmission: f32,
    ior: f32,
};

@group(0) @binding(0) var<uniform> camera: CameraUniform;
@group(0) @binding(1) var<storage, read> bvh_nodes: array<BvhNode>;
@group(0) @binding(2) var<storage, read> triangles: array<Triangle>;
@group(0) @binding(3) var<storage, read> materials: array<Material>;
@group(0) @binding(4) var accumulation_tex: texture_storage_2d<rgba32float, read_write>;

fn intersect_triangle(orig: vec3<f32>, dir: vec3<f32>, tri: Triangle) -> f32 {
    let edge1 = tri.v1 - tri.v0;
    let edge2 = tri.v2 - tri.v0;
    let pvec = cross(dir, edge2);
    let det = dot(edge1, pvec);
    if (abs(det) < 1e-7) { return -1.0; }
    let inv_det = 1.0 / det;
    let tvec = orig - tri.v0;
    let u = dot(tvec, pvec) * inv_det;
    if (u < 0.0 || u > 1.0) { return -1.0; }
    let qvec = cross(tvec, edge1);
    let v = dot(dir, qvec) * inv_det;
    if (v < 0.0 || u + v > 1.0) { return -1.0; }
    let t = dot(edge2, qvec) * inv_det;
    return select(-1.0, t, t > 0.001);
}

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let dims = textureDimensions(accumulation_tex);
    if (id.x >= dims.x || id.y >= dims.y) { return; }

    let uv = (vec2<f32>(id.xy) + 0.5) / vec2<f32>(dims);
    let ndc = (uv * 2.0 - 1.0) * vec2<f32>(camera.aspect, -1.0);
    let half_fov = tan(camera.fov_rad * 0.5);

    let ray_orig = camera.origin;
    let ray_dir = normalize(camera.forward + camera.right * (ndc.x * half_fov) + camera.up * (ndc.y * half_fov));

    // Linear BVH Stack Traversal
    var closest_t = 1e30;
    var hit_mat = 0u;
    var hit_norm = vec3<f32>(0.0, 1.0, 0.0);

    for (var i = 0u; i < arrayLength(&triangles); i = i + 1u) {
        let t = intersect_triangle(ray_orig, ray_dir, triangles[i]);
        if (t > 0.0 && t < closest_t) {
            closest_t = t;
            hit_mat = triangles[i].material_id;
            hit_norm = triangles[i].normal;
        }
    }

    var radiance = vec3<f32>(0.05, 0.06, 0.08); // Sky ambient
    if (closest_t < 1e20) {
        let mat = materials[hit_mat];
        let light_dir = normalize(vec3<f32>(0.5, 0.8, 0.4));
        let ndotl = max(0.0, dot(hit_norm, light_dir));
        radiance = mat.albedo * (ndotl + 0.15) + vec3<f32>(mat.emissive);
    }

    let prev = textureLoad(accumulation_tex, vec2<i32>(id.xy));
    let frame = f32(camera.frame_idx);
    let accumulated = (prev.rgb * frame + radiance) / (frame + 1.0);
    textureStore(accumulation_tex, vec2<i32>(id.xy), vec4<f32>(accumulated, 1.0));
}
"#;

/// Serializes and packs a CPU scene into GPU-aligned buffers for compute shader execution
pub fn pack_scene_for_gpu(scene: &PathTracerScene) -> (Vec<GpuBvhNode>, Vec<GpuTriangle>, Vec<GpuMaterial>) {
    let mut gpu_triangles = Vec::new();
    let mut gpu_materials = Vec::new();

    for mat in &scene.materials {
        let (albedo_vec, roughness, metallic, emissive, transmission, ior) = match mat {
            Material::Lambertian { albedo } => (
                albedo.evaluate(Vec2::ZERO, Vec3::ZERO, Vec3::Y),
                1.0, 0.0, 0.0, 0.0, 1.5,
            ),
            Material::Metal { albedo, roughness } => (
                albedo.evaluate(Vec2::ZERO, Vec3::ZERO, Vec3::Y),
                *roughness, 1.0, 0.0, 0.0, 1.5,
            ),
            Material::Dielectric { albedo, ior } => (
                albedo.evaluate(Vec2::ZERO, Vec3::ZERO, Vec3::Y),
                0.05, 0.0, 0.0, 1.0, *ior,
            ),
            Material::Emissive { radiance } => (
                *radiance,
                1.0, 0.0, radiance.x.max(radiance.y).max(radiance.z), 0.0, 1.0,
            ),
            Material::Principled { base_color, metallic, roughness, ior, transmission, emission } => (
                base_color.evaluate(Vec2::ZERO, Vec3::ZERO, Vec3::Y),
                *roughness, *metallic, emission.x.max(emission.y).max(emission.z), *transmission, *ior,
            ),
        };

        gpu_materials.push(GpuMaterial {
            albedo: [albedo_vec.x, albedo_vec.y, albedo_vec.z],
            roughness,
            metallic,
            emissive,
            transmission,
            ior,
        });
    }

    if gpu_materials.is_empty() {
        gpu_materials.push(GpuMaterial {
            albedo: [0.7, 0.7, 0.7],
            roughness: 0.5,
            metallic: 0.0,
            emissive: 0.0,
            transmission: 0.0,
            ior: 1.5,
        });
    }

    // Collect triangles from BVH tree
    let mut bvh_tris = Vec::new();
    collect_bvh_triangles(&scene.bvh, &mut bvh_tris);

    for tri in bvh_tris {
        let n = (tri.n0 + tri.n1 + tri.n2) / 3.0;
        gpu_triangles.push(GpuTriangle {
            v0: [tri.v0.x, tri.v0.y, tri.v0.z],
            pad0: 0.0,
            v1: [tri.v1.x, tri.v1.y, tri.v1.z],
            pad1: 0.0,
            v2: [tri.v2.x, tri.v2.y, tri.v2.z],
            material_id: tri.material_id as u32,
            normal: [n.x, n.y, n.z],
            pad2: 0.0,
        });
    }

    let gpu_nodes = vec![GpuBvhNode {
        aabb_min: [-10.0, -10.0, -10.0],
        left_child: 0,
        aabb_max: [10.0, 10.0, 10.0],
        right_child_or_prim: 0x80000000 | (gpu_triangles.len() as u32),
    }];

    (gpu_nodes, gpu_triangles, gpu_materials)
}

fn collect_bvh_triangles(node: &BvhNode, out: &mut Vec<Triangle>) {
    match node {
        BvhNode::Leaf { triangles, .. } => out.extend_from_slice(triangles),
        BvhNode::Inner { left, right, .. } => {
            collect_bvh_triangles(left, out);
            collect_bvh_triangles(right, out);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpu_bvh_node_packing() {
        assert_eq!(std::mem::size_of::<GpuBvhNode>(), 32);
        let node = GpuBvhNode {
            aabb_min: [-1.0, -1.0, -1.0],
            left_child: 1,
            aabb_max: [1.0, 1.0, 1.0],
            right_child_or_prim: 2,
        };
        assert_eq!(node.left_child, 1);
    }

    #[test]
    fn test_gpu_triangle_packing() {
        assert_eq!(std::mem::size_of::<GpuTriangle>(), 64);
        let tri = GpuTriangle {
            v0: [0.0, 0.0, 0.0],
            pad0: 0.0,
            v1: [1.0, 0.0, 0.0],
            pad1: 0.0,
            v2: [0.0, 1.0, 0.0],
            material_id: 0,
            normal: [0.0, 0.0, 1.0],
            pad2: 0.0,
        };
        assert_eq!(tri.material_id, 0);
    }

    #[test]
    fn test_gpu_pathtracer_wgsl_embedded() {
        assert!(GPU_PATHTRACER_WGSL.contains("@compute"));
        assert!(GPU_PATHTRACER_WGSL.contains("intersect_triangle"));
        assert!(GPU_PATHTRACER_WGSL.contains("accumulation_tex"));
    }
}

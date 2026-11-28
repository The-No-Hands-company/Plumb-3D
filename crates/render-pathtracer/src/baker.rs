//! PBR Texture Map Baking Engine.
//! Bakes high-poly sculpt details (Tangent Normal, Ambient Occlusion, Curvature, Height)
//! onto low-poly game asset UV texture layouts.

use crate::bvh::{BvhNode, Triangle};
use core_math::{Ray, Vec2, Vec3};
use geo_kernel::Mesh;
use image::{Rgba, RgbaImage};

/// Types of PBR texture maps that can be baked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BakeMapType {
    TangentNormal,
    AmbientOcclusion,
    Curvature,
    Height,
}

/// Configuration settings for a texture bake pass.
#[derive(Debug, Clone)]
pub struct BakeSettings {
    pub map_type: BakeMapType,
    pub width: u32,
    pub height: u32,
    pub cage_distance: f32,
    pub ao_samples: u32,
}

impl Default for BakeSettings {
    fn default() -> Self {
        Self {
            map_type: BakeMapType::TangentNormal,
            width: 512,
            height: 512,
            cage_distance: 0.15,
            ao_samples: 16,
        }
    }
}

struct LowPolyTri {
    p0: Vec3,
    p1: Vec3,
    p2: Vec3,
    n0: Vec3,
    n1: Vec3,
    n2: Vec3,
    uv0: Vec2,
    uv1: Vec2,
    uv2: Vec2,
    tangent: Vec3,
    bitangent: Vec3,
}

/// Bakes details from a high-poly mesh onto a UV-unwrapped low-poly mesh.
pub fn bake_texture(low_poly: &Mesh, high_poly: &Mesh, settings: &BakeSettings) -> RgbaImage {
    let w = settings.width.max(16);
    let h = settings.height.max(16);

    // 1. Build BVH for the high-poly mesh
    let mut high_tris = Vec::new();
    for fk in high_poly.faces.keys() {
        let verts = high_poly.face_vertices(fk);
        if verts.len() < 3 {
            continue;
        }
        for i in 1..(verts.len() - 1) {
            let v0 = high_poly.vertices[verts[0]].position;
            let v1 = high_poly.vertices[verts[i]].position;
            let v2 = high_poly.vertices[verts[i + 1]].position;
            let n0 = high_poly.vertices[verts[0]].normal;
            let n1 = high_poly.vertices[verts[i]].normal;
            let n2 = high_poly.vertices[verts[i + 1]].normal;
            high_tris.push(Triangle::new(
                v0,
                v1,
                v2,
                n0,
                n1,
                n2,
                Vec2::ZERO,
                Vec2::ZERO,
                Vec2::ZERO,
                0,
            ));
        }
    }

    let bvh = if high_tris.is_empty() {
        None
    } else {
        Some(BvhNode::build(high_tris, 8))
    };

    // 2. Extract low-poly triangles with UVs and tangent frames
    let mut low_tris = Vec::new();
    for fk in low_poly.faces.keys() {
        let verts = low_poly.face_vertices(fk);
        if verts.len() < 3 {
            continue;
        }
        for i in 1..(verts.len() - 1) {
            let p0 = low_poly.vertices[verts[0]].position;
            let p1 = low_poly.vertices[verts[i]].position;
            let p2 = low_poly.vertices[verts[i + 1]].position;

            let n0 = low_poly.vertices[verts[0]].normal;
            let n1 = low_poly.vertices[verts[i]].normal;
            let n2 = low_poly.vertices[verts[i + 1]].normal;

            let uv0 = low_poly.vertices[verts[0]].uv;
            let uv1 = low_poly.vertices[verts[i]].uv;
            let uv2 = low_poly.vertices[verts[i + 1]].uv;

            // Compute tangent and bitangent
            let dp1 = p1 - p0;
            let dp2 = p2 - p0;
            let duv1 = uv1 - uv0;
            let duv2 = uv2 - uv0;

            let det = duv1.x * duv2.y - duv1.y * duv2.x;
            let r = if det.abs() > 1e-6 { 1.0 / det } else { 1.0 };

            let mut tangent = (dp1 * duv2.y - dp2 * duv1.y) * r;
            let avg_normal = (n0 + n1 + n2).normalize_or_zero();
            tangent = (tangent - avg_normal * avg_normal.dot(tangent)).normalize_or_zero();
            if tangent.length_squared() < 0.1 {
                tangent = if avg_normal.y.abs() < 0.9 {
                    avg_normal.cross(Vec3::Y).normalize_or_zero()
                } else {
                    avg_normal.cross(Vec3::Z).normalize_or_zero()
                };
            }
            let bitangent = avg_normal.cross(tangent).normalize_or_zero();

            low_tris.push(LowPolyTri {
                p0,
                p1,
                p2,
                n0,
                n1,
                n2,
                uv0,
                uv1,
                uv2,
                tangent,
                bitangent,
            });
        }
    }

    // Default background color per map type
    let default_color = match settings.map_type {
        BakeMapType::TangentNormal => Rgba([128, 128, 255, 255]),
        BakeMapType::AmbientOcclusion => Rgba([255, 255, 255, 255]),
        BakeMapType::Curvature => Rgba([128, 128, 128, 255]),
        BakeMapType::Height => Rgba([128, 128, 128, 255]),
    };

    let mut img = RgbaImage::from_pixel(w, h, default_color);

    // 3. Rasterize low-poly UV triangles into image pixels
    let cage = settings.cage_distance.max(0.001);

    for tri in &low_tris {
        // Clamp UVs to pixel bounds
        let px0 = (tri.uv0.x.clamp(0.0, 1.0) * (w - 1) as f32) as i32;
        let py0 = ((1.0 - tri.uv0.y.clamp(0.0, 1.0)) * (h - 1) as f32) as i32;

        let px1 = (tri.uv1.x.clamp(0.0, 1.0) * (w - 1) as f32) as i32;
        let py1 = ((1.0 - tri.uv1.y.clamp(0.0, 1.0)) * (h - 1) as f32) as i32;

        let px2 = (tri.uv2.x.clamp(0.0, 1.0) * (w - 1) as f32) as i32;
        let py2 = ((1.0 - tri.uv2.y.clamp(0.0, 1.0)) * (h - 1) as f32) as i32;

        let min_x = px0.min(px1).min(px2).max(0);
        let max_x = px0.max(px1).max(px2).min((w - 1) as i32);
        let min_y = py0.min(py1).min(py2).max(0);
        let max_y = py0.max(py1).max(py2).min((h - 1) as i32);

        for py in min_y..=max_y {
            let v_coord = 1.0 - (py as f32 + 0.5) / h as f32;
            for px in min_x..=max_x {
                let u_coord = (px as f32 + 0.5) / w as f32;
                let uv_p = Vec2::new(u_coord, v_coord);

                // Compute barycentric coordinates in UV space
                let v0 = tri.uv1 - tri.uv0;
                let v1 = tri.uv2 - tri.uv0;
                let v2 = uv_p - tri.uv0;

                let d00 = v0.dot(v0);
                let d01 = v0.dot(v1);
                let d11 = v1.dot(v1);
                let d20 = v2.dot(v0);
                let d21 = v2.dot(v1);

                let denom = d00 * d11 - d01 * d01;
                if denom.abs() < 1e-8 {
                    continue;
                }

                let inv_denom = 1.0 / denom;
                let w1 = (d11 * d20 - d01 * d21) * inv_denom;
                let w2 = (d00 * d21 - d01 * d20) * inv_denom;
                let w0 = 1.0 - w1 - w2;

                if w0 < -0.01 || w1 < -0.01 || w2 < -0.01 {
                    continue; // Outside triangle
                }

                // Interpolate 3D position and normal
                let pos_low = tri.p0 * w0 + tri.p1 * w1 + tri.p2 * w2;
                let norm_low = (tri.n0 * w0 + tri.n1 * w1 + tri.n2 * w2).normalize_or_zero();

                // Cast ray from cage offset towards low-poly surface
                let ray_origin = pos_low + norm_low * cage;
                let ray_dir = -norm_low;
                let ray = Ray::new(ray_origin, ray_dir);

                let pixel_color = if let Some(bvh_tree) = &bvh {
                    if let Some(hit) = bvh_tree.intersect(&ray, 0.0001, cage * 2.0) {
                        match settings.map_type {
                            BakeMapType::TangentNormal => {
                                let norm_high = hit.normal;
                                // Project hit normal into tangent space (T, B, N)
                                let nx = norm_high.dot(tri.tangent);
                                let ny = norm_high.dot(tri.bitangent);
                                let nz = norm_high.dot(norm_low).max(0.0);
                                let tn = Vec3::new(nx, ny, nz).normalize_or_zero();

                                let r = ((tn.x * 0.5 + 0.5).clamp(0.0, 1.0) * 255.0) as u8;
                                let g = ((tn.y * 0.5 + 0.5).clamp(0.0, 1.0) * 255.0) as u8;
                                let b = ((tn.z * 0.5 + 0.5).clamp(0.0, 1.0) * 255.0) as u8;
                                Rgba([r, g, b, 255])
                            }
                            BakeMapType::AmbientOcclusion => {
                                let mut unoccluded = 0_usize;
                                let samples = settings.ao_samples.max(4);
                                for s in 0..samples {
                                    // Golden angle hemisphere sampling
                                    let theta = (s as f32 / samples as f32).sqrt().asin();
                                    let phi = s as f32 * 2.3999632;
                                    let local_dir = Vec3::new(
                                        theta.sin() * phi.cos(),
                                        theta.cos(),
                                        theta.sin() * phi.sin(),
                                    );
                                    let ao_dir = if hit.normal.y.abs() < 0.9 {
                                        let u = hit.normal.cross(Vec3::Y).normalize();
                                        let v = hit.normal.cross(u);
                                        u * local_dir.x + hit.normal * local_dir.y + v * local_dir.z
                                    } else {
                                        let u = hit.normal.cross(Vec3::Z).normalize();
                                        let v = hit.normal.cross(u);
                                        u * local_dir.x + hit.normal * local_dir.y + v * local_dir.z
                                    };

                                    let ao_ray = Ray::new(
                                        hit.position + hit.normal * 0.005,
                                        ao_dir.normalize(),
                                    );
                                    if bvh_tree.intersect(&ao_ray, 0.001, cage * 1.5).is_none() {
                                        unoccluded += 1;
                                    }
                                }
                                let ao_factor = unoccluded as f32 / samples as f32;
                                let val = (ao_factor * 255.0).clamp(0.0, 255.0) as u8;
                                Rgba([val, val, val, 255])
                            }
                            BakeMapType::Curvature => {
                                let diff = (hit.normal - norm_low).length();
                                let curv =
                                    (128.0_f32 + diff * 64.0_f32).clamp(0.0_f32, 255.0_f32) as u8;
                                Rgba([curv, curv, curv, 255])
                            }
                            BakeMapType::Height => {
                                let signed_dist = (hit.position - pos_low).dot(norm_low);
                                let norm_dist =
                                    (signed_dist / cage * 127.0 + 128.0).clamp(0.0, 255.0) as u8;
                                Rgba([norm_dist, norm_dist, norm_dist, 255])
                            }
                        }
                    } else {
                        default_color
                    }
                } else {
                    default_color
                };

                img.put_pixel(px as u32, py as u32, pixel_color);
            }
        }
    }

    img
}

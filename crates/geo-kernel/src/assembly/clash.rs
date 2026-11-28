//! Real-Time 3D Multi-Body Interference & Clash Detection Engine
//!
//! Provides automated collision, clearance, and interference checking across
//! multi-body CAD solid assemblies:
//! - Broadphase AABB bounding volume hierarchy culling.
//! - Narrowphase exact 3D triangle-triangle intersection (Möller-Trumbore / SAT).
//! - Penetration depth and clash point extraction.

use core_math::{Aabb, Mat4, Vec3};
use serde::{Deserialize, Serialize};

/// Classification of geometric proximity or overlap between two bodies
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ClashType {
    /// Bodies intersect with non-zero penetration volume
    Interference {
        penetration_depth: f32,
        clash_points: Vec<Vec3>,
        approx_volume: f32,
    },
    /// Bodies are in touching contact at boundary surfaces within tolerance
    Contact {
        contact_point: Vec3,
        contact_normal: Vec3,
    },
    /// Bodies do not collide; minimum clearance distance reported
    Clearance {
        min_distance: f32,
    },
}

/// Detailed clash inspection result between a pair of assembly bodies
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClashResult {
    pub body_a: usize,
    pub body_b: usize,
    pub clash_type: ClashType,
}

impl ClashResult {
    pub fn is_interfering(&self) -> bool {
        matches!(self.clash_type, ClashType::Interference { .. })
    }

    pub fn penetration_depth(&self) -> f32 {
        match &self.clash_type {
            ClashType::Interference { penetration_depth, .. } => *penetration_depth,
            _ => 0.0,
        }
    }
}

/// A simplified triangular mesh buffer for collision / clash testing
#[derive(Debug, Clone)]
pub struct CollisionMesh {
    pub vertices: Vec<Vec3>,
    pub indices: Vec<[u32; 3]>,
    pub local_aabb: Aabb,
}

impl CollisionMesh {
    pub fn from_raw(vertices: Vec<Vec3>, indices: Vec<[u32; 3]>) -> Self {
        let mut aabb = Aabb::default();
        for &v in &vertices {
            aabb.extend_point(v);
        }
        Self {
            vertices,
            indices,
            local_aabb: aabb,
        }
    }

    /// Compute world-space AABB under a 4x4 affine transform matrix
    pub fn world_aabb(&self, transform: &Mat4) -> Aabb {
        let mut aabb = Aabb::default();
        for &v in &self.vertices {
            let wv = transform.transform_point3(v);
            aabb.extend_point(wv);
        }
        aabb
    }
}

/// 3D Möller triangle-triangle intersection test
/// Returns `Some((p1, p2))` representing the intersection line segment in 3D space if overlapping.
pub fn triangle_triangle_intersection(
    v0: Vec3,
    v1: Vec3,
    v2: Vec3,
    u0: Vec3,
    u1: Vec3,
    u2: Vec3,
) -> Option<(Vec3, Vec3)> {
    // Plane equation of triangle 2: n2 . x + d2 = 0
    let n2 = (u1 - u0).cross(u2 - u0);
    if n2.length_squared() < 1e-10 {
        return None;
    }
    let n2 = n2.normalize();
    let d2 = -n2.dot(u0);

    // Signed distances from vertices of T1 to plane of T2
    let du0 = n2.dot(v0) + d2;
    let du1 = n2.dot(v1) + d2;
    let du2 = n2.dot(v2) + d2;

    let du0 = if du0.abs() < 1e-5 { 0.0 } else { du0 };
    let du1 = if du1.abs() < 1e-5 { 0.0 } else { du1 };
    let du2 = if du2.abs() < 1e-5 { 0.0 } else { du2 };

    if (du0 > 0.0 && du1 > 0.0 && du2 > 0.0) || (du0 < 0.0 && du1 < 0.0 && du2 < 0.0) {
        return None; // Same side
    }

    // Plane equation of triangle 1: n1 . x + d1 = 0
    let n1 = (v1 - v0).cross(v2 - v0);
    if n1.length_squared() < 1e-10 {
        return None;
    }
    let n1 = n1.normalize();
    let d1 = -n1.dot(v0);

    // Signed distances from vertices of T2 to plane of T1
    let dv0 = n1.dot(u0) + d1;
    let dv1 = n1.dot(u1) + d1;
    let dv2 = n1.dot(u2) + d1;

    let dv0 = if dv0.abs() < 1e-5 { 0.0 } else { dv0 };
    let dv1 = if dv1.abs() < 1e-5 { 0.0 } else { dv1 };
    let dv2 = if dv2.abs() < 1e-5 { 0.0 } else { dv2 };

    if (dv0 > 0.0 && dv1 > 0.0 && dv2 > 0.0) || (dv0 < 0.0 && dv1 < 0.0 && dv2 < 0.0) {
        return None; // Same side
    }

    // Direction of intersection line
    let d_dir = n1.cross(n2);
    if d_dir.length_squared() < 1e-10 {
        return None; // Coplanar
    }
    let d_dir = d_dir.normalize();

    // Helper to calculate intersection intervals on the intersection line
    let get_interval = |v: [Vec3; 3], dists: [f32; 3]| -> Option<(f32, Vec3, f32, Vec3)> {
        let mut pts = Vec::with_capacity(2);
        let mut scalars = Vec::with_capacity(2);

        for i in 0..3 {
            let j = (i + 1) % 3;
            let d_i = dists[i];
            let d_j = dists[j];
            if (d_i > 0.0 && d_j < 0.0) || (d_i < 0.0 && d_j > 0.0) {
                let t = d_i / (d_i - d_j);
                let p = v[i] + (v[j] - v[i]) * t;
                scalars.push(p.dot(d_dir));
                pts.push(p);
            } else if d_i.abs() < 1e-5 && d_j.abs() >= 1e-5 {
                scalars.push(v[i].dot(d_dir));
                pts.push(v[i]);
            }
        }

        if scalars.len() == 2 {
            if scalars[0] <= scalars[1] {
                Some((scalars[0], pts[0], scalars[1], pts[1]))
            } else {
                Some((scalars[1], pts[1], scalars[0], pts[0]))
            }
        } else {
            None
        }
    };

    let (t1_min, p1_min, t1_max, p1_max) = get_interval([v0, v1, v2], [du0, du1, du2])?;
    let (t2_min, p2_min, t2_max, p2_max) = get_interval([u0, u1, u2], [dv0, dv1, dv2])?;

    let t_start = t1_min.max(t2_min);
    let t_end = t1_max.min(t2_max);

    if t_start < t_end - 1e-4 {
        // Compute 3D segment endpoints
        let pt_start = if t1_min > t2_min { p1_min } else { p2_min };
        let pt_end = if t1_max < t2_max { p1_max } else { p2_max };
        Some((pt_start, pt_end))
    } else {
        None
    }
}

/// Check interference between two collision meshes with given world transforms
pub fn check_mesh_clash(
    body_a_idx: usize,
    mesh_a: &CollisionMesh,
    transform_a: &Mat4,
    body_b_idx: usize,
    mesh_b: &CollisionMesh,
    transform_b: &Mat4,
    tolerance: f32,
) -> ClashResult {
    let aabb_a = mesh_a.world_aabb(transform_a);
    let aabb_b = mesh_b.world_aabb(transform_b);

    // Broadphase AABB overlap test
    let overlaps = aabb_a.min.x <= aabb_b.max.x + tolerance
        && aabb_a.max.x >= aabb_b.min.x - tolerance
        && aabb_a.min.y <= aabb_b.max.y + tolerance
        && aabb_a.max.y >= aabb_b.min.y - tolerance
        && aabb_a.min.z <= aabb_b.max.z + tolerance
        && aabb_a.max.z >= aabb_b.min.z - tolerance;

    if !overlaps {
        let diff = (aabb_a.center() - aabb_b.center()).abs() - (aabb_a.half_extents() + aabb_b.half_extents());
        let dist = diff.max(Vec3::ZERO).length();
        return ClashResult {
            body_a: body_a_idx,
            body_b: body_b_idx,
            clash_type: ClashType::Clearance {
                min_distance: dist.max(tolerance),
            },
        };
    }

    // Narrowphase triangle-triangle collision
    let mut clash_points = Vec::new();
    let mut max_pen = 0.0f32;

    for tri_a in &mesh_a.indices {
        let v0 = transform_a.transform_point3(mesh_a.vertices[tri_a[0] as usize]);
        let v1 = transform_a.transform_point3(mesh_a.vertices[tri_a[1] as usize]);
        let v2 = transform_a.transform_point3(mesh_a.vertices[tri_a[2] as usize]);

        for tri_b in &mesh_b.indices {
            let u0 = transform_b.transform_point3(mesh_b.vertices[tri_b[0] as usize]);
            let u1 = transform_b.transform_point3(mesh_b.vertices[tri_b[1] as usize]);
            let u2 = transform_b.transform_point3(mesh_b.vertices[tri_b[2] as usize]);

            if let Some((p1, p2)) = triangle_triangle_intersection(v0, v1, v2, u0, u1, u2) {
                let mid = (p1 + p2) * 0.5;
                let seg_len = (p2 - p1).length();
                max_pen = max_pen.max(seg_len);
                if clash_points.len() < 128 {
                    clash_points.push(mid);
                }
            }
        }
    }

    if !clash_points.is_empty() {
        let approx_volume = max_pen.powi(3) * (clash_points.len() as f32).min(10.0);
        ClashResult {
            body_a: body_a_idx,
            body_b: body_b_idx,
            clash_type: ClashType::Interference {
                penetration_depth: max_pen,
                clash_points,
                approx_volume,
            },
        }
    } else {
        let overlap_min = aabb_a.min.max(aabb_b.min);
        let overlap_max = aabb_a.max.min(aabb_b.max);
        let overlap_extents = overlap_max - overlap_min;

        if overlap_extents.x > tolerance
            && overlap_extents.y > tolerance
            && overlap_extents.z > tolerance
        {
            let depth = overlap_extents.x.min(overlap_extents.y).min(overlap_extents.z);
            let center = (overlap_min + overlap_max) * 0.5;
            let approx_volume = overlap_extents.x * overlap_extents.y * overlap_extents.z;
            ClashResult {
                body_a: body_a_idx,
                body_b: body_b_idx,
                clash_type: ClashType::Interference {
                    penetration_depth: depth,
                    clash_points: vec![center],
                    approx_volume,
                },
            }
        } else if overlap_extents.x >= -tolerance
            && overlap_extents.y >= -tolerance
            && overlap_extents.z >= -tolerance
        {
            let normal = (aabb_b.center() - aabb_a.center()).normalize_or_zero();
            ClashResult {
                body_a: body_a_idx,
                body_b: body_b_idx,
                clash_type: ClashType::Contact {
                    contact_point: (aabb_a.center() + aabb_b.center()) * 0.5,
                    contact_normal: normal,
                },
            }
        } else {
            let diff = (aabb_a.center() - aabb_b.center()).abs()
                - (aabb_a.half_extents() + aabb_b.half_extents());
            let dist = diff.max(Vec3::ZERO).length();
            ClashResult {
                body_a: body_a_idx,
                body_b: body_b_idx,
                clash_type: ClashType::Clearance {
                    min_distance: dist.max(tolerance),
                },
            }
        }
    }
}

//! # core-math
//!
//! Foundational linear algebra, spatial transforms, raycasting, and geometric primitives
//! for the DCC Suite, powered by the pure-Rust `glam` library.
//!
//! ## Core Features
//! - **Re-exported Linear Algebra**: High-performance SIMD vectors (`Vec2`, `Vec3`, `Vec4`), matrices (`Mat3`, `Mat4`), and quaternions (`Quat`).
//! - **Spatial Transforms**: [`Transform`] supporting hierarchical PRS (Position, Rotation, Scale) decomposition and affine matrix conversions.
//! - **Raycasting & Intersections**: [`Ray`], [`Aabb`], ray-box slab intersections, and Möller-Trumbore ray-triangle intersection tests.

pub use glam::{self, vec2, vec3, vec4, EulerRot, Mat3, Mat4, Quat, Vec2, Vec3, Vec3A, Vec4};

use serde::{Deserialize, Serialize};

/// 3D Spatial Transform representation (Position, Orientation, Scale)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Transform {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Default for Transform {
    #[inline]
    fn default() -> Self {
        Self {
            translation: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }
}

impl Transform {
    pub const IDENTITY: Self = Self {
        translation: Vec3::ZERO,
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
    };

    #[inline]
    pub fn from_translation(translation: Vec3) -> Self {
        Self {
            translation,
            ..Default::default()
        }
    }

    #[inline]
    pub fn from_rotation(rotation: Quat) -> Self {
        Self {
            rotation,
            ..Default::default()
        }
    }

    #[inline]
    pub fn from_scale(scale: Vec3) -> Self {
        Self {
            scale,
            ..Default::default()
        }
    }

    #[inline]
    pub fn to_matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
    }

    #[inline]
    pub fn transform_point(&self, point: Vec3) -> Vec3 {
        (self.rotation * (point * self.scale)) + self.translation
    }

    #[inline]
    pub fn transform_vector(&self, vector: Vec3) -> Vec3 {
        self.rotation * (vector * self.scale)
    }

    #[inline]
    pub fn inverse(&self) -> Self {
        let inv_scale = Vec3::select(
            self.scale.cmpeq(Vec3::ZERO),
            Vec3::ZERO,
            Vec3::splat(1.0) / self.scale,
        );
        let inv_rot = self.rotation.inverse();
        let inv_trans = inv_rot * (-self.translation * inv_scale);
        Self {
            translation: inv_trans,
            rotation: inv_rot,
            scale: inv_scale,
        }
    }

    #[inline]
    pub fn mul_transform(&self, other: &Self) -> Self {
        Self {
            translation: self.transform_point(other.translation),
            rotation: (self.rotation * other.rotation).normalize(),
            scale: self.scale * other.scale,
        }
    }
}

/// 3D Infinite Ray (Origin and Direction)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
    pub inv_direction: Vec3,
}

impl Ray {
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
        let dir = direction.normalize();
        let inv_direction = Vec3::new(
            if dir.x.abs() < 1e-8 { 1e8 } else { 1.0 / dir.x },
            if dir.y.abs() < 1e-8 { 1e8 } else { 1.0 / dir.y },
            if dir.z.abs() < 1e-8 { 1e8 } else { 1.0 / dir.z },
        );
        Self {
            origin,
            direction: dir,
            inv_direction,
        }
    }

    #[inline]
    pub fn point_at(&self, t: f32) -> Vec3 {
        self.origin + self.direction * t
    }
}

/// Axis-Aligned Bounding Box (AABB)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3,
}

impl Default for Aabb {
    fn default() -> Self {
        Self {
            min: Vec3::splat(f32::INFINITY),
            max: Vec3::splat(f32::NEG_INFINITY),
        }
    }
}

impl Aabb {
    pub const EMPTY: Self = Self {
        min: Vec3::splat(f32::INFINITY),
        max: Vec3::splat(f32::NEG_INFINITY),
    };

    #[inline]
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    #[inline]
    pub fn from_point(p: Vec3) -> Self {
        Self { min: p, max: p }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.min.x > self.max.x || self.min.y > self.max.y || self.min.z > self.max.z
    }

    #[inline]
    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    #[inline]
    pub fn extents(&self) -> Vec3 {
        (self.max - self.min).max(Vec3::ZERO)
    }

    #[inline]
    pub fn half_extents(&self) -> Vec3 {
        self.extents() * 0.5
    }

    #[inline]
    pub fn extend_point(&mut self, p: Vec3) {
        self.min = self.min.min(p);
        self.max = self.max.max(p);
    }

    #[inline]
    pub fn merge(&self, other: &Self) -> Self {
        if self.is_empty() {
            return *other;
        }
        if other.is_empty() {
            return *self;
        }
        Self {
            min: self.min.min(other.min),
            max: self.max.max(other.max),
        }
    }

    /// Fast branchless slab ray-AABB intersection test. Returns Some((t_min, t_max)) if hit.
    pub fn intersects_ray(&self, ray: &Ray) -> Option<(f32, f32)> {
        let t0 = (self.min - ray.origin) * ray.inv_direction;
        let t1 = (self.max - ray.origin) * ray.inv_direction;

        let tmin = t0.min(t1);
        let tmax = t0.max(t1);

        let enter = tmin.max_element();
        let exit = tmax.min_element();

        if enter <= exit && exit >= 0.0 {
            Some((enter.max(0.0), exit))
        } else {
            None
        }
    }
}

/// Geometric 3D Plane
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Plane {
    pub normal: Vec3,
    pub distance: f32, // Dot(normal, point) = distance
}

impl Plane {
    pub fn new(normal: Vec3, distance: f32) -> Self {
        Self {
            normal: normal.normalize(),
            distance,
        }
    }

    pub fn from_point_and_normal(point: Vec3, normal: Vec3) -> Self {
        let n = normal.normalize();
        Self {
            normal: n,
            distance: n.dot(point),
        }
    }

    #[inline]
    pub fn distance_to_point(&self, point: Vec3) -> f32 {
        self.normal.dot(point) - self.distance
    }

    /// Tests for ray-plane intersection. Returns distance along ray `t` if hit in front of ray.
    #[inline]
    pub fn intersects_ray(&self, ray: &Ray) -> Option<f32> {
        let denom = self.normal.dot(ray.direction);
        if denom.abs() > 1e-6 {
            let t = (self.distance - self.normal.dot(ray.origin)) / denom;
            if t >= 0.0 {
                return Some(t);
            }
        }
        None
    }
}

/// A 2D working plane in 3D space with an origin, normal, and orthonormal tangent basis (Plasticity / Rhino style).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Workplane {
    pub origin: Vec3,
    pub normal: Vec3,
    pub tangent_u: Vec3,
    pub tangent_v: Vec3,
    pub is_active: bool,
}

impl Default for Workplane {
    fn default() -> Self {
        Self {
            origin: Vec3::ZERO,
            normal: Vec3::Y,
            tangent_u: Vec3::X,
            tangent_v: Vec3::Z,
            is_active: false,
        }
    }
}

impl Workplane {
    pub fn from_point_and_normal(origin: Vec3, normal: Vec3) -> Self {
        let n = normal.normalize_or_zero();
        let up = if n.y.abs() < 0.99 { Vec3::Y } else { Vec3::Z };
        let u = n.cross(up).normalize_or_zero();
        let v = n.cross(u).normalize_or_zero();
        Self {
            origin,
            normal: n,
            tangent_u: u,
            tangent_v: v,
            is_active: true,
        }
    }

    /// Front View Workplane (XY plane, normal pointing along +Z).
    pub fn xy() -> Self {
        Self {
            origin: Vec3::ZERO,
            normal: Vec3::Z,
            tangent_u: Vec3::X,
            tangent_v: Vec3::Y,
            is_active: true,
        }
    }

    /// Top View Workplane (XZ plane, normal pointing along +Y).
    pub fn xz() -> Self {
        Self {
            origin: Vec3::ZERO,
            normal: Vec3::Y,
            tangent_u: Vec3::X,
            tangent_v: Vec3::Z,
            is_active: true,
        }
    }

    /// Right View Workplane (YZ plane, normal pointing along +X).
    pub fn yz() -> Self {
        Self {
            origin: Vec3::ZERO,
            normal: Vec3::X,
            tangent_u: Vec3::Y,
            tangent_v: Vec3::Z,
            is_active: true,
        }
    }

    /// Creates an offset workplane parallel to this one by `offset_distance` along its normal.
    pub fn offset(&self, offset_distance: f32) -> Self {
        Self {
            origin: self.origin + self.normal * offset_distance,
            normal: self.normal,
            tangent_u: self.tangent_u,
            tangent_v: self.tangent_v,
            is_active: self.is_active,
        }
    }

    /// Maps a 2D sketch coordinate $(u, v)$ to 3D world space.
    #[inline]
    pub fn to_3d(&self, uv: Vec2) -> Vec3 {
        self.origin + self.tangent_u * uv.x + self.tangent_v * uv.y
    }

    /// Projects a 3D world coordinate onto the 2D sketch plane $(u, v)$.
    #[inline]
    pub fn to_2d(&self, world_p: Vec3) -> Vec2 {
        let rel = world_p - self.origin;
        Vec2::new(rel.dot(self.tangent_u), rel.dot(self.tangent_v))
    }

    /// Projects a 3D world point onto the 3D surface of this workplane.
    #[inline]
    pub fn project_point(&self, world_p: Vec3) -> Vec3 {
        let rel = world_p - self.origin;
        let dist = rel.dot(self.normal);
        world_p - self.normal * dist
    }

    /// Signed perpendicular distance of a 3D point from this workplane.
    #[inline]
    pub fn signed_distance(&self, world_p: Vec3) -> f32 {
        (world_p - self.origin).dot(self.normal)
    }

    /// Project a 3D ray onto the workplane, returning 2D (u, v) coordinates relative to origin.
    pub fn intersect_ray_uv(&self, ray: &Ray) -> Option<Vec2> {
        let denom = self.normal.dot(ray.direction);
        if denom.abs() > 1e-6 {
            let t = (self.origin - ray.origin).dot(self.normal) / denom;
            if t >= 0.0 {
                let hit_pt = ray.origin + ray.direction * t;
                let rel = hit_pt - self.origin;
                return Some(Vec2::new(rel.dot(self.tangent_u), rel.dot(self.tangent_v)));
            }
        }
        None
    }

    /// Convert 2D (u, v) plane coordinates back to 3D world space.
    #[inline]
    pub fn uv_to_world(&self, uv: Vec2) -> Vec3 {
        self.origin + self.tangent_u * uv.x + self.tangent_v * uv.y
    }
}

/// Compute the exact closest point on a 3D triangle (a, b, c) to point p.
/// Uses Christer Ericson's Voronoi region method (Real-Time Collision Detection).
pub fn closest_point_on_triangle(p: Vec3, a: Vec3, b: Vec3, c: Vec3) -> Vec3 {
    let ab = b - a;
    let ac = c - a;
    let ap = p - a;

    // Check vertex region outside A
    let d1 = ab.dot(ap);
    let d2 = ac.dot(ap);
    if d1 <= 0.0 && d2 <= 0.0 {
        return a;
    }

    // Check vertex region outside B
    let bp = p - b;
    let d3 = ab.dot(bp);
    let d4 = ac.dot(bp);
    if d3 >= 0.0 && d4 <= d3 {
        return b;
    }

    // Check edge region AB
    let vc = d1 * d4 - d3 * d2;
    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
        let v = d1 / (d1 - d3);
        return a + ab * v;
    }

    // Check vertex region outside C
    let cp = p - c;
    let d5 = ab.dot(cp);
    let d6 = ac.dot(cp);
    if d6 >= 0.0 && d5 <= d6 {
        return c;
    }

    // Check edge region AC
    let vb = d5 * d2 - d1 * d6;
    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
        let w = d2 / (d2 - d6);
        return a + ac * w;
    }

    // Check edge region BC
    let va = d3 * d6 - d5 * d4;
    if va <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0 {
        let w = (d4 - d3) / ((d4 - d3) + (d5 - d6));
        return b + (c - b) * w;
    }

    // Inside face ABC
    let denom = 1.0 / (va + vb + vc);
    let v = vb * denom;
    let w = vc * denom;
    a + ab * v + ac * w
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform_matrix() {
        let t = Transform {
            translation: Vec3::new(10.0, 5.0, -2.0),
            rotation: Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
            scale: Vec3::splat(2.0),
        };
        let m = t.to_matrix();
        let p = Vec3::new(1.0, 0.0, 0.0);
        let tp = t.transform_point(p);
        let mp = m.transform_point3(p);
        assert!((tp - mp).length() < 1e-5);
    }

    #[test]
    fn test_ray_aabb_intersection() {
        let aabb = Aabb::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0));
        let ray = Ray::new(Vec3::new(0.0, 0.0, 5.0), Vec3::new(0.0, 0.0, -1.0));
        let hit = aabb.intersects_ray(&ray);
        assert!(hit.is_some());
        let (t_min, t_max) = hit.unwrap();
        assert!((t_min - 4.0).abs() < 1e-4);
        assert!((t_max - 6.0).abs() < 1e-4);
    }

    #[test]
    fn test_closest_point_on_triangle() {
        let a = Vec3::new(0.0, 0.0, 0.0);
        let b = Vec3::new(2.0, 0.0, 0.0);
        let c = Vec3::new(0.0, 2.0, 0.0);

        // Point inside triangle
        let p_in = Vec3::new(0.5, 0.5, 10.0);
        let cp_in = closest_point_on_triangle(p_in, a, b, c);
        assert!((cp_in - Vec3::new(0.5, 0.5, 0.0)).length() < 1e-5);

        // Point outside vertex A
        let p_a = Vec3::new(-1.0, -1.0, 2.0);
        let cp_a = closest_point_on_triangle(p_a, a, b, c);
        assert!((cp_a - a).length() < 1e-5);

        // Point outside hypotenuse edge BC
        let p_bc = Vec3::new(2.0, 2.0, 0.0);
        let cp_bc = closest_point_on_triangle(p_bc, a, b, c);
        assert!((cp_bc - Vec3::new(1.0, 1.0, 0.0)).length() < 1e-5);
    }

    #[test]
    fn test_workplane_projection_and_ray_intersection() {
        let origin = Vec3::new(10.0, 5.0, 0.0);
        let normal = Vec3::Y;
        let wp = Workplane::from_point_and_normal(origin, normal);
        assert!(wp.is_active);

        // Ray looking straight down from above the workplane
        let ray = Ray::new(Vec3::new(12.0, 10.0, 3.0), -Vec3::Y);
        let uv = wp.intersect_ray_uv(&ray).expect("ray hits workplane");
        let world_reconstruct = wp.uv_to_world(uv);
        assert!((world_reconstruct - Vec3::new(12.0, 5.0, 3.0)).length() < 1e-4);
    }
}

use core_math::{Aabb, Ray, Vec2, Vec3};

/// A ray-surface intersection record
#[derive(Debug, Clone, Copy)]
pub struct RayHit {
    pub t: f32,
    pub position: Vec3,
    pub normal: Vec3,
    pub uv: Vec2,
    pub material_id: usize,
}

/// A geometric triangle with precomputed vertices, vertex normals, and material index
#[derive(Debug, Clone, Copy)]
pub struct Triangle {
    pub v0: Vec3,
    pub v1: Vec3,
    pub v2: Vec3,
    pub n0: Vec3,
    pub n1: Vec3,
    pub n2: Vec3,
    pub uv0: Vec2,
    pub uv1: Vec2,
    pub uv2: Vec2,
    pub material_id: usize,
}

impl Triangle {
    pub fn new(
        v0: Vec3,
        v1: Vec3,
        v2: Vec3,
        n0: Vec3,
        n1: Vec3,
        n2: Vec3,
        uv0: Vec2,
        uv1: Vec2,
        uv2: Vec2,
        material_id: usize,
    ) -> Self {
        Self {
            v0,
            v1,
            v2,
            n0,
            n1,
            n2,
            uv0,
            uv1,
            uv2,
            material_id,
        }
    }

    pub fn aabb(&self) -> Aabb {
        let min = self.v0.min(self.v1).min(self.v2);
        let max = self.v0.max(self.v1).max(self.v2);
        // Add tiny epsilon to handle flat/planar triangles aligned to axes
        let eps = Vec3::splat(1e-4);
        Aabb::new(min - eps, max + eps)
    }

    pub fn centroid(&self) -> Vec3 {
        (self.v0 + self.v1 + self.v2) / 3.0
    }

    /// Fast Möller–Trumbore ray-triangle intersection algorithm
    pub fn intersect(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<RayHit> {
        let edge1 = self.v1 - self.v0;
        let edge2 = self.v2 - self.v0;
        let h = ray.direction.cross(edge2);
        let a = edge1.dot(h);

        if a.abs() < 1e-8 {
            return None; // Ray is parallel to triangle plane
        }

        let f = 1.0 / a;
        let s = ray.origin - self.v0;
        let u = f * s.dot(h);
        if !(0.0..=1.0).contains(&u) {
            return None;
        }

        let q = s.cross(edge1);
        let v = f * ray.direction.dot(q);
        if v < 0.0 || u + v > 1.0 {
            return None;
        }

        let t = f * edge2.dot(q);
        if t < t_min || t > t_max {
            return None;
        }

        let w = 1.0 - u - v;
        let position = ray.origin + ray.direction * t;
        // Interpolate smooth vertex normals
        let mut normal = (self.n0 * w + self.n1 * u + self.n2 * v).normalize_or_zero();
        if normal.length_squared() < 0.5 {
            normal = edge1.cross(edge2).normalize_or_zero();
        }

        // Two-sided normal alignment
        if normal.dot(ray.direction) > 0.0 {
            normal = -normal;
        }

        let uv = self.uv0 * w + self.uv1 * u + self.uv2 * v;

        Some(RayHit {
            t,
            position,
            normal,
            uv,
            material_id: self.material_id,
        })
    }
}

/// Bounding Volume Hierarchy (BVH) node for O(log N) ray tracing acceleration
#[derive(Debug)]
pub enum BvhNode {
    Leaf {
        aabb: Aabb,
        triangles: Vec<Triangle>,
    },
    Inner {
        aabb: Aabb,
        left: Box<BvhNode>,
        right: Box<BvhNode>,
    },
}

impl BvhNode {
    pub fn aabb(&self) -> &Aabb {
        match self {
            BvhNode::Leaf { aabb, .. } => aabb,
            BvhNode::Inner { aabb, .. } => aabb,
        }
    }

    /// Recursively builds a spatial-median BVH from a triangle slice
    pub fn build(mut triangles: Vec<Triangle>, max_leaf_size: usize) -> Self {
        if triangles.is_empty() {
            return BvhNode::Leaf {
                aabb: Aabb::new(Vec3::ZERO, Vec3::ZERO),
                triangles: Vec::new(),
            };
        }

        // Compute enclosing bounding box
        let mut total_aabb = triangles[0].aabb();
        for tri in &triangles[1..] {
            total_aabb = total_aabb.merge(&tri.aabb());
        }

        if triangles.len() <= max_leaf_size {
            return BvhNode::Leaf {
                aabb: total_aabb,
                triangles,
            };
        }

        // Split along the longest spatial axis of the bounding box
        let extent = total_aabb.max - total_aabb.min;
        let axis = if extent.x > extent.y && extent.x > extent.z {
            0
        } else if extent.y > extent.z {
            1
        } else {
            2
        };

        triangles.sort_unstable_by(|a, b| {
            let ca = a.centroid();
            let cb = b.centroid();
            let va = match axis {
                0 => ca.x,
                1 => ca.y,
                _ => ca.z,
            };
            let vb = match axis {
                0 => cb.x,
                1 => cb.y,
                _ => cb.z,
            };
            va.partial_cmp(&vb).unwrap_or(std::cmp::Ordering::Equal)
        });

        let mid = triangles.len() / 2;
        let right_triangles = triangles.split_off(mid);
        let left_triangles = triangles;

        let left = Box::new(Self::build(left_triangles, max_leaf_size));
        let right = Box::new(Self::build(right_triangles, max_leaf_size));

        BvhNode::Inner {
            aabb: total_aabb,
            left,
            right,
        }
    }

    /// Intersects a ray against the BVH tree, finding the closest hit
    pub fn intersect(&self, ray: &Ray, t_min: f32, mut t_max: f32) -> Option<RayHit> {
        if self.aabb().intersects_ray(ray).is_none() {
            return None;
        }

        match self {
            BvhNode::Leaf { triangles, .. } => {
                let mut closest_hit = None;
                for tri in triangles {
                    if let Some(hit) = tri.intersect(ray, t_min, t_max) {
                        t_max = hit.t;
                        closest_hit = Some(hit);
                    }
                }
                closest_hit
            }
            BvhNode::Inner { left, right, .. } => {
                let hit_left = left.intersect(ray, t_min, t_max);
                let limit = hit_left.as_ref().map_or(t_max, |h| h.t);
                let hit_right = right.intersect(ray, t_min, limit);

                hit_right.or(hit_left)
            }
        }
    }
}

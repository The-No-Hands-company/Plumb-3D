use crate::mesh::Mesh;
use core_math::{vec3, Aabb, Vec3};

/// A single 3D Gaussian Splat representing an anisotropic 3D ellipsoid with radiance and opacity (Kerbl et al.)
#[derive(Debug, Clone, Copy)]
pub struct GaussianSplat {
    pub position: Vec3,
    pub scale: Vec3,
    pub rotation: [f32; 4], // Quaternion (w, x, y, z)
    pub opacity: f32,       // Sigmoid activated opacity [0, 1]
    pub color: [f32; 3],     // Linear RGB radiance from spherical harmonics SH_0
}

impl Default for GaussianSplat {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            scale: vec3(0.05, 0.05, 0.05),
            rotation: [1.0, 0.0, 0.0, 0.0],
            opacity: 0.95,
            color: [0.8, 0.8, 0.8],
        }
    }
}

/// 3D Gaussian Splatting point cloud collection for real-time radiance rendering and scan reference
#[derive(Debug, Clone, Default)]
pub struct GaussianCloud {
    pub splats: Vec<GaussianSplat>,
}

impl GaussianCloud {
    pub fn new() -> Self {
        Self {
            splats: Vec::new(),
        }
    }

    /// Synthesizes a 3D Gaussian Splat cloud from an existing surface mesh (useful for scan simulation)
    pub fn from_mesh(mesh: &Mesh, default_splat_radius: f32) -> Self {
        let r = default_splat_radius.max(0.001);
        let mut splats = Vec::with_capacity(mesh.vertices.len());

        for v in mesh.vertices.values() {
            // Align ellipsoid z-axis with surface normal
            let n = v.normal;
            let q = quat_from_unit_vectors(vec3(0.0, 0.0, 1.0), n);

            splats.push(GaussianSplat {
                position: v.position,
                scale: vec3(r, r, r * 0.3), // Flattened along normal
                rotation: q,
                opacity: 0.9,
                color: [v.color[0], v.color[1], v.color[2]],
            });
        }

        Self { splats }
    }

    /// Sorts splats by depth relative to camera position and view direction (back-to-front alpha compositing)
    pub fn sort_by_depth(&mut self, camera_pos: Vec3, camera_dir: Vec3) {
        let dir = camera_dir.normalize();
        self.splats.sort_by(|a, b| {
            let depth_a = (a.position - camera_pos).dot(dir);
            let depth_b = (b.position - camera_pos).dot(dir);
            // Sort descending: farthest first for front-to-back or back-to-front compositing
            depth_b.partial_cmp(&depth_a).unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    /// Computes axis-aligned bounding box enclosing all Gaussian splat centroids
    pub fn compute_aabb(&self) -> Aabb {
        let mut aabb = Aabb::EMPTY;
        for s in &self.splats {
            aabb.extend_point(s.position - s.scale);
            aabb.extend_point(s.position + s.scale);
        }
        aabb
    }

    /// Evaluates 3D-to-2D Jacobian and covariance projection for splat index
    /// Returns 2D screen center (cx, cy) and conic coefficients (conic_x, conic_y, conic_z, opacity)
    pub fn evaluate_splat_projection(
        &self,
        splat_idx: usize,
        camera_pos: Vec3,
        view_dir: Vec3,
    ) -> Option<([f32; 2], [f32; 4])> {
        let splat = self.splats.get(splat_idx)?;
        let to_splat = splat.position - camera_pos;
        let depth = to_splat.dot(view_dir);
        if depth <= 0.05 {
            return None;
        }

        let inv_depth = 1.0 / depth;
        let screen_x = to_splat.x * inv_depth;
        let screen_y = to_splat.y * inv_depth;

        // Approximate 2D covariance conic from scale and distance
        let s_avg = (splat.scale.x + splat.scale.y) * 0.5 * inv_depth;
        let radius_sq = s_avg * s_avg + 0.3; // Screen anti-aliasing low-pass filter
        let conic_param = 1.0 / radius_sq.max(1e-4);

        Some((
            [screen_x, screen_y],
            [conic_param, 0.0, conic_param, splat.opacity],
        ))
    }

    /// Parses a standard 3DGS PLY header and records splat count
    pub fn parse_ply_header(header_text: &str) -> Option<usize> {
        for line in header_text.lines() {
            let tokens: Vec<&str> = line.split_whitespace().collect();
            if tokens.len() == 3 && tokens[0] == "element" && tokens[1] == "vertex" {
                return tokens[2].parse::<usize>().ok();
            }
        }
        None
    }
}

/// Computes rotation quaternion (w, x, y, z) that rotates vector u to vector v
fn quat_from_unit_vectors(u: Vec3, v: Vec3) -> [f32; 4] {
    let dot = u.dot(v);
    if dot >= 0.9999 {
        return [1.0, 0.0, 0.0, 0.0];
    }
    if dot <= -0.9999 {
        let axis = if u.x.abs() < 0.9 {
            u.cross(vec3(1.0, 0.0, 0.0)).normalize()
        } else {
            u.cross(vec3(0.0, 1.0, 0.0)).normalize()
        };
        return [0.0, axis.x, axis.y, axis.z];
    }

    let cross = u.cross(v);
    let s = (2.0 * (1.0 + dot)).sqrt();
    let inv_s = 1.0 / s;
    [0.5 * s, cross.x * inv_s, cross.y * inv_s, cross.z * inv_s]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::Primitives;

    #[test]
    fn test_gaussian_cloud_from_mesh() {
        let cube = Primitives::create_cube(2.0);
        let cloud = GaussianCloud::from_mesh(&cube, 0.1);

        assert_eq!(cloud.splats.len(), cube.vertices.len());
        let aabb = cloud.compute_aabb();
        assert!(aabb.max.x > 0.9);
        assert!(aabb.min.x < -0.9);
    }

    #[test]
    fn test_gaussian_cloud_depth_sort() {
        let mut cloud = GaussianCloud::new();
        cloud.splats.push(GaussianSplat {
            position: vec3(0.0, 0.0, 5.0),
            ..Default::default()
        });
        cloud.splats.push(GaussianSplat {
            position: vec3(0.0, 0.0, 1.0),
            ..Default::default()
        });
        cloud.splats.push(GaussianSplat {
            position: vec3(0.0, 0.0, 10.0),
            ..Default::default()
        });

        cloud.sort_by_depth(vec3(0.0, 0.0, 0.0), vec3(0.0, 0.0, 1.0));

        // First splat should be the farthest (z = 10.0)
        assert_eq!(cloud.splats[0].position.z, 10.0);
        assert_eq!(cloud.splats[1].position.z, 5.0);
        assert_eq!(cloud.splats[2].position.z, 1.0);
    }

    #[test]
    fn test_gaussian_splat_projection() {
        let mut cloud = GaussianCloud::new();
        cloud.splats.push(GaussianSplat {
            position: vec3(0.0, 0.0, 4.0),
            scale: vec3(0.2, 0.2, 0.2),
            ..Default::default()
        });

        let proj = cloud.evaluate_splat_projection(
            0,
            vec3(0.0, 0.0, 0.0),
            vec3(0.0, 0.0, 1.0),
        );

        assert!(proj.is_some());
        let (screen_pos, conic) = proj.unwrap();
        assert_eq!(screen_pos, [0.0, 0.0]);
        assert!(conic[0] > 0.0);
    }
}

use crate::mesh::{Mesh, MeshError};
use crate::modifiers::stack::Modifier;
use core_math::Vec3;

fn bernstein(n: usize, i: usize, t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    let binom = match (n, i) {
        (1, 0) | (1, 1) => 1.0,
        (2, 0) | (2, 2) => 1.0,
        (2, 1) => 2.0,
        (3, 0) | (3, 3) => 1.0,
        (3, 1) | (3, 2) => 3.0,
        (4, 0) | (4, 4) => 1.0,
        (4, 1) | (4, 3) => 4.0,
        (4, 2) => 6.0,
        _ => {
            let mut c = 1.0;
            for k in 0..i {
                c = c * (n - k) as f32 / (k + 1) as f32;
            }
            c
        }
    };
    binom * t.powi(i as i32) * (1.0 - t).powi((n - i) as i32)
}

/// Sederberg-Parry Free-Form Deformation (FFD) / Lattice Deformer (Maya Lattice / 3ds Max FFD / Blender Lattice)
/// Evaluates a 3D trivariate tensor-product Bernstein polynomial basis:
/// X(s, t, u) = \sum_{i=0}^l \sum_{j=0}^m \sum_{k=0}^n B_i^l(s) B_j^m(t) B_k^n(u) P_{ijk}
#[derive(Debug, Clone)]
pub struct LatticeModifier {
    pub resolution: (usize, usize, usize),
    pub bbox_min: Vec3,
    pub bbox_max: Vec3,
    pub control_points: Vec<Vec3>,
    pub vertex_group: Option<String>,
    pub enabled: bool,
}

impl LatticeModifier {
    pub fn new(resolution: (usize, usize, usize), bbox_min: Vec3, bbox_max: Vec3) -> Self {
        let (rx, ry, rz) = (
            resolution.0.max(2),
            resolution.1.max(2),
            resolution.2.max(2),
        );
        let mut control_points = Vec::with_capacity(rx * ry * rz);
        let extent = bbox_max - bbox_min;

        for i in 0..rx {
            let s = i as f32 / (rx - 1) as f32;
            for j in 0..ry {
                let t = j as f32 / (ry - 1) as f32;
                for k in 0..rz {
                    let u = k as f32 / (rz - 1) as f32;
                    let pt = bbox_min + Vec3::new(s * extent.x, t * extent.y, u * extent.z);
                    control_points.push(pt);
                }
            }
        }

        Self {
            resolution: (rx, ry, rz),
            bbox_min,
            bbox_max,
            control_points,
            vertex_group: None,
            enabled: true,
        }
    }

    pub fn from_mesh_bounds(mesh: &Mesh, resolution: (usize, usize, usize)) -> Self {
        let mut min = Vec3::splat(f32::INFINITY);
        let mut max = Vec3::splat(f32::NEG_INFINITY);
        for (_, v) in &mesh.vertices {
            min = min.min(v.position);
            max = max.max(v.position);
        }
        if mesh.vertices.is_empty() {
            min = Vec3::splat(-1.0);
            max = Vec3::splat(1.0);
        }
        let margin = (max - min) * 0.05 + Vec3::splat(0.01);
        Self::new(resolution, min - margin, max + margin)
    }

    pub fn displace_control_point(&mut self, i: usize, j: usize, k: usize, delta: Vec3) {
        let (rx, ry, rz) = self.resolution;
        if i < rx && j < ry && k < rz {
            let idx = i * (ry * rz) + j * rz + k;
            self.control_points[idx] += delta;
        }
    }

    pub fn squash_and_stretch(&mut self, factor: f32) {
        let f_y = factor.max(0.1);
        let f_xz = 1.0 / (f_y.sqrt());
        let center = (self.bbox_min + self.bbox_max) * 0.5;

        for pt in &mut self.control_points {
            let offset = *pt - center;
            pt.x = center.x + offset.x * f_xz;
            pt.y = center.y + offset.y * f_y;
            pt.z = center.z + offset.z * f_xz;
        }
    }
}

impl Modifier for LatticeModifier {
    fn name(&self) -> &str {
        "Lattice (FFD)"
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn apply(&self, input: &Mesh) -> Result<Mesh, MeshError> {
        let mut output = input.clone();
        let (rx, ry, rz) = self.resolution;
        let (deg_x, deg_y, deg_z) = (
            rx.saturating_sub(1),
            ry.saturating_sub(1),
            rz.saturating_sub(1),
        );
        let extent = self.bbox_max - self.bbox_min;
        let inv_extent = Vec3::new(
            if extent.x.abs() > 1e-5 {
                1.0 / extent.x
            } else {
                1.0
            },
            if extent.y.abs() > 1e-5 {
                1.0 / extent.y
            } else {
                1.0
            },
            if extent.z.abs() > 1e-5 {
                1.0 / extent.z
            } else {
                1.0
            },
        );

        let vg = self
            .vertex_group
            .as_ref()
            .and_then(|name| input.get_vertex_group(name));

        for (vk, v) in &mut output.vertices {
            let s = ((v.position.x - self.bbox_min.x) * inv_extent.x).clamp(0.0, 1.0);
            let t = ((v.position.y - self.bbox_min.y) * inv_extent.y).clamp(0.0, 1.0);
            let u = ((v.position.z - self.bbox_min.z) * inv_extent.z).clamp(0.0, 1.0);

            let mut deformed = Vec3::ZERO;
            for i in 0..rx {
                let bx = bernstein(deg_x, i, s);
                for j in 0..ry {
                    let by = bernstein(deg_y, j, t);
                    let bxy = bx * by;
                    for k in 0..rz {
                        let bz = bernstein(deg_z, k, u);
                        let w = bxy * bz;
                        let idx = i * (ry * rz) + j * rz + k;
                        deformed += self.control_points[idx] * w;
                    }
                }
            }

            let mask = vg.map(|g| g.get_weight(vk)).unwrap_or(1.0);
            v.position = v.position.lerp(deformed, mask);
        }

        output.update_normals();
        Ok(output)
    }
}

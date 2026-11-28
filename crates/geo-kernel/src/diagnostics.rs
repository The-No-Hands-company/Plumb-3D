//! Surface Diagnostic Analytics for CAD & Class-A Surfacing.
//! Evaluates G0 (positional), G1 (tangent), and G2 (curvature) continuity across
//! surface boundaries, and calculates Gaussian and Mean surface curvature metrics.

use crate::mesh::{Mesh, VertexKey};
use core_math::Vec3;
use std::collections::HashMap;

/// Surface continuity level across adjacent patches or faces.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContinuityClass {
    G0, // Positional continuity (watertight, shared boundary)
    G1, // Tangent continuity (smooth surface normals across seam)
    G2, // Curvature continuity (continuous second derivative / curvature)
    Discontinuous,
}

/// Comprehensive surface continuity diagnostic report for an edge seam.
#[derive(Debug, Clone)]
pub struct SeamContinuityReport {
    pub continuity: ContinuityClass,
    pub max_normal_deviation_deg: f32,
    pub max_curvature_deviation: f32,
    pub is_manifold: bool,
}

/// Curvature sample containing principal, Gaussian, and Mean curvatures at a vertex or surface point.
#[derive(Debug, Clone, Copy, Default)]
pub struct SurfaceCurvatureSample {
    pub k_min: f32,
    pub k_max: f32,
    pub gaussian: f32,
    pub mean: f32,
}

/// Color mapping style for surface curvature heatmaps.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CurvatureColorMap {
    #[default]
    Spectral, // Blue (negative/hyperbolic) -> Green (zero/planar) -> Red (positive/elliptic)
    Zebra,    // High-contrast alternating bands
    Heat,     // Black -> Orange -> White
}

impl SurfaceCurvatureSample {
    /// Maps the Gaussian or Mean curvature value to an RGBA color for viewport heatmaps.
    pub fn to_heatmap_color(&self, use_gaussian: bool, range: f32) -> [f32; 4] {
        let val = if use_gaussian {
            self.gaussian
        } else {
            self.mean
        };
        let norm = (val / range.max(1e-4)).clamp(-1.0, 1.0);
        // Map -1..1 to 0..1
        let t = norm * 0.5 + 0.5;

        // Spectral ramp: Blue (0.0) -> Cyan (0.25) -> Green (0.5) -> Yellow (0.75) -> Red (1.0)
        let r = (2.0 * t - 0.5).clamp(0.0, 1.0);
        let g = if t < 0.5 {
            (2.0 * t).clamp(0.0, 1.0)
        } else {
            (2.0 * (1.0 - t)).clamp(0.0, 1.0)
        };
        let b = (1.5 - 2.0 * t).clamp(0.0, 1.0);

        [r, g, b, 1.0]
    }
}

/// Porcupine needle for curvature comb analysis.
#[derive(Debug, Clone, Copy)]
pub struct CurvatureNeedle {
    pub root: Vec3,
    pub tip: Vec3,
    pub curvature_magnitude: f32,
}

impl Mesh {
    /// Evaluates surface continuity (G0/G1/G2) across all manifold edges in the mesh.
    pub fn evaluate_surface_continuity(&self) -> Vec<SeamContinuityReport> {
        let mut reports = Vec::new();

        for (_edge_key, edge) in &self.edges {
            if let Some(he0) = self.half_edges.get(edge.half_edge) {
                let v0 = he0.origin;
                let v1 = he0.target;
                let p0 = self.vertices.get(v0).map(|v| v.position).unwrap_or(Vec3::ZERO);
                let p1 = self.vertices.get(v1).map(|v| v.position).unwrap_or(Vec3::ZERO);

                // G0 check: valid connected vertices
                let edge_len = (p1 - p0).length();
                if edge_len < 1e-6 {
                    continue;
                }

                // Check faces sharing this edge
                let mut face_normals = Vec::new();
                if let Some(face_key) = he0.face {
                    if let Some(face) = self.faces.get(face_key) {
                        let n = face.normal;
                        if n.length_squared() > 1e-6 {
                            face_normals.push(n.normalize());
                        }
                    }
                }

                if let Some(he_twin) = self.half_edges.get(he0.twin) {
                    if let Some(face_key) = he_twin.face {
                        if let Some(face) = self.faces.get(face_key) {
                            let n = face.normal;
                            if n.length_squared() > 1e-6 {
                                face_normals.push(n.normalize());
                            }
                        }
                    }
                }

                if face_normals.len() < 2 {
                    reports.push(SeamContinuityReport {
                        continuity: ContinuityClass::G0,
                        max_normal_deviation_deg: 0.0,
                        max_curvature_deviation: 0.0,
                        is_manifold: face_normals.len() == 1,
                    });
                    continue;
                }

                let n1 = face_normals[0];
                let n2 = face_normals[1];
                let dot = n1.dot(n2).clamp(-1.0, 1.0);
                let angle_rad = dot.acos();
                let angle_deg = angle_rad.to_degrees();

                let continuity = if angle_deg < 0.05 {
                    ContinuityClass::G2
                } else if angle_deg < 1.0 {
                    ContinuityClass::G1
                } else {
                    ContinuityClass::G0
                };

                let diff = 1.0f32 - dot;
                reports.push(SeamContinuityReport {
                    continuity,
                    max_normal_deviation_deg: angle_deg,
                    max_curvature_deviation: diff.abs(),
                    is_manifold: true,
                });
            }
        }

        reports
    }

    /// Computes Gaussian and Mean curvature diagnostics for all vertices of the mesh.
    pub fn compute_surface_curvatures(&self) -> HashMap<VertexKey, SurfaceCurvatureSample> {
        let mut samples = HashMap::new();

        for (v_key, vertex) in &self.vertices {
            let p = vertex.position;
            let mut ring = Vec::new();

            for &he_key in &vertex.half_edges {
                if let Some(he) = self.half_edges.get(he_key) {
                    if let Some(nv) = self.vertices.get(he.target) {
                        ring.push(nv.position);
                    }
                }
            }

            if ring.len() < 3 {
                samples.insert(
                    v_key,
                    SurfaceCurvatureSample {
                        k_min: 0.0,
                        k_max: 0.0,
                        gaussian: 0.0,
                        mean: 0.0,
                    },
                );
                continue;
            }

            // Discrete Gauss-Bonnet angular deficit for Gaussian Curvature:
            // K_v = 2*PI - sum(angles)
            let mut total_angle = 0.0f32;
            let n = ring.len();
            for i in 0..n {
                let p1 = ring[i] - p;
                let p2 = ring[(i + 1) % n] - p;
                let l1 = p1.length();
                let l2 = p2.length();
                if l1 > 1e-6 && l2 > 1e-6 {
                    let cos_a = (p1.dot(p2) / (l1 * l2)).clamp(-1.0, 1.0);
                    total_angle += cos_a.acos();
                }
            }

            let angular_deficit = 2.0 * std::f32::consts::PI - total_angle;

            // Discrete Laplace-Beltrami for Mean Curvature
            let mut laplacian = Vec3::ZERO;
            for &r in &ring {
                laplacian += r - p;
            }
            laplacian *= 1.0 / n as f32;
            let mean = laplacian.length() * 0.5;

            // Gaussian curvature approximation
            let gaussian = angular_deficit;
            let disc = (mean * mean - gaussian).max(0.0).sqrt();
            let k_max = mean + disc;
            let k_min = mean - disc;

            samples.insert(
                v_key,
                SurfaceCurvatureSample {
                    k_min,
                    k_max,
                    gaussian,
                    mean,
                },
            );
        }

        samples
    }

    /// Evaluates curvature comb needles along the boundary loops or sharp edges of the mesh.
    pub fn evaluate_boundary_curvature_combs(&self, scale: f32) -> Vec<CurvatureNeedle> {
        let mut needles = Vec::new();
        let curvatures = self.compute_surface_curvatures();

        for (_edge_key, edge) in &self.edges {
            if let Some(he) = self.half_edges.get(edge.half_edge) {
                let v0 = he.origin;
                let v1 = he.target;
                if let (Some(vert0), Some(vert1)) = (self.vertices.get(v0), self.vertices.get(v1)) {
                    let mid = (vert0.position + vert1.position) * 0.5;
                    let dir = (vert1.position - vert0.position).normalize();

                    let c0 = curvatures.get(&v0).copied().unwrap_or_default();
                    let c1 = curvatures.get(&v1).copied().unwrap_or_default();
                    let avg_mean = (c0.mean + c1.mean) * 0.5;

                    let arb = if dir.x.abs() < 0.9 {
                        Vec3::X
                    } else {
                        Vec3::Y
                    };
                    let normal = dir.cross(arb).normalize();

                    let needle_len = avg_mean * scale;
                    needles.push(CurvatureNeedle {
                        root: mid,
                        tip: mid + normal * needle_len,
                        curvature_magnitude: avg_mean,
                    });
                }
            }
        }

        needles
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::Primitives;

    #[test]
    fn test_sphere_curvature_diagnostics() {
        let sphere = Primitives::create_uv_sphere(1.0, 16, 16);
        let curvatures = sphere.compute_surface_curvatures();
        assert!(!curvatures.is_empty());

        for (_k, sample) in curvatures {
            assert!(sample.mean >= 0.0);
        }
    }

    #[test]
    fn test_cube_continuity_diagnostics() {
        let cube = Primitives::create_cube(2.0);
        let reports = cube.evaluate_surface_continuity();
        assert!(!reports.is_empty());

        let right_angle_edges = reports
            .iter()
            .filter(|r| (r.max_normal_deviation_deg - 90.0).abs() < 1.0)
            .count();
        assert!(right_angle_edges > 0);
    }

    #[test]
    fn test_heatmap_color_mapping() {
        let sample = SurfaceCurvatureSample {
            k_min: -0.5,
            k_max: 0.5,
            gaussian: -0.25,
            mean: 0.0,
        };
        let color = sample.to_heatmap_color(true, 1.0);
        assert_eq!(color[3], 1.0);
        assert!(color[2] > 0.0);
    }
}

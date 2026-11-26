use crate::mesh::{FaceKey, Mesh};
use core_math::{Vec2, Vec3};
use std::collections::HashMap;

/// Result of UV distortion analysis across a mesh.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UvDistortionSummary {
    /// Mean angular difference between 3D surfaces and 2D flattened UVs in degrees.
    pub mean_angle_error_deg: f32,
    /// Mean area stretch discrepancy ratio (|UV_ratio - 3D_ratio|).
    pub mean_area_stretch: f32,
    /// Maximum corner angle error in degrees.
    pub max_angle_error_deg: f32,
}

impl Mesh {
    /// Computes per-face UV conformal angle and area stretch distortion scalar in `[0.0, 1.0]`.
    ///
    /// 0.0 represents zero distortion (isometric / exact conformal match).
    /// 1.0 represents severe angle shearing or extreme area compression/expansion.
    pub fn compute_uv_distortion(&self) -> HashMap<FaceKey, f32> {
        let mut face_metrics = HashMap::new();
        if self.faces.is_empty() {
            return face_metrics;
        }

        // 1. Calculate total 3D area and total 2D UV area
        let mut total_3d_area = 0.0_f32;
        let mut total_2d_area = 0.0_f32;
        let mut face_areas = Vec::with_capacity(self.faces.len());

        for (fk, _) in &self.faces {
            let verts = self.face_vertices(fk);
            if verts.len() < 3 {
                continue;
            }

            let p0 = self.vertices[verts[0]].position;
            let p1 = self.vertices[verts[1]].position;
            let p2 = self.vertices[verts[2]].position;
            let a3d = (p1 - p0).cross(p2 - p0).length() * 0.5;

            let uv0 = self.vertices[verts[0]].uv;
            let uv1 = self.vertices[verts[1]].uv;
            let uv2 = self.vertices[verts[2]].uv;
            let a2d = ((uv1.x - uv0.x) * (uv2.y - uv0.y) - (uv2.x - uv0.x) * (uv1.y - uv0.y)).abs() * 0.5;

            total_3d_area += a3d;
            total_2d_area += a2d;
            face_areas.push((fk, a3d, a2d, verts));
        }

        let total_3d_safe = total_3d_area.max(1e-6);
        let total_2d_safe = total_2d_area.max(1e-6);

        // 2. Compute per-face angle distortion and area distortion
        for (fk, a3d, a2d, verts) in face_areas {
            let p0 = self.vertices[verts[0]].position;
            let p1 = self.vertices[verts[1]].position;
            let p2 = self.vertices[verts[2]].position;

            let uv0 = self.vertices[verts[0]].uv;
            let uv1 = self.vertices[verts[1]].uv;
            let uv2 = self.vertices[verts[2]].uv;

            // 3D angles
            let a3_0 = corner_angle_3d(p2 - p0, p1 - p0);
            let a3_1 = corner_angle_3d(p0 - p1, p2 - p1);
            let a3_2 = corner_angle_3d(p1 - p2, p0 - p2);

            // 2D UV angles
            let u2_0 = corner_angle_2d(uv2 - uv0, uv1 - uv0);
            let u2_1 = corner_angle_2d(uv0 - uv1, uv2 - uv1);
            let u2_2 = corner_angle_2d(uv1 - uv2, uv0 - uv2);

            let angle_err0 = (u2_0 - a3_0).abs() / a3_0.max(1e-4);
            let angle_err1 = (u2_1 - a3_1).abs() / a3_1.max(1e-4);
            let angle_err2 = (u2_2 - a3_2).abs() / a3_2.max(1e-4);
            let mean_angle_err = (angle_err0 + angle_err1 + angle_err2) / 3.0;

            // Area stretch ratio error
            let norm_3d = a3d / total_3d_safe;
            let norm_2d = a2d / total_2d_safe;
            let area_ratio = if norm_3d > 1e-7 { norm_2d / norm_3d } else { 1.0 };
            let area_err = (area_ratio - 1.0).abs().min(2.0) * 0.5;

            // Weighted combination
            let distortion = (mean_angle_err * 0.7 + area_err * 0.3).clamp(0.0, 1.0);
            face_metrics.insert(fk, distortion);
        }

        face_metrics
    }

    /// Computes per-face RGBA heatmap colors based on UV distortion metrics.
    ///
    /// Blue indicates optimal mapping; Green/Yellow mild distortion; Red severe distortion.
    pub fn compute_uv_distortion_rgba(&self) -> HashMap<FaceKey, [u8; 4]> {
        let distortions = self.compute_uv_distortion();
        let mut colors = HashMap::with_capacity(distortions.len());

        for (fk, d) in distortions {
            let rgba = distortion_to_heatmap_rgba(d);
            colors.insert(fk, rgba);
        }

        colors
    }

    /// Computes overall summary statistics for the UV layout.
    pub fn summarize_uv_distortion(&self) -> UvDistortionSummary {
        let mut total_angle_err_deg = 0.0_f32;
        let mut total_area_stretch = 0.0_f32;
        let mut max_angle_err_deg = 0.0_f32;
        let mut face_count = 0.0_f32;

        let mut total_3d_area = 0.0_f32;
        let mut total_2d_area = 0.0_f32;

        for (fk, _) in &self.faces {
            let verts = self.face_vertices(fk);
            if verts.len() < 3 {
                continue;
            }
            let p0 = self.vertices[verts[0]].position;
            let p1 = self.vertices[verts[1]].position;
            let p2 = self.vertices[verts[2]].position;
            total_3d_area += (p1 - p0).cross(p2 - p0).length() * 0.5;

            let uv0 = self.vertices[verts[0]].uv;
            let uv1 = self.vertices[verts[1]].uv;
            let uv2 = self.vertices[verts[2]].uv;
            total_2d_area += ((uv1.x - uv0.x) * (uv2.y - uv0.y) - (uv2.x - uv0.x) * (uv1.y - uv0.y)).abs() * 0.5;
        }

        let total_3d_safe = total_3d_area.max(1e-6);
        let total_2d_safe = total_2d_area.max(1e-6);

        for (fk, _) in &self.faces {
            let verts = self.face_vertices(fk);
            if verts.len() < 3 {
                continue;
            }

            let p0 = self.vertices[verts[0]].position;
            let p1 = self.vertices[verts[1]].position;
            let p2 = self.vertices[verts[2]].position;
            let a3d = (p1 - p0).cross(p2 - p0).length() * 0.5;

            let uv0 = self.vertices[verts[0]].uv;
            let uv1 = self.vertices[verts[1]].uv;
            let uv2 = self.vertices[verts[2]].uv;
            let a2d = ((uv1.x - uv0.x) * (uv2.y - uv0.y) - (uv2.x - uv0.x) * (uv1.y - uv0.y)).abs() * 0.5;

            let a3_0 = corner_angle_3d(p2 - p0, p1 - p0);
            let a3_1 = corner_angle_3d(p0 - p1, p2 - p1);
            let a3_2 = corner_angle_3d(p1 - p2, p0 - p2);

            let u2_0 = corner_angle_2d(uv2 - uv0, uv1 - uv0);
            let u2_1 = corner_angle_2d(uv0 - uv1, uv2 - uv1);
            let u2_2 = corner_angle_2d(uv1 - uv2, uv0 - uv2);

            let e0 = (u2_0 - a3_0).abs().to_degrees();
            let e1 = (u2_1 - a3_1).abs().to_degrees();
            let e2 = (u2_2 - a3_2).abs().to_degrees();

            let max_e = e0.max(e1).max(e2);
            if max_e > max_angle_err_deg {
                max_angle_err_deg = max_e;
            }
            total_angle_err_deg += (e0 + e1 + e2) / 3.0;

            let norm_3d = a3d / total_3d_safe;
            let norm_2d = a2d / total_2d_safe;
            let area_ratio = if norm_3d > 1e-7 { norm_2d / norm_3d } else { 1.0 };
            total_area_stretch += (area_ratio - 1.0).abs();

            face_count += 1.0;
        }

        let mean_angle = if face_count > 0.0 { total_angle_err_deg / face_count } else { 0.0 };
        let mean_area = if face_count > 0.0 { total_area_stretch / face_count } else { 0.0 };

        UvDistortionSummary {
            mean_angle_error_deg: mean_angle,
            mean_area_stretch: mean_area,
            max_angle_error_deg: max_angle_err_deg,
        }
    }
}

fn corner_angle_3d(v0: Vec3, v1: Vec3) -> f32 {
    let len0 = v0.length();
    let len1 = v1.length();
    if len0 < 1e-6 || len1 < 1e-6 {
        return std::f32::consts::FRAC_PI_2;
    }
    let cos_t = (v0.dot(v1) / (len0 * len1)).clamp(-1.0, 1.0);
    cos_t.acos()
}

fn corner_angle_2d(v0: Vec2, v1: Vec2) -> f32 {
    let len0 = v0.length();
    let len1 = v1.length();
    if len0 < 1e-6 || len1 < 1e-6 {
        return std::f32::consts::FRAC_PI_2;
    }
    let cos_t = (v0.dot(v1) / (len0 * len1)).clamp(-1.0, 1.0);
    cos_t.acos()
}

/// Converts a distortion scalar in `[0.0, 1.0]` to a translucent heatmap RGBA color.
pub fn distortion_to_heatmap_rgba(d: f32) -> [u8; 4] {
    let t = d.clamp(0.0, 1.0);
    let (r, g, b) = if t < 0.25 {
        // Deep blue to cyan
        let s = t / 0.25;
        (30 + (10.0 * s) as u8, 80 + (120.0 * s) as u8, 220)
    } else if t < 0.5 {
        // Cyan to emerald green
        let s = (t - 0.25) / 0.25;
        (40 + (10.0 * s) as u8, 200 + (20.0 * s) as u8, 220 - (130.0 * s) as u8)
    } else if t < 0.75 {
        // Emerald green to amber yellow
        let s = (t - 0.50) / 0.25;
        (50 + (190.0 * s) as u8, 220 - (30.0 * s) as u8, 90 - (50.0 * s) as u8)
    } else {
        // Amber yellow to bright crimson
        let s = (t - 0.75) / 0.25;
        (240, 190 - (150.0 * s) as u8, 40)
    };

    [r, g, b, 140]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::Primitives;
    use crate::uv::ProjectionAxis;

    #[test]
    fn test_uv_distortion_planar_projection() {
        let mut plane = Primitives::create_plane(2.0, 2.0, 1, 1);
        plane.generate_planar_uv(ProjectionAxis::Y);

        let distortions = plane.compute_uv_distortion();
        assert!(!distortions.is_empty());
        for &d in distortions.values() {
            // Planar projection of a flat plane has nearly 0 distortion
            assert!(d < 0.15, "Expected low distortion on flat plane, got {}", d);
        }

        let summary = plane.summarize_uv_distortion();
        assert!(summary.mean_angle_error_deg < 5.0);
    }

    #[test]
    fn test_uv_distortion_heatmap_colors() {
        let c_blue = distortion_to_heatmap_rgba(0.0);
        assert_eq!(c_blue[2], 220); // High blue component
        assert_eq!(c_blue[3], 140);

        let c_red = distortion_to_heatmap_rgba(1.0);
        assert_eq!(c_red[0], 240); // High red component
        assert_eq!(c_red[1], 40);
    }
}

//! Multi-Rail Curve Loft & Sweep Engine (Plasticity / Rhino Style).
//! Generates smooth B-rep surfaces guided by N >= 2 longitudinal rails
//! with G1 (tangent) and G2 (curvature continuous) transverse blending.

use crate::mesh::{Mesh, MeshError, VertexKey};
use crate::operators::variable_bevel::FilletContinuity;
use crate::spline::types::Spline;
use core_math::{vec2, Vec3};

/// Resamples a 3D polyline along arc length to exactly `target_count` points.
pub fn resample_polyline_3d(points: &[Vec3], target_count: usize) -> Vec<Vec3> {
    if points.is_empty() || target_count == 0 {
        return Vec::new();
    }
    if points.len() == 1 || target_count == 1 {
        return vec![points[0]; target_count];
    }

    let mut cumulative_dist = Vec::with_capacity(points.len());
    cumulative_dist.push(0.0);
    let mut total_length = 0.0;
    for i in 1..points.len() {
        total_length += (points[i] - points[i - 1]).length();
        cumulative_dist.push(total_length);
    }

    if total_length < 1e-6 {
        return vec![points[0]; target_count];
    }

    let mut resampled = Vec::with_capacity(target_count);
    for k in 0..target_count {
        let target_d = (k as f32 / (target_count - 1).max(1) as f32) * total_length;

        // Binary search / scan for segment
        let mut idx = 0;
        while idx + 1 < cumulative_dist.len() && cumulative_dist[idx + 1] < target_d {
            idx += 1;
        }

        if idx + 1 >= points.len() {
            resampled.push(points[points.len() - 1]);
        } else {
            let seg_len = cumulative_dist[idx + 1] - cumulative_dist[idx];
            let t = if seg_len > 1e-6 {
                (target_d - cumulative_dist[idx]) / seg_len
            } else {
                0.0
            };
            resampled.push(points[idx].lerp(points[idx + 1], t));
        }
    }

    resampled
}

/// Evaluates a smooth transverse spline across N guide rail points at parameter u in [0, 1].
pub fn interpolate_rail_station(rail_pts: &[Vec3], u: f32, continuity: FilletContinuity) -> Vec3 {
    let n = rail_pts.len();
    if n == 0 {
        return Vec3::ZERO;
    }
    if n == 1 {
        return rail_pts[0];
    }
    let u_clamped = u.clamp(0.0, 1.0);

    // Map u to segment index and local t
    let seg_float = u_clamped * (n - 1) as f32;
    let seg_idx = (seg_float.floor() as usize).min(n - 2);
    let raw_t = seg_float - seg_idx as f32;

    match continuity {
        FilletContinuity::Linear => rail_pts[seg_idx].lerp(rail_pts[seg_idx + 1], raw_t),
        FilletContinuity::G1Tangent => {
            // Hermite cubic interpolation with central difference tangents
            let p0 = rail_pts[seg_idx];
            let p1 = rail_pts[seg_idx + 1];

            // Tangent at p0
            let m0 = if seg_idx > 0 {
                (p1 - rail_pts[seg_idx - 1]) * 0.5
            } else {
                p1 - p0
            };

            // Tangent at p1
            let m1 = if seg_idx + 2 < n {
                (rail_pts[seg_idx + 2] - p0) * 0.5
            } else {
                p1 - p0
            };

            let h00 = 2.0 * raw_t * raw_t * raw_t - 3.0 * raw_t * raw_t + 1.0;
            let h10 = raw_t * raw_t * raw_t - 2.0 * raw_t * raw_t + raw_t;
            let h01 = -2.0 * raw_t * raw_t * raw_t + 3.0 * raw_t * raw_t;
            let h11 = raw_t * raw_t * raw_t - raw_t * raw_t;

            p0 * h00 + m0 * h10 + p1 * h01 + m1 * h11
        }
        FilletContinuity::G2Curvature => {
            // Quintic C2 continuous blending across rails
            let p0 = rail_pts[seg_idx];
            let p1 = rail_pts[seg_idx + 1];

            let m0 = if seg_idx > 0 {
                (p1 - rail_pts[seg_idx - 1]) * 0.5
            } else {
                p1 - p0
            };
            let m1 = if seg_idx + 2 < n {
                (rail_pts[seg_idx + 2] - p0) * 0.5
            } else {
                p1 - p0
            };

            // Quintic Hermite interpolation
            let t2 = raw_t * raw_t;
            let t3 = t2 * raw_t;
            let t4 = t3 * raw_t;
            let t5 = t4 * raw_t;

            let h0 = -6.0 * t5 + 15.0 * t4 - 10.0 * t3 + 1.0;
            let h1 = -3.0 * t5 + 8.0 * t4 - 6.0 * t3 + raw_t;
            let h2 = 6.0 * t5 - 15.0 * t4 + 10.0 * t3;
            let h3 = -3.0 * t5 + 7.0 * t4 - 4.0 * t3;

            p0 * h0 + m0 * h1 + p1 * h2 + m1 * h3
        }
    }
}

impl Spline {
    /// Multi-Rail Curve Loft & Sweep (Plasticity / Rhino CAD style).
    /// Sweeps across N >= 2 longitudinal guide rails.
    ///
    /// - `rails`: Slice of 3D guide curves (minimum 2 rails).
    /// - `samples_u`: Number of transverse subdivision samples across the rails (>= 2).
    /// - `samples_v`: Number of longitudinal station slices along the rails (>= 2).
    /// - `continuity`: G0 (Linear), G1 (Tangent continuous), or G2 (Curvature continuous).
    /// - `is_closed_v`: Whether the loft loops back from the end to the start station.
    pub fn multi_rail_loft(
        rails: &[Vec<Vec3>],
        samples_u: usize,
        samples_v: usize,
        continuity: FilletContinuity,
        is_closed_v: bool,
    ) -> Result<Mesh, MeshError> {
        if rails.len() < 2 {
            return Err(MeshError::InvariantFailed(
                "Multi-rail loft requires at least 2 guide rails".into(),
            ));
        }
        for (i, r) in rails.iter().enumerate() {
            if r.len() < 2 {
                return Err(MeshError::InvariantFailed(format!(
                    "Guide rail {} must have at least 2 points",
                    i
                )));
            }
        }

        let num_rails = rails.len();
        let u_res = samples_u.max(2);
        let v_res = samples_v.max(2);

        // 1. Resample each rail to uniform v_res station points
        let mut resampled_rails: Vec<Vec<Vec3>> = Vec::with_capacity(num_rails);
        for rail in rails {
            resampled_rails.push(resample_polyline_3d(rail, v_res));
        }

        let mut mesh = Mesh::new();
        let mut grid: Vec<Vec<VertexKey>> = Vec::with_capacity(v_res);

        // 2. Generate vertex grid [station_v][sample_u]
        for v_idx in 0..v_res {
            let v_coord = v_idx as f32 / (v_res - 1).max(1) as f32;

            // Collect rail coordinates at station v_idx
            let station_pts: Vec<Vec3> = (0..num_rails)
                .map(|r_idx| resampled_rails[r_idx][v_idx])
                .collect();

            let mut row = Vec::with_capacity(u_res);
            for u_idx in 0..u_res {
                let u_coord = u_idx as f32 / (u_res - 1).max(1) as f32;
                let pos = interpolate_rail_station(&station_pts, u_coord, continuity);
                let vk = mesh.add_vertex_with_uv(pos, vec2(u_coord, v_coord));
                row.push(vk);
            }
            grid.push(row);
        }

        // 3. Connect quad faces between adjacent stations
        let num_v_quads = if is_closed_v { v_res } else { v_res - 1 };
        for v in 0..num_v_quads {
            let next_v = (v + 1) % v_res;
            for u in 0..(u_res - 1) {
                let next_u = u + 1;

                let v00 = grid[v][u];
                let v01 = grid[v][next_u];
                let v11 = grid[next_v][next_u];
                let v10 = grid[next_v][u];

                // Winding: v00 -> v10 -> v11 -> v01
                let _ = mesh.add_face(&[v00, v10, v11, v01]);
            }
        }

        // 4. If open in V, cap ends if requested and valid
        if !is_closed_v && u_res >= 3 {
            let start_cap = grid[0].clone();
            let _ = mesh.add_face(&start_cap);

            let mut end_cap = grid[v_res - 1].clone();
            end_cap.reverse();
            let _ = mesh.add_face(&end_cap);
        }

        mesh.update_face_weighted_normals(true);
        Ok(mesh)
    }

    /// Multi-rail loft from high-level Spline objects.
    pub fn multi_rail_loft_splines(
        rails: &[Spline],
        samples_u: usize,
        samples_v: usize,
        continuity: FilletContinuity,
        is_closed_v: bool,
    ) -> Result<Mesh, MeshError> {
        if rails.len() < 2 {
            return Err(MeshError::InvariantFailed(
                "Multi-rail loft requires at least 2 guide splines".into(),
            ));
        }

        let sampled: Vec<Vec<Vec3>> = rails.iter().map(|s| s.sample(samples_v.max(4))).collect();

        Self::multi_rail_loft(&sampled, samples_u, samples_v, continuity, is_closed_v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spline::types::BezierPoint;
    use core_math::vec3;

    #[test]
    fn test_resample_polyline() {
        let pts = vec![vec3(0.0, 0.0, 0.0), vec3(10.0, 0.0, 0.0)];
        let resampled = resample_polyline_3d(&pts, 5);
        assert_eq!(resampled.len(), 5);
        assert!((resampled[0].x - 0.0).abs() < 1e-4);
        assert!((resampled[2].x - 5.0).abs() < 1e-4);
        assert!((resampled[4].x - 10.0).abs() < 1e-4);
    }

    #[test]
    fn test_multi_rail_loft_three_rails_g2() {
        // 3 rails forming an arched canopy: left rail, center ridge rail, right rail
        let rail_left = vec![
            vec3(-2.0, 0.0, 0.0),
            vec3(-2.0, 0.0, 5.0),
            vec3(-2.0, 0.0, 10.0),
        ];
        let rail_center = vec![
            vec3(0.0, 1.5, 0.0),
            vec3(0.0, 2.0, 5.0),
            vec3(0.0, 1.5, 10.0),
        ];
        let rail_right = vec![
            vec3(2.0, 0.0, 0.0),
            vec3(2.0, 0.0, 5.0),
            vec3(2.0, 0.0, 10.0),
        ];

        let rails = vec![rail_left, rail_center, rail_right];

        let mesh = Spline::multi_rail_loft(&rails, 6, 8, FilletContinuity::G2Curvature, false)
            .expect("Multi-rail loft should succeed");

        assert!(mesh.vertices.len() >= 48);
        assert!(!mesh.faces.is_empty());
        assert!(mesh.validate().is_ok());
    }

    #[test]
    fn test_multi_rail_loft_splines() {
        let mut s1 = Spline::new();
        s1.add_point(BezierPoint::sharp(vec3(-1.0, 0.0, 0.0)));
        s1.add_point(BezierPoint::sharp(vec3(-1.0, 0.0, 4.0)));

        let mut s2 = Spline::new();
        s2.add_point(BezierPoint::sharp(vec3(1.0, 0.0, 0.0)));
        s2.add_point(BezierPoint::sharp(vec3(1.0, 0.0, 4.0)));

        let splines = vec![s1, s2];
        let mesh =
            Spline::multi_rail_loft_splines(&splines, 4, 6, FilletContinuity::G1Tangent, false)
                .expect("multi rail loft splines");

        assert!(!mesh.vertices.is_empty());
        assert!(mesh.validate().is_ok());
    }
}

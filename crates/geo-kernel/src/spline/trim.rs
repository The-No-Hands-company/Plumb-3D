//! CAD Multi-Segment Spline Boolean Trimming and Interactive Arc Tangent Snapping.
//!
//! Provides precision 2D/3D intersection detection between Bézier curves, trimming
//! segments at intersection points, and computing tangent arcs connecting arbitrary curves.

use crate::spline::types::{BezierPoint, Spline};
use core_math::Vec3;

/// Intersection between two spline curves in 3D/workplane space
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SplineIntersection {
    pub point: Vec3,
    pub segment_a: usize,
    pub t_a: f32,
    pub segment_b: usize,
    pub t_b: f32,
}

/// Tangent arc connecting two directed line/curve endpoints with G1 continuity
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TangentArc {
    pub center: Vec3,
    pub radius: f32,
    pub start_point: Vec3,
    pub end_point: Vec3,
    pub normal: Vec3,
}

impl TangentArc {
    /// Converts this circular tangent arc into a cubic Bézier spline approximation.
    pub fn to_spline(&self) -> Spline {
        let v_start = self.start_point - self.center;
        let v_end = self.end_point - self.center;
        let len_start = v_start.length();
        let len_end = v_end.length();
        if len_start < 1e-4 || len_end < 1e-4 {
            return Spline::with_points(vec![
                BezierPoint::sharp(self.start_point),
                BezierPoint::sharp(self.end_point),
            ], false);
        }

        let u_start = v_start / len_start;
        let u_end = v_end / len_end;

        let cos_theta = u_start.dot(u_end).clamp(-0.9999, 0.9999);
        let theta = cos_theta.acos();

        // Tangent directions at start and end
        let t_start = self.normal.cross(u_start).normalize_or_zero();
        let t_end = self.normal.cross(u_end).normalize_or_zero();

        // Orient tangents along the arc direction from start_point to end_point
        let dir_sign = if t_start.dot(self.end_point - self.start_point) >= 0.0 { 1.0 } else { -1.0 };
        let t_start_directed = t_start * dir_sign;
        let t_end_directed = t_end * dir_sign;

        let handle_len = (4.0 / 3.0) * (theta * 0.25).tan() * self.radius;

        let p0 = BezierPoint::with_type(
            self.start_point,
            Vec3::ZERO,
            t_start_directed * handle_len,
            crate::spline::types::HandleType::Aligned,
        );
        let p1 = BezierPoint::with_type(
            self.end_point,
            -t_end_directed * handle_len,
            Vec3::ZERO,
            crate::spline::types::HandleType::Aligned,
        );

        Spline::with_points(vec![p0, p1], false)
    }
}

impl Spline {
    /// Creates a piecewise linear spline from an array of 3D positions.
    pub fn polyline(positions: &[Vec3], is_closed: bool) -> Self {
        let points = positions.iter().map(|&p| BezierPoint::sharp(p)).collect();
        Self::with_points(points, is_closed)
    }

    /// Finds all 3D/planar intersection points between this spline and another cutting spline.
    pub fn find_intersections(&self, other: &Spline, tolerance: f32) -> Vec<SplineIntersection> {
        let mut results = Vec::new();
        let tol_sq = tolerance * tolerance;

        let segs_a = self.tessellate_segments(16);
        let segs_b = other.tessellate_segments(16);

        for (idx_a, p_a0, p_a1, t_a0, t_a1) in &segs_a {
            for (idx_b, p_b0, p_b1, t_b0, t_b1) in &segs_b {
                if let Some((pt, u_a, u_b)) = segment_segment_intersection(*p_a0, *p_a1, *p_b0, *p_b1, tol_sq) {
                    let final_t_a = t_a0 + u_a * (t_a1 - t_a0);
                    let final_t_b = t_b0 + u_b * (t_b1 - t_b0);

                    // Avoid duplicate nearby intersections
                    let is_dup = results.iter().any(|r: &SplineIntersection| (r.point - pt).length_squared() < tol_sq);
                    if !is_dup {
                        results.push(SplineIntersection {
                            point: pt,
                            segment_a: *idx_a,
                            t_a: final_t_a,
                            segment_b: *idx_b,
                            t_b: final_t_b,
                        });
                    }
                }
            }
        }

        results
    }

    /// Splits the spline at all intersection parameters, inserting knot points at exact intersection locations.
    pub fn split_at_intersections(&self, other: &Spline, tolerance: f32) -> Spline {
        let mut intersections = self.find_intersections(other, tolerance);
        if intersections.is_empty() {
            return self.clone();
        }

        // Sort intersections by segment index and parameter t_a descending so insertions don't invalidate indices
        intersections.sort_by(|a, b| {
            b.segment_a.cmp(&a.segment_a).then_with(|| b.t_a.partial_cmp(&a.t_a).unwrap_or(std::cmp::Ordering::Equal))
        });

        let mut out = self.clone();
        for isect in intersections {
            if isect.t_a > 1e-3 && isect.t_a < 0.999 {
                let _ = out.subdivide_segment(isect.segment_a, isect.t_a);
            }
        }

        out
    }

    /// Trims this spline against another cutting spline, keeping either the segments inside or outside the cutter.
    pub fn trim_with_spline(&self, cutter: &Spline, keep_inside: bool, tolerance: f32) -> Vec<Spline> {
        let split = self.split_at_intersections(cutter, tolerance);
        let n = split.points.len();
        if n < 2 {
            return vec![split];
        }

        let is_closed = split.is_closed;
        let seg_count = if is_closed { n } else { n - 1 };

        let mut sub_splines = Vec::new();
        let mut cur_points: Vec<BezierPoint> = Vec::new();

        for i in 0..seg_count {
            let next_i = (i + 1) % n;
            let p0 = split.points[i];
            let p1 = split.points[next_i];
            let mid = Spline::eval_segment(&p0, &p1, 0.5);

            let inside = cutter.contains_point_2d(mid);
            let keep = if keep_inside { inside } else { !inside };

            if keep {
                if cur_points.is_empty() {
                    cur_points.push(p0);
                }
                cur_points.push(p1);
            } else if !cur_points.is_empty() {
                if cur_points.len() >= 2 {
                    sub_splines.push(Spline::with_points(cur_points.clone(), false));
                }
                cur_points.clear();
            }
        }

        if cur_points.len() >= 2 {
            sub_splines.push(Spline::with_points(cur_points, false));
        }

        sub_splines
    }

    /// Tests if a 2D/projected 3D point lies inside a closed spline loop using the ray casting winding algorithm.
    pub fn contains_point_2d(&self, pt: Vec3) -> bool {
        let samples = self.sample(8);
        if samples.len() < 3 {
            return false;
        }

        let mut inside = false;
        let mut j = samples.len() - 1;
        for i in 0..samples.len() {
            let pi = samples[i];
            let pj = samples[j];

            if ((pi.z > pt.z) != (pj.z > pt.z))
                && (pt.x < (pj.x - pi.x) * (pt.z - pi.z) / (pj.z - pi.z + 1e-8) + pi.x)
            {
                inside = !inside;
            }
            j = i;
        }

        inside
    }

    /// Computes an interactive tangent arc connecting a start point and direction to an end point and direction (Plasticity / CAD Fillet Arc).
    pub fn compute_tangent_arc(
        start_pt: Vec3,
        start_dir: Vec3,
        end_pt: Vec3,
        end_dir: Vec3,
    ) -> Option<TangentArc> {
        let d_start = start_dir.normalize_or_zero();
        let d_end = end_dir.normalize_or_zero();

        if d_start.length_squared() < 1e-4 || d_end.length_squared() < 1e-4 {
            return None;
        }

        // Compute normal to the plane containing both points and tangent
        let chord = end_pt - start_pt;
        let chord_len = chord.length();
        if chord_len < 1e-4 {
            return None;
        }

        let normal = d_start.cross(chord).normalize_or_zero();
        let norm = if normal.length_squared() > 1e-4 {
            normal
        } else {
            d_start.cross(Vec3::Y).normalize_or_zero()
        };

        // Perpendicular directions in the plane pointing toward arc center
        let r_start = norm.cross(d_start).normalize();
        let r_end = norm.cross(d_end).normalize();

        // Solve for center where start_pt + r_start * R = end_pt + r_end * R
        let delta = end_pt - start_pt;
        let cross = r_start.cross(r_end);
        let denom = cross.length_squared();

        if denom < 1e-6 {
            // Tangents are parallel: arc radius is half the chord length
            let radius = chord_len * 0.5;
            let center = start_pt + chord * 0.5;
            return Some(TangentArc {
                center,
                radius,
                start_point: start_pt,
                end_point: end_pt,
                normal: norm,
            });
        }

        let t = delta.cross(r_end).dot(cross) / denom;
        let center = start_pt + r_start * t;
        let radius = (center - start_pt).length();

        Some(TangentArc {
            center,
            radius,
            start_point: start_pt,
            end_point: end_pt,
            normal: norm,
        })
    }

    /// Finds the nearest tangent snap point along this spline from an external cursor or reference point.
    pub fn snap_tangent_to_curve(&self, cursor_pt: Vec3) -> Option<(usize, f32, Vec3)> {
        let seg_count = if self.is_closed { self.points.len() } else { self.points.len().saturating_sub(1) };
        if seg_count == 0 {
            return None;
        }

        let mut best_dist = f32::MAX;
        let mut best_result = None;

        for i in 0..seg_count {
            let next_i = (i + 1) % self.points.len();
            let p0 = &self.points[i];
            let p1 = &self.points[next_i];

            for step in 0..=32 {
                let t = step as f32 / 32.0;
                let pt = Spline::eval_segment(p0, p1, t);
                let tangent = Spline::eval_segment_tangent(p0, p1, t).normalize_or_zero();

                // Vector from curve point to cursor
                let to_cursor = cursor_pt - pt;
                let dist = to_cursor.length();

                if dist > 1e-4 && tangent.length_squared() > 1e-4 {
                    // Check if tangent is perpendicular to radius vector (or parallel to line-of-sight)
                    let dot = tangent.dot(to_cursor.normalize());
                    let tangent_score = dot.abs();

                    if tangent_score > 0.96 && dist < best_dist {
                        best_dist = dist;
                        best_result = Some((i, t, pt));
                    }
                }
            }
        }

        best_result
    }

    fn tessellate_segments(&self, steps_per_seg: usize) -> Vec<(usize, Vec3, Vec3, f32, f32)> {
        let mut segments = Vec::new();
        let n = self.points.len();
        let seg_count = if self.is_closed { n } else { n.saturating_sub(1) };

        for i in 0..seg_count {
            let next_i = (i + 1) % n;
            let p0 = &self.points[i];
            let p1 = &self.points[next_i];
            let mut prev_pt = Spline::eval_segment(p0, p1, 0.0);
            let mut prev_t = 0.0_f32;

            for s in 1..=steps_per_seg {
                let cur_t = s as f32 / steps_per_seg as f32;
                let cur_pt = Spline::eval_segment(p0, p1, cur_t);
                segments.push((i, prev_pt, cur_pt, prev_t, cur_t));
                prev_pt = cur_pt;
                prev_t = cur_t;
            }
        }

        segments
    }
}

/// Computes the intersection of two 3D line segments in space within a squared tolerance.
fn segment_segment_intersection(
    p1: Vec3,
    p2: Vec3,
    p3: Vec3,
    p4: Vec3,
    tol_sq: f32,
) -> Option<(Vec3, f32, f32)> {
    let d1 = p2 - p1;
    let d2 = p4 - p3;
    let d12 = p1 - p3;

    let a = d1.dot(d1);
    let b = d1.dot(d2);
    let c = d2.dot(d2);
    let d = d1.dot(d12);
    let e = d2.dot(d12);

    let denom = a * c - b * b;
    if denom.abs() < 1e-7 {
        return None;
    }

    let s = (b * e - c * d) / denom;
    let t = (a * e - b * d) / denom;

    if (0.0..=1.0).contains(&s) && (0.0..=1.0).contains(&t) {
        let pt1 = p1 + d1 * s;
        let pt2 = p3 + d2 * t;
        if (pt1 - pt2).length_squared() <= tol_sq {
            return Some((pt1 * 0.5 + pt2 * 0.5, s, t));
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spline_intersection_and_trim() {
        let a = Spline::polyline(&[Vec3::new(-2.0, 0.0, 0.0), Vec3::new(2.0, 0.0, 0.0)], false);
        let b = Spline::polyline(&[Vec3::new(0.0, 0.0, -2.0), Vec3::new(0.0, 0.0, 2.0)], false);

        let isects = a.find_intersections(&b, 0.1);
        assert_eq!(isects.len(), 1);
        assert!((isects[0].point.x).abs() < 0.05);
        assert!((isects[0].point.z).abs() < 0.05);

        let split = a.split_at_intersections(&b, 0.1);
        assert_eq!(split.points.len(), 3);
        assert!((split.points[1].position.x).abs() < 0.05);
    }

    #[test]
    fn test_tangent_arc_computation() {
        let p1 = Vec3::new(-1.0, 0.0, 0.0);
        let d1 = Vec3::new(0.0, 0.0, 1.0);
        let p2 = Vec3::new(0.0, 0.0, 1.0);
        let d2 = Vec3::new(1.0, 0.0, 0.0);

        let arc = Spline::compute_tangent_arc(p1, d1, p2, d2).expect("Should find tangent arc");
        assert!(arc.radius > 0.0);
        assert!((arc.start_point - p1).length() < 1e-4);
        assert!((arc.end_point - p2).length() < 1e-4);
    }

    #[test]
    fn test_tangent_snap_point() {
        let circle = Spline::circle(2.0, Vec3::ZERO);
        // Tangent from point outside circle at (4, 0, 0)
        let snap = circle.snap_tangent_to_curve(Vec3::new(4.0, 0.0, 0.0));
        assert!(snap.is_some());
    }
}

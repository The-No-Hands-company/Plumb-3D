//! CAD 2D/3D Curve Fillet & Chamfer Engine (Plasticity / Rhino Style).
//! Rounds sharp corners of Bézier spline polylines with circular G1-continuous fillets.

use crate::spline::types::{BezierPoint, HandleType, Spline};
use core_math::Vec3;
use std::collections::HashMap;

impl Spline {
    /// Applies a precision CAD circular fillet of given radius to all sharp corners in the spline.
    /// Replaces sharp vertices with tangent-continuous circular cubic Bézier arcs.
    pub fn fillet_all_corners(&self, radius: f32) -> Spline {
        let n = self.points.len();
        let all_indices: Vec<usize> = (0..n).collect();
        self.fillet_selected_corners(&all_indices, radius)
    }

    /// Applies a precision CAD circular fillet to only the specified corner knot indices.
    pub fn fillet_selected_corners(&self, selected_knots: &[usize], radius: f32) -> Spline {
        let mut radii = HashMap::new();
        for &idx in selected_knots {
            radii.insert(idx, radius);
        }
        self.fillet_corners_variable(&radii)
    }

    /// Applies variable per-corner radius fillets based on a map of `(corner_index -> radius)`.
    pub fn fillet_corners_variable(&self, radii: &HashMap<usize, f32>) -> Spline {
        let n = self.points.len();
        if n < 3 || radii.is_empty() {
            return self.clone();
        }

        let mut out_points = Vec::new();
        let is_closed = self.is_closed;

        let start_idx = if is_closed { 0 } else { 1 };
        let end_idx = if is_closed { n } else { n - 1 };

        if !is_closed {
            out_points.push(self.points[0]);
        }

        for i in start_idx..end_idx {
            let radius = radii.get(&i).copied().unwrap_or(0.0);
            if radius <= 1e-4 {
                out_points.push(self.points[i]);
                continue;
            }

            let prev_idx = if i == 0 { n - 1 } else { i - 1 };
            let next_idx = (i + 1) % n;

            let curr = self.points[i].position;
            let prev = self.points[prev_idx].position;
            let next = self.points[next_idx].position;

            let v1 = prev - curr;
            let v2 = next - curr;

            let l1 = v1.length();
            let l2 = v2.length();

            if l1 < 1e-4 || l2 < 1e-4 {
                out_points.push(self.points[i]);
                continue;
            }

            let u1 = v1 / l1;
            let u2 = v2 / l2;

            let cos_theta = u1.dot(u2).clamp(-0.9999, 0.9999);
            let half_angle = (cos_theta.acos() * 0.5).max(0.01);

            // Tangent trim distance from corner point: d = R / tan(alpha / 2)
            let desired_d = radius / half_angle.tan();
            let max_d = (l1.min(l2) * 0.45).max(1e-4);
            let d = desired_d.min(max_d);
            let effective_radius = d * half_angle.tan();

            // Tangent points on the two legs
            let t1 = curr + u1 * d;
            let t2 = curr + u2 * d;

            // Arc subtended angle: arc_angle = PI - 2 * half_angle
            let arc_angle = std::f32::consts::PI - 2.0 * half_angle;
            // Cubic Bézier handle length for circular arc
            let handle_len = (4.0 / 3.0) * (arc_angle * 0.25).tan() * effective_radius;

            let p1 = BezierPoint::with_type(
                t1,
                Vec3::ZERO,
                -u1 * handle_len,
                HandleType::Aligned,
            );

            let p2 = BezierPoint::with_type(
                t2,
                -u2 * handle_len,
                Vec3::ZERO,
                HandleType::Aligned,
            );

            out_points.push(p1);
            out_points.push(p2);
        }

        if !is_closed {
            out_points.push(self.points[n - 1]);
        }

        Spline {
            points: out_points,
            is_closed,
        }
    }
}

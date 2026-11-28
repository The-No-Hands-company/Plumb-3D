use core_math::Vec3;

use super::types::{BezierPoint, Spline};

/// Single needle sample in a CAD Curvature Comb (Porcupine plot) analysis.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CurvatureCombSample {
    pub origin: Vec3,
    pub needle_tip: Vec3,
    pub tangent: Vec3,
    pub normal: Vec3,
    pub curvature: f32,
}

impl Spline {
    /// Evaluates a single cubic Bézier segment between `p0` and `p1` at parameter `t` in [0, 1].
    pub fn eval_segment(p0: &BezierPoint, p1: &BezierPoint, t: f32) -> Vec3 {
        let t2 = t * t;
        let t3 = t2 * t;
        let mt = 1.0 - t;
        let mt2 = mt * mt;
        let mt3 = mt2 * mt;

        let cp0 = p0.position;
        let cp1 = p0.position + p0.handle_out;
        let cp2 = p1.position + p1.handle_in;
        let cp3 = p1.position;

        cp0 * mt3 + cp1 * (3.0 * mt2 * t) + cp2 * (3.0 * mt * t2) + cp3 * t3
    }

    /// Evaluates the tangent (velocity vector) of a cubic Bézier segment at parameter `t` in [0, 1].
    pub fn eval_segment_tangent(p0: &BezierPoint, p1: &BezierPoint, t: f32) -> Vec3 {
        let mt = 1.0 - t;
        let mt2 = mt * mt;
        let t2 = t * t;

        let cp0 = p0.position;
        let cp1 = p0.position + p0.handle_out;
        let cp2 = p1.position + p1.handle_in;
        let cp3 = p1.position;

        let d = (cp1 - cp0) * (3.0 * mt2) + (cp2 - cp1) * (6.0 * mt * t) + (cp3 - cp2) * (3.0 * t2);
        d.normalize_or_zero()
    }

    /// Evaluates the second derivative (acceleration vector) of a cubic Bézier segment at parameter `t` in [0, 1].
    pub fn eval_segment_second_derivative(p0: &BezierPoint, p1: &BezierPoint, t: f32) -> Vec3 {
        let mt = 1.0 - t;
        let cp0 = p0.position;
        let cp1 = p0.position + p0.handle_out;
        let cp2 = p1.position + p1.handle_in;
        let cp3 = p1.position;

        // C''(t) = 6(1-t)(cp2 - 2cp1 + cp0) + 6t(cp3 - 2cp2 + cp1)
        (cp2 - cp1 * 2.0 + cp0) * (6.0 * mt) + (cp3 - cp2 * 2.0 + cp1) * (6.0 * t)
    }

    /// Evaluates curvature magnitude and principal normal vector at segment parameter `t`.
    pub fn eval_segment_curvature(p0: &BezierPoint, p1: &BezierPoint, t: f32) -> (f32, Vec3) {
        let mt = 1.0 - t;
        let mt2 = mt * mt;
        let t2 = t * t;

        let cp0 = p0.position;
        let cp1 = p0.position + p0.handle_out;
        let cp2 = p1.position + p1.handle_in;
        let cp3 = p1.position;

        let d1 =
            (cp1 - cp0) * (3.0 * mt2) + (cp2 - cp1) * (6.0 * mt * t) + (cp3 - cp2) * (3.0 * t2);
        let speed = d1.length();
        if speed < 1e-5 {
            return (0.0, Vec3::Y);
        }

        let d2 = (cp2 - cp1 * 2.0 + cp0) * (6.0 * mt) + (cp3 - cp2 * 2.0 + cp1) * (6.0 * t);
        let cross = d1.cross(d2);
        let cross_len = cross.length();
        let curvature = cross_len / (speed * speed * speed);

        let normal = if cross_len > 1e-5 {
            cross.cross(d1).normalize_or_zero()
        } else {
            let arbitrary = if d1.normalize_or_zero().y.abs() < 0.9 {
                Vec3::Y
            } else {
                Vec3::X
            };
            d1.cross(arbitrary).normalize_or_zero()
        };

        (curvature, normal)
    }

    /// Evaluates the point and tangent on the entire spline at global parameter `u` in [0, 1].
    pub fn eval(&self, u: f32) -> (Vec3, Vec3) {
        let n = self.points.len();
        if n < 2 {
            let pos = self
                .points
                .first()
                .map(|p| p.position)
                .unwrap_or(Vec3::ZERO);
            return (pos, Vec3::Z);
        }

        let segment_count = if self.is_closed { n } else { n - 1 };
        let clamped_u = u.clamp(0.0, 1.0);

        if !self.is_closed && (clamped_u - 1.0).abs() < 1e-5 {
            let p0 = &self.points[n - 2];
            let p1 = &self.points[n - 1];
            return (p1.position, Self::eval_segment_tangent(p0, p1, 1.0));
        }

        let scaled = clamped_u * segment_count as f32;
        let seg_idx = (scaled.floor() as usize).min(segment_count - 1);
        let t = (scaled - seg_idx as f32).clamp(0.0, 1.0);

        let p0 = &self.points[seg_idx];
        let p1 = &self.points[(seg_idx + 1) % n];

        let pos = Self::eval_segment(p0, p1, t);
        let tan = Self::eval_segment_tangent(p0, p1, t);
        (pos, tan)
    }

    /// Samples the entire spline into a sequence of points with `samples_per_segment`.
    pub fn sample(&self, samples_per_segment: usize) -> Vec<Vec3> {
        let n = self.points.len();
        if n < 2 {
            return self.points.iter().map(|p| p.position).collect();
        }

        let segment_count = if self.is_closed { n } else { n - 1 };
        let mut sampled = Vec::with_capacity(segment_count * samples_per_segment + 1);

        for seg in 0..segment_count {
            let p0 = &self.points[seg];
            let p1 = &self.points[(seg + 1) % n];

            for s in 0..samples_per_segment {
                let t = s as f32 / samples_per_segment as f32;
                sampled.push(Self::eval_segment(p0, p1, t));
            }
        }

        if !self.is_closed {
            sampled.push(self.points.last().unwrap().position);
        }

        sampled
    }

    /// Evaluates CAD Curvature Comb (Porcupine plot) samples along the entire spline.
    /// Returns origin positions, needle tips, tangents, normals, and curvature values.
    /// `scale` scales needle length: `needle_tip = origin + normal * (curvature * scale)`.
    pub fn eval_curvature_comb(
        &self,
        samples_per_segment: usize,
        scale: f32,
    ) -> Vec<CurvatureCombSample> {
        let n = self.points.len();
        if n < 2 {
            return Vec::new();
        }

        let segment_count = if self.is_closed { n } else { n - 1 };
        let mut needles = Vec::with_capacity(segment_count * samples_per_segment + 1);

        for seg in 0..segment_count {
            let p0 = &self.points[seg];
            let p1 = &self.points[(seg + 1) % n];

            for s in 0..samples_per_segment {
                let t = s as f32 / samples_per_segment as f32;
                let origin = Self::eval_segment(p0, p1, t);
                let tangent = Self::eval_segment_tangent(p0, p1, t);
                let (curvature, normal) = Self::eval_segment_curvature(p0, p1, t);
                let needle_tip = origin + normal * (curvature * scale);

                needles.push(CurvatureCombSample {
                    origin,
                    needle_tip,
                    tangent,
                    normal,
                    curvature,
                });
            }
        }

        if !self.is_closed {
            let p0 = &self.points[n - 2];
            let p1 = &self.points[n - 1];
            let origin = p1.position;
            let tangent = Self::eval_segment_tangent(p0, p1, 1.0);
            let (curvature, normal) = Self::eval_segment_curvature(p0, p1, 1.0);
            let needle_tip = origin + normal * (curvature * scale);
            needles.push(CurvatureCombSample {
                origin,
                needle_tip,
                tangent,
                normal,
                curvature,
            });
        }

        needles
    }

    /// Subdivides the curve segment between point `segment_idx` and `segment_idx + 1` at parameter `t` (default 0.5)
    /// using exact De Casteljau cubic Bézier subdivision.
    /// Inserts a new `BezierPoint` preserving the exact curvature and shape of the spline.
    pub fn subdivide_segment(&mut self, segment_idx: usize, t: f32) -> Result<usize, String> {
        let n = self.points.len();
        if n < 2 {
            return Err("Spline requires at least 2 points to subdivide".into());
        }
        let max_seg = if self.is_closed { n } else { n - 1 };
        if segment_idx >= max_seg {
            return Err(format!(
                "Segment index {} out of bounds (max {})",
                segment_idx,
                max_seg.saturating_sub(1)
            ));
        }

        let next_idx = (segment_idx + 1) % n;
        let p0_pos = self.points[segment_idx].position;
        let p1_pos = p0_pos + self.points[segment_idx].handle_out;
        let p3_pos = self.points[next_idx].position;
        let p2_pos = p3_pos + self.points[next_idx].handle_in;

        let clamp_t = t.clamp(0.01, 0.99);
        let omt = 1.0 - clamp_t;

        // Level 1 De Casteljau
        let q0 = p0_pos * omt + p1_pos * clamp_t;
        let q1 = p1_pos * omt + p2_pos * clamp_t;
        let q2 = p2_pos * omt + p3_pos * clamp_t;

        // Level 2 De Casteljau
        let r0 = q0 * omt + q1 * clamp_t;
        let r1 = q1 * omt + q2 * clamp_t;

        // Level 3 (point on curve)
        let s = r0 * omt + r1 * clamp_t;

        // Update original start point handle_out
        self.points[segment_idx].handle_out = q0 - p0_pos;

        // Create new midpoint
        let mid_point = BezierPoint::new(s, r0 - s, r1 - s);

        // Update original end point handle_in
        self.points[next_idx].handle_in = q2 - p3_pos;

        // Insert new point
        let insert_idx = segment_idx + 1;
        self.points.insert(insert_idx, mid_point);

        Ok(insert_idx)
    }

    /// Inserts a new control knot at global parameter `u` in [0, 1] without altering the curve shape.
    pub fn insert_knot_at_parameter(&mut self, u: f32) -> Result<usize, String> {
        let n = self.points.len();
        if n < 2 {
            return Err("Spline requires at least 2 points to insert a knot".into());
        }
        let seg_count = if self.is_closed { n } else { n - 1 };
        let clamped_u = u.clamp(0.001, 0.999);
        let scaled = clamped_u * seg_count as f32;
        let seg_idx = (scaled.floor() as usize).min(seg_count - 1);
        let t = (scaled - seg_idx as f32).clamp(0.01, 0.99);

        self.subdivide_segment(seg_idx, t)
    }

    /// Finds the closest point on the spline curve to `query`, returning `(u_parameter, closest_point, distance)`.
    pub fn closest_point(&self, query: Vec3, samples_per_segment: usize) -> (f32, Vec3, f32) {
        let n = self.points.len();
        if n < 2 {
            let pos = self
                .points
                .first()
                .map(|p| p.position)
                .unwrap_or(Vec3::ZERO);
            return (0.0, pos, (pos - query).length());
        }

        let seg_count = if self.is_closed { n } else { n - 1 };
        let total_samples = seg_count * samples_per_segment.max(4);

        let mut best_u = 0.0;
        let mut best_pt = Vec3::ZERO;
        let mut best_dist_sq = f32::MAX;

        for i in 0..=total_samples {
            let u = i as f32 / total_samples as f32;
            let (pt, _) = self.eval(u);
            let d_sq = (pt - query).length_squared();
            if d_sq < best_dist_sq {
                best_dist_sq = d_sq;
                best_u = u;
                best_pt = pt;
            }
        }

        (best_u, best_pt, best_dist_sq.sqrt())
    }

    /// Projects `point` onto the spline curve and inserts a new knot at the closest location.
    pub fn insert_knot_at_point(&mut self, point: Vec3) -> Result<usize, String> {
        let (u, _, _) = self.closest_point(point, 16);
        self.insert_knot_at_parameter(u)
    }

    /// Reverses spline direction and swaps incoming/outgoing handles for all control points.
    pub fn reverse(&mut self) {
        self.points.reverse();
        for pt in &mut self.points {
            std::mem::swap(&mut pt.handle_in, &mut pt.handle_out);
        }
    }

    /// Calculates total arc length of the spline curve via numerical integration.
    pub fn arc_length(&self, samples_per_segment: usize) -> f32 {
        let sampled = self.sample(samples_per_segment.max(4));
        let mut total = 0.0;
        for i in 1..sampled.len() {
            total += (sampled[i] - sampled[i - 1]).length();
        }
        total
    }

    /// Evaluates position and tangent along the curve at specified arc length `distance`.
    pub fn eval_at_distance(&self, distance: f32, samples_per_segment: usize) -> (Vec3, Vec3) {
        let sampled = self.sample(samples_per_segment.max(8));
        if sampled.len() < 2 {
            let pos = sampled.first().copied().unwrap_or(Vec3::ZERO);
            return (pos, Vec3::Z);
        }

        let mut accumulated = 0.0;
        let target = distance.max(0.0);

        for i in 1..sampled.len() {
            let p_prev = sampled[i - 1];
            let p_curr = sampled[i];
            let seg_len = (p_curr - p_prev).length();

            if accumulated + seg_len >= target || i == sampled.len() - 1 {
                let remain = target - accumulated;
                let factor = if seg_len > 1e-6 {
                    (remain / seg_len).clamp(0.0, 1.0)
                } else {
                    0.0
                };
                let pos = p_prev + (p_curr - p_prev) * factor;
                let tangent = (p_curr - p_prev).normalize_or_zero();
                return (pos, tangent);
            }
            accumulated += seg_len;
        }

        let last = *sampled.last().unwrap();
        let prev = sampled[sampled.len() - 2];
        (last, (last - prev).normalize_or_zero())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core_math::vec3;

    #[test]
    fn test_curvature_comb_circular_arc() {
        // Build a 90-degree circular arc approximation
        let mut s = Spline::new();
        let k = 0.55228475; // standard cubic bezier circle approximation
        let mut p0 = BezierPoint::sharp(vec3(1.0, 0.0, 0.0));
        p0.handle_out = vec3(0.0, k, 0.0);
        let mut p1 = BezierPoint::sharp(vec3(0.0, 1.0, 0.0));
        p1.handle_in = vec3(k, 0.0, 0.0);

        s.add_point(p0);
        s.add_point(p1);

        let needles = s.eval_curvature_comb(8, 1.0);
        assert!(!needles.is_empty());
        // For a unit circle, curvature should be approximately 1.0
        for needle in &needles {
            assert!(needle.curvature > 0.7 && needle.curvature < 1.3);
            assert!(needle.needle_tip != needle.origin);
        }
    }
}

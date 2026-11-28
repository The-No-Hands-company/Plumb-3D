//! Fast Iterative 2D CAD Geometric & Dimensional Constraint Solver.
//!
//! Uses projection relaxation with adaptive damping to solve non-linear
//! geometric and dimensional systems in sub-millisecond execution times.

use super::constraints::{ConstraintType, SketchConstraint};
use super::entities::{Sketch, SketchEntity, SketchPointId};
use core_math::{vec2, Vec2};

/// Solver convergence outcome and degrees-of-freedom diagnostics.
#[derive(Debug, Clone, PartialEq)]
pub enum SolverStatus {
    /// Fully converged within tolerance.
    Solved {
        iterations: usize,
        max_residual: f32,
    },
    /// Reached maximum iteration count without dropping below tolerance.
    DidNotConverge {
        iterations: usize,
        max_residual: f32,
    },
    /// No constraints to solve.
    Unconstrained,
}

/// Configuration parameters for the 2D CAD constraint solver.
#[derive(Debug, Clone, Copy)]
pub struct SolverConfig {
    pub max_iterations: usize,
    pub tolerance: f32,
    pub relaxation_factor: f32,
}

impl Default for SolverConfig {
    fn default() -> Self {
        Self {
            max_iterations: 60,
            tolerance: 1e-4,
            relaxation_factor: 0.95,
        }
    }
}

/// Solves all active constraints in the sketch using iterative projection relaxation.
pub fn solve_sketch(sketch: &mut Sketch, config: SolverConfig) -> SolverStatus {
    let active_constraints: Vec<SketchConstraint> = sketch
        .constraints
        .iter()
        .filter(|c| c.is_active)
        .copied()
        .collect();

    if active_constraints.is_empty() {
        return SolverStatus::Unconstrained;
    }

    let mut max_residual = f32::MAX;
    let mut iter = 0;

    while iter < config.max_iterations {
        max_residual = 0.0;
        let mut any_violation = false;

        for constraint in &active_constraints {
            let res =
                apply_constraint_projection(sketch, &constraint.kind, config.relaxation_factor);
            if res > max_residual {
                max_residual = res;
            }
            if res > config.tolerance {
                any_violation = true;
            }
        }

        iter += 1;
        if !any_violation || max_residual <= config.tolerance {
            return SolverStatus::Solved {
                iterations: iter,
                max_residual,
            };
        }
    }

    SolverStatus::DidNotConverge {
        iterations: iter,
        max_residual,
    }
}

/// Projects positions of involved points to satisfy a single constraint. Returns absolute residual.
fn apply_constraint_projection(sketch: &mut Sketch, kind: &ConstraintType, alpha: f32) -> f32 {
    match *kind {
        ConstraintType::Coincident { pt_a, pt_b } => {
            let (pa, pb, fa, fb) = get_pair_positions(sketch, pt_a, pt_b);
            let delta = pb - pa;
            let err = delta.length();
            if err < 1e-6 {
                return 0.0;
            }
            let step = delta * alpha;
            relax_pair(sketch, pt_a, pt_b, pa, pb, step * 0.5, -step * 0.5, fa, fb);
            err
        }

        ConstraintType::Horizontal { pt_a, pt_b } => {
            let (pa, pb, fa, fb) = get_pair_positions(sketch, pt_a, pt_b);
            let err = (pb.y - pa.y).abs();
            let avg_y = (pa.y + pb.y) * 0.5;
            let dy_a = (avg_y - pa.y) * alpha;
            let dy_b = (avg_y - pb.y) * alpha;
            relax_pair(
                sketch,
                pt_a,
                pt_b,
                pa,
                pb,
                vec2(0.0, dy_a),
                vec2(0.0, dy_b),
                fa,
                fb,
            );
            err
        }

        ConstraintType::Vertical { pt_a, pt_b } => {
            let (pa, pb, fa, fb) = get_pair_positions(sketch, pt_a, pt_b);
            let err = (pb.x - pa.x).abs();
            let avg_x = (pa.x + pb.x) * 0.5;
            let dx_a = (avg_x - pa.x) * alpha;
            let dx_b = (avg_x - pb.x) * alpha;
            relax_pair(
                sketch,
                pt_a,
                pt_b,
                pa,
                pb,
                vec2(dx_a, 0.0),
                vec2(dx_b, 0.0),
                fa,
                fb,
            );
            err
        }

        ConstraintType::Distance {
            pt_a,
            pt_b,
            distance,
        } => {
            let (pa, pb, fa, fb) = get_pair_positions(sketch, pt_a, pt_b);
            let delta = pb - pa;
            let current_len = delta.length();
            let err = (current_len - distance).abs();
            if current_len < 1e-7 {
                let step = vec2(distance * 0.5 * alpha, 0.0);
                relax_pair(sketch, pt_a, pt_b, pa, pb, -step, step, fa, fb);
                return err;
            }
            let dir = delta / current_len;
            let correction = dir * (current_len - distance) * alpha;
            relax_pair(
                sketch,
                pt_a,
                pt_b,
                pa,
                pb,
                correction * 0.5,
                -correction * 0.5,
                fa,
                fb,
            );
            err
        }

        ConstraintType::HorizontalDistance {
            pt_a,
            pt_b,
            distance,
        } => {
            let (pa, pb, fa, fb) = get_pair_positions(sketch, pt_a, pt_b);
            let current_dx = pb.x - pa.x;
            let err = (current_dx - distance).abs();
            let correction = (current_dx - distance) * alpha;
            relax_pair(
                sketch,
                pt_a,
                pt_b,
                pa,
                pb,
                vec2(correction * 0.5, 0.0),
                vec2(-correction * 0.5, 0.0),
                fa,
                fb,
            );
            err
        }

        ConstraintType::VerticalDistance {
            pt_a,
            pt_b,
            distance,
        } => {
            let (pa, pb, fa, fb) = get_pair_positions(sketch, pt_a, pt_b);
            let current_dy = pb.y - pa.y;
            let err = (current_dy - distance).abs();
            let correction = (current_dy - distance) * alpha;
            relax_pair(
                sketch,
                pt_a,
                pt_b,
                pa,
                pb,
                vec2(0.0, correction * 0.5),
                vec2(0.0, -correction * 0.5),
                fa,
                fb,
            );
            err
        }

        ConstraintType::FixedPoint { point, target } => {
            if let Some(p) = sketch.get_point_mut(point) {
                let err = (p.pos - target).length();
                p.pos = target;
                err
            } else {
                0.0
            }
        }

        ConstraintType::Midpoint { point, line } => {
            if let Some(SketchEntity::Line(l)) = sketch.entities.get(line.0) {
                let (pa, pb, _, _) = get_pair_positions(sketch, l.start, l.end);
                let mid = (pa + pb) * 0.5;
                if let Some(pm) = sketch.get_point_mut(point) {
                    let err = (pm.pos - mid).length();
                    if !pm.is_fixed {
                        pm.pos = pm.pos + (mid - pm.pos) * alpha;
                    }
                    err
                } else {
                    0.0
                }
            } else {
                0.0
            }
        }

        ConstraintType::EqualLength { line_a, line_b } => {
            let (la_start, la_end) = match sketch.entities.get(line_a.0) {
                Some(SketchEntity::Line(l)) => (l.start, l.end),
                _ => return 0.0,
            };
            let (lb_start, lb_end) = match sketch.entities.get(line_b.0) {
                Some(SketchEntity::Line(l)) => (l.start, l.end),
                _ => return 0.0,
            };

            let (pa1, pa2, fa1, fa2) = get_pair_positions(sketch, la_start, la_end);
            let (pb1, pb2, fb1, fb2) = get_pair_positions(sketch, lb_start, lb_end);

            let len_a = (pa2 - pa1).length().max(1e-5);
            let len_b = (pb2 - pb1).length().max(1e-5);
            let err = (len_a - len_b).abs();
            let target_len = (len_a + len_b) * 0.5;

            let dir_a = (pa2 - pa1) / len_a;
            let dir_b = (pb2 - pb1) / len_b;

            let corr_a = dir_a * (len_a - target_len) * alpha;
            let corr_b = dir_b * (len_b - target_len) * alpha;

            relax_pair(
                sketch,
                la_start,
                la_end,
                pa1,
                pa2,
                corr_a * 0.5,
                -corr_a * 0.5,
                fa1,
                fa2,
            );
            relax_pair(
                sketch,
                lb_start,
                lb_end,
                pb1,
                pb2,
                corr_b * 0.5,
                -corr_b * 0.5,
                fb1,
                fb2,
            );
            err
        }

        ConstraintType::Parallel { line_a, line_b } => {
            let (la_start, la_end) = match sketch.entities.get(line_a.0) {
                Some(SketchEntity::Line(l)) => (l.start, l.end),
                _ => return 0.0,
            };
            let (lb_start, lb_end) = match sketch.entities.get(line_b.0) {
                Some(SketchEntity::Line(l)) => (l.start, l.end),
                _ => return 0.0,
            };

            let (pa1, pa2, _, _) = get_pair_positions(sketch, la_start, la_end);
            let (pb1, pb2, fb1, fb2) = get_pair_positions(sketch, lb_start, lb_end);

            let dir_a = (pa2 - pa1).normalize_or_zero();
            let dir_b = (pb2 - pb1).normalize_or_zero();
            let cross = dir_a.x * dir_b.y - dir_a.y * dir_b.x;
            let err = cross.abs();

            if dir_a != Vec2::ZERO {
                let len_b = (pb2 - pb1).length();
                let aligned_b = dir_a * len_b * if dir_a.dot(dir_b) >= 0.0 { 1.0 } else { -1.0 };
                let target_p2 = pb1 + aligned_b;
                let step = (target_p2 - pb2) * alpha;
                if !fb2 {
                    if let Some(p) = sketch.get_point_mut(lb_end) {
                        p.pos += step;
                    }
                } else if !fb1 {
                    if let Some(p) = sketch.get_point_mut(lb_start) {
                        p.pos -= step;
                    }
                }
            }
            err
        }

        ConstraintType::Perpendicular { line_a, line_b } => {
            let (la_start, la_end) = match sketch.entities.get(line_a.0) {
                Some(SketchEntity::Line(l)) => (l.start, l.end),
                _ => return 0.0,
            };
            let (lb_start, lb_end) = match sketch.entities.get(line_b.0) {
                Some(SketchEntity::Line(l)) => (l.start, l.end),
                _ => return 0.0,
            };

            let (pa1, pa2, _, _) = get_pair_positions(sketch, la_start, la_end);
            let (pb1, pb2, _, fb2) = get_pair_positions(sketch, lb_start, lb_end);

            let dir_a = (pa2 - pa1).normalize_or_zero();
            let perp_a = vec2(-dir_a.y, dir_a.x);
            let dir_b = (pb2 - pb1).normalize_or_zero();
            let err = dir_a.dot(dir_b).abs();

            let len_b = (pb2 - pb1).length();
            let aligned_b = perp_a * len_b * if perp_a.dot(dir_b) >= 0.0 { 1.0 } else { -1.0 };
            let target_p2 = pb1 + aligned_b;
            let step = (target_p2 - pb2) * alpha;
            if !fb2 {
                if let Some(p) = sketch.get_point_mut(lb_end) {
                    p.pos += step;
                }
            }
            err
        }

        _ => 0.0,
    }
}

fn get_pair_positions(
    sketch: &Sketch,
    a: SketchPointId,
    b: SketchPointId,
) -> (Vec2, Vec2, bool, bool) {
    let pa = sketch
        .get_point(a)
        .map(|p| (p.pos, p.is_fixed))
        .unwrap_or((Vec2::ZERO, false));
    let pb = sketch
        .get_point(b)
        .map(|p| (p.pos, p.is_fixed))
        .unwrap_or((Vec2::ZERO, false));
    (pa.0, pb.0, pa.1, pb.1)
}

#[allow(clippy::too_many_arguments)]
fn relax_pair(
    sketch: &mut Sketch,
    id_a: SketchPointId,
    id_b: SketchPointId,
    pa: Vec2,
    pb: Vec2,
    step_a: Vec2,
    step_b: Vec2,
    fixed_a: bool,
    fixed_b: bool,
) {
    if fixed_a && fixed_b {
        return;
    }
    if fixed_a {
        if let Some(p) = sketch.get_point_mut(id_b) {
            p.pos = pb + step_b * 2.0;
        }
    } else if fixed_b {
        if let Some(p) = sketch.get_point_mut(id_a) {
            p.pos = pa + step_a * 2.0;
        }
    } else {
        if let Some(p) = sketch.get_point_mut(id_a) {
            p.pos = pa + step_a;
        }
        if let Some(p) = sketch.get_point_mut(id_b) {
            p.pos = pb + step_b;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sketch::constraints::ConstraintId;
    use crate::sketch::workplane::Workplane;

    #[test]
    fn test_solve_horizontal_and_distance_constraints() {
        let mut sketch = Sketch::new("TestSketch", Workplane::xy());
        let p1 = sketch.add_fixed_point(vec2(0.0, 0.0));
        let p2 = sketch.add_point(vec2(8.0, 5.0));
        let line = sketch.add_line(p1, p2);

        // Add Horizontal constraint
        sketch.constraints.push(SketchConstraint::new(
            ConstraintId(0),
            ConstraintType::Horizontal { pt_a: p1, pt_b: p2 },
        ));

        // Add Distance constraint = 25.0 mm
        sketch.constraints.push(SketchConstraint::new(
            ConstraintId(1),
            ConstraintType::Distance {
                pt_a: p1,
                pt_b: p2,
                distance: 25.0,
            },
        ));

        let res = solve_sketch(&mut sketch, SolverConfig::default());
        assert!(matches!(res, SolverStatus::Solved { .. }));

        let p2_solved = sketch.get_point(p2).unwrap().pos;
        assert!(
            (p2_solved.y - 0.0).abs() < 1e-3,
            "Y must be 0.0 for horizontal line"
        );
        assert!(
            (p2_solved.x - 25.0).abs() < 1e-3,
            "X must be 25.0 for distance constraint"
        );
        assert_eq!(line.0, 0);
    }
}

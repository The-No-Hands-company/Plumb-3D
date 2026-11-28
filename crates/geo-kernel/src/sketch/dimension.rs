//! Live Dimension Callouts & Interactive Annotation Badges for CAD Sketching.
//!
//! Generates dimension lines, witness extension lines, and formatted metric badges
//! (e.g. "50.00 mm", "45.0°", "R 12.50 mm") for real-time viewport display.

use super::constraints::{ConstraintType, SketchConstraint};
use super::entities::{Sketch, SketchEntity};
use core_math::{vec2, Vec2, Vec3};
use serde::{Deserialize, Serialize};

/// Visual orientation and type of dimension callout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DimensionType {
    LinearAligned,
    LinearHorizontal,
    LinearVertical,
    Radial,
    Angular,
}

/// A rendered 2D dimension callout badge with witness lines and formatted readout.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DimensionCallout {
    /// Associated constraint ID (if driving)
    pub constraint_id: Option<usize>,
    /// Dimension classification
    pub kind: DimensionType,
    /// Numeric metric value (in millimeters or degrees)
    pub value: f32,
    /// Formatted text string shown on badge (e.g., "50.00 mm", "90.0°")
    pub label: String,
    /// First witness reference point (2D sketch plane)
    pub p1: Vec2,
    /// Second witness reference point (2D sketch plane)
    pub p2: Vec2,
    /// Dimension line start point
    pub dim_start: Vec2,
    /// Dimension line end point
    pub dim_end: Vec2,
    /// Midpoint text badge anchor
    pub text_pos: Vec2,
    /// Normal direction of dimension offset
    pub normal: Vec2,
    /// Driving constraint (true = edits modify geometry, false = driven reference dimension)
    pub is_driving: bool,
}

impl DimensionCallout {
    /// Converts the 2D sketch badge positions into 3D world coordinates for viewport drawing.
    pub fn to_3d(&self, sketch: &Sketch) -> (Vec3, Vec3, Vec3, Vec3, Vec3) {
        let wp = &sketch.workplane;
        (
            wp.to_3d(self.p1),
            wp.to_3d(self.p2),
            wp.to_3d(self.dim_start),
            wp.to_3d(self.dim_end),
            wp.to_3d(self.text_pos),
        )
    }
}

/// Extracts all live dimension callouts from a sketch's dimensional constraints.
pub fn extract_dimension_callouts(sketch: &Sketch, leader_offset: f32) -> Vec<DimensionCallout> {
    let mut callouts = Vec::new();

    for constraint in &sketch.constraints {
        if !constraint.is_active {
            continue;
        }

        if let Some(callout) = constraint_to_callout(sketch, constraint, leader_offset) {
            callouts.push(callout);
        }
    }

    callouts
}

fn constraint_to_callout(
    sketch: &Sketch,
    c: &SketchConstraint,
    offset_dist: f32,
) -> Option<DimensionCallout> {
    match c.kind {
        ConstraintType::Distance {
            pt_a,
            pt_b,
            distance,
        } => {
            let pa = sketch.get_point(pt_a)?.pos;
            let pb = sketch.get_point(pt_b)?.pos;
            let delta = pb - pa;
            let len = delta.length();
            if len < 1e-4 {
                return None;
            }
            let dir = delta / len;
            let perp = vec2(-dir.y, dir.x);
            let offset = perp * offset_dist;

            let dim_start = pa + offset;
            let dim_end = pb + offset;
            let text_pos = (dim_start + dim_end) * 0.5;

            Some(DimensionCallout {
                constraint_id: Some(c.id.0),
                kind: DimensionType::LinearAligned,
                value: distance,
                label: format!("{:.2} mm", distance),
                p1: pa,
                p2: pb,
                dim_start,
                dim_end,
                text_pos,
                normal: perp,
                is_driving: true,
            })
        }

        ConstraintType::HorizontalDistance {
            pt_a,
            pt_b,
            distance,
        } => {
            let pa = sketch.get_point(pt_a)?.pos;
            let pb = sketch.get_point(pt_b)?.pos;
            let offset_y = offset_dist.copysign(1.0);

            let dim_start = vec2(pa.x, pa.y.max(pb.y) + offset_y);
            let dim_end = vec2(pb.x, pa.y.max(pb.y) + offset_y);
            let text_pos = (dim_start + dim_end) * 0.5;

            Some(DimensionCallout {
                constraint_id: Some(c.id.0),
                kind: DimensionType::LinearHorizontal,
                value: distance,
                label: format!("H {:.2} mm", distance),
                p1: pa,
                p2: pb,
                dim_start,
                dim_end,
                text_pos,
                normal: vec2(0.0, 1.0),
                is_driving: true,
            })
        }

        ConstraintType::VerticalDistance {
            pt_a,
            pt_b,
            distance,
        } => {
            let pa = sketch.get_point(pt_a)?.pos;
            let pb = sketch.get_point(pt_b)?.pos;
            let offset_x = offset_dist.copysign(1.0);

            let dim_start = vec2(pa.x.max(pb.x) + offset_x, pa.y);
            let dim_end = vec2(pa.x.max(pb.x) + offset_x, pb.y);
            let text_pos = (dim_start + dim_end) * 0.5;

            Some(DimensionCallout {
                constraint_id: Some(c.id.0),
                kind: DimensionType::LinearVertical,
                value: distance,
                label: format!("V {:.2} mm", distance),
                p1: pa,
                p2: pb,
                dim_start,
                dim_end,
                text_pos,
                normal: vec2(1.0, 0.0),
                is_driving: true,
            })
        }

        ConstraintType::Angle {
            line_a,
            line_b,
            angle_rad,
        } => {
            let (la_start, la_end) = match sketch.entities.get(line_a.0)? {
                SketchEntity::Line(l) => (l.start, l.end),
                _ => return None,
            };
            let (lb_start, lb_end) = match sketch.entities.get(line_b.0)? {
                SketchEntity::Line(l) => (l.start, l.end),
                _ => return None,
            };

            let pa1 = sketch.get_point(la_start)?.pos;
            let pa2 = sketch.get_point(la_end)?.pos;
            let pb1 = sketch.get_point(lb_start)?.pos;
            let pb2 = sketch.get_point(lb_end)?.pos;

            let mid_a = (pa1 + pa2) * 0.5;
            let mid_b = (pb1 + pb2) * 0.5;
            let text_pos = (mid_a + mid_b) * 0.5;
            let deg = angle_rad.to_degrees();

            Some(DimensionCallout {
                constraint_id: Some(c.id.0),
                kind: DimensionType::Angular,
                value: deg,
                label: format!("{:.1}°", deg),
                p1: mid_a,
                p2: mid_b,
                dim_start: mid_a,
                dim_end: mid_b,
                text_pos,
                normal: vec2(0.0, 1.0),
                is_driving: true,
            })
        }

        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sketch::constraints::ConstraintId;
    use crate::sketch::workplane::Workplane;

    #[test]
    fn test_extract_dimension_callouts() {
        let mut sketch = Sketch::new("DimSketch", Workplane::xy());
        let p1 = sketch.add_point(vec2(0.0, 0.0));
        let p2 = sketch.add_point(vec2(50.0, 0.0));
        sketch.add_line(p1, p2);

        sketch.constraints.push(SketchConstraint::new(
            ConstraintId(0),
            ConstraintType::Distance {
                pt_a: p1,
                pt_b: p2,
                distance: 50.0,
            },
        ));

        let callouts = extract_dimension_callouts(&sketch, 5.0);
        assert_eq!(callouts.len(), 1);
        assert_eq!(callouts[0].label, "50.00 mm");
        assert_eq!(callouts[0].kind, DimensionType::LinearAligned);
        assert!((callouts[0].text_pos.y - 5.0).abs() < 1e-4);
    }
}

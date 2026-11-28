//! 2D Geometric & Dimensional Constraints for CAD Sketching.

use super::entities::{SketchEntityId, SketchPointId};
use core_math::Vec2;
use serde::{Deserialize, Serialize};

/// Type-safe identifier for sketch constraints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ConstraintId(pub usize);

/// Specific geometric or dimensional condition enforced between sketch elements.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ConstraintType {
    /// Forces two points to share the exact same $(u, v)$ coordinate.
    Coincident {
        pt_a: SketchPointId,
        pt_b: SketchPointId,
    },
    /// Forces a line segment (between pt_a and pt_b) to remain parallel to the U axis ($v_a = v_b$).
    Horizontal {
        pt_a: SketchPointId,
        pt_b: SketchPointId,
    },
    /// Forces a line segment (between pt_a and pt_b) to remain parallel to the V axis ($u_a = u_b$).
    Vertical {
        pt_a: SketchPointId,
        pt_b: SketchPointId,
    },
    /// Dimensional constraint: exact Euclidean distance between two points ($||p_b - p_a|| = d$).
    Distance {
        pt_a: SketchPointId,
        pt_b: SketchPointId,
        distance: f32,
    },
    /// Dimensional constraint: exact horizontal offset ($p_b.x - p_a.x = \pm d$).
    HorizontalDistance {
        pt_a: SketchPointId,
        pt_b: SketchPointId,
        distance: f32,
    },
    /// Dimensional constraint: exact vertical offset ($p_b.y - p_a.y = \pm d$).
    VerticalDistance {
        pt_a: SketchPointId,
        pt_b: SketchPointId,
        distance: f32,
    },
    /// Forces two line segments to remain parallel.
    Parallel {
        line_a: SketchEntityId,
        line_b: SketchEntityId,
    },
    /// Forces two line segments to remain perpendicular ($90^\circ$).
    Perpendicular {
        line_a: SketchEntityId,
        line_b: SketchEntityId,
    },
    /// Forces two line segments to have identical lengths.
    EqualLength {
        line_a: SketchEntityId,
        line_b: SketchEntityId,
    },
    /// Forces two circles or arcs to have identical radii.
    EqualRadius {
        entity_a: SketchEntityId,
        entity_b: SketchEntityId,
    },
    /// Forces two circles or arcs to share the exact same center point.
    Concentric {
        entity_a: SketchEntityId,
        entity_b: SketchEntityId,
    },
    /// Constrains a point to the midpoint between two line endpoints.
    Midpoint {
        point: SketchPointId,
        line: SketchEntityId,
    },
    /// Constrains a point to lie collinear along an infinite line.
    PointOnLine {
        point: SketchPointId,
        line: SketchEntityId,
    },
    /// Dimensional constraint: angle in radians between two lines.
    Angle {
        line_a: SketchEntityId,
        line_b: SketchEntityId,
        angle_rad: f32,
    },
    /// Locks a point to an absolute coordinate $(u, v)$.
    FixedPoint { point: SketchPointId, target: Vec2 },
}

/// A registered constraint within a sketch.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SketchConstraint {
    pub id: ConstraintId,
    pub kind: ConstraintType,
    pub is_active: bool,
}

impl SketchConstraint {
    pub fn new(id: ConstraintId, kind: ConstraintType) -> Self {
        Self {
            id,
            kind,
            is_active: true,
        }
    }

    /// Whether this is a dimensional constraint (defining distances, angles, or radii with live badges).
    pub fn is_dimensional(&self) -> bool {
        matches!(
            self.kind,
            ConstraintType::Distance { .. }
                | ConstraintType::HorizontalDistance { .. }
                | ConstraintType::VerticalDistance { .. }
                | ConstraintType::Angle { .. }
        )
    }
}

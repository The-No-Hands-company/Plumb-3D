//! 2D CAD Sketch Entities (Points, Segments, Circles, Arcs, and Profiles).

use super::constraints::SketchConstraint;
use super::workplane::Workplane;
use core_math::{vec2, Vec2};
use serde::{Deserialize, Serialize};

/// Type-safe identifier for points in a sketch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SketchPointId(pub usize);

/// Type-safe identifier for geometric entities in a sketch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SketchEntityId(pub usize);

/// A 2D point on a sketch workplane.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SketchPoint {
    pub id: SketchPointId,
    pub pos: Vec2,
    /// Locked to absolute coordinates (cannot be moved by constraints).
    pub is_fixed: bool,
    /// Auxiliary construction geometry (not exported to solid faces).
    pub is_construction: bool,
}

impl SketchPoint {
    pub fn new(id: SketchPointId, pos: Vec2) -> Self {
        Self {
            id,
            pos,
            is_fixed: false,
            is_construction: false,
        }
    }

    pub fn fixed(id: SketchPointId, pos: Vec2) -> Self {
        Self {
            id,
            pos,
            is_fixed: true,
            is_construction: false,
        }
    }
}

/// 2D straight line segment between two points.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SketchLine {
    pub start: SketchPointId,
    pub end: SketchPointId,
    pub is_construction: bool,
}

/// 2D circular arc defined by a center point, start point, and end point.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SketchArc {
    pub center: SketchPointId,
    pub start: SketchPointId,
    pub end: SketchPointId,
    pub radius: f32,
    pub ccw: bool,
    pub is_construction: bool,
}

/// 2D full circle defined by center point and radius.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SketchCircle {
    pub center: SketchPointId,
    pub radius: f32,
    pub is_construction: bool,
}

/// Geometric entity on a 2D sketch plane.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SketchEntity {
    Line(SketchLine),
    Arc(SketchArc),
    Circle(SketchCircle),
}

impl SketchEntity {
    pub fn is_construction(&self) -> bool {
        match self {
            Self::Line(l) => l.is_construction,
            Self::Arc(a) => a.is_construction,
            Self::Circle(c) => c.is_construction,
        }
    }
}

/// Complete 2D parametric CAD sketch hosted on a 3D workplane.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sketch {
    pub name: String,
    pub workplane: Workplane,
    pub points: Vec<SketchPoint>,
    pub entities: Vec<SketchEntity>,
    pub constraints: Vec<SketchConstraint>,
    next_point_id: usize,
    next_entity_id: usize,
}

impl Default for Sketch {
    fn default() -> Self {
        Self::new("Sketch_1", Workplane::xy())
    }
}

impl Sketch {
    pub fn new(name: impl Into<String>, workplane: Workplane) -> Self {
        Self {
            name: name.into(),
            workplane,
            points: Vec::new(),
            entities: Vec::new(),
            constraints: Vec::new(),
            next_point_id: 0,
            next_entity_id: 0,
        }
    }

    /// Adds a new point to the sketch and returns its ID.
    pub fn add_point(&mut self, pos: Vec2) -> SketchPointId {
        let id = SketchPointId(self.next_point_id);
        self.next_point_id += 1;
        self.points.push(SketchPoint::new(id, pos));
        id
    }

    /// Adds a fixed anchor point to the sketch.
    pub fn add_fixed_point(&mut self, pos: Vec2) -> SketchPointId {
        let id = SketchPointId(self.next_point_id);
        self.next_point_id += 1;
        self.points.push(SketchPoint::fixed(id, pos));
        id
    }

    /// Retrieves point by ID.
    pub fn get_point(&self, id: SketchPointId) -> Option<&SketchPoint> {
        self.points.iter().find(|p| p.id == id)
    }

    /// Retrieves mutable point by ID.
    pub fn get_point_mut(&mut self, id: SketchPointId) -> Option<&mut SketchPoint> {
        self.points.iter_mut().find(|p| p.id == id)
    }

    /// Adds a line segment between two existing points.
    pub fn add_line(&mut self, start: SketchPointId, end: SketchPointId) -> SketchEntityId {
        let id = SketchEntityId(self.next_entity_id);
        self.next_entity_id += 1;
        self.entities.push(SketchEntity::Line(SketchLine {
            start,
            end,
            is_construction: false,
        }));
        id
    }

    /// Adds a circle with a center point and numeric radius.
    pub fn add_circle(&mut self, center: SketchPointId, radius: f32) -> SketchEntityId {
        let id = SketchEntityId(self.next_entity_id);
        self.next_entity_id += 1;
        self.entities.push(SketchEntity::Circle(SketchCircle {
            center,
            radius: radius.abs(),
            is_construction: false,
        }));
        id
    }

    /// Adds a 3-point circular arc.
    pub fn add_arc(
        &mut self,
        center: SketchPointId,
        start: SketchPointId,
        end: SketchPointId,
        radius: f32,
        ccw: bool,
    ) -> SketchEntityId {
        let id = SketchEntityId(self.next_entity_id);
        self.next_entity_id += 1;
        self.entities.push(SketchEntity::Arc(SketchArc {
            center,
            start,
            end,
            radius: radius.abs(),
            ccw,
            is_construction: false,
        }));
        id
    }

    /// Convenience helper to create a 2-corner rectangle (adds 4 points and 4 lines).
    pub fn add_rectangle(&mut self, p_min: Vec2, p_max: Vec2) -> [SketchEntityId; 4] {
        let p0 = self.add_point(vec2(p_min.x, p_min.y));
        let p1 = self.add_point(vec2(p_max.x, p_min.y));
        let p2 = self.add_point(vec2(p_max.x, p_max.y));
        let p3 = self.add_point(vec2(p_min.x, p_max.y));

        let l0 = self.add_line(p0, p1);
        let l1 = self.add_line(p1, p2);
        let l2 = self.add_line(p2, p3);
        let l3 = self.add_line(p3, p0);

        [l0, l1, l2, l3]
    }

    /// Computes the 2D bounding box of all sketch points.
    pub fn bounding_box(&self) -> (Vec2, Vec2) {
        if self.points.is_empty() {
            return (Vec2::ZERO, Vec2::ZERO);
        }
        let mut min = self.points[0].pos;
        let mut max = self.points[0].pos;
        for p in &self.points[1..] {
            min = min.min(p.pos);
            max = max.max(p.pos);
        }
        (min, max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sketch_entities_creation() {
        let mut sketch = Sketch::new("BaseSketch", Workplane::xy());
        let p1 = sketch.add_point(vec2(0.0, 0.0));
        let p2 = sketch.add_point(vec2(10.0, 0.0));
        let line_id = sketch.add_line(p1, p2);

        assert_eq!(sketch.points.len(), 2);
        assert_eq!(sketch.entities.len(), 1);
        assert_eq!(line_id, SketchEntityId(0));

        let rect_lines = sketch.add_rectangle(vec2(0.0, 0.0), vec2(50.0, 30.0));
        assert_eq!(rect_lines.len(), 4);
        assert_eq!(sketch.points.len(), 6);
    }
}

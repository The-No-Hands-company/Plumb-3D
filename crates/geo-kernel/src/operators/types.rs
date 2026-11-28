use crate::mesh::{EdgeKey, VertexKey};
use core_math::Vec3;

/// Target anchor for a knife / multi-cut point (either an existing vertex or a point along an edge)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CutPoint {
    Vertex(VertexKey),
    Edge(EdgeKey, f32), // edge, t (0.01..0.99)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuadMethod {
    Beauty,
    ShortestDiagonal,
    Fixed,
    Alternate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NgonMethod {
    Beauty,
    EarClipping,
    Fan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymmetrizeDirection {
    PositiveToNegative,
    NegativeToPositive,
}

pub fn triangle_min_angle(a: Vec3, b: Vec3, c: Vec3) -> f32 {
    let ab = (b - a).normalize();
    let bc = (c - b).normalize();
    let ca = (a - c).normalize();

    let cos_a = (-ca).dot(ab).clamp(-1.0, 1.0);
    let cos_b = (-ab).dot(bc).clamp(-1.0, 1.0);
    let cos_c = (-bc).dot(ca).clamp(-1.0, 1.0);

    cos_a.acos().min(cos_b.acos()).min(cos_c.acos())
}

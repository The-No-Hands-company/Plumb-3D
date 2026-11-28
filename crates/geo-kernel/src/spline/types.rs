use core_math::{vec3, Vec3};
use std::f32::consts::PI;

/// Tangent handle constraint behavior for Bézier control points (Blender / Rhino / Illustrator).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HandleType {
    /// In and out handles are independently controlled without angular constraints.
    #[default]
    Free,
    /// In and out handles remain collinear (180° apart) ensuring C1 continuity, but can have different lengths.
    Aligned,
    /// Handle points directly toward the adjacent control point (piecewise linear/chamfer).
    Vector,
    /// Automatically calculated smooth curvature handles (Catmull-Rom tangent estimation).
    Auto,
}

/// A control point for a 3D Bézier curve.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BezierPoint {
    /// Center anchor position
    pub position: Vec3,
    /// Tangent handle before the point (relative to position)
    pub handle_in: Vec3,
    /// Tangent handle after the point (relative to position)
    pub handle_out: Vec3,
    /// Handle constraint behavior
    pub handle_type: HandleType,
}

impl BezierPoint {
    pub fn new(position: Vec3, handle_in: Vec3, handle_out: Vec3) -> Self {
        Self {
            position,
            handle_in,
            handle_out,
            handle_type: HandleType::Free,
        }
    }

    pub fn with_type(
        position: Vec3,
        handle_in: Vec3,
        handle_out: Vec3,
        handle_type: HandleType,
    ) -> Self {
        Self {
            position,
            handle_in,
            handle_out,
            handle_type,
        }
    }

    pub fn sharp(position: Vec3) -> Self {
        Self {
            position,
            handle_in: Vec3::ZERO,
            handle_out: Vec3::ZERO,
            handle_type: HandleType::Vector,
        }
    }

    pub fn smooth(position: Vec3, tangent: Vec3) -> Self {
        Self {
            position,
            handle_in: -tangent,
            handle_out: tangent,
            handle_type: HandleType::Aligned,
        }
    }

    /// Adjusts the in-handle and maintains collinear alignment if handle_type is `Aligned`.
    pub fn adjust_handle_in(&mut self, new_h_in: Vec3) {
        self.handle_in = new_h_in;
        if self.handle_type == HandleType::Aligned {
            let len_out = self.handle_out.length();
            let dir = new_h_in.normalize_or_zero();
            if dir != Vec3::ZERO {
                self.handle_out = -dir * len_out;
            }
        }
    }

    /// Adjusts the out-handle and maintains collinear alignment if handle_type is `Aligned`.
    pub fn adjust_handle_out(&mut self, new_h_out: Vec3) {
        self.handle_out = new_h_out;
        if self.handle_type == HandleType::Aligned {
            let len_in = self.handle_in.length();
            let dir = new_h_out.normalize_or_zero();
            if dir != Vec3::ZERO {
                self.handle_in = -dir * len_in;
            }
        }
    }

    /// Constructs two cubic Bézier control points from an exact quadratic Bézier curve (Q0, Q1, Q2)
    /// via CAD degree elevation without altering the underlying parabolic curve shape.
    pub fn from_quadratic_bezier(q0: Vec3, q1: Vec3, q2: Vec3) -> (Self, Self) {
        let p0 = BezierPoint::new(q0, Vec3::ZERO, (q1 - q0) * (2.0 / 3.0));
        let p1 = BezierPoint::new(q2, (q1 - q2) * (2.0 / 3.0), Vec3::ZERO);
        (p0, p1)
    }
}

/// 3D Parametric Bézier Spline Curve (Maya NURBS / 3ds Max Editable Spline)
#[derive(Debug, Clone, Default)]
pub struct Spline {
    pub points: Vec<BezierPoint>,
    pub is_closed: bool,
}

impl Spline {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_points(points: Vec<BezierPoint>, is_closed: bool) -> Self {
        Self { points, is_closed }
    }

    pub fn add_point(&mut self, point: BezierPoint) {
        self.points.push(point);
    }

    /// Automatically updates tangent handles for points set to `HandleType::Auto` using Catmull-Rom tangent estimation.
    pub fn update_auto_handles(&mut self) {
        let n = self.points.len();
        if n < 2 {
            return;
        }
        let is_closed = self.is_closed;

        for i in 0..n {
            if self.points[i].handle_type != HandleType::Auto {
                continue;
            }

            let prev_p = if i == 0 {
                if is_closed {
                    self.points[n - 1].position
                } else {
                    self.points[0].position
                }
            } else {
                self.points[i - 1].position
            };

            let next_p = if i == n - 1 {
                if is_closed {
                    self.points[0].position
                } else {
                    self.points[n - 1].position
                }
            } else {
                self.points[i + 1].position
            };

            // Standard cubic Hermite to Bézier handle length: (next - prev) / 6.0
            let tangent = (next_p - prev_p) / 6.0;
            self.points[i].handle_in = -tangent;
            self.points[i].handle_out = tangent;
        }
    }

    /// Sets handle type for point `idx` and re-evaluates handles.
    pub fn set_handle_type(&mut self, idx: usize, handle_type: HandleType) {
        if idx >= self.points.len() {
            return;
        }
        self.points[idx].handle_type = handle_type;
        match handle_type {
            HandleType::Free => {}
            HandleType::Aligned => {
                let len_in = self.points[idx].handle_in.length();
                let dir = self.points[idx].handle_out.normalize_or_zero();
                if dir != Vec3::ZERO {
                    self.points[idx].handle_in = -dir * len_in;
                }
            }
            HandleType::Vector => {
                let n = self.points.len();
                let prev_idx = if idx == 0 {
                    if self.is_closed {
                        n - 1
                    } else {
                        0
                    }
                } else {
                    idx - 1
                };
                let next_idx = if idx == n - 1 {
                    if self.is_closed {
                        0
                    } else {
                        n - 1
                    }
                } else {
                    idx + 1
                };
                self.points[idx].handle_in =
                    (self.points[prev_idx].position - self.points[idx].position) / 3.0;
                self.points[idx].handle_out =
                    (self.points[next_idx].position - self.points[idx].position) / 3.0;
            }
            HandleType::Auto => {
                self.update_auto_handles();
            }
        }
    }

    /// Constructs a smooth circular planar Bézier spline in the XZ plane.
    /// Uses 4 cubic Bézier segments with tangent handle length = radius * 4/3 * tan(PI / 8) ≈ 0.55228 * radius.
    pub fn circle(radius: f32, center: Vec3) -> Self {
        let k = radius * 0.552_284_8; // Optimal circle approximation factor
        let points = vec![
            BezierPoint::new(
                center + vec3(radius, 0.0, 0.0),
                vec3(0.0, 0.0, -k),
                vec3(0.0, 0.0, k),
            ),
            BezierPoint::new(
                center + vec3(0.0, 0.0, radius),
                vec3(k, 0.0, 0.0),
                vec3(-k, 0.0, 0.0),
            ),
            BezierPoint::new(
                center + vec3(-radius, 0.0, 0.0),
                vec3(0.0, 0.0, k),
                vec3(0.0, 0.0, -k),
            ),
            BezierPoint::new(
                center + vec3(0.0, 0.0, -radius),
                vec3(-k, 0.0, 0.0),
                vec3(k, 0.0, 0.0),
            ),
        ];

        Self {
            points,
            is_closed: true,
        }
    }

    /// Constructs a 3D helical spiral curve ascending along the Y axis.
    pub fn helix(radius: f32, height: f32, turns: f32, points_per_turn: usize) -> Self {
        let ppt = points_per_turn.max(4);
        let total_points = (turns * ppt as f32).ceil() as usize + 1;
        let mut points = Vec::with_capacity(total_points);

        let d_theta = 2.0 * PI / ppt as f32;
        let d_y = height / ((total_points - 1) as f32);
        let tan_mag = radius * d_theta / 3.0;

        for i in 0..total_points {
            let theta = i as f32 * d_theta;
            let y = i as f32 * d_y;
            let pos = vec3(radius * theta.cos(), y, radius * theta.sin());
            let tangent = vec3(-theta.sin() * tan_mag, d_y / 3.0, theta.cos() * tan_mag);

            points.push(BezierPoint::new(pos, -tangent, tangent));
        }

        Self {
            points,
            is_closed: false,
        }
    }
}

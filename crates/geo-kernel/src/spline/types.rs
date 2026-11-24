use core_math::{vec3, Vec3};
use std::f32::consts::PI;

/// A control point for a 3D Bézier curve.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BezierPoint {
    /// Center anchor position
    pub position: Vec3,
    /// Tangent handle before the point (relative to position)
    pub handle_in: Vec3,
    /// Tangent handle after the point (relative to position)
    pub handle_out: Vec3,
}

impl BezierPoint {
    pub fn new(position: Vec3, handle_in: Vec3, handle_out: Vec3) -> Self {
        Self {
            position,
            handle_in,
            handle_out,
        }
    }

    pub fn sharp(position: Vec3) -> Self {
        Self {
            position,
            handle_in: Vec3::ZERO,
            handle_out: Vec3::ZERO,
        }
    }

    pub fn smooth(position: Vec3, tangent: Vec3) -> Self {
        Self {
            position,
            handle_in: -tangent,
            handle_out: tangent,
        }
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

    /// Constructs a smooth circular planar Bézier spline in the XZ plane.
    /// Uses 4 cubic Bézier segments with tangent handle length = radius * 4/3 * tan(PI / 8) ≈ 0.55228 * radius.
    pub fn circle(radius: f32, center: Vec3) -> Self {
        let k = radius * 0.55228475; // Optimal circle approximation factor
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
        // Tangent handle magnitude along circular arc
        let tan_mag = radius * d_theta / 3.0;

        for i in 0..total_points {
            let theta = i as f32 * d_theta;
            let y = i as f32 * d_y;
            let pos = vec3(radius * theta.cos(), y, radius * theta.sin());

            // Velocity / tangent vector in cylindrical coordinates: (-R*sin(t), dy/dt, R*cos(t))
            let tangent = vec3(-theta.sin() * tan_mag, d_y / 3.0, theta.cos() * tan_mag);

            points.push(BezierPoint::new(pos, -tangent, tangent));
        }

        Self {
            points,
            is_closed: false,
        }
    }
}

use crate::nodes::traits::GeoNodeOp;
use crate::types::*;
use core_math::Vec3;
use geo_kernel::Spline;

#[derive(Debug, Clone)]
pub struct SplineCircleNode;

impl GeoNodeOp for SplineCircleNode {
    fn name(&self) -> &str {
        "Spline Circle"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::CurveOps
    }

    fn inputs(&self) -> Vec<NodeSocket> {
        vec![
            NodeSocket::float("Radius", 1.5),
            NodeSocket::float("Pipe Radius", 0.15),
            NodeSocket::int("Resolution", 16),
        ]
    }

    fn outputs(&self) -> Vec<NodeSocket> {
        vec![NodeSocket::geometry("Mesh")]
    }

    fn execute(&self, inputs: &[SocketValue]) -> Result<Vec<SocketValue>, NodeGraphError> {
        let radius = inputs
            .first()
            .map(|v| v.as_float())
            .unwrap_or(1.5)
            .max(0.01);
        let pipe_radius = inputs
            .get(1)
            .map(|v| v.as_float())
            .unwrap_or(0.15)
            .max(0.005);
        let res = inputs.get(2).map(|v| v.as_int()).unwrap_or(16).clamp(3, 64) as usize;

        let spline = Spline::circle(radius, Vec3::ZERO);
        let mesh = spline
            .to_pipe_mesh(pipe_radius, res, 16)
            .map_err(|e| NodeGraphError::ExecutionError(e.to_string()))?;

        Ok(vec![SocketValue::Geometry(Some(mesh))])
    }

    fn clone_box(&self) -> Box<dyn GeoNodeOp> {
        Box::new(self.clone())
    }
}

#[derive(Debug, Clone)]
pub struct SplineSpiralNode;

impl GeoNodeOp for SplineSpiralNode {
    fn name(&self) -> &str {
        "Spline Spiral"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::CurveOps
    }

    fn inputs(&self) -> Vec<NodeSocket> {
        vec![
            NodeSocket::float("Radius", 1.0),
            NodeSocket::float("Height", 3.0),
            NodeSocket::float("Revolutions", 3.0),
            NodeSocket::float("Pipe Radius", 0.1),
        ]
    }

    fn outputs(&self) -> Vec<NodeSocket> {
        vec![NodeSocket::geometry("Mesh")]
    }

    fn execute(&self, inputs: &[SocketValue]) -> Result<Vec<SocketValue>, NodeGraphError> {
        let radius = inputs
            .first()
            .map(|v| v.as_float())
            .unwrap_or(1.0)
            .max(0.01);
        let height = inputs.get(1).map(|v| v.as_float()).unwrap_or(3.0).max(0.01);
        let revs = inputs.get(2).map(|v| v.as_float()).unwrap_or(3.0).max(0.5);
        let pipe_r = inputs
            .get(3)
            .map(|v| v.as_float())
            .unwrap_or(0.1)
            .max(0.005);

        let spline = Spline::helix(radius, height, revs, 24);
        let mesh = spline
            .to_pipe_mesh(pipe_r, 12, 16)
            .map_err(|e| NodeGraphError::ExecutionError(e.to_string()))?;

        Ok(vec![SocketValue::Geometry(Some(mesh))])
    }

    fn clone_box(&self) -> Box<dyn GeoNodeOp> {
        Box::new(self.clone())
    }
}

#[derive(Debug, Clone)]
pub struct CurveFilletNode;

impl GeoNodeOp for CurveFilletNode {
    fn name(&self) -> &str {
        "Fillet Curve"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::CurveOps
    }

    fn inputs(&self) -> Vec<NodeSocket> {
        vec![
            NodeSocket::float("Fillet Radius", 0.3),
            NodeSocket::float("Pipe Radius", 0.1),
        ]
    }

    fn outputs(&self) -> Vec<NodeSocket> {
        vec![NodeSocket::geometry("Mesh")]
    }

    fn execute(&self, inputs: &[SocketValue]) -> Result<Vec<SocketValue>, NodeGraphError> {
        let radius = inputs
            .first()
            .map(|v| v.as_float())
            .unwrap_or(0.3)
            .max(0.001);
        let pipe_r = inputs
            .get(1)
            .map(|v| v.as_float())
            .unwrap_or(0.1)
            .max(0.005);

        let p0 = geo_kernel::BezierPoint::sharp(Vec3::new(0.0, 0.0, 0.0));
        let p1 = geo_kernel::BezierPoint::sharp(Vec3::new(2.0, 0.0, 0.0));
        let p2 = geo_kernel::BezierPoint::sharp(Vec3::new(2.0, 2.0, 0.0));
        let spline = Spline::with_points(vec![p0, p1, p2], false);
        let filleted = spline.fillet_all_corners(radius);
        let mesh = filleted
            .to_pipe_mesh(pipe_r, 8, 16)
            .map_err(|e| NodeGraphError::ExecutionError(e.to_string()))?;

        Ok(vec![SocketValue::Geometry(Some(mesh))])
    }

    fn clone_box(&self) -> Box<dyn GeoNodeOp> {
        Box::new(self.clone())
    }
}

#[derive(Debug, Clone)]
pub struct CurveRevolveNode;

impl GeoNodeOp for CurveRevolveNode {
    fn name(&self) -> &str {
        "Revolve Spline (Lathe)"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::CurveOps
    }

    fn inputs(&self) -> Vec<NodeSocket> {
        vec![
            NodeSocket::float("Radius", 1.0),
            NodeSocket::float("Height", 2.0),
            NodeSocket::int("Segments", 16),
            NodeSocket::float("Angle", 360.0),
        ]
    }

    fn outputs(&self) -> Vec<NodeSocket> {
        vec![NodeSocket::geometry("Mesh")]
    }

    fn execute(&self, inputs: &[SocketValue]) -> Result<Vec<SocketValue>, NodeGraphError> {
        let radius = inputs
            .first()
            .map(|v| v.as_float())
            .unwrap_or(1.0)
            .max(0.01);
        let height = inputs.get(1).map(|v| v.as_float()).unwrap_or(2.0).max(0.01);
        let segs = inputs
            .get(2)
            .map(|v| v.as_int())
            .unwrap_or(16)
            .clamp(3, 128) as u32;
        let angle = inputs
            .get(3)
            .map(|v| v.as_float())
            .unwrap_or(360.0)
            .clamp(1.0, 360.0);

        let p0 = geo_kernel::BezierPoint::sharp(Vec3::new(0.0, 0.0, 0.0));
        let p1 = geo_kernel::BezierPoint::sharp(Vec3::new(radius, 0.0, 0.0));
        let p2 = geo_kernel::BezierPoint::sharp(Vec3::new(radius * 0.7, height * 0.5, 0.0));
        let p3 = geo_kernel::BezierPoint::sharp(Vec3::new(radius, height, 0.0));
        let spline = Spline::with_points(vec![p0, p1, p2, p3], false);

        let mesh = spline
            .lathe_spline(12, segs, angle.to_radians())
            .map_err(|e| NodeGraphError::ExecutionError(e.to_string()))?;

        Ok(vec![SocketValue::Geometry(Some(mesh))])
    }

    fn clone_box(&self) -> Box<dyn GeoNodeOp> {
        Box::new(self.clone())
    }
}

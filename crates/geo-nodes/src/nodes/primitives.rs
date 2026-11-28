use crate::nodes::traits::GeoNodeOp;
use crate::types::*;
use geo_kernel::Primitives;

#[derive(Debug, Clone)]
pub struct CubeNode;

impl GeoNodeOp for CubeNode {
    fn name(&self) -> &str {
        "Cube"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::Primitives
    }

    fn inputs(&self) -> Vec<NodeSocket> {
        vec![NodeSocket::float("Size", 2.0)]
    }

    fn outputs(&self) -> Vec<NodeSocket> {
        vec![NodeSocket::geometry("Mesh")]
    }

    fn execute(&self, inputs: &[SocketValue]) -> Result<Vec<SocketValue>, NodeGraphError> {
        let size = inputs
            .first()
            .map(|v| v.as_float())
            .unwrap_or(2.0)
            .max(0.01);
        let mesh = Primitives::create_cube(size);
        Ok(vec![SocketValue::Geometry(Some(mesh))])
    }

    fn clone_box(&self) -> Box<dyn GeoNodeOp> {
        Box::new(self.clone())
    }
}

#[derive(Debug, Clone)]
pub struct SphereNode;

impl GeoNodeOp for SphereNode {
    fn name(&self) -> &str {
        "UV Sphere"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::Primitives
    }

    fn inputs(&self) -> Vec<NodeSocket> {
        vec![
            NodeSocket::float("Radius", 1.0),
            NodeSocket::int("Rings", 16),
            NodeSocket::int("Sectors", 16),
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
        let rings = inputs.get(1).map(|v| v.as_int()).unwrap_or(16).clamp(3, 64) as u32;
        let sectors = inputs.get(2).map(|v| v.as_int()).unwrap_or(16).clamp(3, 64) as u32;
        let mesh = Primitives::create_uv_sphere(radius, sectors, rings);
        Ok(vec![SocketValue::Geometry(Some(mesh))])
    }

    fn clone_box(&self) -> Box<dyn GeoNodeOp> {
        Box::new(self.clone())
    }
}

#[derive(Debug, Clone)]
pub struct CylinderNode;

impl GeoNodeOp for CylinderNode {
    fn name(&self) -> &str {
        "Cylinder"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::Primitives
    }

    fn inputs(&self) -> Vec<NodeSocket> {
        vec![
            NodeSocket::float("Radius", 1.0),
            NodeSocket::float("Height", 2.0),
            NodeSocket::int("Segments", 16),
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
        let segments = inputs.get(2).map(|v| v.as_int()).unwrap_or(16).clamp(3, 64) as u32;
        let mesh = Primitives::create_cylinder(radius, height, segments);
        Ok(vec![SocketValue::Geometry(Some(mesh))])
    }

    fn clone_box(&self) -> Box<dyn GeoNodeOp> {
        Box::new(self.clone())
    }
}

#[derive(Debug, Clone)]
pub struct PlaneNode;

impl GeoNodeOp for PlaneNode {
    fn name(&self) -> &str {
        "Grid Plane"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::Primitives
    }

    fn inputs(&self) -> Vec<NodeSocket> {
        vec![
            NodeSocket::float("Width", 4.0),
            NodeSocket::float("Length", 4.0),
            NodeSocket::int("Subdiv X", 4),
            NodeSocket::int("Subdiv Y", 4),
        ]
    }

    fn outputs(&self) -> Vec<NodeSocket> {
        vec![NodeSocket::geometry("Mesh")]
    }

    fn execute(&self, inputs: &[SocketValue]) -> Result<Vec<SocketValue>, NodeGraphError> {
        let width = inputs
            .first()
            .map(|v| v.as_float())
            .unwrap_or(4.0)
            .max(0.01);
        let length = inputs.get(1).map(|v| v.as_float()).unwrap_or(4.0).max(0.01);
        let sub_x = inputs.get(2).map(|v| v.as_int()).unwrap_or(4).clamp(1, 64) as u32;
        let sub_y = inputs.get(3).map(|v| v.as_int()).unwrap_or(4).clamp(1, 64) as u32;
        let mesh = Primitives::create_plane(width, length, sub_x, sub_y);
        Ok(vec![SocketValue::Geometry(Some(mesh))])
    }

    fn clone_box(&self) -> Box<dyn GeoNodeOp> {
        Box::new(self.clone())
    }
}

#[derive(Debug, Clone)]
pub struct TorusNode;

impl GeoNodeOp for TorusNode {
    fn name(&self) -> &str {
        "Torus"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::Primitives
    }

    fn inputs(&self) -> Vec<NodeSocket> {
        vec![
            NodeSocket::float("Major Radius", 1.5),
            NodeSocket::float("Minor Radius", 0.4),
            NodeSocket::int("Major Segments", 24),
            NodeSocket::int("Minor Segments", 16),
        ]
    }

    fn outputs(&self) -> Vec<NodeSocket> {
        vec![NodeSocket::geometry("Mesh")]
    }

    fn execute(&self, inputs: &[SocketValue]) -> Result<Vec<SocketValue>, NodeGraphError> {
        let major = inputs
            .first()
            .map(|v| v.as_float())
            .unwrap_or(1.5)
            .max(0.01);
        let minor = inputs.get(1).map(|v| v.as_float()).unwrap_or(0.4).max(0.01);
        let maj_segs = inputs.get(2).map(|v| v.as_int()).unwrap_or(24).clamp(3, 64) as u32;
        let min_segs = inputs.get(3).map(|v| v.as_int()).unwrap_or(16).clamp(3, 64) as u32;
        let mesh = Primitives::create_torus(major, minor, maj_segs, min_segs);
        Ok(vec![SocketValue::Geometry(Some(mesh))])
    }

    fn clone_box(&self) -> Box<dyn GeoNodeOp> {
        Box::new(self.clone())
    }
}

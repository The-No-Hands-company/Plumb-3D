use crate::nodes::traits::GeoNodeOp;
use crate::types::*;
use core_math::{Mat4, Vec3};
use std::any::Any;

/// Joins two geometry streams into a single composite mesh
#[derive(Debug, Clone, Default)]
pub struct JoinGeometryNode;

impl GeoNodeOp for JoinGeometryNode {
    fn name(&self) -> &str {
        "Join Geometry"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::GeometryOps
    }

    fn inputs(&self) -> Vec<NodeSocket> {
        vec![
            NodeSocket::geometry("Geometry A"),
            NodeSocket::geometry("Geometry B"),
        ]
    }

    fn outputs(&self) -> Vec<NodeSocket> {
        vec![NodeSocket::geometry("Geometry")]
    }

    fn execute(&self, inputs: &[SocketValue]) -> Result<Vec<SocketValue>, NodeGraphError> {
        let geom_a = inputs.get(0).and_then(|v| v.as_geometry());
        let geom_b = inputs.get(1).and_then(|v| v.as_geometry());

        let result = match (geom_a, geom_b) {
            (Some(a), Some(b)) => {
                let mut joined = a.clone();
                let _ = joined.join(b, Mat4::IDENTITY);
                Some(joined)
            }
            (Some(a), None) => Some(a.clone()),
            (None, Some(b)) => Some(b.clone()),
            (None, None) => None,
        };

        Ok(vec![SocketValue::Geometry(result)])
    }

    fn as_any_mut(&mut self) -> Option<&mut dyn Any> {
        Some(self)
    }

    fn clone_box(&self) -> Box<dyn GeoNodeOp> {
        Box::new(self.clone())
    }
}

/// Converts mesh vertices into a Point Cloud for scattering and instancing
#[derive(Debug, Clone, Default)]
pub struct MeshToPointsNode;

impl GeoNodeOp for MeshToPointsNode {
    fn name(&self) -> &str {
        "Mesh to Points"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::PointOps
    }

    fn inputs(&self) -> Vec<NodeSocket> {
        vec![NodeSocket::geometry("Mesh")]
    }

    fn outputs(&self) -> Vec<NodeSocket> {
        vec![NodeSocket::points("Points")]
    }

    fn execute(&self, inputs: &[SocketValue]) -> Result<Vec<SocketValue>, NodeGraphError> {
        let mesh = inputs
            .get(0)
            .and_then(|v| v.as_geometry())
            .ok_or_else(|| NodeGraphError::ExecutionError("Mesh to Points requires valid input mesh".into()))?;

        let mut points = Vec::with_capacity(mesh.vertices.len());
        for v in mesh.vertices.values() {
            points.push(PointInstance {
                position: v.position,
                normal: v.normal,
                scale: Vec3::ONE,
            });
        }

        Ok(vec![SocketValue::Points(points)])
    }

    fn as_any_mut(&mut self) -> Option<&mut dyn Any> {
        Some(self)
    }

    fn clone_box(&self) -> Box<dyn GeoNodeOp> {
        Box::new(self.clone())
    }
}

/// Sets the material slot index on all faces of a geometry
#[derive(Debug, Clone, Default)]
pub struct SetMaterialNode;

impl GeoNodeOp for SetMaterialNode {
    fn name(&self) -> &str {
        "Set Material"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::GeometryOps
    }

    fn inputs(&self) -> Vec<NodeSocket> {
        vec![
            NodeSocket::geometry("Geometry"),
            NodeSocket::int("Material Index", 0),
        ]
    }

    fn outputs(&self) -> Vec<NodeSocket> {
        vec![NodeSocket::geometry("Geometry")]
    }

    fn execute(&self, inputs: &[SocketValue]) -> Result<Vec<SocketValue>, NodeGraphError> {
        let mesh = inputs
            .get(0)
            .and_then(|v| v.as_geometry())
            .ok_or_else(|| NodeGraphError::ExecutionError("Set Material requires input geometry".into()))?;

        let mat_idx = inputs.get(1).map(|v| v.as_int().max(0) as u32).unwrap_or(0);

        let mut output = mesh.clone();
        for f in output.faces.values_mut() {
            f.material_id = mat_idx;
        }

        Ok(vec![SocketValue::Geometry(Some(output))])
    }

    fn as_any_mut(&mut self) -> Option<&mut dyn Any> {
        Some(self)
    }

    fn clone_box(&self) -> Box<dyn GeoNodeOp> {
        Box::new(self.clone())
    }
}

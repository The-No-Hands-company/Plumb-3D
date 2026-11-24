use crate::nodes::traits::GeoNodeOp;
use crate::types::*;
use core_math::Vec3;

#[derive(Debug, Clone)]
pub struct ValueNode {
    pub value: f32,
}

impl Default for ValueNode {
    fn default() -> Self {
        Self { value: 1.0 }
    }
}

impl GeoNodeOp for ValueNode {
    fn name(&self) -> &str {
        "Value"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::Inputs
    }

    fn inputs(&self) -> Vec<NodeSocket> {
        vec![NodeSocket::float("Value", self.value)]
    }

    fn outputs(&self) -> Vec<NodeSocket> {
        vec![NodeSocket::float("Value", self.value)]
    }

    fn execute(&self, inputs: &[SocketValue]) -> Result<Vec<SocketValue>, NodeGraphError> {
        let val = inputs.first().map(|v| v.as_float()).unwrap_or(self.value);
        Ok(vec![SocketValue::Float(val)])
    }

    fn clone_box(&self) -> Box<dyn GeoNodeOp> {
        Box::new(self.clone())
    }
}

#[derive(Debug, Clone)]
pub struct VectorNode {
    pub value: Vec3,
}

impl Default for VectorNode {
    fn default() -> Self {
        Self { value: Vec3::ZERO }
    }
}

impl GeoNodeOp for VectorNode {
    fn name(&self) -> &str {
        "Vector"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::Inputs
    }

    fn inputs(&self) -> Vec<NodeSocket> {
        vec![NodeSocket::vector("Vector", self.value)]
    }

    fn outputs(&self) -> Vec<NodeSocket> {
        vec![NodeSocket::vector("Vector", self.value)]
    }

    fn execute(&self, inputs: &[SocketValue]) -> Result<Vec<SocketValue>, NodeGraphError> {
        let val = inputs.first().map(|v| v.as_vec3()).unwrap_or(self.value);
        Ok(vec![SocketValue::Vector3(val)])
    }

    fn clone_box(&self) -> Box<dyn GeoNodeOp> {
        Box::new(self.clone())
    }
}

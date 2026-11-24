use crate::nodes::traits::GeoNodeOp;
use crate::types::*;
use core_math::Vec3;

#[derive(Debug, Clone)]
pub struct MathNode {
    pub operation: MathOp,
}

impl Default for MathNode {
    fn default() -> Self {
        Self {
            operation: MathOp::Add,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MathOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Power,
    Min,
    Max,
    Sin,
    Cos,
}

impl GeoNodeOp for MathNode {
    fn name(&self) -> &str {
        match self.operation {
            MathOp::Add => "Math: Add",
            MathOp::Subtract => "Math: Subtract",
            MathOp::Multiply => "Math: Multiply",
            MathOp::Divide => "Math: Divide",
            MathOp::Power => "Math: Power",
            MathOp::Min => "Math: Minimum",
            MathOp::Max => "Math: Maximum",
            MathOp::Sin => "Math: Sine",
            MathOp::Cos => "Math: Cosine",
        }
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::Math
    }

    fn inputs(&self) -> Vec<NodeSocket> {
        vec![
            NodeSocket::float("A", 0.0),
            NodeSocket::float("B", 0.0),
        ]
    }

    fn outputs(&self) -> Vec<NodeSocket> {
        vec![NodeSocket::float("Value", 0.0)]
    }

    fn execute(&self, inputs: &[SocketValue]) -> Result<Vec<SocketValue>, NodeGraphError> {
        let a = inputs.get(0).map(|v| v.as_float()).unwrap_or(0.0);
        let b = inputs.get(1).map(|v| v.as_float()).unwrap_or(0.0);

        let res = match self.operation {
            MathOp::Add => a + b,
            MathOp::Subtract => a - b,
            MathOp::Multiply => a * b,
            MathOp::Divide => if b.abs() > 1e-6 { a / b } else { 0.0 },
            MathOp::Power => a.powf(b),
            MathOp::Min => a.min(b),
            MathOp::Max => a.max(b),
            MathOp::Sin => a.sin(),
            MathOp::Cos => a.cos(),
        };

        Ok(vec![SocketValue::Float(res)])
    }

    fn clone_box(&self) -> Box<dyn GeoNodeOp> {
        Box::new(self.clone())
    }
}

#[derive(Debug, Clone)]
pub struct VectorMathNode {
    pub operation: VectorMathOp,
}

impl Default for VectorMathNode {
    fn default() -> Self {
        Self {
            operation: VectorMathOp::Add,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VectorMathOp {
    Add,
    Subtract,
    Multiply,
    Dot,
    Cross,
    Normalize,
    Length,
}

impl GeoNodeOp for VectorMathNode {
    fn name(&self) -> &str {
        match self.operation {
            VectorMathOp::Add => "Vector Math: Add",
            VectorMathOp::Subtract => "Vector Math: Subtract",
            VectorMathOp::Multiply => "Vector Math: Multiply",
            VectorMathOp::Dot => "Vector Math: Dot Product",
            VectorMathOp::Cross => "Vector Math: Cross Product",
            VectorMathOp::Normalize => "Vector Math: Normalize",
            VectorMathOp::Length => "Vector Math: Length",
        }
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::Math
    }

    fn inputs(&self) -> Vec<NodeSocket> {
        vec![
            NodeSocket::vector("A", Vec3::ZERO),
            NodeSocket::vector("B", Vec3::ZERO),
        ]
    }

    fn outputs(&self) -> Vec<NodeSocket> {
        vec![
            NodeSocket::vector("Vector", Vec3::ZERO),
            NodeSocket::float("Value", 0.0),
        ]
    }

    fn execute(&self, inputs: &[SocketValue]) -> Result<Vec<SocketValue>, NodeGraphError> {
        let a = inputs.get(0).map(|v| v.as_vec3()).unwrap_or(Vec3::ZERO);
        let b = inputs.get(1).map(|v| v.as_vec3()).unwrap_or(Vec3::ZERO);

        let (vec_res, val_res) = match self.operation {
            VectorMathOp::Add => (a + b, (a + b).length()),
            VectorMathOp::Subtract => (a - b, (a - b).length()),
            VectorMathOp::Multiply => (a * b, 0.0),
            VectorMathOp::Dot => (Vec3::ZERO, a.dot(b)),
            VectorMathOp::Cross => (a.cross(b), 0.0),
            VectorMathOp::Normalize => (a.normalize_or_zero(), 1.0),
            VectorMathOp::Length => (Vec3::ZERO, a.length()),
        };

        Ok(vec![SocketValue::Vector3(vec_res), SocketValue::Float(val_res)])
    }

    fn clone_box(&self) -> Box<dyn GeoNodeOp> {
        Box::new(self.clone())
    }
}

#[derive(Debug, Clone)]
pub struct CombineXyzNode;

impl GeoNodeOp for CombineXyzNode {
    fn name(&self) -> &str {
        "Combine XYZ"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::Math
    }

    fn inputs(&self) -> Vec<NodeSocket> {
        vec![
            NodeSocket::float("X", 0.0),
            NodeSocket::float("Y", 0.0),
            NodeSocket::float("Z", 0.0),
        ]
    }

    fn outputs(&self) -> Vec<NodeSocket> {
        vec![NodeSocket::vector("Vector", Vec3::ZERO)]
    }

    fn execute(&self, inputs: &[SocketValue]) -> Result<Vec<SocketValue>, NodeGraphError> {
        let x = inputs.get(0).map(|v| v.as_float()).unwrap_or(0.0);
        let y = inputs.get(1).map(|v| v.as_float()).unwrap_or(0.0);
        let z = inputs.get(2).map(|v| v.as_float()).unwrap_or(0.0);
        Ok(vec![SocketValue::Vector3(Vec3::new(x, y, z))])
    }

    fn clone_box(&self) -> Box<dyn GeoNodeOp> {
        Box::new(self.clone())
    }
}

#[derive(Debug, Clone)]
pub struct SeparateXyzNode;

impl GeoNodeOp for SeparateXyzNode {
    fn name(&self) -> &str {
        "Separate XYZ"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::Math
    }

    fn inputs(&self) -> Vec<NodeSocket> {
        vec![NodeSocket::vector("Vector", Vec3::ZERO)]
    }

    fn outputs(&self) -> Vec<NodeSocket> {
        vec![
            NodeSocket::float("X", 0.0),
            NodeSocket::float("Y", 0.0),
            NodeSocket::float("Z", 0.0),
        ]
    }

    fn execute(&self, inputs: &[SocketValue]) -> Result<Vec<SocketValue>, NodeGraphError> {
        let v = inputs.get(0).map(|val| val.as_vec3()).unwrap_or(Vec3::ZERO);
        Ok(vec![
            SocketValue::Float(v.x),
            SocketValue::Float(v.y),
            SocketValue::Float(v.z),
        ])
    }

    fn clone_box(&self) -> Box<dyn GeoNodeOp> {
        Box::new(self.clone())
    }
}

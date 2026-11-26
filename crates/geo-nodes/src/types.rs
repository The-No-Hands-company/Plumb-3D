use core_math::{Transform, Vec3};
use geo_kernel::Mesh;
use serde::{Deserialize, Serialize};
use slotmap::new_key_type;
use thiserror::Error;

new_key_type! {
    /// SlotMap key identifying a node inside a `GeoNodeGraph`
    pub struct NodeKey;
}

/// Data types that can flow through visual node sockets
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SocketType {
    Geometry,
    Points,
    Float,
    Int,
    Vector3,
    Boolean,
    Transform,
    String,
    Field,
}

impl SocketType {
    pub fn name(&self) -> &'static str {
        match self {
            SocketType::Geometry => "Geometry",
            SocketType::Points => "Points",
            SocketType::Float => "Float",
            SocketType::Int => "Integer",
            SocketType::Vector3 => "Vector",
            SocketType::Boolean => "Boolean",
            SocketType::Transform => "Transform",
            SocketType::String => "String",
            SocketType::Field => "Field",
        }
    }

    /// Color representing the socket data type in the visual canvas
    pub fn color_rgba(&self) -> [u8; 4] {
        match self {
            SocketType::Geometry => [34, 197, 94, 255],   // Emerald Green
            SocketType::Points => [217, 70, 239, 255],   // Vibrant Magenta
            SocketType::Float => [234, 179, 8, 255],     // Golden Yellow
            SocketType::Int => [168, 85, 247, 255],     // Purple
            SocketType::Vector3 => [59, 130, 246, 255], // Sky Blue
            SocketType::Boolean => [239, 68, 68, 255],   // Crimson Red
            SocketType::Transform => [249, 115, 22, 255],// Orange
            SocketType::String => [156, 163, 175, 255], // Gray
            SocketType::Field => [14, 165, 233, 255],   // Cyan Field
        }
    }
}

/// A 3D point instance used in scattering and instancing workflows
#[derive(Debug, Clone, PartialEq)]
pub struct PointInstance {
    pub position: Vec3,
    pub normal: Vec3,
    pub scale: Vec3,
}

impl PointInstance {
    pub fn new(position: Vec3, normal: Vec3) -> Self {
        Self {
            position,
            normal,
            scale: Vec3::ONE,
        }
    }
}

/// Domain for procedural attribute fields (Blender Geo Nodes style)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum AttributeDomain {
    #[default]
    Point,
    Vertex,
    Face,
}

/// Data buffer for per-element attribute values
#[derive(Debug, Clone, PartialEq)]
pub enum AttributeData {
    Float(Vec<f32>),
    Vector3(Vec<Vec3>),
    Int(Vec<i32>),
    Color(Vec<[f32; 4]>),
}

impl Default for AttributeData {
    fn default() -> Self {
        Self::Float(Vec::new())
    }
}

/// A container of procedural attribute fields
#[derive(Debug, Clone, PartialEq, Default)]
pub struct AttributeContainer {
    pub name: String,
    pub domain: AttributeDomain,
    pub data: AttributeData,
}

impl AttributeContainer {
    pub fn new_float(name: impl Into<String>, domain: AttributeDomain, values: Vec<f32>) -> Self {
        Self {
            name: name.into(),
            domain,
            data: AttributeData::Float(values),
        }
    }

    pub fn new_vector(name: impl Into<String>, domain: AttributeDomain, values: Vec<Vec3>) -> Self {
        Self {
            name: name.into(),
            domain,
            data: AttributeData::Vector3(values),
        }
    }

    pub fn len(&self) -> usize {
        match &self.data {
            AttributeData::Float(v) => v.len(),
            AttributeData::Vector3(v) => v.len(),
            AttributeData::Int(v) => v.len(),
            AttributeData::Color(v) => v.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn get_float(&self, index: usize) -> Option<f32> {
        match &self.data {
            AttributeData::Float(v) => v.get(index).copied(),
            AttributeData::Int(v) => v.get(index).map(|&i| i as f32),
            _ => None,
        }
    }

    pub fn get_vector(&self, index: usize) -> Option<Vec3> {
        match &self.data {
            AttributeData::Vector3(v) => v.get(index).copied(),
            AttributeData::Float(v) => v.get(index).map(|&f| Vec3::splat(f)),
            _ => None,
        }
    }
}

/// Dynamic value flowing through sockets during graph evaluation
#[derive(Debug, Clone)]
pub enum SocketValue {
    Geometry(Option<Mesh>),
    Points(Vec<PointInstance>),
    Float(f32),
    Int(i64),
    Vector3(Vec3),
    Boolean(bool),
    Transform(Transform),
    String(String),
    Field(AttributeContainer),
}

impl Default for SocketValue {
    fn default() -> Self {
        Self::Float(0.0)
    }
}

impl SocketValue {
    pub fn socket_type(&self) -> SocketType {
        match self {
            SocketValue::Geometry(_) => SocketType::Geometry,
            SocketValue::Points(_) => SocketType::Points,
            SocketValue::Float(_) => SocketType::Float,
            SocketValue::Int(_) => SocketType::Int,
            SocketValue::Vector3(_) => SocketType::Vector3,
            SocketValue::Boolean(_) => SocketType::Boolean,
            SocketValue::Transform(_) => SocketType::Transform,
            SocketValue::String(_) => SocketType::String,
            SocketValue::Field(_) => SocketType::Field,
        }
    }

    pub fn as_field(&self) -> Option<&AttributeContainer> {
        match self {
            SocketValue::Field(f) => Some(f),
            _ => None,
        }
    }

    pub fn as_geometry(&self) -> Option<&Mesh> {
        match self {
            SocketValue::Geometry(Some(m)) => Some(m),
            _ => None,
        }
    }

    pub fn as_points(&self) -> Option<&[PointInstance]> {
        match self {
            SocketValue::Points(pts) => Some(pts.as_slice()),
            _ => None,
        }
    }

    pub fn to_geometry(self) -> Option<Mesh> {
        match self {
            SocketValue::Geometry(m) => m,
            _ => None,
        }
    }

    pub fn as_float(&self) -> f32 {
        match self {
            SocketValue::Float(f) => *f,
            SocketValue::Int(i) => *i as f32,
            SocketValue::Boolean(b) => if *b { 1.0 } else { 0.0 },
            SocketValue::Vector3(v) => v.length(),
            _ => 0.0,
        }
    }

    pub fn as_int(&self) -> i64 {
        match self {
            SocketValue::Int(i) => *i,
            SocketValue::Float(f) => *f as i64,
            SocketValue::Boolean(b) if *b => { 1 },
            _ => 0,
        }
    }

    pub fn as_vec3(&self) -> Vec3 {
        match self {
            SocketValue::Vector3(v) => *v,
            SocketValue::Float(f) => Vec3::splat(*f),
            SocketValue::Int(i) => Vec3::splat(*i as f32),
            _ => Vec3::ZERO,
        }
    }

    pub fn as_bool(&self) -> bool {
        match self {
            SocketValue::Boolean(b) => *b,
            SocketValue::Float(f) => *f != 0.0,
            SocketValue::Int(i) => *i != 0,
            _ => false,
        }
    }

    pub fn as_transform(&self) -> Transform {
        match self {
            SocketValue::Transform(t) => *t,
            SocketValue::Vector3(v) => Transform::from_translation(*v),
            _ => Transform::IDENTITY,
        }
    }

    pub fn as_string(&self) -> String {
        match self {
            SocketValue::String(s) => s.clone(),
            SocketValue::Float(f) => format!("{:.3}", f),
            SocketValue::Int(i) => format!("{}", i),
            SocketValue::Vector3(v) => format!("[{:.2}, {:.2}, {:.2}]", v.x, v.y, v.z),
            SocketValue::Boolean(b) => format!("{}", b),
            _ => String::new(),
        }
    }
}

/// Description of an input or output socket on a node
#[derive(Debug, Clone)]
pub struct NodeSocket {
    pub name: String,
    pub socket_type: SocketType,
    pub value: SocketValue,
}

impl NodeSocket {
    pub fn new(name: impl Into<String>, socket_type: SocketType, default_val: SocketValue) -> Self {
        Self {
            name: name.into(),
            socket_type,
            value: default_val,
        }
    }

    pub fn geometry(name: impl Into<String>) -> Self {
        Self::new(name, SocketType::Geometry, SocketValue::Geometry(None))
    }

    pub fn points(name: impl Into<String>) -> Self {
        Self::new(name, SocketType::Points, SocketValue::Points(Vec::new()))
    }

    pub fn float(name: impl Into<String>, default: f32) -> Self {
        Self::new(name, SocketType::Float, SocketValue::Float(default))
    }

    pub fn int(name: impl Into<String>, default: i64) -> Self {
        Self::new(name, SocketType::Int, SocketValue::Int(default))
    }

    pub fn vector(name: impl Into<String>, default: Vec3) -> Self {
        Self::new(name, SocketType::Vector3, SocketValue::Vector3(default))
    }

    pub fn boolean(name: impl Into<String>, default: bool) -> Self {
        Self::new(name, SocketType::Boolean, SocketValue::Boolean(default))
    }

    pub fn transform(name: impl Into<String>) -> Self {
        Self::new(name, SocketType::Transform, SocketValue::Transform(Transform::IDENTITY))
    }

    pub fn field(name: impl Into<String>) -> Self {
        Self::new(name, SocketType::Field, SocketValue::Field(AttributeContainer::default()))
    }
}

/// Directed connection between an output socket and an input socket
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeLink {
    pub from_node: NodeKey,
    pub from_socket: usize,
    pub to_node: NodeKey,
    pub to_socket: usize,
}

/// Categorical domain of a visual node
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeCategory {
    Inputs,
    Primitives,
    GeometryOps,
    CurveOps,
    PointOps,
    Math,
    Output,
}

#[derive(Error, Debug)]
pub enum NodeGraphError {
    #[error("Node graph contains a cyclic dependency loop")]
    CyclicDependency,
    #[error("Socket index {index} out of bounds on node {node_name}")]
    InvalidSocket { node_name: String, index: usize },
    #[error("Node evaluation failed: {0}")]
    ExecutionError(String),
    #[error("Node graph has no Group Output node")]
    MissingOutput,
}

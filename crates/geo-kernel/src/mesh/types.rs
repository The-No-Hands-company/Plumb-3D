use core_math::{Vec2, Vec3};
use slotmap::{new_key_type, SlotMap};
use smallvec::SmallVec;
use std::collections::HashMap;
use thiserror::Error;

new_key_type! {
    pub struct VertexKey;
    pub struct EdgeKey;
    pub struct FaceKey;
    pub struct HalfEdgeKey;
}

#[derive(Error, Debug, PartialEq)]
pub enum MeshError {
    #[error("Invalid vertex key: {0:?}")]
    InvalidVertex(VertexKey),
    #[error("Invalid edge key: {0:?}")]
    InvalidEdge(EdgeKey),
    #[error("Invalid face key: {0:?}")]
    InvalidFace(FaceKey),
    #[error("Invalid half-edge key: {0:?}")]
    InvalidHalfEdge(HalfEdgeKey),
    #[error("Face must have at least 3 vertices, got {0}")]
    DegenerateFace(usize),
    #[error("Non-manifold geometry detected at edge between {0:?} and {1:?}")]
    NonManifoldEdge(VertexKey, VertexKey),
    #[error("Duplicate vertex in face polygon: {0:?}")]
    DuplicateVertexInFace(VertexKey),
    #[error("Topological invariant failure: {0}")]
    InvariantFailed(String),
}

/// A Vertex in 3D space with position, normal, UV, RGBA color and incident half-edge pointers.
#[derive(Debug, Clone, PartialEq)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub uv: Vec2,
    pub color: [f32; 4],
    /// Vertex sharpness/crease weight in [0.0, 1.0] for Catmull-Clark subdivision surfaces.
    pub crease_weight: f32,
    /// Sculpt mask weight in [0.0, 1.0] where 1.0 pins the vertex from deformation.
    pub sculpt_mask: f32,
    /// List of outgoing half-edges originating from this vertex.
    pub half_edges: SmallVec<[HalfEdgeKey; 6]>,
}

impl Vertex {
    pub fn new(position: Vec3) -> Self {
        Self {
            position,
            normal: Vec3::Y,
            uv: Vec2::ZERO,
            color: [1.0, 1.0, 1.0, 1.0],
            crease_weight: 0.0,
            sculpt_mask: 0.0,
            half_edges: SmallVec::new(),
        }
    }

    pub fn with_uv(position: Vec3, uv: Vec2) -> Self {
        Self {
            position,
            normal: Vec3::Y,
            uv,
            color: [1.0, 1.0, 1.0, 1.0],
            crease_weight: 0.0,
            sculpt_mask: 0.0,
            half_edges: SmallVec::new(),
        }
    }

    pub fn with_uv_and_color(position: Vec3, uv: Vec2, color: [f32; 4]) -> Self {
        Self {
            position,
            normal: Vec3::Y,
            uv,
            color,
            crease_weight: 0.0,
            sculpt_mask: 0.0,
            half_edges: SmallVec::new(),
        }
    }

    pub fn primary_half_edge(&self) -> Option<HalfEdgeKey> {
        self.half_edges.first().copied()
    }
}

/// An undirected Edge connecting two vertices, represented by a pair of directed HalfEdges.
#[derive(Debug, Clone, PartialEq)]
pub struct Edge {
    /// One of the two half-edges forming this edge.
    pub half_edge: HalfEdgeKey,
    /// Edge sharpness/crease flag for subdivision surfaces and hard normal shading.
    pub is_sharp: bool,
    pub crease_weight: f32,
    /// Edge UV seam flag for texture coordinate unwrapping and chart separation.
    pub is_seam: bool,
}

impl Edge {
    pub fn new(half_edge: HalfEdgeKey) -> Self {
        Self {
            half_edge,
            is_sharp: false,
            crease_weight: 0.0,
            is_seam: false,
        }
    }
}

/// A Face (polygon with 3, 4, or N vertices).
#[derive(Debug, Clone, PartialEq)]
pub struct Face {
    /// One half-edge running along the face's boundary loop.
    pub half_edge: HalfEdgeKey,
    pub normal: Vec3,
    pub material_id: u32,
}

impl Face {
    pub fn new(half_edge: HalfEdgeKey) -> Self {
        Self {
            half_edge,
            normal: Vec3::Y,
            material_id: 0,
        }
    }
}

/// A directed Half-Edge representing a directed segment in a polygon boundary.
#[derive(Debug, Clone, PartialEq)]
pub struct HalfEdge {
    /// Vertex this half-edge starts from.
    pub origin: VertexKey,
    /// Vertex this half-edge points towards.
    pub target: VertexKey,
    /// The undirected edge this half-edge belongs to.
    pub edge: EdgeKey,
    /// The face on the left of this half-edge (None if boundary/exterior).
    pub face: Option<FaceKey>,
    /// The next half-edge in the face loop (counter-clockwise) or boundary cycle.
    pub next: HalfEdgeKey,
    /// The previous half-edge in the face loop or boundary cycle.
    pub prev: HalfEdgeKey,
    /// The opposite (twin) half-edge running in the reverse direction.
    pub twin: HalfEdgeKey,
}

/// A named vertex weight group (e.g. for rigging, deformation, modifier masking)
#[derive(Debug, Clone, PartialEq)]
pub struct VertexGroup {
    pub name: String,
    pub weights: HashMap<VertexKey, f32>,
}

impl VertexGroup {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            weights: HashMap::new(),
        }
    }

    pub fn set_weight(&mut self, vertex: VertexKey, weight: f32) {
        let w = weight.clamp(0.0, 1.0);
        if w > 0.0 {
            self.weights.insert(vertex, w);
        } else {
            self.weights.remove(&vertex);
        }
    }

    pub fn get_weight(&self, vertex: VertexKey) -> f32 {
        self.weights.get(&vertex).copied().unwrap_or(0.0)
    }
}

/// Clean-room Half-Edge Mesh data structure supporting arbitrary N-gons,
/// efficient topological queries, and non-destructive modifiers.
#[derive(Debug, Clone, Default)]
pub struct Mesh {
    pub vertices: SlotMap<VertexKey, Vertex>,
    pub edges: SlotMap<EdgeKey, Edge>,
    pub faces: SlotMap<FaceKey, Face>,
    pub half_edges: SlotMap<HalfEdgeKey, HalfEdge>,
    pub vertex_groups: Vec<VertexGroup>,
    pub stashed_normals: Option<Vec<(Vec3, Vec3)>>,
}

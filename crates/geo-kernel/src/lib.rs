//! # geo-kernel
//!
//! Core 3D geometry engine providing a 2-manifold BMesh Half-Edge mesh representation,
//! non-destructive parametric modifiers, digital sculpting brushes, precision spline curves,
//! procedural primitives, and CAD/3D printing interchange.
//!
//! ## Key Modules
//! - [`mesh`]: 2-manifold Half-Edge BMesh data structure maintaining boundary loops, twin pointers, and Euler invariants.
//! - [`modifiers`]: 3ds Max-style non-destructive modifier stack (Subdivision, Catmull-Clark, Solidify, Bevel, Array, Cloth, Particles, Decimate).
//! - [`operators`]: High-level topological operators (Extrude, Inset, Bevel, Knife, Loop Cut, Connect, Rip, Symmetrize).
//! - [`sculpt`]: Multi-brush digital sculpting engine with spatial falloffs and normal displacement.
//! - [`spline`]: Cubic Bézier curves with exact de Casteljau evaluation, arc-length parameterization, and pipe sweeping.
//! - [`io`]: CAD, 3D printing, and interchange exporters (STL ASCII/Binary, PLY ASCII/Binary, glTF 2.0, OBJ).

pub mod analysis;
pub mod mesh;
pub mod primitives;
pub mod triangulation;
pub mod modifiers;
pub mod validation;
pub mod raycast;
pub mod operators;
pub mod io;
pub mod subdivision;
pub mod boolean;
pub mod uv;
pub mod decimate;
pub mod spline;
pub mod sculpt;
pub mod sculpt_dyntopo;
pub mod texture_paint;
pub mod xpbd_cloth;
pub mod multires;
pub mod gaussian_splat;
pub mod lsystem;
pub mod plugin;
pub mod kitbash;

pub use io::{PlmbMetadata, PlmbVersion, PLMB_MAGIC};
pub use plugin::{
    PluginRegistry, PlumbMeshBuffer, PlumbPlugin, PlumbPluginDescriptor, PlumbPluginHeader,
    PlumbPluginType, PluginError, PLUMB_PLUGIN_API_VERSION, PLUMB_PLUGIN_MAGIC,
};
pub use analysis::MeshAnalysisReport;
pub use mesh::{Edge, EdgeKey, Face, FaceKey, HalfEdge, HalfEdgeKey, Mesh, MeshError, Vertex, VertexGroup, VertexKey};
pub use primitives::Primitives;
pub use triangulation::{GpuVertex, RenderMesh, ShadingMode, SubMeshRange};
pub use uv::ProjectionAxis;
pub use operators::{CableGenerator, CutPoint, FilletContinuity, NgonMethod, QuadMethod, SymmetrizeDirection, WeldStyle};
pub use modifiers::{
    ArrayMode, ArrayModifier, BevelModifier, BooleanModifier, ClothModifier, ClothParticle, CurveModifier, DeformAxis, DisplaceModifier, LatticeModifier,
    MirrorAxis, MirrorModifier, Modifier, ModifierStack, MorphTarget, MorpherModifier, Particle, ParticleRenderShape, ParticleSystemModifier,
    RadialAxis, RemeshModifier, ScrewModifier, ShrinkwrapMethod, ShrinkwrapModifier,
    SimpleDeformMethod, SimpleDeformModifier, SkinBone, SkinEnvelopeModifier, SkinModifier, SmoothModifier, SolidifyModifier,
    SubdivideModifier, TaperModifier, TransformModifier, TriangulateModifier, WeldModifier, WeightedNormalMode,
    WeightedNormalModifier, WireframeModifier,
};
pub use boolean::BooleanOp;
pub use subdivision::CatmullClarkModifier;
pub use decimate::{decimate_mesh, DecimateModifier, SymmQuadric};
pub use spline::{BezierPoint, ProfileShape, Spline};
pub use sculpt::{BrushFalloff, SculptBrush, SculptSettings};
pub use sculpt_dyntopo::{DyntopoDetailMode, DyntopoMode, DyntopoSettings, DyntopoStats};
pub use texture_paint::{PaintBlendMode, PaintChannel, PaintTexture, TexturePaintBrush};
pub use xpbd_cloth::{
    SphereCollider, XpbdBendingConstraint, XpbdClothSimulator, XpbdDistanceConstraint, XpbdParticle,
};
pub use multires::{MultiresLevel, MultiresMesh};
pub use gaussian_splat::{GaussianCloud, GaussianSplat};
pub use lsystem::{LSystem, LSystemPreset};
pub use raycast::RaycastHit;
pub use kitbash::{Kitbash, KitbashHardwareType};


#[cfg(test)]
mod tests;

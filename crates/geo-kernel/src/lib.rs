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
pub mod boolean;
pub mod decimate;
pub mod diagnostics;
pub mod fluid_sph;
pub mod gaussian_splat;
pub mod groom;
pub mod io;
pub mod kitbash;
pub mod lsystem;
pub mod mesh;
pub mod modifiers;
pub mod multires;
pub mod openvdb;
pub mod operators;
pub mod plugin;
pub mod primitives;
pub mod quadriflow;
pub mod raycast;
pub mod sculpt;
pub mod sculpt_dyntopo;
pub mod sketch;
pub mod slicer;
pub mod spline;
pub mod subd_nurbs;
pub mod subdivision;
pub mod texture_paint;
pub mod triangulation;
pub mod trimmed_nurbs;
pub mod uv;
pub mod validation;
pub mod xpbd_cloth;

pub use analysis::MeshAnalysisReport;
pub use boolean::BooleanOp;
pub use decimate::{decimate_mesh, DecimateModifier, SymmQuadric};
pub use diagnostics::{
    ContinuityClass, CurvatureColorMap, CurvatureNeedle, SeamContinuityReport,
    SurfaceCurvatureSample,
};
pub use fluid_sph::{FluidParticle, SphFluidSimulator, SphSettings};
pub use gaussian_splat::{GaussianCloud, GaussianSplat};
pub use groom::{GroomBrush, GroomSettings, GroomSystem, HairGuide, HairStrand};
pub use io::{PlmbMetadata, PlmbVersion, PLMB_MAGIC};
pub use kitbash::{Kitbash, KitbashHardwareType};
pub use lsystem::{LSystem, LSystemPreset};
pub use mesh::{
    Edge, EdgeKey, Face, FaceKey, HalfEdge, HalfEdgeKey, Mesh, MeshError, MeshManifoldReport,
    Vertex, VertexGroup, VertexKey,
};
pub use modifiers::{
    ArrayMode, ArrayModifier, BevelModifier, BooleanModifier, ClothModifier, ClothParticle,
    CurveModifier, DeformAxis, DisplaceModifier, LatticeModifier, MirrorAxis, MirrorModifier,
    Modifier, ModifierStack, MorphTarget, MorpherModifier, Particle, ParticleRenderShape,
    ParticleSystemModifier, RadialAxis, RemeshModifier, ScrewModifier, ShrinkwrapMethod,
    ShrinkwrapModifier, SimpleDeformMethod, SimpleDeformModifier, SkinBone, SkinEnvelopeModifier,
    SkinModifier, SmoothModifier, SolidifyModifier, SubdivideModifier, TaperModifier,
    TransformModifier, TriangulateModifier, WeightedNormalMode, WeightedNormalModifier,
    WeldModifier, WireframeModifier,
};
pub use multires::{MultiresLevel, MultiresMesh};
pub use openvdb::{VdbGrid, VdbLeafBrick, VolumeGenerators, VolumetricSample};
pub use operators::{
    bevel::BevelProfile, CableGenerator, CoonsPatchType, CurvatureTensor, CutPoint,
    FieldRemeshSettings, FilletContinuity, FilletMode, NgonMethod, QuadMethod,
    RollingBallFilletSettings, ShellDirection, ShellSettings, SymmetrizeDirection, WeldStyle,
};
pub use plugin::{
    PluginError, PluginRegistry, PlumbMeshBuffer, PlumbPlugin, PlumbPluginDescriptor,
    PlumbPluginHeader, PlumbPluginType, PLUMB_PLUGIN_API_VERSION, PLUMB_PLUGIN_MAGIC,
};
pub use primitives::{CapFillType, Primitives};
pub use quadriflow::{CrossField4RoSy, QuadriflowSettings};
pub use raycast::RaycastHit;
pub use sculpt::{BrushAlphaTexture, BrushFalloff, SculptBrush, SculptSettings};
pub use sculpt_dyntopo::{DyntopoDetailMode, DyntopoMode, DyntopoSettings, DyntopoStats};
pub use sketch::{
    extract_dimension_callouts, extrude_sketch, revolve_sketch, sketch_to_splines, solve_sketch,
    ConstraintId, ConstraintType, DimensionCallout, DimensionType, Sketch, SketchArc, SketchCircle,
    SketchConstraint, SketchEntity, SketchEntityId, SketchLine, SketchPoint, SketchPointId,
    SolverConfig, SolverStatus, Workplane,
};
pub use slicer::{InfillPattern, SlicedLayer, SlicerSettings, ToolpathSegment};
pub use spline::{BezierPoint, HandleType, ProfileShape, Spline};
pub use subd_nurbs::{convert_mesh_to_nurbs_patches, export_subd_nurbs_step, BicubicNurbsPatch};
pub use subdivision::CatmullClarkModifier;
pub use texture_paint::{PaintBlendMode, PaintChannel, PaintTexture, TexturePaintBrush};
pub use triangulation::{GpuVertex, RenderMesh, ShadingMode, SubMeshRange};
pub use trimmed_nurbs::{SurfaceIntersectionPoint, TrimmedNurbsPatch};
pub use uv::ProjectionAxis;
pub use xpbd_cloth::{
    SphereCollider, XpbdBendingConstraint, XpbdClothSimulator, XpbdDistanceConstraint, XpbdParticle,
};

#[cfg(test)]
mod tests;

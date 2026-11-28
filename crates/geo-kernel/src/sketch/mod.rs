//! 2D Parametric CAD Constraint Sketcher & Live Dimensions Engine.
//!
//! Provides workplanes, 2D sketch entities, geometric/dimensional constraints,
//! sub-millisecond iterative relaxation constraint solving, live metric callout badges,
//! and 3D BMesh extrusion/revolving.

pub mod constraints;
pub mod dimension;
pub mod entities;
pub mod mesh_gen;
pub mod solver;
pub mod workplane;

pub use constraints::{ConstraintId, ConstraintType, SketchConstraint};
pub use dimension::{extract_dimension_callouts, DimensionCallout, DimensionType};
pub use entities::{
    Sketch, SketchArc, SketchCircle, SketchEntity, SketchEntityId, SketchLine, SketchPoint,
    SketchPointId,
};
pub use mesh_gen::{extrude_sketch, revolve_sketch, sketch_to_splines};
pub use solver::{solve_sketch, SolverConfig, SolverStatus};
pub use workplane::Workplane;

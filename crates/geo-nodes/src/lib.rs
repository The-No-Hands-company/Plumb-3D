//! # crates/geo-nodes
//!
//! Visual Geometry Node Graph evaluation engine.
//!
//! Provides a pure-Rust, headless Directed Acyclic Graph (DAG) for procedural
//! geometry synthesis and modification, inspired by Blender Geometry Nodes
//! and Unreal Engine Blueprints.
//!
//! Features:
//! - Cycle-preventing topological DAG evaluation via Kahn's algorithm.
//! - Typed data-flow sockets: Geometry (`BMesh`), Float, Vector3, Boolean, Int, Transform, Spline.
//! - Comprehensive procedural operators (Primitives, Extrude, Subdivide, Boolean, Solidify, Math, Curves).
//! - Clean Separation of Concerns (zero GUI/GPU dependencies in this engine crate).

pub mod graph;
pub mod modifier;
pub mod nodes;
pub mod types;

#[cfg(test)]
mod tests;

pub use graph::{GeoNode, GeoNodeGraph};
pub use modifier::*;
pub use nodes::*;
pub use types::*;

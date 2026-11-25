//! # graph-engine
//!
//! Maya-style Directed Acyclic Graph (DAG) Scene Graph, hierarchical transform cascades,
//! parametric constraints, Inverse Kinematics (IK), and animation keyframing.
//!
//! ## Key Modules
//! - [`scene`]: [`SceneGraph`] managing hierarchical DAG nodes with dirty-flag transformation cascades.
//! - [`node`]: [`SceneNode`] supporting polymorphic payload types (Meshes, Lights, Cameras, Joint Armatures, Spline Curves).
//! - [`constraint`]: Rigging constraints including [`AimConstraint`], [`ParentConstraint`], and [`PathConstraint`].
//! - [`ik`]: Analytic Two-Bone IK and iterative Cyclic Coordinate Descent (CCD) multi-joint IK solvers.
//! - [`history`]: Transactional [`Command`] trait and [`HistoryManager`] for undo/redo execution.
//! - [`animation`]: Keyframed parameter and transform tracks with linear and step interpolation.

pub mod animation;
pub mod constraint;
pub mod history;
pub mod ik;
pub mod material;
pub mod node;
pub mod physics;
pub mod scene;
pub mod usda;
pub mod collab;

pub use animation::{AnimationManager, AnimationTrack, InterpolationMode, Keyframe, TransformTrack};
pub use constraint::{AimConstraint, Constraint, ConstraintStack, ParentConstraint, PathConstraint};
pub use history::{Command, HistoryManager};
pub use ik::{solve_two_bone_ik, CcdIkSolver, IkConstraint, TwoBoneIkResult};
pub use material::{LightType, Material};
pub use node::{NodeData, NodeId, SceneNode};
pub use physics::{CollisionShape, PhysicsWorld, RigidBodyComponent, RigidBodyType};
pub use scene::SceneGraph;
pub use usda::{
    export_scene_to_usda, export_scene_to_usda_file, import_scene_from_usda,
    import_scene_from_usda_file,
};
pub use collab::{CollabChatEntry, CollabMessage, CollabSession, CollabUser};

#[cfg(test)]
mod tests;

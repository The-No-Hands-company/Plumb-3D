//! FEA Boundary Conditions, Supports & Applied Load Formulations
//!
//! Defines structural support constraints (Fixed, Pinned, Roller) and
//! external point forces, distributed surface loads, and gravity body loads.

use core_math::Vec3;
use serde::{Deserialize, Serialize};

use super::material::FeaMaterial;

/// Structural Support Constraint Type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SupportType {
    /// Locks all 3 translational degrees of freedom (ux = 0, uy = 0, uz = 0)
    #[default]
    Fixed,
    /// Constrains normal direction only, allowing sliding in tangential plane
    RollerX,
    RollerY,
    RollerZ,
}

/// A structural support applied to a set of mesh nodes
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SupportConstraint {
    pub support_type: SupportType,
    pub node_indices: Vec<usize>,
}

impl SupportConstraint {
    pub fn new_fixed(node_indices: Vec<usize>) -> Self {
        Self {
            support_type: SupportType::Fixed,
            node_indices,
        }
    }
}

/// Mechanical Load Applied to Nodes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppliedLoad {
    /// Total force vector in Newtons (N) distributed equally across target nodes
    pub force_vector_n: Vec3,
    pub node_indices: Vec<usize>,
}

impl AppliedLoad {
    pub fn new(force_vector_n: Vec3, node_indices: Vec<usize>) -> Self {
        Self {
            force_vector_n,
            node_indices,
        }
    }
}

/// Complete FEA Structural Simulation Setup
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FeaConfig {
    pub material: FeaMaterial,
    pub supports: Vec<SupportConstraint>,
    pub loads: Vec<AppliedLoad>,
    pub include_gravity: bool,
    /// Visualization displacement magnification factor (e.g. 50x, 200x)
    pub deformation_scale: f32,
}

impl Default for FeaConfig {
    fn default() -> Self {
        Self {
            material: FeaMaterial::structural_steel(),
            supports: Vec::new(),
            loads: Vec::new(),
            include_gravity: true,
            deformation_scale: 100.0,
        }
    }
}

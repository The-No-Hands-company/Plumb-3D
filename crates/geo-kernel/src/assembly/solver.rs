//! CAD Kinematic Assembly Solver
//!
//! Solves relative component positions and orientations from kinematic mates,
//! propagates motion through joint constraints, and computes degrees of freedom.

use core_math::{Quat, Transform, Vec3};
use serde::{Deserialize, Serialize};

use super::mate::{KinematicMate, MateAlignment, MateConnector, MateType};

/// A rigid solid body component participating in an assembly
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AssemblyBody {
    pub id: usize,
    pub name: String,
    /// Whether this body is permanently fixed/grounded in world space
    pub is_grounded: bool,
    /// World-space rigid transform
    pub transform: Transform,
    /// Remaining unconstrained translational degrees of freedom (0..=3)
    pub translation_dof: u8,
    /// Remaining unconstrained rotational degrees of freedom (0..=3)
    pub rotation_dof: u8,
}

impl AssemblyBody {
    pub fn new(id: usize, name: impl Into<String>, transform: Transform) -> Self {
        Self {
            id,
            name: name.into(),
            is_grounded: false,
            transform,
            translation_dof: 3,
            rotation_dof: 3,
        }
    }

    pub fn grounded(id: usize, name: impl Into<String>, transform: Transform) -> Self {
        Self {
            id,
            name: name.into(),
            is_grounded: true,
            transform,
            translation_dof: 0,
            rotation_dof: 0,
        }
    }
}

/// Assembly Kinematics Solver
#[derive(Debug, Clone, Default)]
pub struct AssemblySolver {
    pub bodies: Vec<AssemblyBody>,
    pub mates: Vec<KinematicMate>,
}

impl AssemblySolver {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a new body in the assembly
    pub fn add_body(&mut self, body: AssemblyBody) -> usize {
        let id = body.id;
        self.bodies.push(body);
        id
    }

    /// Add a kinematic mate constraint between two bodies
    pub fn add_mate(&mut self, mate: KinematicMate) -> usize {
        let id = mate.id;
        self.mates.push(mate);
        id
    }

    /// Find body by index
    pub fn get_body(&self, id: usize) -> Option<&AssemblyBody> {
        self.bodies.iter().find(|b| b.id == id)
    }

    /// Find body by index mutably
    pub fn get_body_mut(&mut self, id: usize) -> Option<&mut AssemblyBody> {
        self.bodies.iter_mut().find(|b| b.id == id)
    }

    /// Set a body as grounded (anchor)
    pub fn set_grounded(&mut self, id: usize, grounded: bool) {
        if let Some(b) = self.get_body_mut(id) {
            b.is_grounded = grounded;
            if grounded {
                b.translation_dof = 0;
                b.rotation_dof = 0;
            }
        }
    }

    /// Actuate a joint parameter (angle or displacement) and re-solve downstream transforms
    pub fn actuate_mate(&mut self, mate_id: usize, param: f32) {
        if let Some(mate) = self.mates.iter_mut().find(|m| m.id == mate_id) {
            mate.current_param = param;
            mate.clamp_param();
        }
        self.solve();
    }

    /// Solve the entire assembly constraint system
    pub fn solve(&mut self) {
        // Reset DOFs for non-grounded bodies
        for body in &mut self.bodies {
            if body.is_grounded {
                body.translation_dof = 0;
                body.rotation_dof = 0;
            } else {
                body.translation_dof = 3;
                body.rotation_dof = 3;
            }
        }

        // Iteratively propagate mates from solved/grounded bodies
        let mut solved_bodies = std::collections::HashSet::new();
        for body in &self.bodies {
            if body.is_grounded {
                solved_bodies.insert(body.id);
            }
        }

        let max_passes = self.bodies.len() + 2;
        for _ in 0..max_passes {
            let mut made_progress = false;

            for mate_idx in 0..self.mates.len() {
                let mate = &self.mates[mate_idx];
                if !mate.enabled {
                    continue;
                }

                let first_solved = solved_bodies.contains(&mate.first_body);
                let second_solved = solved_bodies.contains(&mate.second_body);

                if first_solved && !second_solved {
                    let first_body_tf = self.get_body(mate.first_body).unwrap().transform;
                    let mate_copy = mate.clone();
                    let new_tf = Self::compute_child_transform(
                        &first_body_tf,
                        &mate_copy.first_connector,
                        &mate_copy.second_connector,
                        mate_copy.mate_type,
                        mate_copy.alignment,
                        mate_copy.offset,
                        mate_copy.current_param,
                    );

                    let (trans_dof, rot_dof) = mate_copy.unconstrained_dof();
                    if let Some(child) = self.get_body_mut(mate_copy.second_body) {
                        child.transform = new_tf;
                        child.translation_dof = trans_dof;
                        child.rotation_dof = rot_dof;
                    }

                    solved_bodies.insert(mate_copy.second_body);
                    made_progress = true;
                }
            }

            if !made_progress {
                break;
            }
        }
    }

    /// Compute the world transform for the second (child) body given parent transform and mate parameters
    pub fn compute_child_transform(
        parent_transform: &Transform,
        parent_connector: &MateConnector,
        child_connector: &MateConnector,
        mate_type: MateType,
        alignment: MateAlignment,
        offset: f32,
        current_param: f32,
    ) -> Transform {
        // Connector 1 in world space
        let c1_world = parent_connector.transform_by(parent_transform);

        // Compute target orientation for child connector
        let mut target_rot = c1_world.rotation();

        // If anti-aligned (face-to-face), rotate 180 deg around X-axis
        if alignment == MateAlignment::AntiAligned {
            target_rot *= Quat::from_rotation_x(std::f32::consts::PI);
        }

        // Apply joint actuation to orientation
        match mate_type {
            MateType::Revolute { .. } | MateType::Cylindrical { .. } => {
                // Rotate around local Z normal
                target_rot *= Quat::from_rotation_z(current_param);
            }
            MateType::Gear { ratio } => {
                target_rot *= Quat::from_rotation_z(-current_param * ratio);
            }
            _ => {}
        }

        // Compute target position for child connector
        let mut target_origin = c1_world.origin + c1_world.normal * offset;

        match mate_type {
            MateType::Prismatic { .. } => {
                // Slide along local tangent (X axis)
                target_origin += c1_world.tangent * current_param;
            }
            MateType::Cylindrical { .. } => {
                // Slide along local normal (Z axis)
                target_origin += c1_world.normal * current_param;
            }
            _ => {}
        }

        // Child connector local rotation and origin
        let c2_local_rot = child_connector.rotation();
        let c2_local_origin = child_connector.origin;

        // Child body rotation: R_child = target_rot * c2_local_rot^-1
        let child_rot = (target_rot * c2_local_rot.inverse()).normalize();

        // Child body translation: P_child = target_origin - R_child * c2_local_origin
        let child_trans = target_origin - child_rot * c2_local_origin;

        Transform {
            translation: child_trans,
            rotation: child_rot,
            scale: Vec3::ONE,
        }
    }
}

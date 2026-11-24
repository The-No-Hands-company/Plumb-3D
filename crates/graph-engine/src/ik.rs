use crate::{NodeId, SceneGraph};
use core_math::{Quat, Vec3};

/// Result of a 2-bone IK analytic calculation
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TwoBoneIkResult {
    pub root_rot: Quat,
    pub mid_rot: Quat,
    pub solved_mid_pos: Vec3,
    pub solved_tip_pos: Vec3,
}

/// Analytic Two-Bone Inverse Kinematics Solver (Maya IK RP Solver / Blender Two-Bone IK)
///
/// Uses the Law of Cosines and Pole Vector projection to solve knee/elbow bends
/// instantaneously without iterative convergence lag or instability.
pub fn solve_two_bone_ik(
    root_pos: Vec3,
    mid_pos: Vec3,
    tip_pos: Vec3,
    target_pos: Vec3,
    pole_pos: Vec3,
) -> Option<TwoBoneIkResult> {
    let a = (mid_pos - root_pos).length();
    let b = (tip_pos - mid_pos).length();

    if a < 1e-4 || b < 1e-4 {
        return None;
    }

    let root_to_target = target_pos - root_pos;
    let target_dist = root_to_target.length();

    // Prevent singularity: clamp distance to valid triangle reach
    let max_reach = (a + b) * 0.9999;
    let min_reach = (a - b).abs() * 1.0001;
    let clamped_dist = target_dist.clamp(min_reach, max_reach);

    let target_dir = if target_dist > 1e-4 {
        root_to_target / target_dist
    } else {
        Vec3::Z
    };

    // Law of Cosines:
    // c^2 = a^2 + b^2 - 2ab * cos(beta)  => mid angle
    // b^2 = a^2 + c^2 - 2ac * cos(alpha) => root interior angle
    let cos_alpha = ((a * a + clamped_dist * clamped_dist - b * b) / (2.0 * a * clamped_dist)).clamp(-1.0, 1.0);
    let alpha = cos_alpha.acos();

    let cos_beta = ((a * a + b * b - clamped_dist * clamped_dist) / (2.0 * a * b)).clamp(-1.0, 1.0);
    let beta = cos_beta.acos();

    // Compute the bend plane normal using pole vector
    let pole_dir = pole_pos - root_pos;
    let mut bend_normal = target_dir.cross(pole_dir);
    if bend_normal.length_squared() < 1e-6 {
        // Fallback axis if pole vector is colinear with target
        bend_normal = target_dir.cross(Vec3::Y);
        if bend_normal.length_squared() < 1e-6 {
            bend_normal = target_dir.cross(Vec3::X);
        }
    }
    bend_normal = bend_normal.normalize();

    // Perpendicular vector within the bend plane pointing towards the pole
    let bend_up = bend_normal.cross(target_dir).normalize();

    // Solved mid joint position in 3D world space
    let solved_mid = root_pos + target_dir * (a * cos_alpha) + bend_up * (a * alpha.sin());
    let solved_tip = if target_dist <= max_reach {
        target_pos
    } else {
        root_pos + target_dir * max_reach
    };

    // Compute rotation adjustments relative to original bone directions
    let orig_bone1 = (mid_pos - root_pos).normalize();
    let new_bone1 = (solved_mid - root_pos).normalize();
    let root_rot = Quat::from_rotation_arc(orig_bone1, new_bone1);

    let orig_bone2 = (tip_pos - mid_pos).normalize();
    let new_bone2 = (solved_tip - solved_mid).normalize();
    let mid_rot = Quat::from_rotation_arc(orig_bone2, new_bone2);

    let _ = beta; // Verified by cosine triangle

    Some(TwoBoneIkResult {
        root_rot,
        mid_rot,
        solved_mid_pos: solved_mid,
        solved_tip_pos: solved_tip,
    })
}

/// Cyclic Coordinate Descent (CCD) IK Solver for arbitrary N-bone chains (tentacles, spines, robotic arms)
#[derive(Debug, Clone)]
pub struct CcdIkSolver {
    pub max_iterations: usize,
    pub tolerance: f32,
    pub damping_radians: f32,
}

impl Default for CcdIkSolver {
    fn default() -> Self {
        Self {
            max_iterations: 15,
            tolerance: 1e-3,
            damping_radians: 0.5,
        }
    }
}

impl CcdIkSolver {
    pub fn new() -> Self {
        Self::default()
    }

    /// Solves an arbitrary joint chain in the SceneGraph towards `target_pos`.
    /// `chain` must be ordered from root to end-effector `[root, joint_1, ..., tip]`.
    pub fn solve(
        &self,
        chain: &[NodeId],
        target_pos: Vec3,
        scene: &mut SceneGraph,
    ) -> bool {
        if chain.len() < 2 {
            return false;
        }

        let tip_id = *chain.last().unwrap();

        for _ in 0..self.max_iterations {
            scene.update_transforms();

            let tip_pos = match scene.nodes.get(tip_id) {
                Some(n) => n.world_matrix.transform_point3(Vec3::ZERO),
                None => return false,
            };

            if (tip_pos - target_pos).length() <= self.tolerance {
                return true;
            }

            // Iterate backwards from the joint before tip to the root
            for &joint_id in chain.iter().rev().skip(1) {
                scene.update_transforms();

                let current_tip = match scene.nodes.get(tip_id) {
                    Some(n) => n.world_matrix.transform_point3(Vec3::ZERO),
                    None => return false,
                };

                let (joint_world_pos, joint_local_rot) = match scene.nodes.get(joint_id) {
                    Some(n) => (
                        n.world_matrix.transform_point3(Vec3::ZERO),
                        n.local_transform.rotation,
                    ),
                    None => return false,
                };

                let to_tip = (current_tip - joint_world_pos).normalize_or_zero();
                let to_target = (target_pos - joint_world_pos).normalize_or_zero();

                if to_tip.length_squared() < 0.5 || to_target.length_squared() < 0.5 {
                    continue;
                }

                let cross = to_tip.cross(to_target);
                let dot = to_tip.dot(to_target).clamp(-1.0, 1.0);

                if cross.length_squared() > 1e-6 {
                    let angle = dot.acos().min(self.damping_radians);
                    let axis = cross.normalize();
                    let delta_rot = Quat::from_axis_angle(axis, angle);

                    if let Some(node) = scene.nodes.get_mut(joint_id) {
                        node.local_transform.rotation = (delta_rot * joint_local_rot).normalize();
                        scene.mark_dirty(joint_id);
                    }
                }
            }
        }

        scene.update_transforms();
        let final_tip = scene.nodes[tip_id].world_matrix.transform_point3(Vec3::ZERO);
        (final_tip - target_pos).length() <= self.tolerance
    }
}

/// Kinematic Constraint linking a joint chain to an IK target in the DAG scene
#[derive(Debug, Clone)]
pub struct IkConstraint {
    pub chain: Vec<NodeId>,
    pub target: NodeId,
    pub pole_target: Option<NodeId>,
    pub enabled: bool,
}

impl IkConstraint {
    pub fn new(chain: Vec<NodeId>, target: NodeId) -> Self {
        Self {
            chain,
            target,
            pole_target: None,
            enabled: true,
        }
    }

    pub fn with_pole(mut self, pole: NodeId) -> Self {
        self.pole_target = Some(pole);
        self
    }

    /// Evaluates the IK constraint on the scene graph
    pub fn evaluate(&self, scene: &mut SceneGraph) {
        if !self.enabled || self.chain.len() < 2 {
            return;
        }

        let target_pos = match scene.nodes.get(self.target) {
            Some(n) => n.world_matrix.transform_point3(Vec3::ZERO),
            None => return,
        };

        if self.chain.len() == 3 {
            // Use analytic 2-bone solver
            let root_id = self.chain[0];
            let mid_id = self.chain[1];
            let tip_id = self.chain[2];

            let root_pos = scene.nodes[root_id].world_matrix.transform_point3(Vec3::ZERO);
            let mid_pos = scene.nodes[mid_id].world_matrix.transform_point3(Vec3::ZERO);
            let tip_pos = scene.nodes[tip_id].world_matrix.transform_point3(Vec3::ZERO);

            let pole_pos = if let Some(pole_id) = self.pole_target {
                scene.nodes.get(pole_id)
                    .map(|n| n.world_matrix.transform_point3(Vec3::ZERO))
                    .unwrap_or(mid_pos + Vec3::Z)
            } else {
                mid_pos + Vec3::Z
            };

            if let Some(res) = solve_two_bone_ik(root_pos, mid_pos, tip_pos, target_pos, pole_pos) {
                if let Some(root_node) = scene.nodes.get_mut(root_id) {
                    root_node.local_transform.rotation = (res.root_rot * root_node.local_transform.rotation).normalize();
                    scene.mark_dirty(root_id);
                }
                if let Some(mid_node) = scene.nodes.get_mut(mid_id) {
                    mid_node.local_transform.rotation = (res.mid_rot * mid_node.local_transform.rotation).normalize();
                    scene.mark_dirty(mid_id);
                }
                scene.update_transforms();
            }
        } else {
            // General CCD IK solver
            let solver = CcdIkSolver::new();
            solver.solve(&self.chain, target_pos, scene);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SceneNode;
    use core_math::Transform;

    #[test]
    fn test_analytic_two_bone_ik() {
        let root = Vec3::new(0.0, 0.0, 0.0);
        let mid = Vec3::new(0.0, 2.0, 0.0);
        let tip = Vec3::new(0.0, 4.0, 0.0);

        // Target bent forward along +Z at height Y=2.5
        let target = Vec3::new(0.0, 2.5, 2.5);
        let pole = Vec3::new(0.0, 2.0, 5.0); // Forward knee hint

        let res = solve_two_bone_ik(root, mid, tip, target, pole).unwrap();

        // End-effector must match the target position
        assert!((res.solved_tip_pos - target).length() < 1e-3);

        // Bone lengths must be strictly preserved
        let l1 = (res.solved_mid_pos - root).length();
        let l2 = (res.solved_tip_pos - res.solved_mid_pos).length();
        assert!((l1 - 2.0).abs() < 1e-4);
        assert!((l2 - 2.0).abs() < 1e-4);

        // Knee must bend forward (+Z) towards the pole vector
        assert!(res.solved_mid_pos.z > 0.1);
    }

    #[test]
    fn test_ccd_ik_solver() {
        let mut scene = SceneGraph::new();

        // Create 3-bone chain (Root -> Mid -> Tip)
        let root_id = scene.add_node(SceneNode::new_transform("Root", Transform::IDENTITY));
        let mid_id = scene.add_node(SceneNode::new_transform("Mid", Transform {
            translation: Vec3::new(0.0, 2.0, 0.0),
            ..Default::default()
        }));
        let tip_id = scene.add_node(SceneNode::new_transform("Tip", Transform {
            translation: Vec3::new(0.0, 2.0, 0.0),
            ..Default::default()
        }));

        scene.set_parent(mid_id, Some(root_id));
        scene.set_parent(tip_id, Some(mid_id));
        scene.update_transforms();

        let chain = vec![root_id, mid_id, tip_id];
        let target = Vec3::new(1.5, 2.5, 1.0);

        let solver = CcdIkSolver {
            max_iterations: 30,
            tolerance: 0.05,
            damping_radians: 0.8,
        };

        let reached = solver.solve(&chain, target, &mut scene);
        assert!(reached);

        let final_tip_pos = scene.nodes[tip_id].world_matrix.transform_point3(Vec3::ZERO);
        assert!((final_tip_pos - target).length() < 0.05);
    }
}

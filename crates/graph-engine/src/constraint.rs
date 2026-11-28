use crate::{NodeId, SceneGraph};
use core_math::{Quat, Vec3};
use geo_kernel::Spline;

/// Constrains a node's position and orientation along a 3D parametric Bézier spline (Blender Follow Path / 3ds Max Path Constraint)
#[derive(Debug, Clone)]
pub struct PathConstraint {
    pub target_node: NodeId,
    pub path: Spline,
    /// Normalized parameter u in [0, 1] along the spline
    pub u: f32,
    /// Whether to orient the node's forward axis along the curve tangent
    pub follow_tangent: bool,
    pub world_up: Vec3,
    pub enabled: bool,
}

impl PathConstraint {
    pub fn new(target_node: NodeId, path: Spline) -> Self {
        Self {
            target_node,
            path,
            u: 0.0,
            follow_tangent: true,
            world_up: Vec3::Y,
            enabled: true,
        }
    }

    /// Evaluates the path constraint, updating the target node's translation and rotation in the scene
    pub fn evaluate(&self, scene: &mut SceneGraph) {
        if !self.enabled {
            return;
        }

        let (pos, tangent) = self.path.eval(self.u);

        if let Some(node) = scene.nodes.get_mut(self.target_node) {
            node.local_transform.translation = pos;

            if self.follow_tangent && tangent.length_squared() > 1e-4 {
                // Compute look-at orientation from default forward (Z) to tangent
                let forward = tangent.normalize();
                let right = forward.cross(self.world_up).normalize_or_zero();
                let up = if right.length_squared() > 1e-4 {
                    right.cross(forward).normalize()
                } else {
                    self.world_up
                };

                let rot = Quat::from_rotation_arc(Vec3::Z, forward);
                node.local_transform.rotation = rot;
                let _ = up; // Preserved up vector
            }

            scene.mark_dirty(self.target_node);
        }
    }
}

/// Constrains a node to aim its forward axis towards a target node in 3D space (Maya Aim Constraint / 3ds Max LookAt)
#[derive(Debug, Clone)]
pub struct AimConstraint {
    pub constrained_node: NodeId,
    pub target_node: NodeId,
    pub world_up: Vec3,
    pub enabled: bool,
}

impl AimConstraint {
    pub fn new(constrained_node: NodeId, target_node: NodeId) -> Self {
        Self {
            constrained_node,
            target_node,
            world_up: Vec3::Y,
            enabled: true,
        }
    }

    /// Evaluates the aim constraint in world space
    pub fn evaluate(&self, scene: &mut SceneGraph) {
        if !self.enabled || self.constrained_node == self.target_node {
            return;
        }

        scene.update_transforms();

        let target_world_pos = match scene.nodes.get(self.target_node) {
            Some(n) => n.world_matrix.transform_point3(Vec3::ZERO),
            None => return,
        };

        let node_world_pos = match scene.nodes.get(self.constrained_node) {
            Some(n) => n.world_matrix.transform_point3(Vec3::ZERO),
            None => return,
        };

        let aim_vec = target_world_pos - node_world_pos;
        if aim_vec.length_squared() < 1e-4 {
            return;
        }

        let forward = aim_vec.normalize();
        let rot = Quat::from_rotation_arc(Vec3::Z, forward);

        if let Some(node) = scene.nodes.get_mut(self.constrained_node) {
            node.local_transform.rotation = rot;
            scene.mark_dirty(self.constrained_node);
        }
    }
}

/// Constrains a node's translation and rotation to match a target without altering the scene DAG hierarchy (Maya Parent Constraint)
#[derive(Debug, Clone)]
pub struct ParentConstraint {
    pub constrained_node: NodeId,
    pub target_node: NodeId,
    pub maintain_offset: bool,
    pub offset_translation: Vec3,
    pub offset_rotation: Quat,
    pub enabled: bool,
}

impl ParentConstraint {
    pub fn new(constrained_node: NodeId, target_node: NodeId) -> Self {
        Self {
            constrained_node,
            target_node,
            maintain_offset: false,
            offset_translation: Vec3::ZERO,
            offset_rotation: Quat::IDENTITY,
            enabled: true,
        }
    }

    pub fn with_offset(mut self, offset_translation: Vec3, offset_rotation: Quat) -> Self {
        self.maintain_offset = true;
        self.offset_translation = offset_translation;
        self.offset_rotation = offset_rotation;
        self
    }

    pub fn evaluate(&self, scene: &mut SceneGraph) {
        if !self.enabled || self.constrained_node == self.target_node {
            return;
        }

        scene.update_transforms();

        let (target_pos, target_rot) = match scene.nodes.get(self.target_node) {
            Some(n) => (
                n.world_matrix.transform_point3(Vec3::ZERO),
                n.local_transform.rotation,
            ),
            None => return,
        };

        if let Some(node) = scene.nodes.get_mut(self.constrained_node) {
            if self.maintain_offset {
                node.local_transform.translation =
                    target_pos + target_rot.mul_vec3(self.offset_translation);
                node.local_transform.rotation = (target_rot * self.offset_rotation).normalize();
            } else {
                node.local_transform.translation = target_pos;
                node.local_transform.rotation = target_rot;
            }
            scene.mark_dirty(self.constrained_node);
        }
    }
}

use crate::ik::IkConstraint;

/// Maya / 3ds Max style scene constraint (Aim, Parent, Follow Path, Inverse Kinematics)
#[derive(Debug, Clone)]
pub enum Constraint {
    Path(PathConstraint),
    Aim(AimConstraint),
    Parent(ParentConstraint),
    Ik(IkConstraint),
}

impl Constraint {
    pub fn evaluate(&self, scene: &mut SceneGraph) {
        match self {
            Self::Path(c) => c.evaluate(scene),
            Self::Aim(c) => c.evaluate(scene),
            Self::Parent(c) => c.evaluate(scene),
            Self::Ik(c) => c.evaluate(scene),
        }
    }

    pub fn is_enabled(&self) -> bool {
        match self {
            Self::Path(c) => c.enabled,
            Self::Aim(c) => c.enabled,
            Self::Parent(c) => c.enabled,
            Self::Ik(c) => c.enabled,
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        match self {
            Self::Path(c) => c.enabled = enabled,
            Self::Aim(c) => c.enabled = enabled,
            Self::Parent(c) => c.enabled = enabled,
            Self::Ik(c) => c.enabled = enabled,
        }
    }

    pub fn target_node(&self) -> NodeId {
        match self {
            Self::Path(c) => c.target_node,
            Self::Aim(c) => c.constrained_node,
            Self::Parent(c) => c.constrained_node,
            Self::Ik(c) => *c.chain.last().unwrap_or(&c.target),
        }
    }
}

/// Evaluatable collection of scene constraints
#[derive(Default, Debug, Clone)]
pub struct ConstraintStack {
    pub constraints: Vec<Constraint>,
}

impl ConstraintStack {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, constraint: Constraint) {
        self.constraints.push(constraint);
    }

    pub fn evaluate(&self, scene: &mut SceneGraph) {
        for c in &self.constraints {
            c.evaluate(scene);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SceneNode;
    use core_math::{vec3, Transform};
    use geo_kernel::BezierPoint;

    #[test]
    fn test_path_constraint_follow() {
        let mut scene = SceneGraph::new();
        let node_id = scene.add_node(SceneNode::new_transform("Car", Transform::IDENTITY));

        let mut spline = Spline::new();
        spline.add_point(BezierPoint::sharp(vec3(0.0, 0.0, 0.0)));
        spline.add_point(BezierPoint::sharp(vec3(10.0, 5.0, 0.0)));

        let mut constraint = PathConstraint::new(node_id, spline);

        // At u = 0.0
        constraint.u = 0.0;
        constraint.evaluate(&mut scene);
        assert_eq!(
            scene.nodes[node_id].local_transform.translation,
            vec3(0.0, 0.0, 0.0)
        );

        // At u = 0.5 (halfway)
        constraint.u = 0.5;
        constraint.evaluate(&mut scene);
        let pos = scene.nodes[node_id].local_transform.translation;
        assert!((pos.x - 5.0).abs() < 1e-3);
        assert!((pos.y - 2.5).abs() < 1e-3);

        // At u = 1.0 (end)
        constraint.u = 1.0;
        constraint.evaluate(&mut scene);
        let end_pos = scene.nodes[node_id].local_transform.translation;
        assert!((end_pos.x - 10.0).abs() < 1e-3);
        assert!((end_pos.y - 5.0).abs() < 1e-3);
    }

    #[test]
    fn test_aim_constraint() {
        let mut scene = SceneGraph::new();
        let eye_id = scene.add_node(SceneNode::new_transform("Eye", Transform::IDENTITY));
        let target_id = scene.add_node(SceneNode::new_transform(
            "Target",
            Transform {
                translation: vec3(0.0, 0.0, 10.0),
                ..Default::default()
            },
        ));

        let constraint = AimConstraint::new(eye_id, target_id);
        constraint.evaluate(&mut scene);

        // When target is along +Z, aim rotation should point forward
        let fwd = scene.nodes[eye_id]
            .local_transform
            .rotation
            .mul_vec3(Vec3::Z);
        assert!((fwd.z - 1.0).abs() < 1e-3);
    }

    #[test]
    fn test_parent_constraint() {
        let mut scene = SceneGraph::new();
        let target_id = scene.add_node(SceneNode::new_transform(
            "ParentTarget",
            Transform {
                translation: vec3(3.0, 4.0, 5.0),
                ..Default::default()
            },
        ));
        let follower_id = scene.add_node(SceneNode::new_transform("Follower", Transform::IDENTITY));

        let constraint = ParentConstraint::new(follower_id, target_id);
        constraint.evaluate(&mut scene);

        assert_eq!(
            scene.nodes[follower_id].local_transform.translation,
            vec3(3.0, 4.0, 5.0)
        );
    }

    #[test]
    fn test_constraint_stack_evaluation() {
        let mut scene = SceneGraph::new();
        let target_id = scene.add_node(SceneNode::new_transform(
            "Target",
            Transform {
                translation: vec3(1.0, 2.0, 3.0),
                ..Default::default()
            },
        ));
        let follower_id = scene.add_node(SceneNode::new_transform("Follower", Transform::IDENTITY));

        let mut stack = ConstraintStack::new();
        stack.add(Constraint::Parent(ParentConstraint::new(
            follower_id,
            target_id,
        )));
        stack.evaluate(&mut scene);

        assert_eq!(
            scene.nodes[follower_id].local_transform.translation,
            vec3(1.0, 2.0, 3.0)
        );
    }
}

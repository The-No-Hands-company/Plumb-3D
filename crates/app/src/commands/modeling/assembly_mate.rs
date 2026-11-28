//! Undoable Command for Adding and Solving CAD Kinematic Mates.

use core_math::Transform;
use geo_kernel::{AssemblySolver, KinematicMate};
use graph_engine::{Command, NodeId, SceneGraph};

#[derive(Debug)]
pub struct AddKinematicMateCommand {
    parent_node: NodeId,
    child_node: NodeId,
    mate: KinematicMate,
    before_child_transform: Option<Transform>,
}

impl AddKinematicMateCommand {
    pub fn new(parent_node: NodeId, child_node: NodeId, mate: KinematicMate) -> Self {
        Self {
            parent_node,
            child_node,
            mate,
            before_child_transform: None,
        }
    }
}

impl Command for AddKinematicMateCommand {
    fn name(&self) -> &str {
        "Add Kinematic Mate"
    }

    fn execute(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        let parent = scene
            .nodes
            .get(self.parent_node)
            .ok_or_else(|| format!("Parent node {:?} not found", self.parent_node))?;
        let parent_tf = parent.local_transform;

        let child = scene
            .nodes
            .get_mut(self.child_node)
            .ok_or_else(|| format!("Child node {:?} not found", self.child_node))?;

        if self.before_child_transform.is_none() {
            self.before_child_transform = Some(child.local_transform);
        }

        let new_child_tf = AssemblySolver::compute_child_transform(
            &parent_tf,
            &self.mate.first_connector,
            &self.mate.second_connector,
            self.mate.mate_type,
            self.mate.alignment,
            self.mate.offset,
            self.mate.current_param,
        );

        child.local_transform = new_child_tf;
        scene.mark_dirty(self.child_node);
        Ok(())
    }

    fn undo(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        let before_tf = self
            .before_child_transform
            .ok_or("No previous transform available for undo")?;

        let child = scene
            .nodes
            .get_mut(self.child_node)
            .ok_or_else(|| format!("Child node {:?} not found", self.child_node))?;

        child.local_transform = before_tf;
        scene.mark_dirty(self.child_node);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core_math::Vec3;
    use geo_kernel::{MateAlignment, MateConnector, MateType, Primitives};
    use graph_engine::SceneNode;

    #[test]
    fn test_add_mate_command_execution_and_undo() {
        let mut scene = SceneGraph::new();

        let base_mesh = Primitives::create_cube(2.0);
        let n0 = scene.add_node(SceneNode::new_mesh(
            "Base",
            Transform::IDENTITY,
            base_mesh,
        ));

        let arm_mesh = Primitives::create_cube(1.0);
        let orig_arm_tf = Transform::from_translation(Vec3::new(100.0, 100.0, 100.0));
        let n1 = scene.add_node(SceneNode::new_mesh(
            "Arm",
            orig_arm_tf,
            arm_mesh,
        ));

        let conn0 = MateConnector::new(Vec3::new(0.0, 0.0, 1.0), Vec3::Z, Vec3::X);
        let conn1 = MateConnector::new(Vec3::ZERO, Vec3::Z, Vec3::X);

        let mut mate = KinematicMate::new(
            0,
            "BaseArmMate",
            0,
            conn0,
            1,
            conn1,
            MateType::Revolute { limits: None },
        );
        mate.alignment = MateAlignment::Aligned;

        let mut cmd = AddKinematicMateCommand::new(n0, n1, mate);

        assert!(cmd.execute(&mut scene).is_ok());

        let arm = scene.nodes.get(n1).unwrap();
        assert!((arm.local_transform.translation - Vec3::new(0.0, 0.0, 1.0)).length() < 1e-4);

        assert!(cmd.undo(&mut scene).is_ok());

        let arm_restored = scene.nodes.get(n1).unwrap();
        assert!((arm_restored.local_transform.translation - Vec3::new(100.0, 100.0, 100.0)).length() < 1e-4);
    }
}

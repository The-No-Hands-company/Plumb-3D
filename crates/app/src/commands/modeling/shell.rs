//! Undoable Command for CAD Solid Shell & Uniform Thicken.

use geo_kernel::{Mesh, ShellSettings};
use graph_engine::{Command, NodeData, NodeId, SceneGraph};

#[derive(Debug)]
pub struct ShellSolidCommand {
    node_id: NodeId,
    settings: ShellSettings,
    before_mesh: Option<Mesh>,
}

impl ShellSolidCommand {
    pub fn new(node_id: NodeId, settings: ShellSettings) -> Self {
        Self {
            node_id,
            settings,
            before_mesh: None,
        }
    }
}

impl Command for ShellSolidCommand {
    fn name(&self) -> &str {
        "Shell Solid"
    }

    fn execute(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        let node = scene
            .nodes
            .get_mut(self.node_id)
            .ok_or_else(|| format!("Node {:?} not found", self.node_id))?;

        let mesh = match &mut node.data {
            NodeData::Mesh { base_mesh, .. } => base_mesh,
            _ => return Err("Target node is not a polygon mesh".to_string()),
        };

        if self.before_mesh.is_none() {
            self.before_mesh = Some(mesh.clone());
        }

        let shelled = mesh
            .shell_solid(&self.settings)
            .map_err(|e| format!("Shell operation failed: {:?}", e))?;

        *mesh = shelled;
        Ok(())
    }

    fn undo(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        let before = self
            .before_mesh
            .as_ref()
            .ok_or("No before state available for undo")?;

        let node = scene
            .nodes
            .get_mut(self.node_id)
            .ok_or_else(|| format!("Node {:?} not found", self.node_id))?;

        match &mut node.data {
            NodeData::Mesh { base_mesh, .. } => {
                *base_mesh = before.clone();
                Ok(())
            }
            _ => Err("Target node is not a polygon mesh".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core_math::Transform;
    use geo_kernel::primitives::Primitives;
    use graph_engine::SceneNode;

    #[test]
    fn test_shell_command_and_undo() {
        let mut scene = SceneGraph::new();
        let cube = Primitives::create_cube(2.0);
        let node_id = scene.add_node(SceneNode::new_mesh(
            "TestCube",
            Transform::IDENTITY,
            cube,
        ));

        let settings = ShellSettings::default();
        let mut cmd = ShellSolidCommand::new(node_id, settings);

        assert!(cmd.execute(&mut scene).is_ok());
        assert!(cmd.undo(&mut scene).is_ok());
    }
}

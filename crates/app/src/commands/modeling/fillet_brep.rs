//! Undoable Command for B-Rep Rolling-Ball & Chordal Fillet blending.

use geo_kernel::{EdgeKey, Mesh, RollingBallFilletSettings};
use graph_engine::{Command, NodeData, NodeId, SceneGraph};

#[derive(Debug)]
pub struct RollingBallFilletCommand {
    node_id: NodeId,
    edges: Vec<EdgeKey>,
    settings: RollingBallFilletSettings,
    before_mesh: Option<Mesh>,
}

impl RollingBallFilletCommand {
    pub fn new(node_id: NodeId, edges: Vec<EdgeKey>, settings: RollingBallFilletSettings) -> Self {
        Self {
            node_id,
            edges,
            settings,
            before_mesh: None,
        }
    }
}

impl Command for RollingBallFilletCommand {
    fn name(&self) -> &str {
        "Rolling-Ball Fillet"
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

        mesh.apply_rolling_ball_fillet(&self.edges, &self.settings)
            .map_err(|e| format!("Rolling-ball fillet failed: {:?}", e))?;

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
    fn test_rolling_ball_fillet_command_and_undo() {
        let mut scene = SceneGraph::new();
        let cube = Primitives::create_cube(2.0);
        let edge_key = cube.edges.keys().next().unwrap();
        let node_id = scene.add_node(SceneNode::new_mesh(
            "TestCube",
            Transform::IDENTITY,
            cube,
        ));

        let settings = RollingBallFilletSettings::default();
        let mut cmd = RollingBallFilletCommand::new(node_id, vec![edge_key], settings);

        assert!(cmd.execute(&mut scene).is_ok());
        assert!(cmd.undo(&mut scene).is_ok());
    }
}

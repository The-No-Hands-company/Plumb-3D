use geo_kernel::Mesh;
use graph_engine::{Command, NodeData, NodeId, SceneGraph};

/// Command to project a mesh node's vertices to exact Pixar Catmull-Clark limit surface with Undo/Redo history.
#[derive(Debug)]
pub struct SubdLimitCommand {
    pub node_id: NodeId,
    pub old_mesh: Option<Mesh>,
}

impl SubdLimitCommand {
    pub fn new(node_id: NodeId) -> Self {
        Self {
            node_id,
            old_mesh: None,
        }
    }
}

impl Command for SubdLimitCommand {
    fn name(&self) -> &str {
        "Evaluate SubD Limit Surface"
    }

    fn execute(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        let node = scene.nodes.get_mut(self.node_id).ok_or("Node not found")?;
        match &mut node.data {
            NodeData::Mesh { base_mesh, .. } => {
                let mut limit_mesh = base_mesh.clone();
                limit_mesh.project_to_subd_limit();
                self.old_mesh = Some(std::mem::replace(base_mesh, limit_mesh));
                scene.mark_dirty(self.node_id);
                Ok(())
            }
            _ => Err("Node is not a mesh".into()),
        }
    }

    fn undo(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        if let Some(old) = self.old_mesh.take() {
            let node = scene.nodes.get_mut(self.node_id).ok_or("Node not found")?;
            if let NodeData::Mesh { base_mesh, .. } = &mut node.data {
                *base_mesh = old;
                scene.mark_dirty(self.node_id);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use geo_kernel::{CatmullClarkModifier, Modifier, Primitives};
    use graph_engine::SceneNode;

    #[test]
    fn test_subd_limit_command_execution_and_undo() {
        let mut scene = SceneGraph::new();
        let cube = Primitives::create_cube(2.0);
        let sub = CatmullClarkModifier::new(1)
            .apply(&cube)
            .expect("subdivide");
        let orig_pos = sub.vertices.values().next().unwrap().position;

        let node_id = scene.add_node(SceneNode::new_mesh(
            "TestSubD",
            core_math::Transform::IDENTITY,
            sub,
        ));

        let mut cmd = SubdLimitCommand::new(node_id);
        assert!(cmd.execute(&mut scene).is_ok());

        // Check vertices were projected to limit surface
        if let NodeData::Mesh { base_mesh, .. } = &scene.nodes.get(node_id).unwrap().data {
            let limit_pos = base_mesh.vertices.values().next().unwrap().position;
            assert_ne!(orig_pos, limit_pos);
        }

        // Undo
        assert!(cmd.undo(&mut scene).is_ok());
        if let NodeData::Mesh { base_mesh, .. } = &scene.nodes.get(node_id).unwrap().data {
            let restored_pos = base_mesh.vertices.values().next().unwrap().position;
            assert_eq!(orig_pos, restored_pos);
        }
    }
}

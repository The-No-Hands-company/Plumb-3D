use geo_kernel::{FieldRemeshSettings, Mesh};
use graph_engine::{Command, NodeData, NodeId, SceneGraph};

/// Command to perform Curvature Field-Aligned Quad Retopology on a mesh node with full Undo/Redo history.
#[derive(Debug)]
pub struct FieldRemeshCommand {
    pub node_id: NodeId,
    pub settings: FieldRemeshSettings,
    pub old_mesh: Option<Mesh>,
}

impl FieldRemeshCommand {
    pub fn new(node_id: NodeId, settings: FieldRemeshSettings) -> Self {
        Self {
            node_id,
            settings,
            old_mesh: None,
        }
    }
}

impl Command for FieldRemeshCommand {
    fn name(&self) -> &str {
        "Field-Aligned Quad Remesh"
    }

    fn execute(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        let node = scene.nodes.get_mut(self.node_id).ok_or("Node not found")?;
        match &mut node.data {
            NodeData::Mesh { base_mesh, .. } => {
                let remeshed = base_mesh
                    .remesh_field_aligned(&self.settings)
                    .map_err(|e| format!("Field quad remesh failed: {:?}", e))?;
                self.old_mesh = Some(std::mem::replace(base_mesh, remeshed));
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
    use geo_kernel::Primitives;
    use graph_engine::SceneNode;

    #[test]
    fn test_field_remesh_command_execution_and_undo() {
        let mut scene = SceneGraph::new();
        let cube = Primitives::create_cube(2.0);
        let node_id = scene.add_node(SceneNode::new_mesh(
            "TestCube",
            core_math::Transform::IDENTITY,
            cube,
        ));

        let mut cmd = FieldRemeshCommand::new(
            node_id,
            FieldRemeshSettings {
                target_edge_length: 0.5,
                curvature_influence: 0.8,
                smooth_passes: 2,
            },
        );

        assert!(cmd.execute(&mut scene).is_ok());

        // Check node was remeshed to quads
        if let NodeData::Mesh { base_mesh, .. } = &scene.nodes.get(node_id).unwrap().data {
            assert!(base_mesh.faces.len() > 6);
        }

        // Undo
        assert!(cmd.undo(&mut scene).is_ok());
        if let NodeData::Mesh { base_mesh, .. } = &scene.nodes.get(node_id).unwrap().data {
            assert_eq!(base_mesh.faces.len(), 6);
        }
    }
}

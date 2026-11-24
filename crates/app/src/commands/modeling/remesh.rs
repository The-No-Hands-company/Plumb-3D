use geo_kernel::Mesh;
use graph_engine::{Command, NodeData, NodeId, SceneGraph};

/// Command to destructively voxel-remesh a mesh node with full Undo/Redo history.
#[derive(Debug)]
pub struct VoxelRemeshCommand {
    pub node_id: NodeId,
    pub voxel_size: f32,
    pub old_mesh: Option<Mesh>,
}

impl Command for VoxelRemeshCommand {
    fn name(&self) -> &str {
        "Voxel Remesh (Ctrl+R)"
    }

    fn execute(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        let node = scene.nodes.get_mut(self.node_id).ok_or("Node not found")?;
        match &mut node.data {
            NodeData::Mesh { base_mesh, .. } => {
                let remeshed = base_mesh
                    .voxel_remesh(self.voxel_size)
                    .map_err(|e| format!("Voxel remesh failed: {:?}", e))?;
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

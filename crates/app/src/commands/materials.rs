use geo_kernel::FaceKey;
use graph_engine::{Command, Material, NodeData, NodeId, SceneGraph};

#[derive(Debug)]
pub struct AssignMaterialCommand {
    pub node_id: NodeId,
    pub faces: Vec<FaceKey>,
    pub material_id: u32,
    pub old_assignments: Vec<(FaceKey, u32)>,
}


impl Command for AssignMaterialCommand {
    fn name(&self) -> &str {
        "Assign Material to Faces"
    }

    fn execute(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        let node = scene.nodes.get_mut(self.node_id).ok_or("Node not found")?;
        match &mut node.data {
            NodeData::Mesh { base_mesh, cached_render_mesh, .. } => {
                if self.old_assignments.is_empty() {
                    self.old_assignments = self.faces.iter().filter_map(|&fk| {
                        base_mesh.faces.get(fk).map(|f| (fk, f.material_id))
                    }).collect();
                }
                base_mesh.set_faces_material(&self.faces, self.material_id);
                *cached_render_mesh = None;
                scene.mark_dirty(self.node_id);
                Ok(())
            }
            _ => Err("Node is not a mesh".into()),
        }
    }

    fn undo(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        let node = scene.nodes.get_mut(self.node_id).ok_or("Node not found")?;
        match &mut node.data {
            NodeData::Mesh { base_mesh, cached_render_mesh, .. } => {
                for &(fk, old_mat_id) in &self.old_assignments {
                    if let Some(f) = base_mesh.faces.get_mut(fk) {
                        f.material_id = old_mat_id;
                    }
                }
                *cached_render_mesh = None;
                scene.mark_dirty(self.node_id);
                Ok(())
            }
            _ => Err("Node is not a mesh".into()),
        }
    }
}


#[derive(Debug)]
pub struct AddMaterialSlotCommand {
    pub node_id: NodeId,
    pub material: Material,
}


impl Command for AddMaterialSlotCommand {
    fn name(&self) -> &str {
        "Add Material Slot"
    }

    fn execute(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        let node = scene.nodes.get_mut(self.node_id).ok_or("Node not found")?;
        match &mut node.data {
            NodeData::Mesh { materials, .. } => {
                materials.push(self.material.clone());
                Ok(())
            }
            _ => Err("Node is not a mesh".into()),
        }
    }

    fn undo(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        let node = scene.nodes.get_mut(self.node_id).ok_or("Node not found")?;
        match &mut node.data {
            NodeData::Mesh { materials, .. } => {
                materials.pop();
                Ok(())
            }
            _ => Err("Node is not a mesh".into()),
        }
    }
}


#[derive(Debug)]
pub struct RemoveMaterialSlotCommand {
    pub node_id: NodeId,
    pub slot_index: usize,
    pub removed_material: Option<Material>,
}


impl Command for RemoveMaterialSlotCommand {
    fn name(&self) -> &str {
        "Remove Material Slot"
    }

    fn execute(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        let node = scene.nodes.get_mut(self.node_id).ok_or("Node not found")?;
        match &mut node.data {
            NodeData::Mesh { materials, cached_render_mesh, .. } => {
                if materials.len() <= 1 {
                    return Err("Cannot remove only material slot".into());
                }
                if self.slot_index >= materials.len() {
                    return Err("Slot index out of bounds".into());
                }
                self.removed_material = Some(materials.remove(self.slot_index));
                *cached_render_mesh = None;
                scene.mark_dirty(self.node_id);
                Ok(())
            }
            _ => Err("Node is not a mesh".into()),
        }
    }

    fn undo(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        if let Some(mat) = self.removed_material.take() {
            let node = scene.nodes.get_mut(self.node_id).ok_or("Node not found")?;
            match &mut node.data {
                NodeData::Mesh { materials, cached_render_mesh, .. } => {
                    let idx = self.slot_index.min(materials.len());
                    materials.insert(idx, mat);
                    *cached_render_mesh = None;
                    scene.mark_dirty(self.node_id);
                    Ok(())
                }
                _ => Err("Node is not a mesh".into()),
            }
        } else {
            Err("No removed material to restore".into())
        }
    }
}


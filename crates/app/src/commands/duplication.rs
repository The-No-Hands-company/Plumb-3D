use core_math::Vec3;
use graph_engine::{Command, NodeId, SceneGraph, SceneNode};

/// Command to duplicate a scene node with an optional transform offset and full undo/redo.
#[derive(Debug)]
pub struct DuplicateNodeCommand {
    pub source_id: NodeId,
    pub offset: Vec3,
    pub created_node_id: Option<NodeId>,
    pub cached_duplicate: Option<SceneNode>,
}

impl DuplicateNodeCommand {
    pub fn new(source_id: NodeId, offset: Vec3) -> Self {
        Self {
            source_id,
            offset,
            created_node_id: None,
            cached_duplicate: None,
        }
    }
}

impl Command for DuplicateNodeCommand {
    fn name(&self) -> &str {
        "Duplicate Object (Shift+D)"
    }

    fn execute(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        if let Some(cached) = self.cached_duplicate.as_ref() {
            let parent = cached.parent;
            let id = scene.add_node(cached.duplicate(cached.name.clone(), Vec3::ZERO));
            if let Some(p) = parent {
                scene.set_parent(id, Some(p));
            }
            self.created_node_id = Some(id);
            scene.update_transforms();
            return Ok(());
        }

        let source_node = scene
            .nodes
            .get(self.source_id)
            .ok_or_else(|| "Source node not found in scene".to_string())?;

        let dup_name = format!("{}_copy", source_node.name);
        let dup_node = source_node.duplicate(dup_name.clone(), self.offset);
        let parent = dup_node.parent;
        self.cached_duplicate = Some(dup_node.duplicate(dup_name, Vec3::ZERO));

        let new_id = scene.add_node(dup_node);
        if let Some(p) = parent {
            scene.set_parent(new_id, Some(p));
        }

        self.created_node_id = Some(new_id);
        scene.update_transforms();
        Ok(())
    }

    fn undo(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        if let Some(id) = self.created_node_id.take() {
            scene.remove_node(id);
            scene.update_transforms();
        }
        Ok(())
    }
}

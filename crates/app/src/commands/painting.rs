use core_math::Vec3;
use geo_kernel::{Mesh, MirrorAxis};
use graph_engine::{Command, NodeData, NodeId, SceneGraph};

#[derive(Debug)]
pub struct VertexPaintStrokeCommand {
    pub node_id: NodeId,
    pub old_mesh: Mesh,
    pub new_mesh: Mesh,
}

impl Command for VertexPaintStrokeCommand {
    fn name(&self) -> &str {
        "Vertex Paint Stroke"
    }

    fn execute(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        let node = scene.nodes.get_mut(self.node_id).ok_or("Node not found")?;
        match &mut node.data {
            NodeData::Mesh {
                base_mesh,
                cached_render_mesh,
                ..
            } => {
                *base_mesh = self.new_mesh.clone();
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
            NodeData::Mesh {
                base_mesh,
                cached_render_mesh,
                ..
            } => {
                *base_mesh = self.old_mesh.clone();
                *cached_render_mesh = None;
                scene.mark_dirty(self.node_id);
                Ok(())
            }
            _ => Err("Node is not a mesh".into()),
        }
    }
}

#[derive(Debug)]
pub struct FillVertexColorsCommand {
    pub node_id: NodeId,
    pub color: [f32; 4],
    pub old_mesh: Option<Mesh>,
}

impl Command for FillVertexColorsCommand {
    fn name(&self) -> &str {
        "Fill Vertex Colors"
    }

    fn execute(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        let node = scene.nodes.get_mut(self.node_id).ok_or("Node not found")?;
        match &mut node.data {
            NodeData::Mesh {
                base_mesh,
                cached_render_mesh,
                ..
            } => {
                if self.old_mesh.is_none() {
                    self.old_mesh = Some(base_mesh.clone());
                }
                base_mesh.fill_vertex_colors(self.color);
                *cached_render_mesh = None;
                scene.mark_dirty(self.node_id);
                Ok(())
            }
            _ => Err("Node is not a mesh".into()),
        }
    }

    fn undo(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        if let Some(old) = &self.old_mesh {
            let node = scene.nodes.get_mut(self.node_id).ok_or("Node not found")?;
            match &mut node.data {
                NodeData::Mesh {
                    base_mesh,
                    cached_render_mesh,
                    ..
                } => {
                    *base_mesh = old.clone();
                    *cached_render_mesh = None;
                    scene.mark_dirty(self.node_id);
                    Ok(())
                }
                _ => Err("Node is not a mesh".into()),
            }
        } else {
            Err("No snapshot to restore".into())
        }
    }
}

#[derive(Debug)]
pub struct MirrorVertexGroupCommand {
    pub node_id: NodeId,
    pub source_group: String,
    pub target_group: String,
    pub axis: MirrorAxis,
    pub tolerance: f32,
    pub old_mesh: Option<Mesh>,
}

impl Command for MirrorVertexGroupCommand {
    fn name(&self) -> &str {
        "Mirror Vertex Group"
    }

    fn execute(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        let node = scene.nodes.get_mut(self.node_id).ok_or("Node not found")?;
        match &mut node.data {
            NodeData::Mesh {
                base_mesh,
                cached_render_mesh,
                ..
            } => {
                if self.old_mesh.is_none() {
                    self.old_mesh = Some(base_mesh.clone());
                }
                base_mesh
                    .mirror_vertex_group(
                        &self.source_group,
                        &self.target_group,
                        self.axis,
                        self.tolerance,
                    )
                    .map_err(|e| format!("Mirror vertex group failed: {:?}", e))?;
                *cached_render_mesh = None;
                scene.mark_dirty(self.node_id);
                Ok(())
            }
            _ => Err("Node is not a mesh".into()),
        }
    }

    fn undo(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        if let Some(old) = &self.old_mesh {
            let node = scene.nodes.get_mut(self.node_id).ok_or("Node not found")?;
            match &mut node.data {
                NodeData::Mesh {
                    base_mesh,
                    cached_render_mesh,
                    ..
                } => {
                    *base_mesh = old.clone();
                    *cached_render_mesh = None;
                    scene.mark_dirty(self.node_id);
                    Ok(())
                }
                _ => Err("Node is not a mesh".into()),
            }
        } else {
            Err("No snapshot to restore".into())
        }
    }
}

#[derive(Debug)]
pub struct NormalizeVertexGroupsCommand {
    pub node_id: NodeId,
    pub group_names: Vec<String>,
    pub old_mesh: Option<Mesh>,
}

impl Command for NormalizeVertexGroupsCommand {
    fn name(&self) -> &str {
        "Normalize Vertex Groups"
    }

    fn execute(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        let node = scene.nodes.get_mut(self.node_id).ok_or("Node not found")?;
        match &mut node.data {
            NodeData::Mesh {
                base_mesh,
                cached_render_mesh,
                ..
            } => {
                if self.old_mesh.is_none() {
                    self.old_mesh = Some(base_mesh.clone());
                }
                base_mesh.normalize_vertex_groups(&self.group_names);
                *cached_render_mesh = None;
                scene.mark_dirty(self.node_id);
                Ok(())
            }
            _ => Err("Node is not a mesh".into()),
        }
    }

    fn undo(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        if let Some(old) = &self.old_mesh {
            let node = scene.nodes.get_mut(self.node_id).ok_or("Node not found")?;
            match &mut node.data {
                NodeData::Mesh {
                    base_mesh,
                    cached_render_mesh,
                    ..
                } => {
                    *base_mesh = old.clone();
                    *cached_render_mesh = None;
                    scene.mark_dirty(self.node_id);
                    Ok(())
                }
                _ => Err("Node is not a mesh".into()),
            }
        } else {
            Err("No snapshot to restore".into())
        }
    }
}

#[derive(Debug)]
pub struct ApplyWeightGradientCommand {
    pub node_id: NodeId,
    pub group_name: String,
    pub start_point: Vec3,
    pub end_point: Vec3,
    pub start_weight: f32,
    pub end_weight: f32,
    pub old_mesh: Option<Mesh>,
}

impl Command for ApplyWeightGradientCommand {
    fn name(&self) -> &str {
        "Apply Weight Gradient"
    }

    fn execute(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        let node = scene.nodes.get_mut(self.node_id).ok_or("Node not found")?;
        match &mut node.data {
            NodeData::Mesh {
                base_mesh,
                cached_render_mesh,
                ..
            } => {
                if self.old_mesh.is_none() {
                    self.old_mesh = Some(base_mesh.clone());
                }
                base_mesh.apply_weight_gradient(
                    &self.group_name,
                    self.start_point,
                    self.end_point,
                    self.start_weight,
                    self.end_weight,
                );
                *cached_render_mesh = None;
                scene.mark_dirty(self.node_id);
                Ok(())
            }
            _ => Err("Node is not a mesh".into()),
        }
    }

    fn undo(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        if let Some(old) = &self.old_mesh {
            let node = scene.nodes.get_mut(self.node_id).ok_or("Node not found")?;
            match &mut node.data {
                NodeData::Mesh {
                    base_mesh,
                    cached_render_mesh,
                    ..
                } => {
                    *base_mesh = old.clone();
                    *cached_render_mesh = None;
                    scene.mark_dirty(self.node_id);
                    Ok(())
                }
                _ => Err("Node is not a mesh".into()),
            }
        } else {
            Err("No snapshot to restore".into())
        }
    }
}

#[derive(Debug)]
pub struct WeightPaintStrokeCommand {
    pub node_id: NodeId,
    pub old_mesh: Mesh,
    pub new_mesh: Mesh,
}

impl Command for WeightPaintStrokeCommand {
    fn name(&self) -> &str {
        "Weight Paint Stroke"
    }

    fn execute(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        let node = scene.nodes.get_mut(self.node_id).ok_or("Node not found")?;
        match &mut node.data {
            NodeData::Mesh {
                base_mesh,
                cached_render_mesh,
                ..
            } => {
                *base_mesh = self.new_mesh.clone();
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
            NodeData::Mesh {
                base_mesh,
                cached_render_mesh,
                ..
            } => {
                *base_mesh = self.old_mesh.clone();
                *cached_render_mesh = None;
                scene.mark_dirty(self.node_id);
                Ok(())
            }
            _ => Err("Node is not a mesh".into()),
        }
    }
}

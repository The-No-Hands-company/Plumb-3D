use geo_kernel::{EdgeKey, Mesh, VertexKey};
use graph_engine::{Command, NodeData, NodeId, SceneGraph};

#[derive(Debug)]
pub struct SculptStrokeCommand {
    pub node_id: NodeId,
    pub brush_name: String,
    pub old_mesh: Mesh,
    pub new_mesh: Mesh,
}


impl Command for SculptStrokeCommand {
    fn name(&self) -> &str {
        &self.brush_name
    }

    fn execute(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        if let Some(node) = scene.nodes.get_mut(self.node_id) {
            if let NodeData::Mesh { base_mesh, .. } = &mut node.data {
                *base_mesh = self.new_mesh.clone();
                scene.mark_dirty(self.node_id);
                Ok(())
            } else {
                Err("Node is not a mesh".into())
            }
        } else {
            Err("Node not found".into())
        }
    }

    fn undo(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        if let Some(node) = scene.nodes.get_mut(self.node_id) {
            if let NodeData::Mesh { base_mesh, .. } = &mut node.data {
                *base_mesh = self.old_mesh.clone();
                scene.mark_dirty(self.node_id);
                Ok(())
            } else {
                Err("Node is not a mesh".into())
            }
        } else {
            Err("Node not found".into())
        }
    }
}


#[derive(Debug)]
pub struct ShrinkFattenCommand {
    pub node_id: NodeId,
    pub vertices: Vec<VertexKey>,
    pub offset: f32,
    pub old_mesh: Option<Mesh>,
}


impl Command for ShrinkFattenCommand {
    fn name(&self) -> &str {
        "Shrink/Fatten"
    }

    fn execute(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        let node = scene.nodes.get_mut(self.node_id).ok_or("Node not found")?;
        match &mut node.data {
            NodeData::Mesh { base_mesh, cached_render_mesh, .. } => {
                if self.old_mesh.is_none() {
                    self.old_mesh = Some(base_mesh.clone());
                }
                base_mesh.shrink_fatten(&self.vertices, self.offset)
                    .map_err(|e| format!("Shrink/fatten failed: {:?}", e))?;
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
                NodeData::Mesh { base_mesh, cached_render_mesh, .. } => {
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
pub struct RelaxVerticesCommand {
    pub node_id: NodeId,
    pub vertices: Vec<VertexKey>,
    pub iterations: usize,
    pub factor: f32,
    pub old_mesh: Option<Mesh>,
}


impl Command for RelaxVerticesCommand {
    fn name(&self) -> &str {
        "Relax Vertices"
    }

    fn execute(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        let node = scene.nodes.get_mut(self.node_id).ok_or("Node not found")?;
        match &mut node.data {
            NodeData::Mesh { base_mesh, cached_render_mesh, .. } => {
                if self.old_mesh.is_none() {
                    self.old_mesh = Some(base_mesh.clone());
                }
                base_mesh.relax_vertices(&self.vertices, self.iterations, self.factor)
                    .map_err(|e| format!("Relax vertices failed: {:?}", e))?;
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
                NodeData::Mesh { base_mesh, cached_render_mesh, .. } => {
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
pub struct SlideEdgesCommand {
    pub node_id: NodeId,
    pub edges: Vec<EdgeKey>,
    pub vertices: Vec<VertexKey>,
    pub factor: f32,
    pub old_mesh: Option<Mesh>,
}


impl Command for SlideEdgesCommand {
    fn name(&self) -> &str {
        "Slide Edges/Vertices (G, G)"
    }

    fn execute(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        let node = scene.nodes.get_mut(self.node_id).ok_or("Node not found")?;
        match &mut node.data {
            NodeData::Mesh { base_mesh, cached_render_mesh, .. } => {
                if self.old_mesh.is_none() {
                    self.old_mesh = Some(base_mesh.clone());
                }
                if !self.edges.is_empty() {
                    base_mesh.slide_edges(&self.edges, self.factor)
                        .map_err(|e| format!("Slide edges failed: {:?}", e))?;
                } else if !self.vertices.is_empty() {
                    base_mesh.slide_vertices(&self.vertices, self.factor)
                        .map_err(|e| format!("Slide vertices failed: {:?}", e))?;
                }
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
                NodeData::Mesh { base_mesh, cached_render_mesh, .. } => {
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




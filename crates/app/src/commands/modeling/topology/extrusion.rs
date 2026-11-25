use geo_kernel::{EdgeKey, FaceKey, Mesh, VertexKey};
use graph_engine::{Command, NodeData, NodeId, SceneGraph};

#[derive(Debug)]
pub struct ExtrudeCommand {
    pub node_id: NodeId,
    pub faces: Vec<FaceKey>,
    pub distance: f32,
    pub old_mesh: Option<Mesh>,
}

impl Command for ExtrudeCommand {
    fn name(&self) -> &str {
        "Extrude Face(s)"
    }

    fn execute(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        if let Some(node) = scene.nodes.get_mut(self.node_id) {
            if let NodeData::Mesh { base_mesh, .. } = &mut node.data {
                if self.old_mesh.is_none() {
                    self.old_mesh = Some(base_mesh.clone());
                }
                for &fk in &self.faces {
                    if base_mesh.faces.contains_key(fk) {
                        let _ = base_mesh.extrude_face(fk, self.distance);
                    }
                }
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
        if let Some(old) = &self.old_mesh {
            if let Some(node) = scene.nodes.get_mut(self.node_id) {
                if let NodeData::Mesh { base_mesh, .. } = &mut node.data {
                    *base_mesh = old.clone();
                    scene.mark_dirty(self.node_id);
                    Ok(())
                } else {
                    Err("Node is not a mesh".into())
                }
            } else {
                Err("Node not found".into())
            }
        } else {
            Err("No old mesh snapshot to restore".into())
        }
    }
}

#[derive(Debug)]
pub struct InsetCommand {
    pub node_id: NodeId,
    pub faces: Vec<FaceKey>,
    pub factor: f32,
    pub individual: bool,
    pub old_mesh: Option<Mesh>,
    pub created_faces: Vec<FaceKey>,
}

impl Command for InsetCommand {
    fn name(&self) -> &str {
        "Inset Faces (I)"
    }

    fn execute(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        let node = scene.nodes.get_mut(self.node_id).ok_or("Node not found")?;
        match &mut node.data {
            NodeData::Mesh { base_mesh, cached_render_mesh, .. } => {
                if self.old_mesh.is_none() {
                    self.old_mesh = Some(base_mesh.clone());
                }
                self.created_faces.clear();
                let valid_faces: Vec<FaceKey> = self.faces.iter()
                    .copied()
                    .filter(|fk| base_mesh.faces.contains_key(*fk))
                    .collect();

                self.created_faces = base_mesh.inset_faces(&valid_faces, self.factor, self.individual)
                    .map_err(|e| format!("Inset failed: {:?}", e))?;

                base_mesh.update_normals();
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
pub struct BevelCommand {
    pub node_id: NodeId,
    pub faces: Vec<FaceKey>,
    pub offset: f32,
    pub inset: f32,
    pub old_mesh: Option<Mesh>,
}

impl Command for BevelCommand {
    fn name(&self) -> &str {
        "Bevel Face(s)"
    }

    fn execute(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        if let Some(node) = scene.nodes.get_mut(self.node_id) {
            if let NodeData::Mesh { base_mesh, .. } = &mut node.data {
                if self.old_mesh.is_none() {
                    self.old_mesh = Some(base_mesh.clone());
                }
                for &fk in &self.faces {
                    if base_mesh.faces.contains_key(fk) {
                        let _ = base_mesh.bevel_face(fk, self.offset, self.inset);
                    }
                }
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
        if let Some(old) = &self.old_mesh {
            if let Some(node) = scene.nodes.get_mut(self.node_id) {
                if let NodeData::Mesh { base_mesh, .. } = &mut node.data {
                    *base_mesh = old.clone();
                    scene.mark_dirty(self.node_id);
                    Ok(())
                } else {
                    Err("Node is not a mesh".into())
                }
            } else {
                Err("Node not found".into())
            }
        } else {
            Err("No old mesh snapshot to restore".into())
        }
    }
}

#[derive(Debug)]
pub struct BevelEdgeCommand {
    pub node_id: NodeId,
    pub edges: Vec<EdgeKey>,
    pub width: f32,
    pub segments: usize,
    pub old_mesh: Option<Mesh>,
}

impl Command for BevelEdgeCommand {
    fn name(&self) -> &str {
        "Bevel Edge(s)"
    }

    fn execute(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        if let Some(node) = scene.nodes.get_mut(self.node_id) {
            if let NodeData::Mesh { base_mesh, .. } = &mut node.data {
                if self.old_mesh.is_none() {
                    self.old_mesh = Some(base_mesh.clone());
                }
                let _ = base_mesh.bevel_edges_miter(&self.edges, self.width, self.segments);
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
        if let Some(old) = &self.old_mesh {
            if let Some(node) = scene.nodes.get_mut(self.node_id) {
                if let NodeData::Mesh { base_mesh, .. } = &mut node.data {
                    *base_mesh = old.clone();
                    scene.mark_dirty(self.node_id);
                    Ok(())
                } else {
                    Err("Node is not a mesh".into())
                }
            } else {
                Err("Node not found".into())
            }
        } else {
            Err("No snapshot to restore".into())
        }
    }
}

#[derive(Debug)]
pub struct BevelVertexCommand {
    pub node_id: NodeId,
    pub vertices: Vec<VertexKey>,
    pub offset: f32,
    pub old_mesh: Option<Mesh>,
}

impl Command for BevelVertexCommand {
    fn name(&self) -> &str {
        "Bevel Vertex (Ctrl+Shift+B)"
    }

    fn execute(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        let node = scene.nodes.get_mut(self.node_id).ok_or("Node not found")?;
        match &mut node.data {
            NodeData::Mesh { base_mesh, cached_render_mesh, .. } => {
                if self.old_mesh.is_none() {
                    self.old_mesh = Some(base_mesh.clone());
                }
                base_mesh.bevel_vertices(&self.vertices, self.offset)
                    .map_err(|e| format!("Bevel vertex failed: {:?}", e))?;
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

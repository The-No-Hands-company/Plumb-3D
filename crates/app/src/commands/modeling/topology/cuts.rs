use geo_kernel::{CutPoint, EdgeKey, FaceKey, Mesh, VertexKey};
use graph_engine::{Command, NodeData, NodeId, SceneGraph};

#[derive(Debug)]
pub struct SplitQuadCommand {
    pub node_id: NodeId,
    pub face_key: FaceKey,
    pub ratio: f32,
    pub old_mesh: Option<Mesh>,
}

impl Command for SplitQuadCommand {
    fn name(&self) -> &str {
        "Split Quad / Loop Cut"
    }

    fn execute(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        if let Some(node) = scene.nodes.get_mut(self.node_id) {
            if let NodeData::Mesh { base_mesh, .. } = &mut node.data {
                if self.old_mesh.is_none() {
                    self.old_mesh = Some(base_mesh.clone());
                }
                base_mesh
                    .split_quad_face(self.face_key, 0, self.ratio)
                    .map_err(|e| format!("Split quad failed: {:?}", e))?;
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

#[derive(Debug, Clone)]
pub struct LoopCutCommand {
    pub node_id: NodeId,
    pub edge_key: EdgeKey,
    pub ratio: f32,
    pub num_cuts: usize,
    pub old_mesh: Option<Mesh>,
}

impl LoopCutCommand {
    pub fn new(node_id: NodeId, edge_key: EdgeKey, ratio: f32) -> Self {
        Self {
            node_id,
            edge_key,
            ratio,
            num_cuts: 1,
            old_mesh: None,
        }
    }
}

impl Command for LoopCutCommand {
    fn name(&self) -> &str {
        "Loop Cut Ring"
    }

    fn execute(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        if let Some(node) = scene.nodes.get_mut(self.node_id) {
            if let NodeData::Mesh { base_mesh, .. } = &mut node.data {
                if self.old_mesh.is_none() {
                    self.old_mesh = Some(base_mesh.clone());
                }
                base_mesh
                    .loop_cut_multi(self.edge_key, self.num_cuts, self.ratio)
                    .map_err(|e| format!("Loop cut failed: {:?}", e))?;
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
pub struct ConnectVerticesCommand {
    pub node_id: NodeId,
    pub face_key: FaceKey,
    pub v0: VertexKey,
    pub v1: VertexKey,
    pub old_mesh: Option<Mesh>,
}

impl Command for ConnectVerticesCommand {
    fn name(&self) -> &str {
        "Connect Vertices (J)"
    }

    fn execute(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        if let Some(node) = scene.nodes.get_mut(self.node_id) {
            if let NodeData::Mesh { base_mesh, .. } = &mut node.data {
                if self.old_mesh.is_none() {
                    self.old_mesh = Some(base_mesh.clone());
                }
                base_mesh
                    .connect_vertices(self.face_key, self.v0, self.v1)
                    .map_err(|e| format!("Connect vertices failed: {:?}", e))?;
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
pub struct SplitEdgeCommand {
    pub node_id: NodeId,
    pub edge_key: EdgeKey,
    pub ratio: f32,
    pub old_mesh: Option<Mesh>,
}

impl Command for SplitEdgeCommand {
    fn name(&self) -> &str {
        "Split Edge"
    }

    fn execute(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        if let Some(node) = scene.nodes.get_mut(self.node_id) {
            if let NodeData::Mesh { base_mesh, .. } = &mut node.data {
                if self.old_mesh.is_none() {
                    self.old_mesh = Some(base_mesh.clone());
                }
                base_mesh
                    .split_edge(self.edge_key, self.ratio)
                    .map_err(|e| format!("Split edge failed: {:?}", e))?;
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
pub struct KnifeCutCommand {
    pub node_id: NodeId,
    pub cuts: Vec<(FaceKey, CutPoint, CutPoint)>,
    pub old_mesh: Option<Mesh>,
}

impl Command for KnifeCutCommand {
    fn name(&self) -> &str {
        "Knife Cut"
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
                for &(face_key, start, end) in &self.cuts {
                    let _ = base_mesh.knife_cut(face_key, start, end);
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

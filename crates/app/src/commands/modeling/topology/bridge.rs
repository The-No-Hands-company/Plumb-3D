use geo_kernel::{EdgeKey, FaceKey, Mesh, VertexKey};
use graph_engine::{Command, NodeData, NodeId, SceneGraph};

#[derive(Debug)]
pub struct BridgeEdgeLoopsCommand {
    pub node_id: NodeId,
    pub selected_edges: Vec<EdgeKey>,
    pub loop_a: Option<Vec<VertexKey>>,
    pub loop_b: Option<Vec<VertexKey>>,
    pub twist: i32,
    pub cuts: usize,
    pub created_faces: Vec<FaceKey>,
    pub old_mesh: Option<Mesh>,
}

impl BridgeEdgeLoopsCommand {
    pub fn from_edges(
        node_id: NodeId,
        selected_edges: Vec<EdgeKey>,
        twist: i32,
        cuts: usize,
    ) -> Self {
        Self {
            node_id,
            selected_edges,
            loop_a: None,
            loop_b: None,
            twist,
            cuts,
            created_faces: Vec::new(),
            old_mesh: None,
        }
    }

    pub fn from_loops(
        node_id: NodeId,
        loop_a: Vec<VertexKey>,
        loop_b: Vec<VertexKey>,
        twist: i32,
        cuts: usize,
    ) -> Self {
        Self {
            node_id,
            selected_edges: Vec::new(),
            loop_a: Some(loop_a),
            loop_b: Some(loop_b),
            twist,
            cuts,
            created_faces: Vec::new(),
            old_mesh: None,
        }
    }
}

impl Command for BridgeEdgeLoopsCommand {
    fn name(&self) -> &str {
        "Bridge Edge Loops"
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

                let created = if let (Some(la), Some(lb)) = (&self.loop_a, &self.loop_b) {
                    base_mesh
                        .bridge_edge_loops_advanced(la, lb, self.twist, self.cuts)
                        .map_err(|e| format!("Bridge edge loops failed: {:?}", e))?
                } else {
                    base_mesh
                        .bridge_selected_edges(&self.selected_edges, self.twist, self.cuts)
                        .map_err(|e| format!("Bridge selected edges failed: {:?}", e))?
                };

                self.created_faces = created;
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

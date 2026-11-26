use geo_kernel::{EdgeKey, FaceKey, Mesh, NgonMethod, QuadMethod, VertexKey};
use graph_engine::{Command, NodeData, NodeId, SceneGraph};

#[derive(Debug)]
pub struct PokeFaceCommand {
    pub node_id: NodeId,
    pub face_key: FaceKey,
    pub offset: f32,
    pub old_mesh: Option<Mesh>,
}

impl Command for PokeFaceCommand {
    fn name(&self) -> &str {
        "Poke Face"
    }

    fn execute(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        if let Some(node) = scene.nodes.get_mut(self.node_id) {
            if let NodeData::Mesh { base_mesh, .. } = &mut node.data {
                if self.old_mesh.is_none() {
                    self.old_mesh = Some(base_mesh.clone());
                }
                base_mesh
                    .poke_face(self.face_key, self.offset)
                    .map_err(|e| format!("Poke face failed: {:?}", e))?;
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
pub struct DissolveEdgeCommand {
    pub node_id: NodeId,
    pub edge_key: EdgeKey,
    pub old_mesh: Option<Mesh>,
}

impl Command for DissolveEdgeCommand {
    fn name(&self) -> &str {
        "Dissolve Edge"
    }

    fn execute(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        if let Some(node) = scene.nodes.get_mut(self.node_id) {
            if let NodeData::Mesh { base_mesh, .. } = &mut node.data {
                if self.old_mesh.is_none() {
                    self.old_mesh = Some(base_mesh.clone());
                }
                base_mesh
                    .dissolve_edge(self.edge_key)
                    .map_err(|e| format!("Dissolve edge failed: {:?}", e))?;
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
pub struct RipVerticesCommand {
    pub node_id: NodeId,
    pub vertices: Vec<VertexKey>,
    pub old_mesh: Option<Mesh>,
    pub created_vertices: Vec<VertexKey>,
}

impl Command for RipVerticesCommand {
    fn name(&self) -> &str {
        "Rip Vertices"
    }

    fn execute(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        let node = scene.nodes.get_mut(self.node_id).ok_or("Node not found")?;
        match &mut node.data {
            NodeData::Mesh { base_mesh, cached_render_mesh, .. } => {
                if self.old_mesh.is_none() {
                    self.old_mesh = Some(base_mesh.clone());
                }
                let created = base_mesh
                    .rip_vertices(&self.vertices)
                    .map_err(|e| format!("Rip vertices failed: {:?}", e))?;
                self.created_vertices = created;
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
pub struct TriangulateCommand {
    pub node_id: NodeId,
    pub faces: Vec<FaceKey>,
    pub quad_method: QuadMethod,
    pub ngon_method: NgonMethod,
    pub min_vertices: usize,
    pub old_mesh: Option<Mesh>,
}

impl Command for TriangulateCommand {
    fn name(&self) -> &str {
        "Triangulate Faces"
    }

    fn execute(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        let node = scene.nodes.get_mut(self.node_id).ok_or("Node not found")?;
        match &mut node.data {
            NodeData::Mesh { base_mesh, cached_render_mesh, .. } => {
                if self.old_mesh.is_none() {
                    self.old_mesh = Some(base_mesh.clone());
                }
                base_mesh.triangulate_faces(&self.faces, self.quad_method, self.ngon_method, self.min_vertices)
                    .map_err(|e| format!("Triangulate failed: {:?}", e))?;
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
pub struct MergeVerticesCommand {
    pub node_id: NodeId,
    pub vertices: Vec<VertexKey>,
    pub old_mesh: Option<Mesh>,
}

impl Command for MergeVerticesCommand {
    fn name(&self) -> &str {
        "Merge Vertices at Center (M)"
    }

    fn execute(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        let node = scene.nodes.get_mut(self.node_id).ok_or("Node not found")?;
        match &mut node.data {
            NodeData::Mesh { base_mesh, cached_render_mesh, .. } => {
                if self.old_mesh.is_none() {
                    self.old_mesh = Some(base_mesh.clone());
                }
                base_mesh.merge_vertices_at_center(&self.vertices)
                    .map_err(|e| format!("Merge vertices failed: {:?}", e))?;
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
pub struct DeleteFacesCommand {
    pub node_id: NodeId,
    pub faces: Vec<FaceKey>,
    pub old_mesh: Option<Mesh>,
}

impl Command for DeleteFacesCommand {
    fn name(&self) -> &str {
        "Delete Faces"
    }

    fn execute(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        let node = scene.nodes.get_mut(self.node_id).ok_or("Node not found")?;
        match &mut node.data {
            NodeData::Mesh { base_mesh, cached_render_mesh, .. } => {
                if self.old_mesh.is_none() {
                    self.old_mesh = Some(base_mesh.clone());
                }
                for &fk in &self.faces {
                    base_mesh.delete_face(fk);
                }
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
pub struct DeleteVerticesCommand {
    pub node_id: NodeId,
    pub vertices: Vec<VertexKey>,
    pub old_mesh: Option<Mesh>,
}

impl Command for DeleteVerticesCommand {
    fn name(&self) -> &str {
        "Delete Vertices"
    }

    fn execute(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        let node = scene.nodes.get_mut(self.node_id).ok_or("Node not found")?;
        match &mut node.data {
            NodeData::Mesh { base_mesh, cached_render_mesh, .. } => {
                if self.old_mesh.is_none() {
                    self.old_mesh = Some(base_mesh.clone());
                }
                for &vk in &self.vertices {
                    base_mesh.delete_vertex(vk);
                }
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
pub struct UnBevelFacesCommand {
    pub node_id: NodeId,
    pub faces: Vec<FaceKey>,
    pub old_mesh: Option<Mesh>,
}

impl Command for UnBevelFacesCommand {
    fn name(&self) -> &str {
        "Un-Bevel Faces"
    }

    fn execute(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        let node = scene.nodes.get_mut(self.node_id).ok_or("Node not found")?;
        match &mut node.data {
            NodeData::Mesh { base_mesh, cached_render_mesh, .. } => {
                if self.old_mesh.is_none() {
                    self.old_mesh = Some(base_mesh.clone());
                }
                base_mesh
                    .un_bevel_faces(&self.faces)
                    .map_err(|e| format!("Un-bevel faces failed: {:?}", e))?;
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
pub struct DissolveFacesCommand {
    pub node_id: NodeId,
    pub faces: Vec<FaceKey>,
    pub old_mesh: Option<Mesh>,
}

impl Command for DissolveFacesCommand {
    fn name(&self) -> &str {
        "Dissolve Faces"
    }

    fn execute(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        let node = scene.nodes.get_mut(self.node_id).ok_or("Node not found")?;
        match &mut node.data {
            NodeData::Mesh { base_mesh, cached_render_mesh, .. } => {
                if self.old_mesh.is_none() {
                    self.old_mesh = Some(base_mesh.clone());
                }
                base_mesh
                    .dissolve_faces(&self.faces)
                    .map_err(|e| format!("Dissolve faces failed: {:?}", e))?;
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

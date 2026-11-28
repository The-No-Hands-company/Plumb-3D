use core_math::Vec3;
use geo_kernel::{EdgeKey, FaceKey, Mesh, MirrorAxis, SymmetrizeDirection, VertexKey};
use graph_engine::{Command, NodeData, NodeId, SceneGraph};

#[derive(Debug)]
pub struct SymmetrizeCommand {
    pub node_id: NodeId,
    pub axis: MirrorAxis,
    pub direction: SymmetrizeDirection,
    pub old_mesh: Option<Mesh>,
}

impl Command for SymmetrizeCommand {
    fn name(&self) -> &str {
        "Symmetrize Mesh"
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
                    .symmetrize(self.axis, self.direction)
                    .map_err(|e| format!("Symmetrize failed: {:?}", e))?;
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
pub struct CleanDegenerateCommand {
    pub node_id: NodeId,
    pub min_area: f32,
    pub min_edge_len: f32,
    pub old_mesh: Option<Mesh>,
}

impl Command for CleanDegenerateCommand {
    fn name(&self) -> &str {
        "Clean Degenerate Geometry"
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
                    .clean_degenerate_geometry(self.min_area, self.min_edge_len)
                    .map_err(|e| format!("Clean geometry failed: {:?}", e))?;
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
pub struct SpinCommand {
    pub node_id: NodeId,
    pub faces: Vec<FaceKey>,
    pub axis: Vec3,
    pub center: Vec3,
    pub angle_radians: f32,
    pub steps: usize,
    pub old_mesh: Option<Mesh>,
}

impl Command for SpinCommand {
    fn name(&self) -> &str {
        "Spin Revolve"
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
                    .spin(
                        &self.faces,
                        self.axis,
                        self.center,
                        self.angle_radians,
                        self.steps,
                    )
                    .map_err(|e| format!("Spin failed: {:?}", e))?;
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
pub struct CapHolesCommand {
    pub node_id: NodeId,
    pub triangulate: bool,
    pub old_mesh: Option<Mesh>,
}

impl Command for CapHolesCommand {
    fn name(&self) -> &str {
        "Cap Holes"
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
                    .cap_holes(self.triangulate)
                    .map_err(|e| format!("Cap holes failed: {:?}", e))?;
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
pub struct GridFillCommand {
    pub node_id: NodeId,
    pub boundary: Vec<VertexKey>,
    pub span_count: usize,
    pub old_mesh: Option<Mesh>,
}

impl Command for GridFillCommand {
    fn name(&self) -> &str {
        "Grid Fill"
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
                    .grid_fill(&self.boundary, self.span_count)
                    .map_err(|e| format!("Grid fill failed: {:?}", e))?;
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
pub struct FlipNormalsCommand {
    pub node_id: NodeId,
    pub faces: Vec<FaceKey>,
    pub old_mesh: Option<Mesh>,
}

impl Command for FlipNormalsCommand {
    fn name(&self) -> &str {
        "Flip Normals (Alt+N)"
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
                    .flip_face_normals(&self.faces)
                    .map_err(|e| format!("Flip normals failed: {:?}", e))?;
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
pub struct BisectCommand {
    pub node_id: NodeId,
    pub plane_point: Vec3,
    pub plane_normal: Vec3,
    pub clear_positive: bool,
    pub clear_negative: bool,
    pub fill_cut: bool,
    pub old_mesh: Option<Mesh>,
}

impl Command for BisectCommand {
    fn name(&self) -> &str {
        "Bisect / Plane Cut"
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
                    .bisect(
                        self.plane_point,
                        self.plane_normal,
                        self.clear_positive,
                        self.clear_negative,
                        self.fill_cut,
                    )
                    .map_err(|e| format!("Bisect failed: {:?}", e))?;
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
pub struct MarkSharpCommand {
    pub node_id: NodeId,
    pub edges: Vec<EdgeKey>,
    pub sharp: bool,
    pub old_mesh: Option<Mesh>,
}

impl Command for MarkSharpCommand {
    fn name(&self) -> &str {
        if self.sharp {
            "Mark Sharp (Ctrl+E)"
        } else {
            "Clear Sharp"
        }
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
                for &ek in &self.edges {
                    base_mesh.set_edge_sharp(ek, self.sharp);
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
pub struct SetEdgeCreaseCommand {
    pub node_id: NodeId,
    pub edges: Vec<EdgeKey>,
    pub crease_weight: f32,
    pub old_mesh: Option<Mesh>,
}

impl Command for SetEdgeCreaseCommand {
    fn name(&self) -> &str {
        "Set Edge Crease"
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
                for &ek in &self.edges {
                    base_mesh.set_edge_crease(ek, self.crease_weight);
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

#[derive(Debug)]
pub struct SetVertexCreaseCommand {
    pub node_id: NodeId,
    pub vertices: Vec<VertexKey>,
    pub crease_weight: f32,
    pub old_mesh: Option<Mesh>,
}

impl Command for SetVertexCreaseCommand {
    fn name(&self) -> &str {
        "Set Vertex Crease (Shift+V)"
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
                for &vk in &self.vertices {
                    base_mesh.set_vertex_crease(vk, self.crease_weight);
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

#[derive(Debug)]
pub struct RecalculateNormalsCommand {
    pub node_id: NodeId,
    pub inside: bool,
    pub old_mesh: Option<Mesh>,
}

impl Command for RecalculateNormalsCommand {
    fn name(&self) -> &str {
        if self.inside {
            "Recalculate Normals Inside (Shift+Ctrl+N)"
        } else {
            "Recalculate Normals Outside (Shift+N)"
        }
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
                    .recalculate_face_normals(self.inside)
                    .map_err(|e| format!("{:?}", e))?;
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

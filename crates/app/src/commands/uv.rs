use crate::types::UVProjectionType;
use geo_kernel::{EdgeKey, Mesh, VertexKey};
use graph_engine::{Command, NodeData, NodeId, SceneGraph};

#[derive(Debug)]
pub struct MarkSeamCommand {
    pub node_id: NodeId,
    pub edges: Vec<EdgeKey>,
    pub is_seam: bool,
    pub old_seam_states: Vec<(EdgeKey, bool)>,
}

impl Command for MarkSeamCommand {
    fn name(&self) -> &str {
        if self.is_seam {
            "Mark UV Seam(s)"
        } else {
            "Clear UV Seam(s)"
        }
    }

    fn execute(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        let node = scene.nodes.get_mut(self.node_id).ok_or("Node not found")?;
        if let NodeData::Mesh { base_mesh, .. } = &mut node.data {
            if self.old_seam_states.is_empty() {
                self.old_seam_states = self
                    .edges
                    .iter()
                    .filter_map(|&ek| base_mesh.edges.get(ek).map(|e| (ek, e.is_seam)))
                    .collect();
            }
            base_mesh.mark_seams(&self.edges, self.is_seam);
            scene.mark_dirty(self.node_id);
            Ok(())
        } else {
            Err("Node is not a mesh".into())
        }
    }

    fn undo(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        let node = scene.nodes.get_mut(self.node_id).ok_or("Node not found")?;
        if let NodeData::Mesh { base_mesh, .. } = &mut node.data {
            for &(ek, was_seam) in &self.old_seam_states {
                base_mesh.set_edge_seam(ek, was_seam);
            }
            scene.mark_dirty(self.node_id);
            Ok(())
        } else {
            Err("Node is not a mesh".into())
        }
    }
}

#[derive(Debug)]
pub struct SmartUvProjectCommand {
    pub node_id: NodeId,
    pub angle_threshold_deg: f32,
    pub margin: f32,
    pub old_mesh: Option<Mesh>,
}

impl Command for SmartUvProjectCommand {
    fn name(&self) -> &str {
        "Smart UV Project"
    }

    fn execute(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        let node = scene.nodes.get_mut(self.node_id).ok_or("Node not found")?;
        if let NodeData::Mesh {
            base_mesh,
            cached_render_mesh,
            ..
        } = &mut node.data
        {
            if self.old_mesh.is_none() {
                self.old_mesh = Some(base_mesh.clone());
            }
            base_mesh.smart_uv_project(self.angle_threshold_deg, self.margin);
            *cached_render_mesh = None;
            scene.mark_dirty(self.node_id);
            Ok(())
        } else {
            Err("Node is not a mesh".into())
        }
    }

    fn undo(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        if let Some(old) = &self.old_mesh {
            let node = scene.nodes.get_mut(self.node_id).ok_or("Node not found")?;
            if let NodeData::Mesh {
                base_mesh,
                cached_render_mesh,
                ..
            } = &mut node.data
            {
                *base_mesh = old.clone();
                *cached_render_mesh = None;
                scene.mark_dirty(self.node_id);
                Ok(())
            } else {
                Err("Node is not a mesh".into())
            }
        } else {
            Err("No snapshot to restore".into())
        }
    }
}

#[derive(Debug)]
pub struct UnwrapSeamsCommand {
    pub node_id: NodeId,
    pub margin: f32,
    pub old_mesh: Option<Mesh>,
}

impl Command for UnwrapSeamsCommand {
    fn name(&self) -> &str {
        "Unwrap UV (Seams)"
    }

    fn execute(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        let node = scene.nodes.get_mut(self.node_id).ok_or("Node not found")?;
        if let NodeData::Mesh {
            base_mesh,
            cached_render_mesh,
            ..
        } = &mut node.data
        {
            if self.old_mesh.is_none() {
                self.old_mesh = Some(base_mesh.clone());
            }
            base_mesh.unwrap_seams(self.margin);
            *cached_render_mesh = None;
            scene.mark_dirty(self.node_id);
            Ok(())
        } else {
            Err("Node is not a mesh".into())
        }
    }

    fn undo(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        if let Some(old) = &self.old_mesh {
            let node = scene.nodes.get_mut(self.node_id).ok_or("Node not found")?;
            if let NodeData::Mesh {
                base_mesh,
                cached_render_mesh,
                ..
            } = &mut node.data
            {
                *base_mesh = old.clone();
                *cached_render_mesh = None;
                scene.mark_dirty(self.node_id);
                Ok(())
            } else {
                Err("Node is not a mesh".into())
            }
        } else {
            Err("No snapshot to restore".into())
        }
    }
}

#[derive(Debug)]
pub struct UVProjectionCommand {
    pub node_id: NodeId,
    pub projection: UVProjectionType,
    pub old_mesh: Option<Mesh>,
}

impl Command for UVProjectionCommand {
    fn name(&self) -> &str {
        "Generate UV Projection"
    }

    fn execute(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        if let Some(node) = scene.nodes.get_mut(self.node_id) {
            if let NodeData::Mesh {
                base_mesh,
                cached_render_mesh,
                ..
            } = &mut node.data
            {
                if self.old_mesh.is_none() {
                    self.old_mesh = Some(base_mesh.clone());
                }
                match self.projection {
                    UVProjectionType::Planar(axis) => base_mesh.generate_planar_uv(axis),
                    UVProjectionType::Box => base_mesh.generate_box_uv(),
                    UVProjectionType::Cylindrical(axis) => base_mesh.generate_cylindrical_uv(axis),
                    UVProjectionType::Spherical => base_mesh.generate_spherical_uv(),
                }
                *cached_render_mesh = None;
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
                if let NodeData::Mesh {
                    base_mesh,
                    cached_render_mesh,
                    ..
                } = &mut node.data
                {
                    *base_mesh = old.clone();
                    *cached_render_mesh = None;
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
pub struct PackUvIslandsCommand {
    pub node_id: NodeId,
    pub margin: f32,
    pub old_mesh: Option<Mesh>,
}

impl Command for PackUvIslandsCommand {
    fn name(&self) -> &str {
        "Pack UV Islands"
    }

    fn execute(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        let node = scene.nodes.get_mut(self.node_id).ok_or("Node not found")?;
        if let NodeData::Mesh {
            base_mesh,
            cached_render_mesh,
            ..
        } = &mut node.data
        {
            if self.old_mesh.is_none() {
                self.old_mesh = Some(base_mesh.clone());
            }
            base_mesh.pack_existing_uvs(self.margin);
            *cached_render_mesh = None;
            scene.mark_dirty(self.node_id);
            Ok(())
        } else {
            Err("Node is not a mesh".into())
        }
    }

    fn undo(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        if let Some(old) = &self.old_mesh {
            let node = scene.nodes.get_mut(self.node_id).ok_or("Node not found")?;
            if let NodeData::Mesh {
                base_mesh,
                cached_render_mesh,
                ..
            } = &mut node.data
            {
                *base_mesh = old.clone();
                *cached_render_mesh = None;
                scene.mark_dirty(self.node_id);
                Ok(())
            } else {
                Err("Node is not a mesh".into())
            }
        } else {
            Err("No snapshot to restore".into())
        }
    }
}

#[derive(Debug)]
pub struct RotateUvsCommand {
    pub node_id: NodeId,
    pub clockwise: bool,
    pub old_mesh: Option<Mesh>,
}

impl Command for RotateUvsCommand {
    fn name(&self) -> &str {
        if self.clockwise {
            "Rotate UVs CW"
        } else {
            "Rotate UVs CCW"
        }
    }

    fn execute(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        let node = scene.nodes.get_mut(self.node_id).ok_or("Node not found")?;
        if let NodeData::Mesh {
            base_mesh,
            cached_render_mesh,
            ..
        } = &mut node.data
        {
            if self.old_mesh.is_none() {
                self.old_mesh = Some(base_mesh.clone());
            }
            base_mesh.rotate_uvs_90(self.clockwise);
            *cached_render_mesh = None;
            scene.mark_dirty(self.node_id);
            Ok(())
        } else {
            Err("Node is not a mesh".into())
        }
    }

    fn undo(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        if let Some(old) = &self.old_mesh {
            let node = scene.nodes.get_mut(self.node_id).ok_or("Node not found")?;
            if let NodeData::Mesh {
                base_mesh,
                cached_render_mesh,
                ..
            } = &mut node.data
            {
                *base_mesh = old.clone();
                *cached_render_mesh = None;
                scene.mark_dirty(self.node_id);
                Ok(())
            } else {
                Err("Node is not a mesh".into())
            }
        } else {
            Err("No snapshot to restore".into())
        }
    }
}

#[derive(Debug)]
pub struct FlipUvsCommand {
    pub node_id: NodeId,
    pub horizontal: bool,
    pub old_mesh: Option<Mesh>,
}

impl Command for FlipUvsCommand {
    fn name(&self) -> &str {
        if self.horizontal {
            "Flip UVs Horizontally"
        } else {
            "Flip UVs Vertically"
        }
    }

    fn execute(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        let node = scene.nodes.get_mut(self.node_id).ok_or("Node not found")?;
        if let NodeData::Mesh {
            base_mesh,
            cached_render_mesh,
            ..
        } = &mut node.data
        {
            if self.old_mesh.is_none() {
                self.old_mesh = Some(base_mesh.clone());
            }
            base_mesh.flip_uvs(self.horizontal);
            *cached_render_mesh = None;
            scene.mark_dirty(self.node_id);
            Ok(())
        } else {
            Err("Node is not a mesh".into())
        }
    }

    fn undo(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        if let Some(old) = &self.old_mesh {
            let node = scene.nodes.get_mut(self.node_id).ok_or("Node not found")?;
            if let NodeData::Mesh {
                base_mesh,
                cached_render_mesh,
                ..
            } = &mut node.data
            {
                *base_mesh = old.clone();
                *cached_render_mesh = None;
                scene.mark_dirty(self.node_id);
                Ok(())
            } else {
                Err("Node is not a mesh".into())
            }
        } else {
            Err("No snapshot to restore".into())
        }
    }
}

#[derive(Debug)]
pub struct ScaleUvsCommand {
    pub node_id: NodeId,
    pub factor: f32,
    pub old_mesh: Option<Mesh>,
}

impl Command for ScaleUvsCommand {
    fn name(&self) -> &str {
        "Scale UVs"
    }

    fn execute(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        let node = scene.nodes.get_mut(self.node_id).ok_or("Node not found")?;
        if let NodeData::Mesh {
            base_mesh,
            cached_render_mesh,
            ..
        } = &mut node.data
        {
            if self.old_mesh.is_none() {
                self.old_mesh = Some(base_mesh.clone());
            }
            base_mesh.scale_uvs(self.factor);
            *cached_render_mesh = None;
            scene.mark_dirty(self.node_id);
            Ok(())
        } else {
            Err("Node is not a mesh".into())
        }
    }

    fn undo(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        if let Some(old) = &self.old_mesh {
            let node = scene.nodes.get_mut(self.node_id).ok_or("Node not found")?;
            if let NodeData::Mesh {
                base_mesh,
                cached_render_mesh,
                ..
            } = &mut node.data
            {
                *base_mesh = old.clone();
                *cached_render_mesh = None;
                scene.mark_dirty(self.node_id);
                Ok(())
            } else {
                Err("Node is not a mesh".into())
            }
        } else {
            Err("No snapshot to restore".into())
        }
    }
}

/// Laplacian UV relaxation command to minimize distortion.
#[derive(Debug)]
pub struct RelaxUvsCommand {
    pub node_id: NodeId,
    pub iterations: usize,
    pub factor: f32,
    pub old_mesh: Option<Mesh>,
}

impl Command for RelaxUvsCommand {
    fn name(&self) -> &str {
        "Relax UVs"
    }

    fn execute(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        let node = scene.nodes.get_mut(self.node_id).ok_or("Node not found")?;
        if let NodeData::Mesh {
            base_mesh,
            cached_render_mesh,
            ..
        } = &mut node.data
        {
            if self.old_mesh.is_none() {
                self.old_mesh = Some(base_mesh.clone());
            }
            base_mesh.relax_uvs(self.iterations, self.factor);
            *cached_render_mesh = None;
            scene.mark_dirty(self.node_id);
            Ok(())
        } else {
            Err("Node is not a mesh".into())
        }
    }

    fn undo(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        if let Some(old) = &self.old_mesh {
            let node = scene.nodes.get_mut(self.node_id).ok_or("Node not found")?;
            if let NodeData::Mesh {
                base_mesh,
                cached_render_mesh,
                ..
            } = &mut node.data
            {
                *base_mesh = old.clone();
                *cached_render_mesh = None;
                scene.mark_dirty(self.node_id);
                Ok(())
            } else {
                Err("Node is not a mesh".into())
            }
        } else {
            Err("No snapshot to restore".into())
        }
    }
}

/// Command to pin or unpin UV vertices in the 2D canvas to lock them during unwrapping or relaxation.
#[derive(Debug)]
pub struct PinUvsCommand {
    pub node_id: NodeId,
    pub vertices: Vec<VertexKey>,
    pub pin: bool,
    pub old_pinned_states: Vec<(VertexKey, bool)>,
}

impl Command for PinUvsCommand {
    fn name(&self) -> &str {
        if self.pin {
            "Pin UV Vertices"
        } else {
            "Unpin UV Vertices"
        }
    }

    fn execute(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        let node = scene.nodes.get_mut(self.node_id).ok_or("Node not found")?;
        if let NodeData::Mesh { base_mesh, .. } = &mut node.data {
            if self.old_pinned_states.is_empty() {
                self.old_pinned_states = self
                    .vertices
                    .iter()
                    .filter_map(|&vk| base_mesh.vertices.get(vk).map(|v| (vk, v.uv_pinned)))
                    .collect();
            }
            for &vk in &self.vertices {
                if let Some(v) = base_mesh.vertices.get_mut(vk) {
                    v.uv_pinned = self.pin;
                }
            }
            scene.mark_dirty(self.node_id);
            Ok(())
        } else {
            Err("Node is not a mesh".into())
        }
    }

    fn undo(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        let node = scene.nodes.get_mut(self.node_id).ok_or("Node not found")?;
        if let NodeData::Mesh { base_mesh, .. } = &mut node.data {
            for &(vk, was_pinned) in &self.old_pinned_states {
                if let Some(v) = base_mesh.vertices.get_mut(vk) {
                    v.uv_pinned = was_pinned;
                }
            }
            scene.mark_dirty(self.node_id);
            Ok(())
        } else {
            Err("Node is not a mesh".into())
        }
    }
}

/// Command to move UV vertex coordinates interactively with full undo/redo.
#[derive(Debug)]
pub struct MoveUvVertexCommand {
    pub node_id: NodeId,
    pub vertex: VertexKey,
    pub new_uv: core_math::Vec2,
    pub old_uv: core_math::Vec2,
}

impl Command for MoveUvVertexCommand {
    fn name(&self) -> &str {
        "Move UV Vertex"
    }

    fn execute(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        let node = scene.nodes.get_mut(self.node_id).ok_or("Node not found")?;
        if let NodeData::Mesh {
            base_mesh,
            cached_render_mesh,
            ..
        } = &mut node.data
        {
            if let Some(v) = base_mesh.vertices.get_mut(self.vertex) {
                v.uv = self.new_uv;
            }
            *cached_render_mesh = None;
            scene.mark_dirty(self.node_id);
            Ok(())
        } else {
            Err("Node is not a mesh".into())
        }
    }

    fn undo(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        let node = scene.nodes.get_mut(self.node_id).ok_or("Node not found")?;
        if let NodeData::Mesh {
            base_mesh,
            cached_render_mesh,
            ..
        } = &mut node.data
        {
            if let Some(v) = base_mesh.vertices.get_mut(self.vertex) {
                v.uv = self.old_uv;
            }
            *cached_render_mesh = None;
            scene.mark_dirty(self.node_id);
            Ok(())
        } else {
            Err("Node is not a mesh".into())
        }
    }
}

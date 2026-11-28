//! Production DCC Commands for Variable Radius Fillets, Seam Weld Beads,
//! Floating Mesh Decals, and Flow Guide Topology Alignment.

use core_math::Vec3;
use geo_kernel::{operators::bevel::BevelProfile, EdgeKey, FilletContinuity, Mesh, WeldStyle};
use graph_engine::{Command, NodeData, NodeId, SceneGraph};

/// Non-destructive command to apply variable radius filleting ($R_1 \to R_2$) along an edge chain.
#[derive(Debug)]
pub struct VariableRadiusFilletCommand {
    pub node_id: NodeId,
    pub edges: Vec<EdgeKey>,
    pub r1: f32,
    pub r2: f32,
    pub segments: usize,
    pub continuity: FilletContinuity,
    pub old_mesh: Option<Mesh>,
}

impl Command for VariableRadiusFilletCommand {
    fn name(&self) -> &str {
        "Variable Radius Fillet"
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
                    .bevel_edge_chain_variable(
                        &self.edges,
                        self.r1,
                        self.r2,
                        self.segments,
                        self.continuity,
                        BevelProfile::Round(0.5),
                    )
                    .map_err(|e| format!("Variable fillet failed: {:?}", e))?;
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
            Ok(())
        }
    }
}

/// Command to generate an industrial seam pipe or welded bead (Fuse) along an edge loop.
#[derive(Debug)]
pub struct GenerateWeldBeadCommand {
    pub node_id: NodeId,
    pub edges: Vec<EdgeKey>,
    pub radius: f32,
    pub style: WeldStyle,
    pub old_mesh: Option<Mesh>,
}

impl Command for GenerateWeldBeadCommand {
    fn name(&self) -> &str {
        "Generate Seam Weld Bead"
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
                    .generate_weld_bead(&self.edges, self.radius, self.style)
                    .map_err(|e| format!("Weld bead failed: {:?}", e))?;
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
            Ok(())
        }
    }
}

/// Command to project a floating mesh decal onto a curved hull and stamp boundary normals.
#[derive(Debug)]
pub struct ProjectDecalCommand {
    pub decal_id: NodeId,
    pub hull_id: NodeId,
    pub offset: f32,
    pub old_mesh: Option<Mesh>,
}

impl Command for ProjectDecalCommand {
    fn name(&self) -> &str {
        "Project Floating Decal"
    }

    fn execute(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        let (hull_mesh, hull_mat) = {
            let hull_node = scene
                .nodes
                .get(self.hull_id)
                .ok_or("Target hull node not found")?;
            match &hull_node.data {
                NodeData::Mesh { base_mesh, .. } => (base_mesh.clone(), hull_node.world_matrix),
                _ => return Err("Target node is not a mesh".into()),
            }
        };

        let decal_node = scene
            .nodes
            .get_mut(self.decal_id)
            .ok_or("Decal node not found")?;
        let decal_mat = decal_node.world_matrix;

        match &mut decal_node.data {
            NodeData::Mesh {
                base_mesh,
                cached_render_mesh,
                ..
            } => {
                if self.old_mesh.is_none() {
                    self.old_mesh = Some(base_mesh.clone());
                }
                base_mesh
                    .project_floating_decal(&hull_mesh, &decal_mat, &hull_mat, self.offset)
                    .map_err(|e| format!("Decal projection failed: {:?}", e))?;
                *cached_render_mesh = None;
                scene.mark_dirty(self.decal_id);
                Ok(())
            }
            _ => Err("Decal node is not a mesh".into()),
        }
    }

    fn undo(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        if let Some(old) = &self.old_mesh {
            let node = scene.nodes.get_mut(self.decal_id).ok_or("Node not found")?;
            match &mut node.data {
                NodeData::Mesh {
                    base_mesh,
                    cached_render_mesh,
                    ..
                } => {
                    *base_mesh = old.clone();
                    *cached_render_mesh = None;
                    scene.mark_dirty(self.decal_id);
                    Ok(())
                }
                _ => Err("Node is not a mesh".into()),
            }
        } else {
            Ok(())
        }
    }
}

/// Command to align surface quad topology along a 3D styling flow guide curve.
#[derive(Debug)]
pub struct AlignFlowGuideCommand {
    pub node_id: NodeId,
    pub guide_points: Vec<Vec3>,
    pub iterations: usize,
    pub influence_radius: f32,
    pub old_mesh: Option<Mesh>,
}

impl Command for AlignFlowGuideCommand {
    fn name(&self) -> &str {
        "Align Topology to Flow Guide"
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
                    .align_topology_to_flow_guide(
                        &self.guide_points,
                        self.iterations,
                        self.influence_radius,
                    )
                    .map_err(|e| format!("Flow guide alignment failed: {:?}", e))?;
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
            Ok(())
        }
    }
}

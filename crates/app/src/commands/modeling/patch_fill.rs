//! Undoable Command for N-Sided Boundary Hole Coons Patch Filling.

use geo_kernel::{CoonsPatchType, Mesh, VertexKey};
use graph_engine::{Command, NodeData, NodeId, SceneGraph};

#[derive(Debug)]
pub struct PatchFillCoonsCommand {
    node_id: NodeId,
    boundary_verts: Vec<VertexKey>,
    subdivisions: usize,
    patch_type: CoonsPatchType,
    before_mesh: Option<Mesh>,
}

impl PatchFillCoonsCommand {
    pub fn new(
        node_id: NodeId,
        boundary_verts: Vec<VertexKey>,
        subdivisions: usize,
        patch_type: CoonsPatchType,
    ) -> Self {
        Self {
            node_id,
            boundary_verts,
            subdivisions,
            patch_type,
            before_mesh: None,
        }
    }
}

impl Command for PatchFillCoonsCommand {
    fn name(&self) -> &str {
        "N-Sided Coons Patch Fill"
    }

    fn execute(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        let node = scene
            .nodes
            .get_mut(self.node_id)
            .ok_or_else(|| format!("Node {:?} not found", self.node_id))?;

        let mesh = match &mut node.data {
            NodeData::Mesh { base_mesh, .. } => base_mesh,
            _ => return Err("Target node is not a polygon mesh".to_string()),
        };

        if self.before_mesh.is_none() {
            self.before_mesh = Some(mesh.clone());
        }

        mesh.fill_nsided_hole_coons(&self.boundary_verts, self.subdivisions, self.patch_type)
            .map_err(|e| format!("Coons patch fill failed: {:?}", e))?;

        Ok(())
    }

    fn undo(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        let before = self
            .before_mesh
            .as_ref()
            .ok_or("No before state available for undo")?;

        let node = scene
            .nodes
            .get_mut(self.node_id)
            .ok_or_else(|| format!("Node {:?} not found", self.node_id))?;

        match &mut node.data {
            NodeData::Mesh { base_mesh, .. } => {
                *base_mesh = before.clone();
                Ok(())
            }
            _ => Err("Target node is not a polygon mesh".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core_math::{Transform, Vec3};
    use graph_engine::SceneNode;

    #[test]
    fn test_coons_patch_fill_command_and_undo() {
        let mut scene = SceneGraph::new();
        let mut mesh = Mesh::new();
        let mut b_verts = Vec::new();
        for i in 0..4 {
            let angle = i as f32 * std::f32::consts::TAU / 4.0;
            let vk = mesh.add_vertex(Vec3::new(angle.cos(), 0.0, angle.sin()));
            b_verts.push(vk);
        }

        let node_id = scene.add_node(SceneNode::new_mesh(
            "HoleMesh",
            Transform::IDENTITY,
            mesh,
        ));

        let mut cmd = PatchFillCoonsCommand::new(node_id, b_verts, 2, CoonsPatchType::BicubicHermite);
        assert!(cmd.execute(&mut scene).is_ok());
        assert!(cmd.undo(&mut scene).is_ok());
    }
}

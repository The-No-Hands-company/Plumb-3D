//! Command for 3D Interference & Clash Detection between Scene Bodies.

use core_math::Vec3;
use geo_kernel::{check_mesh_clash, ClashResult, CollisionMesh};
use graph_engine::{Command, NodeData, NodeId, SceneGraph};

#[derive(Debug)]
pub struct DetectClashCommand {
    node_a: NodeId,
    node_b: NodeId,
    tolerance: f32,
    pub result: Option<ClashResult>,
}

impl DetectClashCommand {
    pub fn new(node_a: NodeId, node_b: NodeId, tolerance: f32) -> Self {
        Self {
            node_a,
            node_b,
            tolerance,
            result: None,
        }
    }

    fn extract_collision_mesh(scene: &SceneGraph, node_id: NodeId) -> Result<(CollisionMesh, core_math::Mat4), String> {
        let node = scene
            .nodes
            .get(node_id)
            .ok_or_else(|| format!("Node {:?} not found", node_id))?;

        let render_mesh = match &node.data {
            NodeData::Mesh { base_mesh, shading_mode, .. } => {
                base_mesh.to_render_mesh(*shading_mode)
            }
            _ => return Err(format!("Node {:?} is not a mesh", node_id)),
        };

        let vertices: Vec<Vec3> = render_mesh
            .vertices
            .iter()
            .map(|v| Vec3::from(v.position))
            .collect();

        let mut indices = Vec::new();
        for chunk in render_mesh.indices.as_chunks::<3>().0 {
            indices.push([chunk[0], chunk[1], chunk[2]]);
        }

        let mesh = CollisionMesh::from_raw(vertices, indices);
        Ok((mesh, node.world_matrix))
    }
}

impl Command for DetectClashCommand {
    fn name(&self) -> &str {
        "Detect Clash"
    }

    fn execute(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        let (mesh_a, tf_a) = Self::extract_collision_mesh(scene, self.node_a)?;
        let (mesh_b, tf_b) = Self::extract_collision_mesh(scene, self.node_b)?;

        let clash = check_mesh_clash(0, &mesh_a, &tf_a, 1, &mesh_b, &tf_b, self.tolerance);
        self.result = Some(clash);
        Ok(())
    }

    fn undo(&mut self, _scene: &mut SceneGraph) -> Result<(), String> {
        self.result = None;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core_math::Transform;
    use geo_kernel::Primitives;
    use graph_engine::SceneNode;

    #[test]
    fn test_detect_clash_command_overlapping_nodes() {
        let mut scene = SceneGraph::new();

        let cube_a = Primitives::create_cube(2.0);
        let n_a = scene.add_node(SceneNode::new_mesh(
            "CubeA",
            Transform::IDENTITY,
            cube_a,
        ));

        let cube_b = Primitives::create_cube(2.0);
        let n_b = scene.add_node(SceneNode::new_mesh(
            "CubeB",
            Transform::from_translation(Vec3::new(1.0, 0.0, 0.0)),
            cube_b,
        ));

        scene.mark_dirty(n_a);
        scene.mark_dirty(n_b);
        scene.update_transforms();

        let mut cmd = DetectClashCommand::new(n_a, n_b, 0.01);
        assert!(cmd.execute(&mut scene).is_ok());

        let res = cmd.result.as_ref().unwrap();
        assert!(res.is_interfering(), "Intersecting scene nodes must report interference");
    }
}

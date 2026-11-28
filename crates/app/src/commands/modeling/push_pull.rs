use core_math::Vec3;
use geo_kernel::{FaceKey, Mesh};
use graph_engine::{Command, NodeData, NodeId, SceneGraph};

/// Command to Push-Pull a face bi-directionally (Boss Union outward or Cavity Cut inward) with full Undo/Redo.
#[derive(Debug)]
pub struct PushPullCommand {
    pub node_id: NodeId,
    pub face_key: FaceKey,
    pub distance: f32,
    pub old_mesh: Option<Mesh>,
    pub created_cap: Option<FaceKey>,
    pub created_walls: Vec<FaceKey>,
}

impl Command for PushPullCommand {
    fn name(&self) -> &str {
        if self.distance >= 0.0 {
            "Push-Pull: Boss Union (P)"
        } else {
            "Push-Pull: Cavity Cut (P)"
        }
    }

    fn execute(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        let node = scene.nodes.get_mut(self.node_id).ok_or("Node not found")?;
        match &mut node.data {
            NodeData::Mesh { base_mesh, .. } => {
                let mut new_mesh = base_mesh.clone();
                let res = new_mesh
                    .push_pull_face(self.face_key, self.distance)
                    .map_err(|e| format!("Push-pull failed: {:?}", e))?;

                self.created_cap = Some(res.cap_face);
                self.created_walls = res.wall_faces;
                self.old_mesh = Some(std::mem::replace(base_mesh, new_mesh));
                scene.mark_dirty(self.node_id);
                Ok(())
            }
            _ => Err("Node is not a mesh".into()),
        }
    }

    fn undo(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        if let Some(old) = self.old_mesh.take() {
            let node = scene.nodes.get_mut(self.node_id).ok_or("Node not found")?;
            if let NodeData::Mesh { base_mesh, .. } = &mut node.data {
                *base_mesh = old;
                scene.mark_dirty(self.node_id);
            }
        }
        Ok(())
    }
}

/// Command to inscribe a 2D/3D polygon profile onto an existing face, bisecting it with full Undo/Redo.
#[derive(Debug)]
pub struct InscribePolygonCommand {
    pub node_id: NodeId,
    pub target_face: FaceKey,
    pub polygon_points: Vec<Vec3>,
    pub old_mesh: Option<Mesh>,
    pub created_inner_face: Option<FaceKey>,
    pub created_skirt_faces: Vec<FaceKey>,
}

impl Command for InscribePolygonCommand {
    fn name(&self) -> &str {
        "Inscribe Profile on Face"
    }

    fn execute(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        let node = scene.nodes.get_mut(self.node_id).ok_or("Node not found")?;
        match &mut node.data {
            NodeData::Mesh { base_mesh, .. } => {
                let mut new_mesh = base_mesh.clone();
                let (inner_fk, skirt_fks) = new_mesh
                    .inscribe_polygon_on_face(self.target_face, &self.polygon_points)
                    .map_err(|e| format!("Inscribe profile failed: {:?}", e))?;

                self.created_inner_face = Some(inner_fk);
                self.created_skirt_faces = skirt_fks;
                self.old_mesh = Some(std::mem::replace(base_mesh, new_mesh));
                scene.mark_dirty(self.node_id);
                Ok(())
            }
            _ => Err("Node is not a mesh".into()),
        }
    }

    fn undo(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        if let Some(old) = self.old_mesh.take() {
            let node = scene.nodes.get_mut(self.node_id).ok_or("Node not found")?;
            if let NodeData::Mesh { base_mesh, .. } = &mut node.data {
                *base_mesh = old;
                scene.mark_dirty(self.node_id);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core_math::{vec3, Transform};
    use geo_kernel::Primitives;
    use graph_engine::SceneNode;

    #[test]
    fn test_push_pull_command_boss_and_undo() {
        let mut scene = SceneGraph::new();
        let cube = Primitives::create_cube(2.0);
        let id = scene.add_node(SceneNode::new_mesh("Cube", Transform::IDENTITY, cube));

        let top_fk = match &scene.nodes[id].data {
            NodeData::Mesh { base_mesh, .. } => base_mesh
                .faces
                .iter()
                .find_map(|(fk, f)| if f.normal.y > 0.9 { Some(fk) } else { None })
                .unwrap(),
            _ => panic!("Expected mesh"),
        };

        let mut cmd = PushPullCommand {
            node_id: id,
            face_key: top_fk,
            distance: 0.5,
            old_mesh: None,
            created_cap: None,
            created_walls: Vec::new(),
        };

        assert!(cmd.execute(&mut scene).is_ok());
        assert!(cmd.undo(&mut scene).is_ok());
    }

    #[test]
    fn test_inscribe_command_and_undo() {
        let mut scene = SceneGraph::new();
        let cube = Primitives::create_cube(2.0);
        let id = scene.add_node(SceneNode::new_mesh("Cube", Transform::IDENTITY, cube));

        let top_fk = match &scene.nodes[id].data {
            NodeData::Mesh { base_mesh, .. } => base_mesh
                .faces
                .iter()
                .find_map(|(fk, f)| if f.normal.y > 0.9 { Some(fk) } else { None })
                .unwrap(),
            _ => panic!("Expected mesh"),
        };

        let pocket = vec![
            vec3(-0.3, 1.0, -0.3),
            vec3(0.3, 1.0, -0.3),
            vec3(0.3, 1.0, 0.3),
            vec3(-0.3, 1.0, 0.3),
        ];

        let mut cmd = InscribePolygonCommand {
            node_id: id,
            target_face: top_fk,
            polygon_points: pocket,
            old_mesh: None,
            created_inner_face: None,
            created_skirt_faces: Vec::new(),
        };

        assert!(cmd.execute(&mut scene).is_ok());
        assert!(cmd.undo(&mut scene).is_ok());
    }
}

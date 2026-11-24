use crate::types::AlignPosMode;
use core_math::{EulerRot, Quat, Transform};
use geo_kernel::{FaceKey, Mesh};
use graph_engine::{Command, NodeData, NodeId, SceneGraph, SceneNode};

#[derive(Debug)]
pub struct TransformCommand {
    pub node_id: NodeId,
    pub old_transform: Transform,
    pub new_transform: Transform,
}


impl Command for TransformCommand {
    fn name(&self) -> &str {
        "Transform Object"
    }

    fn execute(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        if let Some(node) = scene.nodes.get_mut(self.node_id) {
            node.local_transform = self.new_transform;
            scene.mark_dirty(self.node_id);
            Ok(())
        } else {
            Err("Node not found".into())
        }
    }

    fn undo(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        if let Some(node) = scene.nodes.get_mut(self.node_id) {
            node.local_transform = self.old_transform;
            scene.mark_dirty(self.node_id);
            Ok(())
        } else {
            Err("Node not found".into())
        }
    }
}


#[derive(Debug)]
pub struct SetParentCommand {
    pub child_id: NodeId,
    pub new_parent: Option<NodeId>,
    pub old_parent: Option<NodeId>,
}


impl Command for SetParentCommand {
    fn name(&self) -> &str {
        "Parent Hierarchy"
    }

    fn execute(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        if self.old_parent.is_none() {
            if let Some(child) = scene.nodes.get(self.child_id) {
                self.old_parent = child.parent;
            }
        }
        scene.set_parent(self.child_id, self.new_parent);
        scene.update_transforms();
        Ok(())
    }

    fn undo(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        scene.set_parent(self.child_id, self.old_parent);
        scene.update_transforms();
        Ok(())
    }
}


#[derive(Debug)]
pub struct JoinMeshesCommand {
    pub target_id: NodeId,
    pub source_id: NodeId,
    pub old_target_mesh: Option<Mesh>,
    pub source_name: String,
    pub source_transform: Transform,
    pub source_base_mesh: Option<Mesh>,
}


impl Command for JoinMeshesCommand {
    fn name(&self) -> &str {
        "Join Meshes (Ctrl+J)"
    }

    fn execute(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        scene.update_transforms();

        let (src_name, src_trans, src_mesh, src_world) = {
            let source_node = scene.nodes.get(self.source_id).ok_or("Source node not found")?;
            match &source_node.data {
                NodeData::Mesh { base_mesh, .. } => (
                    source_node.name.clone(),
                    source_node.local_transform,
                    base_mesh.clone(),
                    source_node.world_matrix,
                ),
                _ => return Err("Source node is not a mesh".into()),
            }
        };

        self.source_name = src_name;
        self.source_transform = src_trans;
        self.source_base_mesh = Some(src_mesh.clone());

        let target_node = scene.nodes.get_mut(self.target_id).ok_or("Target node not found")?;
        let rel_transform = target_node.world_matrix.inverse() * src_world;

        match &mut target_node.data {
            NodeData::Mesh { base_mesh, cached_render_mesh, .. } => {
                if self.old_target_mesh.is_none() {
                    self.old_target_mesh = Some(base_mesh.clone());
                }
                base_mesh.join(&src_mesh, rel_transform).map_err(|e| format!("Join failed: {:?}", e))?;
                *cached_render_mesh = None;
            }
            _ => return Err("Target node is not a mesh".into()),
        }

        scene.remove_node(self.source_id);
        scene.mark_dirty(self.target_id);
        Ok(())
    }

    fn undo(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        if let Some(old_mesh) = &self.old_target_mesh {
            if let Some(target_node) = scene.nodes.get_mut(self.target_id) {
                if let NodeData::Mesh { base_mesh, cached_render_mesh, .. } = &mut target_node.data {
                    *base_mesh = old_mesh.clone();
                    *cached_render_mesh = None;
                }
                scene.mark_dirty(self.target_id);
            }
        }

        if let Some(src_mesh) = self.source_base_mesh.clone() {
            let restored_node = SceneNode::new_mesh(&self.source_name, self.source_transform, src_mesh);
            let id = scene.add_node(restored_node);
            self.source_id = id;
        }

        Ok(())
    }
}


#[derive(Debug)]
pub struct SeparateLoosePartsCommand {
    pub source_id: NodeId,
    pub created_node_ids: Vec<NodeId>,
    pub old_source_mesh: Option<Mesh>,
}


impl Command for SeparateLoosePartsCommand {
    fn name(&self) -> &str {
        "Separate by Loose Parts"
    }

    fn execute(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        let (source_name, source_trans, parent_id, parts) = {
            let source_node = scene.nodes.get_mut(self.source_id).ok_or("Source node not found")?;
            match &mut source_node.data {
                NodeData::Mesh { base_mesh, cached_render_mesh, .. } => {
                    if self.old_source_mesh.is_none() {
                        self.old_source_mesh = Some(base_mesh.clone());
                    }
                    let parts = base_mesh.separate_loose_parts().map_err(|e| format!("Separate loose parts failed: {:?}", e))?;
                    if parts.len() <= 1 {
                        return Ok(());
                    }
                    *base_mesh = parts[0].clone();
                    *cached_render_mesh = None;
                    (source_node.name.clone(), source_node.local_transform, source_node.parent, parts[1..].to_vec())
                }
                _ => return Err("Source node is not a mesh".into()),
            }
        };

        scene.mark_dirty(self.source_id);

        self.created_node_ids.clear();
        for (i, part_mesh) in parts.into_iter().enumerate() {
            let new_name = format!("{}_part_{}", source_name, i + 2);
            let mut new_node = SceneNode::new_mesh(new_name, source_trans, part_mesh);
            new_node.parent = parent_id;
            let new_id = scene.add_node(new_node);
            self.created_node_ids.push(new_id);
        }

        Ok(())
    }

    fn undo(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        for &created_id in &self.created_node_ids {
            scene.remove_node(created_id);
        }
        self.created_node_ids.clear();

        if let Some(old_mesh) = &self.old_source_mesh {
            if let Some(node) = scene.nodes.get_mut(self.source_id) {
                if let NodeData::Mesh { base_mesh, cached_render_mesh, .. } = &mut node.data {
                    *base_mesh = old_mesh.clone();
                    *cached_render_mesh = None;
                }
                scene.mark_dirty(self.source_id);
            }
        }
        Ok(())
    }
}


#[derive(Debug)]
pub struct SeparateByMaterialCommand {
    pub source_id: NodeId,
    pub created_node_ids: Vec<NodeId>,
    pub old_source_mesh: Option<Mesh>,
}


impl Command for SeparateByMaterialCommand {
    fn name(&self) -> &str {
        "Separate by Material"
    }

    fn execute(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        let (source_name, source_trans, parent_id, parts) = {
            let source_node = scene.nodes.get_mut(self.source_id).ok_or("Source node not found")?;
            match &mut source_node.data {
                NodeData::Mesh { base_mesh, cached_render_mesh, .. } => {
                    if self.old_source_mesh.is_none() {
                        self.old_source_mesh = Some(base_mesh.clone());
                    }
                    let parts = base_mesh.separate_by_material().map_err(|e| format!("Separate by material failed: {:?}", e))?;
                    if parts.len() <= 1 {
                        return Ok(());
                    }
                    *base_mesh = parts[0].1.clone();
                    *cached_render_mesh = None;
                    (source_node.name.clone(), source_node.local_transform, source_node.parent, parts)
                }
                _ => return Err("Source node is not a mesh".into()),
            }
        };

        scene.mark_dirty(self.source_id);

        self.created_node_ids.clear();
        for (mat_id, part_mesh) in parts.into_iter().skip(1) {
            let new_name = format!("{}_mat_{}", source_name, mat_id);
            let mut new_node = SceneNode::new_mesh(new_name, source_trans, part_mesh);
            new_node.parent = parent_id;
            let new_id = scene.add_node(new_node);
            self.created_node_ids.push(new_id);
        }

        Ok(())
    }

    fn undo(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        for &created_id in &self.created_node_ids {
            scene.remove_node(created_id);
        }
        self.created_node_ids.clear();

        if let Some(old_mesh) = &self.old_source_mesh {
            if let Some(node) = scene.nodes.get_mut(self.source_id) {
                if let NodeData::Mesh { base_mesh, cached_render_mesh, .. } = &mut node.data {
                    *base_mesh = old_mesh.clone();
                    *cached_render_mesh = None;
                }
                scene.mark_dirty(self.source_id);
            }
        }
        Ok(())
    }
}


#[derive(Debug)]
pub struct SeparateFacesCommand {
    pub source_id: NodeId,
    pub faces_to_extract: Vec<FaceKey>,
    pub created_node_id: Option<NodeId>,
    pub old_source_mesh: Option<Mesh>,
}


impl Command for SeparateFacesCommand {
    fn name(&self) -> &str {
        "Separate Faces (P)"
    }

    fn execute(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        let (source_name, source_trans, extracted_mesh) = {
            let source_node = scene.nodes.get_mut(self.source_id).ok_or("Source node not found")?;
            match &mut source_node.data {
                NodeData::Mesh { base_mesh, cached_render_mesh, .. } => {
                    if self.old_source_mesh.is_none() {
                        self.old_source_mesh = Some(base_mesh.clone());
                    }
                    let extracted = base_mesh.separate_faces(&self.faces_to_extract).map_err(|e| format!("Separate failed: {:?}", e))?;
                    *cached_render_mesh = None;
                    (source_node.name.clone(), source_node.local_transform, extracted)
                }
                _ => return Err("Source node is not a mesh".into()),
            }
        };

        scene.mark_dirty(self.source_id);

        let new_name = format!("{}_separated", source_name);
        let new_node = SceneNode::new_mesh(new_name, source_trans, extracted_mesh);
        let new_id = scene.add_node(new_node);
        self.created_node_id = Some(new_id);

        Ok(())
    }

    fn undo(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        if let Some(created_id) = self.created_node_id.take() {
            scene.remove_node(created_id);
        }

        if let Some(old_mesh) = &self.old_source_mesh {
            if let Some(node) = scene.nodes.get_mut(self.source_id) {
                if let NodeData::Mesh { base_mesh, cached_render_mesh, .. } = &mut node.data {
                    *base_mesh = old_mesh.clone();
                    *cached_render_mesh = None;
                }
                scene.mark_dirty(self.source_id);
            }
        }

        Ok(())
    }
}


#[derive(Debug)]
pub struct AlignObjectsCommand {
    pub source_id: NodeId,
    pub target_id: NodeId,
    pub align_pos_x: bool,
    pub align_pos_y: bool,
    pub align_pos_z: bool,
    pub align_pos_mode: AlignPosMode,
    pub align_rot_x: bool,
    pub align_rot_y: bool,
    pub align_rot_z: bool,
    pub align_scale_x: bool,
    pub align_scale_y: bool,
    pub align_scale_z: bool,
    pub old_transform: Option<Transform>,
}


impl Command for AlignObjectsCommand {
    fn name(&self) -> &str {
        "Align Objects (Alt+A)"
    }

    fn execute(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        let (tgt_trans, tgt_box) = {
            let tgt_node = scene.nodes.get(self.target_id).ok_or("Target node not found")?;
            let b_box = match &tgt_node.data {
                NodeData::Mesh { base_mesh, .. } => {
                    let (min, max) = base_mesh.bounding_box();
                    (tgt_node.world_matrix.transform_point3(min), tgt_node.world_matrix.transform_point3(max))
                }
                _ => (tgt_node.local_transform.translation, tgt_node.local_transform.translation),
            };
            (tgt_node.local_transform, b_box)
        };

        let src_node = scene.nodes.get_mut(self.source_id).ok_or("Source node not found")?;
        if self.old_transform.is_none() {
            self.old_transform = Some(src_node.local_transform);
        }

        let src_box = match &src_node.data {
            NodeData::Mesh { base_mesh, .. } => {
                let (min, max) = base_mesh.bounding_box();
                (src_node.world_matrix.transform_point3(min), src_node.world_matrix.transform_point3(max))
            }
            _ => (src_node.local_transform.translation, src_node.local_transform.translation),
        };

        let src_ref = match self.align_pos_mode {
            AlignPosMode::Pivot => src_node.local_transform.translation,
            AlignPosMode::Center => (src_box.0 + src_box.1) * 0.5,
            AlignPosMode::Minimum => src_box.0,
            AlignPosMode::Maximum => src_box.1,
        };
        let tgt_ref = match self.align_pos_mode {
            AlignPosMode::Pivot => tgt_trans.translation,
            AlignPosMode::Center => (tgt_box.0 + tgt_box.1) * 0.5,
            AlignPosMode::Minimum => tgt_box.0,
            AlignPosMode::Maximum => tgt_box.1,
        };
        let d_pos = tgt_ref - src_ref;

        if self.align_pos_x {
            src_node.local_transform.translation.x += d_pos.x;
        }
        if self.align_pos_y {
            src_node.local_transform.translation.y += d_pos.y;
        }
        if self.align_pos_z {
            src_node.local_transform.translation.z += d_pos.z;
        }

        if self.align_rot_x && self.align_rot_y && self.align_rot_z {
            src_node.local_transform.rotation = tgt_trans.rotation;
        } else if self.align_rot_x || self.align_rot_y || self.align_rot_z {
            let (src_yaw, src_pitch, src_roll) = src_node.local_transform.rotation.to_euler(EulerRot::YXZ);
            let (tgt_yaw, tgt_pitch, tgt_roll) = tgt_trans.rotation.to_euler(EulerRot::YXZ);
            let new_yaw = if self.align_rot_y { tgt_yaw } else { src_yaw };
            let new_pitch = if self.align_rot_x { tgt_pitch } else { src_pitch };
            let new_roll = if self.align_rot_z { tgt_roll } else { src_roll };
            src_node.local_transform.rotation = Quat::from_euler(EulerRot::YXZ, new_yaw, new_pitch, new_roll);
        }

        if self.align_scale_x {
            src_node.local_transform.scale.x = tgt_trans.scale.x;
        }
        if self.align_scale_y {
            src_node.local_transform.scale.y = tgt_trans.scale.y;
        }
        if self.align_scale_z {
            src_node.local_transform.scale.z = tgt_trans.scale.z;
        }

        scene.mark_dirty(self.source_id);
        Ok(())
    }

    fn undo(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        if let Some(old) = self.old_transform {
            let src_node = scene.nodes.get_mut(self.source_id).ok_or("Source node not found")?;
            src_node.local_transform = old;
            scene.mark_dirty(self.source_id);
            Ok(())
        } else {
            Err("No snapshot to restore".into())
        }
    }
}


#[derive(Debug)]
pub struct ApplyModifierCommand {
    pub node_id: NodeId,
    pub modifier_idx: usize,
    pub modifier_name: String,
    pub old_base_mesh: Mesh,
}


impl Command for ApplyModifierCommand {
    fn name(&self) -> &str {
        &self.modifier_name
    }

    fn execute(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        scene.apply_modifier(self.node_id, self.modifier_idx)
    }

    fn undo(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        let node = scene.nodes.get_mut(self.node_id).ok_or("Node not found")?;
        if let NodeData::Mesh { base_mesh, cached_render_mesh, .. } = &mut node.data {
            *base_mesh = self.old_base_mesh.clone();
            *cached_render_mesh = None;
            scene.mark_dirty(self.node_id);
            Ok(())
        } else {
            Err("Node is not a mesh".into())
        }
    }
}


#[derive(Debug)]
pub struct CollapseStackCommand {
    pub node_id: NodeId,
    pub old_base_mesh: Mesh,
}


impl Command for CollapseStackCommand {
    fn name(&self) -> &str {
        "Collapse Modifier Stack"
    }

    fn execute(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        scene.collapse_modifier_stack(self.node_id)
    }

    fn undo(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        let node = scene.nodes.get_mut(self.node_id).ok_or("Node not found")?;
        if let NodeData::Mesh { base_mesh, cached_render_mesh, .. } = &mut node.data {
            *base_mesh = self.old_base_mesh.clone();
            *cached_render_mesh = None;
            scene.mark_dirty(self.node_id);
            Ok(())
        } else {
            Err("Node is not a mesh".into())
        }
    }
}


#[derive(Debug)]
pub struct GroupNodesCommand {
    pub node_ids: Vec<NodeId>,
    pub group_name: String,
    pub created_group_id: Option<NodeId>,
    pub old_parents: Vec<(NodeId, Option<NodeId>, Transform)>,
}

impl Command for GroupNodesCommand {
    fn name(&self) -> &str {
        "Group Nodes (Ctrl+G)"
    }

    fn execute(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        if self.node_ids.is_empty() {
            return Err("No nodes selected to group".into());
        }
        scene.update_transforms();

        let mut sum = core_math::Vec3::ZERO;
        let mut count = 0;
        let mut old_info = Vec::new();
        for &id in &self.node_ids {
            if let Some(node) = scene.nodes.get(id) {
                sum += node.world_matrix.transform_point3(core_math::Vec3::ZERO);
                count += 1;
                old_info.push((id, node.parent, node.local_transform));
            }
        }
        if count == 0 {
            return Err("Selected nodes not found in scene".into());
        }
        self.old_parents = old_info;
        let centroid = sum / count as f32;

        let group_node = SceneNode::new_transform(&self.group_name, Transform::from_translation(centroid));
        let group_id = scene.add_node(group_node);
        self.created_group_id = Some(group_id);

        scene.update_transforms();
        let inv_group_world = scene.nodes.get(group_id).unwrap().world_matrix.inverse();

        for &id in &self.node_ids {
            if let Some(node) = scene.nodes.get_mut(id) {
                let world_pos = node.world_matrix.transform_point3(core_math::Vec3::ZERO);
                let local_in_group = inv_group_world.transform_point3(world_pos);
                node.local_transform.translation = local_in_group;
            }
            scene.set_parent(id, Some(group_id));
        }
        scene.update_transforms();
        Ok(())
    }

    fn undo(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        let Some(group_id) = self.created_group_id else {
            return Ok(());
        };

        for &(id, old_parent, old_trans) in &self.old_parents {
            scene.set_parent(id, old_parent);
            if let Some(node) = scene.nodes.get_mut(id) {
                node.local_transform = old_trans;
                scene.mark_dirty(id);
            }
        }
        scene.remove_node(group_id);
        scene.update_transforms();
        Ok(())
    }
}



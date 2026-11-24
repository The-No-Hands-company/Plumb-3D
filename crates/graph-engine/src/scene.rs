use core_math::{Mat4, Vec3};
use geo_kernel::{RenderMesh, ShadingMode, SkinBone, SkinModifier, Spline};
use slotmap::SlotMap;

use crate::node::{NodeData, NodeId, SceneNode};

/// The Maya-inspired Dependency / Scene Graph Engine
#[derive(Default, Debug)]
pub struct SceneGraph {
    pub nodes: SlotMap<NodeId, SceneNode>,
    pub root_nodes: Vec<NodeId>,
    pub last_added_node: Option<NodeId>,
}

impl SceneGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_node(&mut self, node: SceneNode) -> NodeId {
        let parent = node.parent;
        let id = self.nodes.insert(node);
        self.last_added_node = Some(id);
        if let Some(p) = parent {
            if let Some(parent_node) = self.nodes.get_mut(p) {
                parent_node.children.push(id);
            }
        } else {
            self.root_nodes.push(id);
        }
        self.mark_dirty(id);
        id
    }

    pub fn set_parent(&mut self, child_id: NodeId, new_parent: Option<NodeId>) {
        // Remove from old parent or root
        if let Some(child) = self.nodes.get_mut(child_id) {
            let old_parent = child.parent;
            child.parent = new_parent;

            if let Some(old_p) = old_parent {
                if let Some(p_node) = self.nodes.get_mut(old_p) {
                    p_node.children.retain(|&c| c != child_id);
                }
            } else {
                self.root_nodes.retain(|&r| r != child_id);
            }
        }

        // Add to new parent or root
        if let Some(np) = new_parent {
            if let Some(np_node) = self.nodes.get_mut(np) {
                np_node.children.push(child_id);
            }
        } else {
            self.root_nodes.push(child_id);
        }

        self.mark_dirty(child_id);
    }

    /// Removes a node from the scene graph, unlinking it from parents and promoting children to root.
    pub fn remove_node(&mut self, node_id: NodeId) -> Option<SceneNode> {
        let node = self.nodes.remove(node_id)?;

        // Unlink from parent
        if let Some(parent_id) = node.parent {
            if let Some(parent_node) = self.nodes.get_mut(parent_id) {
                parent_node.children.retain(|&c| c != node_id);
            }
        } else {
            self.root_nodes.retain(|&r| r != node_id);
        }

        // Promote children to root
        for &child_id in &node.children {
            if let Some(child) = self.nodes.get_mut(child_id) {
                child.parent = None;
                self.root_nodes.push(child_id);
                self.mark_dirty(child_id);
            }
        }

        Some(node)
    }

    /// Maya-style dirty flag propagation: marks node and all downstream descendants dirty
    pub fn mark_dirty(&mut self, start_node: NodeId) {
        let mut stack = vec![start_node];
        while let Some(current) = stack.pop() {
            if let Some(node) = self.nodes.get_mut(current) {
                node.is_dirty = true;
                match &mut node.data {
                    NodeData::Mesh { cached_render_mesh, modifier_stack, .. } => {
                        *cached_render_mesh = None;
                        modifier_stack.mark_dirty();
                    }
                    NodeData::Curve { cached_render_mesh, .. } => {
                        *cached_render_mesh = None;
                    }
                    _ => {}
                }
                for &child in &node.children {
                    stack.push(child);
                }
            }
        }
    }

    /// Evaluate world matrices down the hierarchy
    pub fn update_transforms(&mut self) {
        for &root in &self.root_nodes.clone() {
            self.update_node_transform(root, Mat4::IDENTITY);
        }
    }

    fn update_node_transform(&mut self, node_id: NodeId, parent_world_matrix: Mat4) {
        let (world_matrix, children) = if let Some(node) = self.nodes.get_mut(node_id) {
            let local_matrix = node.local_transform.to_matrix();
            let world_matrix = parent_world_matrix * local_matrix;
            node.world_matrix = world_matrix;
            node.is_dirty = false;
            (world_matrix, node.children.clone())
        } else {
            return;
        };

        for child in children {
            self.update_node_transform(child, world_matrix);
        }
    }

    /// Pull-based evaluation: obtain or compute render mesh for a mesh node
    pub fn evaluate_mesh(&mut self, node_id: NodeId) -> Option<&RenderMesh> {
        self.evaluate_mesh_with_weights(node_id, None)
    }

    /// Pull-based evaluation: obtain or compute render mesh with optional vertex group heatmap weighting
    pub fn evaluate_mesh_with_weights(&mut self, node_id: NodeId, weight_group: Option<&str>) -> Option<&RenderMesh> {
        let node = self.nodes.get_mut(node_id)?;
        match &mut node.data {
            NodeData::Mesh {
                base_mesh,
                modifier_stack,
                shading_mode,
                cached_render_mesh,
                ..
            } => {
                if cached_render_mesh.is_none() {
                    modifier_stack.mark_dirty();
                    let final_mesh = modifier_stack.evaluate(base_mesh).ok()?;
                    let render_mesh = final_mesh.to_render_mesh_with_weights(
                        match shading_mode {
                            ShadingMode::Flat => ShadingMode::Flat,
                            ShadingMode::Smooth => ShadingMode::Smooth,
                        },
                        weight_group,
                    );
                    *cached_render_mesh = Some(render_mesh);
                }
                cached_render_mesh.as_ref()
            }
            NodeData::Curve {
                spline,
                bevel_radius,
                bevel_resolution,
                cached_render_mesh,
                profile_shape,
                ..
            } => {
                if *bevel_radius > 0.0 && spline.points.len() >= 2 {
                    if cached_render_mesh.is_none() {
                        let cs = profile_shape.generate_cross_section(*bevel_radius, *bevel_resolution);
                        let sampled = spline.sample(12);
                        if let Ok(beveled_mesh) = Spline::sweep(&cs, &sampled, spline.is_closed) {
                            let render_mesh = beveled_mesh.to_render_mesh(ShadingMode::Smooth);
                            *cached_render_mesh = Some(render_mesh);
                        }
                    }
                    cached_render_mesh.as_ref()
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Synchronizes all `SkinModifier`s across all meshes in the scene with the current world matrices of matching `Joint` nodes.
    pub fn sync_skeletons(&mut self) {
        let mut joint_world_transforms = std::collections::HashMap::new();
        for (_, node) in &self.nodes {
            if matches!(node.data, NodeData::Joint { .. }) {
                joint_world_transforms.insert(node.name.clone(), node.world_matrix);
            }
        }

        if joint_world_transforms.is_empty() {
            return;
        }

        for (_, node) in &mut self.nodes {
            if let NodeData::Mesh { modifier_stack, cached_render_mesh, .. } = &mut node.data {
                let mut modified = false;
                for modifier in modifier_stack.modifiers_mut() {
                    if let Some(skin) = modifier.as_skin_mut() {
                        for bone in &mut skin.bones {
                            if let Some(&new_wm) = joint_world_transforms.get(&bone.name) {
                                if (bone.current_world_matrix - new_wm).abs_diff_eq(Mat4::ZERO, 1e-4) == false {
                                    bone.current_world_matrix = new_wm;
                                    modified = true;
                                }
                            }
                        }
                    }
                }
                if modified {
                    *cached_render_mesh = None;
                }
            }
        }
    }

    /// Synchronizes all `CurveModifier`s across all meshes with their target `Curve` node splines.
    pub fn sync_curves(&mut self) {
        let mut curve_splines = std::collections::HashMap::new();
        for (_, node) in &self.nodes {
            if let NodeData::Curve { spline, .. } = &node.data {
                curve_splines.insert(node.name.clone(), spline.clone());
            }
        }

        if curve_splines.is_empty() {
            return;
        }

        for (_, node) in &mut self.nodes {
            if let NodeData::Mesh { modifier_stack, cached_render_mesh, .. } = &mut node.data {
                let mut modified = false;
                for modifier in modifier_stack.modifiers_mut() {
                    if let Some(curve_mod) = modifier.as_curve_mut() {
                        if let Some(ref target_name) = curve_mod.target_curve {
                            if let Some(new_spline) = curve_splines.get(target_name) {
                                curve_mod.spline = new_spline.clone();
                                modified = true;
                            }
                        }
                    }
                }
                if modified {
                    *cached_render_mesh = None;
                }
            }
        }
    }

    /// Synchronizes all `BooleanModifier`s across all meshes in the scene with their target mesh nodes in world space.
    pub fn sync_booleans(&mut self) {
        let mut mesh_sources = std::collections::HashMap::new();
        for (_, node) in &self.nodes {
            if let NodeData::Mesh { base_mesh, .. } = &node.data {
                mesh_sources.insert(node.name.clone(), (base_mesh.clone(), node.world_matrix));
            }
        }

        if mesh_sources.len() < 2 {
            return;
        }

        for (_, node) in &mut self.nodes {
            let self_world_inv = node.world_matrix.inverse();
            if let NodeData::Mesh { modifier_stack, cached_render_mesh, .. } = &mut node.data {
                let mut modified = false;
                for modifier in modifier_stack.modifiers_mut() {
                    if let Some(bool_mod) = modifier.as_boolean_mut() {
                        if let Some(ref target_name) = bool_mod.target_node_name {
                            if let Some((target_base_mesh, target_world)) = mesh_sources.get(target_name) {
                                let local_transform = self_world_inv * *target_world;
                                let mut local_operand = target_base_mesh.clone();
                                for v in local_operand.vertices.values_mut() {
                                    v.position = local_transform.transform_point3(v.position);
                                }
                                local_operand.update_normals();
                                bool_mod.operand = local_operand;
                                modified = true;
                            }
                        }
                    }
                }
                if modified {
                    *cached_render_mesh = None;
                }
            }
        }
    }

    /// Synchronizes all `ShrinkwrapModifier`s across all meshes in the scene with their target mesh nodes in world space.
    pub fn sync_shrinkwraps(&mut self) {
        let mut mesh_sources = std::collections::HashMap::new();
        for (_, node) in &self.nodes {
            if let NodeData::Mesh { base_mesh, .. } = &node.data {
                mesh_sources.insert(node.name.clone(), (base_mesh.clone(), node.world_matrix));
            }
        }

        if mesh_sources.len() < 2 {
            return;
        }

        for (_, node) in &mut self.nodes {
            let self_world_inv = node.world_matrix.inverse();
            if let NodeData::Mesh { modifier_stack, cached_render_mesh, .. } = &mut node.data {
                let mut modified = false;
                for modifier in modifier_stack.modifiers_mut() {
                    if let Some(shrink_mod) = modifier.as_shrinkwrap_mut() {
                        if let Some(ref target_name) = shrink_mod.target_node_name {
                            if let Some((target_base_mesh, target_world)) = mesh_sources.get(target_name) {
                                let local_transform = self_world_inv * *target_world;
                                let mut local_target = target_base_mesh.clone();
                                for v in local_target.vertices.values_mut() {
                                    v.position = local_transform.transform_point3(v.position);
                                }
                                local_target.update_normals();
                                shrink_mod.target_mesh = Some(local_target);
                                modified = true;
                            }
                        }
                    }
                }
                if modified {
                    *cached_render_mesh = None;
                }
            }
        }
    }

    /// Automatically binds a mesh node to a skeletal armature rooted at `root_joint_id`.
    /// 1. Recursively collects all Joint nodes starting from root_joint_id.
    /// 2. Sets up SkinBone with inverse bind matrices = joint.world_matrix.inverse().
    /// 3. Computes smooth distance-based heat weights for all mesh vertices to the joint positions.
    /// 4. Adds or updates a SkinModifier in the mesh's modifier stack.
    pub fn auto_bind_mesh_to_armature(
        &mut self,
        mesh_id: NodeId,
        root_joint_id: NodeId,
    ) -> Result<(), String> {
        self.update_transforms();

        // Collect all joints in hierarchy
        let mut joint_info = Vec::new(); // (name, world_pos, world_matrix)
        let mut stack = vec![root_joint_id];
        while let Some(jid) = stack.pop() {
            if let Some(node) = self.nodes.get(jid) {
                if matches!(node.data, NodeData::Joint { .. }) {
                    let world_pos = node.world_matrix.transform_point3(Vec3::ZERO);
                    joint_info.push((node.name.clone(), world_pos, node.world_matrix));
                }
                for &child in &node.children {
                    stack.push(child);
                }
            }
        }

        if joint_info.is_empty() {
            return Err("No Joint nodes found under the specified root joint".into());
        }

        let mesh_node = self.nodes.get_mut(mesh_id).ok_or("Mesh node not found")?;
        let mesh_world_mat = mesh_node.world_matrix;

        if let NodeData::Mesh { base_mesh, modifier_stack, cached_render_mesh, .. } = &mut mesh_node.data {
            let mut bones = Vec::with_capacity(joint_info.len());
            for (name, _, wm) in &joint_info {
                bones.push(SkinBone::new(
                    name.clone(),
                    wm.inverse(),
                    *wm,
                ));
            }

            // Create/populate vertex groups for each bone
            for (bone_name, _, _) in &joint_info {
                base_mesh.add_vertex_group(bone_name);
            }

            // Compute distance weighting for each vertex
            let vertex_keys: Vec<_> = base_mesh.vertices.keys().collect();
            for vk in vertex_keys {
                let local_pos = base_mesh.vertices[vk].position;
                let world_pos = mesh_world_mat.transform_point3(local_pos);

                // Inverse distance squared weights to all joints
                let mut weights = Vec::with_capacity(joint_info.len());
                let mut sum_w = 0.0;
                for (_, joint_pos, _) in &joint_info {
                    let d = (world_pos - *joint_pos).length();
                    let w = 1.0 / (d.powi(2) + 0.05);
                    weights.push(w);
                    sum_w += w;
                }

                if sum_w > 1e-6 {
                    for (bone_idx, (bone_name, _, _)) in joint_info.iter().enumerate() {
                        let normalized_w = (weights[bone_idx] / sum_w).clamp(0.0, 1.0);
                        if let Some(vg) = base_mesh.get_vertex_group_mut(bone_name) {
                            vg.set_weight(vk, normalized_w);
                        }
                    }
                }
            }

            let mut found_skin = false;
            for modifier in modifier_stack.modifiers_mut() {
                if let Some(skin) = modifier.as_skin_mut() {
                    skin.bones = bones.clone();
                    found_skin = true;
                    break;
                }
            }
            if !found_skin {
                modifier_stack.add_modifier(Box::new(SkinModifier::new(bones)));
            }

            *cached_render_mesh = None;
            mesh_node.is_dirty = true;
            Ok(())
        } else {
            Err("Target node is not a Mesh".into())
        }
    }

    /// Apply an individual modifier from the node's modifier stack to its base mesh (Blender "Apply Modifier").
    pub fn apply_modifier(&mut self, node_id: NodeId, modifier_idx: usize) -> Result<(), String> {
        let node = self.nodes.get_mut(node_id).ok_or("Node not found")?;
        if let NodeData::Mesh { base_mesh, modifier_stack, cached_render_mesh, .. } = &mut node.data {
            if modifier_idx >= modifier_stack.modifiers().len() {
                return Err("Modifier index out of bounds".into());
            }
            let modifier = modifier_stack.remove_modifier(modifier_idx);
            *base_mesh = modifier.apply(base_mesh).map_err(|e| format!("Failed to apply modifier: {:?}", e))?;
            *cached_render_mesh = None;
            node.is_dirty = true;
            Ok(())
        } else {
            Err("Node is not a mesh".into())
        }
    }

    /// Collapse all modifiers in the stack directly into the base mesh (3ds Max "Collapse All").
    pub fn collapse_modifier_stack(&mut self, node_id: NodeId) -> Result<(), String> {
        let node = self.nodes.get_mut(node_id).ok_or("Node not found")?;
        if let NodeData::Mesh { base_mesh, modifier_stack, cached_render_mesh, .. } = &mut node.data {
            let final_mesh = modifier_stack.evaluate(base_mesh).map_err(|e| format!("Failed to evaluate stack: {:?}", e))?.clone();
            *base_mesh = final_mesh;
            modifier_stack.clear();
            *cached_render_mesh = None;
            node.is_dirty = true;
            Ok(())
        } else {
            Err("Node is not a mesh".into())
        }
    }
}

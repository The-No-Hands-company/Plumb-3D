use geo_kernel::{FaceKey, MirrorAxis};
use graph_engine::{Material, NodeData};

use super::UiActions;
use crate::commands::{
    AddMaterialSlotCommand, AssignMaterialCommand, MirrorVertexGroupCommand,
    NormalizeVertexGroupsCommand, RemoveMaterialSlotCommand,
};
use crate::state::AppState;

pub fn handle_materials_and_groups(state: &mut AppState, actions: &mut UiActions) {
    // Handle Mirror Vertex Group
    if let Some((src_name, dst_name)) = actions.trigger_mirror_vertex_group.take() {
        if let Some(sel_id) = state.selected_node {
            let cmd = Box::new(MirrorVertexGroupCommand {
                node_id: sel_id,
                source_group: src_name,
                target_group: dst_name,
                axis: MirrorAxis::X,
                tolerance: 0.05,
                old_mesh: None,
            });
            let _ = state.history.execute(cmd, &mut state.scene);
        }
    }

    // Handle Normalize Vertex Groups
    if let Some(group_names) = actions.trigger_normalize_vertex_groups.take() {
        if let Some(sel_id) = state.selected_node {
            let cmd = Box::new(NormalizeVertexGroupsCommand {
                node_id: sel_id,
                group_names,
                old_mesh: None,
            });
            let _ = state.history.execute(cmd, &mut state.scene);
        }
    }

    // Handle Add Material Slot
    if actions.trigger_add_material_slot {
        if let Some(sel_id) = state.selected_node {
            let slot_count = if let Some(node) = state.scene.nodes.get(sel_id) {
                if let NodeData::Mesh { materials, .. } = &node.data {
                    materials.len()
                } else {
                    0
                }
            } else {
                0
            };
            let new_mat = Material::new(
                format!("Material.{:03}", slot_count + 1),
                [0.72, 0.73, 0.75, 1.0],
                0.0,
                0.5,
                0.0,
                0,
            );
            let cmd = Box::new(AddMaterialSlotCommand {
                node_id: sel_id,
                material: new_mat,
            });
            let _ = state.history.execute(cmd, &mut state.scene);
            state.active_material_slot = slot_count;
        }
    }

    // Handle Remove Material Slot
    if actions.trigger_remove_material_slot {
        if let Some(sel_id) = state.selected_node {
            let cmd = Box::new(RemoveMaterialSlotCommand {
                node_id: sel_id,
                slot_index: state.active_material_slot,
                removed_material: None,
            });
            let _ = state.history.execute(cmd, &mut state.scene);
            if state.active_material_slot > 0 {
                state.active_material_slot -= 1;
            }
        }
    }

    // Handle Assign Material Slot to Selected Faces
    if let Some(slot_idx) = actions.trigger_assign_material_slot.take() {
        if let Some(sel_id) = state.selected_node {
            let faces: Vec<FaceKey> = state.selected_faces.iter().copied().collect();
            if !faces.is_empty() {
                let cmd = Box::new(AssignMaterialCommand {
                    node_id: sel_id,
                    faces,
                    material_id: slot_idx,
                    old_assignments: Vec::new(),
                });
                let _ = state.history.execute(cmd, &mut state.scene);
            }
        }
    }

    // Handle Select Faces with Material
    if let Some(slot_idx) = actions.trigger_select_material_faces.take() {
        if let Some(sel_id) = state.selected_node {
            if let Some(node) = state.scene.nodes.get(sel_id) {
                if let NodeData::Mesh { base_mesh, .. } = &node.data {
                    for fk in base_mesh.faces_with_material(slot_idx) {
                        state.selected_faces.insert(fk);
                    }
                }
            }
        }
    }

    // Handle Deselect Faces with Material
    if let Some(slot_idx) = actions.trigger_deselect_material_faces.take() {
        if let Some(sel_id) = state.selected_node {
            if let Some(node) = state.scene.nodes.get(sel_id) {
                if let NodeData::Mesh { base_mesh, .. } = &node.data {
                    for fk in base_mesh.faces_with_material(slot_idx) {
                        state.selected_faces.remove(&fk);
                    }
                }
            }
        }
    }
}

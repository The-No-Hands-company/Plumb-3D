use super::UiActions;
use crate::commands::{AlignObjectsCommand, DeleteFacesCommand, DissolveEdgeCommand, GroupNodesCommand, SetParentCommand};
use crate::state::AppState;
use crate::types::{InteractionMode, SubElementLevel};
use graph_engine::NodeData;

pub fn handle_scene_ops(state: &mut AppState, actions: &mut UiActions) {
    // Handle Quick Group
    if actions.trigger_group_nodes {
        let targets: Vec<_> = if !state.selected_nodes.is_empty() {
            state.selected_nodes.iter().copied().collect()
        } else if let Some(sel) = state.selected_node {
            vec![sel]
        } else {
            Vec::new()
        };

        if !targets.is_empty() {
            let count = state.scene.nodes.values().filter(|n| matches!(n.data, NodeData::Transform)).count();
            let group_name = format!("Group_{}", count + 1);
            let cmd = Box::new(GroupNodesCommand {
                node_ids: targets,
                group_name,
                created_group_id: None,
                old_parents: Vec::new(),
            });
            if state.history.execute(cmd, &mut state.scene).is_ok() {
                if let Some(&gid) = state.scene.root_nodes.last() {
                    state.selected_nodes.clear();
                    state.selected_nodes.insert(gid);
                    state.selected_node = Some(gid);
                }
            }
        }
    }

    // Handle Duplicate Node (Shift+D)
    if actions.trigger_duplicate_node {
        if let Some(sel_id) = state.selected_node {
            let cmd = Box::new(crate::commands::DuplicateNodeCommand::new(
                sel_id,
                core_math::Vec3::new(2.0, 0.0, 0.0),
            ));
            if state.history.execute(cmd, &mut state.scene).is_ok() {
                if let Some(new_id) = state.scene.last_added_node {
                    state.selected_node = Some(new_id);
                    state.selected_nodes.clear();
                    state.selected_nodes.insert(new_id);
                }
            }
        }
    }

    // Handle Precision CAD / Plasticity-style Boolean Difference Carve
    if actions.trigger_boolean_carve {
        if let (Some(target_id), Some(cutter_id)) = (state.selected_node, state.previous_selected_node) {
            if target_id != cutter_id {
                let cutter_name = state.scene.nodes.get(cutter_id).map(|n| n.name.clone());
                if let Some(c_name) = cutter_name {
                    if let Some(target_node) = state.scene.nodes.get_mut(target_id) {
                        if let NodeData::Mesh { modifier_stack, cached_render_mesh, .. } = &mut target_node.data {
                            let bool_mod = geo_kernel::BooleanModifier::with_target(
                                geo_kernel::Mesh::new(),
                                geo_kernel::BooleanOp::Difference,
                                c_name,
                            );
                            modifier_stack.add_modifier(Box::new(bool_mod));
                            *cached_render_mesh = None;
                            state.scene.mark_dirty(target_id);
                        }
                    }
                    state.scene.sync_booleans();
                }
            }
        }
    }
    // Handle Precision CAD / Plasticity-style Loft Curves
    if actions.trigger_loft_curves {
        let curve_ids: Vec<_> = state.selected_nodes.iter().copied().filter(|&id| {
            state.scene.nodes.get(id).is_some_and(|n| matches!(n.data, NodeData::Curve { .. }))
        }).collect();
        if curve_ids.len() >= 2 {
            let cmd = Box::new(crate::commands::LoftCurvesCommand {
                curve_node_ids: curve_ids,
                samples_per_curve: 16,
                closed_u: true,
                closed_v: false,
                created_node_id: None,
            });
            let _ = state.history.execute(cmd, &mut state.scene);
            state.window.request_redraw();
        }
    }

    // Handle Align Objects
    if actions.trigger_apply_align {
        if let (Some(sel_id), Some(prev_id)) = (state.selected_node, state.previous_selected_node) {
            let cmd = Box::new(AlignObjectsCommand {
                source_id: sel_id,
                target_id: prev_id,
                align_pos_x: state.align_pos_x,
                align_pos_y: state.align_pos_y,
                align_pos_z: state.align_pos_z,
                align_pos_mode: state.align_pos_mode,
                align_rot_x: state.align_rot_x,
                align_rot_y: state.align_rot_y,
                align_rot_z: state.align_rot_z,
                align_scale_x: state.align_scale_x,
                align_scale_y: state.align_scale_y,
                align_scale_z: state.align_scale_z,
                old_transform: None,
            });
            let _ = state.history.execute(cmd, &mut state.scene);
        }
    }

    // Handle Maya Hierarchy Parenting
    if actions.trigger_parent {
        if let (Some(child_id), Some(parent_id)) = (state.selected_node, state.previous_selected_node) {
            if child_id != parent_id {
                let cmd = Box::new(SetParentCommand {
                    child_id,
                    new_parent: Some(parent_id),
                    old_parent: None,
                });
                let _ = state.history.execute(cmd, &mut state.scene);
            }
        }
    }
    if actions.trigger_unparent {
        if let Some(child_id) = state.selected_node {
            let cmd = Box::new(SetParentCommand {
                child_id,
                new_parent: None,
                old_parent: None,
            });
            let _ = state.history.execute(cmd, &mut state.scene);
        }
    }

    // Handle Undo/Redo
    if actions.trigger_undo {
        let _ = state.history.undo(&mut state.scene);
    }
    if actions.trigger_redo {
        let _ = state.history.redo(&mut state.scene);
    }

    // Handle Delete
    if actions.delete_selected {
        if state.interaction_mode == InteractionMode::Edit {
            if let Some(sel_id) = state.selected_node {
                if state.sub_element_level == SubElementLevel::Face && !state.selected_faces.is_empty() {
                    let faces: Vec<geo_kernel::FaceKey> = state.selected_faces.iter().copied().collect();
                    let cmd = Box::new(DeleteFacesCommand {
                        node_id: sel_id,
                        faces,
                        old_mesh: None,
                    });
                    if state.history.execute(cmd, &mut state.scene).is_ok() {
                        state.selected_faces.clear();
                        state.selected_vertices.clear();
                        state.selected_edges.clear();
                    }
                } else if state.sub_element_level == SubElementLevel::Edge && !state.selected_edges.is_empty() {
                    let edges: Vec<geo_kernel::EdgeKey> = state.selected_edges.iter().copied().collect();
                    for ek in edges {
                        let cmd = Box::new(DissolveEdgeCommand {
                            node_id: sel_id,
                            edge_key: ek,
                            old_mesh: None,
                        });
                        let _ = state.history.execute(cmd, &mut state.scene);
                    }
                    state.selected_edges.clear();
                    state.selected_vertices.clear();
                    state.selected_faces.clear();
                }
            }
        } else if let Some(sel_id) = state.selected_node {
            state.scene.nodes.remove(sel_id);
            state.scene.root_nodes.retain(|&r| r != sel_id);
            state.gpu_mesh_cache.remove(&sel_id);
            state.object_colors.remove(&sel_id);
            state.object_materials.remove(&sel_id);
            state.selected_node = None;
            state.selected_vertices.clear();
            state.selected_edges.clear();
            state.selected_faces.clear();
        }
    }

    // Handle Frame Selected (F)
    if actions.trigger_frame_selected {
        state.frame_selected();
    }
}

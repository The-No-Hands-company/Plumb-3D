use core_math::Transform;
use geo_kernel::FaceKey;
use graph_engine::NodeData;
use render_viewport::GizmoAxis;
use winit::keyboard::KeyCode;

use crate::commands::*;
use crate::state::AppState;
use crate::types::{InteractionMode, SubElementLevel};

pub fn handle_scene_history_key(state: &mut AppState, key_code: KeyCode) -> bool {
    match key_code {
        KeyCode::KeyZ => {
            let _ = state.history.undo(&mut state.scene);
            state.window.request_redraw();
            true
        }
        KeyCode::KeyY => {
            let _ = state.history.redo(&mut state.scene);
            state.window.request_redraw();
            true
        }
        KeyCode::KeyG if state.ctrl_pressed => {
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
                state.window.request_redraw();
                true
            } else {
                false
            }
        }
        KeyCode::KeyD if (state.shift_pressed || state.ctrl_pressed) => {
            if let Some(sel_id) = state.selected_node {
                let cmd = Box::new(DuplicateNodeCommand::new(
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
                state.window.request_redraw();
                true
            } else {
                false
            }
        }
        KeyCode::KeyP => {
            if state.alt_pressed && state.interaction_mode == InteractionMode::Edit {
                // Blender Alt+P: Poke Face
                if let Some(sel_id) = state.selected_node {
                    let target_face = state.selected_faces.iter().copied().next();
                    if let Some(fk) = target_face {
                        let cmd = Box::new(PokeFaceCommand {
                            node_id: sel_id,
                            face_key: fk,
                            offset: state.extrude_distance * 0.5,
                            old_mesh: None,
                        });
                        let _ = state.history.execute(cmd, &mut state.scene);
                    }
                }
            } else if state.interaction_mode == InteractionMode::Edit && !state.selected_faces.is_empty() {
                // Blender P: Separate Selected Faces
                if let Some(sel_id) = state.selected_node {
                    let faces: Vec<FaceKey> = state.selected_faces.iter().copied().collect();
                    let cmd = Box::new(SeparateFacesCommand {
                        source_id: sel_id,
                        faces_to_extract: faces,
                        created_node_id: None,
                        old_source_mesh: None,
                    });
                    if state.history.execute(cmd, &mut state.scene).is_ok() {
                        state.selected_faces.clear();
                    }
                }
            } else if state.shift_pressed {
                // Maya Shift+P: Unparent selected to root
                if let Some(child_id) = state.selected_node {
                    let cmd = Box::new(SetParentCommand {
                        child_id,
                        new_parent: None,
                        old_parent: None,
                    });
                    let _ = state.history.execute(cmd, &mut state.scene);
                }
            } else {
                // Maya P: Parent selected child to parent target
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
            state.window.request_redraw();
            true
        }
        KeyCode::KeyJ => {
            if state.ctrl_pressed {
                // Blender Ctrl+J / Maya Combine: Join Meshes
                if let (Some(target_id), Some(source_id)) = (state.selected_node, state.previous_selected_node) {
                    if target_id != source_id {
                        let cmd = Box::new(JoinMeshesCommand {
                            target_id,
                            source_id,
                            old_target_mesh: None,
                            source_name: String::new(),
                            source_transform: Transform::IDENTITY,
                            source_base_mesh: None,
                        });
                        if state.history.execute(cmd, &mut state.scene).is_ok() {
                            state.previous_selected_node = None;
                        }
                    }
                }
            } else if state.interaction_mode == InteractionMode::Edit {
                if let Some(sel_id) = state.selected_node {
                    if let Some(node) = state.scene.nodes.get(sel_id) {
                        if let NodeData::Mesh { base_mesh, .. } = &node.data {
                            if let Some(fk) = state.selected_faces.iter().copied().next() {
                                let verts = base_mesh.face_vertices(fk);
                                if verts.len() >= 4 {
                                    let cmd = Box::new(ConnectVerticesCommand {
                                        node_id: sel_id,
                                        face_key: fk,
                                        v0: verts[0],
                                        v1: verts[2],
                                        old_mesh: None,
                                    });
                                    let _ = state.history.execute(cmd, &mut state.scene);
                                }
                            }
                        }
                    }
                }
            }
            state.window.request_redraw();
            true
        }
        KeyCode::Delete | KeyCode::KeyX => {
            if state.extrude_modal_active {
                state.extrude_modal_axis_constraint = Some(GizmoAxis::X);
                state.window.request_redraw();
                return true;
            }
            if state.interaction_mode == InteractionMode::Edit && state.sub_element_level == SubElementLevel::Edge && !state.selected_edges.is_empty() {
                if let Some(sel_id) = state.selected_node {
                    if let Some(&ek) = state.selected_edges.iter().next() {
                        let cmd = Box::new(DissolveEdgeCommand {
                            node_id: sel_id,
                            edge_key: ek,
                            old_mesh: None,
                        });
                        if state.history.execute(cmd, &mut state.scene).is_ok() {
                            state.selected_edges.clear();
                        }
                    }
                }
            } else if state.interaction_mode == InteractionMode::CurveEdit {
                if let Some(sel_id) = state.selected_node {
                    if let Some(node) = state.scene.nodes.get_mut(sel_id) {
                        if let NodeData::Curve { spline, cached_render_mesh, .. } = &mut node.data {
                            if spline.points.len() > 2 {
                                if let Some(k_idx) = state.selected_curve_knot {
                                    spline.points.remove(k_idx);
                                    state.selected_curve_knot = Some(k_idx.min(spline.points.len() - 1));
                                    *cached_render_mesh = None;
                                    state.scene.mark_dirty(sel_id);
                                }
                            }
                        }
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
            state.window.request_redraw();
            true
        }
        _ => false,
    }
}

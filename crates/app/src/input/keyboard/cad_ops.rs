//! Specialized keyboard shortcuts for Mesh Normals, UV unwrapping,
//! Decals, Cables, Cutters, and Curvature Combs.

use core_math::Vec3;
use geo_kernel::FaceKey;
use graph_engine::NodeData;
use winit::keyboard::KeyCode;

use crate::commands::*;
use crate::state::AppState;
use crate::types::{InteractionMode, SelectionToolMode};

pub fn handle_mesh_key(state: &mut AppState, key_code: KeyCode) -> bool {
    match key_code {
        KeyCode::KeyU if state.interaction_mode == InteractionMode::Edit => {
            if let Some(sel_id) = state.selected_node {
                let cmd = Box::new(UnwrapSeamsCommand {
                    node_id: sel_id,
                    margin: state.smart_uv_margin,
                    old_mesh: None,
                });
                let _ = state.history.execute(cmd, &mut state.scene);
                state.show_uv_editor = true;
                state.window.request_redraw();
            }
            true
        }
        KeyCode::KeyN if state.alt_pressed && state.interaction_mode == InteractionMode::Edit => {
            if let Some(sel_id) = state.selected_node {
                let target_faces: Vec<FaceKey> = state.selected_faces.iter().copied().collect();
                let cmd = Box::new(FlipNormalsCommand {
                    node_id: sel_id,
                    faces: target_faces,
                    old_mesh: None,
                });
                if let Err(e) = state.history.execute(cmd, &mut state.scene) {
                    eprintln!("Flip normals failed: {}", e);
                }
            }
            state.window.request_redraw();
            true
        }
        KeyCode::KeyN if state.shift_pressed && state.interaction_mode == InteractionMode::Edit => {
            if let Some(sel_id) = state.selected_node {
                let inside = state.ctrl_pressed;
                let cmd = Box::new(RecalculateNormalsCommand {
                    node_id: sel_id,
                    inside,
                    old_mesh: None,
                });
                if let Err(e) = state.history.execute(cmd, &mut state.scene) {
                    eprintln!("Recalculate normals failed: {}", e);
                }
            }
            state.window.request_redraw();
            true
        }
        KeyCode::KeyD if state.alt_pressed => {
            // Project active floating mesh decal onto target scene hull with normal stamping
            if let Some(sel_id) = state.selected_node {
                let mut target_data = None;
                for (nid, node) in &state.scene.nodes {
                    if nid != sel_id {
                        if let NodeData::Mesh { base_mesh, .. } = &node.data {
                            target_data = Some((base_mesh.clone(), node.world_matrix));
                            break;
                        }
                    }
                }
                if let Some((hull_mesh, hull_mat)) = target_data {
                    if let Some(node) = state.scene.nodes.get_mut(sel_id) {
                        if let NodeData::Mesh {
                            base_mesh,
                            cached_render_mesh,
                            ..
                        } = &mut node.data
                        {
                            let decal_mat = node.world_matrix;
                            if base_mesh
                                .project_floating_decal(&hull_mesh, &decal_mat, &hull_mat, 0.0005)
                                .is_ok()
                            {
                                *cached_render_mesh = None;
                                state.scene.mark_dirty(sel_id);
                                state.set_status_message("✨ Floating Mesh Decal Projected & Boundary Normals Stamped (Alt+D)");
                                state.window.request_redraw();
                            }
                        }
                    }
                }
            }
            true
        }
        KeyCode::KeyC if state.ctrl_pressed && state.shift_pressed => {
            let p_a = Vec3::new(-1.0, 1.0, 0.0);
            let p_b = Vec3::new(1.0, 1.0, 0.0);
            let cable_mesh = geo_kernel::CableGenerator::create_cable(p_a, p_b, 0.04, 0.25, 12, 8);
            let name = format!("HydraulicHose_{}", state.scene.nodes.len());
            let node =
                graph_engine::SceneNode::new_mesh(name, core_math::Transform::IDENTITY, cable_mesh);
            let id = state.scene.add_node(node);
            state.selected_node = Some(id);
            state.sync_gpu_meshes();
            state.set_status_message(
                "✨ Dynamic Hydraulic Cable with Fittings Created (Ctrl+Shift+C)",
            );
            state.window.request_redraw();
            true
        }
        KeyCode::KeyC if state.shift_pressed && !state.ctrl_pressed => {
            // Toggle CAD Curvature Combs if in CurveEdit or Curve node selected, else circle select
            let is_curve_selected = state
                .selected_node
                .and_then(|id| state.scene.nodes.get(id))
                .map(|n| matches!(n.data, NodeData::Curve { .. }))
                .unwrap_or(false);
            if state.interaction_mode == InteractionMode::CurveEdit || is_curve_selected {
                state.show_curvature_combs = !state.show_curvature_combs;
                state.set_status_message(if state.show_curvature_combs {
                    "🔍 Curvature Combs: ON (Porcupine analysis needles active)"
                } else {
                    "🔍 Curvature Combs: OFF"
                });
            } else {
                state.selection_tool_mode = SelectionToolMode::CirclePaint;
                state.is_box_selecting = !state.is_box_selecting;
                state.set_status_message(if state.is_box_selecting {
                    "Paint / Circle Select Activated (Drag to paint)"
                } else {
                    "Selection Deactivated"
                });
            }
            state.window.request_redraw();
            true
        }
        KeyCode::KeyC if !state.curve_pen_active => {
            crate::modals::cutter::start(state);
            true
        }
        KeyCode::KeyP
            if state.interaction_mode == InteractionMode::Edit
                && !state.selected_faces.is_empty() =>
        {
            let mouse_pos = (
                state.surface_config.width as f64 * 0.5,
                state.surface_config.height as f64 * 0.5,
            );
            crate::modals::push_pull::start(state, mouse_pos);
            state.set_status_message(
                "Modal Push-Pull: Drag mouse along normal (Up=Boss Union, Down=Cavity Cut)",
            );
            state.window.request_redraw();
            true
        }
        KeyCode::F9 => {
            state.show_last_operator_panel = !state.show_last_operator_panel;
            state.window.request_redraw();
            true
        }
        _ => false,
    }
}

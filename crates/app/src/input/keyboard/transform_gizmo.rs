use core_math::{Quat, Vec3};
use geo_kernel::{EdgeKey, VertexKey};
use render_viewport::GizmoMode;
use winit::keyboard::KeyCode;

use crate::commands::SlideEdgesCommand;
use crate::state::AppState;
use crate::types::InteractionMode;

pub fn handle_transform_gizmo_key(state: &mut AppState, key_code: KeyCode) -> bool {
    match key_code {
        KeyCode::KeyW => {
            if state.alt_pressed {
                crate::commands::workplane::toggle_workplane_align(state);
                return true;
            }
            state.gizmo.mode = GizmoMode::Translate;
            state.window.request_redraw();
            true
        }
        KeyCode::KeyX if state.alt_pressed => {
            crate::commands::workplane::symmetrize_active(state);
            true
        }
        KeyCode::KeyE
            if state.interaction_mode == InteractionMode::Object
                || state.interaction_mode == InteractionMode::Pose =>
        {
            state.gizmo.mode = GizmoMode::Rotate;
            state.window.request_redraw();
            true
        }
        KeyCode::KeyR => {
            if state.alt_pressed && state.interaction_mode == InteractionMode::Pose {
                if let Some(sel_id) = state.selected_node {
                    if let Some(node) = state.scene.nodes.get_mut(sel_id) {
                        node.local_transform.rotation = Quat::IDENTITY;
                        state.scene.mark_dirty(sel_id);
                    }
                }
            } else if !state.ctrl_pressed {
                state.gizmo.mode = GizmoMode::Scale;
            }
            state.window.request_redraw();
            true
        }
        KeyCode::KeyG => {
            if state.alt_pressed && state.interaction_mode == InteractionMode::Pose {
                if let Some(sel_id) = state.selected_node {
                    if let Some(node) = state.scene.nodes.get_mut(sel_id) {
                        node.local_transform.translation = Vec3::ZERO;
                        state.scene.mark_dirty(sel_id);
                    }
                }
            } else if state.interaction_mode == InteractionMode::Edit {
                if let Some(sel_id) = state.selected_node {
                    let target_edges: Vec<EdgeKey> = state.selected_edges.iter().copied().collect();
                    let target_verts: Vec<VertexKey> =
                        state.selected_vertices.iter().copied().collect();
                    if !target_edges.is_empty() || !target_verts.is_empty() {
                        let cmd = Box::new(SlideEdgesCommand {
                            node_id: sel_id,
                            edges: target_edges,
                            vertices: target_verts,
                            factor: state.edge_slide_factor,
                            old_mesh: None,
                        });
                        let _ = state.history.execute(cmd, &mut state.scene);
                        state.window.request_redraw();
                    }
                }
            }
            state.window.request_redraw();
            true
        }
        _ => false,
    }
}

use graph_engine::NodeData;
use winit::dpi::PhysicalPosition;
use winit::event::MouseButton;
use winit::keyboard::KeyCode;

use crate::commands::SetVertexCreaseCommand;
use crate::state::AppState;

pub fn commit(state: &mut AppState) {
    if let Some(sel_id) = state.selected_node {
        let vertices = state.vertex_crease_modal_vertices.clone();
        let weight = state.vertex_crease_modal_weight;
        if let Some(node) = state.scene.nodes.get_mut(sel_id) {
            if let NodeData::Mesh { base_mesh, .. } = &mut node.data {
                for &(vk, orig_w) in &state.vertex_crease_modal_original_weights {
                    base_mesh.set_vertex_crease(vk, orig_w);
                }
            }
        }
        let cmd = Box::new(SetVertexCreaseCommand {
            node_id: sel_id,
            vertices,
            crease_weight: weight,
            old_mesh: None,
        });
        let _ = state.history.execute(cmd, &mut state.scene);
    }
    state.vertex_crease_modal_active = false;
    state.window.request_redraw();
}

pub fn cancel(state: &mut AppState) {
    if let Some(sel_id) = state.selected_node {
        if let Some(node) = state.scene.nodes.get_mut(sel_id) {
            if let NodeData::Mesh { base_mesh, .. } = &mut node.data {
                for &(vk, orig_w) in &state.vertex_crease_modal_original_weights {
                    base_mesh.set_vertex_crease(vk, orig_w);
                }
            }
            state.scene.mark_dirty(sel_id);
        }
    }
    state.vertex_crease_modal_active = false;
    state.window.request_redraw();
}

pub fn handle_mouse_input(state: &mut AppState, button: MouseButton, is_pressed: bool) -> bool {
    if !state.vertex_crease_modal_active {
        return false;
    }
    if button == MouseButton::Left && is_pressed {
        commit(state);
        return true;
    }
    if button == MouseButton::Right && is_pressed {
        cancel(state);
        return true;
    }
    true
}

pub fn handle_cursor_moved(state: &mut AppState, position: PhysicalPosition<f64>) -> bool {
    if !state.vertex_crease_modal_active {
        return false;
    }
    let dx = (position.x - state.vertex_crease_modal_start_mouse.0) as f32;
    let start_w = state.vertex_crease_modal_original_weights.first().map(|&(_, w)| w).unwrap_or(0.0);
    state.vertex_crease_modal_weight = (start_w + dx * 0.005).clamp(0.0, 1.0);
    if let Some(sel_id) = state.selected_node {
        if let Some(node) = state.scene.nodes.get_mut(sel_id) {
            if let NodeData::Mesh { base_mesh, .. } = &mut node.data {
                for &vk in &state.vertex_crease_modal_vertices {
                    base_mesh.set_vertex_crease(vk, state.vertex_crease_modal_weight);
                }
            }
            state.scene.mark_dirty(sel_id);
        }
    }
    state.last_mouse_pos = Some((position.x, position.y));
    state.window.request_redraw();
    true
}

pub fn handle_mouse_wheel(state: &mut AppState, scroll: f32) -> bool {
    if !state.vertex_crease_modal_active {
        return false;
    }
    if scroll > 0.0 {
        state.vertex_crease_modal_weight = (state.vertex_crease_modal_weight + 0.05).min(1.0);
    } else if scroll < 0.0 {
        state.vertex_crease_modal_weight = (state.vertex_crease_modal_weight - 0.05).max(0.0);
    }
    if let Some(sel_id) = state.selected_node {
        if let Some(node) = state.scene.nodes.get_mut(sel_id) {
            if let NodeData::Mesh { base_mesh, .. } = &mut node.data {
                for &vk in &state.vertex_crease_modal_vertices {
                    base_mesh.set_vertex_crease(vk, state.vertex_crease_modal_weight);
                }
            }
            state.scene.mark_dirty(sel_id);
        }
    }
    state.window.request_redraw();
    true
}

pub fn handle_keyboard_input(state: &mut AppState, key_code: KeyCode) -> bool {
    if !state.vertex_crease_modal_active {
        return false;
    }
    match key_code {
        KeyCode::Enter | KeyCode::NumpadEnter => {
            commit(state);
            true
        }
        KeyCode::Escape => {
            cancel(state);
            true
        }
        _ => false,
    }
}

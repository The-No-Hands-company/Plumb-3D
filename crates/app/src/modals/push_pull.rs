use winit::dpi::PhysicalPosition;
use winit::event::MouseButton;
use winit::keyboard::KeyCode;

use crate::commands::PushPullCommand;
use crate::state::AppState;

pub fn start(state: &mut AppState, mouse_pos: (f64, f64)) {
    if state.selected_faces.is_empty() {
        return;
    }
    let face = *state.selected_faces.iter().next().unwrap();
    state.push_pull_modal_active = true;
    state.push_pull_modal_face = Some(face);
    state.push_pull_modal_start_mouse = mouse_pos;
    state.push_pull_modal_distance = 0.0;
    state.modal_numeric_buffer.clear();
    state.window.request_redraw();
}

pub fn commit(state: &mut AppState) {
    if let (Some(sel_id), Some(face_key)) = (state.selected_node, state.push_pull_modal_face) {
        let dist = state.push_pull_modal_distance;
        if dist.abs() > 1e-4 {
            let cmd = Box::new(PushPullCommand {
                node_id: sel_id,
                face_key,
                distance: dist,
                old_mesh: None,
                created_cap: None,
                created_walls: Vec::new(),
            });

            if state.history.execute(cmd, &mut state.scene).is_ok() {
                if let Some(node) = state.scene.nodes.get(sel_id) {
                    if let graph_engine::NodeData::Mesh { base_mesh, .. } = &node.data {
                        state.selected_faces.clear();
                        if let Some(last_fk) = base_mesh.faces.keys().last() {
                            state.selected_faces.insert(last_fk);
                        }
                    }
                }
                state.sync_gpu_meshes();
            }
        }
    }
    state.modal_numeric_buffer.clear();
    state.push_pull_modal_active = false;
    state.push_pull_modal_face = None;
    state.window.request_redraw();
}

pub fn cancel(state: &mut AppState) {
    state.modal_numeric_buffer.clear();
    state.push_pull_modal_active = false;
    state.push_pull_modal_face = None;
    state.window.request_redraw();
}

pub fn handle_mouse_input(state: &mut AppState, button: MouseButton, is_pressed: bool) -> bool {
    if !state.push_pull_modal_active {
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
    if !state.push_pull_modal_active {
        return false;
    }
    let dy = (state.push_pull_modal_start_mouse.1 - position.y) as f32;
    state.push_pull_modal_distance = dy * 0.01;
    state.window.request_redraw();
    true
}

pub fn handle_keyboard_input(state: &mut AppState, key: KeyCode, is_pressed: bool) -> bool {
    if !state.push_pull_modal_active || !is_pressed {
        return false;
    }

    match key {
        KeyCode::Enter | KeyCode::NumpadEnter => {
            if !state.modal_numeric_buffer.is_empty() {
                if let Ok(v) = state.modal_numeric_buffer.parse::<f32>() {
                    state.push_pull_modal_distance = v;
                }
            }
            commit(state);
            true
        }
        KeyCode::Escape => {
            cancel(state);
            true
        }
        KeyCode::Backspace => {
            state.modal_numeric_buffer.pop();
            if let Ok(v) = state.modal_numeric_buffer.parse::<f32>() {
                state.push_pull_modal_distance = v;
            }
            state.window.request_redraw();
            true
        }
        KeyCode::Digit0 | KeyCode::Numpad0 => append_char(state, '0'),
        KeyCode::Digit1 | KeyCode::Numpad1 => append_char(state, '1'),
        KeyCode::Digit2 | KeyCode::Numpad2 => append_char(state, '2'),
        KeyCode::Digit3 | KeyCode::Numpad3 => append_char(state, '3'),
        KeyCode::Digit4 | KeyCode::Numpad4 => append_char(state, '4'),
        KeyCode::Digit5 | KeyCode::Numpad5 => append_char(state, '5'),
        KeyCode::Digit6 | KeyCode::Numpad6 => append_char(state, '6'),
        KeyCode::Digit7 | KeyCode::Numpad7 => append_char(state, '7'),
        KeyCode::Digit8 | KeyCode::Numpad8 => append_char(state, '8'),
        KeyCode::Digit9 | KeyCode::Numpad9 => append_char(state, '9'),
        KeyCode::Period | KeyCode::NumpadDecimal => append_char(state, '.'),
        KeyCode::Minus | KeyCode::NumpadSubtract => append_char(state, '-'),
        _ => false,
    }
}

fn append_char(state: &mut AppState, c: char) -> bool {
    state.modal_numeric_buffer.push(c);
    if let Ok(v) = state.modal_numeric_buffer.parse::<f32>() {
        state.push_pull_modal_distance = v;
    }
    state.window.request_redraw();
    true
}

use winit::dpi::PhysicalPosition;
use winit::event::MouseButton;
use winit::keyboard::KeyCode;

use crate::commands::{BevelCommand, BevelEdgeCommand};
use crate::state::AppState;
use crate::types::LastOperatorInfo;

pub fn commit(state: &mut AppState) {
    if let Some(sel_id) = state.selected_node {
        let edges = state.bevel_modal_edges.clone();
        let faces = state.bevel_modal_faces.clone();
        let width = state.bevel_modal_width;
        let segments = state.bevel_modal_segments;
        if !edges.is_empty() {
            let cmd = Box::new(BevelEdgeCommand {
                node_id: sel_id,
                edges: edges.clone(),
                width,
                segments,
                old_mesh: None,
            });
            let _ = state.history.execute(cmd, &mut state.scene);
            state.last_operator = Some(LastOperatorInfo::Bevel {
                edges,
                width,
                segments,
            });
        } else if !faces.is_empty() {
            let cmd = Box::new(BevelCommand {
                node_id: sel_id,
                faces,
                offset: width * 0.5,
                inset: width * 0.5,
                old_mesh: None,
            });
            let _ = state.history.execute(cmd, &mut state.scene);
        }
    }
    state.modal_numeric_buffer.clear();
    state.bevel_modal_active = false;
    state.window.request_redraw();
}

pub fn cancel(state: &mut AppState) {
    state.modal_numeric_buffer.clear();
    state.bevel_modal_active = false;
    state.window.request_redraw();
}

pub fn handle_mouse_input(state: &mut AppState, button: MouseButton, is_pressed: bool) -> bool {
    if !state.bevel_modal_active {
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
    if !state.bevel_modal_active {
        return false;
    }
    let dx = (position.x - state.bevel_modal_start_mouse.0) as f32;
    state.bevel_modal_width = (0.15 + dx * 0.005).max(0.005);
    state.modal_numeric_buffer.clear();
    state.last_mouse_pos = Some((position.x, position.y));
    state.window.request_redraw();
    true
}

pub fn handle_mouse_wheel(state: &mut AppState, scroll: f32) -> bool {
    if !state.bevel_modal_active {
        return false;
    }
    if scroll > 0.0 {
        state.bevel_modal_segments = (state.bevel_modal_segments + 1).min(8);
    } else if scroll < 0.0 {
        state.bevel_modal_segments = state.bevel_modal_segments.saturating_sub(1).max(1);
    }
    state.window.request_redraw();
    true
}

fn key_to_char(key: KeyCode) -> Option<char> {
    match key {
        KeyCode::Digit0 | KeyCode::Numpad0 => Some('0'),
        KeyCode::Digit1 | KeyCode::Numpad1 => Some('1'),
        KeyCode::Digit2 | KeyCode::Numpad2 => Some('2'),
        KeyCode::Digit3 | KeyCode::Numpad3 => Some('3'),
        KeyCode::Digit4 | KeyCode::Numpad4 => Some('4'),
        KeyCode::Digit5 | KeyCode::Numpad5 => Some('5'),
        KeyCode::Digit6 | KeyCode::Numpad6 => Some('6'),
        KeyCode::Digit7 | KeyCode::Numpad7 => Some('7'),
        KeyCode::Digit8 | KeyCode::Numpad8 => Some('8'),
        KeyCode::Digit9 | KeyCode::Numpad9 => Some('9'),
        KeyCode::Period | KeyCode::NumpadDecimal => Some('.'),
        _ => None,
    }
}

pub fn handle_keyboard_input(state: &mut AppState, key_code: KeyCode) -> bool {
    if !state.bevel_modal_active {
        return false;
    }
    if let Some(ch) = key_to_char(key_code) {
        state.modal_numeric_buffer.push(ch);
        if let Ok(val) = state.modal_numeric_buffer.parse::<f32>() {
            state.bevel_modal_width = val.max(0.001);
        }
        state.window.request_redraw();
        return true;
    }

    if key_code == KeyCode::Backspace {
        state.modal_numeric_buffer.pop();
        if let Ok(val) = state.modal_numeric_buffer.parse::<f32>() {
            state.bevel_modal_width = val.max(0.001);
        }
        state.window.request_redraw();
        return true;
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

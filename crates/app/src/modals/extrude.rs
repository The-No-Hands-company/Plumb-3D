use render_viewport::GizmoAxis;
use winit::dpi::PhysicalPosition;
use winit::event::MouseButton;
use winit::keyboard::KeyCode;

use crate::commands::ExtrudeCommand;
use crate::state::AppState;
use crate::types::LastOperatorInfo;

pub fn commit(state: &mut AppState) {
    if let Some(sel_id) = state.selected_node {
        let faces = state.extrude_modal_faces.clone();
        let dist = state.extrude_modal_distance;
        let axis = state.extrude_modal_axis_constraint;
        if !faces.is_empty() && dist.abs() > 1e-4 {
            let axis_vec = match axis {
                Some(GizmoAxis::X) => Some(core_math::Vec3::X),
                Some(GizmoAxis::Y) => Some(core_math::Vec3::Y),
                Some(GizmoAxis::Z) => Some(core_math::Vec3::Z),
                _ => None,
            };
            let cmd = Box::new(ExtrudeCommand {
                node_id: sel_id,
                faces: faces.clone(),
                distance: dist,
                individual: false,
                axis: axis_vec,
                old_mesh: None,
                created_faces: Vec::new(),
            });
            if state.history.execute(cmd, &mut state.scene).is_ok() {
                if let Some(node) = state.scene.nodes.get(sel_id) {
                    if let graph_engine::NodeData::Mesh { base_mesh, .. } = &node.data {
                        state.selected_faces.clear();
                        let all_fks: Vec<_> = base_mesh.faces.keys().collect();
                        for &fk in all_fks.iter().rev().take(faces.len()) {
                            state.selected_faces.insert(fk);
                        }
                    }
                }
            }
            state.last_operator = Some(LastOperatorInfo::Extrude {
                faces,
                distance: dist,
                axis,
            });
        }
    }
    state.modal_numeric_buffer.clear();
    state.extrude_modal_active = false;
    state.window.request_redraw();
}

pub fn cancel(state: &mut AppState) {
    state.modal_numeric_buffer.clear();
    state.extrude_modal_active = false;
    state.window.request_redraw();
}

pub fn handle_mouse_input(state: &mut AppState, button: MouseButton, is_pressed: bool) -> bool {
    if !state.extrude_modal_active {
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
    if !state.extrude_modal_active {
        return false;
    }
    let dy = (state.extrude_modal_start_mouse.1 - position.y) as f32;
    state.extrude_modal_distance = dy * 0.01;
    state.modal_numeric_buffer.clear();
    state.set_status_message(format!(
        "⚒ Extruding: {:+.3}m (Type numeric digits + Enter to set exact distance)",
        state.extrude_modal_distance
    ));
    state.last_mouse_pos = Some((position.x, position.y));
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
        KeyCode::Minus | KeyCode::NumpadSubtract => Some('-'),
        _ => None,
    }
}

pub fn handle_keyboard_input(state: &mut AppState, key_code: KeyCode) -> bool {
    if !state.extrude_modal_active {
        return false;
    }
    if let Some(ch) = key_to_char(key_code) {
        state.modal_numeric_buffer.push(ch);
        if let Ok(val) = state.modal_numeric_buffer.parse::<f32>() {
            state.extrude_modal_distance = val;
        }
        state.set_status_message(format!(
            "⚒ Extrude Distance: {} m (Press Enter to confirm)",
            state.modal_numeric_buffer
        ));
        state.window.request_redraw();
        return true;
    }

    if key_code == KeyCode::Backspace {
        state.modal_numeric_buffer.pop();
        if let Ok(val) = state.modal_numeric_buffer.parse::<f32>() {
            state.extrude_modal_distance = val;
        }
        state.set_status_message(format!(
            "⚒ Extrude Distance: {} m (Press Enter to confirm)",
            state.modal_numeric_buffer
        ));
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
        KeyCode::KeyX => {
            state.extrude_modal_axis_constraint = Some(GizmoAxis::X);
            state.window.request_redraw();
            true
        }
        KeyCode::KeyY => {
            state.extrude_modal_axis_constraint = Some(GizmoAxis::Y);
            state.window.request_redraw();
            true
        }
        KeyCode::KeyZ => {
            state.extrude_modal_axis_constraint = Some(GizmoAxis::Z);
            state.window.request_redraw();
            true
        }
        KeyCode::KeyC => {
            state.extrude_modal_axis_constraint = None;
            state.window.request_redraw();
            true
        }
        _ => false,
    }
}

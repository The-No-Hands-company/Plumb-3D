use winit::dpi::PhysicalPosition;
use winit::event::MouseButton;
use winit::keyboard::KeyCode;

use crate::commands::InsetCommand;
use crate::state::AppState;
use crate::types::LastOperatorInfo;

pub fn commit(state: &mut AppState) {
    if let Some(sel_id) = state.selected_node {
        let faces = state.inset_modal_faces.clone();
        let factor = state.inset_modal_factor;
        let individual = state.inset_modal_individual;
        let thickness = state.inset_modal_thickness;
        let depth = state.inset_modal_depth;
        let initial_radius = state.inset_modal_initial_radius;
        if !faces.is_empty() {
            let cmd = Box::new(InsetCommand {
                node_id: sel_id,
                faces: faces.clone(),
                factor,
                individual,
                old_mesh: None,
                created_faces: Vec::new(),
            });
            let _ = state.history.execute(cmd, &mut state.scene);
            state.last_operator = Some(LastOperatorInfo::Inset {
                faces: faces.clone(),
                thickness,
                factor,
                individual,
                initial_radius,
            });

            // If Step-Bore depth is set (Ctrl held during Inset), push/extrude the inner face
            if depth.abs() > 1e-4 {
                if let Some(node) = state.scene.nodes.get(sel_id) {
                    if let graph_engine::NodeData::Mesh { base_mesh, .. } = &node.data {
                        if let Some(last_fk) = base_mesh.faces.keys().last() {
                            let extrude_cmd = Box::new(crate::commands::ExtrudeCommand::new(
                                sel_id,
                                vec![last_fk],
                                -depth,
                            ));
                            let _ = state.history.execute(extrude_cmd, &mut state.scene);
                        }
                    }
                }
            }

            if let Some(node) = state.scene.nodes.get(sel_id) {
                if let graph_engine::NodeData::Mesh { base_mesh, .. } = &node.data {
                    state.selected_faces.clear();
                    if let Some(last_fk) = base_mesh.faces.keys().last() {
                        state.selected_faces.insert(last_fk);
                    }
                }
            }
        }
    }
    state.modal_numeric_buffer.clear();
    state.inset_modal_active = false;
    state.inset_modal_depth = 0.0;
    state.window.request_redraw();
}

pub fn cancel(state: &mut AppState) {
    state.modal_numeric_buffer.clear();
    state.inset_modal_active = false;
    state.inset_modal_depth = 0.0;
    state.window.request_redraw();
}

pub fn handle_mouse_input(state: &mut AppState, button: MouseButton, is_pressed: bool) -> bool {
    if !state.inset_modal_active {
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
    if !state.inset_modal_active {
        return false;
    }
    if state.ctrl_pressed {
        // Holding Ctrl adjusts Step-Bore push depth into the mesh
        let dy = (position.y - state.inset_modal_start_mouse.1) as f32;
        state.inset_modal_depth = dy * 0.005;
    } else {
        let dx = (state.inset_modal_start_mouse.0 - position.x) as f32;
        state.inset_modal_factor = (0.8 + dx * 0.003).clamp(0.01, 0.99);
        state.inset_modal_thickness = (1.0 - state.inset_modal_factor) * state.inset_modal_initial_radius;
    }
    state.modal_numeric_buffer.clear();
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
    if !state.inset_modal_active {
        return false;
    }
    if let Some(ch) = key_to_char(key_code) {
        state.modal_numeric_buffer.push(ch);
        if let Ok(val) = state.modal_numeric_buffer.parse::<f32>() {
            let r = state.inset_modal_initial_radius.max(0.001);
            state.inset_modal_thickness = val.max(0.0);
            state.inset_modal_factor = (1.0 - state.inset_modal_thickness / r).clamp(0.01, 0.99);
        }
        state.window.request_redraw();
        return true;
    }

    if key_code == KeyCode::Backspace {
        state.modal_numeric_buffer.pop();
        if let Ok(val) = state.modal_numeric_buffer.parse::<f32>() {
            let r = state.inset_modal_initial_radius.max(0.001);
            state.inset_modal_thickness = val.max(0.0);
            state.inset_modal_factor = (1.0 - state.inset_modal_thickness / r).clamp(0.01, 0.99);
        }
        state.window.request_redraw();
        return true;
    }

    match key_code {
        KeyCode::KeyI => {
            state.inset_modal_individual = !state.inset_modal_individual;
            state.window.request_redraw();
            true
        }
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

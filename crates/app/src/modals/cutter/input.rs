use geo_kernel::MirrorAxis;
use winit::dpi::PhysicalPosition;
use winit::event::MouseButton;
use winit::keyboard::KeyCode;

use super::shapes::CutterShape;
use super::{cancel, commit, CutterStage};
use crate::state::AppState;

pub const RADIAL_PRESETS: [usize; 9] = [1, 2, 3, 4, 6, 8, 12, 16, 24];

pub fn handle_mouse_input(state: &mut AppState, button: MouseButton, is_pressed: bool) -> bool {
    let cutter = match &mut state.cutter_modal {
        Some(c) => c,
        None => return false,
    };

    if button == MouseButton::Right && is_pressed {
        cancel(state);
        return true;
    }

    if button == MouseButton::Left {
        if is_pressed {
            if cutter.stage == CutterStage::ExtrudeDepth {
                commit(state);
                return true;
            }

            if cutter.shape == CutterShape::Ngon {
                // In Ngon mode, each click appends a vertex
                if cutter.polygon_points.is_empty() {
                    cutter.polygon_points.push(cutter.current_uv);
                    cutter.start_uv = cutter.current_uv;
                    cutter.start_pos = cutter.current_pos;
                } else {
                    let first = cutter.polygon_points[0];
                    let dist_to_first = (cutter.current_uv - first).length();
                    if cutter.polygon_points.len() >= 3 && dist_to_first < 0.05 {
                        // Close loop and advance to depth
                        cutter.stage = CutterStage::ExtrudeDepth;
                        cutter.depth = 0.2;
                        state.set_status_message(
                            "Polygon closed! Move mouse to set cut depth | LMB to cut | Esc to cancel",
                        );
                    } else {
                        cutter.polygon_points.push(cutter.current_uv);
                    }
                }
            } else {
                // Box or Circle mode: start drag footprint
                cutter.start_uv = cutter.current_uv;
                cutter.start_pos = cutter.current_pos;
            }
        } else {
            // Left mouse released after drawing footprint -> advance to ExtrudeDepth
            if cutter.stage == CutterStage::DrawFootprint && cutter.shape != CutterShape::Ngon {
                let dist = (cutter.current_uv - cutter.start_uv).length();
                if dist > 0.01 {
                    cutter.stage = CutterStage::ExtrudeDepth;
                    cutter.depth = 0.2;
                    state.set_status_message(
                        "Move mouse to set cut depth | LMB to cut | Esc to cancel",
                    );
                }
            }
        }
        state.window.request_redraw();
        return true;
    }

    false
}

pub fn handle_cursor_moved(state: &mut AppState, position: PhysicalPosition<f64>) -> bool {
    let ray = state.get_screen_ray(position.x, position.y);
    let cutter = match &mut state.cutter_modal {
        Some(c) => c,
        None => return false,
    };

    match cutter.stage {
        CutterStage::DrawFootprint => {
            if let Some(uv) = cutter.workplane.intersect_ray_uv(&ray) {
                cutter.current_uv = uv;
                cutter.current_pos = cutter.workplane.uv_to_world(uv);
                cutter.drag_start_mouse_y = position.y;
                state.window.request_redraw();
                return true;
            }
        }
        CutterStage::ExtrudeDepth => {
            let delta = (position.y - cutter.drag_start_mouse_y) as f32 * 0.005;
            cutter.depth = (0.2 + delta).max(0.01);
            state.window.request_redraw();
            return true;
        }
    }

    false
}

pub fn handle_mouse_wheel(state: &mut AppState, scroll: f32) -> bool {
    let cutter = match &mut state.cutter_modal {
        Some(c) => c,
        None => return false,
    };

    if scroll.abs() < 0.05 {
        return false;
    }

    let curr_idx = RADIAL_PRESETS
        .iter()
        .position(|&n| n >= cutter.radial_count)
        .unwrap_or(0);
    if scroll > 0.0 {
        if curr_idx + 1 < RADIAL_PRESETS.len() {
            cutter.radial_count = RADIAL_PRESETS[curr_idx + 1];
        }
    } else if curr_idx > 0 {
        cutter.radial_count = RADIAL_PRESETS[curr_idx - 1];
    }

    state.window.request_redraw();
    true
}

pub fn handle_keyboard_input(state: &mut AppState, key_code: KeyCode) -> bool {
    if state.cutter_modal.is_none() {
        return false;
    }

    match key_code {
        KeyCode::Escape => {
            cancel(state);
            true
        }
        KeyCode::Enter | KeyCode::NumpadEnter => {
            if let Some(c) = &mut state.cutter_modal {
                if c.stage == CutterStage::DrawFootprint && c.shape == CutterShape::Ngon && c.polygon_points.len() >= 3 {
                    c.stage = CutterStage::ExtrudeDepth;
                    c.depth = 0.2;
                    state.set_status_message("Move mouse to set cut depth | LMB to cut | Esc to cancel");
                    state.window.request_redraw();
                    return true;
                }
            }
            commit(state);
            true
        }
        KeyCode::Backspace => {
            if let Some(cutter) = &mut state.cutter_modal {
                if cutter.shape == CutterShape::Ngon && !cutter.polygon_points.is_empty() {
                    cutter.polygon_points.pop();
                    state.window.request_redraw();
                    return true;
                }
            }
            false
        }
        KeyCode::Tab => {
            if let Some(cutter) = &mut state.cutter_modal {
                cutter.shape = match cutter.shape {
                    CutterShape::Box => CutterShape::Circle,
                    CutterShape::Circle => CutterShape::Ngon,
                    CutterShape::Ngon => CutterShape::Box,
                };
                cutter.polygon_points.clear();
                state.window.request_redraw();
            }
            true
        }
        KeyCode::KeyG => {
            if let Some(cutter) = &mut state.cutter_modal {
                cutter.is_ghost = !cutter.is_ghost;
                state.window.request_redraw();
            }
            true
        }
        KeyCode::KeyM => {
            if let Some(cutter) = &mut state.cutter_modal {
                cutter.symmetry_mirror = match cutter.symmetry_mirror {
                    None => Some(MirrorAxis::X),
                    Some(MirrorAxis::X) => Some(MirrorAxis::Y),
                    Some(MirrorAxis::Y) => Some(MirrorAxis::Z),
                    Some(MirrorAxis::Z) => None,
                };
                let sym_str = match cutter.symmetry_mirror {
                    None => "OFF",
                    Some(MirrorAxis::X) => "Mirror X",
                    Some(MirrorAxis::Y) => "Mirror Y",
                    Some(MirrorAxis::Z) => "Mirror Z",
                };
                state.set_status_message(format!("Symmetry Mirror: {}", sym_str));
                state.window.request_redraw();
            }
            true
        }
        KeyCode::KeyR | KeyCode::BracketRight => {
            if let Some(cutter) = &mut state.cutter_modal {
                let curr_idx = RADIAL_PRESETS
                    .iter()
                    .position(|&n| n >= cutter.radial_count)
                    .unwrap_or(0);
                if curr_idx + 1 < RADIAL_PRESETS.len() {
                    cutter.radial_count = RADIAL_PRESETS[curr_idx + 1];
                }
                state.window.request_redraw();
            }
            true
        }
        KeyCode::BracketLeft => {
            if let Some(cutter) = &mut state.cutter_modal {
                let curr_idx = RADIAL_PRESETS
                    .iter()
                    .position(|&n| n >= cutter.radial_count)
                    .unwrap_or(0);
                if curr_idx > 0 {
                    cutter.radial_count = RADIAL_PRESETS[curr_idx - 1];
                }
                state.window.request_redraw();
            }
            true
        }
        _ => false,
    }
}

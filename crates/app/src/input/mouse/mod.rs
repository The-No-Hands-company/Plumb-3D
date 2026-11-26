pub mod buttons;
pub mod curve_pen;
pub mod gizmo_drag;
pub mod navigation;
pub mod painting_drag;

pub use buttons::*;
pub use gizmo_drag::*;
pub use navigation::*;
pub use painting_drag::*;

use crate::state::AppState;
use crate::types::*;
use winit::dpi::PhysicalPosition;
use winit::event::MouseButton;

pub fn handle_mouse_input(state: &mut AppState, button: MouseButton, is_pressed: bool) {
    if crate::modals::handle_mouse_input(state, button, is_pressed) {
        return;
    }
    match button {
        MouseButton::Left => {
            state.show_context_menu = false;
            if is_pressed {
                handle_mouse_left_down(state);
            } else {
                handle_mouse_left_up(state);
            }
            state.window.request_redraw();
        }
        MouseButton::Middle => {
            state.show_context_menu = false;
            state.is_panning = is_pressed;
        }
        MouseButton::Right
            if is_pressed => {
                if let Some((x, y)) = state.last_mouse_pos {
                    if state.interaction_mode == InteractionMode::Object || state.selected_node.is_none() {
                        state.pick_at_screen_pos(x, y);
                    }
                    state.context_menu_pos = Some((x as f32, y as f32));
                    state.show_context_menu = true;
                    state.window.request_redraw();
                }
            }
        _ => {}
    }
}

pub fn handle_cursor_moved(state: &mut AppState, position: PhysicalPosition<f64>) {
    if crate::modals::handle_cursor_moved(state, position) {
        return;
    }

    if handle_painting_cursor_moved(state, position) {
        return;
    }

    if state.curve_pen_active {
        if state.curve_pen_drag_origin.is_some() {
            state.handle_curve_pen_drag(position.x, position.y);
        } else {
            state.window.request_redraw();
        }
    }

    if let Some((last_x, last_y)) = state.last_mouse_pos {
        let dx = (position.x - last_x) as f32;
        let dy = (position.y - last_y) as f32;
        let move_dist = (dx * dx + dy * dy).sqrt();

        if state.box_select_start.is_some() {
            state.box_select_current = Some((position.x as f32, position.y as f32));
            state.window.request_redraw();
        } else if state.is_box_selecting && state.selection_tool_mode == SelectionToolMode::Lasso && !state.lasso_points.is_empty() {
            let (px, py) = (position.x as f32, position.y as f32);
            let last = state.lasso_points.last().copied().unwrap_or((px, py));
            if (px - last.0).abs() > 3.0 || (py - last.1).abs() > 3.0 {
                state.lasso_points.push((px, py));
                state.window.request_redraw();
            }
        } else if state.is_box_selecting && state.selection_tool_mode == SelectionToolMode::CirclePaint && state.last_mouse_pos.is_some() {
            let (px, py) = (position.x as f32, position.y as f32);
            state.select_in_circle((px, py), state.circle_select_radius);
            state.window.request_redraw();
        } else if let Some(axis) = state.active_gizmo_axis {
            handle_gizmo_drag(state, axis, dx, dy, move_dist);
        } else if state.is_orbiting {
            handle_orbit(state, dx, dy, move_dist);
        } else if state.is_panning {
            handle_pan(state, dx, dy);
        } else {
            let ray = state.get_screen_ray(position.x, position.y);
            if let Some(gizmo_origin) = state.get_gizmo_world_position() {
                let gizmo_orient = state.get_gizmo_orientation();
                let dist = (state.camera.eye_position() - gizmo_origin).length().max(0.1);
                let scale = (dist * 0.16).clamp(0.4, 30.0);
                let hovered = state.gizmo.hit_test_scaled(&ray, gizmo_origin, gizmo_orient, scale);
                if hovered != state.hovered_gizmo_axis {
                    state.hovered_gizmo_axis = hovered;
                    state.window.request_redraw();
                }
            }
        }
    }
    state.last_mouse_pos = Some((position.x, position.y));
}

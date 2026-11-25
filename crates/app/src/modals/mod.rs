pub mod bevel;
pub mod cutter;
pub mod edge_crease;
pub mod extrude;
pub mod inset;
pub mod knife;
pub mod loop_cut;
pub mod vertex_crease;

use winit::dpi::PhysicalPosition;
use winit::event::MouseButton;
use winit::keyboard::KeyCode;

use crate::state::AppState;

pub fn is_any_modal_active(state: &AppState) -> bool {
    state.extrude_modal_active
        || state.bevel_modal_active
        || state.inset_modal_active
        || state.edge_crease_modal_active
        || state.vertex_crease_modal_active
        || state.loop_cut_active
        || state.knife_active
        || state.cutter_modal.is_some()
}

pub fn handle_mouse_input(state: &mut AppState, button: MouseButton, is_pressed: bool) -> bool {
    if state.cutter_modal.is_some() {
        return cutter::handle_mouse_input(state, button, is_pressed);
    }
    if state.extrude_modal_active {
        return extrude::handle_mouse_input(state, button, is_pressed);
    }
    if state.bevel_modal_active {
        return bevel::handle_mouse_input(state, button, is_pressed);
    }
    if state.inset_modal_active {
        return inset::handle_mouse_input(state, button, is_pressed);
    }
    if state.edge_crease_modal_active {
        return edge_crease::handle_mouse_input(state, button, is_pressed);
    }
    if state.vertex_crease_modal_active {
        return vertex_crease::handle_mouse_input(state, button, is_pressed);
    }
    if state.loop_cut_active {
        return loop_cut::handle_mouse_input(state, button, is_pressed);
    }
    if state.knife_active {
        return knife::handle_mouse_input(state, button, is_pressed);
    }
    false
}

pub fn handle_cursor_moved(state: &mut AppState, position: PhysicalPosition<f64>) -> bool {
    if state.cutter_modal.is_some() {
        return cutter::handle_cursor_moved(state, position);
    }
    if state.extrude_modal_active {
        return extrude::handle_cursor_moved(state, position);
    }
    if state.bevel_modal_active {
        return bevel::handle_cursor_moved(state, position);
    }
    if state.inset_modal_active {
        return inset::handle_cursor_moved(state, position);
    }
    if state.edge_crease_modal_active {
        return edge_crease::handle_cursor_moved(state, position);
    }
    if state.vertex_crease_modal_active {
        return vertex_crease::handle_cursor_moved(state, position);
    }
    if state.loop_cut_active {
        return loop_cut::handle_cursor_moved(state, position);
    }
    if state.knife_active {
        return knife::handle_cursor_moved(state, position);
    }
    false
}

pub fn handle_mouse_wheel(state: &mut AppState, scroll: f32) -> bool {
    if state.bevel_modal_active {
        return bevel::handle_mouse_wheel(state, scroll);
    }
    if state.edge_crease_modal_active {
        return edge_crease::handle_mouse_wheel(state, scroll);
    }
    if state.vertex_crease_modal_active {
        return vertex_crease::handle_mouse_wheel(state, scroll);
    }
    if state.loop_cut_active {
        return loop_cut::handle_mouse_wheel(state, scroll);
    }
    false
}

pub fn handle_keyboard_input(state: &mut AppState, key_code: KeyCode) -> bool {
    if state.cutter_modal.is_some() {
        return cutter::handle_keyboard_input(state, key_code);
    }
    if state.extrude_modal_active {
        return extrude::handle_keyboard_input(state, key_code);
    }
    if state.bevel_modal_active {
        return bevel::handle_keyboard_input(state, key_code);
    }
    if state.inset_modal_active {
        return inset::handle_keyboard_input(state, key_code);
    }
    if state.edge_crease_modal_active {
        return edge_crease::handle_keyboard_input(state, key_code);
    }
    if state.vertex_crease_modal_active {
        return vertex_crease::handle_keyboard_input(state, key_code);
    }
    if state.loop_cut_active {
        return loop_cut::handle_keyboard_input(state, key_code);
    }
    if state.knife_active {
        return knife::handle_keyboard_input(state, key_code);
    }
    false
}

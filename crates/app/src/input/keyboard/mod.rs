pub mod animation;
pub mod modeling_ops;
pub mod scene_history;
pub mod selection_mode;
pub mod transform_gizmo;
pub mod view_camera;

use winit::keyboard::KeyCode;

use crate::state::AppState;

pub fn handle_keyboard_input(state: &mut AppState, key_code: KeyCode) {
    if crate::modals::handle_keyboard_input(state, key_code) {
        return;
    }

    if view_camera::handle_view_camera_key(state, key_code) {
        return;
    }

    if selection_mode::handle_selection_mode_key(state, key_code) {
        return;
    }

    if transform_gizmo::handle_transform_gizmo_key(state, key_code) {
        return;
    }

    if scene_history::handle_scene_history_key(state, key_code) {
        return;
    }

    if modeling_ops::handle_modeling_key(state, key_code) {
        return;
    }

    if animation::handle_animation_key(state, key_code) {
        return;
    }
}

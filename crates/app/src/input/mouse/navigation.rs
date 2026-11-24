use crate::state::AppState;
use winit::event::MouseScrollDelta;

fn sync_active_camera_if_locked(state: &mut AppState) {
    if let Some(cam_id) = state.active_camera_node {
        if state.lock_camera_to_view {
            let eye = state.camera.eye_position();
            let target = state.camera.target;
            let forward = (target - eye).normalize_or_zero();
            let right = forward.cross(core_math::Vec3::Y).normalize_or_zero();
            let up = right.cross(forward).normalize_or_zero();
            let rot_mat = core_math::Mat3::from_cols(right, up, -forward);
            let rot = core_math::Quat::from_mat3(&rot_mat);

            if let Some(cam_node) = state.scene.nodes.get_mut(cam_id) {
                cam_node.local_transform.translation = eye;
                cam_node.local_transform.rotation = rot;
                state.scene.mark_dirty(cam_id);
            }
        } else {
            // When lock camera to view is disabled, moving camera pops out to free orbit view
            state.active_camera_node = None;
        }
    }
}

pub fn handle_mouse_wheel(state: &mut AppState, delta: MouseScrollDelta) {
    let scroll = match delta {
        MouseScrollDelta::LineDelta(_, y) => y,
        MouseScrollDelta::PixelDelta(pos) => pos.y as f32 * 0.05,
    };
    if crate::modals::handle_mouse_wheel(state, scroll) {
        return;
    }
    state.camera.zoom(scroll);
    sync_active_camera_if_locked(state);
    state.window.request_redraw();
}

pub fn handle_orbit(state: &mut AppState, dx: f32, dy: f32, move_dist: f32) {
    state.mouse_drag_distance += move_dist;
    state.camera.orbit(dx, dy);
    sync_active_camera_if_locked(state);
    state.window.request_redraw();
}

pub fn handle_pan(state: &mut AppState, dx: f32, dy: f32) {
    state.camera.pan(dx, dy);
    sync_active_camera_if_locked(state);
    state.window.request_redraw();
}

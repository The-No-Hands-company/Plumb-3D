use render_viewport::CameraProjection;
use winit::keyboard::KeyCode;

use crate::state::AppState;
use crate::types::{InteractionMode, TransformOrientation};

pub fn handle_view_camera_key(state: &mut AppState, key_code: KeyCode) -> bool {
    match key_code {
        KeyCode::Comma if state.ctrl_pressed => {
            state.preferences.show_dialog = !state.preferences.show_dialog;
            state.window.request_redraw();
            true
        }
        KeyCode::Comma => {
            state.transform_orientation = match state.transform_orientation {
                TransformOrientation::Global => TransformOrientation::Local,
                TransformOrientation::Local => TransformOrientation::Normal,
                TransformOrientation::Normal => TransformOrientation::View,
                TransformOrientation::View => TransformOrientation::Global,
            };
            state.window.request_redraw();
            true
        }
        KeyCode::KeyO => {
            state.proportional_editing = !state.proportional_editing;
            state.window.request_redraw();
            true
        }
        KeyCode::Numpad1 => {
            state.camera.projection = CameraProjection::Orthographic;
            state.camera.yaw = if state.ctrl_pressed {
                std::f32::consts::PI
            } else {
                0.0
            };
            state.camera.pitch = 0.0;
            state.window.request_redraw();
            true
        }
        KeyCode::Numpad3 => {
            state.camera.projection = CameraProjection::Orthographic;
            state.camera.yaw = if state.ctrl_pressed {
                std::f32::consts::FRAC_PI_2
            } else {
                -std::f32::consts::FRAC_PI_2
            };
            state.camera.pitch = 0.0;
            state.window.request_redraw();
            true
        }
        KeyCode::Numpad7 => {
            state.camera.projection = CameraProjection::Orthographic;
            state.camera.yaw = 0.0;
            state.camera.pitch = if state.ctrl_pressed {
                -std::f32::consts::FRAC_PI_2 + 0.001
            } else {
                std::f32::consts::FRAC_PI_2 - 0.001
            };
            state.window.request_redraw();
            true
        }
        KeyCode::Numpad5 => {
            state.camera.projection = match state.camera.projection {
                CameraProjection::Perspective => CameraProjection::Orthographic,
                CameraProjection::Orthographic => CameraProjection::Perspective,
            };
            state.window.request_redraw();
            true
        }
        KeyCode::Numpad0 => {
            if state.ctrl_pressed {
                // Ctrl + Numpad 0: Align Active Camera to Current View
                let cam = state
                    .active_camera_node
                    .or_else(|| {
                        state.selected_node.filter(|&id| {
                            matches!(
                                state.scene.nodes.get(id).map(|n| &n.data),
                                Some(graph_engine::NodeData::Camera { .. })
                            )
                        })
                    })
                    .or_else(|| {
                        state.scene.nodes.iter().find_map(|(id, node)| {
                            if matches!(node.data, graph_engine::NodeData::Camera { .. }) {
                                Some(id)
                            } else {
                                None
                            }
                        })
                    });

                if let Some(cam_id) = cam {
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
                    state.active_camera_node = Some(cam_id);
                }
            } else {
                // Numpad 0: Toggle Active Camera View / Free Orbit
                if state.active_camera_node.is_some() {
                    state.active_camera_node = None;
                } else {
                    let cam = state
                        .selected_node
                        .filter(|&id| {
                            matches!(
                                state.scene.nodes.get(id).map(|n| &n.data),
                                Some(graph_engine::NodeData::Camera { .. })
                            )
                        })
                        .or_else(|| {
                            state.scene.nodes.iter().find_map(|(id, node)| {
                                if matches!(node.data, graph_engine::NodeData::Camera { .. }) {
                                    Some(id)
                                } else {
                                    None
                                }
                            })
                        });
                    if let Some(cam_id) = cam {
                        state.active_camera_node = Some(cam_id);
                    }
                }
            }
            state.window.request_redraw();
            true
        }
        KeyCode::Numpad9 => {
            state.camera.yaw += std::f32::consts::PI;
            state.window.request_redraw();
            true
        }
        KeyCode::Numpad4 => {
            state.camera.orbit(30.0, 0.0);
            state.window.request_redraw();
            true
        }
        KeyCode::Numpad6 => {
            state.camera.orbit(-30.0, 0.0);
            state.window.request_redraw();
            true
        }
        KeyCode::Numpad8 => {
            state.camera.orbit(0.0, -30.0);
            state.window.request_redraw();
            true
        }
        KeyCode::Numpad2 => {
            state.camera.orbit(0.0, 30.0);
            state.window.request_redraw();
            true
        }
        KeyCode::KeyF | KeyCode::NumpadDecimal => {
            state.frame_selected();
            state.window.request_redraw();
            true
        }
        KeyCode::Escape => {
            if state.curve_pen_active {
                state.curve_pen_active = false;
                state.curve_pen_points.clear();
                state.curve_pen_drag_origin = None;
                state.window.request_redraw();
                true
            } else if state.measure_mode {
                state.measure_mode = false;
                state.window.request_redraw();
                true
            } else {
                false
            }
        }
        KeyCode::Enter | KeyCode::NumpadEnter if state.curve_pen_active => {
            state.commit_curve_pen();
            true
        }
        KeyCode::KeyC if state.curve_pen_active && state.curve_pen_points.len() >= 2 => {
            state.curve_pen_is_closed = true;
            state.commit_curve_pen();
            true
        }
        KeyCode::Backspace | KeyCode::Delete if state.curve_pen_active => {
            state.curve_pen_points.pop();
            state.window.request_redraw();
            true
        }
        KeyCode::KeyP => {
            state.curve_pen_active = !state.curve_pen_active;
            if state.curve_pen_active {
                state.curve_pen_points.clear();
                state.curve_pen_is_closed = false;
                state.curve_pen_drag_origin = None;
            }
            state.window.request_redraw();
            true
        }
        KeyCode::KeyM if state.interaction_mode == InteractionMode::Object => {
            state.measure_mode = !state.measure_mode;
            state.window.request_redraw();
            true
        }
        KeyCode::F1 => {
            state.show_help_dialog = !state.show_help_dialog;
            state.window.request_redraw();
            true
        }
        KeyCode::F2 => {
            state.welcome_hub.show_dialog = !state.welcome_hub.show_dialog;
            state.window.request_redraw();
            true
        }
        KeyCode::F3 => {
            state.command_palette.is_open = !state.command_palette.is_open;
            state.command_palette.focus_requested = false;
            state.window.request_redraw();
            true
        }
        KeyCode::KeyK if state.ctrl_pressed => {
            state.command_palette.is_open = !state.command_palette.is_open;
            state.command_palette.focus_requested = false;
            state.window.request_redraw();
            true
        }
        KeyCode::F11 => {
            state.production_render.engine = crate::ui::render_dialog::RenderEngine::DirectPbr;
            state.production_render.show_dialog = true;
            crate::ui::render_dialog::start_production_render(state);
            state.window.request_redraw();
            true
        }
        KeyCode::F12 => {
            state.production_render.engine = crate::ui::render_dialog::RenderEngine::PathTracer;
            state.production_render.show_dialog = true;
            crate::ui::render_dialog::start_production_render(state);
            state.window.request_redraw();
            true
        }
        _ => false,
    }
}

//! Hardware Kitbash / Insert Mesh Tool modal handler.
//! Provides rapid drop of mechanical hardware (Hex Bolts, Allen Screws, Vents, Latches)
//! aligned directly onto mesh face normals with mouse wheel scaling.

use core_math::{Quat, Transform, Vec3};
use geo_kernel::KitbashHardwareType;
use graph_engine::SceneNode;
use winit::dpi::PhysicalPosition;
use winit::event::MouseButton;
use winit::keyboard::KeyCode;

use crate::state::AppState;

#[derive(Debug, Clone)]
pub struct KitbashModalState {
    pub hardware_type: KitbashHardwareType,
    pub scale: f32,
    pub is_subtractive: bool,
    pub hover_pos: Vec3,
    pub hover_normal: Vec3,
    pub has_target: bool,
}

impl Default for KitbashModalState {
    fn default() -> Self {
        Self {
            hardware_type: KitbashHardwareType::HexBolt,
            scale: 1.0,
            is_subtractive: false,
            hover_pos: Vec3::ZERO,
            hover_normal: Vec3::Y,
            has_target: false,
        }
    }
}

pub fn start(state: &mut AppState) {
    state.kitbash_modal = Some(KitbashModalState::default());
    state.set_status_message(
        "🔩 Kitbash Hardware Tool: Hover surface & LMB to drop | Wheel: Scale | Tab/1-4: Hardware | B: Union/Cut | Esc: Exit",
    );
    state.window.request_redraw();
}

pub fn handle_cursor_moved(state: &mut AppState, position: PhysicalPosition<f64>) -> bool {
    let ray = state.get_screen_ray(position.x, position.y);
    let mut best_hit = None;

    for node in state.scene.nodes.values() {
        if let graph_engine::NodeData::Mesh { base_mesh, .. } = &node.data {
            if let Some(hit) = base_mesh.raycast(&ray, &node.world_matrix) {
                if best_hit
                    .as_ref()
                    .is_none_or(|b: &geo_kernel::RaycastHit| hit.distance < b.distance)
                {
                    best_hit = Some(hit);
                }
            }
        }
    }

    if let Some(kb) = &mut state.kitbash_modal {
        if let Some(hit) = best_hit {
            kb.hover_pos = hit.point;
            kb.hover_normal = hit.normal;
            kb.has_target = true;
        } else {
            kb.has_target = false;
        }
        state.window.request_redraw();
        return true;
    }

    false
}

pub fn handle_mouse_wheel(state: &mut AppState, scroll: f32) -> bool {
    if let Some(kb) = &mut state.kitbash_modal {
        kb.scale = (kb.scale + scroll * 0.1).clamp(0.1, 5.0);
        state.window.request_redraw();
        return true;
    }
    false
}

pub fn handle_keyboard_input(state: &mut AppState, key_code: KeyCode) -> bool {
    match key_code {
        KeyCode::Tab => {
            let label = if let Some(kb) = &mut state.kitbash_modal {
                kb.hardware_type = match kb.hardware_type {
                    KitbashHardwareType::HexBolt => KitbashHardwareType::AllenScrew,
                    KitbashHardwareType::AllenScrew => KitbashHardwareType::SlottedVent,
                    KitbashHardwareType::SlottedVent => KitbashHardwareType::LatchHandle,
                    KitbashHardwareType::LatchHandle => KitbashHardwareType::HexBolt,
                };
                Some(kb.hardware_type.label())
            } else {
                None
            };
            if let Some(lbl) = label {
                state.set_status_message(format!("Selected Hardware: {}", lbl));
                state.window.request_redraw();
            }
            true
        }
        KeyCode::Digit1 => {
            let label = if let Some(kb) = &mut state.kitbash_modal {
                kb.hardware_type = KitbashHardwareType::HexBolt;
                Some(kb.hardware_type.label())
            } else {
                None
            };
            if let Some(lbl) = label {
                state.set_status_message(format!("Selected Hardware: {}", lbl));
                state.window.request_redraw();
            }
            true
        }
        KeyCode::Digit2 => {
            let label = if let Some(kb) = &mut state.kitbash_modal {
                kb.hardware_type = KitbashHardwareType::AllenScrew;
                Some(kb.hardware_type.label())
            } else {
                None
            };
            if let Some(lbl) = label {
                state.set_status_message(format!("Selected Hardware: {}", lbl));
                state.window.request_redraw();
            }
            true
        }
        KeyCode::Digit3 => {
            let label = if let Some(kb) = &mut state.kitbash_modal {
                kb.hardware_type = KitbashHardwareType::SlottedVent;
                Some(kb.hardware_type.label())
            } else {
                None
            };
            if let Some(lbl) = label {
                state.set_status_message(format!("Selected Hardware: {}", lbl));
                state.window.request_redraw();
            }
            true
        }
        KeyCode::Digit4 => {
            let label = if let Some(kb) = &mut state.kitbash_modal {
                kb.hardware_type = KitbashHardwareType::LatchHandle;
                Some(kb.hardware_type.label())
            } else {
                None
            };
            if let Some(lbl) = label {
                state.set_status_message(format!("Selected Hardware: {}", lbl));
                state.window.request_redraw();
            }
            true
        }
        KeyCode::KeyB => {
            let mode_str = if let Some(kb) = &mut state.kitbash_modal {
                kb.is_subtractive = !kb.is_subtractive;
                Some(if kb.is_subtractive {
                    "Difference / Counterbore Cut"
                } else {
                    "Union / Hardware Add"
                })
            } else {
                None
            };
            if let Some(str_val) = mode_str {
                state.set_status_message(format!("Hardware Mode: {}", str_val));
                state.window.request_redraw();
            }
            true
        }
        KeyCode::Escape => {
            state.kitbash_modal = None;
            state.set_status_message("Exited Kitbash Tool");
            state.window.request_redraw();
            true
        }
        _ => false,
    }
}

pub fn handle_mouse_input(state: &mut AppState, button: MouseButton, is_pressed: bool) -> bool {
    if !is_pressed {
        return false;
    }

    match button {
        MouseButton::Left => {
            commit(state);
            true
        }
        MouseButton::Right => {
            state.kitbash_modal = None;
            state.set_status_message("Exited Kitbash Tool");
            state.window.request_redraw();
            true
        }
        _ => false,
    }
}

pub fn commit(state: &mut AppState) {
    let (hw_type, scale, is_subtractive, pos, normal, has_target) = match &state.kitbash_modal {
        Some(kb) => (
            kb.hardware_type,
            kb.scale,
            kb.is_subtractive,
            kb.hover_pos,
            kb.hover_normal,
            kb.has_target,
        ),
        None => return,
    };

    if !has_target {
        state.set_status_message("Hover over a mesh surface to insert hardware");
        return;
    }

    let mesh = hw_type.create_mesh(scale);
    let rotation = Quat::from_rotation_arc(Vec3::Y, normal.normalize_or_zero());
    let transform = Transform {
        translation: pos,
        rotation,
        scale: Vec3::splat(1.0),
    };

    let name = format!(
        "{}_{}",
        hw_type
            .label()
            .split_whitespace()
            .last()
            .unwrap_or("Hardware"),
        state.scene.nodes.len()
    );
    let new_node = SceneNode::new_mesh(name, transform, mesh);
    let id = state.scene.add_node(new_node);
    state.selected_node = Some(id);
    state.sync_gpu_meshes();

    let op_msg = if is_subtractive {
        "Socket Cut"
    } else {
        "Placed"
    };
    state.set_status_message(format!("{} {} at surface normal", op_msg, hw_type.label()));
    state.window.request_redraw();
}

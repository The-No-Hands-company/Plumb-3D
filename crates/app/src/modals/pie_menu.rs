use winit::event::MouseButton;
use winit::keyboard::KeyCode;

use crate::state::AppState;
use crate::types::InteractionMode;
use crate::ui::actions::UiActions;

/// Definition of an 8-directional radial pie slice
#[derive(Debug, Clone, Copy)]
pub struct PieSliceDef {
    pub name: &'static str,
    pub icon: &'static str,
    pub shortcut: &'static str,
    pub angle_deg: f32, // 0 = North, 45 = NE, 90 = E, 135 = SE, 180 = S, 225 = SW, 270 = W, 315 = NW
}

pub const PIE_SLICES: [PieSliceDef; 8] = [
    PieSliceDef {
        name: "Extrude",
        icon: "⬆",
        shortcut: "E",
        angle_deg: 0.0,
    },
    PieSliceDef {
        name: "Push-Pull",
        icon: "⇗",
        shortcut: "P",
        angle_deg: 45.0,
    },
    PieSliceDef {
        name: "Bevel",
        icon: "➡",
        shortcut: "Ctrl+B",
        angle_deg: 90.0,
    },
    PieSliceDef {
        name: "Inset",
        icon: "⇘",
        shortcut: "I",
        angle_deg: 135.0,
    },
    PieSliceDef {
        name: "SubD Limit",
        icon: "⬇",
        shortcut: "Limit",
        angle_deg: 180.0,
    },
    PieSliceDef {
        name: "Quad Remesh",
        icon: "⇙",
        shortcut: "Remesh",
        angle_deg: 225.0,
    },
    PieSliceDef {
        name: "Loop Cut",
        icon: "⬅",
        shortcut: "Ctrl+R",
        angle_deg: 270.0,
    },
    PieSliceDef {
        name: "Knife / Cut",
        icon: "⇖",
        shortcut: "K",
        angle_deg: 315.0,
    },
];

pub fn open(state: &mut AppState, mouse_pos: (f64, f64)) {
    state.pie_menu_active = true;
    state.pie_menu_center = mouse_pos;
    state.pie_menu_hovered_slice = None;
    state.window.request_redraw();
}

pub fn update_cursor(state: &mut AppState, mouse_pos: (f64, f64)) {
    if !state.pie_menu_active {
        return;
    }

    let dx = (mouse_pos.0 - state.pie_menu_center.0) as f32;
    let dy = (mouse_pos.1 - state.pie_menu_center.1) as f32;
    let dist = (dx * dx + dy * dy).sqrt();

    // Deadzone in the center (radius 22px)
    if dist < 22.0 {
        state.pie_menu_hovered_slice = None;
        return;
    }

    // Angle in screen space where North is up (-Y):
    // atan2(dx, -dy) gives 0 at North, PI/2 at East, PI at South, -PI/2 at West
    let mut rad = dx.atan2(-dy);
    if rad < 0.0 {
        rad += std::f32::consts::PI * 2.0;
    }
    let deg = rad.to_degrees();

    // 8 slices of 45 degrees, offset by 22.5 to center North at [337.5, 22.5]
    let slice_idx = (((deg + 22.5) / 45.0).floor() as usize) % 8;
    state.pie_menu_hovered_slice = Some(slice_idx);
    state.window.request_redraw();
}

pub fn commit(state: &mut AppState, actions: &mut UiActions) {
    if !state.pie_menu_active {
        return;
    }

    let chosen = state.pie_menu_hovered_slice;
    state.pie_menu_active = false;
    state.pie_menu_hovered_slice = None;
    state.window.request_redraw();

    let slice = match chosen {
        Some(idx) => idx,
        None => return,
    };

    match slice {
        0 => {
            // Extrude
            state.interaction_mode = InteractionMode::Edit;
            actions.trigger_extrude = true;
        }
        1 => {
            // Push-Pull Modal
            state.interaction_mode = InteractionMode::Edit;
            if !state.selected_faces.is_empty() {
                crate::modals::push_pull::start(state, state.pie_menu_center);
            }
        }
        2 => {
            // Bevel
            state.interaction_mode = InteractionMode::Edit;
            actions.trigger_bevel = true;
        }
        3 => {
            // Inset
            state.interaction_mode = InteractionMode::Edit;
            actions.trigger_inset = true;
        }
        4 => {
            // SubD Limit
            if let Some(sel_id) = state.selected_node {
                let cmd = Box::new(crate::commands::SubdLimitCommand::new(sel_id));
                let _ = state.history.execute(cmd, &mut state.scene);
                state.sync_gpu_meshes();
                state.set_status_message("Mesh projected to exact Pixar SubD limit surface.");
            }
        }
        5 => {
            // Quad Remesh
            if let Some(sel_id) = state.selected_node {
                let settings = geo_kernel::FieldRemeshSettings {
                    target_edge_length: state.field_remesh_edge_len,
                    curvature_influence: state.field_remesh_curvature_influence,
                    smooth_passes: 4,
                };
                let cmd = Box::new(crate::commands::FieldRemeshCommand::new(sel_id, settings));
                let _ = state.history.execute(cmd, &mut state.scene);
                state.sync_gpu_meshes();
                state.set_status_message("Mesh remeshed to field-aligned quads.");
            }
        }
        6 => {
            // Loop Cut
            state.interaction_mode = InteractionMode::Edit;
            actions.trigger_loop_cut = true;
        }
        7 => {
            // Knife / Cut
            state.interaction_mode = InteractionMode::Edit;
            state.knife_active = true;
        }
        _ => {}
    }
}

pub fn cancel(state: &mut AppState) {
    state.pie_menu_active = false;
    state.pie_menu_hovered_slice = None;
    state.window.request_redraw();
}

pub fn handle_mouse_input(
    state: &mut AppState,
    actions: &mut UiActions,
    button: MouseButton,
    is_pressed: bool,
) -> bool {
    if !state.pie_menu_active {
        return false;
    }

    if button == MouseButton::Left && !is_pressed {
        commit(state, actions);
        return true;
    } else if button == MouseButton::Right && is_pressed {
        cancel(state);
        return true;
    }

    true
}

pub fn handle_keyboard_input(
    state: &mut AppState,
    actions: &mut UiActions,
    key: KeyCode,
    is_pressed: bool,
) -> bool {
    if !state.pie_menu_active {
        return false;
    }

    if key == KeyCode::Escape {
        cancel(state);
        return true;
    }

    if key == KeyCode::KeyV && !is_pressed {
        commit(state, actions);
        return true;
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pie_menu_angles_and_slices() {
        assert_eq!(PIE_SLICES.len(), 8);
        assert_eq!(PIE_SLICES[0].name, "Extrude");
        assert_eq!(PIE_SLICES[1].name, "Push-Pull");
        assert_eq!(PIE_SLICES[2].name, "Bevel");
        assert_eq!(PIE_SLICES[4].name, "SubD Limit");
    }
}

use crate::state::AppState;
use crate::types::*;
use crate::ui::actions::UiActions;

pub fn draw_align_dialog(state: &mut AppState, actions: &mut UiActions) {
    // Render Align Objects Dialog (Maya Align / 3ds Max Align Tool)
    if state.show_align_dialog {
        let mut close_dialog = false;
        let can_align = state.selected_node.is_some() && state.previous_selected_node.is_some();

        egui::Window::new("Align Objects (3ds Max / Maya)")
            .open(&mut state.show_align_dialog)
            .resizable(false)
            .show(&state.egui_ctx, |ui| {
                if !can_align {
                    ui.label("Select a source object, then a target object to align.");
                    return;
                }

                let src_name = state
                    .selected_node
                    .and_then(|id| state.scene.nodes.get(id))
                    .map(|n| n.name.as_str())
                    .unwrap_or("Current");
                let tgt_name = state
                    .previous_selected_node
                    .and_then(|id| state.scene.nodes.get(id))
                    .map(|n| n.name.as_str())
                    .unwrap_or("Target");

                ui.label(format!("Source: {} -> Target: {}", src_name, tgt_name));
                ui.separator();

                ui.heading("Align Position");
                ui.horizontal(|ui| {
                    ui.checkbox(&mut state.align_pos_x, "X");
                    ui.checkbox(&mut state.align_pos_y, "Y");
                    ui.checkbox(&mut state.align_pos_z, "Z");
                });
                ui.horizontal(|ui| {
                    ui.label("Align Mode:");
                    ui.selectable_value(&mut state.align_pos_mode, AlignPosMode::Center, "Center");
                    ui.selectable_value(&mut state.align_pos_mode, AlignPosMode::Pivot, "Pivot");
                    ui.selectable_value(&mut state.align_pos_mode, AlignPosMode::Minimum, "Min");
                    ui.selectable_value(&mut state.align_pos_mode, AlignPosMode::Maximum, "Max");
                });

                ui.separator();
                ui.heading("Align Orientation (Rotation)");
                ui.horizontal(|ui| {
                    ui.checkbox(&mut state.align_rot_x, "X");
                    ui.checkbox(&mut state.align_rot_y, "Y");
                    ui.checkbox(&mut state.align_rot_z, "Z");
                });

                ui.separator();
                ui.heading("Align Scale");
                ui.horizontal(|ui| {
                    ui.checkbox(&mut state.align_scale_x, "X");
                    ui.checkbox(&mut state.align_scale_y, "Y");
                    ui.checkbox(&mut state.align_scale_z, "Z");
                });

                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("Apply Alignment").clicked() {
                        actions.trigger_apply_align = true;
                        close_dialog = true;
                    }
                    if ui.button("Close").clicked() {
                        close_dialog = true;
                    }
                });
            });

        if close_dialog {
            state.show_align_dialog = false;
        }
    }
}

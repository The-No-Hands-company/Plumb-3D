use crate::state::AppState;

/// Renders the interactive Undo/Redo Action History panel (Photoshop / Blender / Maya style)
pub fn draw_history_inner(state: &mut AppState, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.heading("Action History");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let redo_enabled = state.history.can_redo();
            if ui.add_enabled(redo_enabled, egui::Button::new("Redo >")).on_hover_text("Redo next action (Ctrl+Y)").clicked() {
                let _ = state.history.redo(&mut state.scene);
                state.window.request_redraw();
            }
            let undo_enabled = state.history.can_undo();
            if ui.add_enabled(undo_enabled, egui::Button::new("< Undo")).on_hover_text("Undo previous action (Ctrl+Z)").clicked() {
                let _ = state.history.undo(&mut state.scene);
                state.window.request_redraw();
            }
        });
    });
    ui.separator();

    let undo_names = state.history.undo_stack_names();
    let redo_names = state.history.redo_stack_names();
    let total_undo = undo_names.len();

    if total_undo == 0 && redo_names.is_empty() {
        ui.label(egui::RichText::new("No history recorded yet.").weak().italics());
        ui.label(egui::RichText::new("Actions such as adding primitives, extruding, insetting, or beveling will appear here.").small().weak());
        return;
    }

    let mut jump_target = None;
    let mut jump_initial = false;
    let mut redo_steps = None;

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            // 1. Initial Scene State
            let is_at_start = total_undo == 0;
            let start_label = if is_at_start {
                egui::RichText::new("[0] Initial Scene State (Current)").strong().color(egui::Color32::from_rgb(226, 179, 87))
            } else {
                egui::RichText::new("[0] Initial Scene State").color(egui::Color32::LIGHT_GRAY)
            };
            if ui.selectable_label(is_at_start, start_label).on_hover_text("Roll back entire scene to initial state").clicked() {
                if total_undo > 0 {
                    jump_initial = true;
                }
            }

            // 2. Executed Undo Stack (Active History)
            for (i, name) in undo_names.iter().enumerate() {
                let is_current = i + 1 == total_undo && redo_names.is_empty();
                let text = if is_current {
                    format!("[{}] {} (Current)", i + 1, name)
                } else {
                    format!("[{}] {}", i + 1, name)
                };

                let label = if is_current {
                    egui::RichText::new(text).strong().color(egui::Color32::from_rgb(100, 200, 255))
                } else {
                    egui::RichText::new(text).color(egui::Color32::from_rgb(220, 225, 235))
                };

                if ui.selectable_label(is_current, label).on_hover_text(format!("Click to jump directly back to step {}", i + 1)).clicked() {
                    if i + 1 < total_undo || !redo_names.is_empty() {
                        jump_target = Some(i);
                    }
                }
            }

            // 3. Undone Redo Stack (Future History)
            if !redo_names.is_empty() {
                ui.add_space(4.0);
                ui.separator();
                ui.label(egui::RichText::new("Undone Actions (Click to Redo):").small().weak());

                for (j, name) in redo_names.iter().enumerate() {
                    let text = format!("[+{}] {}", j + 1, name);
                    let label = egui::RichText::new(text).italics().color(egui::Color32::from_rgb(140, 150, 165));

                    if ui.button(label).on_hover_text("Click to redo forward to this step").clicked() {
                        redo_steps = Some(j + 1);
                    }
                }
            }
        });

    if jump_initial {
        let _ = state.history.jump_to_initial_state(&mut state.scene);
        state.window.request_redraw();
    } else if let Some(target) = jump_target {
        let _ = state.history.jump_to_undo_step(&mut state.scene, target);
        state.window.request_redraw();
    } else if let Some(steps) = redo_steps {
        let _ = state.history.jump_forward_redo(&mut state.scene, steps);
        state.window.request_redraw();
    }
}

/// Standalone History Window toggleable via Edit -> Undo History (Ctrl+Alt+Z)
pub fn draw_history_window(state: &mut AppState) {
    if !state.show_history_window {
        return;
    }

    let mut is_open = state.show_history_window;
    let egui_ctx = state.egui_ctx.clone();

    egui::Window::new("Undo History & Actions")
        .open(&mut is_open)
        .default_width(280.0)
        .default_height(380.0)
        .resizable(true)
        .show(&egui_ctx, |ui| {
            draw_history_inner(state, ui);
        });

    state.show_history_window = is_open;
}

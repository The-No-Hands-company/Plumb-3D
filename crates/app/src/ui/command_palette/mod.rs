pub mod items;

use crate::state::AppState;
use crate::ui::actions::UiActions;
pub use items::*;

/// Runtime state for the Omnipresent Command Palette modal (F3 / Ctrl+K).
#[derive(Debug, Clone, Default)]
pub struct CommandPaletteState {
    pub is_open: bool,
    pub query: String,
    pub selected_index: usize,
    pub focus_requested: bool,
}

/// Draws the spotlight-style Omnipresent Command Palette modal dialog.
pub fn draw_command_palette(state: &mut AppState, actions: &mut UiActions) {
    if !state.command_palette.is_open {
        return;
    }

    let ctx = state.egui_ctx.clone();
    let mut close = false;
    let mut trigger_action = None;

    let q = state.command_palette.query.trim().to_lowercase();
    let filtered_indices: Vec<usize> = COMMAND_LIST
        .iter()
        .enumerate()
        .filter(|(_, item)| {
            if q.is_empty() {
                return true;
            }
            item.name.to_lowercase().contains(&q)
                || item.category.to_lowercase().contains(&q)
                || item.shortcut.to_lowercase().contains(&q)
                || item.keywords.iter().any(|k| k.to_lowercase().contains(&q))
        })
        .map(|(idx, _)| idx)
        .collect();

    // Check keyboard navigation in egui
    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        close = true;
    }
    if ctx.input(|i| i.key_pressed(egui::Key::ArrowDown)) && !filtered_indices.is_empty() {
        state.command_palette.selected_index =
            (state.command_palette.selected_index + 1) % filtered_indices.len();
    }
    if ctx.input(|i| i.key_pressed(egui::Key::ArrowUp)) && !filtered_indices.is_empty() {
        if state.command_palette.selected_index == 0 {
            state.command_palette.selected_index = filtered_indices.len() - 1;
        } else {
            state.command_palette.selected_index -= 1;
        }
    }
    if ctx.input(|i| i.key_pressed(egui::Key::Enter)) && !filtered_indices.is_empty() {
        let sel = state
            .command_palette
            .selected_index
            .min(filtered_indices.len() - 1);
        trigger_action = Some(COMMAND_LIST[filtered_indices[sel]].action);
        close = true;
    }

    egui::Window::new("🔍 Plumb Command Palette")
        .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 80.0))
        .fixed_size(egui::vec2(580.0, 420.0))
        .collapsible(false)
        .resizable(false)
        .title_bar(false)
        .frame(
            egui::Frame::window(&ctx.style())
                .fill(egui::Color32::from_rgb(20, 23, 29))
                .stroke(egui::Stroke::new(
                    1.5_f32,
                    egui::Color32::from_rgb(226, 179, 87),
                ))
                .rounding(egui::Rounding::same(8.0))
                .inner_margin(egui::Margin::same(12.0)),
        )
        .show(&ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("🔍").size(18.0));
                let resp = ui.add(
                    egui::TextEdit::singleline(&mut state.command_palette.query)
                        .hint_text("Search operator, tool, workspace, theme, or shortcut... (e.g. 'bridge', 'bevel', 'caliper')")
                        .desired_width(490.0)
                        .font(egui::TextStyle::Heading),
                );
                if !state.command_palette.focus_requested {
                    resp.request_focus();
                    state.command_palette.focus_requested = true;
                }
                if ui.button("❌").clicked() {
                    close = true;
                }
            });

            ui.separator();

            if filtered_indices.is_empty() {
                ui.add_space(20.0);
                ui.vertical_centered(|ui| {
                    ui.label(
                        egui::RichText::new("No matching commands found")
                            .color(egui::Color32::GRAY),
                    );
                    ui.label(
                        egui::RichText::new(
                            "Tip: Try searching by action (e.g. 'bridge', 'cut', 'round', 'save')",
                        )
                        .small()
                        .color(egui::Color32::DARK_GRAY),
                    );
                });
            } else {
                egui::ScrollArea::vertical()
                    .max_height(330.0)
                    .show(ui, |ui| {
                        for (list_idx, &cmd_idx) in filtered_indices.iter().enumerate() {
                            let item = &COMMAND_LIST[cmd_idx];
                            let is_selected = list_idx == state.command_palette.selected_index;

                            let bg_color = if is_selected {
                                egui::Color32::from_rgb(46, 52, 68)
                            } else {
                                egui::Color32::TRANSPARENT
                            };

                            let row_resp = egui::Frame::none()
                                .fill(bg_color)
                                .rounding(egui::Rounding::same(4.0))
                                .inner_margin(egui::Margin::symmetric(8.0, 5.0))
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        // Category Badge
                                        let (badge_bg, badge_fg) = match item.category {
                                            "Modeling" => (
                                                egui::Color32::from_rgb(40, 80, 120),
                                                egui::Color32::from_rgb(170, 220, 255),
                                            ),
                                            "CAD" => (
                                                egui::Color32::from_rgb(30, 90, 60),
                                                egui::Color32::from_rgb(160, 250, 200),
                                            ),
                                            "File" => (
                                                egui::Color32::from_rgb(90, 70, 30),
                                                egui::Color32::from_rgb(250, 210, 140),
                                            ),
                                            "Workspace" => (
                                                egui::Color32::from_rgb(70, 40, 90),
                                                egui::Color32::from_rgb(230, 180, 255),
                                            ),
                                            "Theme" => (
                                                egui::Color32::from_rgb(80, 40, 50),
                                                egui::Color32::from_rgb(255, 180, 190),
                                            ),
                                            _ => (
                                                egui::Color32::from_rgb(45, 50, 60),
                                                egui::Color32::WHITE,
                                            ),
                                        };

                                        egui::Frame::none()
                                            .fill(badge_bg)
                                            .rounding(egui::Rounding::same(3.0))
                                            .inner_margin(egui::Margin::symmetric(5.0, 2.0))
                                            .show(ui, |ui| {
                                                ui.label(
                                                    egui::RichText::new(item.category)
                                                        .size(10.0)
                                                        .strong()
                                                        .color(badge_fg),
                                                );
                                            });

                                        let text_color = if is_selected {
                                            egui::Color32::from_rgb(240, 210, 120)
                                        } else {
                                            egui::Color32::from_rgb(230, 235, 245)
                                        };
                                        ui.label(
                                            egui::RichText::new(item.name)
                                                .size(13.0)
                                                .color(text_color),
                                        );

                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Center),
                                            |ui| {
                                                if !item.shortcut.is_empty() {
                                                    ui.label(
                                                        egui::RichText::new(item.shortcut)
                                                            .monospace()
                                                            .size(11.0)
                                                            .color(egui::Color32::from_rgb(
                                                                160, 170, 190,
                                                            )),
                                                    );
                                                }
                                            },
                                        );
                                    });
                                })
                                .response;

                            if row_resp.clicked() {
                                trigger_action = Some(item.action);
                                close = true;
                            }
                        }
                    });
            }

            ui.add_space(4.0);
            ui.separator();
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("↑/↓ Navigate")
                        .size(10.0)
                        .color(egui::Color32::GRAY),
                );
                ui.label(egui::RichText::new("•").size(10.0).color(egui::Color32::DARK_GRAY));
                ui.label(
                    egui::RichText::new("Enter Execute")
                        .size(10.0)
                        .color(egui::Color32::GRAY),
                );
                ui.label(egui::RichText::new("•").size(10.0).color(egui::Color32::DARK_GRAY));
                ui.label(
                    egui::RichText::new("Esc Close")
                        .size(10.0)
                        .color(egui::Color32::GRAY),
                );
            });
        });

    if close {
        state.command_palette.is_open = false;
        state.command_palette.focus_requested = false;
        state.command_palette.query.clear();
        state.command_palette.selected_index = 0;
    }

    if let Some(act) = trigger_action {
        execute_palette_action(state, actions, act);
    }
}

fn execute_palette_action(state: &mut AppState, actions: &mut UiActions, act: PaletteAction) {
    match act {
        PaletteAction::TriggerAction(f) => f(state, actions),
        PaletteAction::SetWorkspace(w) => {
            state.current_workspace = w;
            crate::ui::workspaces::apply_workspace(state, w);
        }
        PaletteAction::SetTheme(t) => {
            state.set_theme(t);
        }
        PaletteAction::SetInteractionMode(m) => {
            state.interaction_mode = m;
        }
        PaletteAction::SetShading(s) => {
            state.viewport_shading = s;
        }
        PaletteAction::ToggleWireframe => {
            state.show_wireframe = !state.show_wireframe;
        }
        PaletteAction::ToggleGrid => {
            state.show_grid = !state.show_grid;
        }
        PaletteAction::ToggleMeasure => {
            state.measure_mode = !state.measure_mode;
        }
        PaletteAction::ToggleSnap => {
            state.snap_enabled = !state.snap_enabled;
        }
        PaletteAction::OpenWelcomeHub => {
            state.welcome_hub.show_dialog = true;
        }
        PaletteAction::OpenPreferences => {
            state.preferences.show_dialog = true;
        }
        PaletteAction::OpenHelp(tab) => {
            state.show_help_dialog = true;
            state.help_dialog_tab = tab;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_list_completeness() {
        assert!(COMMAND_LIST.len() >= 30);
        for cmd in COMMAND_LIST {
            assert!(!cmd.name.is_empty());
            assert!(!cmd.category.is_empty());
            assert!(!cmd.keywords.is_empty());
        }
    }
}

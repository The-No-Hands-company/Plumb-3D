pub mod menus;
pub mod quick_controls;
pub mod render_menu;

pub use menus::*;
pub use quick_controls::*;
pub use render_menu::*;

use crate::state::AppState;
use crate::ui::actions::UiActions;

pub fn draw_top_menu(state: &mut AppState, actions: &mut UiActions) {
    let ctx = state.egui_ctx.clone();

    // 1. Application Main Menu Bar
    egui::TopBottomPanel::top("top_app_menu_bar").show(&ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Search (F3)").on_hover_text("Open Omnipresent Command Palette (F3 / Ctrl+K)").clicked() {
                    state.command_palette.is_open = true;
                    state.command_palette.focus_requested = false;
                }
                if ui.selectable_label(state.welcome_hub.show_dialog, "Welcome").on_hover_text("Open Welcome & Project Templates Hub (F2)").clicked() {
                    state.welcome_hub.show_dialog = !state.welcome_hub.show_dialog;
                }
                if ui.selectable_label(state.show_help_dialog, "Help (F1)").on_hover_text("Open User Manual & Shortcuts (F1)").clicked() {
                    state.show_help_dialog = !state.show_help_dialog;
                }
                if ui.button("Render (F12)").on_hover_text("Open Production Path Tracer").clicked() {
                    state.production_render.show_dialog = true;
                    crate::ui::render_dialog::start_production_render(state);
                }

                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    draw_file_menu(state, actions, ui);
                    draw_edit_menu(state, actions, ui);
                    draw_add_menu(actions, ui);
                    draw_render_menu(state, ui);
                    draw_window_menu(state, ui);
                    draw_help_menu(state, ui);

                    ui.separator();
                    crate::ui::workspaces::draw_workspace_bar(state, ui);
                });
            });
        });
    });

    // 2. Viewport Ribbon / Mode Header Bar (Plasticity / Blender 4.x Style)
    egui::TopBottomPanel::top("viewport_ribbon_header")
        .frame(egui::Frame::none()
            .fill(egui::Color32::from_rgb(26, 28, 33))
            .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(38, 41, 48)))
            .inner_margin(egui::Margin::symmetric(10.0, 4.0)))
        .show(&ctx, |ui| {
            draw_quick_controls(state, ui);
        });
}

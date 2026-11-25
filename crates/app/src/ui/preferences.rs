use crate::state::AppState;
use crate::types::*;
use crate::ui::theme::ThemePreset;

/// Persistent preferences state for user customization and personal layout.
#[derive(Debug, Clone)]
pub struct PreferencesState {
    pub show_dialog: bool,
    pub active_tab: usize,
    pub ui_density: UiDensity,
    pub keymap_style: KeymapStyle,
    pub show_viewport_hints: bool,
    pub undo_depth_limit: usize,
    pub custom_workspace_name: String,
}

impl Default for PreferencesState {
    fn default() -> Self {
        Self {
            show_dialog: false,
            active_tab: 0,
            ui_density: UiDensity::Comfortable,
            keymap_style: KeymapStyle::PlumbStandard,
            show_viewport_hints: true,
            undo_depth_limit: 64,
            custom_workspace_name: String::new(),
        }
    }
}

/// Applies interface spacing and typography scale based on [`UiDensity`].
pub fn apply_ui_density(ctx: &egui::Context, density: UiDensity) {
    let mut style = (*ctx.style()).clone();
    match density {
        UiDensity::Compact => {
            style.spacing.item_spacing = egui::vec2(5.0, 3.0);
            style.spacing.button_padding = egui::vec2(6.0, 3.0);
            style.spacing.window_margin = egui::Margin::same(6.0);
        }
        UiDensity::Comfortable => {
            style.spacing.item_spacing = egui::vec2(8.0, 5.0);
            style.spacing.button_padding = egui::vec2(10.0, 5.0);
            style.spacing.window_margin = egui::Margin::same(10.0);
        }
    }
    ctx.set_style(style);
}

/// Draws the comprehensive Preferences & Customization modal dialog.
pub fn draw_preferences_dialog(state: &mut AppState) {
    if !state.preferences.show_dialog {
        return;
    }

    let ctx = state.egui_ctx.clone();
    let mut close = false;

    egui::Window::new("⚙ Plumb 3D — Preferences & Customization")
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .fixed_size(egui::vec2(580.0, 420.0))
        .collapsible(false)
        .resizable(false)
        .frame(
            egui::Frame::window(&ctx.style())
                .fill(egui::Color32::from_rgb(20, 22, 28))
                .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(50, 58, 72)))
                .rounding(egui::Rounding::same(8.0))
                .inner_margin(egui::Margin::same(14.0)),
        )
        .show(&ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut state.preferences.active_tab, 0, "🖥 Interface & Density");
                ui.selectable_value(&mut state.preferences.active_tab, 1, "⌨ Keymap Navigation");
                ui.selectable_value(&mut state.preferences.active_tab, 2, "📐 Workspaces");
                ui.selectable_value(&mut state.preferences.active_tab, 3, "⚡ System & Undo");

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("❌ Close").clicked() {
                        close = true;
                    }
                });
            });

            ui.add_space(6.0);
            ui.separator();
            ui.add_space(6.0);

            match state.preferences.active_tab {
                0 => {
                    // Interface & Density Tab
                    ui.label(egui::RichText::new("UI Density & Screen Scaling").strong().size(13.0));
                    ui.horizontal(|ui| {
                        if ui.selectable_value(&mut state.preferences.ui_density, UiDensity::Comfortable, "Comfortable (Standard)").clicked() {
                            apply_ui_density(&state.egui_ctx, UiDensity::Comfortable);
                        }
                        if ui.selectable_value(&mut state.preferences.ui_density, UiDensity::Compact, "Compact (Power / Laptop)").clicked() {
                            apply_ui_density(&state.egui_ctx, UiDensity::Compact);
                        }
                    });
                    ui.label(egui::RichText::new("Compact mode tightens button margins to maximize 3D viewport canvas.").small().color(egui::Color32::GRAY));

                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(10.0);

                    ui.label(egui::RichText::new("Theme Selection").strong().size(13.0));
                    for preset in ThemePreset::ALL {
                        let is_cur = state.current_theme == preset;
                        if ui.selectable_label(is_cur, preset.name()).clicked() {
                            state.set_theme(preset);
                        }
                    }

                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(10.0);

                    ui.checkbox(&mut state.preferences.show_viewport_hints, "Show dynamic context action hints at viewport bottom");
                    ui.checkbox(&mut state.show_viewport_hud, "Show scene statistics HUD (Vertices, Edges, Faces)");
                }
                1 => {
                    // Keymap Navigation Tab
                    ui.label(egui::RichText::new("Navigation Keymap Preset").strong().size(13.0));
                    ui.radio_value(&mut state.preferences.keymap_style, KeymapStyle::PlumbStandard, "Plumb 3D Native (W/E/R Translate/Rotate/Scale, Tab Edit Mode)");
                    ui.radio_value(&mut state.preferences.keymap_style, KeymapStyle::BlenderVeteran, "Blender Veteran (G Grab, S Scale, R Rotate, Shift+A Add, Tab Edit)");
                    ui.radio_value(&mut state.preferences.keymap_style, KeymapStyle::MayaStudio, "Maya Studio (Alt+LMB Orbit, Alt+MMB Pan, W/E/R Manipulators)");

                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(10.0);

                    ui.label(egui::RichText::new("Quick Shortcut Reference").strong().size(13.0));
                    egui::Grid::new("pref_shortcut_grid").num_columns(2).spacing([20.0, 4.0]).show(ui, |ui| {
                        ui.label(egui::RichText::new("F3 or Ctrl+K").strong()); ui.label("Omnipresent Command Palette"); ui.end_row();
                        ui.label(egui::RichText::new("F1").strong()); ui.label("In-Software Manual & Help Reference"); ui.end_row();
                        ui.label(egui::RichText::new("Tab").strong()); ui.label("Toggle Object / Edit Mode"); ui.end_row();
                        ui.label(egui::RichText::new("1, 2, 3").strong()); ui.label("Vertex, Edge, Face Select Level"); ui.end_row();
                        ui.label(egui::RichText::new("E, I, Ctrl+B").strong()); ui.label("Extrude, Inset, Bevel"); ui.end_row();
                        ui.label(egui::RichText::new("K, Ctrl+R").strong()); ui.label("Knife Tool, Loop Cut and Slide"); ui.end_row();
                        ui.label(egui::RichText::new("Shift+Tab").strong()); ui.label("Toggle Snapping to Grid/Vertices"); ui.end_row();
                    });
                }
                2 => {
                    // Workspaces Tab
                    ui.label(egui::RichText::new("Personalized Custom Workspaces").strong().size(13.0));
                    ui.label(egui::RichText::new("Save your current UI panels, shading, and inspector arrangement as a named workspace.").small().color(egui::Color32::GRAY));
                    ui.add_space(6.0);

                    ui.horizontal(|ui| {
                        ui.label("Name:");
                        ui.text_edit_singleline(&mut state.preferences.custom_workspace_name);
                        if ui.button("💾 Save Current Setup").clicked() && !state.preferences.custom_workspace_name.trim().is_empty() {
                            let name = state.preferences.custom_workspace_name.trim().to_string();
                            state.custom_workspaces.push(crate::ui::workspaces::CustomWorkspace {
                                name,
                                interaction_mode: state.interaction_mode,
                                shading: state.viewport_shading,
                                show_outliner: state.show_outliner,
                                show_inspector: state.show_inspector,
                                show_left_toolbar: state.show_left_toolbar,
                                show_wireframe: state.show_wireframe,
                                inspector_tab: state.inspector_tab,
                            });
                            state.preferences.custom_workspace_name.clear();
                        }
                    });

                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(8.0);

                    ui.label(egui::RichText::new("User Saved Workspaces").strong().size(12.0));
                    if state.custom_workspaces.is_empty() {
                        ui.label(egui::RichText::new("No custom workspaces saved yet.").color(egui::Color32::DARK_GRAY));
                    } else {
                        let mut to_load = None;
                        let mut to_remove = None;
                        for idx in 0..state.custom_workspaces.len() {
                            let name = state.custom_workspaces[idx].name.clone();
                            ui.horizontal(|ui| {
                                ui.label(format!("📁 {}", name));
                                if ui.button("Load").clicked() {
                                    to_load = Some(idx);
                                }
                                if ui.button("🗑").clicked() {
                                    to_remove = Some(idx);
                                }
                            });
                        }
                        if let Some(load_idx) = to_load {
                            crate::ui::workspaces::apply_custom_workspace(state, load_idx);
                        }
                        if let Some(del_idx) = to_remove {
                            state.custom_workspaces.remove(del_idx);
                        }
                    }
                }
                _ => {
                    // System & Undo Tab
                    ui.label(egui::RichText::new("Undo & History Memory").strong().size(13.0));
                    ui.horizontal(|ui| {
                        ui.label("Max Undo Steps:");
                        ui.add(egui::Slider::new(&mut state.preferences.undo_depth_limit, 16..=256));
                    });

                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(10.0);

                    ui.label(egui::RichText::new("Hardware & Rendering").strong().size(13.0));
                    ui.label("Device: pure safe Rust + wgpu / WebGPU".to_string());
                    ui.label(format!("Cached Meshes in VRAM: {}", state.gpu_mesh_cache.len()));
                    ui.label(format!("Target FPS: 60.0 (Current: {:.0} FPS)", state.fps));
                }
            }
        });

    if close {
        state.preferences.show_dialog = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preferences_defaults() {
        let p = PreferencesState::default();
        assert_eq!(p.ui_density, UiDensity::Comfortable);
        assert_eq!(p.keymap_style, KeymapStyle::PlumbStandard);
        assert!(p.show_viewport_hints);
        assert_eq!(p.undo_depth_limit, 64);
    }
}

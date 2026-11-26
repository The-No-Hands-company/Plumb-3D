use crate::state::AppState;
use crate::types::*;
use crate::ui::theme::ThemePreset;

/// Configurable keybinding definition with category, description, and conflict tracking.
#[derive(Debug, Clone, PartialEq)]
pub struct KeybindingEntry {
    pub action_id: String,
    pub category: String,
    pub description: String,
    pub key: String,
    pub default_key: String,
}

impl KeybindingEntry {
    pub fn new(action_id: &str, category: &str, description: &str, key: &str) -> Self {
        Self {
            action_id: action_id.to_string(),
            category: category.to_string(),
            description: description.to_string(),
            key: key.to_string(),
            default_key: key.to_string(),
        }
    }
}

/// Returns the factory default keybinding map across all DCC operational domains.
pub fn default_keybindings() -> Vec<KeybindingEntry> {
    vec![
        // Modeling
        KeybindingEntry::new("model.extrude", "Modeling", "Extrude Region / Faces", "E"),
        KeybindingEntry::new("model.inset", "Modeling", "Inset Polygons", "I"),
        KeybindingEntry::new("model.bevel", "Modeling", "Bevel Edges / Vertices", "Ctrl+B"),
        KeybindingEntry::new("model.loop_cut", "Modeling", "Loop Cut and Slide", "Ctrl+R"),
        KeybindingEntry::new("model.knife", "Modeling", "Interactive Knife Topology Cut", "K"),
        KeybindingEntry::new("model.delete", "Modeling", "Delete / Dissolve Sub-Elements", "X"),
        KeybindingEntry::new("model.bridge", "Modeling", "Bridge Edge Loops", "Alt+B"),
        KeybindingEntry::new("model.subdivide", "Modeling", "Subdivide Selected", "W"),

        // Navigation & Gizmo
        KeybindingEntry::new("nav.translate", "Navigation", "Translate Manipulator Gizmo", "G"),
        KeybindingEntry::new("nav.rotate", "Navigation", "Rotate Manipulator Gizmo", "R"),
        KeybindingEntry::new("nav.scale", "Navigation", "Scale Manipulator Gizmo", "S"),
        KeybindingEntry::new("nav.frame_selected", "Navigation", "Frame / Focus Selected Object", "Numpad ."),

        // Selection
        KeybindingEntry::new("select.box", "Selection", "Box Selection Mode", "B"),
        KeybindingEntry::new("select.circle", "Selection", "Circle / Paint Selection Brush", "C"),
        KeybindingEntry::new("select.lasso", "Selection", "Lasso Selection Mode", "Shift+Space"),
        KeybindingEntry::new("select.invert", "Selection", "Invert Current Selection", "Ctrl+I"),
        KeybindingEntry::new("select.all", "Selection", "Select / Deselect All", "A"),

        // General & Interface
        KeybindingEntry::new("general.mode_toggle", "General", "Toggle Object / Edit Mode", "Tab"),
        KeybindingEntry::new("general.palette", "General", "Omnipresent Command Palette", "F3"),
        KeybindingEntry::new("general.help", "General", "In-Software Help Reference Manual", "F1"),
        KeybindingEntry::new("general.snap", "General", "Toggle Snapping to Grid/Vertices", "Shift+Tab"),
        KeybindingEntry::new("general.undo", "General", "Undo Last Action", "Ctrl+Z"),
        KeybindingEntry::new("general.redo", "General", "Redo Action", "Ctrl+Y"),
        KeybindingEntry::new("general.save", "General", "Save Project File", "Ctrl+S"),
        KeybindingEntry::new("general.render", "General", "Render Production Viewport", "F12"),
    ]
}

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
    pub keybindings: Vec<KeybindingEntry>,
    pub keybinding_filter: String,
    pub editing_binding_idx: Option<usize>,
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
            keybindings: default_keybindings(),
            keybinding_filter: String::new(),
            editing_binding_idx: None,
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

    egui::Window::new("Plumb 3D -- Preferences & Customization")
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .fixed_size(egui::vec2(640.0, 500.0))
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
                ui.selectable_value(&mut state.preferences.active_tab, 0, "Interface & Density");
                ui.selectable_value(&mut state.preferences.active_tab, 1, "Keymap Navigation");
                ui.selectable_value(&mut state.preferences.active_tab, 2, "Workspaces");
                ui.selectable_value(&mut state.preferences.active_tab, 3, "System & Undo");

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Close").clicked() {
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
                    // Keymap Navigation & Visual Editor Tab
                    ui.label(egui::RichText::new("Navigation Keymap Preset").strong().size(13.0));
                    ui.horizontal(|ui| {
                        ui.radio_value(&mut state.preferences.keymap_style, KeymapStyle::PlumbStandard, "Plumb Native");
                        ui.radio_value(&mut state.preferences.keymap_style, KeymapStyle::BlenderVeteran, "Blender Veteran");
                        ui.radio_value(&mut state.preferences.keymap_style, KeymapStyle::MayaStudio, "Maya Studio");
                    });

                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(6.0);

                    // Conflict detection pass
                    let mut conflicts = std::collections::HashSet::new();
                    for i in 0..state.preferences.keybindings.len() {
                        let k1 = state.preferences.keybindings[i].key.trim().to_uppercase();
                        if k1.is_empty() {
                            continue;
                        }
                        for j in (i + 1)..state.preferences.keybindings.len() {
                            let k2 = state.preferences.keybindings[j].key.trim().to_uppercase();
                            if k1 == k2 {
                                conflicts.insert(i);
                                conflicts.insert(j);
                            }
                        }
                    }

                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Visual Hotkey Keymap Editor").strong().size(13.0));
                        if !conflicts.is_empty() {
                            ui.label(
                                egui::RichText::new(format!("⚠️ {} Key Conflicts Detected!", conflicts.len()))
                                    .color(egui::Color32::from_rgb(255, 120, 80))
                                    .strong(),
                            );
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Filter:");
                        ui.add(
                            egui::TextEdit::singleline(&mut state.preferences.keybinding_filter)
                                .hint_text("Search action or key..."),
                        );
                        if ui.button("Clear").clicked() {
                            state.preferences.keybinding_filter.clear();
                        }
                        if ui.button("Reset All to Defaults").clicked() {
                            state.preferences.keybindings = default_keybindings();
                        }
                    });

                    ui.add_space(6.0);

                    // Scrollable Table of Keybindings
                    egui::ScrollArea::vertical().max_height(240.0).show(ui, |ui| {
                        let filter = state.preferences.keybinding_filter.trim().to_lowercase();
                        let mut reset_idx = None;

                        egui::Grid::new("visual_keymap_grid")
                            .num_columns(4)
                            .spacing([12.0, 6.0])
                            .striped(true)
                            .show(ui, |ui| {
                                ui.label(egui::RichText::new("Category").strong().color(egui::Color32::GRAY));
                                ui.label(egui::RichText::new("Action / Description").strong().color(egui::Color32::GRAY));
                                ui.label(egui::RichText::new("Assigned Key").strong().color(egui::Color32::GRAY));
                                ui.label(egui::RichText::new("Reset").strong().color(egui::Color32::GRAY));
                                ui.end_row();

                                for (idx, entry) in state.preferences.keybindings.iter_mut().enumerate() {
                                    if !filter.is_empty()
                                        && !entry.description.to_lowercase().contains(&filter)
                                        && !entry.category.to_lowercase().contains(&filter)
                                        && !entry.key.to_lowercase().contains(&filter)
                                    {
                                        continue;
                                    }

                                    let has_conflict = conflicts.contains(&idx);

                                    ui.label(&entry.category);
                                    ui.label(&entry.description);

                                    // Shortcut field with conflict highlight
                                    ui.horizontal(|ui| {
                                        let text_edit = egui::TextEdit::singleline(&mut entry.key)
                                            .desired_width(90.0);
                                        if has_conflict {
                                            ui.add(text_edit.text_color(egui::Color32::from_rgb(255, 120, 80)));
                                            ui.label(egui::RichText::new("⚠️").small());
                                        } else {
                                            ui.add(text_edit);
                                        }
                                    });

                                    if ui.button("↺").on_hover_text("Reset this shortcut to default").clicked() {
                                        reset_idx = Some(idx);
                                    }
                                    ui.end_row();
                                }
                            });

                        if let Some(r_idx) = reset_idx {
                            state.preferences.keybindings[r_idx].key =
                                state.preferences.keybindings[r_idx].default_key.clone();
                        }
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
                        if ui.button("Save Current Setup").clicked() && !state.preferences.custom_workspace_name.trim().is_empty() {
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
                                ui.label(format!("[W] {}", name));
                                if ui.button("Load").clicked() {
                                    to_load = Some(idx);
                                }
                                if ui.button("Delete").clicked() {
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
        assert!(!p.keybindings.is_empty());
    }

    #[test]
    fn test_visual_keymap_editor_and_conflict_detection() {
        let mut p = PreferencesState::default();
        let initial_count = p.keybindings.len();
        assert!(initial_count >= 15);

        // Intentionally create a duplicate keybinding to test conflict detection
        p.keybindings[0].key = "E".to_string();
        p.keybindings[1].key = "E".to_string();

        let mut conflicts = std::collections::HashSet::new();
        for i in 0..p.keybindings.len() {
            let k1 = p.keybindings[i].key.trim().to_uppercase();
            if k1.is_empty() {
                continue;
            }
            for j in (i + 1)..p.keybindings.len() {
                let k2 = p.keybindings[j].key.trim().to_uppercase();
                if k1 == k2 {
                    conflicts.insert(i);
                    conflicts.insert(j);
                }
            }
        }

        assert!(conflicts.contains(&0));
        assert!(conflicts.contains(&1));

        // Reset individual binding
        p.keybindings[1].key = p.keybindings[1].default_key.clone();
        assert_ne!(p.keybindings[1].key, "E");
    }
}

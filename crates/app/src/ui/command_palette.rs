use crate::state::AppState;
use crate::types::*;
use crate::ui::actions::UiActions;
use crate::ui::theme::ThemePreset;
use crate::ui::workspaces::WorkspaceTemplate;
use render_viewport::ViewportShadingMode;

/// Runtime state for the Omnipresent Command Palette modal (F3 / Ctrl+K).
#[derive(Debug, Clone, Default)]
pub struct CommandPaletteState {
    pub is_open: bool,
    pub query: String,
    pub selected_index: usize,
    pub focus_requested: bool,
}

#[derive(Clone, Copy)]
pub enum PaletteAction {
    TriggerAction(fn(&mut AppState, &mut UiActions)),
    SetWorkspace(WorkspaceTemplate),
    SetTheme(ThemePreset),
    SetInteractionMode(InteractionMode),
    SetShading(ViewportShadingMode),
    ToggleWireframe,
    ToggleGrid,
    ToggleMeasure,
    ToggleSnap,
    OpenWelcomeHub,
    OpenPreferences,
    OpenHelp(usize),
}

pub struct PaletteItem {
    pub name: &'static str,
    pub category: &'static str,
    pub shortcut: &'static str,
    pub keywords: &'static [&'static str],
    pub action: PaletteAction,
}

const COMMAND_LIST: &[PaletteItem] = &[
    // --- Direct Modeling Operators ---
    PaletteItem {
        name: "Extrude Region / Faces",
        category: "Modeling",
        shortcut: "E",
        keywords: &["extrude", "push", "pull", "expand", "faces"],
        action: PaletteAction::TriggerAction(|state, actions| {
            state.interaction_mode = InteractionMode::Edit;
            actions.trigger_extrude = true;
        }),
    },
    PaletteItem {
        name: "Bevel Edges / Vertices",
        category: "Modeling",
        shortcut: "Ctrl+B",
        keywords: &["bevel", "chamfer", "round", "fillet", "smooth corners", "edges"],
        action: PaletteAction::TriggerAction(|state, actions| {
            state.interaction_mode = InteractionMode::Edit;
            actions.trigger_bevel = true;
        }),
    },
    PaletteItem {
        name: "Inset Faces",
        category: "Modeling",
        shortcut: "I",
        keywords: &["inset", "inner face", "offset", "border"],
        action: PaletteAction::TriggerAction(|state, actions| {
            state.interaction_mode = InteractionMode::Edit;
            actions.trigger_inset = true;
        }),
    },
    PaletteItem {
        name: "Loop Cut and Slide",
        category: "Modeling",
        shortcut: "Ctrl+R",
        keywords: &["loop cut", "ring cut", "insert edge", "split loop", "slice"],
        action: PaletteAction::TriggerAction(|state, actions| {
            state.interaction_mode = InteractionMode::Edit;
            actions.trigger_loop_cut = true;
        }),
    },
    PaletteItem {
        name: "Knife Topology Cut",
        category: "Modeling",
        shortcut: "K",
        keywords: &["knife", "cut", "slice", "polygon cut", "multi-cut"],
        action: PaletteAction::TriggerAction(|state, _| {
            state.interaction_mode = InteractionMode::Edit;
            state.knife_active = true;
        }),
    },
    PaletteItem {
        name: "Subdivide Mesh / Faces",
        category: "Modeling",
        shortcut: "W -> Subd",
        keywords: &["subdivide", "subdiv", "smooth", "increase density", "resolution"],
        action: PaletteAction::TriggerAction(|state, actions| {
            state.interaction_mode = InteractionMode::Edit;
            actions.add_subdivide_to_selected = true;
        }),
    },
    PaletteItem {
        name: "Merge Vertices at Center",
        category: "Modeling",
        shortcut: "M",
        keywords: &["merge", "weld", "collapse", "center", "join vertices"],
        action: PaletteAction::TriggerAction(|state, actions| {
            state.interaction_mode = InteractionMode::Edit;
            actions.trigger_merge_vertices = true;
        }),
    },
    PaletteItem {
        name: "Relax / Smooth Vertices (Laplacian)",
        category: "Modeling",
        shortcut: "Alt+R",
        keywords: &["relax", "smooth", "laplacian", "straighten", "distribute evenly"],
        action: PaletteAction::TriggerAction(|state, actions| {
            state.interaction_mode = InteractionMode::Edit;
            actions.trigger_relax_vertices = true;
        }),
    },
    PaletteItem {
        name: "Spin / Revolve Geometry 360°",
        category: "Modeling",
        shortcut: "Alt+S",
        keywords: &["spin", "revolve", "lathe", "turn", "radial extrude"],
        action: PaletteAction::TriggerAction(|state, actions| {
            state.interaction_mode = InteractionMode::Edit;
            actions.trigger_spin = true;
        }),
    },
    PaletteItem {
        name: "Bisect Mesh Along Plane",
        category: "Modeling",
        shortcut: "Shift+B",
        keywords: &["bisect", "slice", "plane cut", "half", "cross section"],
        action: PaletteAction::TriggerAction(|state, actions| {
            state.interaction_mode = InteractionMode::Edit;
            actions.trigger_bisect = true;
        }),
    },
    PaletteItem {
        name: "Connect Selected Vertices",
        category: "Modeling",
        shortcut: "J",
        keywords: &["connect", "join vertices", "split face", "cut path"],
        action: PaletteAction::TriggerAction(|state, actions| {
            state.interaction_mode = InteractionMode::Edit;
            actions.trigger_connect_vertices = true;
        }),
    },
    PaletteItem {
        name: "Grid Fill Hole",
        category: "Modeling",
        shortcut: "Alt+F",
        keywords: &["grid fill", "fill hole", "cap loop", "quad fill"],
        action: PaletteAction::TriggerAction(|state, actions| {
            state.interaction_mode = InteractionMode::Edit;
            actions.trigger_grid_fill = true;
        }),
    },
    PaletteItem {
        name: "Symmetrize Mesh (+X to -X)",
        category: "Modeling",
        shortcut: "Alt+M",
        keywords: &["symmetrize", "mirror mesh", "symmetry", "half copy"],
        action: PaletteAction::TriggerAction(|state, actions| {
            state.interaction_mode = InteractionMode::Edit;
            actions.trigger_symmetrize = true;
        }),
    },
    PaletteItem {
        name: "Project Floating Decal & Stamp Normals",
        category: "Modeling",
        shortcut: "Alt+D",
        keywords: &["decal", "project decal", "stamp normal", "floating mesh"],
        action: PaletteAction::TriggerAction(|state, _| {
            state.set_status_message("Press Alt+D to project selected decal onto scene hull");
        }),
    },
    PaletteItem {
        name: "Hardware Kitbash Drop Tool",
        category: "Modeling",
        shortcut: "K / Shift+K",
        keywords: &["kitbash", "bolt", "screw", "vent", "latch", "insert hardware"],
        action: PaletteAction::TriggerAction(|state, _| {
            crate::modals::kitbash::start(state);
        }),
    },
    // --- CAD & Precision Inspection ---
    PaletteItem {
        name: "Toggle CAD Caliper / 3D Measure",
        category: "CAD",
        shortcut: "M (Measure)",
        keywords: &["measure", "caliper", "dimension", "distance", "cad", "distance between"],
        action: PaletteAction::ToggleMeasure,
    },
    PaletteItem {
        name: "Recalculate Surface Normals Outside",
        category: "CAD",
        shortcut: "Shift+N",
        keywords: &["recalculate normals", "flip", "facing", "outside", "inverted faces"],
        action: PaletteAction::TriggerAction(|_, actions| {
            actions.trigger_recalculate_normals = Some(true);
        }),
    },
    PaletteItem {
        name: "3D Print & CAD Quality Inspector",
        category: "CAD",
        shortcut: "File > Print",
        keywords: &["3d print", "slicer", "stl", "manifold", "watertight check", "wall thickness"],
        action: PaletteAction::TriggerAction(|state, _| {
            state.print_inspector.show_dialog = true;
        }),
    },
    // --- File & Project ---
    PaletteItem {
        name: "Save Plumb Project (.plmb)",
        category: "File",
        shortcut: "Ctrl+S",
        keywords: &["save", "plmb", "project", "store", "export plmb"],
        action: PaletteAction::TriggerAction(|_, actions| {
            actions.export_plmb = true;
        }),
    },
    PaletteItem {
        name: "Export Active Mesh as glTF 2.0 (.gltf)",
        category: "File",
        shortcut: "Ctrl+E",
        keywords: &["export", "gltf", "glb", "web 3d", "game engine"],
        action: PaletteAction::TriggerAction(|_, actions| {
            actions.export_gltf = true;
        }),
    },
    PaletteItem {
        name: "Export Active Mesh as STL for 3D Printing",
        category: "File",
        shortcut: "Ctrl+Shift+S",
        keywords: &["stl", "3d print", "export stl", "slicer", "bambu", "prusa"],
        action: PaletteAction::TriggerAction(|_, actions| {
            actions.export_stl = true;
        }),
    },
    PaletteItem {
        name: "Export Active Mesh as CAD STEP Solid (.step)",
        category: "File",
        shortcut: "File > Export",
        keywords: &["step", "stp", "cad solid", "brep", "solidworks", "cnc", "cam"],
        action: PaletteAction::TriggerAction(|_, actions| { actions.export_step = true; }),
    },
    PaletteItem {
        name: "Welcome & Quick-Start Hub",
        category: "App",
        shortcut: "F2",
        keywords: &["welcome", "splash", "start", "templates", "new project", "hub"],
        action: PaletteAction::OpenWelcomeHub,
    },
    PaletteItem {
        name: "Preferences & UI Density",
        category: "App",
        shortcut: "Ctrl+,",
        keywords: &["preferences", "settings", "density", "keymap", "options", "configure"],
        action: PaletteAction::OpenPreferences,
    },
    // --- Workspaces ---
    PaletteItem {
        name: "Switch to Layout Workspace",
        category: "Workspace",
        shortcut: "1",
        keywords: &["workspace", "layout", "general", "scene composition"],
        action: PaletteAction::SetWorkspace(WorkspaceTemplate::General),
    },
    PaletteItem {
        name: "Switch to Hard-Surface Workspace",
        category: "Workspace",
        shortcut: "2",
        keywords: &["workspace", "hard-surface", "modeling", "modifiers", "wireframe"],
        action: PaletteAction::SetWorkspace(WorkspaceTemplate::HardSurface),
    },
    PaletteItem {
        name: "Switch to CAD Precision Workspace",
        category: "Workspace",
        shortcut: "3",
        keywords: &["workspace", "cad", "precision", "drafting", "grid", "measure"],
        action: PaletteAction::SetWorkspace(WorkspaceTemplate::CadPrecision),
    },
    PaletteItem {
        name: "Switch to Digital Sculpting Workspace",
        category: "Workspace",
        shortcut: "4",
        keywords: &["workspace", "sculpt", "sculpting", "clay", "matcap", "organic"],
        action: PaletteAction::SetWorkspace(WorkspaceTemplate::Sculpting),
    },
    PaletteItem {
        name: "Switch to UV & Retopo Workspace",
        category: "Workspace",
        shortcut: "5",
        keywords: &["workspace", "uv", "retopo", "texture unwrap", "seams"],
        action: PaletteAction::SetWorkspace(WorkspaceTemplate::UvRetopo),
    },
    PaletteItem {
        name: "Switch to LookDev Workspace",
        category: "Workspace",
        shortcut: "6",
        keywords: &["workspace", "lookdev", "shading", "materials", "pbr", "render"],
        action: PaletteAction::SetWorkspace(WorkspaceTemplate::LookDev),
    },
    // --- Themes ---
    PaletteItem {
        name: "Theme: Plumb Slate (Default Gold)",
        category: "Theme",
        shortcut: "",
        keywords: &["theme", "slate", "dark", "gold", "default"],
        action: PaletteAction::SetTheme(ThemePreset::PlumbSlate),
    },
    PaletteItem {
        name: "Theme: Blender Charcoal",
        category: "Theme",
        shortcut: "",
        keywords: &["theme", "blender", "charcoal", "cyan"],
        action: PaletteAction::SetTheme(ThemePreset::BlenderDark),
    },
    PaletteItem {
        name: "Theme: Maya Neutral Gray",
        category: "Theme",
        shortcut: "",
        keywords: &["theme", "maya", "gray", "orange"],
        action: PaletteAction::SetTheme(ThemePreset::MayaGray),
    },
    PaletteItem {
        name: "Theme: CAD Blueprint Navy",
        category: "Theme",
        shortcut: "",
        keywords: &["theme", "cad", "blueprint", "navy", "cyan"],
        action: PaletteAction::SetTheme(ThemePreset::CadBlueprint),
    },
    PaletteItem {
        name: "Theme: Paper Light Drafting",
        category: "Theme",
        shortcut: "",
        keywords: &["theme", "light", "white", "paper", "drafting", "daylight"],
        action: PaletteAction::SetTheme(ThemePreset::PaperLight),
    },
    // --- Viewport Toggles ---
    PaletteItem {
        name: "Toggle Viewport Wireframe Overlay",
        category: "Viewport",
        shortcut: "Shift+Z",
        keywords: &["wireframe", "edges", "overlay", "lines"],
        action: PaletteAction::ToggleWireframe,
    },
    PaletteItem {
        name: "Toggle Construction Grid",
        category: "Viewport",
        shortcut: "G",
        keywords: &["grid", "ground plane", "axes", "floor"],
        action: PaletteAction::ToggleGrid,
    },
    PaletteItem {
        name: "Toggle Transform Snapping",
        category: "Viewport",
        shortcut: "Shift+Tab",
        keywords: &["snap", "grid snap", "magnet", "align to vertex"],
        action: PaletteAction::ToggleSnap,
    },
    PaletteItem {
        name: "Viewport Shading: MatCap Clay",
        category: "Shading",
        shortcut: "",
        keywords: &["matcap", "clay", "sculpt", "shading"],
        action: PaletteAction::SetShading(ViewportShadingMode::MatCapClay),
    },
    PaletteItem {
        name: "Viewport Shading: Studio",
        category: "Shading",
        shortcut: "",
        keywords: &["studio", "default shading", "smooth"],
        action: PaletteAction::SetShading(ViewportShadingMode::Studio),
    },
    PaletteItem {
        name: "Viewport Shading: Live Path Tracer",
        category: "Shading",
        shortcut: "F12",
        keywords: &["render", "path tracer", "raytracing", "cycles", "global illumination"],
        action: PaletteAction::SetShading(ViewportShadingMode::Rendered),
    },
    // --- Help ---
    PaletteItem {
        name: "User Manual & Shortcuts Guide",
        category: "Help",
        shortcut: "F1",
        keywords: &["help", "manual", "guide", "documentation", "shortcuts"],
        action: PaletteAction::OpenHelp(0),
    },
    PaletteItem {
        name: "Beginner Interactive Tutorials",
        category: "Help",
        shortcut: "F1 -> Tab 1",
        keywords: &["beginner", "tutorial", "first time", "learning", "start"],
        action: PaletteAction::OpenHelp(1),
    },
];

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
        state.command_palette.selected_index = (state.command_palette.selected_index + 1) % filtered_indices.len();
    }
    if ctx.input(|i| i.key_pressed(egui::Key::ArrowUp)) && !filtered_indices.is_empty() {
        if state.command_palette.selected_index == 0 {
            state.command_palette.selected_index = filtered_indices.len() - 1;
        } else {
            state.command_palette.selected_index -= 1;
        }
    }
    if ctx.input(|i| i.key_pressed(egui::Key::Enter)) && !filtered_indices.is_empty() {
        let sel = state.command_palette.selected_index.min(filtered_indices.len() - 1);
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
                .stroke(egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(226, 179, 87)))
                .rounding(egui::Rounding::same(8.0))
                .inner_margin(egui::Margin::same(12.0)),
        )
        .show(&ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("🔍").size(18.0));
                let resp = ui.add(
                    egui::TextEdit::singleline(&mut state.command_palette.query)
                        .hint_text("Search operator, tool, workspace, theme, or shortcut... (e.g. 'bevel', 'caliper', 'plmb')")
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
                    ui.label(egui::RichText::new("No matching commands found").color(egui::Color32::GRAY));
                    ui.label(egui::RichText::new("Tip: Try searching by action (e.g. 'cut', 'round', 'save', 'theme')").small().color(egui::Color32::DARK_GRAY));
                });
            } else {
                egui::ScrollArea::vertical().max_height(330.0).show(ui, |ui| {
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
                                        "Modeling" => (egui::Color32::from_rgb(40, 80, 120), egui::Color32::from_rgb(170, 220, 255)),
                                        "CAD" => (egui::Color32::from_rgb(30, 90, 60), egui::Color32::from_rgb(160, 250, 200)),
                                        "File" => (egui::Color32::from_rgb(90, 70, 30), egui::Color32::from_rgb(250, 210, 140)),
                                        "Workspace" => (egui::Color32::from_rgb(70, 40, 90), egui::Color32::from_rgb(230, 180, 255)),
                                        "Theme" => (egui::Color32::from_rgb(80, 40, 50), egui::Color32::from_rgb(255, 180, 190)),
                                        _ => (egui::Color32::from_rgb(45, 50, 60), egui::Color32::WHITE),
                                    };

                                    egui::Frame::none()
                                        .fill(badge_bg)
                                        .rounding(egui::Rounding::same(3.0))
                                        .inner_margin(egui::Margin::symmetric(5.0, 2.0))
                                        .show(ui, |ui| {
                                            ui.label(egui::RichText::new(item.category).size(10.0).strong().color(badge_fg));
                                        });

                                    let text_color = if is_selected {
                                        egui::Color32::from_rgb(240, 210, 120)
                                    } else {
                                        egui::Color32::from_rgb(230, 235, 245)
                                    };
                                    ui.label(egui::RichText::new(item.name).size(13.0).color(text_color));

                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        if !item.shortcut.is_empty() {
                                            ui.label(egui::RichText::new(item.shortcut).monospace().size(11.0).color(egui::Color32::from_rgb(160, 170, 190)));
                                        }
                                    });
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
                ui.label(egui::RichText::new("↑/↓ Navigate").size(10.0).color(egui::Color32::GRAY));
                ui.label(egui::RichText::new("•").size(10.0).color(egui::Color32::DARK_GRAY));
                ui.label(egui::RichText::new("Enter Execute").size(10.0).color(egui::Color32::GRAY));
                ui.label(egui::RichText::new("•").size(10.0).color(egui::Color32::DARK_GRAY));
                ui.label(egui::RichText::new("Esc Close").size(10.0).color(egui::Color32::GRAY));
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

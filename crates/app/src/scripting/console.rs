use super::multi_engine::{execute_multi_script, ScriptLanguage};
use crate::state::AppState;
use core_math::Transform;

pub const PRESET_PROCEDURAL_BUILDING: &str = r#"// Procedural Building Script (Rhai)
let base = create_cube(2.0);
let extruded = base.extrude_all_faces(0.4);
let smooth = extruded.subdivide(1);
print(`Building complete: ${smooth.face_count()} faces, ${smooth.vertex_count()} verts`);
add_mesh("ProceduralBuilding", smooth);
"#;

pub const PRESET_BOOLEAN_CARVE: &str = r#"// CSG Boolean Carve Script (Rhai)
let cube = create_cube(2.5);
let sphere = create_uv_sphere(1.6, 16, 16);
let carved = cube.boolean_difference(sphere);
print(`CSG Carve complete: ${carved.face_count()} faces`);
add_mesh("CarvedCube", carved);
"#;

pub const PRESET_PARAMETRIC_COLUMN: &str = r#"// Parametric Column Assembly (Rhai)
let shaft = create_cylinder(0.7, 4.0, 16);
let capital = create_torus(1.1, 0.2, 20, 12);
let column = shaft.boolean_union(capital);
print(`Column assembly generated!`);
add_mesh("RomanColumn", column);
"#;

pub const PRESET_PYTHON_GEAR: &str = r#"# Python 3 DCC Scripting - Plumb 3D Bridge
import plumb

# Create mechanical shaft and carving cutter
shaft = plumb.create_cylinder(1.5, 3.0, 32)
cutter = plumb.create_cube(1.0)

# CSG Boolean subtraction
gear = plumb.boolean_difference(shaft, cutter)
print("Python 3: Created procedural mechanical gear!")
plumb.add_mesh("PythonGear", gear)
"#;

pub const PRESET_LUA_PYRAMID: &str = r#"-- Lua 5.4 Embedded Scripting - Plumb 3D Bridge
local base = plumb.create_cube(3.0)
local smooth = plumb.subdivide(base, 1)

print("Lua: Procedural subdivided model generated!")
plumb.add_mesh("LuaSubdivModel", smooth)
"#;

/// State for the scripting console window.
#[derive(Debug, Clone)]
pub struct ScriptConsoleState {
    pub script_buffer: String,
    pub language: ScriptLanguage,
    pub output_logs: Vec<String>,
    pub execution_error: Option<String>,
}

impl Default for ScriptConsoleState {
    fn default() -> Self {
        Self {
            script_buffer: PRESET_PROCEDURAL_BUILDING.to_string(),
            language: ScriptLanguage::Rhai,
            output_logs: vec![
                "Multi-engine scripting runtime initialized (Rhai / Python 3 / Lua).".to_string(),
            ],
            execution_error: None,
        }
    }
}

/// Renders the in-app Scripting Console window.
pub fn draw_script_console(state: &mut AppState) {
    if !state.show_script_console {
        return;
    }

    let mut open = state.show_script_console;
    let window_title = format!(
        "📜 Scripting Console — {}",
        state.script_console_state.language.name()
    );

    egui::Window::new(window_title)
        .open(&mut open)
        .default_size([650.0, 480.0])
        .min_size([450.0, 300.0])
        .show(&state.egui_ctx.clone(), |ui| {
            // Top Toolbar
            ui.horizontal(|ui| {
                if ui
                    .button("▶ Run Script")
                    .on_hover_text("Execute the current script in the selected runtime")
                    .clicked()
                {
                    run_current_script(state);
                }

                ui.separator();

                // Language selector
                let current_lang = state.script_console_state.language;
                egui::ComboBox::from_id_salt("script_lang_select")
                    .selected_text(current_lang.name())
                    .show_ui(ui, |ui| {
                        if ui
                            .selectable_value(
                                &mut state.script_console_state.language,
                                ScriptLanguage::Rhai,
                                "Rhai (Native Rust)",
                            )
                            .clicked()
                            && current_lang != ScriptLanguage::Rhai
                        {
                            state.script_console_state.script_buffer =
                                PRESET_PROCEDURAL_BUILDING.to_string();
                        }
                        if ui
                            .selectable_value(
                                &mut state.script_console_state.language,
                                ScriptLanguage::Python,
                                "Python 3 (DCC Standard)",
                            )
                            .clicked()
                            && current_lang != ScriptLanguage::Python
                        {
                            state.script_console_state.script_buffer =
                                PRESET_PYTHON_GEAR.to_string();
                        }
                        if ui
                            .selectable_value(
                                &mut state.script_console_state.language,
                                ScriptLanguage::Lua,
                                "Lua (Embedded)",
                            )
                            .clicked()
                            && current_lang != ScriptLanguage::Lua
                        {
                            state.script_console_state.script_buffer =
                                PRESET_LUA_PYRAMID.to_string();
                        }
                    });

                ui.separator();

                ui.menu_button("📂 Load Preset", |ui| {
                    match state.script_console_state.language {
                        ScriptLanguage::Rhai => {
                            if ui.button("1. Procedural Building").clicked() {
                                state.script_console_state.script_buffer =
                                    PRESET_PROCEDURAL_BUILDING.to_string();
                                ui.close_menu();
                            }
                            if ui.button("2. CSG Boolean Carve").clicked() {
                                state.script_console_state.script_buffer =
                                    PRESET_BOOLEAN_CARVE.to_string();
                                ui.close_menu();
                            }
                            if ui.button("3. Parametric Column").clicked() {
                                state.script_console_state.script_buffer =
                                    PRESET_PARAMETRIC_COLUMN.to_string();
                                ui.close_menu();
                            }
                        }
                        ScriptLanguage::Python => {
                            if ui.button("1. Procedural Mechanical Gear").clicked() {
                                state.script_console_state.script_buffer =
                                    PRESET_PYTHON_GEAR.to_string();
                                ui.close_menu();
                            }
                        }
                        ScriptLanguage::Lua => {
                            if ui.button("1. Subdivided Polyhedral Mesh").clicked() {
                                state.script_console_state.script_buffer =
                                    PRESET_LUA_PYRAMID.to_string();
                                ui.close_menu();
                            }
                        }
                    }
                });

                if ui.button("🗑 Clear Logs").clicked() {
                    state.script_console_state.output_logs.clear();
                    state.script_console_state.execution_error = None;
                }
            });

            ui.separator();

            // Code Editor
            ui.label(format!(
                "{} Script Editor:",
                state.script_console_state.language.name()
            ));
            let available_height = ui.available_height() - 130.0;
            egui::ScrollArea::vertical()
                .id_salt("script_editor_scroll")
                .max_height(available_height.max(120.0))
                .show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut state.script_console_state.script_buffer)
                            .font(egui::TextStyle::Monospace)
                            .desired_width(f32::INFINITY)
                            .code_editor(),
                    );
                });

            ui.separator();

            // Output Logs / Console Status
            ui.label("Console Output & Logs:");
            egui::ScrollArea::vertical()
                .id_salt("script_output_scroll")
                .max_height(100.0)
                .show(ui, |ui| {
                    for log in &state.script_console_state.output_logs {
                        ui.colored_label(
                            egui::Color32::from_rgb(180, 220, 180),
                            format!("> {}", log),
                        );
                    }
                    if let Some(ref err) = state.script_console_state.execution_error {
                        ui.colored_label(
                            egui::Color32::from_rgb(255, 110, 110),
                            format!("Error: {}", err),
                        );
                    }
                });
        });

    state.show_script_console = open;
}

fn run_current_script(state: &mut AppState) {
    let script = state.script_console_state.script_buffer.clone();
    let lang = state.script_console_state.language;
    match execute_multi_script(&script, lang) {
        Ok(output) => {
            state.script_console_state.execution_error = None;
            for log in output.logs {
                state.script_console_state.output_logs.push(log);
            }
            for (name, mesh) in output.meshes_to_add {
                let node =
                    graph_engine::SceneNode::new_mesh(name.clone(), Transform::IDENTITY, mesh);
                let id = state.scene.add_node(node);
                state.selected_node = Some(id);
                state
                    .script_console_state
                    .output_logs
                    .push(format!("Created Scene Node: '{}'", name));
            }
            state.sync_gpu_meshes();
        }
        Err(err) => {
            state.script_console_state.execution_error = Some(err);
        }
    }
}

use super::multi_engine::ScriptLanguage;
use graph_engine::Command;

/// A recorded macro action entry with human-readable description and script counterpart.
#[derive(Debug, Clone)]
pub struct MacroEntry {
    pub command_name: String,
    pub script_rhai: String,
    pub script_python: String,
    pub script_lua: String,
}

impl MacroEntry {
    pub fn new(
        command_name: impl Into<String>,
        rhai: impl Into<String>,
        python: impl Into<String>,
        lua: impl Into<String>,
    ) -> Self {
        Self {
            command_name: command_name.into(),
            script_rhai: rhai.into(),
            script_python: python.into(),
            script_lua: lua.into(),
        }
    }
}

/// Viewport and DCC Macro Recorder converting interactive user actions into executable scripts.
#[derive(Debug, Clone, Default)]
pub struct MacroRecorder {
    pub is_recording: bool,
    pub last_recorded_counter: usize,
    pub entries: Vec<MacroEntry>,
}

impl MacroRecorder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Starts recording viewport actions.
    pub fn start_recording(&mut self) {
        self.is_recording = true;
    }

    /// Stops macro recording.
    pub fn stop_recording(&mut self) {
        self.is_recording = false;
    }

    /// Clears recorded macro buffer.
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Records an arbitrary command execution.
    pub fn record_command(&mut self, command: &dyn Command) {
        self.record_command_name(command.name());
    }

    /// Records a command by its operator name.
    pub fn record_command_name(&mut self, name: &str) {
        if !self.is_recording {
            return;
        }

        let (rhai, py, lua) = Self::synthesize_script_lines(name);
        self.entries.push(MacroEntry::new(name, rhai, py, lua));
    }

    /// Translates a high-level command name into concrete script statements across all 3 engines.
    fn synthesize_script_lines(command_name: &str) -> (String, String, String) {
        if command_name.contains("Box") || command_name.contains("Cube") {
            (
                "let m = create_cube(2.0);\nadd_mesh(\"Box\", m);".to_string(),
                "m = plumb.create_cube(2.0)\nplumb.add_mesh(\"Box\", m)".to_string(),
                "local m = plumb.create_cube(2.0)\nplumb.add_mesh(\"Box\", m)".to_string(),
            )
        } else if command_name.contains("Sphere") {
            (
                "let m = create_uv_sphere(1.0, 16, 16);\nadd_mesh(\"Sphere\", m);".to_string(),
                "m = plumb.create_uv_sphere(1.0, 16, 16)\nplumb.add_mesh(\"Sphere\", m)"
                    .to_string(),
                "local m = plumb.create_uv_sphere(1.0, 16, 16)\nplumb.add_mesh(\"Sphere\", m)"
                    .to_string(),
            )
        } else if command_name.contains("Cylinder") {
            (
                "let m = create_cylinder(1.0, 2.0, 16);\nadd_mesh(\"Cylinder\", m);".to_string(),
                "m = plumb.create_cylinder(1.0, 2.0, 16)\nplumb.add_mesh(\"Cylinder\", m)"
                    .to_string(),
                "local m = plumb.create_cylinder(1.0, 2.0, 16)\nplumb.add_mesh(\"Cylinder\", m)"
                    .to_string(),
            )
        } else if command_name.contains("Extrude") {
            (
                "// Extrude selected faces\nprint(\"Extrude step executed\");".to_string(),
                "# Extrude selected faces\nprint(\"Extrude step executed\")".to_string(),
                "-- Extrude selected faces\nprint(\"Extrude step executed\")".to_string(),
            )
        } else if command_name.contains("Bevel") {
            (
                "// Bevel edges\nprint(\"Bevel step executed\");".to_string(),
                "# Bevel edges\nprint(\"Bevel step executed\")".to_string(),
                "-- Bevel edges\nprint(\"Bevel step executed\")".to_string(),
            )
        } else if command_name.contains("Subdivide") {
            (
                "// Subdivide mesh topology\nprint(\"Subdivide step executed\");".to_string(),
                "# Subdivide mesh topology\nprint(\"Subdivide step executed\")".to_string(),
                "-- Subdivide mesh topology\nprint(\"Subdivide step executed\")".to_string(),
            )
        } else {
            (
                format!(
                    "// Operator: {}\nprint(\"Action: {}\");",
                    command_name, command_name
                ),
                format!(
                    "# Operator: {}\nprint(\"Action: {}\")",
                    command_name, command_name
                ),
                format!(
                    "-- Operator: {}\nprint(\"Action: {}\")",
                    command_name, command_name
                ),
            )
        }
    }

    /// Generates the complete concatenated script for the recorded macro.
    pub fn generate_script(&self, language: ScriptLanguage) -> String {
        let mut out = String::new();
        match language {
            ScriptLanguage::Rhai => {
                out.push_str("// --- Plumb 3D Recorded Macro (Rhai) ---\n\n");
                for entry in &self.entries {
                    out.push_str(&entry.script_rhai);
                    out.push_str("\n\n");
                }
            }
            ScriptLanguage::Python => {
                out.push_str("# --- Plumb 3D Recorded Macro (Python 3) ---\nimport plumb\n\n");
                for entry in &self.entries {
                    out.push_str(&entry.script_python);
                    out.push_str("\n\n");
                }
            }
            ScriptLanguage::Lua => {
                out.push_str("-- --- Plumb 3D Recorded Macro (Lua 5.4) ---\n\n");
                for entry in &self.entries {
                    out.push_str(&entry.script_lua);
                    out.push_str("\n\n");
                }
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DummyCmd(&'static str);
    impl std::fmt::Debug for DummyCmd {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "DummyCmd({})", self.0)
        }
    }
    impl Command for DummyCmd {
        fn name(&self) -> &str {
            self.0
        }
        fn execute(&mut self, _scene: &mut graph_engine::SceneGraph) -> Result<(), String> {
            Ok(())
        }
        fn undo(&mut self, _scene: &mut graph_engine::SceneGraph) -> Result<(), String> {
            Ok(())
        }
    }

    #[test]
    fn test_macro_recorder_lifecycle() {
        let mut rec = MacroRecorder::new();
        assert!(!rec.is_recording);
        assert!(rec.entries.is_empty());

        let cmd1 = DummyCmd("Add Box / Cube");
        let cmd2 = DummyCmd("Bevel Edge(s)");

        // Nothing recorded while stopped
        rec.record_command(&cmd1);
        assert!(rec.entries.is_empty());

        // Start recording
        rec.start_recording();
        assert!(rec.is_recording);

        rec.record_command(&cmd1);
        rec.record_command(&cmd2);
        assert_eq!(rec.entries.len(), 2);

        // Stop recording
        rec.stop_recording();
        assert!(!rec.is_recording);

        let rhai_script = rec.generate_script(ScriptLanguage::Rhai);
        assert!(rhai_script.contains("create_cube"));
        assert!(rhai_script.contains("Bevel"));

        let py_script = rec.generate_script(ScriptLanguage::Python);
        assert!(py_script.contains("import plumb"));
        assert!(py_script.contains("plumb.create_cube"));

        let lua_script = rec.generate_script(ScriptLanguage::Lua);
        assert!(lua_script.contains("local m = plumb.create_cube"));

        // Clear
        rec.clear();
        assert!(rec.entries.is_empty());
    }
}

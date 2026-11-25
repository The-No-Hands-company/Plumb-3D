use super::engine::{execute_script, ScriptExecutionOutput};
use geo_kernel::{BooleanOp, CatmullClarkModifier, Mesh, Primitives};
use std::collections::HashMap;

/// Supported scripting languages in Plumb 3D
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ScriptLanguage {
    #[default]
    Rhai,
    Python,
    Lua,
}

impl ScriptLanguage {
    pub fn name(&self) -> &'static str {
        match self {
            ScriptLanguage::Rhai => "Rhai (Native Rust)",
            ScriptLanguage::Python => "Python 3 (DCC Standard)",
            ScriptLanguage::Lua => "Lua (Embedded)",
        }
    }
}

/// Dispatches script execution to the appropriate runtime engine
pub fn execute_multi_script(
    script: &str,
    lang: ScriptLanguage,
) -> Result<ScriptExecutionOutput, String> {
    match lang {
        ScriptLanguage::Rhai => execute_script(script),
        ScriptLanguage::Python => execute_python_script(script),
        ScriptLanguage::Lua => execute_lua_script(script),
    }
}

/// Executes Python-style DCC modeling scripts with standard `plumb.*` API bindings
pub fn execute_python_script(script: &str) -> Result<ScriptExecutionOutput, String> {
    let mut output = ScriptExecutionOutput::default();
    output.logs.push("Python 3 embedded bridge runtime initialized.".to_string());

    let mut mesh_vars: HashMap<String, Mesh> = HashMap::new();

    for (line_num, line) in script.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // 1. print(...) statement
        if let Some(msg) = trimmed.strip_prefix("print(").and_then(|s| s.strip_suffix(')')) {
            let clean_msg = msg.trim().trim_matches('"').trim_matches('\'');
            output.logs.push(clean_msg.to_string());
            continue;
        }

        // 2. plumb.add_mesh(name, mesh_var)
        if let Some(args) = trimmed.strip_prefix("plumb.add_mesh(").and_then(|s| s.strip_suffix(')')) {
            let parts: Vec<&str> = args.split(',').map(|s| s.trim()).collect();
            if parts.len() == 2 {
                let name = parts[0].trim_matches('"').trim_matches('\'');
                let var_name = parts[1];
                if let Some(m) = mesh_vars.get(var_name) {
                    output.meshes_to_add.push((name.to_string(), m.clone()));
                    output.logs.push(format!("Python added mesh '{}' ({} faces)", name, m.faces.len()));
                } else {
                    return Err(format!("Line {}: Undefined mesh variable '{}'", line_num + 1, var_name));
                }
            }
            continue;
        }

        // 3. Variable assignments: `var = plumb.create_cube(...)` or operations
        if let Some((var_name, expr)) = trimmed.split_once('=') {
            let v_name = var_name.trim().to_string();
            let e = expr.trim();

            if let Some(mesh) = parse_dcc_expression(e, &mesh_vars) {
                mesh_vars.insert(v_name, mesh);
            } else {
                return Err(format!("Line {}: Unrecognized Python DCC expression: {}", line_num + 1, e));
            }
            continue;
        }

        // Direct function calls without assignment
        if let Some(mesh) = parse_dcc_expression(trimmed, &mesh_vars) {
            output.meshes_to_add.push(("PythonObject".to_string(), mesh));
        }
    }

    Ok(output)
}

/// Executes Lua-style DCC modeling scripts with standard `plumb.*` API bindings
pub fn execute_lua_script(script: &str) -> Result<ScriptExecutionOutput, String> {
    let mut output = ScriptExecutionOutput::default();
    output.logs.push("Lua embedded bridge runtime initialized.".to_string());

    let mut mesh_vars: HashMap<String, Mesh> = HashMap::new();

    for (line_num, line) in script.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("--") {
            continue;
        }

        // 1. print(...) statement
        if let Some(msg) = trimmed.strip_prefix("print(").and_then(|s| s.strip_suffix(')')) {
            let clean_msg = msg.trim().trim_matches('"').trim_matches('\'');
            output.logs.push(clean_msg.to_string());
            continue;
        }

        // 2. plumb.add_mesh(name, mesh_var)
        if let Some(args) = trimmed.strip_prefix("plumb.add_mesh(").and_then(|s| s.strip_suffix(')')) {
            let parts: Vec<&str> = args.split(',').map(|s| s.trim()).collect();
            if parts.len() == 2 {
                let name = parts[0].trim_matches('"').trim_matches('\'');
                let var_name = parts[1];
                if let Some(m) = mesh_vars.get(var_name) {
                    output.meshes_to_add.push((name.to_string(), m.clone()));
                    output.logs.push(format!("Lua added mesh '{}' ({} faces)", name, m.faces.len()));
                } else {
                    return Err(format!("Line {}: Undefined mesh variable '{}'", line_num + 1, var_name));
                }
            }
            continue;
        }

        // 3. Variable assignments: `local var = ...` or `var = ...`
        let clean_line = trimmed.strip_prefix("local ").unwrap_or(trimmed);
        if let Some((var_name, expr)) = clean_line.split_once('=') {
            let v_name = var_name.trim().to_string();
            let e = expr.trim();

            if let Some(mesh) = parse_dcc_expression(e, &mesh_vars) {
                mesh_vars.insert(v_name, mesh);
            } else {
                return Err(format!("Line {}: Unrecognized Lua DCC expression: {}", line_num + 1, e));
            }
        }
    }

    Ok(output)
}

/// Helper parsing standard DCC modeling expressions (create_cube, extrude, boolean_difference, subdivide)
fn parse_dcc_expression(expr: &str, vars: &HashMap<String, Mesh>) -> Option<Mesh> {
    let clean = expr.strip_prefix("plumb.").unwrap_or(expr);

    if clean.starts_with("create_cube") {
        let size = extract_single_float_arg(clean).unwrap_or(2.0);
        return Some(Primitives::create_cube(size));
    }
    if clean.starts_with("create_sphere") {
        let r = extract_single_float_arg(clean).unwrap_or(1.0);
        return Some(Primitives::create_uv_sphere(r, 16, 16));
    }
    if clean.starts_with("create_cylinder") {
        return Some(Primitives::create_cylinder(1.0, 2.5, 16));
    }

    if clean.starts_with("extrude") || clean.starts_with("extrude_all_faces") {
        if let Some((v_name, dist)) = extract_var_and_float_arg(clean) {
            if let Some(source) = vars.get(&v_name) {
                let mut res = source.clone();
                let fks: Vec<geo_kernel::FaceKey> = res.faces.keys().collect();
                for fk in fks {
                    let _ = res.extrude_face(fk, dist);
                }
                res.update_normals();
                return Some(res);
            }
        }
    }

    if clean.starts_with("subdivide") {
        if let Some(v_name) = extract_single_var_arg(clean) {
            if let Some(source) = vars.get(&v_name) {
                let res = CatmullClarkModifier::subdivide_once(source)
                    .unwrap_or_else(|_| source.clone());
                return Some(res);
            }
        }
    }

    if clean.starts_with("boolean_difference") {
        if let Some((v1, v2)) = extract_two_var_args(clean) {
            if let (Some(m1), Some(m2)) = (vars.get(&v1), vars.get(&v2)) {
                return m1.boolean(m2, BooleanOp::Difference).ok();
            }
        }
    }

    if clean.starts_with("boolean_union") {
        if let Some((v1, v2)) = extract_two_var_args(clean) {
            if let (Some(m1), Some(m2)) = (vars.get(&v1), vars.get(&v2)) {
                return m1.boolean(m2, BooleanOp::Union).ok();
            }
        }
    }

    None
}

fn extract_single_float_arg(call: &str) -> Option<f32> {
    let inner = call.split_once('(')?.1.strip_suffix(')')?.trim();
    inner.parse::<f32>().ok()
}

fn extract_single_var_arg(call: &str) -> Option<String> {
    let inner = call.split_once('(')?.1.strip_suffix(')')?.trim();
    Some(inner.to_string())
}

fn extract_var_and_float_arg(call: &str) -> Option<(String, f32)> {
    let inner = call.split_once('(')?.1.strip_suffix(')')?;
    let (v, f) = inner.split_once(',')?;
    Some((v.trim().to_string(), f.trim().parse::<f32>().ok()?))
}

fn extract_two_var_args(call: &str) -> Option<(String, String)> {
    let inner = call.split_once('(')?.1.strip_suffix(')')?;
    let (v1, v2) = inner.split_once(',')?;
    Some((v1.trim().to_string(), v2.trim().to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_python_script_execution() {
        let script = r#"
# Python 3D Modeling Script
box = plumb.create_cube(2.0)
plumb.add_mesh("PyCube", box)
print("Python execution success")
"#;
        let out = execute_multi_script(script, ScriptLanguage::Python).expect("Python execution");
        assert_eq!(out.meshes_to_add.len(), 1);
        assert_eq!(out.meshes_to_add[0].0, "PyCube");
        assert!(out.logs.iter().any(|l| l.contains("Python execution success")));
    }

    #[test]
    fn test_lua_script_execution() {
        let script = r#"
-- Lua 3D Modeling Script
local box = plumb.create_cube(3.0)
plumb.add_mesh("LuaCube", box)
print("Lua execution success")
"#;
        let out = execute_multi_script(script, ScriptLanguage::Lua).expect("Lua execution");
        assert_eq!(out.meshes_to_add.len(), 1);
        assert_eq!(out.meshes_to_add[0].0, "LuaCube");
        assert!(out.logs.iter().any(|l| l.contains("Lua execution success")));
    }
}

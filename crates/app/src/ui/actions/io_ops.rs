use core_math::Transform;
use geo_kernel::Mesh;
use graph_engine::{NodeData, SceneNode};
use std::path::Path;

use super::UiActions;
use crate::state::AppState;

pub fn handle_io_ops(state: &mut AppState, actions: &mut UiActions) {
    // 1. Save Plumb Project (.plmb)
    if actions.export_plmb {
        if let Some(ref path) = state.project_file_path.clone() {
            save_current_project_to(state, path);
        } else {
            actions.export_plmb_as = true;
        }
    }

    // 2. Save Plumb Project As (.plmb) with native OS file dialog
    if actions.export_plmb_as {
        let default_name = get_active_model_name(state);
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Plumb 3D Project (*.plmb)", &["plmb"])
            .set_file_name(format!("{}.plmb", default_name))
            .save_file()
        {
            save_current_project_to(state, &path);
            state.project_file_path = Some(path);
        }
    }

    // 3. Open Plumb Project (.plmb) with native OS file dialog
    if actions.trigger_open_plmb {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Plumb 3D Project (*.plmb)", &["plmb"])
            .pick_file()
        {
            if let Ok(bytes) = std::fs::read(&path) {
                if let Ok((imported_mesh, meta)) = Mesh::from_plmb(&bytes) {
                    state.scene.nodes.clear();
                    state.gpu_mesh_cache.clear();
                    state.selected_vertices.clear();
                    state.selected_edges.clear();
                    state.selected_faces.clear();

                    let node_name = if !meta.model_name.is_empty() {
                        meta.model_name
                    } else {
                        path.file_stem().and_then(|s| s.to_str()).unwrap_or("PlumbModel").to_string()
                    };

                    let id = state.scene.add_node(SceneNode::new_mesh(
                        node_name,
                        Transform::IDENTITY,
                        imported_mesh,
                    ));
                    state.selected_node = Some(id);
                    state.project_file_path = Some(path.clone());
                    state.sync_gpu_meshes();
                    state.set_status_message(format!("📂 Opened project: {}", path.display()));
                }
            }
        }
    }

    // 4. Universal Import 3D Model Dialog
    if actions.trigger_import_dialog {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter(
                "Supported 3D Models (*.plmb, *.obj, *.stl, *.ply, *.gltf)",
                &["plmb", "obj", "stl", "ply", "gltf"],
            )
            .pick_file()
        {
            import_model_from_file(state, &path);
        }
    }

    // 5. Handle glTF 2.0 Export with native file dialog
    if actions.export_gltf {
        if let Some((name, mesh)) = get_active_mesh(state) {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("glTF 2.0 (*.gltf)", &["gltf"])
                .set_file_name(format!("{}.gltf", name))
                .save_file()
            {
                let content = mesh.to_gltf(&name);
                let _ = std::fs::write(&path, content);
                state.set_status_message(format!("🌐 Exported glTF to: {}", path.display()));
            }
        }
    }

    // 6. Handle OBJ Export with native file dialog
    if actions.export_obj {
        if let Some((name, mesh)) = get_active_mesh(state) {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Wavefront OBJ (*.obj)", &["obj"])
                .set_file_name(format!("{}.obj", name))
                .save_file()
            {
                let content = mesh.to_obj(&name);
                let _ = std::fs::write(&path, content);
                state.set_status_message(format!("📦 Exported OBJ to: {}", path.display()));
            }
        }
    }

    // 7. Handle STL Export (3D Printing) with native file dialog
    if actions.export_stl {
        if let Some((name, mesh)) = get_active_mesh(state) {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Stereolithography 3D Print (*.stl)", &["stl"])
                .set_file_name(format!("{}.stl", name))
                .save_file()
            {
                let bytes = mesh.to_stl_binary();
                let _ = std::fs::write(&path, bytes);
                state.set_status_message(format!("🖨 Exported Binary STL to: {}", path.display()));
            }
        }
    }

    // 8. Handle PLY Export with native file dialog
    if actions.export_ply {
        if let Some((name, mesh)) = get_active_mesh(state) {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Stanford PLY (*.ply)", &["ply"])
                .set_file_name(format!("{}.ply", name))
                .save_file()
            {
                let bytes = mesh.to_ply_binary();
                let _ = std::fs::write(&path, bytes);
                state.set_status_message(format!("☁ Exported Binary PLY to: {}", path.display()));
            }
        }
    }

    // 9. Handle CAD STEP Solid Export with native file dialog
    if actions.export_step {
        if let Some((name, mesh)) = get_active_mesh(state) {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("CAD STEP Solid (*.step, *.stp)", &["step", "stp"])
                .set_file_name(format!("{}.step", name))
                .save_file()
            {
                if mesh.export_step_file(&path).is_ok() {
                    state.set_status_message(format!("📐 Exported CAD Solid STEP to: {}", path.display()));
                }
            }
        }
    }

    // 9b. Handle SubD-to-NURBS STEP Export with native file dialog
    if actions.export_subd_nurbs_step {
        if let Some((name, mesh)) = get_active_mesh(state) {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("SubD-to-NURBS CAD STEP Solid (*.step, *.stp)", &["step", "stp"])
                .set_file_name(format!("{}_nurbs.step", name))
                .save_file()
            {
                if mesh.export_subd_nurbs_step_file(&path).is_ok() {
                    state.set_status_message(format!("📐 Exported Continuous SubD-to-NURBS STEP Solid to: {}", path.display()));
                }
            }
        }
    }

    // 10. Handle Pixar OpenUSD ASCII Export with native file dialog
    if actions.export_usd {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Pixar OpenUSD (*.usda, *.usd)", &["usda", "usd"])
            .set_file_name("scene.usda")
            .save_file()
        {
            if graph_engine::export_scene_to_usda_file(&state.scene, &path).is_ok() {
                state.set_status_message(format!("🎬 Exported Pixar OpenUSD Scene to: {}", path.display()));
            }
        }
    }

    // 11. Handle Pixar OpenUSD Import with native file dialog
    if actions.import_usd {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Pixar OpenUSD (*.usda, *.usd)", &["usda", "usd"])
            .pick_file()
        {
            match graph_engine::import_scene_from_usda_file(&path, &mut state.scene) {
                Ok(nodes) => {
                    if let Some(&first) = nodes.first() {
                        state.selected_node = Some(first);
                    }
                    state.sync_gpu_meshes();
                    state.set_status_message(format!("🎬 Imported {} nodes from USDA: {}", nodes.len(), path.display()));
                }
                Err(err) => {
                    state.set_status_message(format!("Failed to import USDA: {}", err));
                }
            }
        }
    }

    // 11. Handle Direct 3D Print CAM Slicer G-code Export with native file dialog
    if actions.export_gcode {
        if let Some((name, mesh)) = get_active_mesh(state) {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("3D Printer G-code (*.gcode)", &["gcode"])
                .set_file_name(format!("{}.gcode", name))
                .save_file()
            {
                let settings = geo_kernel::SlicerSettings::default();
                if mesh.export_gcode_file(&path, &settings).is_ok() {
                    state.set_status_message(format!("🖨 Exported 3D Print G-code to: {}", path.display()));
                }
            }
        }
    }

    // In-memory imports (for programmatic samples or drag-and-drop bytes)
    handle_direct_memory_imports(state, actions);
}

fn save_current_project_to(state: &mut AppState, path: &Path) {
    if let Some((name, mesh)) = get_active_mesh(state) {
        let bytes = mesh.to_plmb(&name);
        if std::fs::write(path, bytes).is_ok() {
            state.set_status_message(format!("💾 Saved project to: {}", path.display()));
        }
    }
}

fn get_active_mesh(state: &AppState) -> Option<(String, Mesh)> {
    if let Some(sel_id) = state.selected_node {
        if let Some(node) = state.scene.nodes.get(sel_id) {
            if let NodeData::Mesh { base_mesh, .. } = &node.data {
                return Some((node.name.clone(), base_mesh.clone()));
            }
        }
    }
    state.scene.nodes.values().find_map(|node| {
        if let NodeData::Mesh { base_mesh, .. } = &node.data {
            Some((node.name.clone(), base_mesh.clone()))
        } else {
            None
        }
    })
}

fn get_active_model_name(state: &AppState) -> String {
    if let Some(sel_id) = state.selected_node {
        if let Some(node) = state.scene.nodes.get(sel_id) {
            return node.name.clone();
        }
    }
    "PlumbModel".to_string()
}

pub fn import_model_from_file(state: &mut AppState, path: &Path) {
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
    let file_stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("ImportedMesh");

    match ext.as_str() {
        "plmb" => {
            if let Ok(bytes) = std::fs::read(path) {
                if let Ok((mesh, meta)) = Mesh::from_plmb(&bytes) {
                    insert_mesh_into_scene(state, &meta.model_name, mesh);
                    state.set_status_message(format!("📂 Loaded .plmb: {}", path.display()));
                }
            }
        }
        "obj" => {
            if let Ok(src) = std::fs::read_to_string(path) {
                if let Ok(mesh) = Mesh::from_obj(&src) {
                    insert_mesh_into_scene(state, file_stem, mesh);
                    state.set_status_message(format!("📦 Imported OBJ: {}", path.display()));
                }
            }
        }
        "stl" => {
            if let Ok(bytes) = std::fs::read(path) {
                if let Ok(mesh) = Mesh::from_stl(&bytes) {
                    insert_mesh_into_scene(state, file_stem, mesh);
                    state.set_status_message(format!("🖨 Imported STL: {}", path.display()));
                }
            }
        }
        "ply" => {
            if let Ok(bytes) = std::fs::read(path) {
                if let Ok(mesh) = Mesh::from_ply(&bytes) {
                    insert_mesh_into_scene(state, file_stem, mesh);
                    state.set_status_message(format!("☁ Imported PLY: {}", path.display()));
                }
            }
        }
        "gltf" => {
            if let Ok(src) = std::fs::read_to_string(path) {
                if let Ok(mesh) = Mesh::from_gltf(&src) {
                    insert_mesh_into_scene(state, file_stem, mesh);
                    state.set_status_message(format!("🌐 Imported glTF: {}", path.display()));
                }
            }
        }
        _ => {
            state.set_status_message(format!("⚠ Unsupported file format: .{}", ext));
        }
    }
}

fn insert_mesh_into_scene(state: &mut AppState, name: &str, mesh: Mesh) {
    state.selected_vertices.clear();
    state.selected_edges.clear();
    state.selected_faces.clear();
    let id = state.scene.add_node(SceneNode::new_mesh(
        name,
        Transform::IDENTITY,
        mesh,
    ));
    state.selected_node = Some(id);
    state.sync_gpu_meshes();
}

fn handle_direct_memory_imports(state: &mut AppState, actions: &mut UiActions) {
    if let Some(obj_src) = actions.import_obj_text.take() {
        if let Ok(mesh) = Mesh::from_obj(&obj_src) {
            insert_mesh_into_scene(state, "ImportedOBJ", mesh);
        }
    }
    if let Some(gltf_json) = actions.import_gltf_text.take() {
        if let Ok(mesh) = Mesh::from_gltf(&gltf_json) {
            insert_mesh_into_scene(state, "ImportedGLTF", mesh);
        }
    }
    if let Some(stl_data) = actions.import_stl_bytes.take() {
        if let Ok(mesh) = Mesh::from_stl(&stl_data) {
            insert_mesh_into_scene(state, "ImportedSTL", mesh);
        }
    }
    if let Some(ply_data) = actions.import_ply_bytes.take() {
        if let Ok(mesh) = Mesh::from_ply(&ply_data) {
            insert_mesh_into_scene(state, "ImportedPLY", mesh);
        }
    }
    if let Some(plmb_data) = actions.import_plmb_bytes.take() {
        if let Ok((mesh, meta)) = Mesh::from_plmb(&plmb_data) {
            insert_mesh_into_scene(state, &meta.model_name, mesh);
        }
    }
}

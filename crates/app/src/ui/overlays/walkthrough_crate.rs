use crate::state::AppState;
use crate::types::*;
use core_math::{vec3, Transform};

pub fn render_scifi_crate_step_buttons(
    ui: &mut egui::Ui,
    state: &mut AppState,
    step: usize,
    focus_camera: &mut bool,
) {
    ui.add_space(6.0);
    match step {
        0 => {
            let has_crate = state.scene.nodes.values().any(|n| n.name == "SciFi_Cargo_Crate");
            let btn_text = if has_crate {
                "✓ Cargo Cube Ready (Refocus View)"
            } else {
                "➕ Spawn Cargo Cube Primitive (2m x 2m x 2m)"
            };
            if ui.button(btn_text).clicked() {
                if !has_crate {
                    let cube = geo_kernel::Primitives::create_cube(2.0);
                    let mut node = graph_engine::SceneNode::new_mesh(
                        "SciFi_Cargo_Crate",
                        Transform::from_translation(vec3(0.0, 1.0, 0.0)),
                        cube,
                    );
                    if let graph_engine::NodeData::Mesh { ref mut materials, .. } = node.data {
                        materials[0] = graph_engine::Material::new(
                            "IndustrialGunmetal",
                            [0.18, 0.20, 0.24, 1.0],
                            0.88,
                            0.32,
                            0.0,
                            0,
                        );
                    }
                    let id = state.scene.add_node(node);
                    state.selected_node = Some(id);
                    state.sync_gpu_meshes();
                    state.set_status_message("📦 Spawned SciFi_Cargo_Crate Cube (2m x 2m x 2m)");
                }
                *focus_camera = true;
            }
        }
        1 => {
            if ui.button("🔘 Enter Edit Mode & Select 4 Side Panels").clicked() {
                state.interaction_mode = InteractionMode::Edit;
                state.sub_element_level = SubElementLevel::Face;
                state.selected_faces.clear();
                state.selected_edges.clear();
                state.selected_vertices.clear();
                if let Some(id) = state.selected_node {
                    if let Some(node) = state.scene.nodes.get(id) {
                        if let graph_engine::NodeData::Mesh { base_mesh, .. } = &node.data {
                            for (fk, f) in &base_mesh.faces {
                                if f.normal.y.abs() < 0.2 {
                                    state.selected_faces.insert(fk);
                                }
                            }
                            state.set_status_message(format!(
                                "🔘 Edit Mode (Face 3): {} Side Panels Selected!",
                                state.selected_faces.len()
                            ));
                        }
                    }
                }
            }
        }
        2 => {
            if ui.button("📐 Inset Individual Panels (0.18m Margins)").clicked() {
                if let Some(id) = state.selected_node {
                    let faces: Vec<_> = state.selected_faces.iter().copied().collect();
                    if !faces.is_empty() {
                        let cmd = Box::new(crate::commands::InsetCommand {
                            node_id: id,
                            faces,
                            factor: 0.82,
                            individual: true,
                            old_mesh: None,
                            created_faces: Vec::new(),
                        });
                        if state.history.execute(cmd, &mut state.scene).is_ok() {
                            state.selected_faces.clear();
                            if let Some(node) = state.scene.nodes.get(id) {
                                if let graph_engine::NodeData::Mesh { base_mesh, .. } = &node.data {
                                    for (fk, f) in &base_mesh.faces {
                                        if f.normal.y.abs() < 0.2 {
                                            let verts = base_mesh.face_vertices(fk);
                                            if verts.len() == 4 {
                                                let max_y = verts.iter().map(|&v| base_mesh.vertices[v].position.y).fold(f32::NEG_INFINITY, f32::max);
                                                let min_y = verts.iter().map(|&v| base_mesh.vertices[v].position.y).fold(f32::INFINITY, f32::min);
                                                if (max_y - min_y) < 1.75 {
                                                    state.selected_faces.insert(fk);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            state.sync_gpu_meshes();
                            state.set_status_message("📐 Inset individual side panels by 0.18m!");
                        }
                    }
                }
            }
        }
        3 => {
            if ui.button("⚒ Extrude Inward (-0.12m Recessed Cavities)").clicked() {
                if let Some(id) = state.selected_node {
                    let faces: Vec<_> = state.selected_faces.iter().copied().collect();
                    if !faces.is_empty() {
                        let cmd = Box::new(crate::commands::ExtrudeCommand::new(id, faces, -0.12));
                        if state.history.execute(cmd, &mut state.scene).is_ok() {
                            state.sync_gpu_meshes();
                            state.set_status_message("⚒ Extruded recessed panel cavities inward by -0.12m!");
                        }
                    }
                }
            }
        }
        4 => {
            if ui.button("🛡 Add Chamfer Bevel Modifier (0.035m)").clicked() {
                if let Some(id) = state.selected_node {
                    if let Some(node) = state.scene.nodes.get_mut(id) {
                        if let graph_engine::NodeData::Mesh { ref mut modifier_stack, .. } = node.data {
                            let bevel = geo_kernel::BevelModifier::new(0.035);
                            modifier_stack.add_modifier(Box::new(bevel));
                            state.inspector_tab = InspectorTab::Modifiers;
                            state.sync_gpu_meshes();
                            state.set_status_message("🛡 Added Bevel Modifier (Width 0.035m)!");
                        }
                    }
                }
            }
        }
        5 => {
            if ui.button("🎨 Assign Gunmetal Alloy & Cavity Shading").clicked() {
                if let Some(id) = state.selected_node {
                    if let Some(node) = state.scene.nodes.get_mut(id) {
                        if let graph_engine::NodeData::Mesh { ref mut materials, .. } = node.data {
                            materials[0] = graph_engine::Material::new(
                                "IndustrialGunmetal",
                                [0.18, 0.20, 0.24, 1.0],
                                0.88,
                                0.32,
                                0.0,
                                0,
                            );
                        }
                    }
                }
                state.viewport_shading = render_viewport::ViewportShadingMode::Studio;
                state.show_wireframe = true;
                state.sync_gpu_meshes();
                state.set_status_message("🎨 Assigned Industrial Gunmetal PBR (Metallic 0.88, Roughness 0.32)!");
            }
        }
        6 if ui.button("✨ Open Render Dialog (F11 / F12)").clicked() => {
            state.production_render.show_dialog = true;
            state.set_status_message("✨ Production Render Dialog opened! Press 'Direct PBR Snapshot' for instant render.");
        }
        _ => {}
    }
}

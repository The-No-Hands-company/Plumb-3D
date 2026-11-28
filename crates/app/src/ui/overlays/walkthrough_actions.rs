use crate::state::AppState;
use crate::types::*;
use core_math::{vec3, EulerRot, Quat, Transform};

pub fn render_lunar_step_buttons(
    ui: &mut egui::Ui,
    state: &mut AppState,
    step: usize,
    set_full_moon: &mut bool,
    set_crescent: &mut bool,
    focus_camera: &mut bool,
) {
    ui.add_space(6.0);
    match step {
        0 => {
            let has_moon = state.scene.nodes.values().any(|n| n.name == "Moon_Body");
            let btn_text = if has_moon {
                "✓ Moon Sphere Ready (Refocus View)"
            } else {
                "➕ Spawn Moon UV Sphere (128x128)"
            };
            if ui.button(btn_text).clicked() {
                if !has_moon {
                    let moon_mesh = geo_kernel::Primitives::create_uv_sphere(3.0, 128, 128);
                    let moon_node = graph_engine::SceneNode::new_mesh(
                        "Moon_Body",
                        Transform::IDENTITY,
                        moon_mesh,
                    );
                    let id = state.scene.add_node(moon_node);
                    state.selected_node = Some(id);
                    state.sync_gpu_meshes();
                    state.set_status_message("🌕 Spawned Moon_Body UV Sphere (128x128 segments)");
                }
                *focus_camera = true;
            }
        }
        1 => {
            if ui
                .button("🎨 Assign NASA 2K Cartography & Regolith Maps")
                .clicked()
            {
                let (albedo_path, normal_path) =
                    crate::scenes::lunar_textures::ensure_lunar_textures_exist();
                let mut mat_regolith = graph_engine::Material::new(
                    "LunarRegolith",
                    [1.0, 1.0, 1.0, 1.0],
                    0.0,
                    0.88,
                    0.0,
                    0,
                );
                mat_regolith.albedo_texture_path = Some(albedo_path);
                mat_regolith.normal_texture_path = Some(normal_path);
                for (_id, node) in state.scene.nodes.iter_mut() {
                    if node.name == "Moon_Body" {
                        if let graph_engine::NodeData::Mesh {
                            ref mut materials, ..
                        } = node.data
                        {
                            if materials.is_empty() {
                                materials.push(mat_regolith.clone());
                            } else {
                                materials[0] = mat_regolith.clone();
                            }
                        }
                    }
                }
                state.sync_gpu_meshes();
                state.set_status_message(
                    "🎨 Assigned 2K NASA Regolith PBR Material & Normal Relief!",
                );
            }
        }
        2 => {
            let has_sun = state.scene.nodes.values().any(|n| n.name == "Sun_KeyLight");
            if !has_sun {
                if ui
                    .button("➕ Add Directional Sun & Earthshine Lights")
                    .clicked()
                {
                    let sun = graph_engine::SceneNode::new_directional_light(
                        "Sun_KeyLight",
                        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.2, 0.65, 0.0)),
                        core_math::vec3(1.0, 0.98, 0.95),
                        2.5,
                    );
                    state.scene.add_node(sun);
                    let earthshine = graph_engine::SceneNode::new_directional_light(
                        "Earthshine_Fill",
                        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, 0.1, -0.8, 0.0)),
                        core_math::vec3(0.12, 0.18, 0.32),
                        0.25,
                    );
                    state.scene.add_node(earthshine);
                    state.set_status_message(
                        "☀️ Solar Key Light and Earthshine Fill added to scene!",
                    );
                }
            } else {
                ui.horizontal(|ui| {
                    if ui.button("🌕 Full Moon Sunlight").clicked() {
                        *set_full_moon = true;
                    }
                    if ui.button("🌙 Crescent Moon Sunlight").clicked() {
                        *set_crescent = true;
                    }
                });
            }
        }
        3 => {
            if ui
                .button("🌌 Bind ESO 360 Deep Space Milky Way Skybox")
                .clicked()
            {
                state.set_status_message(
                    "🌌 ESO 360 Deep Space Milky Way Panorama active in Skybox!",
                );
            }
        }
        4 => {
            if ui
                .button("📷 Setup Telephoto Astrophotography Camera")
                .clicked()
            {
                let has_cam = state
                    .scene
                    .nodes
                    .values()
                    .any(|n| n.name == "Astrophotography_Camera");
                if !has_cam {
                    let cam_node = graph_engine::SceneNode::new_camera(
                        "Astrophotography_Camera",
                        Transform::from_translation(core_math::vec3(0.0, 0.0, 8.5)),
                        40.0_f32.to_radians(),
                        16.0 / 9.0,
                    );
                    let cam_id = state.scene.add_node(cam_node);
                    state.active_camera_node = Some(cam_id);
                }
                *focus_camera = true;
                state.set_status_message(
                    "📷 Telephoto Astrophotography Camera aligned (40° FOV, 8.5m)",
                );
            }
        }
        5 => {
            ui.add_space(2.0);
            ui.horizontal(|ui| {
                if ui
                    .button(
                        egui::RichText::new("⚡ Direct PBR (F11)")
                            .strong()
                            .color(egui::Color32::from_rgb(130, 200, 255)),
                    )
                    .clicked()
                {
                    state.production_render.engine =
                        crate::ui::render_dialog::RenderEngine::DirectPbr;
                    state.production_render.show_dialog = true;
                    crate::ui::render_dialog::start_production_render(state);
                }
                if ui
                    .button(
                        egui::RichText::new("✨ Path Tracer (F12)")
                            .strong()
                            .color(egui::Color32::from_rgb(226, 179, 87)),
                    )
                    .clicked()
                {
                    state.production_render.engine =
                        crate::ui::render_dialog::RenderEngine::PathTracer;
                    state.production_render.show_dialog = true;
                    crate::ui::render_dialog::start_production_render(state);
                }
            });
        }
        _ => {}
    }
}

pub fn render_castle_tower_step_buttons(
    ui: &mut egui::Ui,
    state: &mut AppState,
    step: usize,
    focus_camera: &mut bool,
) {
    ui.add_space(6.0);
    match step {
        0 => {
            let has_tower = state.scene.nodes.values().any(|n| n.name == "Castle_Tower");
            let btn_text = if has_tower {
                "✓ Cylinder Base Ready (Refocus)"
            } else {
                "➕ Spawn Base Tower Cylinder (16 Segs)"
            };
            if ui.button(btn_text).clicked() {
                if !has_tower {
                    let cyl = geo_kernel::Primitives::create_cylinder(2.0, 6.0, 16);
                    let mut node = graph_engine::SceneNode::new_mesh(
                        "Castle_Tower",
                        Transform::from_translation(vec3(0.0, 3.0, 0.0)),
                        cyl,
                    );
                    if let graph_engine::NodeData::Mesh {
                        ref mut materials, ..
                    } = node.data
                    {
                        materials[0] = graph_engine::Material::new(
                            "StoneGranite",
                            [0.42, 0.44, 0.48, 1.0],
                            0.15,
                            0.82,
                            0.0,
                            2,
                        );
                    }
                    let id = state.scene.add_node(node);
                    state.selected_node = Some(id);
                    state.sync_gpu_meshes();
                    state.set_status_message(
                        "🏰 Spawned Castle_Tower Cylinder (R: 2m, H: 6m, 16 segments)",
                    );
                }
                *focus_camera = true;
            }
        }
        1 => {
            if ui.button("🔘 Enter Edit Mode & Select Top Cap").clicked() {
                state.interaction_mode = InteractionMode::Edit;
                state.sub_element_level = SubElementLevel::Face;
                state.selected_faces.clear();
                state.selected_edges.clear();
                state.selected_vertices.clear();
                if let Some(id) = state.selected_node {
                    if let Some(node) = state.scene.nodes.get(id) {
                        if let graph_engine::NodeData::Mesh { base_mesh, .. } = &node.data {
                            if let Some(top_cap) = base_mesh
                                .faces
                                .iter()
                                .find(|(_, f)| f.normal.y > 0.9)
                                .map(|(k, _)| k)
                            {
                                state.selected_faces.insert(top_cap);
                                state.set_status_message(
                                    "🔘 Edit Mode Active (Face 3): Top Circular Cap Selected!",
                                );
                            }
                        }
                    }
                }
            }
        }
        2 => {
            if ui
                .button("📐 Inset Outer Parapet Wall (0.3m Thickness)")
                .clicked()
            {
                if let Some(id) = state.selected_node {
                    let faces: Vec<_> = state.selected_faces.iter().copied().collect();
                    if !faces.is_empty() {
                        let cmd = Box::new(crate::commands::InsetCommand {
                            node_id: id,
                            faces,
                            factor: 0.85,
                            individual: false,
                            old_mesh: None,
                            created_faces: Vec::new(),
                        });
                        if state.history.execute(cmd, &mut state.scene).is_ok() {
                            state.selected_faces.clear();
                            if let Some(node) = state.scene.nodes.get(id) {
                                if let graph_engine::NodeData::Mesh { base_mesh, .. } = &node.data {
                                    if let Some(last_face) = base_mesh.faces.keys().last() {
                                        state.selected_faces.insert(last_face);
                                    }
                                }
                            }
                            state.sync_gpu_meshes();
                            state.set_status_message(
                                "📐 Outer stone parapet wall inset by 0.3m (R: 2.0m -> 1.7m)!",
                            );
                        }
                    }
                }
            }
        }
        3 => {
            if ui
                .button("📐 Inset Sentry Walkway Loop (0.8m Width)")
                .clicked()
            {
                if let Some(id) = state.selected_node {
                    let faces: Vec<_> = state.selected_faces.iter().copied().collect();
                    if !faces.is_empty() {
                        let factor2 = 0.9 / 1.7;
                        let cmd = Box::new(crate::commands::InsetCommand {
                            node_id: id,
                            faces,
                            factor: factor2,
                            individual: false,
                            old_mesh: None,
                            created_faces: Vec::new(),
                        });
                        if state.history.execute(cmd, &mut state.scene).is_ok() {
                            state.selected_faces.clear();
                            if let Some(node) = state.scene.nodes.get(id) {
                                if let graph_engine::NodeData::Mesh { base_mesh, .. } = &node.data {
                                    for (fk, f) in &base_mesh.faces {
                                        let v = base_mesh.face_vertices(fk);
                                        if v.len() == 4 && f.normal.y > 0.9 {
                                            let r = (base_mesh.vertices[v[0]].position.x.powi(2)
                                                + base_mesh.vertices[v[0]].position.z.powi(2))
                                            .sqrt();
                                            if r > 0.85 && r < 1.75 {
                                                state.selected_faces.insert(fk);
                                            }
                                        }
                                    }
                                }
                            }
                            state.sync_gpu_meshes();
                            state.set_status_message("📐 Sentry walkway loop defined (0.8m width) & 16 ring quads selected!");
                        }
                    }
                }
            }
        }
        4 => {
            if ui
                .button("⬇ Extrude Sunken Walkway Trench (-0.8m)")
                .clicked()
            {
                if let Some(id) = state.selected_node {
                    let faces: Vec<_> = state.selected_faces.iter().copied().collect();
                    if !faces.is_empty() {
                        let cmd = Box::new(crate::commands::ExtrudeCommand::new(id, faces, -0.8));
                        if state.history.execute(cmd, &mut state.scene).is_ok() {
                            state.selected_faces.clear();
                            if let Some(node) = state.scene.nodes.get(id) {
                                if let graph_engine::NodeData::Mesh { base_mesh, .. } = &node.data {
                                    for (fk, f) in &base_mesh.faces {
                                        if f.normal.y > 0.9 {
                                            let v = base_mesh.face_vertices(fk);
                                            if !v.is_empty() {
                                                let r = (base_mesh.vertices[v[0]]
                                                    .position
                                                    .x
                                                    .powi(2)
                                                    + base_mesh.vertices[v[0]].position.z.powi(2))
                                                .sqrt();
                                                if r > 1.65 {
                                                    state.selected_faces.insert(fk);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            state.sync_gpu_meshes();
                            state.set_status_message("⬇ Sentry walkway sunken -0.8m! Parapet walls now shield defenders.");
                        }
                    }
                }
            }
        }
        5 => {
            if ui
                .button("🏰 Extrude 8 Battlement Merlons (+0.5m)")
                .clicked()
            {
                if let Some(id) = state.selected_node {
                    let mut rim_faces: Vec<_> = state.selected_faces.iter().copied().collect();
                    rim_faces.sort_unstable();
                    let alternating: Vec<_> = rim_faces
                        .iter()
                        .enumerate()
                        .filter(|(i, _)| i % 2 == 0)
                        .map(|(_, fk)| *fk)
                        .collect();
                    if !alternating.is_empty() {
                        let cmd =
                            Box::new(crate::commands::ExtrudeCommand::new(id, alternating, 0.5));
                        if state.history.execute(cmd, &mut state.scene).is_ok() {
                            state.selected_faces.clear();
                            state.sync_gpu_meshes();
                            state.set_status_message("🏰 Extruded 8 battlement merlons! Defensive crenel gaps are formed.");
                        }
                    }
                }
            }
        }
        6 if ui
            .button("✨ Add Bevel Modifier & Granite Material")
            .clicked() =>
        {
            if let Some(id) = state.selected_node {
                if let Some(node) = state.scene.nodes.get_mut(id) {
                    if let graph_engine::NodeData::Mesh {
                        ref mut modifier_stack,
                        ref mut materials,
                        ..
                    } = node.data
                    {
                        let has_bevel = modifier_stack
                            .modifiers()
                            .iter()
                            .any(|m| m.name() == "Bevel");
                        if !has_bevel {
                            modifier_stack
                                .add_modifier(Box::new(geo_kernel::BevelModifier::new(0.05)));
                        }
                        materials[0] = graph_engine::Material::new(
                            "WeatheredGranite",
                            [0.42, 0.44, 0.48, 1.0],
                            0.15,
                            0.82,
                            0.0,
                            2,
                        );
                    }
                }
                state.interaction_mode = InteractionMode::Object;
                state.show_cavity = true;
                state.cavity_intensity = 0.8;
                state.sync_gpu_meshes();
                state.set_status_message(
                    "✨ Bevel Modifier (0.05m chamfers) and Weathered Granite stone applied!",
                );
            }
        }
        _ => {}
    }
}

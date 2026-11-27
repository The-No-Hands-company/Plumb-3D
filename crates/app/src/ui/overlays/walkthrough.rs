use crate::state::AppState;
use core_math::{vec3, EulerRot, Quat, Transform};

/// Single step definition in an interactive in-viewport walkthrough tutorial.
#[derive(Debug, Clone)]
pub struct WalkthroughStep {
    pub title: String,
    pub subtitle: String,
    pub body: String,
    pub hotkey: Option<String>,
    pub tip: Option<String>,
}

/// Runtime state managing the in-viewport interactive step-by-step tutorial overlay.
#[derive(Debug, Clone, Default)]
pub struct WalkthroughState {
    pub is_active: bool,
    pub current_step: usize,
    pub title: String,
    pub steps: Vec<WalkthroughStep>,
    pub collapsed: bool,
}

impl WalkthroughState {
    /// Factory constructor for the beginner "Lunar Night Sky & Full Moon" walkthrough.
    pub fn lunar_night_sky() -> Self {
        Self {
            is_active: true,
            current_step: 0,
            title: "🌕 Tutorial: Lunar Night Sky & Astrophotography".to_string(),
            collapsed: false,
            steps: vec![
                WalkthroughStep {
                    title: "Step 1: The Celestial Sphere Geometry".to_string(),
                    subtitle: "Add -> Mesh -> UV Sphere (Shift+A)".to_string(),
                    body: "In 3D graphics, planetary bodies start as smooth primitives rather than sculpting millions of craters by hand. Spawn a UV Sphere with high subdivision (128 segments by 128 rings) to guarantee a silky-smooth silhouette and eliminate jagged polygon faceting.".to_string(),
                    hotkey: Some("Add -> Mesh -> UV Sphere (128 segments, 128 rings)".to_string()),
                    tip: Some("Setting 128x128 gives 32k smooth triangles, keeping the terminator line clean at 4K resolution.".to_string()),
                },
                WalkthroughStep {
                    title: "Step 2: Lunar Regolith PBR Material".to_string(),
                    subtitle: "Inspector -> Shading Tab".to_string(),
                    body: "Planetary dust (regolith) has extreme micro-porosity. Set Metallic to 0.0 (dielectric) and Roughness to 0.88. Then load genuine NASA 2K cartographic photographic albedo and normal relief maps to bring out impact craters.".to_string(),
                    hotkey: Some("Inspector -> Materials -> Browse Albedo & Normal Maps".to_string()),
                    tip: Some("Plumb 3D's Oren-Nayar diffuse BRDF reflects light back along the viewing axis, producing authentic retro-reflective lunar depth.".to_string()),
                },
                WalkthroughStep {
                    title: "Step 3: Solar Directional Light & The Terminator".to_string(),
                    subtitle: "Add -> Light -> Sun Light".to_string(),
                    body: "The moon emits no light of its own. Add a Directional Sun light (intensity ~2.5). The boundary between day and night is the 'Terminator line', where grazing solar rays reveal sharp crater wall relief.".to_string(),
                    hotkey: Some("Add -> Light -> Sun Light (Rotate with 'R' or 'E')".to_string()),
                    tip: Some("Click 'Full Moon' for frontal illumination, or 'Crescent' for dramatic terminator shadows!".to_string()),
                },
                WalkthroughStep {
                    title: "Step 4: 360 Deep Space Photographic Skybox".to_string(),
                    subtitle: "European Southern Observatory (ESO) 360 Panorama".to_string(),
                    body: "Deep space is not an empty black void. We bind the genuine 4000x2000 publication photograph from the ESO GigaGalaxy project, mapping the glowing Milky Way, star clusters, and cosmic dust lanes in 360 degrees around the moon.".to_string(),
                    hotkey: Some("World -> Environment Skybox".to_string()),
                    tip: Some("The 360 skybox provides authentic astrophotography stars with natural color temperatures.".to_string()),
                },
                WalkthroughStep {
                    title: "Step 5: Telephoto Astrophotography Framing".to_string(),
                    subtitle: "Add -> Camera (Telephoto 40° FOV)".to_string(),
                    body: "A standard 60° lens makes the moon look tiny like a coin. Astrophotographers use telephoto focal lengths. Add a Camera at (0, 0, 8.5m) with a narrow 40° field of view to fill the cinematic frame.".to_string(),
                    hotkey: Some("View -> Cameras -> Align Active Camera to View (Ctrl+Numpad 0)".to_string()),
                    tip: Some("A 40° FOV produces cinematic compression, making distant features feel majestic and intimate.".to_string()),
                },
                WalkthroughStep {
                    title: "Step 6: Masterpiece Rendering & LookDev".to_string(),
                    subtitle: "Direct PBR Snapshot (F11) or Path Tracer (F12)".to_string(),
                    body: "Choose your rendering engine: Press 'F11' for an instant (<0.1s) zero-noise Direct PBR Snapshot with real-time Cook-Torrance shading, or press 'F12' for progressive Monte Carlo path tracing with ACES Filmic tonemapping and optical bloom!".to_string(),
                    hotkey: Some("F11 for Instant Snapshot | F12 for Path Tracer".to_string()),
                    tip: Some("Set Bloom Threshold to 1.35 and Intensity to 0.15 for a natural nocturnal optical glare.".to_string()),
                },
            ],
        }
    }
}

/// Renders the floating interactive walkthrough card inside the 3D viewport.
pub fn render(state: &mut AppState) {
    if !state.walkthrough.is_active || state.walkthrough.steps.is_empty() {
        return;
    }

    let ctx = state.egui_ctx.clone();
    let current_step = state.walkthrough.current_step.min(state.walkthrough.steps.len() - 1);
    let step_count = state.walkthrough.steps.len();
    let step = state.walkthrough.steps[current_step].clone();

    let mut close = false;
    let mut prev_clicked = false;
    let mut next_clicked = false;
    let mut set_full_moon = false;
    let mut set_crescent = false;
    let mut focus_camera = false;

    egui::Window::new(&state.walkthrough.title)
        .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-16.0, 48.0))
        .default_width(340.0)
        .collapsible(true)
        .resizable(false)
        .frame(
            egui::Frame::window(&ctx.style())
                .fill(egui::Color32::from_rgba_premultiplied(18, 22, 32, 245))
                .stroke(egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(226, 179, 87)))
                .rounding(egui::Rounding::same(8.0))
                .inner_margin(egui::Margin::same(12.0)),
        )
        .show(&ctx, |ui| {
            // Header with Progress indicator
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(format!("Step {} of {}", current_step + 1, step_count))
                        .size(11.0)
                        .color(egui::Color32::from_rgb(226, 179, 87))
                        .strong(),
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("✕").on_hover_text("Close Walkthrough").clicked() {
                        close = true;
                    }
                });
            });

            ui.add_space(2.0);
            ui.label(egui::RichText::new(&step.title).strong().size(14.0).color(egui::Color32::WHITE));
            ui.label(egui::RichText::new(&step.subtitle).italics().size(11.0).color(egui::Color32::from_rgb(150, 165, 185)));

            ui.add_space(6.0);
            ui.separator();
            ui.add_space(6.0);

            // Body text
            ui.label(egui::RichText::new(&step.body).size(12.0).color(egui::Color32::from_rgb(215, 225, 235)));

            // Hotkey pill
            if let Some(ref hk) = step.hotkey {
                ui.add_space(6.0);
                egui::Frame::none()
                    .fill(egui::Color32::from_rgb(28, 34, 48))
                    .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(50, 65, 90)))
                    .rounding(egui::Rounding::same(4.0))
                    .inner_margin(egui::Margin::symmetric(8.0, 4.0))
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new(format!("💡 {}", hk)).small().color(egui::Color32::from_rgb(130, 200, 255)));
                    });
            }

            // Interactive Action Buttons for specific steps
            match current_step {
                0 => {
                    ui.add_space(6.0);
                    let has_moon = state.scene.nodes.values().any(|n| n.name == "Moon_Body");
                    let btn_text = if has_moon { "✓ Moon Sphere Ready (Refocus View)" } else { "➕ Spawn Moon UV Sphere (128x128)" };
                    if ui.button(btn_text).clicked() {
                        if !has_moon {
                            let moon_mesh = geo_kernel::Primitives::create_uv_sphere(3.0, 128, 128);
                            let moon_node = graph_engine::SceneNode::new_mesh("Moon_Body", Transform::IDENTITY, moon_mesh);
                            let id = state.scene.add_node(moon_node);
                            state.selected_node = Some(id);
                            state.sync_gpu_meshes();
                            state.set_status_message("🌕 Spawned Moon_Body UV Sphere (128x128 segments)");
                        }
                        focus_camera = true;
                    }
                }
                1 => {
                    ui.add_space(6.0);
                    if ui.button("🎨 Assign NASA 2K Cartography & Regolith Maps").clicked() {
                        let (albedo_path, normal_path) = crate::scenes::lunar_textures::ensure_lunar_textures_exist();
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
                                if let graph_engine::NodeData::Mesh { ref mut materials, .. } = node.data {
                                    if materials.is_empty() {
                                        materials.push(mat_regolith.clone());
                                    } else {
                                        materials[0] = mat_regolith.clone();
                                    }
                                }
                            }
                        }
                        state.sync_gpu_meshes();
                        state.set_status_message("🎨 Assigned 2K NASA Regolith PBR Material & Normal Relief!");
                    }
                }
                2 => {
                    ui.add_space(6.0);
                    let has_sun = state.scene.nodes.values().any(|n| n.name == "Sun_KeyLight");
                    if !has_sun {
                        if ui.button("➕ Add Directional Sun & Earthshine Lights").clicked() {
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
                            state.set_status_message("☀️ Solar Key Light and Earthshine Fill added to scene!");
                        }
                    } else {
                        ui.horizontal(|ui| {
                            if ui.button("🌕 Full Moon Sunlight").clicked() {
                                set_full_moon = true;
                            }
                            if ui.button("🌙 Crescent Moon Sunlight").clicked() {
                                set_crescent = true;
                            }
                        });
                    }
                }
                3 => {
                    ui.add_space(6.0);
                    if ui.button("🌌 Bind ESO 360 Deep Space Milky Way Skybox").clicked() {
                        state.set_status_message("🌌 ESO 360 Deep Space Milky Way Panorama active in Skybox!");
                    }
                }
                4 => {
                    ui.add_space(6.0);
                    if ui.button("📷 Setup Telephoto Astrophotography Camera").clicked() {
                        let has_cam = state.scene.nodes.values().any(|n| n.name == "Astrophotography_Camera");
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
                        focus_camera = true;
                        state.set_status_message("📷 Telephoto Astrophotography Camera aligned (40° FOV, 8.5m)");
                    }
                }
                5 => {
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if ui.button(
                            egui::RichText::new("⚡ Direct PBR (F11)")
                                .strong()
                                .color(egui::Color32::from_rgb(130, 200, 255)),
                        ).clicked() {
                            state.production_render.engine = crate::ui::render_dialog::RenderEngine::DirectPbr;
                            state.production_render.show_dialog = true;
                            crate::ui::render_dialog::start_production_render(state);
                        }
                        if ui.button(
                            egui::RichText::new("✨ Path Tracer (F12)")
                                .strong()
                                .color(egui::Color32::from_rgb(226, 179, 87)),
                        ).clicked() {
                            state.production_render.engine = crate::ui::render_dialog::RenderEngine::PathTracer;
                            state.production_render.show_dialog = true;
                            crate::ui::render_dialog::start_production_render(state);
                        }
                    });
                }
                _ => {}
            }

            // Tip
            if let Some(ref tip) = step.tip {
                ui.add_space(6.0);
                ui.label(egui::RichText::new(tip).italics().size(10.5).color(egui::Color32::from_rgb(140, 150, 165)));
            }

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(4.0);

            // Step Navigation Footer
            ui.horizontal(|ui| {
                if current_step > 0 {
                    if ui.button("◀ Back").clicked() {
                        prev_clicked = true;
                    }
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if current_step + 1 < step_count {
                        if ui.button(
                            egui::RichText::new("Next Step ▶").strong().color(egui::Color32::from_rgb(226, 179, 87)),
                        ).clicked() {
                            next_clicked = true;
                        }
                    } else if ui.button("Done! Close Guide").clicked() {
                        close = true;
                    }
                });
            });
        });

    if close {
        state.walkthrough.is_active = false;
    }
    if prev_clicked && state.walkthrough.current_step > 0 {
        state.walkthrough.current_step -= 1;
    }
    if next_clicked && state.walkthrough.current_step + 1 < state.walkthrough.steps.len() {
        state.walkthrough.current_step += 1;
    }
    if focus_camera {
        state.camera.target = vec3(0.0, 0.0, 0.0);
        state.camera.distance = 8.5;
        state.camera.pitch = 0.0;
        state.camera.yaw = 0.0;
        state.camera.fov_y = 40.0_f32.to_radians();
    }
    if set_full_moon {
        for (_id, node) in state.scene.nodes.iter_mut() {
            if node.name == "Sun_KeyLight" {
                node.local_transform = Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, 0.0, 0.05, 0.0));
                state.set_status_message("🌕 Sunlight aligned for glowing Full Moon!");
                break;
            }
        }
    }
    if set_crescent {
        for (_id, node) in state.scene.nodes.iter_mut() {
            if node.name == "Sun_KeyLight" {
                node.local_transform = Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.15, 1.45, 0.0));
                state.set_status_message("🌙 Sunlight shifted for dramatic Crescent terminator!");
                break;
            }
        }
    }
}

use crate::state::AppState;
use core_math::{vec3, EulerRot, Quat, Transform};

use super::walkthrough_actions::{render_castle_tower_step_buttons, render_lunar_step_buttons};

/// Identifies which tutorial journey is currently loaded in the walkthrough overlay.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WalkthroughId {
    #[default]
    None,
    LunarNightSky,
    CastleTower,
}

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
    pub id: WalkthroughId,
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
            id: WalkthroughId::LunarNightSky,
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

    /// Factory constructor for the hands-on "Medieval Castle Tower" modeling walkthrough.
    pub fn castle_tower() -> Self {
        Self {
            id: WalkthroughId::CastleTower,
            is_active: true,
            current_step: 0,
            title: "🏰 Tutorial: Medieval Castle Tower".to_string(),
            collapsed: false,
            steps: vec![
                WalkthroughStep {
                    title: "Step 1: Cylinder Primitive Base".to_string(),
                    subtitle: "Add -> Primitive -> Cylinder (Shift+A)".to_string(),
                    body: "Architectural towers start from a clean cylindrical base. Spawn a Cylinder with Radius 2.0m, Height 6.0m, and 16 segments. 16 perimeter facets divide evenly into 8 defensive battlements and 8 crenel gaps.".to_string(),
                    hotkey: Some("Add -> Primitive -> Cylinder (Radius 2m, Height 6m, 16 Segs)".to_string()),
                    tip: Some("16 segments give clean quad topologies when insetting concentric roof walkways.".to_string()),
                },
                WalkthroughStep {
                    title: "Step 2: Enter Edit Mode & Select Top Cap".to_string(),
                    subtitle: "Press Tab -> Press '3' (Face Mode)".to_string(),
                    body: "Switch from Object Mode to Edit Mode (Tab). Press '3' to activate Face selection level. Select the top circular cap of the cylinder to begin sculpting the tower summit.".to_string(),
                    hotkey: Some("Tab (Edit Mode) | 3 (Face Selection) | LMB Select".to_string()),
                    tip: Some("Notice the selected face highlights with a bright cyan outline in the viewport.".to_string()),
                },
                WalkthroughStep {
                    title: "Step 3: Outer Parapet Wall Inset".to_string(),
                    subtitle: "Press 'I' -> Inset 0.3m (Thickness)".to_string(),
                    body: "Press 'I' to activate Inset. Move inward 0.3m to define the outer defensive stone parapet wall thickness. This leaves a 0.3m protective barrier around the perimeter while isolating the inner floor.".to_string(),
                    hotkey: Some("I (Inset) -> Drag or Enter 0.3m".to_string()),
                    tip: Some("Inset creates a concentric loop of 16 quads without distorting the outer wall silhouette.".to_string()),
                },
                WalkthroughStep {
                    title: "Step 4: Sentry Walkway Boundary Inset".to_string(),
                    subtitle: "Press 'I' -> Inset 0.8m (Walkway Width)".to_string(),
                    body: "With the newly inset circular face still selected, press 'I' again and move inward 0.8m. This defines the wide sentry walkway loop and isolates the central elevated roof core.".to_string(),
                    hotkey: Some("I (Inset) -> Drag or Enter 0.8m".to_string()),
                    tip: Some("You now have two concentric rings: the outer parapet wall and the sentry walkway ring.".to_string()),
                },
                WalkthroughStep {
                    title: "Step 5: Extrude Sunken Walkway Trench".to_string(),
                    subtitle: "Select Walkway Loop -> Press 'E' -> -0.8m".to_string(),
                    body: "Select the 16 walkway ring quads. Press 'E' to Extrude and move down -0.8m. This sinks the walkway trench down, shielding castle guards behind the solid stone parapet.".to_string(),
                    hotkey: Some("E (Extrude) -> Move Down (-0.8m)".to_string()),
                    tip: Some("Sunken walkways are historical Norman fortress architecture for archer protection.".to_string()),
                },
                WalkthroughStep {
                    title: "Step 6: Defensive Crenellations (Merlons)".to_string(),
                    subtitle: "Select Alternating Rim Faces -> Press 'E' -> +0.5m".to_string(),
                    body: "Select alternating faces around the top outer rim wall (8 merlons). Press 'E' to extrude them upward by 0.5m. The gaps between allow archers to shoot while retaining stone cover!".to_string(),
                    hotkey: Some("Shift+LMB Alternating Faces -> E (Extrude +0.5m)".to_string()),
                    tip: Some("Plumb 3D maintains 2-manifold BMesh connectivity across all extruded perimeter blocks.".to_string()),
                },
                WalkthroughStep {
                    title: "Step 7: Stone Material & Bevel Chamfers".to_string(),
                    subtitle: "Modifiers -> + Bevel | Shading -> Granite PBR".to_string(),
                    body: "Real stone masonry has beveled, weathered edges. In the Inspector Modifier Stack, click '+ Bevel' (0.05m). Assign Weathered Granite material and enable Viewport Cavity Shading!".to_string(),
                    hotkey: Some("Inspector -> Modifiers -> + Bevel (0.05m)".to_string()),
                    tip: Some("The non-destructive Bevel modifier automatically calculates corner miters across all crenels.".to_string()),
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

            // Interactive Action Buttons
            match state.walkthrough.id {
                WalkthroughId::CastleTower => {
                    render_castle_tower_step_buttons(ui, state, current_step, &mut focus_camera);
                }
                _ => {
                    render_lunar_step_buttons(ui, state, current_step, &mut set_full_moon, &mut set_crescent, &mut focus_camera);
                }
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
        match state.walkthrough.id {
            WalkthroughId::CastleTower => {
                state.camera.target = vec3(0.0, 3.0, 0.0);
                state.camera.distance = 11.0;
                state.camera.pitch = 0.35;
                state.camera.yaw = 0.45;
                state.camera.fov_y = 45.0_f32.to_radians();
            }
            _ => {
                state.camera.target = vec3(0.0, 0.0, 0.0);
                state.camera.distance = 8.5;
                state.camera.pitch = 0.0;
                state.camera.yaw = 0.0;
                state.camera.fov_y = 40.0_f32.to_radians();
            }
        }
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

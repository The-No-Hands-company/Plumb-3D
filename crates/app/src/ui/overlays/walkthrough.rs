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
            title: "🌕 Journey: Lunar Night Sky & Full Moon".to_string(),
            collapsed: false,
            steps: vec![
                WalkthroughStep {
                    title: "Step 1: The Celestial Sphere".to_string(),
                    subtitle: "No manual sculpting needed".to_string(),
                    body: "The moon in front of you is a standard UV Sphere primitive. In 3D graphics, planetary bodies don't require sculpting millions of craters by hand — smooth geometry combined with lighting and materials does all the magic!".to_string(),
                    hotkey: Some("MMB or Alt+LMB: Orbit Camera".to_string()),
                    tip: Some("Hold Middle-Mouse-Button or Alt+Left-Click to orbit around the moon and observe its shape.".to_string()),
                },
                WalkthroughStep {
                    title: "Step 2: Lunar Regolith Material".to_string(),
                    subtitle: "Authentic matte planetary surface".to_string(),
                    body: "Notice the silvery-gray surface with dark patches. That's our procedural Lunar Regolith material: low albedo, high roughness (0.85), and marble noise producing dark basalt 'maria' (lunar seas) and bright crater highlands.".to_string(),
                    hotkey: Some("Right Panel -> Materials Tab".to_string()),
                    tip: Some("Real moon dust is non-glossy. High roughness gives the authentic soft lunar appearance.".to_string()),
                },
                WalkthroughStep {
                    title: "Step 3: Solar Illumination & The Terminator".to_string(),
                    subtitle: "Creating lunar phases with light".to_string(),
                    body: "The moon does not emit light; it reflects sunlight! The Directional Sun light shines across space. The boundary between light and shadow is called the 'Terminator line', which reveals crater contrast.".to_string(),
                    hotkey: Some("Use buttons below to switch Moon phases".to_string()),
                    tip: Some("Click 'Full Moon' for maximum brilliance, or 'Crescent' for dramatic terminator shadows!".to_string()),
                },
                WalkthroughStep {
                    title: "Step 4: Astrophotography Telephoto Framing".to_string(),
                    subtitle: "Cinematic focal framing".to_string(),
                    body: "Wide-angle camera lenses make the moon look tiny like a coin. We have configured an astrophotography camera with a 40° field of view at 8.5m distance, perfectly filling the frame while revealing surface features against deep space!".to_string(),
                    hotkey: Some("Dedicated Astrophotography Camera".to_string()),
                    tip: Some("Scroll Mouse Wheel to adjust framing or rely on the dedicated Astrophotography Camera during production rendering.".to_string()),
                },
                WalkthroughStep {
                    title: "Step 5: The Masterpiece Render".to_string(),
                    subtitle: "Progressive Monte Carlo with Bloom".to_string(),
                    body: "Everything is set! When you render, the path tracer traces millions of light photons, while the Bloom compositor produces that soft, radiant nocturnal lunar halo against deep space.".to_string(),
                    hotkey: Some("Press F12 to Render".to_string()),
                    tip: Some("Click the golden Render button below or press F12 on your keyboard.".to_string()),
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
    let step = &state.walkthrough.steps[current_step];

    let mut close = false;
    let mut prev_clicked = false;
    let mut next_clicked = false;
    let mut render_clicked = false;
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
                    if ui.button("🎯 Focus Camera on Moon (Numpad .)").clicked() {
                        focus_camera = true;
                    }
                }
                2 => {
                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        if ui.button("🌕 Full Moon Sunlight").clicked() {
                            set_full_moon = true;
                        }
                        if ui.button("🌙 Crescent Moon Sunlight").clicked() {
                            set_crescent = true;
                        }
                    });
                }
                4 => {
                    ui.add_space(8.0);
                    if ui.button(
                        egui::RichText::new("✨ Render Lunar Night Sky (F12)")
                            .strong()
                            .size(13.0)
                            .color(egui::Color32::from_rgb(226, 179, 87)),
                    ).clicked() {
                        render_clicked = true;
                    }
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
    if render_clicked {
        state.production_render.show_dialog = true;
    }
}

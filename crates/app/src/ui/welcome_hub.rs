use core_math::Transform;
use geo_kernel::Primitives;
use graph_engine::{SceneGraph, SceneNode};
use render_viewport::{CameraProjection, ViewportShadingMode};

use crate::state::AppState;
use crate::types::*;
use crate::ui::actions::UiActions;
use crate::ui::workspaces::{apply_studio_domain, apply_workspace, StudioDomain, WorkspaceTemplate};

/// Runtime state for the Interactive Welcome & Quick-Start Hub.
#[derive(Clone)]
pub struct WelcomeHubState {
    pub show_dialog: bool,
    pub dont_show_on_startup: bool,
    pub splash_texture: Option<egui::TextureHandle>,
}

impl std::fmt::Debug for WelcomeHubState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WelcomeHubState")
            .field("show_dialog", &self.show_dialog)
            .field("dont_show_on_startup", &self.dont_show_on_startup)
            .field("has_splash_texture", &self.splash_texture.is_some())
            .finish()
    }
}

impl Default for WelcomeHubState {
    fn default() -> Self {
        Self {
            show_dialog: true,
            dont_show_on_startup: false,
            splash_texture: None,
        }
    }
}

static SPLASH_BYTES: &[u8] = include_bytes!("../../../../docs/images/castle_citadel_hero.png");

/// Project starter template identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectTemplate {
    General,
    HardSurface,
    CadPrecision,
    Sculpting,
    SciFiCrateTutorial,
    SciFiCrate,
    CastleTowerTutorial,
    CastleTower,
    LunarTutorialBlank,
    LunarNightSky,
}

/// Draws the interactive Welcome & Quick-Start Hub modal window.
pub fn draw_welcome_hub(state: &mut AppState, _actions: &mut UiActions) {
    if !state.welcome_hub.show_dialog {
        return;
    }

    let ctx = state.egui_ctx.clone();
    let mut close = false;
    let mut selected_template = None;

    // Lazily load splash artwork texture on first render
    if state.welcome_hub.splash_texture.is_none() {
        if let Ok(img) = image::load_from_memory(SPLASH_BYTES) {
            let rgba = img.to_rgba8();
            let color_img = egui::ColorImage::from_rgba_unmultiplied(
                [rgba.width() as usize, rgba.height() as usize],
                &rgba,
            );
            state.welcome_hub.splash_texture = Some(ctx.load_texture(
                "plumb_splash_hero_tex",
                color_img,
                egui::TextureOptions::LINEAR,
            ));
        }
    }

    egui::Window::new("Plumb 3D -- Welcome & Quick-Start Hub")
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .fixed_size(egui::vec2(660.0, 680.0))
        .collapsible(false)
        .resizable(false)
        .frame(
            egui::Frame::window(&ctx.style())
                .fill(egui::Color32::from_rgb(18, 20, 26))
                .stroke(egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(226, 179, 87)))
                .rounding(egui::Rounding::same(8.0))
                .inner_margin(egui::Margin::same(16.0)),
        )
        .show(&ctx, |ui| {
            // Hero Splash Artwork Banner (Rendered with Plumb 3D Monte Carlo Path Tracer)
            if let Some(ref splash_tex) = state.welcome_hub.splash_texture {
                let banner_size = egui::vec2(628.0, 190.0);
                let tex_size = splash_tex.size_vec2();
                let img_aspect = (tex_size.x / tex_size.y.max(1.0)).max(0.01);
                let target_aspect = banner_size.x / banner_size.y;

                let uv_rect = if img_aspect < target_aspect {
                    // Image is taller than banner: crop top/bottom to fill width edge-to-edge
                    let uv_h = img_aspect / target_aspect;
                    let uv_y0 = (1.0 - uv_h) * 0.5;
                    egui::Rect::from_min_max(egui::pos2(0.0, uv_y0), egui::pos2(1.0, uv_y0 + uv_h))
                } else {
                    // Image is wider than banner: crop left/right to fill height
                    let uv_w = target_aspect / img_aspect;
                    let uv_x0 = (1.0 - uv_w) * 0.5;
                    egui::Rect::from_min_max(egui::pos2(uv_x0, 0.0), egui::pos2(uv_x0 + uv_w, 1.0))
                };

                ui.add(
                    egui::Image::new(splash_tex)
                        .uv(uv_rect)
                        .maintain_aspect_ratio(false)
                        .fit_to_exact_size(banner_size)
                        .rounding(egui::Rounding::same(6.0)),
                );
                ui.add_space(2.0);
                ui.label(
                    egui::RichText::new("Medieval Citadel & Fortress - Plumb 3D v1.0.0 Official Release Splash Artwork")
                        .size(10.0)
                        .italics()
                        .color(egui::Color32::from_rgb(130, 140, 160)),
                );
                ui.add_space(4.0);
            }

            // Header Banner
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new("PLUMB 3D").size(24.0).strong().color(egui::Color32::from_rgb(226, 179, 87)));
                    ui.label(egui::RichText::new("Precision & Hard-Surface DCC Suite - v1.0.0").size(12.0).color(egui::Color32::from_rgb(160, 170, 190)));
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Close").clicked() {
                        close = true;
                    }
                });
            });

            // Unsaved Session Crash Recovery Banner
            if state.autosave.check_recovery_available().is_some() {
                ui.add_space(4.0);
                egui::Frame::none()
                    .fill(egui::Color32::from_rgb(42, 34, 18))
                    .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(226, 179, 87)))
                    .rounding(egui::Rounding::same(6.0))
                    .inner_margin(egui::Margin::symmetric(10.0, 6.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Unsaved Session Recovery Available").strong().color(egui::Color32::from_rgb(255, 215, 0)));
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.button(egui::RichText::new("Restore Session").strong().color(egui::Color32::from_rgb(226, 179, 87))).clicked() {
                                    let mut autosave = std::mem::take(&mut state.autosave);
                                    let _ = autosave.restore_session(state);
                                    state.autosave = autosave;
                                    close = true;
                                }
                            });
                        });
                    });
                ui.add_space(4.0);
            }

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            ui.columns(2, |columns| {
                // Left Column: New Project Starter Templates
                columns[0].label(egui::RichText::new("New Project Templates").strong().size(14.0).color(egui::Color32::WHITE));
                columns[0].add_space(4.0);

                let templates = [
                    (ProjectTemplate::General, "General 3D", "Standard scene with default cube and studio lighting"),
                    (ProjectTemplate::HardSurface, "Hard-Surface Modeling", "Sub-d quad mesh, face mode, wireframe active"),
                    (ProjectTemplate::CadPrecision, "CAD Precision Drafting", "1mm grid snapping, calipers active, orthogonal camera"),
                    (ProjectTemplate::Sculpting, "Organic Sculpting", "Sculpting sphere, MatCap Clay, brush tools active"),
                    (ProjectTemplate::SciFiCrateTutorial, "📦 Sci-Fi Crate: Guided Tutorial", "Box modeling masterclass: Cube, Inset, Recessed Extrude & Bevel chamfers"),
                    (ProjectTemplate::CastleTowerTutorial, "🏰 Castle Tower: Guided Tutorial", "Build from scratch: Cylinder, Inset parapet, Extrude sunken walkway & crenels"),
                    (ProjectTemplate::CastleTower, "🏰 Medieval Citadel: Finished Scene", "Multi-mesh fortress with keep, ramparts, watchtower & torchlights"),
                    (ProjectTemplate::LunarTutorialBlank, "🌕 Lunar Night Sky: Guided Tutorial", "Build from scratch: 128x128 UV Sphere, PBR regolith, sunlight & skybox"),
                    (ProjectTemplate::LunarNightSky, "🌕 Lunar Night Sky: Ready Showcase", "Finished astrophotography scene with ESO 360 Milky Way & camera setup"),
                ];

                for (tmpl, title, desc) in templates {
                    let frame = egui::Frame::none()
                        .fill(egui::Color32::from_rgb(26, 30, 38))
                        .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(45, 52, 66)))
                        .rounding(egui::Rounding::same(6.0))
                        .inner_margin(egui::Margin::same(8.0));

                    let resp = frame.show(&mut columns[0], |ui| {
                        ui.label(egui::RichText::new(title).strong().color(egui::Color32::from_rgb(235, 240, 250)));
                        ui.label(egui::RichText::new(desc).size(10.5).color(egui::Color32::from_rgb(140, 150, 170)));
                    }).response;

                    if resp.interact(egui::Sense::click()).clicked() {
                        selected_template = Some(tmpl);
                        close = true;
                    }
                    columns[0].add_space(4.0);
                }

                // Right Column: Getting Started & Settings
                columns[1].label(egui::RichText::new("Getting Started").strong().size(14.0).color(egui::Color32::WHITE));
                columns[1].add_space(4.0);

                let doc_frame = egui::Frame::none()
                    .fill(egui::Color32::from_rgb(26, 30, 38))
                    .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(45, 52, 66)))
                    .rounding(egui::Rounding::same(6.0))
                    .inner_margin(egui::Margin::same(10.0));

                doc_frame.show(&mut columns[1], |ui| {
                    if ui.button("Open User Manual (F1)").clicked() {
                        state.show_help_dialog = true;
                        state.help_dialog_tab = 0;
                        close = true;
                    }
                    ui.add_space(4.0);
                    if ui.button("Beginner Step-by-Step Tutorials").clicked() {
                        state.show_help_dialog = true;
                        state.help_dialog_tab = 1;
                        close = true;
                    }
                    ui.add_space(4.0);
                    if ui.button("Search Commands (F3 / Ctrl+K)").clicked() {
                        state.command_palette.is_open = true;
                        close = true;
                    }
                });

                columns[1].add_space(10.0);
                columns[1].label(egui::RichText::new("Navigation Keymap Style").strong().size(13.0).color(egui::Color32::WHITE));
                columns[1].add_space(2.0);

                egui::ComboBox::from_id_salt("welcome_keymap_combo")
                    .selected_text(match state.preferences.keymap_style {
                        KeymapStyle::PlumbStandard => "Plumb 3D Native (W/E/R, Tab)",
                        KeymapStyle::BlenderVeteran => "Blender Veteran (G/S/R, Tab)",
                        KeymapStyle::MayaStudio => "Maya Studio (Alt+LMB, W/E/R)",
                    })
                    .show_ui(&mut columns[1], |ui| {
                        ui.selectable_value(&mut state.preferences.keymap_style, KeymapStyle::PlumbStandard, "Plumb 3D Native (W/E/R, Tab)");
                        ui.selectable_value(&mut state.preferences.keymap_style, KeymapStyle::BlenderVeteran, "Blender Veteran (G/S/R, Tab)");
                        ui.selectable_value(&mut state.preferences.keymap_style, KeymapStyle::MayaStudio, "Maya Studio (Alt+LMB, W/E/R)");
                    });

                columns[1].add_space(10.0);
                columns[1].label(egui::RichText::new("UI Density").strong().size(13.0).color(egui::Color32::WHITE));
                columns[1].horizontal(|ui| {
                    if ui.selectable_value(&mut state.preferences.ui_density, UiDensity::Comfortable, "Comfortable").clicked() {
                        crate::ui::preferences::apply_ui_density(&state.egui_ctx, UiDensity::Comfortable);
                    }
                    if ui.selectable_value(&mut state.preferences.ui_density, UiDensity::Compact, "Compact").clicked() {
                        crate::ui::preferences::apply_ui_density(&state.egui_ctx, UiDensity::Compact);
                    }
                });
            });

            ui.add_space(10.0);
            ui.separator();
            ui.horizontal(|ui| {
                ui.checkbox(&mut state.welcome_hub.dont_show_on_startup, "Don't show this splash on startup");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Continue to Viewport").clicked() {
                        close = true;
                    }
                });
            });
        });

    if close {
        state.welcome_hub.show_dialog = false;
    }

    if let Some(tmpl) = selected_template {
        apply_project_template(state, tmpl);
    }
}

/// Applies a starter template by building the initial mesh geometry and configuring workspace state.
pub fn apply_project_template(state: &mut AppState, template: ProjectTemplate) {
    state.scene = SceneGraph::new();
    state.gpu_mesh_cache.clear();
    state.object_colors.clear();
    state.object_materials.clear();
    state.selected_vertices.clear();
    state.selected_edges.clear();
    state.selected_faces.clear();

    match template {
        ProjectTemplate::General => {
            apply_studio_domain(state, StudioDomain::HardSurfaceCad);
            apply_workspace(state, WorkspaceTemplate::General);
            let cube = Primitives::create_cube(2.0);
            let node = SceneNode::new_mesh("Cube", Transform::IDENTITY, cube);
            let id = state.scene.add_node(node);
            state.selected_node = Some(id);
            state.interaction_mode = InteractionMode::Object;
            state.viewport_shading = ViewportShadingMode::Studio;
            state.camera.projection = CameraProjection::Perspective;
            state.inspector_tab = InspectorTab::Transform;
        }
        ProjectTemplate::HardSurface => {
            apply_studio_domain(state, StudioDomain::HardSurfaceCad);
            apply_workspace(state, WorkspaceTemplate::HardSurface);
            let cube = Primitives::create_cube(2.0);
            let node = SceneNode::new_mesh("HardSurfaceBase", Transform::IDENTITY, cube);
            let id = state.scene.add_node(node);
            state.selected_node = Some(id);
            state.interaction_mode = InteractionMode::Edit;
            state.sub_element_level = SubElementLevel::Face;
            state.viewport_shading = ViewportShadingMode::Studio;
            state.show_wireframe = true;
            state.inspector_tab = InspectorTab::Modifiers;
        }
        ProjectTemplate::CadPrecision => {
            apply_studio_domain(state, StudioDomain::HardSurfaceCad);
            apply_workspace(state, WorkspaceTemplate::CadPrecision);
            let cyl = Primitives::create_cylinder(1.5, 3.0, 16);
            let node = SceneNode::new_mesh("CadPart", Transform::IDENTITY, cyl);
            let id = state.scene.add_node(node);
            state.selected_node = Some(id);
            state.interaction_mode = InteractionMode::Edit;
            state.sub_element_level = SubElementLevel::Vertex;
            state.show_grid = true;
            state.show_wireframe = true;
            state.snap_enabled = true;
            state.snap_to_grid = true;
            state.measure_mode = true;
            state.camera.projection = CameraProjection::Orthographic;
            state.inspector_tab = InspectorTab::Transform;
        }
        ProjectTemplate::Sculpting => {
            apply_studio_domain(state, StudioDomain::Sculpting);
            apply_workspace(state, WorkspaceTemplate::Sculpting);
            let sphere = Primitives::create_uv_sphere(1.5, 32, 32);
            let node = SceneNode::new_mesh("SculptSphere", Transform::IDENTITY, sphere);
            let id = state.scene.add_node(node);
            state.selected_node = Some(id);
            state.interaction_mode = InteractionMode::Sculpt;
            state.viewport_shading = ViewportShadingMode::MatCapClay;
            state.show_wireframe = false;
            state.show_outliner = false;
            state.inspector_tab = InspectorTab::Tools;
        }
        ProjectTemplate::SciFiCrateTutorial => {
            apply_studio_domain(state, StudioDomain::HardSurfaceCad);
            apply_workspace(state, WorkspaceTemplate::HardSurface);
            crate::scenes::build_scifi_crate_tutorial_blank(state);
            return;
        }
        ProjectTemplate::SciFiCrate => {
            apply_studio_domain(state, StudioDomain::HardSurfaceCad);
            apply_workspace(state, WorkspaceTemplate::HardSurface);
            let (scene, camera, id) = crate::scenes::build_scifi_crate_finished_graph();
            state.scene = scene;
            state.camera = camera;
            state.selected_node = Some(id);
            state.interaction_mode = InteractionMode::Object;
            state.viewport_shading = ViewportShadingMode::Studio;
            state.show_wireframe = true;
            state.sync_gpu_meshes();
            state.set_status_message("📦 Loaded Finished Sci-Fi Storage Crate showcase scene!");
            return;
        }
        ProjectTemplate::CastleTowerTutorial => {
            apply_studio_domain(state, StudioDomain::HardSurfaceCad);
            apply_workspace(state, WorkspaceTemplate::HardSurface);
            crate::scenes::build_castle_tower_tutorial_blank(state);
            return;
        }
        ProjectTemplate::CastleTower => {
            apply_studio_domain(state, StudioDomain::HardSurfaceCad);
            apply_workspace(state, WorkspaceTemplate::HardSurface);
            crate::scenes::build_medieval_citadel_scene(state);
            return;
        }
        ProjectTemplate::LunarTutorialBlank => {
            apply_studio_domain(state, StudioDomain::CompleteSuite);
            apply_workspace(state, WorkspaceTemplate::General);
            crate::scenes::build_lunar_night_sky_tutorial_blank(state);
            return;
        }
        ProjectTemplate::LunarNightSky => {
            apply_studio_domain(state, StudioDomain::CompleteSuite);
            apply_workspace(state, WorkspaceTemplate::General);
            crate::scenes::build_lunar_night_sky_scene(state);
            return;
        }
    }

    state.sync_gpu_meshes();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_welcome_hub_defaults() {
        let state = WelcomeHubState::default();
        assert!(state.show_dialog);
        assert!(!state.dont_show_on_startup);
    }
}

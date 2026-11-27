use core_math::Vec3;
use render_pathtracer::{PathtracerCamera, PathTracerScene, RenderBuffer};
use std::time::Instant;

use crate::state::AppState;
use super::types::{ResolutionPreset, SamplePreset};

pub fn draw_render_dialog(state: &mut AppState) {
    if !state.production_render.show_dialog {
        return;
    }

    // Run progressive rendering step if active
    step_progressive_render(state);

    let ctx = state.egui_ctx.clone();
    let mut show = state.production_render.show_dialog;

    egui::Window::new("Plumb 3D Production Renderer")
        .open(&mut show)
        .default_pos(egui::pos2(90.0, 45.0))
        .default_size(egui::vec2(580.0, 820.0))
        .min_width(420.0)
        .min_height(520.0)
        .resizable(true)
        .show(&ctx, |ui| {
            ui.set_min_height(500.0);
            ui.vertical(|ui| {
                // Top Half: Live Framebuffer Canvas & Viewport Controls
                let avail_h = ui.available_height();
                let preview_h = (avail_h * 0.44).clamp(240.0, 520.0);
                draw_preview_canvas(state, ui, preview_h);

                ui.add_space(4.0);
                ui.separator();
                ui.add_space(4.0);

                // Bottom Half: Scrollable Settings, Camera, Post-Processing & Export
                egui::ScrollArea::vertical()
                    .id_salt("render_dialog_portrait_settings_scroll")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        draw_sidebar_controls(state, ui);
                    });
            });
        });

    state.production_render.show_dialog = show;
}

fn draw_sidebar_controls(state: &mut AppState, ui: &mut egui::Ui) {
    ui.heading("Render Settings");
    ui.horizontal(|ui| {
        ui.selectable_value(&mut state.production_render.engine, super::types::RenderEngine::PathTracer, "✨ Path Tracer");
        ui.selectable_value(&mut state.production_render.engine, super::types::RenderEngine::DirectPbr, "⚡ Instant Direct PBR");
    });
    let desc = match state.production_render.engine {
        super::types::RenderEngine::PathTracer => "Monte Carlo Ray Tracing & Multi-Bounce GI",
        super::types::RenderEngine::DirectPbr => "Direct PBR Evaluation (Instant / Zero Noise / Space & Celestial)",
    };
    ui.label(egui::RichText::new(desc).size(10.5).color(egui::Color32::GRAY));
    ui.add_space(8.0);

    // Resolution Presets
    egui::CollapsingHeader::new("Resolution")
        .default_open(true)
        .show(ui, |ui| {
            let current_preset = state.production_render.resolution_preset;
            egui::ComboBox::from_label("Preset")
                .selected_text(current_preset.label())
                .show_ui(ui, |ui| {
                    for preset in [
                        ResolutionPreset::Preview360p,
                        ResolutionPreset::Hd720p,
                        ResolutionPreset::FullHd1080p,
                        ResolutionPreset::Qhd1440p,
                        ResolutionPreset::Custom,
                    ] {
                        if ui.selectable_value(&mut state.production_render.resolution_preset, preset, preset.label()).clicked()
                            && preset != ResolutionPreset::Custom {
                                let (w, h) = preset.dimensions();
                                state.production_render.settings.width = w;
                                state.production_render.settings.height = h;
                            }
                    }
                });

            ui.horizontal(|ui| {
                ui.label("Width:");
                if ui.add(egui::DragValue::new(&mut state.production_render.settings.width).range(160..=3840).suffix(" px")).changed() {
                    state.production_render.resolution_preset = ResolutionPreset::Custom;
                }
            });
            ui.horizontal(|ui| {
                ui.label("Height:");
                if ui.add(egui::DragValue::new(&mut state.production_render.settings.height).range(90..=2160).suffix(" px")).changed() {
                    state.production_render.resolution_preset = ResolutionPreset::Custom;
                }
            });
        });

    ui.add_space(4.0);

    // Quality & Sampling
    egui::CollapsingHeader::new("Quality & Sampling")
        .default_open(true)
        .show(ui, |ui| {
            let current_sample_preset = state.production_render.sample_preset;
            egui::ComboBox::from_label("Samples")
                .selected_text(current_sample_preset.label())
                .show_ui(ui, |ui| {
                    for preset in [
                        SamplePreset::Draft16,
                        SamplePreset::Preview64,
                        SamplePreset::Medium128,
                        SamplePreset::High256,
                        SamplePreset::Ultra512,
                        SamplePreset::Custom,
                    ] {
                        if ui.selectable_value(&mut state.production_render.sample_preset, preset, preset.label()).clicked()
                            && preset != SamplePreset::Custom {
                                state.production_render.settings.target_samples = preset.target_samples();
                            }
                    }
                });

            ui.horizontal(|ui| {
                ui.label("Target SPP:");
                if ui.add(egui::DragValue::new(&mut state.production_render.settings.target_samples).range(1..=4096)).changed() {
                    state.production_render.sample_preset = SamplePreset::Custom;
                }
            });

            ui.horizontal(|ui| {
                ui.label("Max Bounces:");
                ui.add(egui::Slider::new(&mut state.production_render.settings.max_bounces, 1..=12));
            });
        });

    ui.add_space(4.0);

    // Camera Selection
    egui::CollapsingHeader::new("Camera Source")
        .default_open(true)
        .show(ui, |ui| {
            let has_scene_camera = state.active_camera_node.is_some();
            ui.radio_value(&mut state.production_render.use_scene_camera, false, "Viewport Orbit Camera");
            let scene_cam_resp = ui.add_enabled(
                has_scene_camera,
                egui::RadioButton::new(state.production_render.use_scene_camera, "Active Scene Camera Node"),
            );
            if scene_cam_resp.clicked() {
                state.production_render.use_scene_camera = true;
            }
            if !has_scene_camera {
                ui.label(egui::RichText::new("(No active camera node in scene)").size(10.0).color(egui::Color32::DARK_GRAY));
            }
        });

    ui.add_space(4.0);

    // Depth of Field (DoF) & Bokeh
    egui::CollapsingHeader::new("Depth of Field (Bokeh)")
        .default_open(false)
        .show(ui, |ui| {
            ui.checkbox(&mut state.production_render.dof_enabled, "Enable Optical Blur");
            if state.production_render.dof_enabled {
                ui.horizontal(|ui| {
                    ui.label("Aperture:");
                    ui.add(egui::Slider::new(&mut state.production_render.aperture, 0.01..=0.8).logarithmic(true));
                });
                ui.horizontal(|ui| {
                    ui.label("Focus Dist:");
                    ui.add(egui::DragValue::new(&mut state.production_render.focus_distance).range(0.1..=100.0).speed(0.1).suffix(" m"));
                });
                if ui.button("Autofocus to Target").clicked() {
                    let eye = state.camera.eye_position();
                    let target = state.camera.target;
                    state.production_render.focus_distance = (eye - target).length().max(0.1);
                }
            }
        });

    ui.add_space(4.0);

    // Post-Production & LookDev (VFB Compositor)
    egui::CollapsingHeader::new("Post-Production & LookDev")
        .default_open(true)
        .show(ui, |ui| {
            ui.checkbox(&mut state.production_render.post_process.enabled, "Enable Post-Processing");
            if state.production_render.post_process.enabled {
                ui.checkbox(&mut state.production_render.post_process.bloom_enabled, "Optical Bloom / Glow");
                if state.production_render.post_process.bloom_enabled {
                    ui.horizontal(|ui| {
                        ui.label("Bloom Int:");
                        ui.add(egui::Slider::new(&mut state.production_render.post_process.bloom_intensity, 0.0..=1.0));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Threshold:");
                        ui.add(egui::Slider::new(&mut state.production_render.post_process.bloom_threshold, 0.5..=3.0));
                    });
                }
                ui.horizontal(|ui| {
                    ui.label("Vignette:");
                    ui.add(egui::Slider::new(&mut state.production_render.post_process.vignette_intensity, 0.0..=0.8));
                });
                ui.horizontal(|ui| {
                    ui.label("Chromatic Aberr:");
                    ui.add(egui::Slider::new(&mut state.production_render.post_process.chromatic_aberration, 0.0..=1.0));
                });
                ui.horizontal(|ui| {
                    ui.label("Temperature:");
                    ui.add(egui::Slider::new(&mut state.production_render.post_process.temperature, -1.0..=1.0));
                });
                ui.horizontal(|ui| {
                    ui.label("Exposure EV:");
                    ui.add(egui::Slider::new(&mut state.production_render.post_process.exposure, 0.2..=3.0));
                });
                ui.horizontal(|ui| {
                    ui.label("Contrast:");
                    ui.add(egui::Slider::new(&mut state.production_render.post_process.contrast, 0.7..=1.5));
                });
                ui.horizontal(|ui| {
                    ui.label("Saturation:");
                    ui.add(egui::Slider::new(&mut state.production_render.post_process.saturation, 0.0..=2.0));
                });
                ui.checkbox(&mut state.production_render.post_process.filmic_tonemap, "ACES Filmic Tone Mapping");
            }
        });

    ui.add_space(10.0);

    // Render Action Buttons
    let is_rendering = state.production_render.is_rendering;
    let is_paused = state.production_render.is_paused;

    if !is_rendering {
        let btn_text = if state.production_render.buffer.is_some() {
            "Re-render (F12)"
        } else {
            "Start Render (F12)"
        };
        if ui.add_sized([ui.available_width(), 32.0], egui::Button::new(egui::RichText::new(btn_text).strong().size(14.0))).clicked() {
            start_production_render(state);
        }
    } else {
        ui.horizontal(|ui| {
            let pause_label = if is_paused { "Resume" } else { "Pause" };
            if ui.button(pause_label).clicked() {
                state.production_render.is_paused = !is_paused;
            }
            if ui.button("Stop").clicked() {
                state.production_render.is_rendering = false;
                state.production_render.status_message = "Render stopped by user".to_string();
            }
        });
    }

    ui.add_space(8.0);

    ui.separator();
    ui.add_space(4.0);

    // File Export Panel
    egui::CollapsingHeader::new("Export Image")
        .default_open(true)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("Path:");
                ui.text_edit_singleline(&mut state.production_render.export_path);
            });
            let can_save = state.production_render.buffer.as_ref().is_some_and(|b| b.samples_rendered > 0);
            if ui.add_enabled(can_save, egui::Button::new("Save Image as PNG")).clicked() {
                if let Some(buf) = &state.production_render.buffer {
                    let path = state.production_render.export_path.clone();
                    let res = if state.production_render.post_process.enabled {
                        buf.save_png_post_processed(&path, &state.production_render.post_process)
                    } else {
                        buf.save_png_filmic(&path, true)
                    };
                    match res {
                        Ok(()) => {
                            state.production_render.status_message = format!("Saved successfully to {}", path);
                        }
                        Err(e) => {
                            state.production_render.status_message = format!("Error: {}", e);
                        }
                    }
                }
            }
        });

    ui.add_space(8.0);
    ui.label(egui::RichText::new(&state.production_render.status_message).size(11.0).color(egui::Color32::GOLD));
}

fn draw_preview_canvas(state: &mut AppState, ui: &mut egui::Ui, preview_h: f32) {
    let current_samples = state.production_render.buffer.as_ref().map(|b| b.samples_rendered).unwrap_or(0);
    let target_samples = state.production_render.settings.target_samples.max(1);
    let is_rendering = state.production_render.is_rendering;
    let is_paused = state.production_render.is_paused;

    // Top Framebuffer Control Toolbar
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Live Framebuffer").strong().color(egui::Color32::WHITE));
        ui.label(
            egui::RichText::new(format!(
                "{}x{} | {}/{} SPP",
                state.production_render.settings.width,
                state.production_render.settings.height,
                current_samples,
                target_samples
            ))
            .small()
            .color(egui::Color32::from_rgb(140, 160, 190)),
        );

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if !is_rendering {
                let btn_text = if current_samples > 0 { "Re-render (F12)" } else { "Start Render (F12)" };
                if ui.button(egui::RichText::new(btn_text).strong().color(egui::Color32::from_rgb(226, 179, 87))).clicked() {
                    start_production_render(state);
                }
            } else {
                if ui.button("Stop").clicked() {
                    state.production_render.is_rendering = false;
                    state.production_render.status_message = "Render stopped by user".to_string();
                }
                let pause_text = if is_paused { "Resume" } else { "Pause" };
                if ui.button(pause_text).clicked() {
                    state.production_render.is_paused = !is_paused;
                }
            }
        });
    });

    ui.add_space(4.0);

    // Framebuffer Display Canvas with explicit guaranteed height
    let canvas_h = (preview_h - 40.0).max(180.0);
    let canvas_size = egui::vec2(ui.available_width(), canvas_h);
    egui::Frame::none()
        .fill(egui::Color32::from_rgb(10, 12, 16))
        .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(35, 42, 54)))
        .rounding(egui::Rounding::same(6.0))
        .show(ui, |ui| {
            ui.set_min_size(canvas_size);
            if let Some(tex) = &state.production_render.texture_handle {
                let img_size = tex.size_vec2();
                let aspect = img_size.x / img_size.y.max(1.0);

                let max_w = (canvas_size.x - 16.0).max(64.0);
                let max_h = (canvas_size.y - 16.0).max(64.0);
                let (draw_w, draw_h) = if max_w / max_h > aspect {
                    (max_h * aspect, max_h)
                } else {
                    (max_w, max_w / aspect)
                };

                ui.centered_and_justified(|ui| {
                    ui.add(egui::Image::new(tex).fit_to_exact_size(egui::vec2(draw_w, draw_h)));
                });
            } else {
                ui.centered_and_justified(|ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(canvas_size.y * 0.28);
                        ui.label(egui::RichText::new("🌕 Plumb 3D Production Renderer").size(17.0).color(egui::Color32::WHITE));
                        ui.add_space(6.0);
                        ui.label(egui::RichText::new("Click 'Start Render (F12)' to begin Monte Carlo path tracing").size(12.0).color(egui::Color32::GRAY));
                    });
                });
            }
        });

    // Progress Bar & Performance Counter directly beneath Live Canvas
    if let Some(buf) = &state.production_render.buffer {
        ui.add_space(6.0);
        let current = buf.samples_rendered;
        let target = state.production_render.settings.target_samples.max(1);
        let progress = (current as f32 / target as f32).clamp(0.0, 1.0);

        ui.add(egui::ProgressBar::new(progress).text(format!("Sample {} / {} ({:.0}%)", current, target, progress * 100.0)));

        let elapsed = state.production_render.total_elapsed_secs;
        let sps = if elapsed > 0.05 { current as f32 / elapsed } else { 0.0 };
        ui.label(egui::RichText::new(format!("Elapsed: {:.1}s  |  Speed: {:.1} spp/s", elapsed, sps)).size(11.0).color(egui::Color32::LIGHT_GRAY));
    }
}

pub fn start_production_render(state: &mut AppState) {
    // 1. Sync transforms & modifiers to ensure up-to-date geometry
    state.sync_gpu_meshes();

    let width = state.production_render.settings.width;
    let height = state.production_render.settings.height;
    let aspect_ratio = width as f32 / height as f32;

    // 2. Build BVH Ray Tracing Scene from SceneGraph
    let scene = PathTracerScene::from_scene_graph(&state.scene);

    // 3. Resolve Camera
    let aperture = if state.production_render.dof_enabled {
        state.production_render.aperture
    } else {
        0.0
    };
    let focus_dist = state.production_render.focus_distance;

    let camera = if let (true, Some(cam_id)) = (state.production_render.use_scene_camera, state.active_camera_node) {
        if let Some(cam_node) = state.scene.nodes.get(cam_id) {
            let look_from = cam_node.world_matrix.transform_point3(Vec3::ZERO);
            let look_dir = -cam_node.world_matrix.transform_vector3(Vec3::Z).normalize_or_zero();
            let look_at = look_from + look_dir * focus_dist;
            let vup = cam_node.world_matrix.transform_vector3(Vec3::Y).normalize_or_zero();
            let fov = if let graph_engine::NodeData::Camera { fov_y_radians, .. } = cam_node.data {
                fov_y_radians
            } else {
                45.0_f32.to_radians()
            };
            PathtracerCamera::new_thin_lens(look_from, look_at, vup, fov, aspect_ratio, aperture, focus_dist)
        } else {
            create_viewport_pathtracer_camera(state, aspect_ratio)
        }
    } else {
        create_viewport_pathtracer_camera(state, aspect_ratio)
    };

    // 4. Initialize render buffer / execute Direct PBR if selected
    if state.production_render.engine == super::types::RenderEngine::DirectPbr {
        let mut buffer = RenderBuffer::new(width, height);
        buffer.render_sample_pass(&scene, &camera, 1, None);
        buffer.samples_rendered = state.production_render.settings.target_samples;

        let rgba = if state.production_render.post_process.enabled {
            buffer.to_rgba8_post_processed(&state.production_render.post_process)
        } else {
            buffer.to_rgba8_filmic()
        };
        let color_image = egui::ColorImage::from_rgba_unmultiplied(
            [width as usize, height as usize],
            &rgba,
        );
        if let Some(tex) = &mut state.production_render.texture_handle {
            tex.set(color_image, egui::TextureOptions::LINEAR);
        } else {
            state.production_render.texture_handle = Some(state.egui_ctx.load_texture(
                "render_preview_tex",
                color_image,
                egui::TextureOptions::LINEAR,
            ));
        }
        state.production_render.buffer = Some(buffer);
        state.production_render.scene = Some(scene);
        state.production_render.camera = Some(camera);
        state.production_render.is_rendering = false;
        state.production_render.is_paused = false;
        state.production_render.total_elapsed_secs = 0.05;
        state.production_render.status_message = "Direct PBR Snapshot Complete (Instant / Zero Noise)".to_string();
        return;
    }

    state.production_render.buffer = Some(RenderBuffer::new(width, height));
    state.production_render.scene = Some(scene);
    state.production_render.camera = Some(camera);
    state.production_render.is_rendering = true;
    state.production_render.is_paused = false;
    state.production_render.render_start_instant = Some(Instant::now());
    state.production_render.total_elapsed_secs = 0.0;
    state.production_render.status_message = format!("Rendering {}x{} at {} target SPP...", width, height, state.production_render.settings.target_samples);
}

fn create_viewport_pathtracer_camera(state: &AppState, aspect_ratio: f32) -> PathtracerCamera {
    let look_from = state.camera.eye_position();
    let look_at = state.camera.target;
    let vup = Vec3::Y;
    let fov = state.camera.fov_y;
    let aperture = if state.production_render.dof_enabled {
        state.production_render.aperture
    } else {
        0.0
    };
    let focus_dist = state.production_render.focus_distance;
    PathtracerCamera::new_thin_lens(look_from, look_at, vup, fov, aspect_ratio, aperture, focus_dist)
}

fn step_progressive_render(state: &mut AppState) {
    if !state.production_render.is_rendering || state.production_render.is_paused {
        return;
    }

    let target_samples = state.production_render.settings.target_samples;
    let max_bounces = state.production_render.settings.max_bounces;
    let samples_per_frame = state.production_render.samples_per_frame.max(1);

    if let (Some(buffer), Some(scene), Some(camera)) = (
        &mut state.production_render.buffer,
        &state.production_render.scene,
        &state.production_render.camera,
    ) {
        if buffer.samples_rendered < target_samples {
            for _ in 0..samples_per_frame {
                if buffer.samples_rendered >= target_samples {
                    break;
                }
                buffer.render_sample_pass(scene, camera, max_bounces, None);
            }

            if let Some(start) = state.production_render.render_start_instant {
                state.production_render.total_elapsed_secs = start.elapsed().as_secs_f32();
            }

            // Update egui texture with current accumulating radiance and post-production
            let rgba = if state.production_render.post_process.enabled {
                buffer.to_rgba8_post_processed(&state.production_render.post_process)
            } else {
                buffer.to_rgba8_filmic()
            };
            let color_image = egui::ColorImage::from_rgba_unmultiplied(
                [buffer.width as usize, buffer.height as usize],
                &rgba,
            );

            if let Some(tex) = &mut state.production_render.texture_handle {
                tex.set(color_image, egui::TextureOptions::LINEAR);
            } else {
                state.production_render.texture_handle = Some(state.egui_ctx.load_texture(
                    "render_preview_tex",
                    color_image,
                    egui::TextureOptions::LINEAR,
                ));
            }

            // Immediately request next frame so progressive rendering proceeds smoothly
            state.egui_ctx.request_repaint();
        } else {
            state.production_render.is_rendering = false;
            let elapsed = state.production_render.total_elapsed_secs;
            let sps = buffer.samples_rendered as f32 / elapsed.max(0.001);
            state.production_render.status_message = format!(
                "Render Complete: {} samples in {:.2}s ({:.1} spp/s)",
                buffer.samples_rendered, elapsed, sps
            );
        }
    }
}

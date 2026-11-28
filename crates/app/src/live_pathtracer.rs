use crate::state::AppState;
use core_math::Vec3;
use render_pathtracer::{PathTracerScene, PathtracerCamera, RenderBuffer};
use render_viewport::ViewportShadingMode;

pub fn step_live_viewport_pathtracer(state: &mut AppState) {
    if state.viewport_shading != ViewportShadingMode::Rendered {
        return;
    }

    let eye = state.camera.eye_position();
    let target = state.camera.target;
    let camera_moved = (eye - state.live_pathtracer_last_eye_target.0).length_squared() > 1e-4
        || (target - state.live_pathtracer_last_eye_target.1).length_squared() > 1e-4;

    let width = (state.surface_config.width as f32 * 0.5).clamp(160.0, 960.0) as u32;
    let height = (state.surface_config.height as f32 * 0.5).clamp(90.0, 540.0) as u32;

    let needs_rebuild = state.live_pathtracer_buffer.is_none()
        || state.live_pathtracer_scene.is_none()
        || camera_moved;

    if needs_rebuild {
        let scene = PathTracerScene::from_scene_graph(&state.scene);
        let aspect_ratio = width as f32 / height as f32;
        let camera = PathtracerCamera::new_thin_lens(
            eye,
            target,
            Vec3::Y,
            state.camera.fov_y,
            aspect_ratio,
            0.0,
            (eye - target).length(),
        );

        state.live_pathtracer_buffer = Some(RenderBuffer::new(width, height));
        state.live_pathtracer_scene = Some(scene);
        state.live_pathtracer_camera = Some(camera);
        state.live_pathtracer_last_eye_target = (eye, target);
    }

    let max_samples = state.live_pathtracer_max_samples.max(1);
    if let (Some(buffer), Some(scene), Some(camera)) = (
        &mut state.live_pathtracer_buffer,
        &state.live_pathtracer_scene,
        &state.live_pathtracer_camera,
    ) {
        if buffer.samples_rendered < max_samples {
            buffer.render_sample_pass(scene, camera, 3, None);
            let rgba = buffer.to_rgba8();
            let color_image = egui::ColorImage::from_rgba_unmultiplied(
                [buffer.width as usize, buffer.height as usize],
                &rgba,
            );
            if let Some(handle) = &mut state.live_pathtracer_tex_handle {
                handle.set(color_image, egui::TextureOptions::LINEAR);
            } else {
                let handle = state.egui_ctx.load_texture(
                    "live_pathtracer_tex",
                    color_image,
                    egui::TextureOptions::LINEAR,
                );
                state.live_pathtracer_tex_handle = Some(handle);
            }
            state.window.request_redraw();
        }
    }
}

pub fn render_live_pathtracer_overlay(state: &mut AppState) {
    if state.viewport_shading != ViewportShadingMode::Rendered {
        return;
    }

    let width = state.surface_config.width as f32;
    let height = state.surface_config.height as f32;

    if let Some(tex) = &state.live_pathtracer_tex_handle {
        let painter = state.egui_ctx.layer_painter(egui::LayerId::new(
            egui::Order::Background,
            egui::Id::new("live_pathtracer_background"),
        ));
        let rect = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(width, height));
        painter.image(
            tex.id(),
            rect,
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            egui::Color32::WHITE,
        );
    }

    let samples = state
        .live_pathtracer_buffer
        .as_ref()
        .map(|b| b.samples_rendered)
        .unwrap_or(0);
    let max_s = state.live_pathtracer_max_samples;
    let ctx = state.egui_ctx.clone();
    egui::Area::new(egui::Id::new("live_pathtracer_hud"))
        .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-280.0, 48.0))
        .order(egui::Order::Middle)
        .show(&ctx, |ui| {
            egui::Frame::window(ui.style())
                .fill(egui::Color32::from_rgba_premultiplied(18, 22, 28, 230))
                .stroke(egui::Stroke::new(
                    1.0_f32,
                    egui::Color32::from_rgb(255, 180, 50),
                ))
                .rounding(egui::Rounding::same(5.0))
                .inner_margin(egui::Margin::symmetric(10.0, 5.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let icon = if samples < max_s { "☀" } else { "✓" };
                        let color = if samples < max_s {
                            egui::Color32::from_rgb(255, 200, 80)
                        } else {
                            egui::Color32::from_rgb(100, 255, 120)
                        };
                        ui.label(
                            egui::RichText::new(format!(
                                "{} Live Path Tracer: {}/{} SPP",
                                icon, samples, max_s
                            ))
                            .strong()
                            .color(color),
                        );
                    });
                });
        });
}

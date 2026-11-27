use crate::state::AppState;
use crate::ui::render_dialog::RenderEngine;

pub fn draw_render_menu(state: &mut AppState, ui: &mut egui::Ui) {
    ui.menu_button("Render", |ui| {
        if ui.button("Direct PBR Snapshot (F11)").clicked() {
            state.production_render.engine = RenderEngine::DirectPbr;
            state.production_render.show_dialog = true;
            crate::ui::render_dialog::start_production_render(state);
            ui.close_menu();
        }
        if ui.button("Render Image - Path Tracer (F12)").clicked() {
            state.production_render.engine = RenderEngine::PathTracer;
            state.production_render.show_dialog = true;
            crate::ui::render_dialog::start_production_render(state);
            ui.close_menu();
        }
        if ui.button("Render Settings & Framebuffer...").clicked() {
            state.production_render.show_dialog = true;
            ui.close_menu();
        }
        ui.separator();
        if ui.button("PBR Texture Baking...").clicked() {
            state.bake_state.show_dialog = true;
            ui.close_menu();
        }
        if ui.button("Evaluate Hardware RT Core Acceleration...").clicked() {
            let report = render_pathtracer::evaluate_hardware_rt_speedup(150_000, (1920, 1080), 1);
            state.set_status_message(format!(
                "RT Cores: {:.1}x Speedup (Est: {:.1} FPS @ 1080p, {:.0} MRays/s)",
                report.speedup_factor, report.hardware_rt_fps_estimate, report.hardware_mrays_per_sec
            ));
            ui.close_menu();
        }
        ui.separator();
        ui.label(egui::RichText::new("Dual Engines: Direct PBR & Path Tracer").size(11.0).color(egui::Color32::GRAY));
        ui.label(egui::RichText::new("Pure Safe Rust + Rayon BVH").size(10.0).color(egui::Color32::DARK_GRAY));
    });
}

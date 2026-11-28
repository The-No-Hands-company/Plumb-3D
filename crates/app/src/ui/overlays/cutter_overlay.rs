//! In-Viewport Interactive Boolean Cutter Overlay (BoxCutter / HardOps style).
//!
//! Renders real-time live footprints, extrusion wireframe bounds, radial array previews,
//! and millimeter caliper measurement badges for surface carving.

use crate::state::AppState;

pub fn render(state: &mut AppState) {
    let cutter = match &state.cutter_modal {
        Some(c) => c,
        None => return,
    };

    let width = state.surface_config.width as f32;
    let height = state.surface_config.height as f32;
    let painter = state.egui_ctx.layer_painter(egui::LayerId::new(
        egui::Order::Foreground,
        egui::Id::new("cutter_overlay"),
    ));

    let cutter_color = if cutter.is_ghost {
        egui::Color32::from_rgb(0, 230, 255) // Cyan for Live Ghost Cutter
    } else {
        egui::Color32::from_rgb(255, 140, 20) // Bright BoxCutter Orange
    };

    let min_u = cutter.start_uv.x.min(cutter.current_uv.x);
    let max_u = cutter.start_uv.x.max(cutter.current_uv.x);
    let min_v = cutter.start_uv.y.min(cutter.current_uv.y);
    let max_v = cutter.start_uv.y.max(cutter.current_uv.y);

    let p00 = cutter
        .workplane
        .uv_to_world(core_math::Vec2::new(min_u, min_v));
    let p10 = cutter
        .workplane
        .uv_to_world(core_math::Vec2::new(max_u, min_v));
    let p11 = cutter
        .workplane
        .uv_to_world(core_math::Vec2::new(max_u, max_v));
    let p01 = cutter
        .workplane
        .uv_to_world(core_math::Vec2::new(min_u, max_v));

    if let (Some(s00), Some(s10), Some(s11), Some(s01)) = (
        state.camera.world_to_screen(p00, width, height),
        state.camera.world_to_screen(p10, width, height),
        state.camera.world_to_screen(p11, width, height),
        state.camera.world_to_screen(p01, width, height),
    ) {
        let pts = [
            egui::pos2(s00.x, s00.y),
            egui::pos2(s10.x, s10.y),
            egui::pos2(s11.x, s11.y),
            egui::pos2(s01.x, s01.y),
        ];

        for i in 0..4 {
            painter.line_segment(
                [pts[i], pts[(i + 1) % 4]],
                egui::Stroke::new(2.5_f32, cutter_color),
            );
        }

        // Draw radial pattern preview if N > 1
        if cutter.radial_count > 1 {
            let n = cutter.radial_count;
            let v0 = cutter.current_uv - cutter.start_uv;
            for k in 1..n {
                let theta = (k as f32 / n as f32) * std::f32::consts::TAU;
                let cos_t = theta.cos();
                let sin_t = theta.sin();
                let rot_v =
                    core_math::Vec2::new(v0.x * cos_t - v0.y * sin_t, v0.x * sin_t + v0.y * cos_t);
                let inst_center = cutter.start_uv + rot_v;
                let inst_p = cutter.workplane.uv_to_world(inst_center);
                if let Some(sc) = state.camera.world_to_screen(inst_p, width, height) {
                    painter.circle_stroke(
                        egui::pos2(sc.x, sc.y),
                        8.0,
                        egui::Stroke::new(1.5_f32, cutter_color),
                    );
                }
            }
        }

        if cutter.stage == crate::modals::cutter::CutterStage::ExtrudeDepth {
            let n = cutter.workplane.normal;
            let d = cutter.depth;
            let b00 = p00 - n * d;
            let b10 = p10 - n * d;
            let b11 = p11 - n * d;
            let b01 = p01 - n * d;

            if let (Some(sb00), Some(sb10), Some(sb11), Some(sb01)) = (
                state.camera.world_to_screen(b00, width, height),
                state.camera.world_to_screen(b10, width, height),
                state.camera.world_to_screen(b11, width, height),
                state.camera.world_to_screen(b01, width, height),
            ) {
                let bpts = [
                    egui::pos2(sb00.x, sb00.y),
                    egui::pos2(sb10.x, sb10.y),
                    egui::pos2(sb11.x, sb11.y),
                    egui::pos2(sb01.x, sb01.y),
                ];
                for i in 0..4 {
                    painter.line_segment(
                        [bpts[i], bpts[(i + 1) % 4]],
                        egui::Stroke::new(1.5_f32, cutter_color),
                    );
                    painter
                        .line_segment([pts[i], bpts[i]], egui::Stroke::new(1.5_f32, cutter_color));
                }
            }
        }

        let dim_w = ((max_u - min_u).abs() * 1000.0).max(1.0);
        let dim_h = ((max_v - min_v).abs() * 1000.0).max(1.0);
        let dim_d = (cutter.depth * 1000.0).max(1.0);
        let shape_label = match cutter.shape {
            crate::modals::cutter::CutterShape::Box => "Box",
            crate::modals::cutter::CutterShape::Circle => "Circle",
            crate::modals::cutter::CutterShape::Ngon => "N-gon",
        };
        let array_label = if cutter.radial_count > 1 {
            format!(" | Array N={}", cutter.radial_count)
        } else {
            String::new()
        };
        let ghost_label = if cutter.is_ghost { " | Ghost" } else { "" };
        let sym_label = match cutter.symmetry_mirror {
            Some(geo_kernel::MirrorAxis::X) => " | Mirror X",
            Some(geo_kernel::MirrorAxis::Y) => " | Mirror Y",
            Some(geo_kernel::MirrorAxis::Z) => " | Mirror Z",
            None => "",
        };
        let caliper_text = format!(
            "{} W: {:.1} mm H: {:.1} mm D: {:.1} mm{}{}{}",
            shape_label, dim_w, dim_h, dim_d, array_label, sym_label, ghost_label
        );

        let center_screen = egui::pos2(
            (pts[0].x + pts[2].x) * 0.5,
            (pts[0].y + pts[2].y) * 0.5 - 20.0,
        );
        let pill_rect = egui::Rect::from_center_size(
            center_screen,
            egui::vec2(caliper_text.len() as f32 * 6.5 + 16.0, 22.0),
        );
        painter.rect_filled(pill_rect, 4.0, egui::Color32::from_black_alpha(230));
        painter.rect_stroke(pill_rect, 4.0, egui::Stroke::new(1.0_f32, cutter_color));
        painter.text(
            center_screen,
            egui::Align2::CENTER_CENTER,
            caliper_text,
            egui::FontId::proportional(11.5),
            egui::Color32::WHITE,
        );
    }
}

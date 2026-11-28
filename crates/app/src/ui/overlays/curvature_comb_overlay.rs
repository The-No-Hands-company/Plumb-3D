//! Viewport overlay for CAD Curvature Comb (Porcupine plot) analysis.
//! Visualizes second-derivative curvature needles and envelope curves along splines
//! providing real-time visual diagnostics for G1 tangent and G2 curvature continuity.

use crate::state::AppState;
use graph_engine::NodeData;

pub fn render(state: &mut AppState) {
    if !state.show_curvature_combs {
        return;
    }

    let width = state.surface_config.width as f32;
    let height = state.surface_config.height as f32;
    let painter = state.egui_ctx.layer_painter(egui::LayerId::new(
        egui::Order::Foreground,
        egui::Id::new("curvature_comb_overlay"),
    ));

    let density = state.curvature_comb_density.clamp(4, 32);
    let scale = state.curvature_comb_scale.clamp(0.01, 10.0);

    for (node_id, node) in &state.scene.nodes {
        // If an object is selected, highlight that curve; otherwise show all visible curves
        if let Some(sel_id) = state.selected_node {
            if node_id != sel_id && state.selected_nodes.len() <= 1 {
                continue;
            }
        }

        if let NodeData::Curve { spline, .. } = &node.data {
            let needles = spline.eval_curvature_comb(density, scale);
            if needles.is_empty() {
                continue;
            }

            let wmat = node.world_matrix;
            let mut prev_screen_tip: Option<egui::Pos2> = None;
            let mut max_curvature = 0.0f32;
            let mut max_curv_screen: Option<egui::Pos2> = None;

            for sample in &needles {
                let w_origin = wmat.transform_point3(sample.origin);
                let w_tip = wmat.transform_point3(sample.needle_tip);

                let s_orig_opt = state.camera.world_to_screen(w_origin, width, height);
                let s_tip_opt = state.camera.world_to_screen(w_tip, width, height);

                if let (Some(s_orig), Some(s_tip)) = (s_orig_opt, s_tip_opt) {
                    let p0 = egui::pos2(s_orig.x, s_orig.y);
                    let p1 = egui::pos2(s_tip.x, s_tip.y);

                    // Curvature color ramp: Cyan (low) -> Lime (med) -> Orange (high)
                    let c_val = (sample.curvature * 2.0).clamp(0.0, 1.0);
                    let color = if c_val < 0.5 {
                        let t = c_val * 2.0;
                        egui::Color32::from_rgb(
                            (0.0 * (1.0 - t) + 80.0 * t) as u8,
                            (220.0 * (1.0 - t) + 255.0 * t) as u8,
                            (255.0 * (1.0 - t) + 80.0 * t) as u8,
                        )
                    } else {
                        let t = (c_val - 0.5) * 2.0;
                        egui::Color32::from_rgb(
                            (80.0 * (1.0 - t) + 255.0 * t) as u8,
                            (255.0 * (1.0 - t) + 120.0 * t) as u8,
                            (80.0 * (1.0 - t) + 40.0 * t) as u8,
                        )
                    };

                    // Draw needle stroke
                    painter.line_segment([p0, p1], egui::Stroke::new(1.2_f32, color));

                    // Connect tips to form the curvature envelope
                    if let Some(prev_p1) = prev_screen_tip {
                        painter.line_segment(
                            [prev_p1, p1],
                            egui::Stroke::new(1.6_f32, egui::Color32::from_rgb(0, 240, 255)),
                        );
                    }
                    prev_screen_tip = Some(p1);

                    if sample.curvature > max_curvature {
                        max_curvature = sample.curvature;
                        max_curv_screen = Some(p1);
                    }
                } else {
                    prev_screen_tip = None;
                }
            }

            // Draw max curvature badge
            if let Some(pos) = max_curv_screen {
                let label = format!("κ_max: {:.2}", max_curvature);
                let badge_pos = egui::pos2(pos.x, pos.y - 12.0);
                painter.rect_filled(
                    egui::Rect::from_center_size(badge_pos, egui::vec2(70.0, 16.0)),
                    3.0,
                    egui::Color32::from_black_alpha(180),
                );
                painter.text(
                    badge_pos,
                    egui::Align2::CENTER_CENTER,
                    label,
                    egui::FontId::proportional(10.0),
                    egui::Color32::from_rgb(0, 240, 255),
                );
            }
        } else if let NodeData::Mesh { base_mesh, .. } = &node.data {
            let needles = base_mesh.evaluate_boundary_curvature_combs(scale);
            let wmat = node.world_matrix;

            for sample in &needles {
                let w_origin = wmat.transform_point3(sample.root);
                let w_tip = wmat.transform_point3(sample.tip);

                let s_orig_opt = state.camera.world_to_screen(w_origin, width, height);
                let s_tip_opt = state.camera.world_to_screen(w_tip, width, height);

                if let (Some(s_orig), Some(s_tip)) = (s_orig_opt, s_tip_opt) {
                    let p0 = egui::pos2(s_orig.x, s_orig.y);
                    let p1 = egui::pos2(s_tip.x, s_tip.y);
                    let color = egui::Color32::from_rgb(0, 230, 255);
                    painter.line_segment([p0, p1], egui::Stroke::new(1.0_f32, color));
                }
            }
        }
    }
}

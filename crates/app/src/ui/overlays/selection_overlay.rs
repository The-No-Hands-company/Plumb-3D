//! Viewport overlay rendering for multi-element selection tools (Box, Lasso, Circle/Paint).

use crate::state::AppState;
use crate::types::SelectionToolMode;

pub fn render_selection_overlay(state: &AppState) {
    if let (Some(start), Some(curr)) = (state.box_select_start, state.box_select_current) {
        let min_x = start.0.min(curr.0);
        let max_x = start.0.max(curr.0);
        let min_y = start.1.min(curr.1);
        let max_y = start.1.max(curr.1);
        let painter = state.egui_ctx.layer_painter(egui::LayerId::new(
            egui::Order::Foreground,
            egui::Id::new("marquee_overlay"),
        ));
        painter.rect(
            egui::Rect::from_min_max(egui::pos2(min_x, min_y), egui::pos2(max_x, max_y)),
            0.0,
            egui::Color32::from_rgba_premultiplied(40, 130, 255, 30),
            egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(80, 170, 255)),
        );
    } else if state.is_box_selecting && state.selection_tool_mode == SelectionToolMode::Lasso && state.lasso_points.len() >= 2 {
        let painter = state.egui_ctx.layer_painter(egui::LayerId::new(
            egui::Order::Foreground,
            egui::Id::new("lasso_overlay"),
        ));
        let pts: Vec<egui::Pos2> = state.lasso_points.iter().map(|&(x, y)| egui::pos2(x, y)).collect();
        for i in 0..pts.len() - 1 {
            painter.line_segment([pts[i], pts[i + 1]], egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(80, 170, 255)));
        }
        if pts.len() >= 3 {
            painter.line_segment([*pts.last().unwrap(), pts[0]], egui::Stroke::new(1.0_f32, egui::Color32::from_rgba_premultiplied(80, 170, 255, 120)));
        }
    } else if state.is_box_selecting && state.selection_tool_mode == SelectionToolMode::CirclePaint {
        if let Some((x, y)) = state.last_mouse_pos {
            let painter = state.egui_ctx.layer_painter(egui::LayerId::new(
                egui::Order::Foreground,
                egui::Id::new("circle_select_overlay"),
            ));
            painter.circle(
                egui::pos2(x as f32, y as f32),
                state.circle_select_radius,
                egui::Color32::from_rgba_premultiplied(40, 130, 255, 35),
                egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(80, 170, 255)),
            );
        }
    }
}

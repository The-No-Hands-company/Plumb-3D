//! Contextual 8-Direction Radial Pie Menu Overlay.
//!
//! Renders a floating gesture-responsive pie menu at the cursor position
//! providing muscle-memory cognitive velocity for the top 8 modeling actions.

use crate::modals::pie_menu::{cancel, commit, PIE_SLICES};
use crate::state::AppState;
use crate::ui::actions::UiActions;

pub fn render(state: &mut AppState, actions: &mut UiActions) {
    if !state.pie_menu_active {
        return;
    }

    let painter = state.egui_ctx.layer_painter(egui::LayerId::new(
        egui::Order::Foreground,
        egui::Id::new("pie_menu_radial_overlay"),
    ));

    let center = egui::pos2(
        state.pie_menu_center.0 as f32,
        state.pie_menu_center.1 as f32,
    );
    let r_inner = 24.0f32;
    let r_outer = 125.0f32;
    let r_label = 78.0f32;

    // 1. Dark translucent backdrop disk
    painter.circle_filled(center, r_outer + 8.0, egui::Color32::from_black_alpha(170));
    painter.circle_stroke(
        center,
        r_outer + 8.0,
        egui::Stroke::new(1.5_f32, egui::Color32::from_white_alpha(40)),
    );

    // 2. Center deadzone hub
    painter.circle_filled(
        center,
        r_inner,
        egui::Color32::from_rgba_premultiplied(30, 32, 40, 240),
    );
    painter.circle_stroke(
        center,
        r_inner,
        egui::Stroke::new(1.5_f32, egui::Color32::from_white_alpha(60)),
    );

    // Center crosshair / guide
    painter.line_segment(
        [center - egui::vec2(6.0, 0.0), center + egui::vec2(6.0, 0.0)],
        egui::Stroke::new(1.0_f32, egui::Color32::from_white_alpha(120)),
    );
    painter.line_segment(
        [center - egui::vec2(0.0, 6.0), center + egui::vec2(0.0, 6.0)],
        egui::Stroke::new(1.0_f32, egui::Color32::from_white_alpha(120)),
    );

    let active_slice = state.pie_menu_hovered_slice;

    // 3. Render 8 directional slices
    for (i, slice) in PIE_SLICES.iter().enumerate() {
        let is_hovered = active_slice == Some(i);

        // Direction angle in radians (0 = North / up, clockwise)
        let angle_rad = slice.angle_deg.to_radians();
        let sin_a = angle_rad.sin();
        let cos_a = angle_rad.cos();

        // Direction vector where Y is down in screen space
        let dir = egui::vec2(sin_a, -cos_a);

        // Divider ray line between slices (offset by 22.5 deg)
        let div_angle = (slice.angle_deg - 22.5).to_radians();
        let div_dir = egui::vec2(div_angle.sin(), -div_angle.cos());
        painter.line_segment(
            [
                center + div_dir * (r_inner + 2.0),
                center + div_dir * r_outer,
            ],
            egui::Stroke::new(1.0_f32, egui::Color32::from_white_alpha(30)),
        );

        // Position of label / button
        let label_pos = center + dir * r_label;

        // Hover highlight badge
        if is_hovered {
            painter.circle_filled(
                label_pos,
                30.0,
                egui::Color32::from_rgba_premultiplied(240, 180, 40, 60),
            );
            painter.circle_stroke(
                label_pos,
                30.0,
                egui::Stroke::new(2.0_f32, egui::Color32::from_rgb(255, 200, 60)),
            );
        }

        // Draw tool icon
        let icon_col = if is_hovered {
            egui::Color32::from_rgb(255, 220, 80)
        } else {
            egui::Color32::from_rgb(200, 210, 225)
        };

        painter.text(
            label_pos - egui::vec2(0.0, 10.0),
            egui::Align2::CENTER_CENTER,
            slice.icon,
            egui::FontId::proportional(16.0),
            icon_col,
        );

        // Draw tool name
        let text_col = if is_hovered {
            egui::Color32::WHITE
        } else {
            egui::Color32::from_rgb(180, 190, 205)
        };

        painter.text(
            label_pos + egui::vec2(0.0, 6.0),
            egui::Align2::CENTER_CENTER,
            slice.name,
            egui::FontId::proportional(11.0),
            text_col,
        );

        // Draw shortcut pill
        painter.text(
            label_pos + egui::vec2(0.0, 18.0),
            egui::Align2::CENTER_CENTER,
            format!("[{}]", slice.shortcut),
            egui::FontId::monospace(9.0),
            egui::Color32::from_white_alpha(110),
        );
    }

    // 4. Check for click to commit or escape
    let ctx = state.egui_ctx.clone();
    if ctx.input(|i| i.pointer.primary_clicked()) {
        commit(state, actions);
    } else if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        cancel(state);
    }
}

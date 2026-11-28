use crate::state::AppState;
use graph_engine::NodeData;

/// Draws the cinematic Camera Film Gate, Passepartout (masking outside frame),
/// Rule of Thirds composition guides, and center crosshair when looking through a Camera node.
pub fn render(state: &AppState) {
    let cam_id = match state.active_camera_node {
        Some(id) => id,
        None => return,
    };

    let cam_node = match state.scene.nodes.get(cam_id) {
        Some(node) => node,
        None => return,
    };

    let cam_aspect = match cam_node.data {
        NodeData::Camera { aspect_ratio, .. } => aspect_ratio,
        _ => 16.0 / 9.0,
    };

    let painter = state.egui_ctx.layer_painter(egui::LayerId::new(
        egui::Order::Middle,
        egui::Id::new("camera_passepartout_layer"),
    ));

    let screen_w = state.surface_config.width as f32;
    let screen_h = state.surface_config.height as f32;

    let margin = 35.0_f32;
    let available_w = (screen_w - margin * 2.0).max(10.0);
    let available_h = (screen_h - margin * 2.0).max(10.0);

    let (gate_w, gate_h) = if available_w / available_h > cam_aspect {
        (available_h * cam_aspect, available_h)
    } else {
        (available_w, available_w / cam_aspect)
    };

    let gate_x = (screen_w - gate_w) * 0.5;
    let gate_y = (screen_h - gate_h) * 0.5;
    let gate_rect =
        egui::Rect::from_min_size(egui::pos2(gate_x, gate_y), egui::vec2(gate_w, gate_h));

    let mask_color = egui::Color32::from_black_alpha(175);

    // 1. Draw Passepartout (Pillarbox & Letterbox dimming outside the camera gate)
    // Top bar
    painter.rect_filled(
        egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(screen_w, gate_y)),
        0.0,
        mask_color,
    );
    // Bottom bar
    painter.rect_filled(
        egui::Rect::from_min_max(
            egui::pos2(0.0, gate_y + gate_h),
            egui::pos2(screen_w, screen_h),
        ),
        0.0,
        mask_color,
    );
    // Left bar
    painter.rect_filled(
        egui::Rect::from_min_max(egui::pos2(0.0, gate_y), egui::pos2(gate_x, gate_y + gate_h)),
        0.0,
        mask_color,
    );
    // Right bar
    painter.rect_filled(
        egui::Rect::from_min_max(
            egui::pos2(gate_x + gate_w, gate_y),
            egui::pos2(screen_w, gate_y + gate_h),
        ),
        0.0,
        mask_color,
    );

    // 2. Camera Gate Border
    painter.rect_stroke(
        gate_rect,
        0.0,
        egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(220, 220, 230)),
    );

    // 3. Rule of Thirds Composition Lines
    let guide_stroke = egui::Stroke::new(
        1.0_f32,
        egui::Color32::from_rgba_unmultiplied(255, 255, 255, 45),
    );
    // Vertical thirds
    let v_third1 = gate_x + gate_w * (1.0 / 3.0);
    let v_third2 = gate_x + gate_w * (2.0 / 3.0);
    painter.line_segment(
        [
            egui::pos2(v_third1, gate_y),
            egui::pos2(v_third1, gate_y + gate_h),
        ],
        guide_stroke,
    );
    painter.line_segment(
        [
            egui::pos2(v_third2, gate_y),
            egui::pos2(v_third2, gate_y + gate_h),
        ],
        guide_stroke,
    );

    // Horizontal thirds
    let h_third1 = gate_y + gate_h * (1.0 / 3.0);
    let h_third2 = gate_y + gate_h * (2.0 / 3.0);
    painter.line_segment(
        [
            egui::pos2(gate_x, h_third1),
            egui::pos2(gate_x + gate_w, h_third1),
        ],
        guide_stroke,
    );
    painter.line_segment(
        [
            egui::pos2(gate_x, h_third2),
            egui::pos2(gate_x + gate_w, h_third2),
        ],
        guide_stroke,
    );

    // 4. Center Crosshair
    let center = gate_rect.center();
    let cross_len = 8.0_f32;
    let cross_stroke = egui::Stroke::new(
        1.0_f32,
        egui::Color32::from_rgba_unmultiplied(255, 255, 255, 80),
    );
    painter.line_segment(
        [
            egui::pos2(center.x - cross_len, center.y),
            egui::pos2(center.x + cross_len, center.y),
        ],
        cross_stroke,
    );
    painter.line_segment(
        [
            egui::pos2(center.x, center.y - cross_len),
            egui::pos2(center.x, center.y + cross_len),
        ],
        cross_stroke,
    );

    // 5. Camera Badge & HUD Label
    let badge_pos = egui::pos2(gate_x + 8.0, (gate_y - 18.0).max(35.0));
    painter.text(
        badge_pos,
        egui::Align2::LEFT_TOP,
        format!("[Cam] {} | Gate {:.2}:1", cam_node.name, cam_aspect),
        egui::FontId::proportional(11.0),
        egui::Color32::from_rgb(200, 210, 225),
    );
}

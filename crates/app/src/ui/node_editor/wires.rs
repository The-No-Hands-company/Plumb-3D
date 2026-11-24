use egui::{epaint::CubicBezierShape, Color32, Painter, Pos2, Stroke, Vec2};

/// Renders a smooth cubic Bézier cable between two node sockets.
pub fn draw_bezier_cable(
    painter: &Painter,
    from: Pos2,
    to: Pos2,
    color: Color32,
    thickness: f32,
) {
    let dx = (to.x - from.x).abs();
    let curvature = (dx * 0.5).clamp(30.0, 150.0);

    let ctrl1 = from + Vec2::new(curvature, 0.0);
    let ctrl2 = to - Vec2::new(curvature, 0.0);

    // Subtle drop shadow / ambient wire outline
    let shadow_color = Color32::from_black_alpha(100);
    let shadow_stroke = Stroke::new(thickness + 2.0, shadow_color);
    let shadow_shape = CubicBezierShape::from_points_stroke(
        [
            from + Vec2::new(0.0, 1.5),
            ctrl1 + Vec2::new(0.0, 1.5),
            ctrl2 + Vec2::new(0.0, 1.5),
            to + Vec2::new(0.0, 1.5),
        ],
        false,
        Color32::TRANSPARENT,
        shadow_stroke,
    );
    painter.add(shadow_shape);

    // Main colored data cable
    let stroke = Stroke::new(thickness, color);
    let bezier_shape = CubicBezierShape::from_points_stroke(
        [from, ctrl1, ctrl2, to],
        false,
        Color32::TRANSPARENT,
        stroke,
    );
    painter.add(bezier_shape);
}

/// Returns the characteristic UI theme color for a socket data type.
pub fn socket_color(socket_type: &geo_nodes::SocketType) -> Color32 {
    let [r, g, b, a] = socket_type.color_rgba();
    Color32::from_rgba_premultiplied(r, g, b, a)
}

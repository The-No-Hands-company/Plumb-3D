use crate::ui::node_editor::wires::socket_color;
use egui::{
    epaint::RectShape, Color32, CursorIcon, Id, Pos2, Rect, Rounding, Sense, Stroke, Ui, Vec2,
};
use geo_nodes::{GeoNode, NodeCategory, NodeKey, SocketValue};

pub const NODE_WIDTH: f32 = 180.0;
pub const HEADER_HEIGHT: f32 = 28.0;
pub const SOCKET_ROW_HEIGHT: f32 = 22.0;
pub const SOCKET_RADIUS: f32 = 5.0;

/// Information about a socket's screen position and identity
#[derive(Debug, Clone, Copy)]
pub struct SocketHit {
    pub node_key: NodeKey,
    pub is_output: bool,
    pub socket_index: usize,
    pub screen_pos: Pos2,
}

/// Renders a single visual node card. Returns any interaction events.
#[allow(clippy::too_many_arguments)]
pub fn draw_node_card(
    ui: &mut Ui,
    node_key: NodeKey,
    node: &mut GeoNode,
    pan: Vec2,
    zoom: f32,
    is_selected: bool,
    active_connected_inputs: &[usize],
    active_connected_outputs: &[usize],
    dragged_node: &mut Option<NodeKey>,
    node_to_delete: &mut Option<NodeKey>,
    socket_hits: &mut Vec<SocketHit>,
) -> Option<Pos2> {
    let screen_origin = Pos2::new(node.position[0] * zoom + pan.x, node.position[1] * zoom + pan.y);
    let rows = node.input_sockets.len().max(node.output_sockets.len()).max(1);
    let card_height = HEADER_HEIGHT + (rows as f32) * SOCKET_ROW_HEIGHT + 8.0;
    let card_rect = Rect::from_min_size(screen_origin, Vec2::new(NODE_WIDTH * zoom, card_height * zoom));

    let painter = ui.painter().clone();

    // 1. Background Card Body
    let bg_color = Color32::from_rgb(28, 30, 36);
    let border_color = if is_selected {
        Color32::from_rgb(60, 140, 240)
    } else {
        Color32::from_rgb(45, 48, 56)
    };
    painter.add(RectShape::filled(
        card_rect,
        Rounding::same(6.0 * zoom),
        bg_color,
    ));
    painter.rect_stroke(
        card_rect,
        Rounding::same(6.0 * zoom),
        Stroke::new(if is_selected { 1.8_f32 } else { 1.0_f32 }, border_color),
    );

    // 2. Header Bar
    let header_rect = Rect::from_min_size(
        screen_origin,
        Vec2::new(NODE_WIDTH * zoom, HEADER_HEIGHT * zoom),
    );
    let header_color = category_color(node.op.category());
    painter.add(RectShape::filled(
        header_rect,
        Rounding {
            nw: 6.0 * zoom,
            ne: 6.0 * zoom,
            sw: 0.0,
            se: 0.0,
        },
        header_color,
    ));

    // Header Title Text
    let title_pos = Pos2::new(screen_origin.x + 8.0 * zoom, screen_origin.y + 6.0 * zoom);
    painter.text(
        title_pos,
        egui::Align2::LEFT_TOP,
        node.op.name(),
        egui::FontId::proportional(12.0 * zoom),
        Color32::WHITE,
    );

    // Header Drag Handle Interaction
    let header_id = Id::new(node_key).with("header");
    let header_resp = ui.interact(header_rect, header_id, Sense::click_and_drag());
    if header_resp.hovered() {
        ui.ctx().set_cursor_icon(CursorIcon::Grab);
    }
    if header_resp.drag_started() {
        *dragged_node = Some(node_key);
    }
    if header_resp.dragged() {
        let delta = header_resp.drag_delta();
        node.position[0] += delta.x / zoom;
        node.position[1] += delta.y / zoom;
    }

    // Header Delete Button
    let delete_rect = Rect::from_min_size(
        Pos2::new(screen_origin.x + (NODE_WIDTH - 20.0) * zoom, screen_origin.y + 4.0 * zoom),
        Vec2::splat(16.0 * zoom),
    );
    let del_id = Id::new(node_key).with("del");
    let del_resp = ui.interact(delete_rect, del_id, Sense::click());
    if del_resp.hovered() {
        painter.text(
            delete_rect.center(),
            egui::Align2::CENTER_CENTER,
            "x",
            egui::FontId::proportional(14.0 * zoom),
            Color32::from_rgb(255, 90, 90),
        );
    } else {
        painter.text(
            delete_rect.center(),
            egui::Align2::CENTER_CENTER,
            "x",
            egui::FontId::proportional(12.0 * zoom),
            Color32::from_white_alpha(150),
        );
    }
    if del_resp.clicked() {
        *node_to_delete = Some(node_key);
    }

    // 3. Draw Sockets
    let body_start_y = screen_origin.y + HEADER_HEIGHT * zoom + 6.0 * zoom;

    // A. Input Sockets (Left Side)
    for (i, in_socket) in node.input_sockets.iter_mut().enumerate() {
        let row_center_y = body_start_y + (i as f32 * SOCKET_ROW_HEIGHT + SOCKET_ROW_HEIGHT * 0.5) * zoom;
        let socket_center = Pos2::new(screen_origin.x, row_center_y);
        let col = socket_color(&in_socket.socket_type);

        let is_connected = active_connected_inputs.contains(&i);
        draw_socket_dot(&painter, socket_center, col, is_connected, zoom);

        socket_hits.push(SocketHit {
            node_key,
            is_output: false,
            socket_index: i,
            screen_pos: socket_center,
        });

        // Input socket label
        painter.text(
            Pos2::new(socket_center.x + (SOCKET_RADIUS + 4.0) * zoom, row_center_y),
            egui::Align2::LEFT_CENTER,
            &in_socket.name,
            egui::FontId::proportional(11.0 * zoom),
            Color32::from_white_alpha(200),
        );

        // If not connected and is float or int, provide quick inline value drag
        if !is_connected && zoom > 0.6 {
            let input_widget_rect = Rect::from_center_size(
                Pos2::new(screen_origin.x + (NODE_WIDTH * 0.55) * zoom, row_center_y),
                Vec2::new(45.0 * zoom, 15.0 * zoom),
            );
            match &mut in_socket.value {
                SocketValue::Float(val) => {
                    let mut temp = *val;
                    let resp = ui.put(
                        input_widget_rect,
                        egui::DragValue::new(&mut temp).speed(0.05).range(0.0_f32..=100.0_f32),
                    );
                    if resp.changed() {
                        *val = temp;
                    }
                }
                SocketValue::Int(val) => {
                    let mut temp = *val;
                    let resp = ui.put(
                        input_widget_rect,
                        egui::DragValue::new(&mut temp).speed(1.0).range(0_i64..=128_i64),
                    );
                    if resp.changed() {
                        *val = temp;
                    }
                }
                _ => {}
            }
        }
    }

    // B. Output Sockets (Right Side)
    for (i, out_socket) in node.output_sockets.iter().enumerate() {
        let row_center_y = body_start_y + (i as f32 * SOCKET_ROW_HEIGHT + SOCKET_ROW_HEIGHT * 0.5) * zoom;
        let socket_center = Pos2::new(screen_origin.x + NODE_WIDTH * zoom, row_center_y);
        let col = socket_color(&out_socket.socket_type);

        let is_connected = active_connected_outputs.contains(&i);
        draw_socket_dot(&painter, socket_center, col, is_connected, zoom);

        socket_hits.push(SocketHit {
            node_key,
            is_output: true,
            socket_index: i,
            screen_pos: socket_center,
        });

        // Output socket label
        painter.text(
            Pos2::new(socket_center.x - (SOCKET_RADIUS + 4.0) * zoom, row_center_y),
            egui::Align2::RIGHT_CENTER,
            &out_socket.name,
            egui::FontId::proportional(11.0 * zoom),
            Color32::from_white_alpha(200),
        );
    }

    None
}

fn draw_socket_dot(
    painter: &egui::Painter,
    center: Pos2,
    color: Color32,
    is_connected: bool,
    zoom: f32,
) {
    let r = SOCKET_RADIUS * zoom;
    if is_connected {
        painter.circle_filled(center, r, color);
        painter.circle_stroke(center, r, Stroke::new(1.0_f32, Color32::from_black_alpha(150)));
    } else {
        painter.circle_filled(center, r * 0.7_f32, Color32::from_rgb(22, 24, 28));
        painter.circle_stroke(center, r, Stroke::new(1.8_f32 * zoom, color));
    }
}

fn category_color(cat: NodeCategory) -> Color32 {
    match cat {
        NodeCategory::Primitives => Color32::from_rgb(38, 92, 140),   // Blue slate
        NodeCategory::GeometryOps => Color32::from_rgb(32, 115, 80),   // Teal / Green
        NodeCategory::CurveOps => Color32::from_rgb(110, 48, 140),     // Purple
        NodeCategory::PointOps => Color32::from_rgb(170, 50, 190),     // Magenta
        NodeCategory::Math => Color32::from_rgb(130, 85, 30),         // Amber / Orange
        NodeCategory::Inputs => Color32::from_rgb(60, 65, 75),        // Grey
        NodeCategory::Output => Color32::from_rgb(140, 40, 40),       // Deep Red / Crimson
    }
}

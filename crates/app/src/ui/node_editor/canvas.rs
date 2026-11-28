use super::node_card::{draw_node_card, SocketHit, SOCKET_RADIUS};
use super::wires::{draw_bezier_cable, socket_color};
use egui::{Color32, Pos2, Rect, Sense, Stroke, Ui, Vec2};
use geo_nodes::{GeoNodeGraph, NodeKey};
use std::collections::HashMap;

/// Persistent UI state for the 2D Node Editor canvas
#[derive(Debug, Clone)]
pub struct NodeCanvasState {
    pub pan: Vec2,
    pub zoom: f32,
    pub selected_node: Option<NodeKey>,
    pub dragging_node: Option<NodeKey>,
    pub connecting_from: Option<(NodeKey, usize)>, // (source_node, output_socket_idx)
    pub eval_error: Option<String>,
}

impl Default for NodeCanvasState {
    fn default() -> Self {
        Self {
            pan: Vec2::new(100.0, 100.0),
            zoom: 1.0,
            selected_node: None,
            dragging_node: None,
            connecting_from: None,
            eval_error: None,
        }
    }
}

/// Renders the complete 2D interactive node canvas.
pub fn draw_canvas(ui: &mut Ui, canvas_state: &mut NodeCanvasState, graph: &mut GeoNodeGraph) {
    let (canvas_rect, response) =
        ui.allocate_exact_size(ui.available_size(), Sense::click_and_drag());
    let painter = ui.painter_at(canvas_rect);

    // 1. Background Fill & Grid Pattern
    let bg_color = Color32::from_rgb(18, 19, 23);
    painter.rect_filled(canvas_rect, 0.0, bg_color);
    draw_canvas_grid(&painter, canvas_rect, canvas_state.pan, canvas_state.zoom);

    // 2. Pan and Zoom Handling
    if response.dragged_by(egui::PointerButton::Middle)
        || response.dragged_by(egui::PointerButton::Secondary)
    {
        canvas_state.pan += response.drag_delta();
    }
    let scroll_delta = ui.input(|i| i.raw_scroll_delta.y);
    if scroll_delta != 0.0 && response.hovered() {
        if let Some(mouse_pos) = ui.input(|i| i.pointer.hover_pos()) {
            let old_zoom = canvas_state.zoom;
            let zoom_factor = (scroll_delta * 0.0015).exp();
            let new_zoom = (old_zoom * zoom_factor).clamp(0.25, 2.5);

            // Zoom centered at mouse position
            let mouse_in_canvas = mouse_pos - canvas_rect.min;
            canvas_state.pan =
                mouse_in_canvas - (mouse_in_canvas - canvas_state.pan) * (new_zoom / old_zoom);
            canvas_state.zoom = new_zoom;
        }
    }

    // 3. Precompute connected sockets for each node
    let mut node_connected_inputs: HashMap<NodeKey, Vec<usize>> = HashMap::new();
    let mut node_connected_outputs: HashMap<NodeKey, Vec<usize>> = HashMap::new();
    for link in &graph.links {
        node_connected_outputs
            .entry(link.from_node)
            .or_default()
            .push(link.from_socket);
        node_connected_inputs
            .entry(link.to_node)
            .or_default()
            .push(link.to_socket);
    }

    // 4. Render Nodes and Gather Socket Screen Positions
    let mut socket_hits: Vec<SocketHit> = Vec::new();
    let mut node_to_delete: Option<NodeKey> = None;
    let pan = canvas_rect.min.to_vec2() + canvas_state.pan;
    let zoom = canvas_state.zoom;

    for (key, node) in &mut graph.nodes {
        let conn_inputs = node_connected_inputs.get(&key).cloned().unwrap_or_default();
        let conn_outputs = node_connected_outputs
            .get(&key)
            .cloned()
            .unwrap_or_default();
        let is_sel = canvas_state.selected_node == Some(key);

        draw_node_card(
            ui,
            key,
            node,
            pan,
            zoom,
            is_sel,
            &conn_inputs,
            &conn_outputs,
            &mut canvas_state.dragging_node,
            &mut node_to_delete,
            &mut socket_hits,
        );
    }

    // 5. Delete Node if Requested
    if let Some(del_key) = node_to_delete {
        graph.remove_node(del_key);
        if canvas_state.selected_node == Some(del_key) {
            canvas_state.selected_node = None;
        }
    }

    // 6. Draw Cables between Sockets
    for link in &graph.links {
        let from_hit = socket_hits.iter().find(|h| {
            h.node_key == link.from_node && h.is_output && h.socket_index == link.from_socket
        });
        let to_hit = socket_hits.iter().find(|h| {
            h.node_key == link.to_node && !h.is_output && h.socket_index == link.to_socket
        });

        if let (Some(f), Some(t)) = (from_hit, to_hit) {
            let color = graph
                .nodes
                .get(link.from_node)
                .and_then(|n| n.output_sockets.get(link.from_socket))
                .map(|s| socket_color(&s.socket_type))
                .unwrap_or(Color32::from_rgb(180, 180, 180));

            draw_bezier_cable(
                &painter,
                f.screen_pos,
                t.screen_pos,
                color,
                2.5 * zoom.max(0.5),
            );
        }
    }

    // 7. Draw Active Connection Cable while Dragging
    if let Some((from_node, from_socket_idx)) = canvas_state.connecting_from {
        let from_hit = socket_hits
            .iter()
            .find(|h| h.node_key == from_node && h.is_output && h.socket_index == from_socket_idx);

        if let Some(f) = from_hit {
            if let Some(mouse_pos) = ui.input(|i| i.pointer.hover_pos()) {
                let color = graph
                    .nodes
                    .get(from_node)
                    .and_then(|n| n.output_sockets.get(from_socket_idx))
                    .map(|s| socket_color(&s.socket_type))
                    .unwrap_or(Color32::from_rgb(255, 200, 60));

                draw_bezier_cable(
                    &painter,
                    f.screen_pos,
                    mouse_pos,
                    color,
                    2.0 * zoom.max(0.5),
                );
            }
        }
    }

    // 8. Socket Interaction (Click & Connect)
    let pointer_pos = ui.input(|i| i.pointer.interact_pos());
    let pointer_clicked = ui.input(|i| i.pointer.primary_clicked());

    if let Some(pos) = pointer_pos {
        for hit in &socket_hits {
            let dist = (hit.screen_pos - pos).length();
            if dist < (SOCKET_RADIUS * zoom).max(9.0) {
                // Highlight socket on hover
                painter.circle_stroke(
                    hit.screen_pos,
                    (SOCKET_RADIUS + 3.0_f32) * zoom,
                    Stroke::new(1.5_f32, Color32::WHITE),
                );

                if pointer_clicked {
                    if hit.is_output {
                        // Start wire connection from this output
                        canvas_state.connecting_from = Some((hit.node_key, hit.socket_index));
                    } else if let Some((from_node, from_socket)) = canvas_state.connecting_from {
                        // Target is an input socket: complete connection!
                        if from_node != hit.node_key {
                            if let Err(e) =
                                graph.link(from_node, from_socket, hit.node_key, hit.socket_index)
                            {
                                canvas_state.eval_error = Some(format!("Link error: {:?}", e));
                            } else {
                                canvas_state.eval_error = None;
                            }
                        }
                        canvas_state.connecting_from = None;
                    } else {
                        // Clicking an existing input socket disconnects its link
                        graph.links.retain(|l| {
                            !(l.to_node == hit.node_key && l.to_socket == hit.socket_index)
                        });
                    }
                }
            }
        }
    }

    // Cancel cable connection if clicking empty space
    if pointer_clicked && canvas_state.connecting_from.is_some() {
        let is_over_socket = pointer_pos.is_some_and(|pos| {
            socket_hits
                .iter()
                .any(|h| (h.screen_pos - pos).length() < 12.0)
        });
        if !is_over_socket {
            canvas_state.connecting_from = None;
        }
    }
}

/// Draws an infinite pan/zoom dot grid on the node canvas.
fn draw_canvas_grid(painter: &egui::Painter, rect: Rect, pan: Vec2, zoom: f32) {
    let grid_size = 32.0 * zoom;
    if grid_size < 8.0 {
        return;
    }

    let start_x = rect.min.x + (pan.x % grid_size);
    let start_y = rect.min.y + (pan.y % grid_size);
    let dot_color = Color32::from_rgb(32, 34, 40);

    let mut x = start_x;
    while x < rect.max.x {
        let mut y = start_y;
        while y < rect.max.y {
            painter.circle_filled(Pos2::new(x, y), 1.0, dot_color);
            y += grid_size;
        }
        x += grid_size;
    }
}

pub mod canvas;
pub mod node_card;
pub mod palette;
pub mod wires;

pub use canvas::*;

use crate::state::AppState;
use core_math::Transform;
use graph_engine::NodeData;

/// Main UI entry point for the Visual Geometry Node Editor.
pub fn draw_node_editor(state: &mut AppState) {
    if !state.show_node_editor {
        return;
    }

    let mut open = state.show_node_editor;
    egui::Window::new("Geometry Nodes (Visual DAG)")
        .open(&mut open)
        .default_size([760.0, 480.0])
        .min_size([500.0, 320.0])
        .show(&state.egui_ctx.clone(), |ui| {
            draw_node_editor_inner(state, ui);
        });

    state.show_node_editor = open;
}

pub fn draw_node_editor_inner(state: &mut AppState, ui: &mut egui::Ui) {
    // Top Control Bar
    ui.horizontal(|ui| {
        if let Some(action) = palette::show_add_node_menu(ui) {
            // Spawn node near canvas center
            let center_pos = [
                (300.0 - state.node_canvas_state.pan.x) / state.node_canvas_state.zoom,
                (200.0 - state.node_canvas_state.pan.y) / state.node_canvas_state.zoom,
            ];
            action.spawn_into(&mut state.geo_node_graph, center_pos);
        }

        ui.separator();

        if ui.button("Evaluate to Viewport").on_hover_text("Evaluate geometry DAG and sync into active scene mesh").clicked() {
            match state.geo_node_graph.evaluate() {
                Ok(Some(mesh)) => {
                    state.node_canvas_state.eval_error = None;
                    if let Some(sel_id) = state.selected_node {
                        if let Some(node) = state.scene.nodes.get_mut(sel_id) {
                            if let NodeData::Mesh { base_mesh, cached_render_mesh, .. } = &mut node.data {
                                *base_mesh = mesh;
                                *cached_render_mesh = None;
                            }
                        }
                    } else {
                        let new_node = graph_engine::SceneNode::new_mesh(
                            "Procedural_GeoNodes",
                            Transform::IDENTITY,
                            mesh,
                        );
                        let id = state.scene.add_node(new_node);
                        state.selected_node = Some(id);
                    }
                    state.sync_gpu_meshes();
                }
                Ok(None) => {
                    state.node_canvas_state.eval_error = Some("Graph has no output geometry".to_string());
                }
                Err(err) => {
                    state.node_canvas_state.eval_error = Some(format!("Evaluation failed: {:?}", err));
                }
            }
        }

        if ui.button("Reset Graph").on_hover_text("Reset to standard procedural starter graph").clicked() {
            state.geo_node_graph = geo_nodes::GeoNodeGraph::default();
            state.node_canvas_state = NodeCanvasState::default();
        }

        ui.separator();

        // Graph Info & Status
        let node_count = state.geo_node_graph.nodes.len();
        let link_count = state.geo_node_graph.links.len();
        ui.label(format!("Nodes: {} | Links: {}", node_count, link_count));

        if let Some(ref err) = state.node_canvas_state.eval_error {
            ui.colored_label(egui::Color32::from_rgb(255, 100, 100), format!("Error: {}", err));
        }
    });

    ui.separator();

    // 2D Interactive Canvas
    canvas::draw_canvas(ui, &mut state.node_canvas_state, &mut state.geo_node_graph);
}

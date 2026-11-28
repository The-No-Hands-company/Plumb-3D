use graph_engine::NodeData;
use std::collections::HashSet;
use winit::keyboard::KeyCode;

use crate::state::AppState;
use crate::types::{InteractionMode, SubElementLevel};

pub fn handle_selection_mode_key(state: &mut AppState, key_code: KeyCode) -> bool {
    match key_code {
        KeyCode::Tab => {
            if state.shift_pressed {
                state.snap_enabled = !state.snap_enabled;
                state.window.request_redraw();
                return true;
            }
            if let Some(sel_id) = state.selected_node {
                if let Some(node) = state.scene.nodes.get(sel_id) {
                    if matches!(node.data, NodeData::Curve { .. }) {
                        state.interaction_mode = match state.interaction_mode {
                            InteractionMode::CurveEdit => InteractionMode::Object,
                            _ => InteractionMode::CurveEdit,
                        };
                        state.scene.mark_dirty(sel_id);
                        state.window.request_redraw();
                        return true;
                    }
                    if matches!(node.data, NodeData::Sketch { .. }) {
                        state.interaction_mode = match state.interaction_mode {
                            InteractionMode::Sketch => InteractionMode::Object,
                            _ => InteractionMode::Sketch,
                        };
                        state.scene.mark_dirty(sel_id);
                        state.window.request_redraw();
                        return true;
                    }
                }
            }
            state.interaction_mode = match state.interaction_mode {
                InteractionMode::Object => InteractionMode::Edit,
                InteractionMode::Edit => InteractionMode::Sculpt,
                InteractionMode::Sculpt => InteractionMode::WeightPaint,
                InteractionMode::WeightPaint => InteractionMode::VertexPaint,
                InteractionMode::VertexPaint => InteractionMode::TexturePaint,
                InteractionMode::TexturePaint => InteractionMode::Pose,
                InteractionMode::Pose => InteractionMode::Object,
                InteractionMode::CurveEdit => InteractionMode::Object,
                InteractionMode::Sketch => InteractionMode::Object,
            };
            if let Some(sel_id) = state.selected_node {
                state.scene.mark_dirty(sel_id);
            }
            state.window.request_redraw();
            true
        }
        KeyCode::Digit1 => {
            state.sub_element_level = SubElementLevel::Vertex;
            state.window.request_redraw();
            true
        }
        KeyCode::Digit2 => {
            state.sub_element_level = SubElementLevel::Edge;
            state.window.request_redraw();
            true
        }
        KeyCode::Digit3 => {
            state.sub_element_level = SubElementLevel::Face;
            state.window.request_redraw();
            true
        }
        KeyCode::KeyA => {
            if state.alt_pressed && state.interaction_mode == InteractionMode::Object {
                state.show_align_dialog = !state.show_align_dialog;
                state.window.request_redraw();
            } else if state.interaction_mode == InteractionMode::Edit {
                if let Some(sel_id) = state.selected_node {
                    if let Some(node) = state.scene.nodes.get(sel_id) {
                        if let NodeData::Mesh { base_mesh, .. } = &node.data {
                            match state.sub_element_level {
                                SubElementLevel::Vertex => {
                                    if state.selected_vertices.len() == base_mesh.vertex_count() {
                                        state.selected_vertices.clear();
                                    } else {
                                        state.selected_vertices =
                                            base_mesh.vertices.keys().collect();
                                    }
                                }
                                SubElementLevel::Edge => {
                                    if state.selected_edges.len() == base_mesh.edge_count() {
                                        state.selected_edges.clear();
                                    } else {
                                        state.selected_edges = base_mesh.edges.keys().collect();
                                    }
                                }
                                SubElementLevel::Face => {
                                    if state.selected_faces.len() == base_mesh.face_count() {
                                        state.selected_faces.clear();
                                    } else {
                                        state.selected_faces = base_mesh.faces.keys().collect();
                                    }
                                }
                            }
                        }
                    }
                }
            } else if state.selected_node.is_some() {
                state.selected_node = None;
            } else if let Some(&first_root) = state.scene.root_nodes.first() {
                state.selected_node = Some(first_root);
            }
            state.window.request_redraw();
            true
        }
        KeyCode::Equal | KeyCode::NumpadAdd => {
            if state.interaction_mode == InteractionMode::Edit {
                if let Some(sel_id) = state.selected_node {
                    if let Some(node) = state.scene.nodes.get(sel_id) {
                        if let NodeData::Mesh { base_mesh, .. } = &node.data {
                            match state.sub_element_level {
                                SubElementLevel::Vertex => {
                                    state.selected_vertices =
                                        base_mesh.grow_vertex_selection(&state.selected_vertices);
                                }
                                SubElementLevel::Edge => {
                                    state.selected_edges =
                                        base_mesh.grow_edge_selection(&state.selected_edges);
                                }
                                SubElementLevel::Face => {
                                    state.selected_faces =
                                        base_mesh.grow_face_selection(&state.selected_faces);
                                }
                            }
                        }
                    }
                }
            }
            state.window.request_redraw();
            true
        }
        KeyCode::Minus | KeyCode::NumpadSubtract => {
            if state.interaction_mode == InteractionMode::Edit {
                if let Some(sel_id) = state.selected_node {
                    if let Some(node) = state.scene.nodes.get(sel_id) {
                        if let NodeData::Mesh { base_mesh, .. } = &node.data {
                            match state.sub_element_level {
                                SubElementLevel::Vertex => {
                                    state.selected_vertices =
                                        base_mesh.shrink_vertex_selection(&state.selected_vertices);
                                }
                                SubElementLevel::Edge => {
                                    state.selected_edges =
                                        base_mesh.shrink_edge_selection(&state.selected_edges);
                                }
                                SubElementLevel::Face => {
                                    state.selected_faces =
                                        base_mesh.shrink_face_selection(&state.selected_faces);
                                }
                            }
                        }
                    }
                }
            }
            state.window.request_redraw();
            true
        }
        KeyCode::KeyL => {
            if state.ctrl_pressed {
                // Plasticity / Rhino: Ctrl+L: Loft Selected Curves
                let curve_ids: Vec<_> = state
                    .selected_nodes
                    .iter()
                    .copied()
                    .filter(|&id| {
                        state
                            .scene
                            .nodes
                            .get(id)
                            .is_some_and(|n| matches!(n.data, NodeData::Curve { .. }))
                    })
                    .collect();
                if curve_ids.len() >= 2 {
                    let cmd = Box::new(crate::commands::LoftCurvesCommand {
                        curve_node_ids: curve_ids,
                        samples_per_curve: 16,
                        closed_u: true,
                        closed_v: false,
                        created_node_id: None,
                    });
                    let _ = state.history.execute(cmd, &mut state.scene);
                    state.window.request_redraw();
                    return true;
                }
            } else if state.interaction_mode == InteractionMode::Edit {
                if let Some(sel_id) = state.selected_node {
                    if let Some(node) = state.scene.nodes.get(sel_id) {
                        if let NodeData::Mesh { base_mesh, .. } = &node.data {
                            match state.sub_element_level {
                                SubElementLevel::Edge => {
                                    let mut updated = HashSet::new();
                                    for &ek in &state.selected_edges {
                                        let edges = if state.alt_pressed {
                                            base_mesh.select_edge_ring(ek)
                                        } else {
                                            base_mesh.select_edge_loop(ek)
                                        };
                                        for edge in edges {
                                            updated.insert(edge);
                                        }
                                    }
                                    if !updated.is_empty() {
                                        state.selected_edges = updated;
                                    }
                                }
                                SubElementLevel::Face => {
                                    let mut updated = HashSet::new();
                                    for &fk in &state.selected_faces {
                                        let faces = if state.alt_pressed && state.ctrl_pressed {
                                            base_mesh.select_coplanar_faces(fk, 1.5)
                                        } else if state.alt_pressed {
                                            base_mesh.select_face_loop(fk)
                                        } else {
                                            base_mesh.select_linked(fk)
                                        };
                                        for face in faces {
                                            updated.insert(face);
                                        }
                                    }
                                    if !updated.is_empty() {
                                        state.selected_faces = updated;
                                    }
                                }
                                SubElementLevel::Vertex => {
                                    let mut updated = state.selected_vertices.clone();
                                    for &vk in &state.selected_vertices {
                                        for neighbor in base_mesh.vertex_neighbors(vk) {
                                            updated.insert(neighbor);
                                        }
                                    }
                                    state.selected_vertices = updated;
                                }
                            }
                        }
                    }
                }
            }
            state.window.request_redraw();
            true
        }
        _ => false,
    }
}

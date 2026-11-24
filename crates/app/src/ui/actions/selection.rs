use std::collections::HashSet;

use geo_kernel::{EdgeKey, FaceKey, VertexKey};
use graph_engine::NodeData;

use super::UiActions;
use crate::state::AppState;
use crate::types::{InteractionMode, SubElementLevel};

pub fn handle_selection(state: &mut AppState, actions: &mut UiActions) {
    // Handle Select All (A)
    if actions.trigger_select_all {
        if state.interaction_mode == InteractionMode::Edit {
            if let Some(sel_id) = state.selected_node {
                if let Some(node) = state.scene.nodes.get(sel_id) {
                    if let NodeData::Mesh { base_mesh, .. } = &node.data {
                        match state.sub_element_level {
                            SubElementLevel::Vertex => {
                                state.selected_vertices = base_mesh.vertices.keys().collect();
                            }
                            SubElementLevel::Edge => {
                                state.selected_edges = base_mesh.edges.keys().collect();
                            }
                            SubElementLevel::Face => {
                                state.selected_faces = base_mesh.faces.keys().collect();
                            }
                        }
                    }
                }
            }
        } else if let Some(&first_root) = state.scene.root_nodes.first() {
            state.selected_node = Some(first_root);
        }
    }

    // Handle Deselect All (Alt+A)
    if actions.trigger_deselect_all {
        if state.interaction_mode == InteractionMode::Edit {
            state.selected_vertices.clear();
            state.selected_edges.clear();
            state.selected_faces.clear();
        } else {
            state.selected_node = None;
        }
    }

    // Handle Invert Selection (Ctrl+I)
    if actions.trigger_invert_selection {
        if state.interaction_mode == InteractionMode::Edit {
            if let Some(sel_id) = state.selected_node {
                if let Some(node) = state.scene.nodes.get(sel_id) {
                    if let NodeData::Mesh { base_mesh, .. } = &node.data {
                        match state.sub_element_level {
                            SubElementLevel::Vertex => {
                                let all_v: HashSet<VertexKey> = base_mesh.vertices.keys().collect();
                                state.selected_vertices = all_v.difference(&state.selected_vertices).copied().collect();
                            }
                            SubElementLevel::Edge => {
                                let all_e: HashSet<EdgeKey> = base_mesh.edges.keys().collect();
                                state.selected_edges = all_e.difference(&state.selected_edges).copied().collect();
                            }
                            SubElementLevel::Face => {
                                let all_f: HashSet<FaceKey> = base_mesh.faces.keys().collect();
                                state.selected_faces = all_f.difference(&state.selected_faces).copied().collect();
                            }
                        }
                    }
                }
            }
        }
    }

    // Handle Grow Selection (+)
    if actions.trigger_grow_selection {
        if state.interaction_mode == InteractionMode::Edit {
            if let Some(sel_id) = state.selected_node {
                if let Some(node) = state.scene.nodes.get(sel_id) {
                    if let NodeData::Mesh { base_mesh, .. } = &node.data {
                        match state.sub_element_level {
                            SubElementLevel::Vertex => {
                                state.selected_vertices = base_mesh.grow_vertex_selection(&state.selected_vertices);
                            }
                            SubElementLevel::Edge => {
                                let mut new_edges = state.selected_edges.clone();
                                for &ek in &state.selected_edges {
                                    if let Some(e) = base_mesh.edges.get(ek) {
                                        let he = &base_mesh.half_edges[e.half_edge];
                                        for &v_key in &[he.origin, he.target] {
                                            if let Some(v) = base_mesh.vertices.get(v_key) {
                                                for &v_he in &v.half_edges {
                                                    new_edges.insert(base_mesh.half_edges[v_he].edge);
                                                }
                                            }
                                        }
                                    }
                                }
                                state.selected_edges = new_edges;
                            }
                            SubElementLevel::Face => {
                                state.selected_faces = base_mesh.grow_face_selection(&state.selected_faces);
                            }
                        }
                    }
                }
            }
        }
    }

    // Handle Shrink Selection (-)
    if actions.trigger_shrink_selection {
        if state.interaction_mode == InteractionMode::Edit {
            if let Some(sel_id) = state.selected_node {
                if let Some(node) = state.scene.nodes.get(sel_id) {
                    if let NodeData::Mesh { base_mesh, .. } = &node.data {
                        match state.sub_element_level {
                            SubElementLevel::Vertex => {
                                state.selected_vertices = base_mesh.shrink_vertex_selection(&state.selected_vertices);
                            }
                            SubElementLevel::Edge => {
                                let mut kept = HashSet::new();
                                for &ek in &state.selected_edges {
                                    if let Some(e) = base_mesh.edges.get(ek) {
                                        let he = &base_mesh.half_edges[e.half_edge];
                                        let both_selected = base_mesh.vertices.contains_key(he.origin)
                                            && base_mesh.vertices.contains_key(he.target);
                                        if both_selected {
                                            kept.insert(ek);
                                        }
                                    }
                                }
                                state.selected_edges = kept;
                            }
                            SubElementLevel::Face => {
                                state.selected_faces = base_mesh.shrink_face_selection(&state.selected_faces);
                            }
                        }
                    }
                }
            }
        }
    }

    // Handle Edge Loop Selection
    if actions.trigger_edge_loop {
        if state.interaction_mode == InteractionMode::Edit && state.sub_element_level == SubElementLevel::Edge {
            if let Some(sel_id) = state.selected_node {
                if let Some(node) = state.scene.nodes.get(sel_id) {
                    if let NodeData::Mesh { base_mesh, .. } = &node.data {
                        let mut full_loop = HashSet::new();
                        for &ek in &state.selected_edges {
                            for edge in base_mesh.select_edge_loop(ek) {
                                full_loop.insert(edge);
                            }
                        }
                        if !full_loop.is_empty() {
                            state.selected_edges = full_loop;
                        }
                    }
                }
            }
        }
    }

    // Handle Edge Ring Selection
    if actions.trigger_edge_ring {
        if state.interaction_mode == InteractionMode::Edit && state.sub_element_level == SubElementLevel::Edge {
            if let Some(sel_id) = state.selected_node {
                if let Some(node) = state.scene.nodes.get(sel_id) {
                    if let NodeData::Mesh { base_mesh, .. } = &node.data {
                        let mut full_ring = HashSet::new();
                        for &ek in &state.selected_edges {
                            for edge in base_mesh.select_edge_ring(ek) {
                                full_ring.insert(edge);
                            }
                        }
                        if !full_ring.is_empty() {
                            state.selected_edges = full_ring;
                        }
                    }
                }
            }
        }
    }
}

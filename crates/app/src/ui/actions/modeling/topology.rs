use geo_kernel::{EdgeKey, FaceKey, VertexKey};
use graph_engine::NodeData;

use crate::commands::*;
use crate::state::AppState;
use crate::ui::actions::UiActions;

pub fn handle_topology_actions(state: &mut AppState, actions: &mut UiActions) {
    // Handle Extrude Action with History
    if actions.trigger_extrude {
        if let Some(sel_id) = state.selected_node {
            let target_faces: Vec<FaceKey> = if !state.selected_faces.is_empty() {
                state.selected_faces.iter().copied().collect()
            } else if let Some(node) = state.scene.nodes.get(sel_id) {
                if let NodeData::Mesh { base_mesh, .. } = &node.data {
                    base_mesh.faces.keys().take(1).collect()
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            };

            if !target_faces.is_empty() {
                let cmd = Box::new(ExtrudeCommand::new(
                    sel_id,
                    target_faces,
                    state.extrude_distance,
                ));
                if let Err(e) = state.history.execute(cmd, &mut state.scene) {
                    eprintln!("Extrude failed: {}", e);
                } else if let Some(node) = state.scene.nodes.get(sel_id) {
                    if let NodeData::Mesh { base_mesh, .. } = &node.data {
                        state.selected_faces.clear();
                        if let Some(last_fk) = base_mesh.faces.keys().last() {
                            state.selected_faces.insert(last_fk);
                        }
                    }
                }
            }
        }
    }

    // Handle Bevel Action with History
    if actions.trigger_bevel {
        if let Some(sel_id) = state.selected_node {
            let target_faces: Vec<FaceKey> = if !state.selected_faces.is_empty() {
                state.selected_faces.iter().copied().collect()
            } else if let Some(node) = state.scene.nodes.get(sel_id) {
                if let NodeData::Mesh { base_mesh, .. } = &node.data {
                    base_mesh.faces.keys().take(1).collect()
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            };

            if !target_faces.is_empty() {
                let cmd = Box::new(BevelCommand {
                    node_id: sel_id,
                    faces: target_faces,
                    offset: state.bevel_offset,
                    inset: state.bevel_inset,
                    old_mesh: None,
                });
                if let Err(e) = state.history.execute(cmd, &mut state.scene) {
                    eprintln!("Bevel failed: {}", e);
                } else if let Some(node) = state.scene.nodes.get(sel_id) {
                    if let NodeData::Mesh { base_mesh, .. } = &node.data {
                        state.selected_faces.clear();
                        if let Some(last_fk) = base_mesh.faces.keys().last() {
                            state.selected_faces.insert(last_fk);
                        }
                    }
                }
            }
        }
    }

    // Handle Split Quad / Loop Cut with History
    if actions.trigger_split_quad {
        if let Some(sel_id) = state.selected_node {
            let target_face = state.selected_faces.iter().copied().next().or_else(|| {
                if let Some(node) = state.scene.nodes.get(sel_id) {
                    if let NodeData::Mesh { base_mesh, .. } = &node.data {
                        return base_mesh.faces.keys().next();
                    }
                }
                None
            });

            if let Some(fk) = target_face {
                let cmd = Box::new(SplitQuadCommand {
                    node_id: sel_id,
                    face_key: fk,
                    ratio: state.loop_cut_ratio,
                    old_mesh: None,
                });
                if let Err(e) = state.history.execute(cmd, &mut state.scene) {
                    eprintln!("Split quad failed: {}", e);
                }
            }
        }
    }

    // Handle Loop Cut Ring (Edge Ring Cut)
    if actions.trigger_loop_cut {
        if let Some(sel_id) = state.selected_node {
            if let Some(&ek) = state.selected_edges.iter().next() {
                let cmd = Box::new(LoopCutCommand::new(
                    sel_id,
                    ek,
                    state.loop_cut_ratio,
                ));
                if let Err(e) = state.history.execute(cmd, &mut state.scene) {
                    eprintln!("Loop cut failed: {}", e);
                }
            }
        }
    }

    // Handle Connect Vertices (Blender J)
    if actions.trigger_connect_vertices {
        if let Some(sel_id) = state.selected_node {
            if state.selected_vertices.len() >= 2 {
                let verts: Vec<VertexKey> = state.selected_vertices.iter().copied().take(2).collect();
                if let Some(node) = state.scene.nodes.get(sel_id) {
                    if let NodeData::Mesh { base_mesh, .. } = &node.data {
                        let f0 = base_mesh.vertex_faces(verts[0]);
                        let f1 = base_mesh.vertex_faces(verts[1]);
                        if let Some(&shared_f) = f0.iter().find(|fk| f1.contains(fk)) {
                            let cmd = Box::new(ConnectVerticesCommand {
                                node_id: sel_id,
                                face_key: shared_f,
                                v0: verts[0],
                                v1: verts[1],
                                old_mesh: None,
                            });
                            if let Err(e) = state.history.execute(cmd, &mut state.scene) {
                                eprintln!("Connect vertices failed: {}", e);
                            }
                        }
                    }
                }
            }
        }
    }

    // Handle Bevel Edge
    if actions.trigger_bevel_edge {
        if let Some(sel_id) = state.selected_node {
            let edges: Vec<EdgeKey> = state.selected_edges.iter().copied().collect();
            if !edges.is_empty() {
                let cmd = Box::new(BevelEdgeCommand {
                    node_id: sel_id,
                    edges,
                    width: state.bevel_modal_width,
                    segments: state.bevel_modal_segments,
                    old_mesh: None,
                });
                if let Err(e) = state.history.execute(cmd, &mut state.scene) {
                    eprintln!("Bevel edge failed: {}", e);
                } else {
                    state.selected_edges.clear();
                }
            }
        }
    }

    // Handle Split Edge
    if actions.trigger_split_edge {
        if let Some(sel_id) = state.selected_node {
            if let Some(&ek) = state.selected_edges.iter().next() {
                let cmd = Box::new(SplitEdgeCommand {
                    node_id: sel_id,
                    edge_key: ek,
                    ratio: 0.5,
                    old_mesh: None,
                });
                if let Err(e) = state.history.execute(cmd, &mut state.scene) {
                    eprintln!("Split edge failed: {}", e);
                }
            }
        }
    }

    // Handle Poke Face
    if actions.trigger_poke_face {
        if let Some(sel_id) = state.selected_node {
            let target_face = state.selected_faces.iter().copied().next().or_else(|| {
                if let Some(node) = state.scene.nodes.get(sel_id) {
                    if let NodeData::Mesh { base_mesh, .. } = &node.data {
                        return base_mesh.faces.keys().next();
                    }
                }
                None
            });

            if let Some(fk) = target_face {
                let cmd = Box::new(PokeFaceCommand {
                    node_id: sel_id,
                    face_key: fk,
                    offset: 0.2,
                    old_mesh: None,
                });
                if let Err(e) = state.history.execute(cmd, &mut state.scene) {
                    eprintln!("Poke face failed: {}", e);
                } else {
                    state.selected_faces.clear();
                }
            }
        }
    }

    // Handle Dissolve Edge
    if actions.trigger_dissolve_edge {
        if let Some(sel_id) = state.selected_node {
            if let Some(&ek) = state.selected_edges.iter().next() {
                let cmd = Box::new(DissolveEdgeCommand {
                    node_id: sel_id,
                    edge_key: ek,
                    old_mesh: None,
                });
                if let Err(e) = state.history.execute(cmd, &mut state.scene) {
                    eprintln!("Dissolve edge failed: {}", e);
                } else {
                    state.selected_edges.clear();
                }
            }
        }
    }

    // Handle Inset Faces (Blender I / 3ds Max Inset)
    if actions.trigger_inset {
        if let Some(sel_id) = state.selected_node {
            let target_faces: Vec<FaceKey> = if !state.selected_faces.is_empty() {
                state.selected_faces.iter().copied().collect()
            } else if let Some(node) = state.scene.nodes.get(sel_id) {
                if let NodeData::Mesh { base_mesh, .. } = &node.data {
                    base_mesh.faces.keys().collect()
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            };

            if !target_faces.is_empty() {
                let cmd = Box::new(InsetCommand {
                    node_id: sel_id,
                    faces: target_faces,
                    factor: state.inset_factor,
                    individual: state.inset_individual,
                    old_mesh: None,
                    created_faces: Vec::new(),
                });
                if let Err(e) = state.history.execute(cmd, &mut state.scene) {
                    eprintln!("Inset failed: {}", e);
                } else if let Some(node) = state.scene.nodes.get(sel_id) {
                    if let NodeData::Mesh { base_mesh, .. } = &node.data {
                        state.selected_faces.clear();
                        if let Some(last_fk) = base_mesh.faces.keys().last() {
                            state.selected_faces.insert(last_fk);
                        }
                    }
                }
            }
        }
    }

    // Handle Triangulate Faces
    if actions.trigger_triangulate {
        if let Some(sel_id) = state.selected_node {
            let faces: Vec<FaceKey> = if !state.selected_faces.is_empty() {
                state.selected_faces.iter().copied().collect()
            } else if let Some(node) = state.scene.nodes.get(sel_id) {
                if let NodeData::Mesh { base_mesh, .. } = &node.data {
                    base_mesh.faces.keys().collect()
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            };

            let cmd = Box::new(TriangulateCommand {
                node_id: sel_id,
                faces,
                quad_method: state.triangulate_quad_method,
                ngon_method: state.triangulate_ngon_method,
                min_vertices: state.triangulate_min_vertices,
                old_mesh: None,
            });
            if let Err(e) = state.history.execute(cmd, &mut state.scene) {
                eprintln!("Triangulate failed: {}", e);
            }
        }
    }

    // Handle Bevel Vertex
    if actions.trigger_bevel_vertex {
        if let Some(sel_id) = state.selected_node {
            let verts: Vec<VertexKey> = state.selected_vertices.iter().copied().collect();
            if !verts.is_empty() {
                let cmd = Box::new(BevelVertexCommand {
                    node_id: sel_id,
                    vertices: verts,
                    offset: state.vertex_bevel_offset,
                    old_mesh: None,
                });
                if let Err(e) = state.history.execute(cmd, &mut state.scene) {
                    eprintln!("Bevel vertex failed: {}", e);
                } else {
                    state.selected_vertices.clear();
                }
            }
        }
    }

    // Handle Merge Vertices at Center
    if actions.trigger_merge_vertices {
        if let Some(sel_id) = state.selected_node {
            let verts: Vec<VertexKey> = state.selected_vertices.iter().copied().collect();
            if verts.len() >= 2 {
                let cmd = Box::new(MergeVerticesCommand {
                    node_id: sel_id,
                    vertices: verts,
                    old_mesh: None,
                });
                if let Err(e) = state.history.execute(cmd, &mut state.scene) {
                    eprintln!("Merge vertices failed: {}", e);
                } else {
                    state.selected_vertices.clear();
                }
            }
        }
    }

    // Handle Rip Vertices (Blender V)
    if actions.trigger_rip_vertices {
        if let Some(sel_id) = state.selected_node {
            let verts: Vec<VertexKey> = state.selected_vertices.iter().copied().collect();
            if !verts.is_empty() {
                let cmd = Box::new(RipVerticesCommand {
                    node_id: sel_id,
                    vertices: verts,
                    old_mesh: None,
                    created_vertices: Vec::new(),
                });
                let _ = state.history.execute(cmd, &mut state.scene);
            }
        }
    }

    // Handle Un-Bevel Faces (MeshMachine / HardOps Alt+Shift+B)
    if actions.trigger_un_bevel {
        if let Some(sel_id) = state.selected_node {
            let target_faces: Vec<FaceKey> = state.selected_faces.iter().copied().collect();
            if !target_faces.is_empty() {
                let cmd = Box::new(UnBevelFacesCommand {
                    node_id: sel_id,
                    faces: target_faces.clone(),
                    old_mesh: None,
                });
                if state.history.execute(cmd, &mut state.scene).is_ok() {
                    state.sync_gpu_meshes();
                    state.set_status_message(format!("Un-Beveled {} chamfer face(s) back to crisp edge! (Ctrl+Z to Undo)", target_faces.len()));
                    state.selected_faces.clear();
                }
            } else {
                state.set_status_message("Select bevel/chamfer face(s) to un-bevel (Alt+Shift+B)");
            }
        }
    }
}

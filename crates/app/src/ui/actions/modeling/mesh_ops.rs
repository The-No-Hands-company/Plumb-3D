use geo_kernel::{EdgeKey, FaceKey, VertexKey};
use graph_engine::NodeData;

use crate::commands::*;
use crate::state::AppState;
use crate::ui::actions::UiActions;

pub fn handle_mesh_ops_actions(state: &mut AppState, actions: &mut UiActions) {
    // Handle Flip Normals (Blender Alt+N)
    if actions.trigger_flip_normals {
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
                let cmd = Box::new(FlipNormalsCommand {
                    node_id: sel_id,
                    faces: target_faces,
                    old_mesh: None,
                });
                if let Err(e) = state.history.execute(cmd, &mut state.scene) {
                    eprintln!("Flip normals failed: {}", e);
                }
            }
        }
    }

    // Handle Recalculate Normals Outside (Shift+N) / Inside (Shift+Ctrl+N)
    if let Some(inside) = actions.trigger_recalculate_normals {
        if let Some(sel_id) = state.selected_node {
            let cmd = Box::new(RecalculateNormalsCommand {
                node_id: sel_id,
                inside,
                old_mesh: None,
            });
            if let Err(e) = state.history.execute(cmd, &mut state.scene) {
                eprintln!("Recalculate normals failed: {}", e);
            }
        }
    }

    // Handle Bisect Plane
    if actions.trigger_bisect {
        if let Some(sel_id) = state.selected_node {
            let (plane_normal, plane_point) = match state.bisect_plane_axis {
                0 => (core_math::vec3(1.0, 0.0, 0.0), core_math::Vec3::ZERO),
                1 => (core_math::vec3(0.0, 1.0, 0.0), core_math::Vec3::ZERO),
                _ => (core_math::vec3(0.0, 0.0, 1.0), core_math::Vec3::ZERO),
            };
            let cmd = Box::new(BisectCommand {
                node_id: sel_id,
                plane_point,
                plane_normal,
                clear_positive: state.bisect_clear_positive,
                clear_negative: state.bisect_clear_negative,
                fill_cut: state.bisect_fill_cut,
                old_mesh: None,
            });
            if let Err(e) = state.history.execute(cmd, &mut state.scene) {
                eprintln!("Bisect failed: {}", e);
            } else {
                state.selected_faces.clear();
                state.selected_edges.clear();
                state.selected_vertices.clear();
            }
        }
    }

    // Handle Slide Edges
    if actions.trigger_slide_edges {
        if let Some(sel_id) = state.selected_node {
            let edges: Vec<EdgeKey> = state.selected_edges.iter().copied().collect();
            let vertices: Vec<VertexKey> = state.selected_vertices.iter().copied().collect();
            if !edges.is_empty() || !vertices.is_empty() {
                let cmd = Box::new(SlideEdgesCommand {
                    node_id: sel_id,
                    edges,
                    vertices,
                    factor: state.edge_slide_factor,
                    old_mesh: None,
                });
                if let Err(e) = state.history.execute(cmd, &mut state.scene) {
                    eprintln!("Slide edges failed: {}", e);
                }
            }
        }
    }

    // Handle Mark / Clear Sharp Edges
    if actions.trigger_mark_sharp || actions.trigger_clear_sharp {
        if let Some(sel_id) = state.selected_node {
            let edges: Vec<EdgeKey> = state.selected_edges.iter().copied().collect();
            if !edges.is_empty() {
                let sharp = actions.trigger_mark_sharp;
                let cmd = Box::new(MarkSharpCommand {
                    node_id: sel_id,
                    edges,
                    sharp,
                    old_mesh: None,
                });
                if let Err(e) = state.history.execute(cmd, &mut state.scene) {
                    eprintln!("Mark/clear sharp failed: {}", e);
                }
            }
        }
    }

    // Handle Clear Crease
    if actions.trigger_clear_crease {
        if let Some(sel_id) = state.selected_node {
            let edges: Vec<EdgeKey> = state.selected_edges.iter().copied().collect();
            if !edges.is_empty() {
                let cmd = Box::new(SetEdgeCreaseCommand {
                    node_id: sel_id,
                    edges,
                    crease_weight: 0.0,
                    old_mesh: None,
                });
                if let Err(e) = state.history.execute(cmd, &mut state.scene) {
                    eprintln!("Clear crease failed: {}", e);
                }
            }
        }
    }

    // Handle Auto Smooth Normals
    if actions.trigger_auto_smooth {
        if let Some(sel_id) = state.selected_node {
            if let Some(node) = state.scene.nodes.get_mut(sel_id) {
                if let NodeData::Mesh { base_mesh, cached_render_mesh, .. } = &mut node.data {
                    base_mesh.auto_smooth_normals(state.auto_smooth_angle_degrees.to_radians());
                    *cached_render_mesh = None;
                    state.scene.mark_dirty(sel_id);
                }
            }
        }
    }

    // Handle Symmetrize Mesh (Blender / Maya)
    if actions.trigger_symmetrize {
        if let Some(sel_id) = state.selected_node {
            let cmd = Box::new(SymmetrizeCommand {
                node_id: sel_id,
                axis: state.symmetrize_axis,
                direction: state.symmetrize_direction,
                old_mesh: None,
            });
            if let Err(e) = state.history.execute(cmd, &mut state.scene) {
                eprintln!("Symmetrize failed: {}", e);
            } else {
                state.selected_faces.clear();
                state.selected_edges.clear();
                state.selected_vertices.clear();
            }
        }
    }

    // Handle Clean Degenerate Geometry
    if actions.trigger_clean_degenerate {
        if let Some(sel_id) = state.selected_node {
            let cmd = Box::new(CleanDegenerateCommand {
                node_id: sel_id,
                min_area: state.clean_min_area,
                min_edge_len: state.clean_min_edge_len,
                old_mesh: None,
            });
            if let Err(e) = state.history.execute(cmd, &mut state.scene) {
                eprintln!("Clean degenerate failed: {}", e);
            } else {
                state.selected_faces.clear();
                state.selected_edges.clear();
                state.selected_vertices.clear();
            }
        }
    }

    // Handle Spin / Revolve
    if actions.trigger_spin {
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

            if !faces.is_empty() {
                let cmd = Box::new(SpinCommand {
                    node_id: sel_id,
                    faces,
                    axis: core_math::Vec3::Y,
                    center: core_math::Vec3::ZERO,
                    angle_radians: state.spin_angle_degrees.to_radians(),
                    steps: state.spin_steps,
                    old_mesh: None,
                });
                if let Err(e) = state.history.execute(cmd, &mut state.scene) {
                    eprintln!("Spin failed: {}", e);
                } else {
                    state.selected_faces.clear();
                    state.selected_edges.clear();
                    state.selected_vertices.clear();
                }
            }
        }
    }

    // Handle Cap Holes
    if actions.trigger_cap_holes {
        if let Some(sel_id) = state.selected_node {
            let cmd = Box::new(CapHolesCommand {
                node_id: sel_id,
                triangulate: false,
                old_mesh: None,
            });
            if let Err(e) = state.history.execute(cmd, &mut state.scene) {
                eprintln!("Cap holes failed: {}", e);
            }
        }
    }

    // Handle Grid Fill
    if actions.trigger_grid_fill {
        if let Some(sel_id) = state.selected_node {
            if let Some(node) = state.scene.nodes.get(sel_id) {
                if let NodeData::Mesh { base_mesh, .. } = &node.data {
                    let boundary_loops = base_mesh.boundary_loops();
                    if let Some(loop_verts) = boundary_loops.first() {
                        let span_count = (loop_verts.len() / 4).max(1);
                        let cmd = Box::new(GridFillCommand {
                            node_id: sel_id,
                            boundary: loop_verts.clone(),
                            span_count,
                            old_mesh: None,
                        });
                        if let Err(e) = state.history.execute(cmd, &mut state.scene) {
                            eprintln!("Grid fill failed: {}", e);
                        }
                    }
                }
            }
        }
    }

    // Handle Bridge Edge Loops
    if actions.trigger_bridge_edge_loops {
        if let Some(sel_id) = state.selected_node {
            let edges: Vec<EdgeKey> = state.selected_edges.iter().copied().collect();
            let cmd = Box::new(BridgeEdgeLoopsCommand::from_edges(
                sel_id,
                edges.clone(),
                0,
                0,
            ));
            if let Err(e) = state.history.execute(cmd, &mut state.scene) {
                eprintln!("Bridge edge loops failed: {}", e);
            } else {
                state.last_operator = Some(crate::types::LastOperatorInfo::Bridge {
                    edges,
                    twist: 0,
                    cuts: 0,
                });
                state.show_last_operator_panel = true;
                state.selected_edges.clear();
            }
        }
    }

    // Handle Shortest Path Selection (Dijkstra)
    if actions.trigger_shortest_path {
        if let Some(sel_id) = state.selected_node {
            if state.selected_vertices.len() >= 2 {
                let verts: Vec<VertexKey> = state.selected_vertices.iter().copied().take(2).collect();
                let start = verts[0];
                let end = verts[1];
                if let Some(node) = state.scene.nodes.get(sel_id) {
                    if let NodeData::Mesh { base_mesh, .. } = &node.data {
                        if let Some(path) = base_mesh.shortest_path(start, end) {
                            for vk in path {
                                state.selected_vertices.insert(vk);
                            }
                        }
                    }
                }
            }
        }
    }

    // Handle Fill Vertex Colors
    if actions.trigger_fill_vertex_color {
        if let Some(sel_id) = state.selected_node {
            let cmd = Box::new(FillVertexColorsCommand {
                node_id: sel_id,
                color: state.vertex_paint_color,
                old_mesh: None,
            });
            let _ = state.history.execute(cmd, &mut state.scene);
        }
    }

    // Handle Shrink / Fatten (Alt+S)
    if actions.trigger_shrink_fatten {
        if let Some(sel_id) = state.selected_node {
            let verts: Vec<VertexKey> = if !state.selected_vertices.is_empty() {
                state.selected_vertices.iter().copied().collect()
            } else if let Some(node) = state.scene.nodes.get(sel_id) {
                if let NodeData::Mesh { base_mesh, .. } = &node.data {
                    base_mesh.vertices.keys().collect()
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            };

            if !verts.is_empty() {
                let cmd = Box::new(ShrinkFattenCommand {
                    node_id: sel_id,
                    vertices: verts,
                    offset: state.shrink_fatten_offset,
                    old_mesh: None,
                });
                let _ = state.history.execute(cmd, &mut state.scene);
            }
        }
    }

    // Handle Relax Vertices
    if actions.trigger_relax_vertices {
        if let Some(sel_id) = state.selected_node {
            let verts: Vec<VertexKey> = if !state.selected_vertices.is_empty() {
                state.selected_vertices.iter().copied().collect()
            } else if let Some(node) = state.scene.nodes.get(sel_id) {
                if let NodeData::Mesh { base_mesh, .. } = &node.data {
                    base_mesh.vertices.keys().collect()
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            };

            if !verts.is_empty() {
                let cmd = Box::new(RelaxVerticesCommand {
                    node_id: sel_id,
                    vertices: verts,
                    iterations: state.relax_iterations,
                    factor: state.relax_factor,
                    old_mesh: None,
                });
                let _ = state.history.execute(cmd, &mut state.scene);
            }
        }
    }

    // Handle Sculpt Mask Operations
    if actions.trigger_invert_sculpt_mask {
        if let Some(sel_id) = state.selected_node {
            if let Some(node) = state.scene.nodes.get_mut(sel_id) {
                if let NodeData::Mesh { base_mesh, cached_render_mesh, .. } = &mut node.data {
                    base_mesh.invert_sculpt_mask();
                    *cached_render_mesh = None;
                    state.scene.mark_dirty(sel_id);
                }
            }
        }
    }

    if actions.trigger_clear_sculpt_mask {
        if let Some(sel_id) = state.selected_node {
            if let Some(node) = state.scene.nodes.get_mut(sel_id) {
                if let NodeData::Mesh { base_mesh, cached_render_mesh, .. } = &mut node.data {
                    base_mesh.clear_sculpt_mask();
                    *cached_render_mesh = None;
                    state.scene.mark_dirty(sel_id);
                }
            }
        }
    }

    if actions.trigger_smooth_sculpt_mask {
        if let Some(sel_id) = state.selected_node {
            if let Some(node) = state.scene.nodes.get_mut(sel_id) {
                if let NodeData::Mesh { base_mesh, cached_render_mesh, .. } = &mut node.data {
                    base_mesh.smooth_sculpt_mask(0.5);
                    *cached_render_mesh = None;
                    state.scene.mark_dirty(sel_id);
                }
            }
        }
    }
}

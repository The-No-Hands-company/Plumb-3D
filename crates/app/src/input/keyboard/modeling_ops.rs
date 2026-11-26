use std::collections::HashSet;
use core_math::Vec3;
use geo_kernel::{BezierPoint, EdgeKey, FaceKey, VertexKey};
use graph_engine::{InterpolationMode, NodeData};
use winit::keyboard::KeyCode;

use crate::commands::*;
use crate::state::AppState;
use crate::types::{CurveHandleSelection, InteractionMode, SelectionToolMode, SnapTarget, SubElementLevel};

pub fn handle_modeling_key(state: &mut AppState, key_code: KeyCode) -> bool {
    match key_code {
        KeyCode::KeyE => {
            if state.interaction_mode == InteractionMode::CurveEdit {
                if let Some(sel_id) = state.selected_node {
                    if let Some(node) = state.scene.nodes.get_mut(sel_id) {
                        if let NodeData::Curve { spline, cached_render_mesh, .. } = &mut node.data {
                            let cur_idx = state.selected_curve_knot.unwrap_or_else(|| spline.points.len().saturating_sub(1));
                            if let Some(cur_pt) = spline.points.get(cur_idx).copied() {
                                let dir = if cur_pt.handle_out.length_squared() > 1e-4 {
                                    cur_pt.handle_out.normalize()
                                } else {
                                    Vec3::X
                                };
                                let new_pos = cur_pt.position + dir * 1.5;
                                let new_pt = BezierPoint::smooth(new_pos, dir * 0.5);
                                let ins_idx = (cur_idx + 1).min(spline.points.len());
                                spline.points.insert(ins_idx, new_pt);
                                state.selected_curve_knot = Some(ins_idx);
                                state.selected_curve_handle = Some(CurveHandleSelection::Knot);
                                *cached_render_mesh = None;
                                state.scene.mark_dirty(sel_id);
                            }
                        }
                    }
                }
            } else if state.ctrl_pressed && state.sub_element_level == SubElementLevel::Edge {
                // Blender Ctrl+E: Mark Sharp
                if let Some(sel_id) = state.selected_node {
                    let target_edges: Vec<EdgeKey> = state.selected_edges.iter().copied().collect();
                    if !target_edges.is_empty() {
                        let cmd = Box::new(MarkSharpCommand {
                            node_id: sel_id,
                            edges: target_edges,
                            sharp: true,
                            old_mesh: None,
                        });
                        let _ = state.history.execute(cmd, &mut state.scene);
                        state.window.request_redraw();
                    }
                }
            } else if state.shift_pressed && state.sub_element_level == SubElementLevel::Edge {
                // Blender Shift+E: Interactive Edge Crease Modal
                if let Some(sel_id) = state.selected_node {
                    if let Some(node) = state.scene.nodes.get(sel_id) {
                        if let NodeData::Mesh { base_mesh, .. } = &node.data {
                            let edges: Vec<EdgeKey> = state.selected_edges.iter().copied().collect();
                            if !edges.is_empty() {
                                state.edge_crease_modal_edges = edges.clone();
                                state.edge_crease_modal_original_weights = edges
                                    .iter()
                                    .map(|&ek| (ek, base_mesh.get_edge_crease(ek)))
                                    .collect();
                                state.edge_crease_modal_weight = state
                                    .edge_crease_modal_original_weights
                                    .first()
                                    .map(|&(_, w)| w)
                                    .unwrap_or(0.0);
                                state.edge_crease_modal_start_mouse = state.last_mouse_pos.unwrap_or((0.0, 0.0));
                                state.edge_crease_modal_active = true;
                                state.window.request_redraw();
                            }
                        }
                    }
                }
            } else if state.shift_pressed {
                state.push_pull_tool = !state.push_pull_tool;
                state.set_status_message(if state.push_pull_tool {
                    "Direct Push-Pull Active: Click and drag any face to extrude along normal"
                } else {
                    "Direct Push-Pull Disabled"
                });
                state.window.request_redraw();
                return true;
            } else if state.interaction_mode == InteractionMode::Edit {
                state.extrude_modal_active = !state.extrude_modal_active;
                if state.extrude_modal_active {
                    state.extrude_modal_distance = 0.0;
                    state.extrude_modal_axis_constraint = None;
                    state.extrude_modal_start_mouse = state.last_mouse_pos.unwrap_or((0.0, 0.0));
                    state.extrude_modal_faces = if !state.selected_faces.is_empty() {
                        state.selected_faces.iter().copied().collect()
                    } else if let Some(sel_id) = state.selected_node {
                        if let Some(node) = state.scene.nodes.get(sel_id) {
                            if let NodeData::Mesh { base_mesh, .. } = &node.data {
                                base_mesh.faces.keys().take(1).collect()
                            } else { Vec::new() }
                        } else { Vec::new() }
                    } else { Vec::new() };
                }
            }
            state.window.request_redraw();
            true
        }
        KeyCode::KeyR if state.ctrl_pressed && state.interaction_mode == InteractionMode::Sculpt => {
            if let Some(sel_id) = state.selected_node {
                let cmd = Box::new(crate::commands::VoxelRemeshCommand {
                    node_id: sel_id,
                    voxel_size: 0.08,
                    old_mesh: None,
                });
                let _ = state.history.execute(cmd, &mut state.scene);
                state.window.request_redraw();
                true
            } else {
                false
            }
        }
        KeyCode::KeyR if state.ctrl_pressed && state.interaction_mode == InteractionMode::Edit => {
            state.loop_cut_active = !state.loop_cut_active;
            state.loop_cut_sliding = false;
            state.loop_cut_factor = 0.5;
            state.loop_cut_num_cuts = 1;
            state.loop_cut_hovered_edge = state.selected_edges.iter().copied().next();
            state.window.request_redraw();
            true
        }
        KeyCode::KeyB => {
            if state.alt_pressed && state.shift_pressed && state.interaction_mode == InteractionMode::Edit {
                // MeshMachine Alt+Shift+B: Un-Bevel & Dissolve Chamfer Strip
                if let Some(sel_id) = state.selected_node {
                    let target_faces: Vec<FaceKey> = state.selected_faces.iter().copied().collect();
                    if !target_faces.is_empty() {
                        let cmd = Box::new(UnBevelFacesCommand {
                            node_id: sel_id,
                            faces: target_faces.clone(),
                            old_mesh: None,
                        });
                        match state.history.execute(cmd, &mut state.scene) {
                            Ok(()) => {
                                state.sync_gpu_meshes();
                                state.set_status_message(format!("Un-Beveled {} chamfer face(s) back to crisp edge! (Ctrl+Z to Undo)", target_faces.len()));
                                state.selected_faces.clear();
                            }
                            Err(e) => {
                                state.set_status_message(format!("Un-Bevel failed: {}", e));
                            }
                        }
                    } else {
                        state.set_status_message("Select bevel/chamfer face(s) to un-bevel (Alt+Shift+B)");
                    }
                }
                state.window.request_redraw();
                return true;
            } else if state.ctrl_pressed && state.shift_pressed && state.interaction_mode == InteractionMode::Edit {
                if let Some(sel_id) = state.selected_node {
                    let target_verts: Vec<VertexKey> = if !state.selected_vertices.is_empty() {
                        state.selected_vertices.iter().copied().collect()
                    } else if let Some(node) = state.scene.nodes.get(sel_id) {
                        if let NodeData::Mesh { base_mesh, .. } = &node.data {
                            base_mesh.vertices.keys().take(1).collect()
                        } else {
                            Vec::new()
                        }
                    } else {
                        Vec::new()
                    };

                    if !target_verts.is_empty() {
                        let cmd = Box::new(BevelVertexCommand {
                            node_id: sel_id,
                            vertices: target_verts,
                            offset: state.vertex_bevel_offset,
                            old_mesh: None,
                        });
                        let _ = state.history.execute(cmd, &mut state.scene);
                        state.selected_vertices.clear();
                        state.window.request_redraw();
                    }
                }
            } else if state.ctrl_pressed && state.interaction_mode == InteractionMode::Edit {
                state.bevel_modal_active = !state.bevel_modal_active;
                if state.bevel_modal_active {
                    state.bevel_modal_width = 0.15;
                    state.bevel_modal_segments = 1;
                    state.bevel_modal_start_mouse = state.last_mouse_pos.unwrap_or((0.0, 0.0));
                    state.bevel_modal_edges = if !state.selected_edges.is_empty() {
                        state.selected_edges.iter().copied().collect()
                    } else if let Some(sel_id) = state.selected_node {
                        if let Some(node) = state.scene.nodes.get(sel_id) {
                            if let NodeData::Mesh { base_mesh, .. } = &node.data {
                                base_mesh.edges.keys().take(1).collect()
                            } else { Vec::new() }
                        } else { Vec::new() }
                    } else { Vec::new() };
                    state.bevel_modal_faces = state.selected_faces.iter().copied().collect();
                }
            } else if state.alt_pressed {
                state.selection_tool_mode = SelectionToolMode::Lasso;
                state.is_box_selecting = !state.is_box_selecting;
                state.set_status_message(if state.is_box_selecting { "Lasso Select Activated (Drag to outline)" } else { "Selection Deactivated" });
            } else if state.shift_pressed || state.interaction_mode == InteractionMode::Object || (state.selected_faces.is_empty() && state.selected_edges.is_empty()) {
                state.selection_tool_mode = SelectionToolMode::Box;
                state.is_box_selecting = !state.is_box_selecting;
                state.set_status_message(if state.is_box_selecting { "Box Select Activated (Drag rectangle)" } else { "Selection Deactivated" });
            } else if state.interaction_mode == InteractionMode::Edit {
                if let Some(sel_id) = state.selected_node {
                    let target_faces: Vec<FaceKey> = state.selected_faces.iter().copied().collect();
                    if !target_faces.is_empty() {
                        let cmd = Box::new(BevelCommand {
                            node_id: sel_id,
                            faces: target_faces,
                            offset: state.bevel_offset,
                            inset: state.bevel_inset,
                            old_mesh: None,
                        });
                        let _ = state.history.execute(cmd, &mut state.scene);
                        state.window.request_redraw();
                    }
                }
            }
            state.window.request_redraw();
            true
        }
        KeyCode::KeyT if state.ctrl_pressed && state.interaction_mode == InteractionMode::Edit => {
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

                let cmd = Box::new(TriangulateCommand {
                    node_id: sel_id,
                    faces: target_faces,
                    quad_method: state.triangulate_quad_method,
                    ngon_method: state.triangulate_ngon_method,
                    min_vertices: state.triangulate_min_vertices,
                    old_mesh: None,
                });
                let _ = state.history.execute(cmd, &mut state.scene);
                state.selected_faces.clear();
                state.window.request_redraw();
            }
            true
        }
        KeyCode::KeyV => {
            if state.shift_pressed && state.interaction_mode == InteractionMode::Edit && !state.selected_vertices.is_empty() {
                // Blender Shift+V: Interactive Vertex Crease Modal (OpenSubdiv Semi-Sharp Corners)
                if let Some(sel_id) = state.selected_node {
                    if let Some(node) = state.scene.nodes.get(sel_id) {
                        if let NodeData::Mesh { base_mesh, .. } = &node.data {
                            let verts: Vec<VertexKey> = state.selected_vertices.iter().copied().collect();
                            if !verts.is_empty() {
                                state.vertex_crease_modal_vertices = verts.clone();
                                state.vertex_crease_modal_original_weights = verts
                                    .iter()
                                    .map(|&vk| (vk, base_mesh.get_vertex_crease(vk)))
                                    .collect();
                                state.vertex_crease_modal_weight = state
                                    .vertex_crease_modal_original_weights
                                    .first()
                                    .map(|&(_, w)| w)
                                    .unwrap_or(0.0);
                                state.vertex_crease_modal_start_mouse = state.last_mouse_pos.unwrap_or((0.0, 0.0));
                                state.vertex_crease_modal_active = true;
                                state.window.request_redraw();
                            }
                        }
                    }
                }
            } else if state.interaction_mode == InteractionMode::Edit && !state.selected_vertices.is_empty() {
                if let Some(sel_id) = state.selected_node {
                    let verts: Vec<VertexKey> = state.selected_vertices.iter().copied().collect();
                    let cmd = Box::new(RipVerticesCommand {
                        node_id: sel_id,
                        vertices: verts,
                        old_mesh: None,
                        created_vertices: Vec::new(),
                    });
                    if state.history.execute(cmd, &mut state.scene).is_ok() {
                        state.window.request_redraw();
                    }
                }
            } else {
                state.snap_enabled = !state.snap_enabled || state.snap_target != SnapTarget::Vertex;
                state.snap_target = SnapTarget::Vertex;
                let msg = if state.snap_enabled {
                    "🧲 Precision Vertex Snapping (V): ON"
                } else {
                    "🧲 Precision Snapping: OFF"
                };
                state.set_status_message(msg);
                state.window.request_redraw();
            }
            true
        }
        KeyCode::KeyK if state.shift_pressed || state.interaction_mode == InteractionMode::Object => {
            if state.kitbash_modal.is_some() {
                state.kitbash_modal = None;
                state.set_status_message("Exited Kitbash Tool");
            } else {
                crate::modals::kitbash::start(state);
            }
            state.window.request_redraw();
            true
        }
        KeyCode::KeyK if state.interaction_mode == InteractionMode::Edit => {
            state.knife_active = !state.knife_active;
            if !state.knife_active {
                state.knife_points.clear();
                state.knife_hovered_hit = None;
            }
            state.window.request_redraw();
            true
        }
        KeyCode::KeyM if state.interaction_mode == InteractionMode::Sculpt => {
            if state.alt_pressed {
                if let Some(sel_id) = state.selected_node {
                    if let Some(node) = state.scene.nodes.get_mut(sel_id) {
                        if let NodeData::Mesh { base_mesh, cached_render_mesh, .. } = &mut node.data {
                            base_mesh.clear_sculpt_mask();
                            *cached_render_mesh = None;
                            state.scene.mark_dirty(sel_id);
                        }
                    }
                }
            } else {
                state.sculpt_settings.brush = geo_kernel::SculptBrush::Mask;
            }
            state.window.request_redraw();
            true
        }
        KeyCode::KeyM if state.interaction_mode == InteractionMode::Edit => {
            if let Some(sel_id) = state.selected_node {
                let target_verts: Vec<VertexKey> = if !state.selected_vertices.is_empty() {
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

                if target_verts.len() >= 2 {
                    let cmd = Box::new(MergeVerticesCommand {
                        node_id: sel_id,
                        vertices: target_verts,
                        old_mesh: None,
                    });
                    let _ = state.history.execute(cmd, &mut state.scene);
                    state.selected_vertices.clear();
                    state.selected_edges.clear();
                    state.selected_faces.clear();
                    state.window.request_redraw();
                }
            }
            true
        }
        KeyCode::KeyS if state.alt_pressed && state.interaction_mode == InteractionMode::Edit => {
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
                    state.window.request_redraw();
                }
            }
            true
        }
        KeyCode::KeyI => {
            if state.ctrl_pressed && state.interaction_mode == InteractionMode::Sculpt {
                if let Some(sel_id) = state.selected_node {
                    if let Some(node) = state.scene.nodes.get_mut(sel_id) {
                        if let NodeData::Mesh { base_mesh, cached_render_mesh, .. } = &mut node.data {
                            base_mesh.invert_sculpt_mask();
                            *cached_render_mesh = None;
                            state.scene.mark_dirty(sel_id);
                        }
                    }
                }
            } else if state.ctrl_pressed && state.interaction_mode == InteractionMode::Edit {
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
            } else if state.interaction_mode == InteractionMode::Edit {
                state.inset_modal_active = !state.inset_modal_active;
                if state.inset_modal_active {
                    state.modal_numeric_buffer.clear();
                    state.inset_modal_factor = 0.8;
                    state.inset_modal_start_mouse = state.last_mouse_pos.unwrap_or((0.0, 0.0));
                    state.inset_modal_faces = if !state.selected_faces.is_empty() {
                        state.selected_faces.iter().copied().collect()
                    } else if let Some(sel_id) = state.selected_node {
                        if let Some(node) = state.scene.nodes.get(sel_id) {
                            if let NodeData::Mesh { base_mesh, .. } = &node.data {
                                base_mesh.faces.keys().take(1).collect()
                            } else {
                                Vec::new()
                            }
                        } else {
                            Vec::new()
                        }
                    } else {
                        Vec::new()
                    };

                    let mut avg_r = 1.0_f32;
                    if let Some(sel_id) = state.selected_node {
                        if let Some(node) = state.scene.nodes.get(sel_id) {
                            if let NodeData::Mesh { base_mesh, .. } = &node.data {
                                let mut total_r = 0.0_f32;
                                let mut count = 0;
                                for &fk in &state.inset_modal_faces {
                                    let c = base_mesh.face_centroid(fk);
                                    for &vk in &base_mesh.face_vertices(fk) {
                                        total_r += (base_mesh.vertices[vk].position - c).length();
                                        count += 1;
                                    }
                                }
                                if count > 0 && total_r > 1e-4 {
                                    avg_r = total_r / count as f32;
                                }
                            }
                        }
                    }
                    state.inset_modal_initial_radius = avg_r;
                    state.inset_modal_thickness = (1.0 - state.inset_modal_factor) * avg_r;
                }
            } else if let Some(sel_id) = state.selected_node {
                if let Some(node) = state.scene.nodes.get(sel_id) {
                    let frame = state.animation.current_frame.round();
                    state.animation.insert_keyframe(sel_id, frame, node.local_transform, InterpolationMode::Linear);
                }
            }
            state.window.request_redraw();
            true
        }
        KeyCode::KeyU if state.interaction_mode == InteractionMode::Edit => {
            if let Some(sel_id) = state.selected_node {
                let cmd = Box::new(UnwrapSeamsCommand {
                    node_id: sel_id,
                    margin: state.smart_uv_margin,
                    old_mesh: None,
                });
                let _ = state.history.execute(cmd, &mut state.scene);
                state.show_uv_editor = true;
                state.window.request_redraw();
            }
            true
        }
        KeyCode::KeyN if state.alt_pressed && state.interaction_mode == InteractionMode::Edit => {
            if let Some(sel_id) = state.selected_node {
                let target_faces: Vec<FaceKey> = state.selected_faces.iter().copied().collect();
                let cmd = Box::new(FlipNormalsCommand {
                    node_id: sel_id,
                    faces: target_faces,
                    old_mesh: None,
                });
                if let Err(e) = state.history.execute(cmd, &mut state.scene) {
                    eprintln!("Flip normals failed: {}", e);
                }
            }
            state.window.request_redraw();
            true
        }
        KeyCode::KeyN if state.shift_pressed && state.interaction_mode == InteractionMode::Edit => {
            if let Some(sel_id) = state.selected_node {
                let inside = state.ctrl_pressed;
                let cmd = Box::new(RecalculateNormalsCommand {
                    node_id: sel_id,
                    inside,
                    old_mesh: None,
                });
                if let Err(e) = state.history.execute(cmd, &mut state.scene) {
                    eprintln!("Recalculate normals failed: {}", e);
                }
            }
            state.window.request_redraw();
            true
        }
        KeyCode::KeyD if state.alt_pressed => {
            // Project active floating mesh decal onto target scene hull with normal stamping
            if let Some(sel_id) = state.selected_node {
                let mut target_data = None;
                for (nid, node) in &state.scene.nodes {
                    if nid != sel_id {
                        if let NodeData::Mesh { base_mesh, .. } = &node.data {
                            target_data = Some((base_mesh.clone(), node.world_matrix));
                            break;
                        }
                    }
                }
                if let Some((hull_mesh, hull_mat)) = target_data {
                    if let Some(node) = state.scene.nodes.get_mut(sel_id) {
                        if let NodeData::Mesh { base_mesh, cached_render_mesh, .. } = &mut node.data {
                            let decal_mat = node.world_matrix;
                            if base_mesh.project_floating_decal(&hull_mesh, &decal_mat, &hull_mat, 0.0005).is_ok() {
                                *cached_render_mesh = None;
                                state.scene.mark_dirty(sel_id);
                                state.set_status_message("✨ Floating Mesh Decal Projected & Boundary Normals Stamped (Alt+D)");
                                state.window.request_redraw();
                            }
                        }
                    }
                }
            }
            true
        }
        KeyCode::KeyC if state.ctrl_pressed && state.shift_pressed => {
            let p_a = Vec3::new(-1.0, 1.0, 0.0);
            let p_b = Vec3::new(1.0, 1.0, 0.0);
            let cable_mesh = geo_kernel::CableGenerator::create_cable(p_a, p_b, 0.04, 0.25, 12, 8);
            let name = format!("HydraulicHose_{}", state.scene.nodes.len());
            let node = graph_engine::SceneNode::new_mesh(name, core_math::Transform::IDENTITY, cable_mesh);
            let id = state.scene.add_node(node);
            state.selected_node = Some(id);
            state.sync_gpu_meshes();
            state.set_status_message("✨ Dynamic Hydraulic Cable with Fittings Created (Ctrl+Shift+C)");
            state.window.request_redraw();
            true
        }
        KeyCode::KeyC if state.shift_pressed && !state.ctrl_pressed => {
            state.selection_tool_mode = SelectionToolMode::CirclePaint;
            state.is_box_selecting = !state.is_box_selecting;
            state.set_status_message(if state.is_box_selecting { "Paint / Circle Select Activated (Drag to paint)" } else { "Selection Deactivated" });
            state.window.request_redraw();
            true
        }
        KeyCode::KeyC if !state.curve_pen_active => {
            crate::modals::cutter::start(state);
            true
        }
        KeyCode::F9 => {
            state.show_last_operator_panel = !state.show_last_operator_panel;
            state.window.request_redraw();
            true
        }
        _ => false,
    }
}

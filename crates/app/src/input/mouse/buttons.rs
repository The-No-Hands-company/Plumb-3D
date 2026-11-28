use crate::commands::*;
use crate::state::AppState;
use crate::types::*;
use graph_engine::NodeData;

pub fn handle_mouse_left_down(state: &mut AppState) {
    state.mouse_drag_distance = 0.0;
    if state.push_pull_tool {
        if let Some((mx, my)) = state.last_mouse_pos {
            let ray = state.get_screen_ray(mx, my);
            let mut face_hit = None;
            if let Some(sel_id) = state.selected_node {
                if let Some(node) = state.scene.nodes.get(sel_id) {
                    if let NodeData::Mesh { base_mesh, .. } = &node.data {
                        if let Some(hit) = base_mesh.raycast(&ray, &node.world_matrix) {
                            face_hit = Some(hit.face_key);
                        }
                    }
                }
            }
            if let Some(fk) = face_hit {
                state.selected_faces.clear();
                state.selected_faces.insert(fk);
                state.extrude_modal_active = true;
                state.extrude_modal_distance = 0.0;
                state.extrude_modal_faces = vec![fk];
                state.extrude_modal_axis_constraint = None;
                state.extrude_modal_start_mouse = (mx, my);
                state.set_status_message(
                    "Direct Push-Pull: Drag along normal | LMB: Commit | Esc: Cancel",
                );
                state.window.request_redraw();
                return;
            }
        }
    }
    if state.curve_pen_active {
        if let Some((mx, my)) = state.last_mouse_pos {
            state.handle_curve_pen_click(mx, my);
            return;
        }
    }
    if state.measure_mode {
        if let Some((mx, my)) = state.last_mouse_pos {
            let ray = state.get_screen_ray(mx, my);
            let mut hit_pt = None;
            for (_nid, n) in &state.scene.nodes {
                if let NodeData::Mesh { base_mesh, .. } = &n.data {
                    if let Some(hit) = base_mesh.raycast(&ray, &n.world_matrix) {
                        hit_pt = Some(hit.point);
                        break;
                    }
                }
            }
            let pt = hit_pt.unwrap_or_else(|| {
                if ray.direction.y.abs() > 1e-4 {
                    let t = -ray.origin.y / ray.direction.y;
                    if t > 0.0 {
                        ray.origin + ray.direction * t
                    } else {
                        ray.origin + ray.direction * 5.0
                    }
                } else {
                    ray.origin + ray.direction * 5.0
                }
            });
            state.measure_points.push(pt);
            state.window.request_redraw();
        }
    } else if state.interaction_mode == InteractionMode::Sculpt {
        state.is_sculpting = true;
        if let Some(sel_id) = state.selected_node {
            if let Some(node) = state.scene.nodes.get(sel_id) {
                if let NodeData::Mesh { base_mesh, .. } = &node.data {
                    state.sculpt_stroke_start_mesh = Some(base_mesh.clone());
                }
            }
        }
        if let Some((x, y)) = state.last_mouse_pos {
            state.sculpt_step(x, y, None);
        }
    } else if state.interaction_mode == InteractionMode::WeightPaint {
        if state.weight_paint_tool == WeightPaintTool::LinearGradient {
            if let Some((x, y)) = state.last_mouse_pos {
                let ray = state.get_screen_ray(x, y);
                if let Some(sel_id) = state.selected_node {
                    if let Some(node) = state.scene.nodes.get(sel_id) {
                        if let NodeData::Mesh { base_mesh, .. } = &node.data {
                            if let Some(hit) = base_mesh.raycast(&ray, &node.world_matrix) {
                                state.weight_gradient_start = Some(hit.point);
                                state.weight_gradient_end = Some(hit.point);
                            }
                        }
                    }
                }
            }
        } else {
            state.is_weight_painting = true;
            if let Some(sel_id) = state.selected_node {
                if let Some(node) = state.scene.nodes.get(sel_id) {
                    if let NodeData::Mesh { base_mesh, .. } = &node.data {
                        state.weight_paint_stroke_start_mesh = Some(base_mesh.clone());
                    }
                }
            }
            if let Some((x, y)) = state.last_mouse_pos {
                state.weight_paint_step(x, y);
            }
        }
    } else if state.interaction_mode == InteractionMode::VertexPaint {
        state.is_vertex_painting = true;
        if let Some(sel_id) = state.selected_node {
            if let Some(node) = state.scene.nodes.get(sel_id) {
                if let NodeData::Mesh { base_mesh, .. } = &node.data {
                    state.sculpt_stroke_start_mesh = Some(base_mesh.clone());
                }
            }
        }
        if let Some((x, y)) = state.last_mouse_pos {
            state.vertex_paint_step(x, y);
        }
    } else if state.interaction_mode == InteractionMode::TexturePaint {
        state.is_texture_painting = true;
        if let Some((x, y)) = state.last_mouse_pos {
            state.texture_paint_step(x, y);
        }
    } else if state.is_box_selecting {
        if let Some((x, y)) = state.last_mouse_pos {
            let (px, py) = (x as f32, y as f32);
            match state.selection_tool_mode {
                SelectionToolMode::Box => {
                    state.box_select_start = Some((px, py));
                    state.box_select_current = Some((px, py));
                }
                SelectionToolMode::Lasso => {
                    state.lasso_points.clear();
                    state.lasso_points.push((px, py));
                }
                SelectionToolMode::CirclePaint => {
                    state.select_in_circle((px, py), state.circle_select_radius);
                }
            }
        }
    } else if let Some(axis) = state.hovered_gizmo_axis {
        state.active_gizmo_axis = Some(axis);
        if let Some(sel_id) = state.selected_node {
            if let Some(node) = state.scene.nodes.get(sel_id) {
                state.drag_start_transform = Some(node.local_transform);
            }
        }
    } else {
        state.is_orbiting = true;
    }
}

pub fn handle_mouse_left_up(state: &mut AppState) {
    if state.push_pull_tool && state.extrude_modal_active {
        if state.extrude_modal_distance.abs() > 0.001 {
            crate::modals::extrude::commit(state);
            state.set_status_message("Direct Push-Pull Extrusion Applied! (Ctrl+Z to Undo)");
        } else {
            crate::modals::extrude::cancel(state);
        }
        state.window.request_redraw();
        return;
    }
    if state.is_sculpting {
        state.is_sculpting = false;
        if let (Some(old_m), Some(sel_id)) =
            (state.sculpt_stroke_start_mesh.take(), state.selected_node)
        {
            if let Some(node) = state.scene.nodes.get(sel_id) {
                if let NodeData::Mesh { base_mesh, .. } = &node.data {
                    let brush_name = format!("Sculpt {:?}", state.sculpt_settings.brush);
                    let cmd = Box::new(SculptStrokeCommand {
                        node_id: sel_id,
                        brush_name,
                        old_mesh: old_m,
                        new_mesh: base_mesh.clone(),
                    });
                    let _ = state.history.execute(cmd, &mut state.scene);
                }
            }
        }
    } else if state.interaction_mode == InteractionMode::WeightPaint {
        if state.weight_paint_tool == WeightPaintTool::LinearGradient {
            if let (Some(start), Some(end)) = (
                state.weight_gradient_start.take(),
                state.weight_gradient_end.take(),
            ) {
                if (end - start).length_squared() > 1e-4 {
                    if let (Some(sel_id), Some(group)) =
                        (state.selected_node, state.active_vertex_group.clone())
                    {
                        let cmd = Box::new(ApplyWeightGradientCommand {
                            node_id: sel_id,
                            group_name: group,
                            start_point: start,
                            end_point: end,
                            start_weight: state.weight_gradient_start_w,
                            end_weight: state.weight_gradient_end_w,
                            old_mesh: None,
                        });
                        let _ = state.history.execute(cmd, &mut state.scene);
                    }
                }
            }
        } else if state.is_weight_painting {
            state.is_weight_painting = false;
            if let (Some(old_m), Some(sel_id)) = (
                state.weight_paint_stroke_start_mesh.take(),
                state.selected_node,
            ) {
                if let Some(node) = state.scene.nodes.get(sel_id) {
                    if let NodeData::Mesh { base_mesh, .. } = &node.data {
                        let cmd = Box::new(WeightPaintStrokeCommand {
                            node_id: sel_id,
                            old_mesh: old_m,
                            new_mesh: base_mesh.clone(),
                        });
                        let _ = state.history.execute(cmd, &mut state.scene);
                    }
                }
            }
        }
    } else if state.interaction_mode == InteractionMode::VertexPaint {
        state.is_vertex_painting = false;
        if let (Some(old_m), Some(sel_id)) =
            (state.sculpt_stroke_start_mesh.take(), state.selected_node)
        {
            if let Some(node) = state.scene.nodes.get(sel_id) {
                if let NodeData::Mesh { base_mesh, .. } = &node.data {
                    let cmd = Box::new(VertexPaintStrokeCommand {
                        node_id: sel_id,
                        old_mesh: old_m,
                        new_mesh: base_mesh.clone(),
                    });
                    let _ = state.history.execute(cmd, &mut state.scene);
                }
            }
        }
    } else if state.interaction_mode == InteractionMode::TexturePaint {
        state.is_texture_painting = false;
    } else if state.is_box_selecting {
        match state.selection_tool_mode {
            SelectionToolMode::Box => {
                if let (Some(start), Some(curr)) = (
                    state.box_select_start.take(),
                    state.box_select_current.take(),
                ) {
                    if (start.0 - curr.0).abs() > 3.0 || (start.1 - curr.1).abs() > 3.0 {
                        state.select_in_marquee_box(start, curr);
                    }
                }
                state.is_box_selecting = false;
            }
            SelectionToolMode::Lasso => {
                if state.lasso_points.len() >= 3 {
                    let pts = std::mem::take(&mut state.lasso_points);
                    state.select_in_lasso(&pts);
                } else {
                    state.lasso_points.clear();
                }
                state.is_box_selecting = false;
            }
            SelectionToolMode::CirclePaint => {
                // Circle select remains active for continuous paint brushing until toggled off
            }
        }
    } else if let Some(start_t) = state.drag_start_transform.take() {
        if let Some(sel_id) = state.selected_node {
            if let Some(node) = state.scene.nodes.get(sel_id) {
                let end_t = node.local_transform;
                if (start_t.translation - end_t.translation).length() > 1e-4
                    || (start_t.scale - end_t.scale).length() > 1e-4
                    || start_t.rotation != end_t.rotation
                {
                    let cmd = Box::new(TransformCommand {
                        node_id: sel_id,
                        old_transform: start_t,
                        new_transform: end_t,
                    });
                    let _ = state.history.execute(cmd, &mut state.scene);
                }
            }
        }
    }

    state.active_gizmo_axis = None;
    state.is_orbiting = false;
    state.last_snap_hit = None;
    if state.curve_pen_active {
        state.curve_pen_drag_origin = None;
    }

    if !crate::modals::is_any_modal_active(state)
        && !state.measure_mode
        && !state.curve_pen_active
        && state.mouse_drag_distance < 5.0
    {
        if let Some((x, y)) = state.last_mouse_pos {
            state.pick_at_screen_pos(x, y);
        }
    }
}

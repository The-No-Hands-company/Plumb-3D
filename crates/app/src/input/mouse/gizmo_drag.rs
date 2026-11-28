use crate::state::AppState;
use crate::types::*;
use core_math::{Quat, Vec3};
use geo_kernel::VertexKey;
use graph_engine::NodeData;
use render_viewport::{GizmoAxis, GizmoMode, TransformGizmo};
use std::collections::HashSet;

pub fn handle_gizmo_drag(state: &mut AppState, axis: GizmoAxis, dx: f32, dy: f32, move_dist: f32) {
    state.mouse_drag_distance += move_dist;
    let Some(sel_id) = state.selected_node else {
        state.window.request_redraw();
        return;
    };

    let forward = (state.camera.target - state.camera.eye_position()).normalize();
    let right = forward.cross(Vec3::Y).normalize();
    let up = right.cross(forward).normalize();
    let dist_to_cam = (state.camera.eye_position() - state.camera.target).length();

    match state.gizmo.mode {
        GizmoMode::Translate => {
            let gizmo_orient = state.get_gizmo_orientation();
            let mut delta = TransformGizmo::calculate_translation_delta(
                axis,
                gizmo_orient,
                right,
                up,
                dx,
                dy,
                dist_to_cam,
            );
            let snap_active = state.snap_enabled ^ state.ctrl_pressed;
            if snap_active {
                let current_pivot = state.get_gizmo_world_position().unwrap_or(Vec3::ZERO);
                let proposed_pos = current_pivot + delta;
                let mut hit_info = None;
                let mut snapped_pos = proposed_pos;
                match state.snap_target {
                    SnapTarget::Grid => {
                        let s = state.grid_snap_size.max(0.001);
                        snapped_pos.x = (snapped_pos.x / s).round() * s;
                        snapped_pos.y = (snapped_pos.y / s).round() * s;
                        snapped_pos.z = (snapped_pos.z / s).round() * s;
                        hit_info = Some((snapped_pos, "Grid"));
                    }
                    SnapTarget::Vertex => {
                        if let Some(hit) = state.find_closest_snap_vertex(
                            proposed_pos,
                            state.snap_distance_threshold,
                            &state.selected_vertices,
                        ) {
                            snapped_pos = hit;
                            hit_info = Some((hit, "Vertex"));
                        }
                    }
                    SnapTarget::EdgeMidpoint => {
                        if let Some(hit) = state.find_closest_snap_edge_midpoint(
                            proposed_pos,
                            state.snap_distance_threshold,
                        ) {
                            snapped_pos = hit;
                            hit_info = Some((hit, "Edge Midpoint"));
                        }
                    }
                    SnapTarget::FaceCenter => {
                        if let Some(hit) = state.find_closest_snap_face_center(
                            proposed_pos,
                            state.snap_distance_threshold,
                        ) {
                            snapped_pos = hit;
                            hit_info = Some((hit, "Face Center"));
                        }
                    }
                }
                if let Some((hit_pos, label)) = hit_info {
                    state.last_snap_hit = Some((hit_pos, label));
                    match axis {
                        GizmoAxis::X => delta.x = snapped_pos.x - current_pivot.x,
                        GizmoAxis::Y => delta.y = snapped_pos.y - current_pivot.y,
                        GizmoAxis::Z => delta.z = snapped_pos.z - current_pivot.z,
                        _ => delta = snapped_pos - current_pivot,
                    }
                } else {
                    state.last_snap_hit = None;
                }
            } else if state.snap_to_grid && state.grid_snap_size > 1e-4 {
                let s = state.grid_snap_size;
                delta.x = (delta.x / s).round() * s;
                delta.y = (delta.y / s).round() * s;
                delta.z = (delta.z / s).round() * s;
            }

            state.set_status_message(format!(
                "⬌ Move: X: {:+.3}m, Y: {:+.3}m, Z: {:+.3}m (Dist: {:.3}m)",
                delta.x,
                delta.y,
                delta.z,
                delta.length()
            ));

            if state.interaction_mode == InteractionMode::Object
                || state.interaction_mode == InteractionMode::Pose
            {
                if state.selected_nodes.len() > 1 {
                    for &nid in &state.selected_nodes {
                        if let Some(node) = state.scene.nodes.get_mut(nid) {
                            node.local_transform.translation += delta;
                            state.scene.mark_dirty(nid);
                        }
                    }
                } else if let Some(node) = state.scene.nodes.get_mut(sel_id) {
                    node.local_transform.translation += delta;
                    state.scene.mark_dirty(sel_id);
                }
            } else if state.interaction_mode == InteractionMode::CurveEdit {
                if let Some(node) = state.scene.nodes.get_mut(sel_id) {
                    if let NodeData::Curve {
                        spline,
                        cached_render_mesh,
                        ..
                    } = &mut node.data
                    {
                        if let Some(k_idx) = state.selected_curve_knot {
                            if let Some(pt) = spline.points.get_mut(k_idx) {
                                let inv_mat = node.world_matrix.inverse();
                                let local_delta = inv_mat.transform_vector3(delta);
                                match state.selected_curve_handle {
                                    None | Some(CurveHandleSelection::Knot) => {
                                        pt.position += local_delta;
                                    }
                                    Some(CurveHandleSelection::HandleIn) => {
                                        pt.handle_in += local_delta;
                                    }
                                    Some(CurveHandleSelection::HandleOut) => {
                                        pt.handle_out += local_delta;
                                    }
                                }
                                *cached_render_mesh = None;
                                state.scene.mark_dirty(sel_id);
                            }
                        }
                    }
                }
            } else if let Some(node) = state.scene.nodes.get_mut(sel_id) {
                if let NodeData::Mesh { base_mesh, .. } = &mut node.data {
                    let mut direct_verts: HashSet<VertexKey> = HashSet::new();
                    match state.sub_element_level {
                        SubElementLevel::Vertex => {
                            direct_verts = state.selected_vertices.clone();
                        }
                        SubElementLevel::Edge => {
                            for &ek in &state.selected_edges {
                                if let Some(edge) = base_mesh.edges.get(ek) {
                                    let he = &base_mesh.half_edges[edge.half_edge];
                                    direct_verts.insert(he.origin);
                                    direct_verts.insert(he.target);
                                }
                            }
                        }
                        SubElementLevel::Face => {
                            for &fk in &state.selected_faces {
                                for vk in base_mesh.face_vertices(fk) {
                                    direct_verts.insert(vk);
                                }
                            }
                        }
                    }

                    if !direct_verts.is_empty() {
                        let mut centroid = Vec3::ZERO;
                        for &vk in &direct_verts {
                            if let Some(v) = base_mesh.vertices.get(vk) {
                                centroid += v.position;
                            }
                        }
                        centroid /= direct_verts.len() as f32;

                        let inv_mat = node.world_matrix.inverse();
                        let local_delta = inv_mat.transform_vector3(delta);
                        let r_prop = state.proportional_radius;
                        let use_prop = state.proportional_editing && r_prop > 1e-4;

                        // Compute primary vertex deltas
                        let mut vert_deltas: Vec<(VertexKey, Vec3)> = Vec::new();
                        for (vk, v) in &base_mesh.vertices {
                            let is_direct = direct_verts.contains(&vk);
                            let weight = if is_direct {
                                1.0f32
                            } else if use_prop {
                                let d = (v.position - centroid).length();
                                if d <= r_prop {
                                    let t = 1.0 - (d / r_prop);
                                    t * t * (3.0 - 2.0 * t)
                                } else {
                                    0.0
                                }
                            } else {
                                0.0
                            };

                            if weight > 0.0 {
                                vert_deltas.push((vk, local_delta * weight));
                            }
                        }

                        // If X-Mirror symmetry editing is active, find counterpart vertices across X=0 plane
                        if state.x_mirror_editing {
                            let mut mirror_deltas: Vec<(VertexKey, Vec3)> = Vec::new();
                            let moved_keys: HashSet<VertexKey> =
                                vert_deltas.iter().map(|(vk, _)| *vk).collect();
                            let sym_threshold = 0.01_f32;

                            for &(vk, d) in &vert_deltas {
                                if let Some(v) = base_mesh.vertices.get(vk) {
                                    let orig_pos = v.position;
                                    if orig_pos.x.abs() < sym_threshold {
                                        // On symmetry plane: keep x delta 0 to stay centered
                                        continue;
                                    }
                                    let target_mirror =
                                        Vec3::new(-orig_pos.x, orig_pos.y, orig_pos.z);
                                    let mut best_key = None;
                                    let mut best_dist_sq = sym_threshold * sym_threshold;
                                    for (other_vk, other_v) in &base_mesh.vertices {
                                        if other_vk == vk || moved_keys.contains(&other_vk) {
                                            continue;
                                        }
                                        let dist_sq =
                                            (other_v.position - target_mirror).length_squared();
                                        if dist_sq < best_dist_sq {
                                            best_dist_sq = dist_sq;
                                            best_key = Some(other_vk);
                                        }
                                    }
                                    if let Some(sym_vk) = best_key {
                                        let sym_delta = Vec3::new(-d.x, d.y, d.z);
                                        mirror_deltas.push((sym_vk, sym_delta));
                                    }
                                }
                            }
                            vert_deltas.extend(mirror_deltas);
                        }

                        for (vk, d) in vert_deltas {
                            if let Some(v) = base_mesh.vertices.get_mut(vk) {
                                v.position += d;
                                if state.x_mirror_editing && v.position.x.abs() < 1e-4 {
                                    v.position.x = 0.0;
                                }
                            }
                        }
                        base_mesh.update_normals();
                        state.scene.mark_dirty(sel_id);
                    }
                }
            }
        }
        GizmoMode::Rotate => {
            let gizmo_orient = state.get_gizmo_orientation();
            let mut delta_rot =
                TransformGizmo::calculate_rotation_delta(axis, gizmo_orient, dx, dy);
            let angle_snap_active = state.snap_angle || state.ctrl_pressed || state.shift_pressed;
            if angle_snap_active {
                let snap_deg = if state.ctrl_pressed && state.shift_pressed {
                    90.0
                } else if state.shift_pressed {
                    45.0
                } else if state.ctrl_pressed {
                    15.0
                } else {
                    state.angle_snap_degrees.max(1.0)
                };
                let step = snap_deg.to_radians();
                let (rot_axis, angle) = delta_rot.to_axis_angle();
                let snapped_angle = (angle / step).round() * step;
                delta_rot = Quat::from_axis_angle(rot_axis, snapped_angle);

                let pivot = state.get_gizmo_world_position().unwrap_or(Vec3::ZERO);
                state.last_snap_hit = Some((pivot, "Angle"));
            } else {
                state.last_snap_hit = None;
            }

            let (_, rot_angle) = delta_rot.to_axis_angle();
            state.set_status_message(format!(
                "↻ Rotate: {:+.1}° ({:?})",
                rot_angle.to_degrees(),
                axis
            ));
            if state.interaction_mode == InteractionMode::Object
                || state.interaction_mode == InteractionMode::Pose
            {
                if state.selected_nodes.len() > 1 {
                    let pivot = state.get_gizmo_world_position().unwrap_or(Vec3::ZERO);
                    for &nid in &state.selected_nodes {
                        if let Some(node) = state.scene.nodes.get_mut(nid) {
                            let pos = node.world_matrix.transform_point3(Vec3::ZERO);
                            let new_pos = pivot + delta_rot * (pos - pivot);
                            node.local_transform.translation += new_pos - pos;
                            node.local_transform.rotation =
                                (delta_rot * node.local_transform.rotation).normalize();
                            state.scene.mark_dirty(nid);
                        }
                    }
                } else if let Some(node) = state.scene.nodes.get_mut(sel_id) {
                    node.local_transform.rotation =
                        (delta_rot * node.local_transform.rotation).normalize();
                    state.scene.mark_dirty(sel_id);
                }
            } else if let Some(node) = state.scene.nodes.get_mut(sel_id) {
                if let NodeData::Mesh { base_mesh, .. } = &mut node.data {
                    let mut direct_verts: HashSet<VertexKey> = HashSet::new();
                    match state.sub_element_level {
                        SubElementLevel::Vertex => {
                            direct_verts = state.selected_vertices.clone();
                        }
                        SubElementLevel::Edge => {
                            for &ek in &state.selected_edges {
                                if let Some(edge) = base_mesh.edges.get(ek) {
                                    let he = &base_mesh.half_edges[edge.half_edge];
                                    direct_verts.insert(he.origin);
                                    direct_verts.insert(he.target);
                                }
                            }
                        }
                        SubElementLevel::Face => {
                            for &fk in &state.selected_faces {
                                for vk in base_mesh.face_vertices(fk) {
                                    direct_verts.insert(vk);
                                }
                            }
                        }
                    }

                    if !direct_verts.is_empty() {
                        let mut centroid = Vec3::ZERO;
                        for &vk in &direct_verts {
                            if let Some(v) = base_mesh.vertices.get(vk) {
                                centroid += v.position;
                            }
                        }
                        centroid /= direct_verts.len() as f32;

                        let inv_rot = node.local_transform.rotation.inverse();
                        let local_rot =
                            (inv_rot * delta_rot * node.local_transform.rotation).normalize();
                        let r_prop = state.proportional_radius;
                        let use_prop = state.proportional_editing && r_prop > 1e-4;

                        for (vk, v) in &mut base_mesh.vertices {
                            let is_direct = direct_verts.contains(&vk);
                            let weight = if is_direct {
                                1.0f32
                            } else if use_prop {
                                let d = (v.position - centroid).length();
                                if d <= r_prop {
                                    let t = 1.0 - (d / r_prop);
                                    t * t * (3.0 - 2.0 * t)
                                } else {
                                    0.0
                                }
                            } else {
                                0.0
                            };

                            if weight > 0.0 {
                                let q = Quat::IDENTITY.slerp(local_rot, weight);
                                v.position = centroid + q * (v.position - centroid);
                            }
                        }
                        base_mesh.update_normals();
                        state.scene.mark_dirty(sel_id);
                    }
                }
            }
        }
        GizmoMode::Scale => {
            let delta_scale = TransformGizmo::calculate_scale_delta(axis, dx, dy);
            let total_scale = Vec3::ONE + delta_scale;
            state.set_status_message(format!(
                "⤢ Scale: [{:.3}, {:.3}, {:.3}] ({:?})",
                total_scale.x, total_scale.y, total_scale.z, axis
            ));
            if state.interaction_mode == InteractionMode::Object
                || state.interaction_mode == InteractionMode::Pose
            {
                if state.selected_nodes.len() > 1 {
                    let pivot = state.get_gizmo_world_position().unwrap_or(Vec3::ZERO);
                    let scale_factor = Vec3::ONE + delta_scale;
                    for &nid in &state.selected_nodes {
                        if let Some(node) = state.scene.nodes.get_mut(nid) {
                            let pos = node.world_matrix.transform_point3(Vec3::ZERO);
                            let new_pos = pivot + (pos - pivot) * scale_factor;
                            node.local_transform.translation += new_pos - pos;
                            node.local_transform.scale =
                                (node.local_transform.scale * scale_factor).max(Vec3::splat(0.01));
                            state.scene.mark_dirty(nid);
                        }
                    }
                } else if let Some(node) = state.scene.nodes.get_mut(sel_id) {
                    node.local_transform.scale =
                        (node.local_transform.scale + delta_scale).max(Vec3::splat(0.01));
                    state.scene.mark_dirty(sel_id);
                }
            } else if let Some(node) = state.scene.nodes.get_mut(sel_id) {
                if let NodeData::Mesh { base_mesh, .. } = &mut node.data {
                    let mut direct_verts: HashSet<VertexKey> = HashSet::new();
                    match state.sub_element_level {
                        SubElementLevel::Vertex => {
                            direct_verts = state.selected_vertices.clone();
                        }
                        SubElementLevel::Edge => {
                            for &ek in &state.selected_edges {
                                if let Some(edge) = base_mesh.edges.get(ek) {
                                    let he = &base_mesh.half_edges[edge.half_edge];
                                    direct_verts.insert(he.origin);
                                    direct_verts.insert(he.target);
                                }
                            }
                        }
                        SubElementLevel::Face => {
                            for &fk in &state.selected_faces {
                                for vk in base_mesh.face_vertices(fk) {
                                    direct_verts.insert(vk);
                                }
                            }
                        }
                    }

                    if !direct_verts.is_empty() {
                        let mut centroid = Vec3::ZERO;
                        for &vk in &direct_verts {
                            if let Some(v) = base_mesh.vertices.get(vk) {
                                centroid += v.position;
                            }
                        }
                        centroid /= direct_verts.len() as f32;

                        let factor = Vec3::ONE + delta_scale;
                        let r_prop = state.proportional_radius;
                        let use_prop = state.proportional_editing && r_prop > 1e-4;

                        for (vk, v) in &mut base_mesh.vertices {
                            let is_direct = direct_verts.contains(&vk);
                            let weight = if is_direct {
                                1.0f32
                            } else if use_prop {
                                let d = (v.position - centroid).length();
                                if d <= r_prop {
                                    let t = 1.0 - (d / r_prop);
                                    t * t * (3.0 - 2.0 * t)
                                } else {
                                    0.0
                                }
                            } else {
                                0.0
                            };

                            if weight > 0.0 {
                                let s = Vec3::ONE.lerp(factor, weight);
                                v.position = centroid + (v.position - centroid) * s;
                            }
                        }
                        base_mesh.update_normals();
                        state.scene.mark_dirty(sel_id);
                    }
                }
            }
        }
    }
    state.window.request_redraw();
}

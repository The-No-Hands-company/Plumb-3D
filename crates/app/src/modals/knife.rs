use geo_kernel::CutPoint;
use graph_engine::NodeData;
use winit::dpi::PhysicalPosition;
use winit::event::MouseButton;
use winit::keyboard::KeyCode;

use crate::commands::KnifeCutCommand;
use crate::state::AppState;

pub fn commit(state: &mut AppState) {
    if state.knife_points.len() >= 2 {
        if let Some(sel_id) = state.selected_node {
            let mut cuts = Vec::new();
            for i in 0..state.knife_points.len() - 1 {
                let (p_start, _, f_start) = state.knife_points[i];
                let (p_end, _, f_end) = state.knife_points[i + 1];
                let common_face = if f_start == f_end {
                    Some(f_start)
                } else if let Some(node) = state.scene.nodes.get(sel_id) {
                    if let NodeData::Mesh { base_mesh, .. } = &node.data {
                        let p_start_faces = match p_start {
                            CutPoint::Vertex(vk) => base_mesh.vertex_faces(vk),
                            CutPoint::Edge(ek, _) => base_mesh.edge_faces(ek),
                        };
                        let p_end_faces = match p_end {
                            CutPoint::Vertex(vk) => base_mesh.vertex_faces(vk),
                            CutPoint::Edge(ek, _) => base_mesh.edge_faces(ek),
                        };
                        p_start_faces.into_iter().find(|fk| p_end_faces.contains(fk))
                    } else {
                        None
                    }
                } else {
                    None
                };

                if let Some(fk) = common_face {
                    cuts.push((fk, p_start, p_end));
                }
            }
            if !cuts.is_empty() {
                let cmd = Box::new(KnifeCutCommand {
                    node_id: sel_id,
                    cuts,
                    old_mesh: None,
                });
                let _ = state.history.execute(cmd, &mut state.scene);
            }
        }
    }
    state.knife_active = false;
    state.knife_points.clear();
    state.knife_hovered_hit = None;
    state.window.request_redraw();
}

pub fn cancel(state: &mut AppState) {
    state.knife_points.clear();
    state.knife_hovered_hit = None;
    state.knife_active = false;
    state.window.request_redraw();
}

pub fn handle_mouse_input(state: &mut AppState, button: MouseButton, is_pressed: bool) -> bool {
    if !state.knife_active {
        return false;
    }
    if button == MouseButton::Left && is_pressed {
        if let Some(hit) = state.knife_hovered_hit {
            state.knife_points.push(hit);
            state.window.request_redraw();
        }
        return true;
    }
    if button == MouseButton::Right && is_pressed {
        cancel(state);
        return true;
    }
    true
}

pub fn handle_cursor_moved(state: &mut AppState, position: PhysicalPosition<f64>) -> bool {
    if !state.knife_active {
        return false;
    }
    let ray = state.get_screen_ray(position.x, position.y);
    if let Some(sel_id) = state.selected_node {
        if let Some(node) = state.scene.nodes.get(sel_id) {
            if let NodeData::Mesh { base_mesh, .. } = &node.data {
                if let Some(hit) = base_mesh.raycast(&ray, &node.world_matrix) {
                    let inv_mat = node.world_matrix.inverse();
                    let local_pt = inv_mat.transform_point3(hit.point);
                    let f_verts = base_mesh.face_vertices(hit.face_key);

                    let mut closest_vert = None;
                    let mut min_v_dist = f32::MAX;
                    for &vk in &f_verts {
                        if let Some(v) = base_mesh.vertices.get(vk) {
                            let d = (v.position - local_pt).length();
                            if d < min_v_dist {
                                min_v_dist = d;
                                closest_vert = Some(vk);
                            }
                        }
                    }

                    if let (true, Some(vk)) = (min_v_dist < 0.15, closest_vert) {
                        let world_v = node.world_matrix.transform_point3(base_mesh.vertices[vk].position);
                        state.knife_hovered_hit = Some((CutPoint::Vertex(vk), world_v, hit.face_key));
                    } else {
                        let f_hes = base_mesh.face_half_edges(hit.face_key);
                        let mut closest_edge = None;
                        let mut min_e_dist = f32::MAX;
                        let mut best_t = 0.5;
                        let mut best_pos = local_pt;

                        for &he_key in &f_hes {
                            let he = &base_mesh.half_edges[he_key];
                            if let (Some(v0), Some(v1)) = (base_mesh.vertices.get(he.origin), base_mesh.vertices.get(he.target)) {
                                let p0 = v0.position;
                                let p1 = v1.position;
                                let seg = p1 - p0;
                                let seg_len_sq = seg.length_squared();
                                let t = if seg_len_sq > 1e-6 {
                                    ((local_pt - p0).dot(seg) / seg_len_sq).clamp(0.01, 0.99)
                                } else {
                                    0.5
                                };
                                let proj = p0 + seg * t;
                                let d = (local_pt - proj).length();
                                if d < min_e_dist {
                                    min_e_dist = d;
                                    closest_edge = Some(he.edge);
                                    best_t = t;
                                    best_pos = proj;
                                }
                            }
                        }

                        if let Some(ek) = closest_edge {
                            let world_e = node.world_matrix.transform_point3(best_pos);
                            state.knife_hovered_hit = Some((CutPoint::Edge(ek, best_t), world_e, hit.face_key));
                        } else {
                            state.knife_hovered_hit = None;
                        }
                    }
                } else {
                    state.knife_hovered_hit = None;
                }
            }
        }
    }
    state.last_mouse_pos = Some((position.x, position.y));
    state.window.request_redraw();
    true
}

pub fn handle_keyboard_input(state: &mut AppState, key_code: KeyCode) -> bool {
    if !state.knife_active {
        return false;
    }
    match key_code {
        KeyCode::Enter | KeyCode::NumpadEnter => {
            commit(state);
            true
        }
        KeyCode::Escape => {
            cancel(state);
            true
        }
        _ => false,
    }
}

use graph_engine::NodeData;
use winit::dpi::PhysicalPosition;
use winit::event::MouseButton;
use winit::keyboard::KeyCode;

use crate::commands::LoopCutCommand;
use crate::state::AppState;
use crate::types::LastOperatorInfo;

pub fn commit(state: &mut AppState) {
    if let (Some(sel_id), Some(edge)) = (state.selected_node, state.loop_cut_hovered_edge) {
        let cuts = state.loop_cut_num_cuts;
        let factor = state.loop_cut_factor;
        for k in 0..cuts {
            let t = if cuts == 1 {
                factor
            } else {
                let base = (k + 1) as f32 / (cuts + 1) as f32;
                (base + (factor - 0.5) * 0.4).clamp(0.05, 0.95)
            };
            let cmd = Box::new(LoopCutCommand {
                node_id: sel_id,
                edge_key: edge,
                ratio: t,
                old_mesh: None,
            });
            let _ = state.history.execute(cmd, &mut state.scene);
        }
        state.last_operator = Some(LastOperatorInfo::LoopCut {
            edge,
            num_cuts: cuts,
            factor,
        });
    }
    state.loop_cut_active = false;
    state.loop_cut_sliding = false;
    state.loop_cut_hovered_edge = None;
    state.window.request_redraw();
}

pub fn cancel(state: &mut AppState) {
    state.loop_cut_active = false;
    state.loop_cut_sliding = false;
    state.loop_cut_hovered_edge = None;
    state.window.request_redraw();
}

pub fn handle_mouse_input(state: &mut AppState, button: MouseButton, is_pressed: bool) -> bool {
    if !state.loop_cut_active {
        return false;
    }
    if button == MouseButton::Left && is_pressed {
        if !state.loop_cut_sliding {
            if state.loop_cut_hovered_edge.is_some() {
                state.loop_cut_sliding = true;
                if let Some((mx, _)) = state.last_mouse_pos {
                    state.loop_cut_mouse_start_x = mx;
                }
            }
        } else {
            commit(state);
        }
        state.window.request_redraw();
        return true;
    }
    if button == MouseButton::Right && is_pressed {
        cancel(state);
        return true;
    }
    true
}

pub fn handle_cursor_moved(state: &mut AppState, position: PhysicalPosition<f64>) -> bool {
    if !state.loop_cut_active {
        return false;
    }
    let ray = state.get_screen_ray(position.x, position.y);
    if let Some(sel_id) = state.selected_node {
        if let Some(node) = state.scene.nodes.get(sel_id) {
            if let NodeData::Mesh { base_mesh, .. } = &node.data {
                if !state.loop_cut_sliding {
                    if let Some(hit) = base_mesh.raycast(&ray, &node.world_matrix) {
                        let inv_mat = node.world_matrix.inverse();
                        let local_pt = inv_mat.transform_point3(hit.point);
                        let f_hes = base_mesh.face_half_edges(hit.face_key);

                        let mut closest_edge = None;
                        let mut min_e_dist = f32::MAX;
                        for &he_key in &f_hes {
                            let he = &base_mesh.half_edges[he_key];
                            if let (Some(v0), Some(v1)) = (base_mesh.vertices.get(he.origin), base_mesh.vertices.get(he.target)) {
                                let p0 = v0.position;
                                let p1 = v1.position;
                                let seg = p1 - p0;
                                let seg_len_sq = seg.length_squared();
                                let t = if seg_len_sq > 1e-6 {
                                    ((local_pt - p0).dot(seg) / seg_len_sq).clamp(0.0, 1.0)
                                } else {
                                    0.5
                                };
                                let proj = p0 + seg * t;
                                let d = (local_pt - proj).length();
                                if d < min_e_dist {
                                    min_e_dist = d;
                                    closest_edge = Some(he.edge);
                                }
                            }
                        }
                        state.loop_cut_hovered_edge = closest_edge;
                    } else {
                        state.loop_cut_hovered_edge = None;
                    }
                } else {
                    let dx = (position.x - state.loop_cut_mouse_start_x) as f32;
                    state.loop_cut_factor = (0.5 + dx * 0.003).clamp(0.05, 0.95);
                }
            }
        }
    }
    state.last_mouse_pos = Some((position.x, position.y));
    state.window.request_redraw();
    true
}

pub fn handle_mouse_wheel(state: &mut AppState, scroll: f32) -> bool {
    if !state.loop_cut_active {
        return false;
    }
    if !state.loop_cut_sliding {
        if scroll > 0.0 {
            state.loop_cut_num_cuts = (state.loop_cut_num_cuts + 1).min(8);
        } else if scroll < 0.0 {
            state.loop_cut_num_cuts = state.loop_cut_num_cuts.saturating_sub(1).max(1);
        }
        state.window.request_redraw();
    }
    true
}

pub fn handle_keyboard_input(state: &mut AppState, key_code: KeyCode) -> bool {
    if !state.loop_cut_active {
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

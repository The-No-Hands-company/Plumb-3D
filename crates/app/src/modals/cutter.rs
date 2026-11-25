use core_math::{Vec2, Vec3, Workplane};
use geo_kernel::{BooleanOp, Mesh, MeshError};
use graph_engine::NodeData;
use winit::dpi::PhysicalPosition;
use winit::event::MouseButton;
use winit::keyboard::KeyCode;

use crate::state::AppState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CutterShape {
    Box,
    Circle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CutterStage {
    DrawFootprint,
    ExtrudeDepth,
}

#[derive(Debug, Clone)]
pub struct CutterModalState {
    pub shape: CutterShape,
    pub stage: CutterStage,
    pub workplane: Workplane,
    pub start_pos: Vec3,
    pub current_pos: Vec3,
    pub start_uv: Vec2,
    pub current_uv: Vec2,
    pub depth: f32,
    pub operation: BooleanOp,
    pub drag_start_mouse_y: f64,
}

impl Default for CutterModalState {
    fn default() -> Self {
        Self {
            shape: CutterShape::Box,
            stage: CutterStage::DrawFootprint,
            workplane: Workplane::default(),
            start_pos: Vec3::ZERO,
            current_pos: Vec3::ZERO,
            start_uv: Vec2::ZERO,
            current_uv: Vec2::ZERO,
            depth: 0.2,
            operation: BooleanOp::Difference,
            drag_start_mouse_y: 0.0,
        }
    }
}

pub fn start(state: &mut AppState) {
    if state.selected_node.is_none() {
        state.set_status_message("Select an object first to perform precision cuts");
        return;
    }

    let workplane = if state.workplane.is_active {
        state.workplane
    } else if let Some(sel_id) = state.selected_node {
        if let Some(node) = state.scene.nodes.get(sel_id) {
            if let NodeData::Mesh { base_mesh, .. } = &node.data {
                if !state.selected_faces.is_empty() {
                    let &fk = state.selected_faces.iter().next().unwrap();
                    let world_mat = node.local_transform.to_matrix();
                    let pos = world_mat.transform_point3(base_mesh.face_centroid(fk));
                    let n = (node.local_transform.rotation * base_mesh.faces[fk].normal).normalize_or_zero();
                    Workplane::from_point_and_normal(pos, n)
                } else {
                    Workplane::from_point_and_normal(node.local_transform.translation, node.local_transform.rotation * Vec3::Y)
                }
            } else {
                Workplane::from_point_and_normal(Vec3::ZERO, Vec3::Y)
            }
        } else {
            Workplane::from_point_and_normal(Vec3::ZERO, Vec3::Y)
        }
    } else {
        Workplane::from_point_and_normal(Vec3::ZERO, Vec3::Y)
    };

    state.cutter_modal = Some(CutterModalState {
        workplane,
        ..Default::default()
    });

    state.set_status_message("Cutter Active (C): Click & Drag footprint | Tab: Shape | Enter: Cut | Esc: Cancel");
    state.window.request_redraw();
}

pub fn cancel(state: &mut AppState) {
    state.cutter_modal = None;
    state.window.request_redraw();
}

pub fn commit(state: &mut AppState) {
    let cutter = match state.cutter_modal.take() {
        Some(c) => c,
        None => return,
    };

    let sel_id = match state.selected_node {
        Some(id) => id,
        None => return,
    };

    let cutter_mesh = match cutter.shape {
        CutterShape::Box => build_box_cutter(&cutter.workplane, cutter.start_uv, cutter.current_uv, cutter.depth),
        CutterShape::Circle => {
            let radius = (cutter.current_uv - cutter.start_uv).length();
            build_circle_cutter(&cutter.workplane, cutter.start_uv, radius, cutter.depth, 24)
        }
    };

    let cutter_mesh = match cutter_mesh {
        Ok(m) => m,
        Err(_) => return,
    };

    if let Some(node) = state.scene.nodes.get_mut(sel_id) {
        if let NodeData::Mesh { base_mesh, .. } = &mut node.data {
            // Transform cutter mesh to node's local coordinate system
            let inv_mat = node.local_transform.to_matrix().inverse();
            let mut local_cutter = cutter_mesh.clone();
            for v in local_cutter.vertices.values_mut() {
                v.position = inv_mat.transform_point3(v.position);
            }
            local_cutter.update_normals();

            if let Ok(mut new_mesh) = base_mesh.boolean(&local_cutter, cutter.operation) {
                new_mesh.update_face_weighted_normals(true);
                *base_mesh = new_mesh;
                state.sync_gpu_meshes();
                state.scene.mark_dirty(sel_id);
                state.set_status_message("Hard-Surface Cut Applied! (Ctrl+Z to Undo)");
            }
        }
    }

    state.window.request_redraw();
}

pub fn handle_mouse_input(state: &mut AppState, button: MouseButton, is_pressed: bool) -> bool {
    let cutter = match &mut state.cutter_modal {
        Some(c) => c,
        None => return false,
    };

    if button == MouseButton::Right && is_pressed {
        cancel(state);
        return true;
    }

    if button == MouseButton::Left {
        if is_pressed {
            if cutter.stage == CutterStage::ExtrudeDepth {
                commit(state);
                return true;
            }
            // In DrawFootprint: start drag
            cutter.start_uv = cutter.current_uv;
            cutter.start_pos = cutter.current_pos;
        } else {
            // Left mouse released after drawing footprint -> advance to ExtrudeDepth
            if cutter.stage == CutterStage::DrawFootprint {
                let dist = (cutter.current_uv - cutter.start_uv).length();
                if dist > 0.01 {
                    cutter.stage = CutterStage::ExtrudeDepth;
                    cutter.depth = 0.2;
                    state.set_status_message("Move mouse to set cut depth | LMB to cut | Esc to cancel");
                }
            }
        }
        state.window.request_redraw();
        return true;
    }

    false
}

pub fn handle_cursor_moved(state: &mut AppState, position: PhysicalPosition<f64>) -> bool {
    let ray = state.get_screen_ray(position.x, position.y);
    let cutter = match &mut state.cutter_modal {
        Some(c) => c,
        None => return false,
    };

    match cutter.stage {
        CutterStage::DrawFootprint => {
            if let Some(uv) = cutter.workplane.intersect_ray_uv(&ray) {
                cutter.current_uv = uv;
                cutter.current_pos = cutter.workplane.uv_to_world(uv);
                cutter.drag_start_mouse_y = position.y;
                state.window.request_redraw();
                return true;
            }
        }
        CutterStage::ExtrudeDepth => {
            let delta = (position.y - cutter.drag_start_mouse_y) as f32 * 0.005;
            cutter.depth = (0.2 + delta).max(0.01);
            state.window.request_redraw();
            return true;
        }
    }

    false
}

pub fn handle_keyboard_input(state: &mut AppState, key_code: KeyCode) -> bool {
    if state.cutter_modal.is_none() {
        return false;
    }

    match key_code {
        KeyCode::Escape => {
            cancel(state);
            true
        }
        KeyCode::Enter | KeyCode::NumpadEnter => {
            commit(state);
            true
        }
        KeyCode::Tab => {
            if let Some(cutter) = &mut state.cutter_modal {
                cutter.shape = match cutter.shape {
                    CutterShape::Box => CutterShape::Circle,
                    CutterShape::Circle => CutterShape::Box,
                };
                state.window.request_redraw();
            }
            true
        }
        _ => false,
    }
}

pub fn build_box_cutter(wp: &Workplane, start_uv: Vec2, curr_uv: Vec2, depth: f32) -> Result<Mesh, MeshError> {
    let min_u = start_uv.x.min(curr_uv.x);
    let max_u = start_uv.x.max(curr_uv.x);
    let min_v = start_uv.y.min(curr_uv.y);
    let max_v = start_uv.y.max(curr_uv.y);

    let n = wp.normal;
    let d = depth.abs().max(0.01);
    let eps = 0.05;

    let p00 = wp.uv_to_world(Vec2::new(min_u, min_v)) - n * d;
    let p10 = wp.uv_to_world(Vec2::new(max_u, min_v)) - n * d;
    let p11 = wp.uv_to_world(Vec2::new(max_u, max_v)) - n * d;
    let p01 = wp.uv_to_world(Vec2::new(min_u, max_v)) - n * d;

    let t00 = wp.uv_to_world(Vec2::new(min_u, min_v)) + n * eps;
    let t10 = wp.uv_to_world(Vec2::new(max_u, min_v)) + n * eps;
    let t11 = wp.uv_to_world(Vec2::new(max_u, max_v)) + n * eps;
    let t01 = wp.uv_to_world(Vec2::new(min_u, max_v)) + n * eps;

    let mut mesh = Mesh::new();
    let v0 = mesh.add_vertex(p00);
    let v1 = mesh.add_vertex(p10);
    let v2 = mesh.add_vertex(p11);
    let v3 = mesh.add_vertex(p01);

    let v4 = mesh.add_vertex(t00);
    let v5 = mesh.add_vertex(t10);
    let v6 = mesh.add_vertex(t11);
    let v7 = mesh.add_vertex(t01);

    mesh.add_face(&[v0, v3, v2, v1])?;
    mesh.add_face(&[v4, v5, v6, v7])?;
    mesh.add_face(&[v0, v1, v5, v4])?;
    mesh.add_face(&[v1, v2, v6, v5])?;
    mesh.add_face(&[v2, v3, v7, v6])?;
    mesh.add_face(&[v3, v0, v4, v7])?;

    mesh.update_normals();
    Ok(mesh)
}

pub fn build_circle_cutter(wp: &Workplane, center_uv: Vec2, radius: f32, depth: f32, segments: usize) -> Result<Mesh, MeshError> {
    let n = wp.normal;
    let d = depth.abs().max(0.01);
    let eps = 0.05;
    let r = radius.max(0.01);
    let segs = segments.max(16);

    let mut mesh = Mesh::new();
    let mut bot_verts = Vec::with_capacity(segs);
    let mut top_verts = Vec::with_capacity(segs);

    for i in 0..segs {
        let theta = (i as f32 / segs as f32) * std::f32::consts::TAU;
        let uv = center_uv + Vec2::new(theta.cos() * r, theta.sin() * r);
        let pt_bot = wp.uv_to_world(uv) - n * d;
        let pt_top = wp.uv_to_world(uv) + n * eps;
        bot_verts.push(mesh.add_vertex(pt_bot));
        top_verts.push(mesh.add_vertex(pt_top));
    }

    let mut bot_rev = bot_verts.clone();
    bot_rev.reverse();
    mesh.add_face(&bot_rev)?;
    mesh.add_face(&top_verts)?;

    for i in 0..segs {
        let next = (i + 1) % segs;
        mesh.add_face(&[bot_verts[i], bot_verts[next], top_verts[next], top_verts[i]])?;
    }

    mesh.update_normals();
    Ok(mesh)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_box_cutter_mesh_generation() {
        let wp = Workplane::from_point_and_normal(Vec3::ZERO, Vec3::Y);
        let cutter = build_box_cutter(&wp, Vec2::new(-0.5, -0.5), Vec2::new(0.5, 0.5), 0.3).expect("box cutter mesh");
        assert_eq!(cutter.vertices.len(), 8);
        assert_eq!(cutter.faces.len(), 6);
        assert_eq!(cutter.euler_characteristic(), 2);
    }

    #[test]
    fn test_circle_cutter_mesh_generation() {
        let wp = Workplane::from_point_and_normal(Vec3::new(1.0, 2.0, 3.0), Vec3::Z);
        let cutter = build_circle_cutter(&wp, Vec2::ZERO, 0.4, 0.5, 16).expect("circle cutter mesh");
        assert_eq!(cutter.vertices.len(), 32);
        assert_eq!(cutter.faces.len(), 18);
        assert_eq!(cutter.euler_characteristic(), 2);
    }
}

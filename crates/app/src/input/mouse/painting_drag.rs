use crate::state::AppState;
use crate::types::*;
use core_math::Vec3;
use graph_engine::NodeData;
use winit::dpi::PhysicalPosition;

pub fn handle_painting_cursor_moved(state: &mut AppState, position: PhysicalPosition<f64>) -> bool {
    if state.interaction_mode != InteractionMode::Sculpt
        && state.interaction_mode != InteractionMode::WeightPaint
        && state.interaction_mode != InteractionMode::VertexPaint
    {
        return false;
    }

    let ray = state.get_screen_ray(position.x, position.y);
    if let Some(sel_id) = state.selected_node {
        if let Some(node) = state.scene.nodes.get(sel_id) {
            if let NodeData::Mesh { base_mesh, .. } = &node.data {
                if let Some(hit) = base_mesh.raycast(&ray, &node.world_matrix) {
                    state.last_sculpt_hit = Some((hit.point, hit.normal));
                } else {
                    state.last_sculpt_hit = None;
                }
            }
        }
    }

    if state.interaction_mode == InteractionMode::Sculpt && state.is_sculpting {
        let motion_delta = if let Some((last_x, last_y)) = state.last_mouse_pos {
            let dx = (position.x - last_x) as f32;
            let dy = (position.y - last_y) as f32;
            let forward = (state.camera.target - state.camera.eye_position()).normalize();
            let right = forward.cross(Vec3::Y).normalize();
            let up = right.cross(forward).normalize();
            Some((right * dx - up * dy) * 0.01)
        } else {
            None
        };
        state.sculpt_step(position.x, position.y, motion_delta);
    } else if state.interaction_mode == InteractionMode::WeightPaint {
        if state.weight_paint_tool == WeightPaintTool::LinearGradient && state.weight_gradient_start.is_some() {
            let ray = state.get_screen_ray(position.x, position.y);
            let mut hit_found = false;
            if let Some(sel_id) = state.selected_node {
                if let Some(node) = state.scene.nodes.get(sel_id) {
                    if let NodeData::Mesh { base_mesh, .. } = &node.data {
                        if let Some(hit) = base_mesh.raycast(&ray, &node.world_matrix) {
                            state.weight_gradient_end = Some(hit.point);
                            hit_found = true;
                        }
                    }
                }
            }
            if !hit_found {
                if let Some(start) = state.weight_gradient_start {
                    let cam_fwd = (state.camera.target - state.camera.eye_position()).normalize_or_zero();
                    let denom = ray.direction.dot(cam_fwd);
                    if denom.abs() > 1e-4 {
                        let t = (start - ray.origin).dot(cam_fwd) / denom;
                        if t > 0.0 {
                            state.weight_gradient_end = Some(ray.origin + ray.direction * t);
                        }
                    }
                }
            }
        } else if state.is_weight_painting {
            state.weight_paint_step(position.x, position.y);
        }
    } else if state.interaction_mode == InteractionMode::VertexPaint && state.is_vertex_painting {
        state.vertex_paint_step(position.x, position.y);
    }

    state.last_mouse_pos = Some((position.x, position.y));
    state.window.request_redraw();
    true
}

impl AppState {
    pub fn sculpt_step(&mut self, x: f64, y: f64, motion_delta: Option<Vec3>) {
        let ray = self.get_screen_ray(x, y);
        if let Some(sel_id) = self.selected_node {
            if let Some(node) = self.scene.nodes.get_mut(sel_id) {
                let world_mat = node.world_matrix;
                let inv_mat = world_mat.inverse();
                if let NodeData::Mesh { base_mesh, .. } = &mut node.data {
                    if let Some(hit) = base_mesh.raycast(&ray, &world_mat) {
                        self.last_sculpt_hit = Some((hit.point, hit.normal));
                        let local_hit_point = inv_mat.transform_point3(hit.point);
                        let local_hit_normal = inv_mat.transform_vector3(hit.normal).normalize();
                        let local_motion_delta = motion_delta.map(|m| inv_mat.transform_vector3(m));

                        base_mesh.apply_sculpt_brush(
                            local_hit_point,
                            local_hit_normal,
                            &self.sculpt_settings,
                            local_motion_delta,
                        );

                        if self.sculpt_symmetry_x {
                            let sym_hit = Vec3::new(-local_hit_point.x, local_hit_point.y, local_hit_point.z);
                            let sym_norm = Vec3::new(-local_hit_normal.x, local_hit_normal.y, local_hit_normal.z);
                            let sym_motion = local_motion_delta.map(|m| Vec3::new(-m.x, m.y, m.z));
                            base_mesh.apply_sculpt_brush(sym_hit, sym_norm, &self.sculpt_settings, sym_motion);
                        }
                        if self.sculpt_symmetry_y {
                            let sym_hit = Vec3::new(local_hit_point.x, -local_hit_point.y, local_hit_point.z);
                            let sym_norm = Vec3::new(local_hit_normal.x, -local_hit_normal.y, local_hit_normal.z);
                            let sym_motion = local_motion_delta.map(|m| Vec3::new(m.x, -m.y, m.z));
                            base_mesh.apply_sculpt_brush(sym_hit, sym_norm, &self.sculpt_settings, sym_motion);
                        }
                        if self.sculpt_symmetry_z {
                            let sym_hit = Vec3::new(local_hit_point.x, local_hit_point.y, -local_hit_point.z);
                            let sym_norm = Vec3::new(local_hit_normal.x, local_hit_normal.y, -local_hit_normal.z);
                            let sym_motion = local_motion_delta.map(|m| Vec3::new(m.x, m.y, -m.z));
                            base_mesh.apply_sculpt_brush(sym_hit, sym_norm, &self.sculpt_settings, sym_motion);
                        }
                        self.scene.mark_dirty(sel_id);
                    }
                }
            }
        }
    }

    pub fn weight_paint_step(&mut self, x: f64, y: f64) {
        let ray = self.get_screen_ray(x, y);
        if let Some(sel_id) = self.selected_node {
            if let Some(node) = self.scene.nodes.get_mut(sel_id) {
                let world_mat = node.world_matrix;
                let inv_mat = world_mat.inverse();
                if let NodeData::Mesh { base_mesh, .. } = &mut node.data {
                    if let Some(hit) = base_mesh.raycast(&ray, &world_mat) {
                        self.last_sculpt_hit = Some((hit.point, hit.normal));
                        let local_hit_point = inv_mat.transform_point3(hit.point);

                        if self.active_vertex_group.is_none() {
                            if let Some(first_vg) = base_mesh.vertex_groups.first() {
                                self.active_vertex_group = Some(first_vg.name.clone());
                            } else {
                                let idx = base_mesh.add_vertex_group("Group.001");
                                self.active_vertex_group = Some(base_mesh.vertex_groups[idx].name.clone());
                            }
                        }

                        if let Some(ref group_name) = self.active_vertex_group {
                            base_mesh.apply_weight_brush(
                                group_name,
                                local_hit_point,
                                self.weight_brush_target,
                                self.weight_brush_radius,
                                self.weight_brush_strength,
                                self.weight_brush_falloff,
                            );
                            self.scene.mark_dirty(sel_id);
                        }
                    }
                }
            }
        }
    }

    pub fn vertex_paint_step(&mut self, x: f64, y: f64) {
        let ray = self.get_screen_ray(x, y);
        if let Some(sel_id) = self.selected_node {
            if let Some(node) = self.scene.nodes.get_mut(sel_id) {
                let world_mat = node.world_matrix;
                let inv_mat = world_mat.inverse();
                if let NodeData::Mesh { base_mesh, cached_render_mesh, .. } = &mut node.data {
                    if let Some(hit) = base_mesh.raycast(&ray, &world_mat) {
                        self.last_sculpt_hit = Some((hit.point, hit.normal));
                        let local_hit_point = inv_mat.transform_point3(hit.point);

                        base_mesh.apply_color_brush(
                            local_hit_point,
                            self.vertex_paint_color,
                            self.vertex_paint_radius,
                            self.vertex_paint_strength,
                            self.vertex_paint_falloff,
                        );
                        *cached_render_mesh = None;
                        self.scene.mark_dirty(sel_id);
                    }
                }
            }
        }
    }
}

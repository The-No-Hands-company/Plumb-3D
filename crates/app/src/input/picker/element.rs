use crate::state::AppState;
use crate::types::*;
use core_math::Vec3;
use geo_kernel::FaceKey;
use graph_engine::{NodeData, NodeId};

impl AppState {
    pub fn pick_at_screen_pos(&mut self, x: f64, y: f64) {
        let ray = self.get_screen_ray(x, y);

        if self.interaction_mode == InteractionMode::CurveEdit {
            if let Some(sel_id) = self.selected_node {
                if let Some(node) = self.scene.nodes.get(sel_id) {
                    if let NodeData::Curve { spline, .. } = &node.data {
                        let mut best_hit = None;
                        let mut min_dist = 0.35_f32;

                        for (idx, pt) in spline.points.iter().enumerate() {
                            let p_world = node.world_matrix.transform_point3(pt.position);
                            let d = (p_world - ray.origin).cross(ray.direction).length() / ray.direction.length();
                            if d < min_dist {
                                min_dist = d;
                                best_hit = Some((idx, CurveHandleSelection::Knot));
                            }

                            if pt.handle_in.length_squared() > 1e-6 {
                                let p_hin = node.world_matrix.transform_point3(pt.position + pt.handle_in);
                                let d = (p_hin - ray.origin).cross(ray.direction).length() / ray.direction.length();
                                if d < min_dist {
                                    min_dist = d;
                                    best_hit = Some((idx, CurveHandleSelection::HandleIn));
                                }
                            }

                            if pt.handle_out.length_squared() > 1e-6 {
                                let p_hout = node.world_matrix.transform_point3(pt.position + pt.handle_out);
                                let d = (p_hout - ray.origin).cross(ray.direction).length() / ray.direction.length();
                                if d < min_dist {
                                    min_dist = d;
                                    best_hit = Some((idx, CurveHandleSelection::HandleOut));
                                }
                            }
                        }

                        if let Some((k_idx, handle_sel)) = best_hit {
                            self.selected_curve_knot = Some(k_idx);
                            self.selected_curve_handle = Some(handle_sel);
                            return;
                        }
                    }
                }
            }
        }

        if self.interaction_mode == InteractionMode::Edit {
            if let Some(sel_id) = self.selected_node {
                if let Some(node) = self.scene.nodes.get(sel_id) {
                    if let NodeData::Mesh { base_mesh, .. } = &node.data {
                        match self.sub_element_level {
                            SubElementLevel::Vertex => {
                                if let Some(vk) = base_mesh.pick_vertex(&ray, &node.world_matrix, 0.25) {
                                    if self.shift_pressed {
                                        if !self.selected_vertices.remove(&vk) {
                                            self.selected_vertices.insert(vk);
                                        }
                                    } else {
                                        self.selected_vertices.clear();
                                        self.selected_vertices.insert(vk);
                                    }
                                    return;
                                }
                            }
                            SubElementLevel::Edge => {
                                if let Some(ek) = base_mesh.pick_edge(&ray, &node.world_matrix, 0.2) {
                                    if self.alt_pressed {
                                        // Blender Alt+Click: Edge Loop selection
                                        let loop_edges = base_mesh.select_edge_loop(ek);
                                        if !self.shift_pressed {
                                            self.selected_edges.clear();
                                        }
                                        for e in loop_edges {
                                            self.selected_edges.insert(e);
                                        }
                                    } else if self.shift_pressed {
                                        if !self.selected_edges.remove(&ek) {
                                            self.selected_edges.insert(ek);
                                        }
                                    } else {
                                        self.selected_edges.clear();
                                        self.selected_edges.insert(ek);
                                    }
                                    return;
                                }
                            }
                            SubElementLevel::Face => {
                                if let Some(hit) = base_mesh.raycast(&ray, &node.world_matrix) {
                                    let fk = hit.face_key;
                                    if self.alt_pressed {
                                        // Blender Alt+Click: Face Loop / Ring selection
                                        let loop_faces = base_mesh.select_face_loop(fk);
                                        if !self.shift_pressed {
                                            self.selected_faces.clear();
                                        }
                                        for f in loop_faces {
                                            self.selected_faces.insert(f);
                                        }
                                    } else if self.shift_pressed {
                                        if !self.selected_faces.remove(&fk) {
                                            self.selected_faces.insert(fk);
                                        }
                                    } else {
                                        self.selected_faces.clear();
                                        self.selected_faces.insert(fk);
                                    }
                                    return;
                                }
                            }
                        }
                    }
                }
            }
        }

        // Object picking (Meshes, Lights, Cameras, Joints, Curves)
        let mut closest_hit: Option<(NodeId, Option<FaceKey>, f32)> = None;
        for (node_id, node) in &self.scene.nodes {
            if !node.visible {
                continue;
            }
            if self.interaction_mode == InteractionMode::Pose && !matches!(node.data, NodeData::Joint { .. }) {
                continue;
            }
            match &node.data {
                NodeData::Mesh { base_mesh, .. } => {
                    if let Some(hit) = base_mesh.raycast(&ray, &node.world_matrix) {
                        if closest_hit.is_none() || hit.distance < closest_hit.unwrap().2 {
                            closest_hit = Some((node_id, Some(hit.face_key), hit.distance));
                        }
                    }
                }
                NodeData::Light { .. } | NodeData::Camera { .. } => {
                    let pos = node.world_matrix.transform_point3(Vec3::ZERO);
                    let to_pos = pos - ray.origin;
                    let proj = to_pos.dot(ray.direction);
                    if proj > 0.0 {
                        let perp_dist_sq = to_pos.length_squared() - proj * proj;
                        if perp_dist_sq <= 0.45 * 0.45 {
                            if closest_hit.is_none() || proj < closest_hit.unwrap().2 {
                                closest_hit = Some((node_id, None, proj));
                            }
                        }
                    }
                }
                NodeData::Joint { length, radius } => {
                    let p_base = node.world_matrix.transform_point3(Vec3::ZERO);
                    let p_tip = node.world_matrix.transform_point3(Vec3::new(0.0, *length, 0.0));
                    for frac in [0.0, 0.25, 0.5, 0.75, 1.0] {
                        let pt = p_base * (1.0 - frac) + p_tip * frac;
                        let to_pt = pt - ray.origin;
                        let proj = to_pt.dot(ray.direction);
                        if proj > 0.0 {
                            let perp_dist_sq = to_pt.length_squared() - proj * proj;
                            let threshold = (radius * 1.5).max(0.25);
                            if perp_dist_sq <= threshold * threshold {
                                if closest_hit.is_none() || proj < closest_hit.unwrap().2 {
                                    closest_hit = Some((node_id, None, proj));
                                }
                            }
                        }
                    }
                }
                NodeData::Curve { spline, .. } => {
                    for pt in spline.sample(12) {
                        let world_pt = node.world_matrix.transform_point3(pt);
                        let to_pt = world_pt - ray.origin;
                        let proj = to_pt.dot(ray.direction);
                        if proj > 0.0 {
                            let perp_dist_sq = to_pt.length_squared() - proj * proj;
                            if perp_dist_sq <= 0.3 * 0.3 {
                                if closest_hit.is_none() || proj < closest_hit.unwrap().2 {
                                    closest_hit = Some((node_id, None, proj));
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        if let Some((node_id, face_key, _)) = closest_hit {
            if self.shift_pressed {
                if self.selected_nodes.contains(&node_id) {
                    self.selected_nodes.remove(&node_id);
                    if self.selected_node == Some(node_id) {
                        self.selected_node = self.selected_nodes.iter().copied().next();
                    }
                } else {
                    self.selected_nodes.insert(node_id);
                    self.previous_selected_node = self.selected_node;
                    self.selected_node = Some(node_id);
                }
            } else {
                self.selected_nodes.clear();
                self.selected_nodes.insert(node_id);
                self.previous_selected_node = self.selected_node;
                self.selected_node = Some(node_id);
            }
            self.selected_faces.clear();
            if let Some(fk) = face_key {
                self.selected_faces.insert(fk);
            }
        }
    }
}

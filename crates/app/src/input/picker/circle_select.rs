use crate::state::AppState;
use crate::types::*;
use core_math::Vec3;
use graph_engine::NodeData;

impl AppState {
    pub fn select_in_circle(&mut self, center: (f32, f32), radius: f32) {
        let r_sq = radius * radius;
        let width = self.surface_config.width as f32;
        let height = self.surface_config.height as f32;
        let subtract = self.ctrl_pressed;

        if self.interaction_mode == InteractionMode::Edit {
            if let Some(sel_id) = self.selected_node {
                if let Some(node) = self.scene.nodes.get(sel_id) {
                    if let NodeData::Mesh { base_mesh, .. } = &node.data {
                        match self.sub_element_level {
                            SubElementLevel::Vertex => {
                                for (vk, v) in &base_mesh.vertices {
                                    let p_world = node.world_matrix.transform_point3(v.position);
                                    if !self.show_wireframe {
                                        let view_dir = p_world - self.camera.eye_position();
                                        let has_front =
                                            base_mesh.vertex_faces(vk).iter().any(|&fk| {
                                                base_mesh.faces.get(fk).is_some_and(|f| {
                                                    node.world_matrix
                                                        .transform_vector3(f.normal)
                                                        .dot(view_dir)
                                                        < 0.0
                                                })
                                            });
                                        if !has_front {
                                            continue;
                                        }
                                    }
                                    if let Some(sp) =
                                        self.camera.world_to_screen(p_world, width, height)
                                    {
                                        let d_sq =
                                            (sp.x - center.0).powi(2) + (sp.y - center.1).powi(2);
                                        if d_sq <= r_sq {
                                            if subtract {
                                                self.selected_vertices.remove(&vk);
                                            } else {
                                                self.selected_vertices.insert(vk);
                                            }
                                        }
                                    }
                                }
                            }
                            SubElementLevel::Edge => {
                                for (ek, edge) in &base_mesh.edges {
                                    let he = &base_mesh.half_edges[edge.half_edge];
                                    let p0 = base_mesh.vertices[he.origin].position;
                                    let p1 = base_mesh.vertices[he.target].position;
                                    let mid = (p0 + p1) * 0.5;
                                    let p_world = node.world_matrix.transform_point3(mid);
                                    if !self.show_wireframe {
                                        let view_dir = p_world - self.camera.eye_position();
                                        let f_left = he.face;
                                        let f_right = base_mesh
                                            .half_edges
                                            .get(he.twin)
                                            .and_then(|th| th.face);
                                        let left_front = f_left
                                            .and_then(|fk| base_mesh.faces.get(fk))
                                            .is_some_and(|f| {
                                                node.world_matrix
                                                    .transform_vector3(f.normal)
                                                    .dot(view_dir)
                                                    < 0.0
                                            });
                                        let right_front = f_right
                                            .and_then(|fk| base_mesh.faces.get(fk))
                                            .is_some_and(|f| {
                                                node.world_matrix
                                                    .transform_vector3(f.normal)
                                                    .dot(view_dir)
                                                    < 0.0
                                            });
                                        if !left_front && !right_front {
                                            continue;
                                        }
                                    }
                                    if let Some(sp) =
                                        self.camera.world_to_screen(p_world, width, height)
                                    {
                                        let d_sq =
                                            (sp.x - center.0).powi(2) + (sp.y - center.1).powi(2);
                                        if d_sq <= r_sq {
                                            if subtract {
                                                self.selected_edges.remove(&ek);
                                            } else {
                                                self.selected_edges.insert(ek);
                                            }
                                        }
                                    }
                                }
                            }
                            SubElementLevel::Face => {
                                for (fk, face) in &base_mesh.faces {
                                    let vks = base_mesh.face_vertices(fk);
                                    if !vks.is_empty() {
                                        let mut sum = Vec3::ZERO;
                                        for vk in &vks {
                                            sum += base_mesh.vertices[*vk].position;
                                        }
                                        let centroid = sum / vks.len() as f32;
                                        let p_world = node.world_matrix.transform_point3(centroid);
                                        if !self.show_wireframe {
                                            let world_normal =
                                                node.world_matrix.transform_vector3(face.normal);
                                            let view_dir = p_world - self.camera.eye_position();
                                            if world_normal.dot(view_dir) >= 0.0 {
                                                continue;
                                            }
                                        }
                                        if let Some(sp) =
                                            self.camera.world_to_screen(p_world, width, height)
                                        {
                                            let d_sq = (sp.x - center.0).powi(2)
                                                + (sp.y - center.1).powi(2);
                                            if d_sq <= r_sq {
                                                if subtract {
                                                    self.selected_faces.remove(&fk);
                                                } else {
                                                    self.selected_faces.insert(fk);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        } else {
            // Object mode circle / paint selection
            for (node_id, node) in &self.scene.nodes {
                if !node.visible {
                    continue;
                }
                let p_world = node.world_matrix.transform_point3(Vec3::ZERO);
                if let Some(sp) = self.camera.world_to_screen(p_world, width, height) {
                    let d_sq = (sp.x - center.0).powi(2) + (sp.y - center.1).powi(2);
                    if d_sq <= r_sq {
                        if subtract {
                            self.selected_nodes.remove(&node_id);
                            if self.selected_node == Some(node_id) {
                                self.selected_node = self.selected_nodes.iter().copied().next();
                            }
                        } else {
                            self.selected_nodes.insert(node_id);
                            self.previous_selected_node = self.selected_node;
                            self.selected_node = Some(node_id);
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_circle_distance_threshold() {
        let center = (100.0, 100.0);
        let radius = 25.0_f32;
        let r_sq = radius * radius;

        let inside = (110.0_f32, 110.0_f32);
        let d_sq_inside = (inside.0 - center.0).powi(2) + (inside.1 - center.1).powi(2);
        assert!(d_sq_inside <= r_sq);

        let outside = (130.0_f32, 100.0_f32);
        let d_sq_outside = (outside.0 - center.0).powi(2) + (outside.1 - center.1).powi(2);
        assert!(d_sq_outside > r_sq);
    }
}

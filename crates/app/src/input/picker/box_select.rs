use crate::state::AppState;
use crate::types::*;
use core_math::Vec3;
use graph_engine::NodeData;

impl AppState {
    pub fn select_in_marquee_box(&mut self, start: (f32, f32), current: (f32, f32)) {
        let min_x = start.0.min(current.0);
        let max_x = start.0.max(current.0);
        let min_y = start.1.min(current.1);
        let max_y = start.1.max(current.1);

        let width = self.surface_config.width as f32;
        let height = self.surface_config.height as f32;

        let subtract = self.ctrl_pressed;
        let add = self.shift_pressed;

        if !add && !subtract && self.interaction_mode == InteractionMode::Edit {
            self.selected_vertices.clear();
            self.selected_edges.clear();
            self.selected_faces.clear();
        }

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
                                        if sp.x >= min_x
                                            && sp.x <= max_x
                                            && sp.y >= min_y
                                            && sp.y <= max_y
                                        {
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
                                        if sp.x >= min_x
                                            && sp.x <= max_x
                                            && sp.y >= min_y
                                            && sp.y <= max_y
                                        {
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
                                                continue; // Backface culled from box select
                                            }
                                        }

                                        if let Some(sp) =
                                            self.camera.world_to_screen(p_world, width, height)
                                        {
                                            if sp.x >= min_x
                                                && sp.x <= max_x
                                                && sp.y >= min_y
                                                && sp.y <= max_y
                                            {
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
            // Object mode marquee selection
            if !add && !subtract {
                self.selected_nodes.clear();
                self.selected_node = None;
            }

            for (node_id, node) in &self.scene.nodes {
                if !node.visible {
                    continue;
                }
                let p_world = node.world_matrix.transform_point3(Vec3::ZERO);
                if let Some(sp) = self.camera.world_to_screen(p_world, width, height) {
                    if sp.x >= min_x && sp.x <= max_x && sp.y >= min_y && sp.y <= max_y {
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

use crate::state::AppState;
use crate::types::*;
use core_math::Vec3;
use graph_engine::NodeData;

/// Returns true if 2D point (px, py) is strictly inside polygon using the ray casting crossing test.
pub fn point_in_polygon(px: f32, py: f32, poly: &[(f32, f32)]) -> bool {
    if poly.len() < 3 {
        return false;
    }
    let mut inside = false;
    let mut j = poly.len() - 1;
    for i in 0..poly.len() {
        let (xi, yi) = poly[i];
        let (xj, yj) = poly[j];
        let intersect = ((yi > py) != (yj > py))
            && (px < (xj - xi) * (py - yi) / (yj - yi) + xi);
        if intersect {
            inside = !inside;
        }
        j = i;
    }
    inside
}

impl AppState {
    pub fn select_in_lasso(&mut self, polygon: &[(f32, f32)]) {
        if polygon.len() < 3 {
            return;
        }
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
                                        let has_front = base_mesh.vertex_faces(vk).iter().any(|&fk| {
                                            base_mesh.faces.get(fk).is_some_and(|f| {
                                                node.world_matrix.transform_vector3(f.normal).dot(view_dir) < 0.0
                                            })
                                        });
                                        if !has_front {
                                            continue;
                                        }
                                    }
                                    if let Some(sp) = self.camera.world_to_screen(p_world, width, height) {
                                        if point_in_polygon(sp.x, sp.y, polygon) {
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
                                        let f_right = base_mesh.half_edges.get(he.twin).and_then(|th| th.face);
                                        let left_front = f_left.and_then(|fk| base_mesh.faces.get(fk)).is_some_and(|f| {
                                            node.world_matrix.transform_vector3(f.normal).dot(view_dir) < 0.0
                                        });
                                        let right_front = f_right.and_then(|fk| base_mesh.faces.get(fk)).is_some_and(|f| {
                                            node.world_matrix.transform_vector3(f.normal).dot(view_dir) < 0.0
                                        });
                                        if !left_front && !right_front {
                                            continue;
                                        }
                                    }
                                    if let Some(sp) = self.camera.world_to_screen(p_world, width, height) {
                                        if point_in_polygon(sp.x, sp.y, polygon) {
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
                                            let world_normal = node.world_matrix.transform_vector3(face.normal);
                                            let view_dir = p_world - self.camera.eye_position();
                                            if world_normal.dot(view_dir) >= 0.0 {
                                                continue;
                                            }
                                        }
                                        if let Some(sp) = self.camera.world_to_screen(p_world, width, height) {
                                            if point_in_polygon(sp.x, sp.y, polygon) {
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
            // Object mode lasso selection
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
                    if point_in_polygon(sp.x, sp.y, polygon) {
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
    use super::*;

    #[test]
    fn test_point_in_polygon_square() {
        let poly = vec![(0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 10.0)];
        assert!(point_in_polygon(5.0, 5.0, &poly));
        assert!(!point_in_polygon(15.0, 5.0, &poly));
        assert!(!point_in_polygon(-1.0, 5.0, &poly));
    }

    #[test]
    fn test_point_in_polygon_triangle() {
        let poly = vec![(0.0, 0.0), (10.0, 0.0), (5.0, 10.0)];
        assert!(point_in_polygon(5.0, 3.0, &poly));
        assert!(!point_in_polygon(0.0, 10.0, &poly));
    }
}

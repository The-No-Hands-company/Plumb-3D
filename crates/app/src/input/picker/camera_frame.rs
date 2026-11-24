use crate::state::AppState;
use crate::types::*;
use core_math::Vec3;
use graph_engine::NodeData;

impl AppState {
    /// Smoothly frames the camera on the currently selected node or sub-elements (Maya / Blender Focus: F).
    pub fn frame_selected(&mut self) {
        let sel_id = match self.selected_node {
            Some(id) => id,
            None => {
                self.camera.focus_on(Vec3::ZERO, 2.5);
                self.window.request_redraw();
                return;
            }
        };

        let node = match self.scene.nodes.get(sel_id) {
            Some(n) => n,
            None => return,
        };

        match self.interaction_mode {
            InteractionMode::Edit => {
                if let NodeData::Mesh { base_mesh, .. } = &node.data {
                    let mut sum = Vec3::ZERO;
                    let mut count = 0;
                    match self.sub_element_level {
                        SubElementLevel::Vertex => {
                            for &vk in &self.selected_vertices {
                                if let Some(v) = base_mesh.vertices.get(vk) {
                                    sum += v.position;
                                    count += 1;
                                }
                            }
                        }
                        SubElementLevel::Edge => {
                            for &ek in &self.selected_edges {
                                if let Some(e) = base_mesh.edges.get(ek) {
                                    let he = &base_mesh.half_edges[e.half_edge];
                                    if let (Some(v0), Some(v1)) = (base_mesh.vertices.get(he.origin), base_mesh.vertices.get(he.target)) {
                                        sum += (v0.position + v1.position) * 0.5;
                                        count += 1;
                                    }
                                }
                            }
                        }
                        SubElementLevel::Face => {
                            for &fk in &self.selected_faces {
                                let f_verts = base_mesh.face_vertices(fk);
                                if !f_verts.is_empty() {
                                    let mut f_sum = Vec3::ZERO;
                                    for vk in &f_verts {
                                        if let Some(v) = base_mesh.vertices.get(*vk) {
                                            f_sum += v.position;
                                        }
                                    }
                                    sum += f_sum / f_verts.len() as f32;
                                    count += 1;
                                }
                            }
                        }
                    }

                    if count > 0 {
                        let local_center = sum / count as f32;
                        let world_center = node.world_matrix.transform_point3(local_center);
                        let mut max_dist = 0.5_f32;
                        for &vk in &self.selected_vertices {
                            if let Some(v) = base_mesh.vertices.get(vk) {
                                max_dist = max_dist.max((v.position - local_center).length());
                            }
                        }
                        let scale = node.local_transform.scale.max_element().max(0.1);
                        self.camera.focus_on(world_center, (max_dist * scale).max(0.5));
                        self.window.request_redraw();
                        return;
                    }
                }
            }
            InteractionMode::CurveEdit => {
                if let NodeData::Curve { spline, .. } = &node.data {
                    if let Some(k_idx) = self.selected_curve_knot {
                        if let Some(pt) = spline.points.get(k_idx) {
                            let world_pos = node.world_matrix.transform_point3(pt.position);
                            self.camera.focus_on(world_pos, 1.0);
                            self.window.request_redraw();
                            return;
                        }
                    }
                }
            }
            _ => {}
        }

        let world_center = node.world_matrix.transform_point3(Vec3::ZERO);
        let radius = match &node.data {
            NodeData::Mesh { base_mesh, .. } => {
                let aabb = base_mesh.compute_aabb();
                let center_world = node.world_matrix.transform_point3(aabb.center());
                let scale = node.local_transform.scale.max_element().max(0.1);
                let extents = aabb.extents().length() * 0.5 * scale;
                self.camera.focus_on(center_world, extents.max(0.5));
                self.window.request_redraw();
                return;
            }
            NodeData::Curve { spline, .. } => {
                let mut max_d = 1.0_f32;
                for pt in &spline.points {
                    max_d = max_d.max(pt.position.length());
                }
                max_d * node.local_transform.scale.max_element()
            }
            NodeData::Joint { length, .. } => *length * 1.2,
            _ => 1.5,
        };

        self.camera.focus_on(world_center, radius.max(0.5));
        self.window.request_redraw();
    }
}

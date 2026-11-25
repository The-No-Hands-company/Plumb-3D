use crate::state::AppState;
use crate::types::*;
use core_math::{vec2, EulerRot, Quat, Vec3};
use graph_engine::NodeData;

impl AppState {
    pub fn get_screen_ray(&self, x: f64, y: f64) -> core_math::Ray {
        let screen_uv = vec2(
            x as f32 / self.surface_config.width as f32,
            y as f32 / self.surface_config.height as f32,
        );
        self.camera.screen_to_ray(screen_uv)
    }

    pub fn get_gizmo_world_position(&self) -> Option<Vec3> {
        let sel_id = self.selected_node?;
        let node = self.scene.nodes.get(sel_id)?;

        if self.interaction_mode == InteractionMode::Edit {
            if let NodeData::Mesh { base_mesh, .. } = &node.data {
                match self.sub_element_level {
                    SubElementLevel::Vertex => {
                        if !self.selected_vertices.is_empty() {
                            let mut sum = Vec3::ZERO;
                            let mut count = 0;
                            for &vk in &self.selected_vertices {
                                if let Some(v) = base_mesh.vertices.get(vk) {
                                    sum += v.position;
                                    count += 1;
                                }
                            }
                            if count > 0 {
                                return Some(node.world_matrix.transform_point3(sum / count as f32));
                            }
                        }
                    }
                    SubElementLevel::Edge => {
                        if !self.selected_edges.is_empty() {
                            let mut sum = Vec3::ZERO;
                            let mut count = 0;
                            for &ek in &self.selected_edges {
                                if let Some(edge) = base_mesh.edges.get(ek) {
                                    let he = &base_mesh.half_edges[edge.half_edge];
                                    if let (Some(v0), Some(v1)) = (
                                        base_mesh.vertices.get(he.origin),
                                        base_mesh.vertices.get(he.target),
                                    ) {
                                        sum += (v0.position + v1.position) * 0.5;
                                        count += 1;
                                    }
                                }
                            }
                            if count > 0 {
                                return Some(node.world_matrix.transform_point3(sum / count as f32));
                            }
                        }
                    }
                    SubElementLevel::Face => {
                        if !self.selected_faces.is_empty() {
                            let mut sum = Vec3::ZERO;
                            let mut count = 0;
                            for &fk in &self.selected_faces {
                                let vks = base_mesh.face_vertices(fk);
                                if !vks.is_empty() {
                                    let mut f_sum = Vec3::ZERO;
                                    for vk in &vks {
                                        if let Some(v) = base_mesh.vertices.get(*vk) {
                                            f_sum += v.position;
                                        }
                                    }
                                    sum += f_sum / vks.len() as f32;
                                    count += 1;
                                }
                            }
                            if count > 0 {
                                return Some(node.world_matrix.transform_point3(sum / count as f32));
                            }
                        }
                    }
                }
            }
        }

        if self.interaction_mode == InteractionMode::CurveEdit {
            if let NodeData::Curve { spline, .. } = &node.data {
                if let Some(k_idx) = self.selected_curve_knot {
                    if let Some(pt) = spline.points.get(k_idx) {
                        let local_pos = match self.selected_curve_handle {
                            None | Some(CurveHandleSelection::Knot) => pt.position,
                            Some(CurveHandleSelection::HandleIn) => pt.position + pt.handle_in,
                            Some(CurveHandleSelection::HandleOut) => pt.position + pt.handle_out,
                        };
                        return Some(node.world_matrix.transform_point3(local_pos));
                    }
                }
            }
        }

        if (self.interaction_mode == InteractionMode::Object || self.interaction_mode == InteractionMode::Pose)
            && self.selected_nodes.len() > 1 {
                let mut sum = Vec3::ZERO;
                let mut count = 0;
                for &nid in &self.selected_nodes {
                    if let Some(n) = self.scene.nodes.get(nid) {
                        sum += n.world_matrix.transform_point3(Vec3::ZERO);
                        count += 1;
                    }
                }
                if count > 0 {
                    return Some(sum / count as f32);
                }
            }

        Some(node.world_matrix.transform_point3(Vec3::ZERO))
    }

    pub fn get_gizmo_orientation(&self) -> Quat {
        match self.transform_orientation {
            TransformOrientation::Global => Quat::IDENTITY,
            TransformOrientation::Local => {
                if let Some(sel_id) = self.selected_node {
                    if let Some(node) = self.scene.nodes.get(sel_id) {
                        return node.local_transform.rotation;
                    }
                }
                Quat::IDENTITY
            }
            TransformOrientation::View => {
                Quat::from_euler(EulerRot::YXZ, self.camera.yaw, self.camera.pitch, 0.0)
            }
            TransformOrientation::Normal => {
                if self.interaction_mode == InteractionMode::Edit {
                    if let Some(sel_id) = self.selected_node {
                        if let Some(node) = self.scene.nodes.get(sel_id) {
                            if let NodeData::Mesh { base_mesh, .. } = &node.data {
                                let mut normal_sum = Vec3::ZERO;
                                let mut count = 0.0f32;

                                if !self.selected_faces.is_empty() {
                                    for &fk in &self.selected_faces {
                                        let verts = base_mesh.face_vertices(fk);
                                        let n = base_mesh.compute_face_normal_newell(&verts);
                                        if n.length_squared() > 1e-5 {
                                            normal_sum += n;
                                            count += 1.0;
                                        }
                                    }
                                } else if !self.selected_vertices.is_empty() {
                                    for &vk in &self.selected_vertices {
                                        if let Some(v) = base_mesh.vertices.get(vk) {
                                            if v.normal.length_squared() > 1e-5 {
                                                normal_sum += v.normal;
                                                count += 1.0;
                                            }
                                        }
                                    }
                                } else if !self.selected_edges.is_empty() {
                                    for &ek in &self.selected_edges {
                                        if let Some(edge) = base_mesh.edges.get(ek) {
                                            let he = &base_mesh.half_edges[edge.half_edge];
                                            if let (Some(v0), Some(v1)) = (
                                                base_mesh.vertices.get(he.origin),
                                                base_mesh.vertices.get(he.target),
                                            ) {
                                                let n = (v0.normal + v1.normal).normalize_or_zero();
                                                if n.length_squared() > 1e-5 {
                                                    normal_sum += n;
                                                    count += 1.0;
                                                }
                                            }
                                        }
                                    }
                                }

                                if count > 0.0 && normal_sum.length_squared() > 1e-5 {
                                    let local_norm = normal_sum.normalize();
                                    let world_norm = (node.local_transform.rotation * local_norm).normalize();
                                    let dot = Vec3::Z.dot(world_norm);
                                    let rot = if dot < -0.9999 {
                                        Quat::from_axis_angle(Vec3::Y, std::f32::consts::PI)
                                    } else if dot > 0.9999 {
                                        Quat::IDENTITY
                                    } else {
                                        Quat::from_rotation_arc(Vec3::Z, world_norm)
                                    };
                                    if rot.is_finite() {
                                        return rot;
                                    }
                                }
                            }
                        }
                    }
                }

                // Fallback to local orientation if not edit mode or no sub-element selected
                if let Some(sel_id) = self.selected_node {
                    if let Some(node) = self.scene.nodes.get(sel_id) {
                        return node.local_transform.rotation;
                    }
                }
                Quat::IDENTITY
            }
        }
    }
}

use crate::state::AppState;
use core_math::Vec3;
use geo_kernel::VertexKey;
use graph_engine::NodeData;
use std::collections::HashSet;

impl AppState {
    pub fn find_closest_snap_vertex(&self, world_pos: Vec3, max_dist: f32, exclude_verts: &HashSet<VertexKey>) -> Option<Vec3> {
        let mut best: Option<(Vec3, f32)> = None;
        for (node_id, node) in &self.scene.nodes {
            if let NodeData::Mesh { base_mesh, .. } = &node.data {
                let wmat = node.world_matrix;
                let is_sel = Some(node_id) == self.selected_node;
                for (vk, v) in &base_mesh.vertices {
                    if is_sel && exclude_verts.contains(&vk) {
                        continue;
                    }
                    let w_v = wmat.transform_point3(v.position);
                    let d = (w_v - world_pos).length();
                    if d < max_dist {
                        if best.is_none() || d < best.as_ref().unwrap().1 {
                            best = Some((w_v, d));
                        }
                    }
                }
            }
        }
        best.map(|(p, _)| p)
    }

    pub fn find_closest_snap_edge_midpoint(&self, world_pos: Vec3, max_dist: f32) -> Option<Vec3> {
        let mut best: Option<(Vec3, f32)> = None;
        for (_, node) in &self.scene.nodes {
            if let NodeData::Mesh { base_mesh, .. } = &node.data {
                let wmat = node.world_matrix;
                for edge in base_mesh.edges.values() {
                    if let Some(he) = base_mesh.half_edges.get(edge.half_edge) {
                        if let (Some(v1), Some(v2)) = (
                            base_mesh.vertices.get(he.origin),
                            base_mesh.vertices.get(he.target),
                        ) {
                            let mid = 0.5 * (v1.position + v2.position);
                            let w_mid = wmat.transform_point3(mid);
                            let d = (w_mid - world_pos).length();
                            if d < max_dist {
                                if best.is_none() || d < best.as_ref().unwrap().1 {
                                    best = Some((w_mid, d));
                                }
                            }
                        }
                    }
                }
            }
        }
        best.map(|(p, _)| p)
    }

    pub fn find_closest_snap_face_center(&self, world_pos: Vec3, max_dist: f32) -> Option<Vec3> {
        let mut best: Option<(Vec3, f32)> = None;
        for (_, node) in &self.scene.nodes {
            if let NodeData::Mesh { base_mesh, .. } = &node.data {
                let wmat = node.world_matrix;
                for face_key in base_mesh.faces.keys() {
                    let center = base_mesh.face_centroid(face_key);
                    let w_center = wmat.transform_point3(center);
                    let d = (w_center - world_pos).length();
                    if d < max_dist {
                        if best.is_none() || d < best.as_ref().unwrap().1 {
                            best = Some((w_center, d));
                        }
                    }
                }
            }
        }
        best.map(|(p, _)| p)
    }
}

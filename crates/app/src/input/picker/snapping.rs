use crate::state::AppState;
use core_math::Vec3;
use geo_kernel::VertexKey;
use graph_engine::NodeData;
use std::collections::HashSet;

impl AppState {
    pub fn find_closest_snap_vertex(
        &self,
        world_pos: Vec3,
        max_dist: f32,
        exclude_verts: &HashSet<VertexKey>,
    ) -> Option<Vec3> {
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
                    if d < max_dist && (best.is_none() || d < best.as_ref().unwrap().1) {
                        best = Some((w_v, d));
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
                            if d < max_dist && (best.is_none() || d < best.as_ref().unwrap().1) {
                                best = Some((w_mid, d));
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
                    if d < max_dist && (best.is_none() || d < best.as_ref().unwrap().1) {
                        best = Some((w_center, d));
                    }
                }
            }
        }
        best.map(|(p, _)| p)
    }

    /// Snaps orthogonally (perpendicularly) to the nearest edge across all scene meshes.
    pub fn find_closest_snap_perpendicular(&self, world_pos: Vec3, max_dist: f32) -> Option<Vec3> {
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
                            let p0 = wmat.transform_point3(v1.position);
                            let p1 = wmat.transform_point3(v2.position);
                            let v = p1 - p0;
                            let v_len_sq = v.length_squared();
                            if v_len_sq > 1e-6 {
                                let t = ((world_pos - p0).dot(v) / v_len_sq).clamp(0.0, 1.0);
                                let proj = p0 + v * t;
                                let d = (proj - world_pos).length();
                                if d < max_dist && (best.is_none() || d < best.as_ref().unwrap().1)
                                {
                                    best = Some((proj, d));
                                }
                            }
                        }
                    }
                }
            }
        }
        best.map(|(p, _)| p)
    }

    /// Snaps to the center of another scene object for rapid assembly alignment.
    pub fn find_closest_snap_object_center(&self, world_pos: Vec3, max_dist: f32) -> Option<Vec3> {
        let mut best: Option<(Vec3, f32)> = None;
        for (node_id, node) in &self.scene.nodes {
            if Some(node_id) == self.selected_node {
                continue;
            }
            let center = match &node.data {
                NodeData::Mesh { base_mesh, .. } => node
                    .world_matrix
                    .transform_point3(base_mesh.bounding_center()),
                _ => node.world_matrix.transform_point3(Vec3::ZERO),
            };
            let d = (center - world_pos).length();
            if d < max_dist && (best.is_none() || d < best.as_ref().unwrap().1) {
                best = Some((center, d));
            }
        }
        best.map(|(p, _)| p)
    }

    /// Snaps tangent-perpendicularly onto splines and circular curved edges.
    pub fn find_closest_snap_tangent(&self, world_pos: Vec3, max_dist: f32) -> Option<Vec3> {
        let mut best: Option<(Vec3, f32)> = None;
        for (_, node) in &self.scene.nodes {
            if let NodeData::Curve { spline, .. } = &node.data {
                let wmat = node.world_matrix;
                let local_pos = wmat.inverse().transform_point3(world_pos);
                let (u, local_pt, _) = spline.closest_point(local_pos, 16);
                let (_, local_tan) = spline.eval(u);
                let world_pt = wmat.transform_point3(local_pt);
                let world_tan = wmat.transform_vector3(local_tan).normalize_or_zero();
                let to_query = (world_pos - world_pt).normalize_or_zero();
                let dot = to_query.dot(world_tan).abs();
                let d = (world_pt - world_pos).length();
                if d < max_dist && dot < 0.35 && (best.is_none() || d < best.as_ref().unwrap().1) {
                    best = Some((world_pt, d));
                }
            }
        }
        best.map(|(p, _)| p)
    }
}

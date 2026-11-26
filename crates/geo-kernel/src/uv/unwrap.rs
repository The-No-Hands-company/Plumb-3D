use crate::mesh::{EdgeKey, FaceKey, HalfEdgeKey, Mesh, VertexKey};
use core_math::{vec2, Vec2, Vec3};
use std::collections::{HashMap, HashSet, VecDeque};

impl Mesh {
    /// Automatically marks edges as UV seams wherever the angle between adjacent faces exceeds `angle_threshold_deg`.
    pub fn mark_seams_by_angle(&mut self, angle_threshold_deg: f32) {
        let cos_threshold = (angle_threshold_deg.to_radians()).cos();
        let edge_keys: Vec<EdgeKey> = self.edges.keys().collect();

        for ek in edge_keys {
            let edge = &self.edges[ek];
            let h1 = edge.half_edge;
            let h2 = self.half_edges[h1].twin;

            let f1 = self.half_edges[h1].face;
            let f2 = self.half_edges[h2].face;

            if let (Some(f1), Some(f2)) = (f1, f2) {
                let n1 = self.faces[f1].normal;
                let n2 = self.faces[f2].normal;
                let dot = n1.dot(n2);
                if dot < cos_threshold {
                    self.edges[ek].is_seam = true;
                }
            } else {
                // Open boundary edge
                self.edges[ek].is_seam = true;
            }
        }
    }

    /// Partitions the mesh's faces into connected UV charts / islands separated by marked seams and boundary loops.
    pub fn partition_uv_charts(&self) -> Vec<Vec<FaceKey>> {
        let mut visited = HashSet::new();
        let mut charts = Vec::new();

        for fk in self.faces.keys() {
            if visited.contains(&fk) {
                continue;
            }

            let mut chart = Vec::new();
            let mut queue = VecDeque::new();
            queue.push_back(fk);
            visited.insert(fk);

            while let Some(curr_fk) = queue.pop_front() {
                chart.push(curr_fk);
                let hes = self.face_half_edges(curr_fk);
                for he_key in hes {
                    let he = &self.half_edges[he_key];
                    let edge = &self.edges[he.edge];
                    if !edge.is_seam {
                        let twin_he = &self.half_edges[he.twin];
                        if let Some(adj_fk) = twin_he.face {
                            if !visited.contains(&adj_fk) {
                                visited.insert(adj_fk);
                                queue.push_back(adj_fk);
                            }
                        }
                    }
                }
            }
            charts.push(chart);
        }
        charts
    }

    /// Unwraps a single chart using Conformal / Harmonic Tutte Disk Parameterization or best-fit tangent plane projection.
    pub fn unwrap_chart(&self, chart: &[FaceKey]) -> HashMap<VertexKey, Vec2> {
        let mut uv_map = HashMap::new();
        if chart.is_empty() {
            return uv_map;
        }

        // Collect all unique vertices in the chart
        let mut chart_faces_set = HashSet::new();
        let mut chart_verts = HashSet::new();
        for &fk in chart {
            chart_faces_set.insert(fk);
            for vk in self.face_vertices(fk) {
                chart_verts.insert(vk);
            }
        }

        // Compute average normal and 3D centroid
        let mut avg_normal = Vec3::ZERO;
        let mut center = Vec3::ZERO;
        let mut total_3d_area = 0.0_f32;

        for &fk in chart {
            avg_normal += self.faces[fk].normal;
            let fverts = self.face_vertices(fk);
            if fverts.len() >= 3 {
                let p0 = self.vertices[fverts[0]].position;
                let p1 = self.vertices[fverts[1]].position;
                let p2 = self.vertices[fverts[2]].position;
                total_3d_area += (p1 - p0).cross(p2 - p0).length() * 0.5;
            }
            for vk in fverts {
                center += self.vertices[vk].position;
            }
        }
        if !chart_verts.is_empty() {
            center /= chart_verts.len() as f32;
        }

        let n = avg_normal.normalize_or_zero();
        let up = if n.y.abs() < 0.9 { Vec3::Y } else { Vec3::Z };
        let t_axis = n.cross(up).normalize_or_zero();
        let b_axis = n.cross(t_axis).normalize_or_zero();

        // Identify chart boundary half-edges (half-edges whose twin is outside the chart or is a seam)
        let mut boundary_hes = Vec::new();
        for &fk in chart {
            for he_key in self.face_half_edges(fk) {
                let he = &self.half_edges[he_key];
                let is_chart_seam = self.edges[he.edge].is_seam;
                let twin_face = self.half_edges[he.twin].face;
                let is_outside = twin_face.is_none_or(|f| !chart_faces_set.contains(&f));
                if is_chart_seam || is_outside {
                    boundary_hes.push(he_key);
                }
            }
        }

        // Try to chain boundary half-edges into an ordered boundary cycle
        let mut boundary_cycle = Vec::new();
        if !boundary_hes.is_empty() {
            let mut remaining_hes: HashSet<HalfEdgeKey> = boundary_hes.into_iter().collect();
            if let Some(&first_he) = remaining_hes.iter().next() {
                remaining_hes.remove(&first_he);
                boundary_cycle.push(self.half_edges[first_he].origin);
                let mut current_target = self.half_edges[first_he].target;

                while !remaining_hes.is_empty() {
                    boundary_cycle.push(current_target);
                    if let Some(&next_he) = remaining_hes.iter().find(|&&h| self.half_edges[h].origin == current_target) {
                        remaining_hes.remove(&next_he);
                        current_target = self.half_edges[next_he].target;
                    } else {
                        break;
                    }
                }
            }
        }

        // If a valid boundary loop with >= 3 vertices was found, use Tutte harmonic embedding
        let boundary_set: HashSet<VertexKey> = boundary_cycle.iter().copied().collect();
        let use_tutte = boundary_cycle.len() >= 3 && boundary_set.len() >= 3 && boundary_set.len() < chart_verts.len();

        if use_tutte {
            // Measure cumulative perimeter lengths
            let b_len = boundary_cycle.len();
            let mut lengths = Vec::with_capacity(b_len);
            let mut total_perimeter = 0.0_f32;

            for i in 0..b_len {
                let p0 = self.vertices[boundary_cycle[i]].position;
                let p1 = self.vertices[boundary_cycle[(i + 1) % b_len]].position;
                let seg_len = (p1 - p0).length().max(1e-5);
                lengths.push(seg_len);
                total_perimeter += seg_len;
            }

            // Assign boundary vertices around a 2D circle with proportional arc lengths
            let mut current_len = 0.0_f32;
            for i in 0..b_len {
                let theta = (current_len / total_perimeter) * std::f32::consts::TAU;
                let u = 0.5 + 0.5 * theta.cos();
                let v = 0.5 + 0.5 * theta.sin();
                uv_map.insert(boundary_cycle[i], vec2(u, v));
                current_len += lengths[i];
            }

            // Initial guess for interior vertices (projected onto tangent plane, scaled to [0.1, 0.9])
            for &vk in &chart_verts {
                if !boundary_set.contains(&vk) {
                    let p = self.vertices[vk].position - center;
                    let u = 0.5 + (p.dot(t_axis) * 0.1).clamp(-0.4, 0.4);
                    let v = 0.5 + (p.dot(b_axis) * 0.1).clamp(-0.4, 0.4);
                    uv_map.insert(vk, vec2(u, v));
                }
            }

            // Build chart 1-ring adjacency graph for interior vertices
            let mut adj_graph: HashMap<VertexKey, Vec<VertexKey>> = HashMap::new();
            for &fk in chart {
                let fverts = self.face_vertices(fk);
                for i in 0..fverts.len() {
                    let v0 = fverts[i];
                    let v1 = fverts[(i + 1) % fverts.len()];
                    adj_graph.entry(v0).or_default().push(v1);
                    adj_graph.entry(v1).or_default().push(v0);
                }
            }

            // Run 35 iterations of harmonic relaxation for interior vertices
            for _ in 0..35 {
                for &vk in &chart_verts {
                    if !boundary_set.contains(&vk) {
                        if let Some(neighbors) = adj_graph.get(&vk) {
                            if !neighbors.is_empty() {
                                let mut sum_uv = Vec2::ZERO;
                                for &nbr in neighbors {
                                    sum_uv += uv_map.get(&nbr).copied().unwrap_or(Vec2::splat(0.5));
                                }
                                uv_map.insert(vk, sum_uv / (neighbors.len() as f32));
                            }
                        }
                    }
                }
            }
        } else {
            // Planar tangent projection for flat charts or when boundary chaining is unavailable
            for &vk in &chart_verts {
                let p = self.vertices[vk].position - center;
                let u = p.dot(t_axis);
                let v = p.dot(b_axis);
                uv_map.insert(vk, vec2(u, v));
            }
        }

        // Normalize chart coordinates to local [0, 1] bounds
        let mut min_uv = Vec2::splat(f32::INFINITY);
        let mut max_uv = Vec2::splat(f32::NEG_INFINITY);
        for uv in uv_map.values() {
            min_uv = min_uv.min(*uv);
            max_uv = max_uv.max(*uv);
        }

        let range = max_uv - min_uv;
        let scale = if range.x.max(range.y) > 1e-6 {
            1.0 / range.x.max(range.y)
        } else {
            1.0
        };

        // Scale proportional to 3D surface area for uniform texel density
        let area_weight = (total_3d_area.max(1e-4)).sqrt();
        for uv in uv_map.values_mut() {
            *uv = (*uv - min_uv) * scale * area_weight;
        }

        uv_map
    }
}

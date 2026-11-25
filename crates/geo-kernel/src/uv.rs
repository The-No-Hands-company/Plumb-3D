use crate::mesh::{EdgeKey, FaceKey, HalfEdgeKey, Mesh, VertexKey};
use core_math::{vec2, Vec2, Vec3};
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectionAxis {
    X,
    Y,
    Z,
}

impl Mesh {
    /// Apply planar UV mapping projection along the chosen axis.
    /// Maps 3D coordinates onto a [0, 1] UV space normalized to the mesh bounding box.
    pub fn generate_planar_uv(&mut self, axis: ProjectionAxis) {
        if self.vertices.is_empty() {
            return;
        }

        let mut min_coord = Vec2::splat(f32::INFINITY);
        let mut max_coord = Vec2::splat(f32::NEG_INFINITY);

        // Pass 1: compute bounds along projection plane
        for (_, v) in &self.vertices {
            let (u_raw, v_raw) = match axis {
                ProjectionAxis::X => (v.position.z, v.position.y),
                ProjectionAxis::Y => (v.position.x, v.position.z),
                ProjectionAxis::Z => (v.position.x, v.position.y),
            };
            min_coord.x = min_coord.x.min(u_raw);
            min_coord.y = min_coord.y.min(v_raw);
            max_coord.x = max_coord.x.max(u_raw);
            max_coord.y = max_coord.y.max(v_raw);
        }

        let size = max_coord - min_coord;
        let scale_x = if size.x.abs() > 1e-6 { 1.0 / size.x } else { 1.0 };
        let scale_y = if size.y.abs() > 1e-6 { 1.0 / size.y } else { 1.0 };

        // Pass 2: assign normalized UV coordinates
        for (_, v) in &mut self.vertices {
            let (u_raw, v_raw) = match axis {
                ProjectionAxis::X => (v.position.z, v.position.y),
                ProjectionAxis::Y => (v.position.x, v.position.z),
                ProjectionAxis::Z => (v.position.x, v.position.y),
            };
            v.uv = vec2((u_raw - min_coord.x) * scale_x, (v_raw - min_coord.y) * scale_y);
        }
    }

    /// Apply cylindrical UV mapping projection wrapped around the specified central axis.
    /// Computes azimuth angle theta for U [0, 1] and height for V [0, 1].
    pub fn generate_cylindrical_uv(&mut self, axis: ProjectionAxis) {
        if self.vertices.is_empty() {
            return;
        }

        let mut min_h = f32::INFINITY;
        let mut max_h = f32::NEG_INFINITY;

        for (_, v) in &self.vertices {
            let h = match axis {
                ProjectionAxis::X => v.position.x,
                ProjectionAxis::Y => v.position.y,
                ProjectionAxis::Z => v.position.z,
            };
            min_h = min_h.min(h);
            max_h = max_h.max(h);
        }

        let h_range = (max_h - min_h).max(1e-6);

        for (_, v) in &mut self.vertices {
            let (x, z, h) = match axis {
                ProjectionAxis::X => (v.position.y, v.position.z, v.position.x),
                ProjectionAxis::Y => (v.position.x, v.position.z, v.position.y),
                ProjectionAxis::Z => (v.position.x, v.position.y, v.position.z),
            };

            let theta = z.atan2(x); // [-PI, PI]
            let u = (theta / std::f32::consts::TAU) + 0.5; // [0, 1]
            let v_coord = (h - min_h) / h_range;

            v.uv = vec2(u.clamp(0.0, 1.0), v_coord.clamp(0.0, 1.0));
        }
    }

    /// Apply spherical UV mapping projection (equirectangular projection).
    /// Computes longitude for U [0, 1] and latitude for V [0, 1].
    pub fn generate_spherical_uv(&mut self) {
        if self.vertices.is_empty() {
            return;
        }

        // Center of mesh
        let mut center = Vec3::ZERO;
        for (_, v) in &self.vertices {
            center += v.position;
        }
        center /= self.vertices.len() as f32;

        for (_, v) in &mut self.vertices {
            let d = (v.position - center).normalize_or_zero();
            let u = (d.z.atan2(d.x) / std::f32::consts::TAU) + 0.5;
            let v_coord = d.y.clamp(-1.0, 1.0).asin() / std::f32::consts::PI + 0.5;

            v.uv = vec2(u.clamp(0.0, 1.0), v_coord.clamp(0.0, 1.0));
        }
    }

    /// Apply box (tri-planar) UV mapping projection.
    /// Projects each face along its dominant normal axis (+-X, +-Y, or +-Z).
    pub fn generate_box_uv(&mut self) {
        let fk_list: Vec<FaceKey> = self.faces.keys().collect();

        for fk in fk_list {
            let normal = self.faces[fk].normal;
            let abs_n = normal.abs();

            let dominant_axis = if abs_n.x >= abs_n.y && abs_n.x >= abs_n.z {
                ProjectionAxis::X
            } else if abs_n.y >= abs_n.x && abs_n.y >= abs_n.z {
                ProjectionAxis::Y
            } else {
                ProjectionAxis::Z
            };

            let v_keys = self.face_vertices(fk);
            for vk in v_keys {
                let p = self.vertices[vk].position;
                let (u, v) = match dominant_axis {
                    ProjectionAxis::X => (p.z, p.y),
                    ProjectionAxis::Y => (p.x, p.z),
                    ProjectionAxis::Z => (p.x, p.y),
                };
                self.vertices[vk].uv = vec2(u.fract().abs(), v.fract().abs());
            }
        }
    }

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

    /// Packs multiple unwrapped UV charts / islands into the normalized [margin, 1.0 - margin] texture space.
    pub fn pack_uv_islands(&mut self, charts_uvs: &[HashMap<VertexKey, Vec2>], margin: f32) {
        if charts_uvs.is_empty() {
            return;
        }

        struct IslandInfo {
            chart_idx: usize,
            min: Vec2,
            size: Vec2,
        }

        let mut islands = Vec::with_capacity(charts_uvs.len());
        for (idx, chart) in charts_uvs.iter().enumerate() {
            if chart.is_empty() {
                continue;
            }
            let mut min = Vec2::splat(f32::INFINITY);
            let mut max = Vec2::splat(f32::NEG_INFINITY);
            for uv in chart.values() {
                min = min.min(*uv);
                max = max.max(*uv);
            }
            let size = max - min;
            islands.push(IslandInfo {
                chart_idx: idx,
                min,
                size: vec2(size.x.max(0.01), size.y.max(0.01)),
            });
        }

        if islands.is_empty() {
            return;
        }

        // Sort islands by height descending for shelf bin packing
        islands.sort_by(|a, b| b.size.y.partial_cmp(&a.size.y).unwrap_or(std::cmp::Ordering::Equal));

        let num_islands = islands.len();
        let grid_cols = ((num_islands as f32).sqrt().ceil() as usize).max(1);

        let mut packed_positions = Vec::with_capacity(islands.len());
        let mut cur_x = 0.0_f32;
        let mut cur_y = 0.0_f32;
        let mut row_h = 0.0_f32;
        let mut col_count = 0;
        let mut total_w = 0.0_f32;
        let mut total_h = 0.0_f32;

        let padding = margin * 2.0;

        for item in &islands {
            if col_count >= grid_cols {
                cur_y += row_h + padding;
                cur_x = 0.0;
                row_h = 0.0;
                col_count = 0;
            }

            packed_positions.push(vec2(cur_x, cur_y));
            cur_x += item.size.x + padding;
            row_h = row_h.max(item.size.y);
            col_count += 1;

            total_w = total_w.max(cur_x);
            total_h = total_h.max(cur_y + row_h);
        }

        let available_space = (1.0 - 2.0 * margin).max(0.1);
        let max_dim = total_w.max(total_h).max(1e-4);
        let final_scale = available_space / max_dim;

        // Apply final packed coordinates to the mesh's vertex UV attributes
        for (i, item) in islands.iter().enumerate() {
            let offset = packed_positions[i];
            let chart_map = &charts_uvs[item.chart_idx];
            for (&vk, &uv) in chart_map {
                let local = uv - item.min;
                let packed_uv = vec2(
                    margin + (offset.x + local.x) * final_scale,
                    margin + (offset.y + local.y) * final_scale,
                );
                self.vertices[vk].uv = packed_uv;
            }
        }
    }

    /// Smart UV Project: automatically detects seams by dihedral angle, partitions into charts, unwraps, and packs.
    pub fn smart_uv_project(&mut self, angle_threshold_deg: f32, margin: f32) {
        self.mark_seams_by_angle(angle_threshold_deg);
        let charts = self.partition_uv_charts();
        let mut charts_uvs = Vec::with_capacity(charts.len());
        for chart in &charts {
            charts_uvs.push(self.unwrap_chart(chart));
        }
        self.pack_uv_islands(&charts_uvs, margin);
    }

    /// Unwrap: partitions mesh into charts using existing marked seams (`edge.is_seam`), unwraps, and packs.
    pub fn unwrap_seams(&mut self, margin: f32) {
        let charts = self.partition_uv_charts();
        let mut charts_uvs = Vec::with_capacity(charts.len());
        for chart in &charts {
            charts_uvs.push(self.unwrap_chart(chart));
        }
        self.pack_uv_islands(&charts_uvs, margin);
    }

    /// Packs current UV coordinates into normalized [0, 1] texture space with margin.
    pub fn pack_existing_uvs(&mut self, margin: f32) {
        let charts = self.partition_uv_charts();
        let mut charts_uvs = Vec::with_capacity(charts.len());
        for chart in &charts {
            let mut chart_map = HashMap::new();
            for &fk in chart {
                for vk in self.face_vertices(fk) {
                    chart_map.insert(vk, self.vertices[vk].uv);
                }
            }
            if !chart_map.is_empty() {
                charts_uvs.push(chart_map);
            }
        }
        self.pack_uv_islands(&charts_uvs, margin);
    }

    /// Rotate all vertex UV coordinates by 90 degrees around pivot (0.5, 0.5)
    pub fn rotate_uvs_90(&mut self, clockwise: bool) {
        let pivot = vec2(0.5, 0.5);
        for v in self.vertices.values_mut() {
            let rel = v.uv - pivot;
            let rot = if clockwise {
                vec2(rel.y, -rel.x)
            } else {
                vec2(-rel.y, rel.x)
            };
            v.uv = rot + pivot;
        }
    }

    /// Flip all vertex UV coordinates horizontally or vertically around pivot (0.5, 0.5)
    pub fn flip_uvs(&mut self, horizontal: bool) {
        let pivot = 0.5_f32;
        for v in self.vertices.values_mut() {
            if horizontal {
                v.uv.x = pivot - (v.uv.x - pivot);
            } else {
                v.uv.y = pivot - (v.uv.y - pivot);
            }
        }
    }

    /// Uniformly scale all vertex UV coordinates relative to pivot (0.5, 0.5)
    pub fn scale_uvs(&mut self, factor: f32) {
        let pivot = vec2(0.5, 0.5);
        for v in self.vertices.values_mut() {
            v.uv = (v.uv - pivot) * factor + pivot;
        }
    }

    /// Relaxes and smooths vertex UV coordinates using Laplacian diffusion to minimize texture distortion.
    /// Pinned boundary vertices retain chart contours while interior UVs equalize tension.
    pub fn relax_uvs(&mut self, iterations: usize, factor: f32) {
        if self.vertices.len() < 3 || iterations == 0 {
            return;
        }

        let blend = factor.clamp(0.01, 1.0);

        // Build vertex adjacency and identify boundaries
        let mut adj: HashMap<VertexKey, Vec<VertexKey>> = HashMap::new();
        let mut boundary_verts = HashSet::new();

        for (_ek, edge) in &self.edges {
            let he = &self.half_edges[edge.half_edge];
            let v0 = he.origin;
            let v1 = he.target;
            adj.entry(v0).or_default().push(v1);
            adj.entry(v1).or_default().push(v0);

            // Boundary edge: half-edge has no face or twin has no face, or marked as seam
            let twin_he = &self.half_edges[he.twin];
            if edge.is_seam || he.face.is_none() || twin_he.face.is_none() {
                boundary_verts.insert(v0);
                boundary_verts.insert(v1);
            }
        }

        for _ in 0..iterations {
            let mut new_uvs: HashMap<VertexKey, Vec2> = HashMap::with_capacity(self.vertices.len());

            for (vk, v) in &self.vertices {
                // If boundary vertex, keep pinned to preserve UV island boundaries
                if boundary_verts.contains(&vk) {
                    continue;
                }

                if let Some(nbrs) = adj.get(&vk) {
                    if !nbrs.is_empty() {
                        let mut sum_uv = Vec2::ZERO;
                        for &n_vk in nbrs {
                            sum_uv += self.vertices[n_vk].uv;
                        }
                        let avg_uv = sum_uv / (nbrs.len() as f32);
                        new_uvs.insert(vk, v.uv.lerp(avg_uv, blend));
                    }
                }
            }

            for (vk, uv) in new_uvs {
                self.vertices[vk].uv = uv;
            }
        }
    }
}

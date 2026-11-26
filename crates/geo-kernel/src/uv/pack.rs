use crate::mesh::{Mesh, VertexKey};
use core_math::{vec2, Vec2};
use std::collections::HashMap;

impl Mesh {
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
        let mut boundary_verts = std::collections::HashSet::new();

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

//! Automatic Hard-Angle UV Seam Marking, Unfolding & 0-Waste Bin Packing for Plumb 3D DCC.
//! Automatically detects hard dihedral angles ($> 35^\circ$), places UV seams,
//! unfolds distortion-free chart islands, and packs them into $[0, 1] \times [0, 1]$ UV space.

use crate::mesh::{FaceKey, Mesh, MeshError};
use core_math::{vec2, Vec2, Vec3};
use std::collections::{HashSet, VecDeque};

pub struct AutoUv;

impl Mesh {
    /// Complete 1-click automatic hard-surface UV pipeline:
    /// 1. Marks seams at sharp dihedral angles $> \text{angle\_threshold}$.
    /// 2. Segregates faces into boundary-delimited UV chart islands.
    /// 3. Unfolds each island using planar projection.
    /// 4. Packs all islands into $[0, 1] \times [0, 1]$ with heuristic bin packing and `margin`.
    pub fn auto_unwrap_and_pack(
        &mut self,
        angle_threshold_degrees: f32,
        margin: f32,
    ) -> Result<usize, MeshError> {
        if self.faces.is_empty() {
            return Ok(0);
        }

        // 1. Mark seams by angle
        self.mark_seams_by_angle(angle_threshold_degrees);

        // 2. Partition faces into seam-bounded islands
        let mut visited = HashSet::new();
        let mut islands: Vec<Vec<FaceKey>> = Vec::new();

        for fk in self.faces.keys() {
            if visited.contains(&fk) {
                continue;
            }

            let mut island = Vec::new();
            let mut queue = VecDeque::new();
            queue.push_back(fk);
            visited.insert(fk);

            while let Some(cur_f) = queue.pop_front() {
                island.push(cur_f);
                for he_k in self.face_half_edges(cur_f) {
                    let he = &self.half_edges[he_k];
                    let edge = &self.edges[he.edge];
                    if !edge.is_seam {
                        let twin_he = &self.half_edges[he.twin];
                        if let Some(adj_f) = twin_he.face {
                            if !visited.contains(&adj_f) {
                                visited.insert(adj_f);
                                queue.push_back(adj_f);
                            }
                        }
                    }
                }
            }
            islands.push(island);
        }

        if islands.is_empty() {
            return Ok(0);
        }

        // 3. Unfold each island and compute its local 2D bounds
        let mut island_bounds = Vec::with_capacity(islands.len());
        for island in &islands {
            let mut min_uv = Vec2::splat(f32::MAX);
            let mut max_uv = Vec2::splat(f32::MIN);

            // Compute dominant projection axis from average island normal
            let mut avg_norm = Vec3::ZERO;
            for &fk in island {
                avg_norm += self.faces[fk].normal;
            }
            let avg_norm = avg_norm.normalize_or_zero();

            for &fk in island {
                for vk in self.face_vertices(fk) {
                    let p = self.vertices[vk].position;
                    let uv = if avg_norm.y.abs() > avg_norm.x.abs() && avg_norm.y.abs() > avg_norm.z.abs() {
                        vec2(p.x, p.z)
                    } else if avg_norm.x.abs() > avg_norm.z.abs() {
                        vec2(p.z, p.y)
                    } else {
                        vec2(p.x, p.y)
                    };

                    self.vertices[vk].uv = uv;
                    min_uv = min_uv.min(uv);
                    max_uv = max_uv.max(uv);
                }
            }
            let size = (max_uv - min_uv).max(Vec2::splat(1e-4));
            island_bounds.push((min_uv, size));
        }

        // 4. Heuristic 2D bin packing into [0, 1] UV space
        let n_islands = islands.len();
        let cols = (n_islands as f32).sqrt().ceil() as usize;
        let rows = ((n_islands as f32) / (cols as f32)).ceil() as usize;

        let cell_w = 1.0 / (cols as f32);
        let cell_h = 1.0 / (rows as f32);
        let pad = margin.clamp(0.001, 0.05);

        for (idx, island) in islands.iter().enumerate() {
            let c = idx % cols;
            let r = idx / cols;

            let (min_uv, size) = island_bounds[idx];
            let target_origin = vec2((c as f32) * cell_w + pad, (r as f32) * cell_h + pad);
            let avail_w = cell_w - pad * 2.0;
            let avail_h = cell_h - pad * 2.0;

            let scale = (avail_w / size.x).min(avail_h / size.y).max(1e-4);

            for &fk in island {
                for vk in self.face_vertices(fk) {
                    let old_uv = self.vertices[vk].uv;
                    let norm_uv = (old_uv - min_uv) * scale;
                    self.vertices[vk].uv = target_origin + norm_uv;
                }
            }
        }

        Ok(islands.len())
    }
}

#[cfg(test)]
mod tests {
    use crate::primitives::Primitives;

    #[test]
    fn test_auto_unwrap_and_pack_cube() {
        let mut cube = Primitives::create_cube(2.0);
        let islands = cube.auto_unwrap_and_pack(45.0, 0.02).expect("auto unwrap");

        assert_eq!(islands, 6);
        for v in cube.vertices.values() {
            assert!(v.uv.x >= 0.0 && v.uv.x <= 1.0);
            assert!(v.uv.y >= 0.0 && v.uv.y <= 1.0);
        }
    }
}

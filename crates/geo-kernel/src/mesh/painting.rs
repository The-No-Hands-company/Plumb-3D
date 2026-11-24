use core_math::Vec3;

use super::types::{Mesh, MeshError, VertexKey};

impl Mesh {
    /// Paint weights into the named vertex group around `hit_point` with brush `radius`, `strength`, and `falloff`.
    pub fn apply_weight_brush(
        &mut self,
        group_name: &str,
        hit_point: Vec3,
        target_weight: f32,
        radius: f32,
        strength: f32,
        falloff: crate::sculpt::BrushFalloff,
    ) -> usize {
        let r_sq = radius * radius;
        let mut affected = 0;

        let to_update: Vec<(VertexKey, f32)> = self.vertices
            .iter()
            .filter_map(|(vk, v)| {
                let d_sq = (v.position - hit_point).length_squared();
                if d_sq <= r_sq {
                    let dist = d_sq.sqrt();
                    let f = falloff.evaluate(dist / radius.max(1e-4));
                    Some((vk, f))
                } else {
                    None
                }
            })
            .collect();

        for (vk, factor) in to_update {
            let current = self.get_vertex_weight(group_name, vk);
            let blend = factor * strength;
            let new_w = current + (target_weight - current) * blend;
            self.set_vertex_weight(group_name, vk, new_w);
            affected += 1;
        }

        affected
    }

    /// Linear weight gradient across the mesh between start and end 3D coordinates.
    pub fn apply_weight_gradient(
        &mut self,
        group_name: &str,
        start_point: Vec3,
        end_point: Vec3,
        start_weight: f32,
        end_weight: f32,
    ) -> usize {
        let seg = end_point - start_point;
        let len_sq = seg.length_squared();
        if len_sq < 1e-6 {
            return 0;
        }

        let to_update: Vec<(VertexKey, f32)> = self.vertices
            .iter()
            .map(|(vk, v)| {
                let t = ((v.position - start_point).dot(seg) / len_sq).clamp(0.0, 1.0);
                let w = start_weight + (end_weight - start_weight) * t;
                (vk, w)
            })
            .collect();

        let count = to_update.len();
        for (vk, w) in to_update {
            self.set_vertex_weight(group_name, vk, w);
        }

        count
    }

    /// Paint RGBA color onto vertices around `hit_point` with brush `radius`, `strength`, and `falloff`.
    pub fn apply_color_brush(
        &mut self,
        hit_point: Vec3,
        target_color: [f32; 4],
        radius: f32,
        strength: f32,
        falloff: crate::sculpt::BrushFalloff,
    ) -> usize {
        let r_sq = radius * radius;
        let mut affected = 0;

        let to_update: Vec<(VertexKey, f32)> = self.vertices
            .iter()
            .filter_map(|(vk, v)| {
                let d_sq = (v.position - hit_point).length_squared();
                if d_sq <= r_sq {
                    let dist = d_sq.sqrt();
                    let f = falloff.evaluate(dist / radius.max(1e-4));
                    Some((vk, f))
                } else {
                    None
                }
            })
            .collect();

        for (vk, factor) in to_update {
            if let Some(v) = self.vertices.get_mut(vk) {
                let blend = (factor * strength).clamp(0.0, 1.0);
                v.color[0] = (v.color[0] + (target_color[0] - v.color[0]) * blend).clamp(0.0, 1.0);
                v.color[1] = (v.color[1] + (target_color[1] - v.color[1]) * blend).clamp(0.0, 1.0);
                v.color[2] = (v.color[2] + (target_color[2] - v.color[2]) * blend).clamp(0.0, 1.0);
                v.color[3] = (v.color[3] + (target_color[3] - v.color[3]) * blend).clamp(0.0, 1.0);
                affected += 1;
            }
        }

        affected
    }

    /// Mirrors a vertex group across a coordinate axis to a target vertex group (e.g. Left -> Right).
    /// For each vertex in the source group, finds the symmetric vertex across the mirror plane within `tolerance`.
    /// Creates the target group if it does not already exist.
    /// Returns the number of mapped vertex weights.
    pub fn mirror_vertex_group(
        &mut self,
        source_group: &str,
        target_group: &str,
        axis: crate::modifiers::MirrorAxis,
        tolerance: f32,
    ) -> Result<usize, MeshError> {
        let src_weights = if let Some(vg) = self.get_vertex_group(source_group) {
            vg.weights.clone()
        } else {
            return Ok(0);
        };

        let tol_sq = tolerance * tolerance;
        let mut mapped_weights: Vec<(VertexKey, f32)> = Vec::new();

        for (src_vk, &weight) in &src_weights {
            if let Some(src_v) = self.vertices.get(*src_vk) {
                let mut mirror_pos = src_v.position;
                match axis {
                    crate::modifiers::MirrorAxis::X => mirror_pos.x = -mirror_pos.x,
                    crate::modifiers::MirrorAxis::Y => mirror_pos.y = -mirror_pos.y,
                    crate::modifiers::MirrorAxis::Z => mirror_pos.z = -mirror_pos.z,
                }

                // Find closest vertex to mirror_pos
                let mut best_vk = None;
                let mut best_dist_sq = tol_sq;
                for (other_vk, other_v) in &self.vertices {
                    let d_sq = (other_v.position - mirror_pos).length_squared();
                    if d_sq < best_dist_sq {
                        best_dist_sq = d_sq;
                        best_vk = Some(other_vk);
                    }
                }

                if let Some(target_vk) = best_vk {
                    mapped_weights.push((target_vk, weight));
                }
            }
        }

        let count = mapped_weights.len();
        let target_idx = if let Some(idx) = self.vertex_groups.iter().position(|vg| vg.name == target_group) {
            idx
        } else {
            self.add_vertex_group(target_group)
        };

        for (vk, w) in mapped_weights {
            self.vertex_groups[target_idx].set_weight(vk, w);
        }

        Ok(count)
    }

    /// Normalizes vertex group weights across the specified vertex groups for every vertex
    /// so that the sum of weights for each vertex equals 1.0 (vital for Linear Blend Skinning).
    pub fn normalize_vertex_groups(&mut self, group_names: &[String]) {
        let indices: Vec<usize> = group_names
            .iter()
            .filter_map(|name| self.vertex_groups.iter().position(|vg| &vg.name == name))
            .collect();

        if indices.is_empty() {
            return;
        }

        for (vk, _) in &self.vertices {
            let sum: f32 = indices
                .iter()
                .map(|&idx| self.vertex_groups[idx].get_weight(vk))
                .sum();

            if sum > 1e-5 {
                let inv_sum = 1.0 / sum;
                for &idx in &indices {
                    let current = self.vertex_groups[idx].get_weight(vk);
                    if current > 0.0 {
                        self.vertex_groups[idx].set_weight(vk, current * inv_sum);
                    }
                }
            }
        }
    }
}

use crate::mesh::{Mesh, VertexKey};
use core_math::Vec3;

/// Available sculpt brush types (Blender / Maya / ZBrush style)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SculptBrush {
    /// Displaces vertices outward/inward along the hit surface normal
    Draw,
    /// Displaces vertices outward/inward along each vertex's individual normal
    Inflate,
    /// Laplacian smoothing towards neighboring 1-ring centroid
    Smooth,
    /// Projects vertices towards the local hit tangent plane to create planar flats
    Flatten,
    /// Translates vertices along an arbitrary 3D drag vector
    Grab,
    /// Pulls vertices radially inward toward the brush center (or pushes outward if inverted)
    Pinch,
    /// Carves a sharp groove or ridge by displacing along normal and pinching laterally
    Crease,
    /// Pulls geometry along cursor motion, dynamically extruding forms
    SnakeHook,
    /// Builds up volume clamped to an offset plane like physical clay
    Clay,
    /// Paints an immutable mask pinning vertices from deformation
    Mask,
}

/// Falloff curves for sculpt brushes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrushFalloff {
    /// Smoothstep cubic curve: (1 - t)^2 * (3 - 2t)
    Smooth,
    /// Spherical dome curve: sqrt(1 - t^2)
    Spherical,
    /// Linear decline: 1 - t
    Linear,
    /// Sharp tip: (1 - t)^3
    Sharp,
}

/// Procedural and sampled alpha texture for sculpting brushes (ZBrush Alphas / Stencils)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BrushAlphaTexture {
    #[default]
    None,
    Tile,
    Scales,
    Noise,
}

impl BrushAlphaTexture {
    #[inline]
    pub fn evaluate(&self, u: f32, v: f32) -> f32 {
        match self {
            Self::None => 1.0,
            Self::Tile => {
                let su = (u * 6.0).sin();
                let sv = (v * 6.0).sin();
                (su * sv).abs().clamp(0.15, 1.0)
            }
            Self::Scales => {
                let r = (u * u + v * v).sqrt() * 8.0;
                let angle = v.atan2(u) * 5.0;
                ((r.sin() + angle.cos()) * 0.5 + 0.5).clamp(0.1, 1.0)
            }
            Self::Noise => {
                let n = ((u * 12.34 + v * 56.78).sin() * 43758.547).fract();
                0.25 + 0.75 * n.abs()
            }
        }
    }
}

impl BrushFalloff {
    #[inline]
    pub fn evaluate(&self, normalized_dist: f32) -> f32 {
        let t = normalized_dist.clamp(0.0, 1.0);
        match self {
            BrushFalloff::Smooth => {
                let mt = 1.0 - t;
                mt * mt * (3.0 - 2.0 * mt)
            }
            BrushFalloff::Spherical => (1.0 - t * t).max(0.0).sqrt(),
            BrushFalloff::Linear => 1.0 - t,
            BrushFalloff::Sharp => {
                let mt = 1.0 - t;
                mt * mt * mt
            }
        }
    }
}

/// Sculpt brush configuration settings
#[derive(Debug, Clone, Copy)]
pub struct SculptSettings {
    pub brush: SculptBrush,
    pub radius: f32,
    pub strength: f32,
    pub falloff: BrushFalloff,
    pub alpha: BrushAlphaTexture,
    /// When true, inverts the stroke (e.g. Draw indents instead of extrudes, Deflate instead of Inflate)
    pub invert: bool,
}

impl Default for SculptSettings {
    fn default() -> Self {
        Self {
            brush: SculptBrush::Draw,
            radius: 0.5,
            strength: 0.35,
            falloff: BrushFalloff::Smooth,
            alpha: BrushAlphaTexture::None,
            invert: false,
        }
    }
}

impl Mesh {
    /// Apply a single brush step at `hit_point` with surface `hit_normal`.
    /// `motion_delta` is used by the `Grab` brush to translate vertices along the cursor stroke.
    pub fn apply_sculpt_brush(
        &mut self,
        hit_point: Vec3,
        hit_normal: Vec3,
        settings: &SculptSettings,
        motion_delta: Option<Vec3>,
    ) -> usize {
        let r = settings.radius.max(1e-4);
        let sign = if settings.invert { -1.0 } else { 1.0 };
        let str_mag = settings.strength;

        // 1. Identify all vertices within the spherical brush footprint
        let mut affected: Vec<(VertexKey, f32, f32)> = Vec::new(); // (VertexKey, dist, weight)

        for (vk, v) in &self.vertices {
            let dist = (v.position - hit_point).length();
            if dist <= r {
                let norm_dist = dist / r;
                let local_p = v.position - hit_point;
                let normal_proj = hit_normal * local_p.dot(hit_normal);
                let tangent_p = local_p - normal_proj;
                let u = tangent_p.x / r;
                let v_coord = tangent_p.z / r;
                let alpha_val = settings.alpha.evaluate(u, v_coord);
                let raw_weight = settings.falloff.evaluate(norm_dist) * alpha_val;
                let unmasked = (1.0 - v.sculpt_mask).clamp(0.0, 1.0);
                let weight = if settings.brush == SculptBrush::Mask { raw_weight } else { raw_weight * unmasked };
                if weight > 1e-5 {
                    affected.push((vk, dist, weight));
                }
            }
        }

        if affected.is_empty() {
            return 0;
        }

        let count = affected.len();

        match settings.brush {
            SculptBrush::Draw => {
                let disp = hit_normal * (str_mag * sign * 0.1);
                for &(vk, _, weight) in &affected {
                    if let Some(v) = self.vertices.get_mut(vk) {
                        v.position += disp * weight;
                    }
                }
            }
            SculptBrush::Inflate => {
                for &(vk, _, weight) in &affected {
                    let v_norm = self.vertices[vk].normal;
                    let disp = v_norm * (str_mag * sign * 0.1 * weight);
                    if let Some(v) = self.vertices.get_mut(vk) {
                        v.position += disp;
                    }
                }
            }
            SculptBrush::Smooth => {
                // Laplacian smoothing using 1-ring half-edge neighbor averaging
                let mut target_positions: Vec<(VertexKey, Vec3)> = Vec::with_capacity(count);

                for &(vk, _, weight) in &affected {
                    let v = &self.vertices[vk];
                    let mut neighbor_sum = Vec3::ZERO;
                    let mut neighbor_count = 0;

                    for &he_key in &v.half_edges {
                        if let Some(he) = self.half_edges.get(he_key) {
                            let neighbor_vk = he.target;
                            if let Some(n_vert) = self.vertices.get(neighbor_vk) {
                                neighbor_sum += n_vert.position;
                                neighbor_count += 1;
                            }
                        }
                    }

                    if neighbor_count > 0 {
                        let avg_pos = neighbor_sum / neighbor_count as f32;
                        let alpha = (str_mag * weight).clamp(0.0, 1.0);
                        let smoothed = v.position * (1.0 - alpha) + avg_pos * alpha;
                        target_positions.push((vk, smoothed));
                    }
                }

                for (vk, smoothed) in target_positions {
                    if let Some(v) = self.vertices.get_mut(vk) {
                        v.position = smoothed;
                    }
                }
            }
            SculptBrush::Flatten => {
                // Project vertices towards the tangent plane passing through hit_point with hit_normal
                let plane_point = hit_point;
                let plane_normal = hit_normal;

                for &(vk, _, weight) in &affected {
                    if let Some(v) = self.vertices.get_mut(vk) {
                        let dist_to_plane = (v.position - plane_point).dot(plane_normal);
                        let alpha = (str_mag * weight).clamp(0.0, 1.0);
                        v.position -= plane_normal * (dist_to_plane * alpha);
                    }
                }
            }
            SculptBrush::Grab => {
                let drag = motion_delta.unwrap_or(Vec3::ZERO) * sign;
                for &(vk, _, weight) in &affected {
                    if let Some(v) = self.vertices.get_mut(vk) {
                        v.position += drag * weight;
                    }
                }
            }
            SculptBrush::Pinch => {
                for &(vk, _, weight) in &affected {
                    if let Some(v) = self.vertices.get_mut(vk) {
                        let to_center = hit_point - v.position;
                        let disp = to_center * (str_mag * sign * 0.2 * weight);
                        v.position += disp;
                    }
                }
            }
            SculptBrush::Crease => {
                for &(vk, _, weight) in &affected {
                    if let Some(v) = self.vertices.get_mut(vk) {
                        let norm_disp = -hit_normal * (str_mag * sign * 0.12 * weight);
                        let to_center = hit_point - v.position;
                        let lateral = to_center - hit_normal * to_center.dot(hit_normal);
                        let lateral_disp = lateral * (str_mag * 0.2 * weight * sign);
                        v.position += norm_disp + lateral_disp;
                    }
                }
            }
            SculptBrush::SnakeHook => {
                let drag = motion_delta.unwrap_or(Vec3::ZERO) * sign;
                for &(vk, _, weight) in &affected {
                    if let Some(v) = self.vertices.get_mut(vk) {
                        let factor = weight * weight * (3.0 - 2.0 * weight);
                        v.position += drag * factor * (1.0 + str_mag * 0.5);
                    }
                }
            }
            SculptBrush::Clay => {
                let clay_offset = hit_normal * (str_mag * sign * 0.15);
                let plane_point = hit_point + clay_offset;
                for &(vk, _, weight) in &affected {
                    if let Some(v) = self.vertices.get_mut(vk) {
                        let dist_to_plane = (plane_point - v.position).dot(hit_normal);
                        if (sign >= 0.0 && dist_to_plane > 0.0) || (sign < 0.0 && dist_to_plane < 0.0) {
                            let alpha = (str_mag * weight).clamp(0.0, 1.0);
                            v.position += hit_normal * (dist_to_plane * alpha);
                        }
                    }
                }
            }
            SculptBrush::Mask => {
                let mask_delta = str_mag * sign;
                for &(vk, _, weight) in &affected {
                    if let Some(v) = self.vertices.get_mut(vk) {
                        v.sculpt_mask = (v.sculpt_mask + mask_delta * weight).clamp(0.0, 1.0);
                    }
                }
            }
        }

        self.update_normals();
        count
    }

    /// Inverts the sculpt mask across all vertices (Blender Ctrl+I in Sculpt Mode).
    pub fn invert_sculpt_mask(&mut self) {
        for v in self.vertices.values_mut() {
            v.sculpt_mask = (1.0 - v.sculpt_mask).clamp(0.0, 1.0);
        }
    }

    /// Clears the sculpt mask across all vertices (Blender Alt+M in Sculpt Mode).
    pub fn clear_sculpt_mask(&mut self) {
        for v in self.vertices.values_mut() {
            v.sculpt_mask = 0.0;
        }
    }

    /// Smooths the sculpt mask across 1-ring neighbors for soft feathered boundaries.
    pub fn smooth_sculpt_mask(&mut self, factor: f32) {
        let f = factor.clamp(0.0, 1.0);
        let mut smoothed_masks: Vec<(VertexKey, f32)> = Vec::with_capacity(self.vertices.len());
        for (vk, v) in &self.vertices {
            let mut neighbor_sum = 0.0;
            let mut count = 0;
            for &he_key in &v.half_edges {
                if let Some(he) = self.half_edges.get(he_key) {
                    if let Some(nv) = self.vertices.get(he.target) {
                        neighbor_sum += nv.sculpt_mask;
                        count += 1;
                    }
                }
            }
            if count > 0 {
                let avg = neighbor_sum / count as f32;
                smoothed_masks.push((vk, v.sculpt_mask * (1.0 - f) + avg * f));
            }
        }
        for (vk, mask) in smoothed_masks {
            if let Some(v) = self.vertices.get_mut(vk) {
                v.sculpt_mask = mask.clamp(0.0, 1.0);
            }
        }
    }

    /// Sets an individual vertex's sculpt mask weight.
    pub fn set_vertex_sculpt_mask(&mut self, vk: VertexKey, mask: f32) {
        if let Some(v) = self.vertices.get_mut(vk) {
            v.sculpt_mask = mask.clamp(0.0, 1.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_brush_alpha_textures() {
        let none = BrushAlphaTexture::None;
        assert_eq!(none.evaluate(0.5, 0.5), 1.0);

        let tile = BrushAlphaTexture::Tile;
        let v_tile = tile.evaluate(0.2, 0.3);
        assert!(v_tile >= 0.1 && v_tile <= 1.0);

        let scales = BrushAlphaTexture::Scales;
        let v_scales = scales.evaluate(0.1, 0.2);
        assert!(v_scales >= 0.05 && v_scales <= 1.0);

        let noise = BrushAlphaTexture::Noise;
        let v_noise = noise.evaluate(0.4, 0.7);
        assert!(v_noise >= 0.25 && v_noise <= 1.0);
    }
}

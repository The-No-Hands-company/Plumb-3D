use crate::mesh::Mesh;
use crate::sculpt::BrushFalloff;
use core_math::{vec2, vec3, Vec2, Vec3};

/// PBR material channel target for texture painting
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaintChannel {
    Albedo,
    Roughness,
    Metallic,
    Height,
}

/// Blending modes for texture brush strokes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaintBlendMode {
    Mix,
    Add,
    Multiply,
    Screen,
    Erase,
}

/// Dynamic 2D RGBA bitmap texture canvas for viewport painting
#[derive(Debug, Clone)]
pub struct PaintTexture {
    pub width: usize,
    pub height: usize,
    pub data: Vec<[u8; 4]>,
}

impl PaintTexture {
    pub fn new(width: usize, height: usize, fill_color: [u8; 4]) -> Self {
        let size = width * height;
        Self {
            width,
            height,
            data: vec![fill_color; size],
        }
    }

    #[inline]
    pub fn get_pixel(&self, x: usize, y: usize) -> [u8; 4] {
        if x < self.width && y < self.height {
            self.data[y * self.width + x]
        } else {
            [0, 0, 0, 0]
        }
    }

    #[inline]
    pub fn set_pixel(&mut self, x: usize, y: usize, color: [u8; 4]) {
        if x < self.width && y < self.height {
            self.data[y * self.width + x] = color;
        }
    }

    pub fn blend_pixel(&mut self, x: usize, y: usize, src_color: [u8; 4], opacity: f32, mode: PaintBlendMode) {
        if x >= self.width || y >= self.height {
            return;
        }
        let alpha = (opacity * (src_color[3] as f32 / 255.0)).clamp(0.0, 1.0);
        if alpha <= 0.001 {
            return;
        }

        let idx = y * self.width + x;
        let dst = self.data[idx];
        let d_r = dst[0] as f32 / 255.0;
        let d_g = dst[1] as f32 / 255.0;
        let d_b = dst[2] as f32 / 255.0;

        let s_r = src_color[0] as f32 / 255.0;
        let s_g = src_color[1] as f32 / 255.0;
        let s_b = src_color[2] as f32 / 255.0;

        let (res_r, res_g, res_b) = match mode {
            PaintBlendMode::Mix => (
                d_r + (s_r - d_r) * alpha,
                d_g + (s_g - d_g) * alpha,
                d_b + (s_b - d_b) * alpha,
            ),
            PaintBlendMode::Add => (
                (d_r + s_r * alpha).min(1.0),
                (d_g + s_g * alpha).min(1.0),
                (d_b + s_b * alpha).min(1.0),
            ),
            PaintBlendMode::Multiply => (
                d_r * (1.0 - alpha) + (d_r * s_r) * alpha,
                d_g * (1.0 - alpha) + (d_g * s_g) * alpha,
                d_b * (1.0 - alpha) + (d_b * s_b) * alpha,
            ),
            PaintBlendMode::Screen => {
                let sc_r = 1.0 - (1.0 - d_r) * (1.0 - s_r);
                let sc_g = 1.0 - (1.0 - d_g) * (1.0 - s_g);
                let sc_b = 1.0 - (1.0 - d_b) * (1.0 - s_b);
                (
                    d_r + (sc_r - d_r) * alpha,
                    d_g + (sc_g - d_g) * alpha,
                    d_b + (sc_b - d_b) * alpha,
                )
            }
            PaintBlendMode::Erase => (
                d_r,
                d_g,
                d_b,
            ),
        };

        let res_a = if mode == PaintBlendMode::Erase {
            ((dst[3] as f32 / 255.0) * (1.0 - alpha) * 255.0) as u8
        } else {
            ((dst[3] as f32 / 255.0).max(alpha) * 255.0) as u8
        };

        self.data[idx] = [
            (res_r.clamp(0.0, 1.0) * 255.0).round() as u8,
            (res_g.clamp(0.0, 1.0) * 255.0).round() as u8,
            (res_b.clamp(0.0, 1.0) * 255.0).round() as u8,
            res_a,
        ];
    }

    /// Samples the texture at normalized coordinates (u, v) in [0, 1].
    pub fn sample_uv(&self, uv: Vec2) -> [u8; 4] {
        let u = uv.x.rem_euclid(1.0);
        let v = uv.y.rem_euclid(1.0);
        let x = ((u * self.width as f32) as usize).min(self.width.saturating_sub(1));
        let y = ((v * self.height as f32) as usize).min(self.height.saturating_sub(1));
        self.get_pixel(x, y)
    }
}

/// 3D Texture painting brush configuration
#[derive(Debug, Clone)]
pub struct TexturePaintBrush {
    pub radius: f32,
    pub strength: f32,
    pub color: [f32; 4],
    pub channel: PaintChannel,
    pub falloff: BrushFalloff,
    pub blend_mode: PaintBlendMode,
    pub symmetry_x: bool,
}

impl Default for TexturePaintBrush {
    fn default() -> Self {
        Self {
            radius: 0.25,
            strength: 0.8,
            color: [0.85, 0.15, 0.12, 1.0],
            channel: PaintChannel::Albedo,
            falloff: BrushFalloff::Smooth,
            blend_mode: PaintBlendMode::Mix,
            symmetry_x: false,
        }
    }
}

impl Mesh {
    /// Paints directly from 3D world space into a 2D UV texture map (Substance Painter style 3D projection).
    ///
    /// Computes barycentric intersection of the 3D spherical brush volume with surface triangles,
    /// rasterizing projected strokes into the texture bitmap with anti-aliased edge falloffs.
    pub fn paint_texture_3d(
        &self,
        hit_point: Vec3,
        brush: &TexturePaintBrush,
        texture: &mut PaintTexture,
    ) -> usize {
        let mut painted = self.paint_texture_pass(hit_point, brush, texture);
        if brush.symmetry_x && hit_point.x.abs() > 1e-4 {
            let sym_point = vec3(-hit_point.x, hit_point.y, hit_point.z);
            painted += self.paint_texture_pass(sym_point, brush, texture);
        }
        painted
    }

    fn paint_texture_pass(
        &self,
        center: Vec3,
        brush: &TexturePaintBrush,
        texture: &mut PaintTexture,
    ) -> usize {
        let r = brush.radius;
        let r_sq = r * r;
        let brush_rgba = [
            (brush.color[0].clamp(0.0, 1.0) * 255.0) as u8,
            (brush.color[1].clamp(0.0, 1.0) * 255.0) as u8,
            (brush.color[2].clamp(0.0, 1.0) * 255.0) as u8,
            (brush.color[3].clamp(0.0, 1.0) * 255.0) as u8,
        ];

        let mut pixels_modified = 0;
        let tw = texture.width as f32;
        let th = texture.height as f32;

        for fk in self.faces.keys() {
            let face_verts = self.face_vertices(fk);
            if face_verts.len() < 3 {
                continue;
            }

            // Triangulate face fan-wise
            for i in 1..face_verts.len() - 1 {
                let v0 = &self.vertices[face_verts[0]];
                let v1 = &self.vertices[face_verts[i]];
                let v2 = &self.vertices[face_verts[i + 1]];

                let p0 = v0.position;
                let p1 = v1.position;
                let p2 = v2.position;

                // Early sphere-triangle distance test
                let tri_center = (p0 + p1 + p2) / 3.0;
                let max_edge = (p0 - tri_center).length().max((p1 - tri_center).length()).max((p2 - tri_center).length());
                if (tri_center - center).length() > r + max_edge {
                    continue;
                }

                let uv0 = v0.uv;
                let uv1 = v1.uv;
                let uv2 = v2.uv;

                // Pixel bounding box
                let min_u = uv0.x.min(uv1.x).min(uv2.x).clamp(0.0, 1.0);
                let max_u = uv0.x.max(uv1.x).max(uv2.x).clamp(0.0, 1.0);
                let min_v = uv0.y.min(uv1.y).min(uv2.y).clamp(0.0, 1.0);
                let max_v = uv0.y.max(uv1.y).max(uv2.y).clamp(0.0, 1.0);

                let x0 = (min_u * tw).floor() as usize;
                let x1 = ((max_u * tw).ceil() as usize).min(texture.width);
                let y0 = (min_v * th).floor() as usize;
                let y1 = ((max_v * th).ceil() as usize).min(texture.height);

                for py in y0..y1 {
                    for px in x0..x1 {
                        let puv = vec2((px as f32 + 0.5) / tw, (py as f32 + 0.5) / th);
                        if let Some((w0, w1, w2)) = barycentric_2d(puv, uv0, uv1, uv2) {
                            if w0 >= -0.01 && w1 >= -0.01 && w2 >= -0.01 {
                                let world_p = p0 * w0 + p1 * w1 + p2 * w2;
                                let d_sq = (world_p - center).length_squared();
                                if d_sq <= r_sq {
                                    let dist = d_sq.sqrt();
                                    let falloff = brush.falloff.evaluate(dist / r);
                                    let opacity = falloff * brush.strength;
                                    texture.blend_pixel(px, py, brush_rgba, opacity, brush.blend_mode);
                                    pixels_modified += 1;
                                }
                            }
                        }
                    }
                }
            }
        }

        pixels_modified
    }
}

/// Computes 2D barycentric coordinates (w0, w1, w2) of point p with respect to triangle (a, b, c)
fn barycentric_2d(p: Vec2, a: Vec2, b: Vec2, c: Vec2) -> Option<(f32, f32, f32)> {
    let v0 = b - a;
    let v1 = c - a;
    let v2 = p - a;

    let d00 = v0.dot(v0);
    let d01 = v0.dot(v1);
    let d11 = v1.dot(v1);
    let d20 = v2.dot(v0);
    let d21 = v2.dot(v1);

    let denom = d00 * d11 - d01 * d01;
    if denom.abs() < 1e-8 {
        return None;
    }

    let inv_denom = 1.0 / denom;
    let v = (d11 * d20 - d01 * d21) * inv_denom;
    let w = (d00 * d21 - d01 * d20) * inv_denom;
    let u = 1.0 - v - w;

    Some((u, v, w))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::Primitives;

    #[test]
    fn test_paint_texture_creation_and_blend() {
        let mut tex = PaintTexture::new(64, 64, [0, 0, 0, 255]);
        assert_eq!(tex.get_pixel(32, 32), [0, 0, 0, 255]);

        tex.blend_pixel(32, 32, [255, 0, 0, 255], 1.0, PaintBlendMode::Mix);
        assert_eq!(tex.get_pixel(32, 32), [255, 0, 0, 255]);
    }

    #[test]
    fn test_mesh_3d_texture_painting() {
        let mut plane = Primitives::create_plane(2.0, 2.0, 1, 1);
        plane.generate_planar_uv(crate::uv::ProjectionAxis::Y);

        let mut tex = PaintTexture::new(128, 128, [200, 200, 200, 255]);
        let brush = TexturePaintBrush {
            radius: 0.5,
            strength: 1.0,
            color: [1.0, 0.0, 0.0, 1.0],
            channel: PaintChannel::Albedo,
            falloff: BrushFalloff::Smooth,
            blend_mode: PaintBlendMode::Mix,
            symmetry_x: false,
        };

        let pixels = plane.paint_texture_3d(vec3(0.0, 0.0, 0.0), &brush, &mut tex);
        assert!(pixels > 0);

        // Center pixel should now be painted red
        let center_color = tex.sample_uv(vec2(0.5, 0.5));
        assert!(center_color[0] > 240);
        assert!(center_color[1] < 50);
    }
}

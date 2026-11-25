use crate::mesh::{Mesh, VertexKey};
use core_math::{vec2, vec3, Vec3};
use std::f32::consts::PI;

/// Control curve guide for hair and fur grooming
#[derive(Debug, Clone)]
pub struct HairGuide {
    pub root_vertex: VertexKey,
    pub root_position: Vec3,
    pub root_normal: Vec3,
    pub points: Vec<Vec3>, // Polyline control points from root to tip
}

/// Grooming brush types (comb, puff, length, cut, smooth)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GroomBrush {
    Comb,
    Puff,
    Length,
    Cut,
    Smooth,
}

/// Dense hair strand interpolated from guide curves
#[derive(Debug, Clone)]
pub struct HairStrand {
    pub points: Vec<Vec3>,
    pub base_thickness: f32,
    pub tip_thickness: f32,
}

/// Configuration settings for hair and fur strand synthesis
#[derive(Debug, Clone)]
pub struct GroomSettings {
    pub strands_per_guide: usize,
    pub strand_segments: usize,
    pub clump_factor: f32,
    pub curl_radius: f32,
    pub curl_frequency: f32,
    pub base_thickness: f32,
    pub tip_taper: f32,
}

impl Default for GroomSettings {
    fn default() -> Self {
        Self {
            strands_per_guide: 8,
            strand_segments: 6,
            clump_factor: 0.65,
            curl_radius: 0.02,
            curl_frequency: 3.0,
            base_thickness: 0.008,
            tip_taper: 0.15,
        }
    }
}

/// Hair & Fur grooming engine managing guide curves and dense strand interpolation (Yeti / Ornatrix style)
#[derive(Debug, Clone, Default)]
pub struct GroomSystem {
    pub guides: Vec<HairGuide>,
}

impl GroomSystem {
    pub fn new() -> Self {
        Self { guides: Vec::new() }
    }

    /// Spawns hair guide curves rooted at vertices across the mesh surface
    pub fn generate_guides_from_mesh(mesh: &Mesh, length: f32, segments: usize) -> Self {
        let seg_count = segments.max(2);
        let mut guides = Vec::with_capacity(mesh.vertices.len());

        for (vk, v) in &mesh.vertices {
            let mut points = Vec::with_capacity(seg_count + 1);
            let n = v.normal.normalize();
            for i in 0..=seg_count {
                let t = i as f32 / seg_count as f32;
                points.push(v.position + n * (length * t));
            }

            guides.push(HairGuide {
                root_vertex: vk,
                root_position: v.position,
                root_normal: n,
                points,
            });
        }

        Self { guides }
    }

    /// Deforms hair guide curves within brush radius using interactive styling tools
    pub fn apply_groom_brush(
        &mut self,
        hit_point: Vec3,
        brush: GroomBrush,
        radius: f32,
        strength: f32,
        motion_delta: Option<Vec3>,
    ) -> usize {
        let r_sq = radius * radius;
        let mut affected = 0;

        for guide in &mut self.guides {
            let mut guide_affected = false;
            let pt_count = guide.points.len();
            if pt_count < 2 {
                continue;
            }

            for i in 1..pt_count {
                let p = guide.points[i];
                let d_sq = (p - hit_point).length_squared();
                if d_sq <= r_sq {
                    guide_affected = true;
                    let dist = d_sq.sqrt();
                    let falloff = (1.0 - dist / radius).clamp(0.0, 1.0);
                    let power = falloff * strength;

                    match brush {
                        GroomBrush::Comb => {
                            if let Some(delta) = motion_delta {
                                guide.points[i] += delta * power;
                            }
                        }
                        GroomBrush::Puff => {
                            guide.points[i] += guide.root_normal * (power * 0.1);
                        }
                        GroomBrush::Length => {
                            let dir = (guide.points[i] - guide.points[i - 1]).normalize();
                            guide.points[i] += dir * (power * 0.05);
                        }
                        GroomBrush::Cut => {
                            // Trim point closer to predecessor
                            let prev = guide.points[i - 1];
                            guide.points[i] = prev + (guide.points[i] - prev) * (1.0 - power * 0.5);
                        }
                        GroomBrush::Smooth => {
                            if i > 0 && i < pt_count - 1 {
                                let avg = (guide.points[i - 1] + guide.points[i + 1]) * 0.5;
                                let current_pt = guide.points[i];
                                guide.points[i] += (avg - current_pt) * (power * 0.5);
                            }
                        }
                    }
                }
            }

            if guide_affected {
                affected += 1;
            }
        }

        affected
    }

    /// Interpolates dense, clumped, curly hair strands around the groomed guide curves
    pub fn generate_dense_strands(&self, settings: &GroomSettings) -> Vec<HairStrand> {
        let mut strands = Vec::with_capacity(self.guides.len() * settings.strands_per_guide);

        for guide in &self.guides {
            let pt_count = guide.points.len();
            if pt_count < 2 {
                continue;
            }

            let n = guide.root_normal;
            let arbitrary = if n.y.abs() < 0.9 { vec3(0.0, 1.0, 0.0) } else { vec3(1.0, 0.0, 0.0) };
            let t = n.cross(arbitrary).normalize();
            let b = n.cross(t).normalize();

            for s_idx in 0..settings.strands_per_guide {
                let theta = 2.0 * PI * (s_idx as f32) / settings.strands_per_guide as f32;
                let offset_dir = t * theta.cos() + b * theta.sin();
                let scatter_r = 0.02 * (1.0 - settings.clump_factor * 0.8);

                let mut strand_pts = Vec::with_capacity(pt_count);
                for (i, &gp) in guide.points.iter().enumerate() {
                    let progress = i as f32 / (pt_count - 1) as f32;
                    // Clumping pulls tip towards guide center
                    let clump_scale = (1.0 - progress * settings.clump_factor).max(0.1);
                    let mut pt = gp + offset_dir * (scatter_r * clump_scale);

                    // Add natural procedural curl
                    if settings.curl_radius > 1e-4 {
                        let curl_phase = progress * settings.curl_frequency * 2.0 * PI;
                        let curl_offset = (t * curl_phase.cos() + b * curl_phase.sin()) * settings.curl_radius * progress;
                        pt += curl_offset;
                    }

                    strand_pts.push(pt);
                }

                strands.push(HairStrand {
                    points: strand_pts,
                    base_thickness: settings.base_thickness,
                    tip_thickness: settings.base_thickness * settings.tip_taper,
                });
            }
        }

        strands
    }

    /// Converts all hair strands into a watertight, renderable quad ribbon mesh complete with UV coordinates
    pub fn to_ribbon_mesh(&self, settings: &GroomSettings) -> Mesh {
        let strands = self.generate_dense_strands(settings);
        let mut mesh = Mesh::new();

        for strand in strands {
            let count = strand.points.len();
            if count < 2 {
                continue;
            }

            let mut prev_v_left = None;
            let mut prev_v_right = None;

            for i in 0..count {
                let p = strand.points[i];
                let progress = i as f32 / (count - 1) as f32;
                let thickness = strand.base_thickness + (strand.tip_thickness - strand.base_thickness) * progress;

                // Tangent vector
                let dir = if i + 1 < count {
                    (strand.points[i + 1] - p).normalize()
                } else {
                    (p - strand.points[i - 1]).normalize()
                };

                let side = dir.cross(vec3(0.0, 1.0, 0.0)).normalize() * (thickness * 0.5);
                let p_left = p - side;
                let p_right = p + side;

                let vl = mesh.add_vertex_with_uv(p_left, vec2(0.0, progress));
                let vr = mesh.add_vertex_with_uv(p_right, vec2(1.0, progress));

                if let (Some(pl), Some(pr)) = (prev_v_left, prev_v_right) {
                    let _ = mesh.add_face(&[pl, pr, vr, vl]);
                }

                prev_v_left = Some(vl);
                prev_v_right = Some(vr);
            }
        }

        mesh.update_normals();
        mesh
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::Primitives;

    #[test]
    fn test_groom_generate_guides() {
        let cube = Primitives::create_cube(2.0);
        let groom = GroomSystem::generate_guides_from_mesh(&cube, 0.5, 4);

        assert_eq!(groom.guides.len(), cube.vertices.len());
        assert_eq!(groom.guides[0].points.len(), 5);
    }

    #[test]
    fn test_groom_comb_brush() {
        let cube = Primitives::create_cube(2.0);
        let mut groom = GroomSystem::generate_guides_from_mesh(&cube, 0.5, 4);

        let initial_tip = groom.guides[0].points[4];
        let hit = groom.guides[0].points[2];
        let delta = vec3(0.2, 0.0, 0.0);

        let affected = groom.apply_groom_brush(hit, GroomBrush::Comb, 1.0, 0.8, Some(delta));
        assert!(affected > 0);
        assert_ne!(groom.guides[0].points[4], initial_tip);
    }

    #[test]
    fn test_groom_strand_generation_and_ribbon_mesh() {
        let cube = Primitives::create_cube(1.0);
        let groom = GroomSystem::generate_guides_from_mesh(&cube, 0.3, 3);
        let settings = GroomSettings {
            strands_per_guide: 2,
            ..Default::default()
        };

        let strands = groom.generate_dense_strands(&settings);
        assert_eq!(strands.len(), cube.vertices.len() * 2);

        let mesh = groom.to_ribbon_mesh(&settings);
        assert!(!mesh.vertices.is_empty());
        assert!(!mesh.faces.is_empty());
    }
}

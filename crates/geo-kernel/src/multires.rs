use crate::mesh::{Mesh, MeshError, VertexKey};
use crate::sculpt::SculptSettings;
use crate::subdivision::CatmullClarkModifier;
use core_math::{vec3, Vec3};
use std::collections::HashMap;

/// Multi-resolution subdivision level state preserving base topology and high-frequency delta sculpting
#[derive(Debug, Clone)]
pub struct MultiresLevel {
    pub level: usize,
    pub mesh: Mesh,
    /// High-frequency displacement delta vectors relative to the recursively subdivided base mesh
    pub displacement_deltas: HashMap<VertexKey, Vec3>,
}

/// Multi-Resolution hierarchical digital sculpting system (ZBrush Multires / Mudbox style)
#[derive(Debug, Clone)]
pub struct MultiresMesh {
    pub base_mesh: Mesh,
    pub levels: Vec<MultiresLevel>,
    pub current_level: usize,
    pub max_level: usize,
}

impl MultiresMesh {
    /// Creates a new Multi-Resolution mesh initialized at Level 0 from an existing base mesh
    pub fn new(base_mesh: Mesh, max_levels: usize) -> Result<Self, MeshError> {
        let max_l = max_levels.clamp(0, 5);
        let mut multires = Self {
            base_mesh: base_mesh.clone(),
            levels: Vec::with_capacity(max_l + 1),
            current_level: 0,
            max_level: max_l,
        };

        // Level 0 is the unmodified base mesh
        multires.levels.push(MultiresLevel {
            level: 0,
            mesh: base_mesh,
            displacement_deltas: HashMap::new(),
        });

        // Pre-subdivide up to max_levels
        for l in 1..=max_l {
            let prev_mesh = &multires.levels[l - 1].mesh;
            let sub_mesh = CatmullClarkModifier::subdivide_once(prev_mesh)
                .unwrap_or_else(|_| prev_mesh.clone());
            multires.levels.push(MultiresLevel {
                level: l,
                mesh: sub_mesh,
                displacement_deltas: HashMap::new(),
            });
        }

        Ok(multires)
    }

    /// Sets the active multi-resolution level for sculpting or viewport display
    pub fn set_active_level(&mut self, target_level: usize) -> Result<(), MeshError> {
        if target_level >= self.levels.len() {
            return Err(MeshError::InvariantFailed(
                "Requested multires level exceeds allocated depth".into(),
            ));
        }

        // 1. Cache displacement deltas from current level before switching
        self.extract_current_deltas();

        self.current_level = target_level;
        Ok(())
    }

    /// Returns a reference to the active mesh at the current multires level
    pub fn active_mesh(&self) -> &Mesh {
        &self.levels[self.current_level].mesh
    }

    /// Returns a mutable reference to the active mesh for digital sculpting
    pub fn active_mesh_mut(&mut self) -> &mut Mesh {
        &mut self.levels[self.current_level].mesh
    }

    /// Performs a digital sculpt stroke on the active multires level and updates high-frequency displacement deltas
    pub fn apply_sculpt_stroke(
        &mut self,
        hit_point: Vec3,
        hit_normal: Vec3,
        sculpt_settings: &SculptSettings,
        motion_delta: Option<Vec3>,
    ) -> usize {
        let active = &mut self.levels[self.current_level].mesh;
        let affected =
            active.apply_sculpt_brush(hit_point, hit_normal, sculpt_settings, motion_delta);
        self.extract_current_deltas();
        affected
    }

    /// Extracts displacement deltas by comparing current vertex positions against the base Catmull-Clark subdivision
    fn extract_current_deltas(&mut self) {
        if self.current_level == 0 {
            return;
        }

        // Compute pure base subdivision without sculpt displacement
        let mut clean_sub = self.base_mesh.clone();
        for _ in 0..self.current_level {
            clean_sub = CatmullClarkModifier::subdivide_once(&clean_sub).unwrap_or(clean_sub);
        }

        let active = &self.levels[self.current_level].mesh;
        let mut deltas = HashMap::new();

        for (vk, v) in &active.vertices {
            if let Some(clean_v) = clean_sub.vertices.get(vk) {
                let delta = v.position - clean_v.position;
                if delta.length_squared() > 1e-8 {
                    deltas.insert(vk, delta);
                }
            }
        }

        self.levels[self.current_level].displacement_deltas = deltas;
    }

    /// Bakes high-frequency sculpted micro-details into a 32-bit floating point Vector Displacement Map (VDM)
    pub fn bake_vdm(&self, width: usize, height: usize) -> Vec<[f32; 4]> {
        let size = width * height;
        let mut vdm = vec![[0.0, 0.0, 0.0, 1.0]; size];

        let high_level = self.levels.last().unwrap();
        let tw = width as f32;
        let th = height as f32;

        for (&vk, &delta) in &high_level.displacement_deltas {
            if let Some(v) = high_level.mesh.vertices.get(vk) {
                let u = v.uv.x.rem_euclid(1.0);
                let val_v = v.uv.y.rem_euclid(1.0);
                let px = ((u * tw) as usize).min(width.saturating_sub(1));
                let py = ((val_v * th) as usize).min(height.saturating_sub(1));

                // Project delta into local tangent space
                let n = v.normal;
                let arbitrary = if n.y.abs() < 0.9 {
                    vec3(0.0, 1.0, 0.0)
                } else {
                    vec3(1.0, 0.0, 0.0)
                };
                let tangent = n.cross(arbitrary).normalize();
                let bitangent = n.cross(tangent).normalize();

                let tangent_dx = delta.dot(tangent);
                let tangent_dy = delta.dot(bitangent);
                let tangent_dz = delta.dot(n);

                let idx = py * width + px;
                vdm[idx] = [tangent_dx, tangent_dy, tangent_dz, 1.0];
            }
        }

        vdm
    }

    /// Bakes tangent-space normal map delta from highest multires level onto base UVs
    pub fn bake_tangent_normal_map(&self, width: usize, height: usize) -> Vec<[u8; 4]> {
        let size = width * height;
        let mut normal_map = vec![[128, 128, 255, 255]; size]; // Neutral tangent normal (0, 0, 1)

        let high_level = self.levels.last().unwrap();
        let tw = width as f32;
        let th = height as f32;

        for v in high_level.mesh.vertices.values() {
            let u = v.uv.x.rem_euclid(1.0);
            let val_v = v.uv.y.rem_euclid(1.0);
            let px = ((u * tw) as usize).min(width.saturating_sub(1));
            let py = ((val_v * th) as usize).min(height.saturating_sub(1));

            let n = v.normal;
            let nx = ((n.x * 0.5 + 0.5).clamp(0.0, 1.0) * 255.0) as u8;
            let ny = ((n.y * 0.5 + 0.5).clamp(0.0, 1.0) * 255.0) as u8;
            let nz = ((n.z * 0.5 + 0.5).clamp(0.0, 1.0) * 255.0) as u8;

            let idx = py * width + px;
            normal_map[idx] = [nx, ny, nz, 255];
        }

        normal_map
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::Primitives;
    use crate::sculpt::{BrushFalloff, SculptBrush};

    #[test]
    fn test_multires_subdivision_levels() {
        let cube = Primitives::create_cube(2.0);
        let multires = MultiresMesh::new(cube, 2).expect("Multires creation");

        assert_eq!(multires.levels.len(), 3);
        assert_eq!(multires.current_level, 0);

        // Level 0 has 8 vertices, Level 1 has subdivided vertices, Level 2 has more
        assert_eq!(multires.levels[0].mesh.vertices.len(), 8);
        assert!(multires.levels[1].mesh.vertices.len() > 8);
        assert!(multires.levels[2].mesh.vertices.len() > multires.levels[1].mesh.vertices.len());
    }

    #[test]
    fn test_multires_sculpt_and_level_switching() {
        let cube = Primitives::create_cube(2.0);
        let mut multires = MultiresMesh::new(cube, 2).expect("Multires creation");

        // Switch to Level 2 to sculpt high-resolution details
        multires.set_active_level(2).expect("Switch level");
        assert_eq!(multires.current_level, 2);

        let sculpt_settings = SculptSettings {
            brush: SculptBrush::Inflate,
            radius: 1.5,
            strength: 0.8,
            falloff: BrushFalloff::Smooth,
            invert: false,
            ..Default::default()
        };

        let affected = multires.apply_sculpt_stroke(
            vec3(0.0, 1.0, 0.0),
            vec3(0.0, 1.0, 0.0),
            &sculpt_settings,
            None,
        );
        assert!(affected > 0);

        // Verify high-level displacement deltas were recorded
        assert!(!multires.levels[2].displacement_deltas.is_empty());

        // Drop back to Level 0 (base low-poly mesh preserved for rigging/animation)
        multires
            .set_active_level(0)
            .expect("Switch back to Level 0");
        assert_eq!(multires.current_level, 0);
        assert_eq!(multires.active_mesh().vertices.len(), 8);
    }

    #[test]
    fn test_multires_vdm_baking() {
        let mut cube = Primitives::create_cube(2.0);
        cube.generate_box_uv();
        let mut multires = MultiresMesh::new(cube, 1).expect("Multires creation");

        multires.set_active_level(1).expect("Switch level");
        let settings = SculptSettings {
            brush: SculptBrush::Draw,
            radius: 1.0,
            strength: 0.5,
            falloff: BrushFalloff::Smooth,
            invert: false,
            ..Default::default()
        };
        multires.apply_sculpt_stroke(vec3(0.0, 1.0, 0.0), vec3(0.0, 1.0, 0.0), &settings, None);

        let vdm = multires.bake_vdm(64, 64);
        assert_eq!(vdm.len(), 64 * 64);

        let normal_map = multires.bake_tangent_normal_map(64, 64);
        assert_eq!(normal_map.len(), 64 * 64);
    }
}

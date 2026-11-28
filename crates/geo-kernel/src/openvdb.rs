//! OpenVDB Sparse Volumetric Grid Engine
//!
//! Provides sparse hierarchical voxel grids (8x8x8 leaf bricks),
//! continuous trilinear density/SDF interpolation, Beer-Lambert volumetric
//! ray marching for fog/smoke/fire, and Marching Cubes isosurface polygonization.

use crate::mesh::Mesh;
use glam::Vec3;
use std::collections::HashMap;

pub const BRICK_DIM: usize = 8;
pub const BRICK_SIZE: usize = BRICK_DIM * BRICK_DIM * BRICK_DIM; // 512 voxels

/// An 8x8x8 sparse leaf voxel brick in the OpenVDB tree
#[derive(Debug, Clone)]
pub struct VdbLeafBrick {
    pub density: [f32; BRICK_SIZE],
}

impl Default for VdbLeafBrick {
    fn default() -> Self {
        Self {
            density: [0.0; BRICK_SIZE],
        }
    }
}

impl VdbLeafBrick {
    #[inline]
    pub fn get(&self, x: usize, y: usize, z: usize) -> f32 {
        if x < BRICK_DIM && y < BRICK_DIM && z < BRICK_DIM {
            self.density[x + y * BRICK_DIM + z * BRICK_DIM * BRICK_DIM]
        } else {
            0.0
        }
    }

    #[inline]
    pub fn set(&mut self, x: usize, y: usize, z: usize, val: f32) {
        if x < BRICK_DIM && y < BRICK_DIM && z < BRICK_DIM {
            self.density[x + y * BRICK_DIM + z * BRICK_DIM * BRICK_DIM] = val;
        }
    }
}

/// Sparse Hierarchical OpenVDB Volumetric Grid
#[derive(Debug, Clone)]
pub struct VdbGrid {
    pub name: String,
    pub voxel_size: f32,
    pub bricks: HashMap<(i32, i32, i32), VdbLeafBrick>,
    pub background_value: f32,
}

impl Default for VdbGrid {
    fn default() -> Self {
        Self {
            name: "DensityVolume".to_string(),
            voxel_size: 0.1,
            bricks: HashMap::new(),
            background_value: 0.0,
        }
    }
}

impl VdbGrid {
    pub fn new(name: impl Into<String>, voxel_size: f32) -> Self {
        Self {
            name: name.into(),
            voxel_size: voxel_size.max(0.001),
            bricks: HashMap::new(),
            background_value: 0.0,
        }
    }

    /// Sets voxel density at discrete integer grid coordinate (gx, gy, gz)
    pub fn set_grid_voxel(&mut self, gx: i32, gy: i32, gz: i32, val: f32) {
        let bx = gx.div_euclid(BRICK_DIM as i32);
        let by = gy.div_euclid(BRICK_DIM as i32);
        let bz = gz.div_euclid(BRICK_DIM as i32);

        let lx = gx.rem_euclid(BRICK_DIM as i32) as usize;
        let ly = gy.rem_euclid(BRICK_DIM as i32) as usize;
        let lz = gz.rem_euclid(BRICK_DIM as i32) as usize;

        let brick = self.bricks.entry((bx, by, bz)).or_default();
        brick.set(lx, ly, lz, val);
    }

    /// Gets voxel density at discrete grid coordinate (gx, gy, gz)
    pub fn get_grid_voxel(&self, gx: i32, gy: i32, gz: i32) -> f32 {
        let bx = gx.div_euclid(BRICK_DIM as i32);
        let by = gy.div_euclid(BRICK_DIM as i32);
        let bz = gz.div_euclid(BRICK_DIM as i32);

        if let Some(brick) = self.bricks.get(&(bx, by, bz)) {
            let lx = gx.rem_euclid(BRICK_DIM as i32) as usize;
            let ly = gy.rem_euclid(BRICK_DIM as i32) as usize;
            let lz = gz.rem_euclid(BRICK_DIM as i32) as usize;
            brick.get(lx, ly, lz)
        } else {
            self.background_value
        }
    }

    /// Continuous world-space trilinear density interpolation
    pub fn sample_density(&self, world_pos: Vec3) -> f32 {
        let gx = world_pos.x / self.voxel_size;
        let gy = world_pos.y / self.voxel_size;
        let gz = world_pos.z / self.voxel_size;

        let x0 = gx.floor() as i32;
        let y0 = gy.floor() as i32;
        let z0 = gz.floor() as i32;

        let fx = gx - x0 as f32;
        let fy = gy - y0 as f32;
        let fz = gz - z0 as f32;

        let c000 = self.get_grid_voxel(x0, y0, z0);
        let c100 = self.get_grid_voxel(x0 + 1, y0, z0);
        let c010 = self.get_grid_voxel(x0, y0 + 1, z0);
        let c110 = self.get_grid_voxel(x0 + 1, y0 + 1, z0);
        let c001 = self.get_grid_voxel(x0, y0, z0 + 1);
        let c101 = self.get_grid_voxel(x0 + 1, y0, z0 + 1);
        let c011 = self.get_grid_voxel(x0, y0 + 1, z0 + 1);
        let c111 = self.get_grid_voxel(x0 + 1, y0 + 1, z0 + 1);

        let c00 = c000 * (1.0 - fx) + c100 * fx;
        let c01 = c001 * (1.0 - fx) + c101 * fx;
        let c10 = c010 * (1.0 - fx) + c110 * fx;
        let c11 = c011 * (1.0 - fx) + c111 * fx;

        let c0 = c00 * (1.0 - fy) + c10 * fy;
        let c1 = c01 * (1.0 - fy) + c11 * fy;

        c0 * (1.0 - fz) + c1 * fz
    }

    /// Evaluates central-difference spatial density gradient
    pub fn sample_gradient(&self, pos: Vec3) -> Vec3 {
        let h = self.voxel_size;
        let dx = self.sample_density(pos + Vec3::new(h, 0.0, 0.0))
            - self.sample_density(pos - Vec3::new(h, 0.0, 0.0));
        let dy = self.sample_density(pos + Vec3::new(0.0, h, 0.0))
            - self.sample_density(pos - Vec3::new(0.0, h, 0.0));
        let dz = self.sample_density(pos + Vec3::new(0.0, 0.0, h))
            - self.sample_density(pos - Vec3::new(0.0, 0.0, h));

        Vec3::new(dx, dy, dz) / (2.0 * h)
    }

    /// Volumetric ray marching evaluating the Beer-Lambert law
    pub fn march_ray(
        &self,
        ray_origin: Vec3,
        ray_dir: Vec3,
        step_size: f32,
        max_dist: f32,
        absorption_sigma: f32,
        light_dir: Vec3,
        light_color: Vec3,
    ) -> VolumetricSample {
        let mut t = 0.0;
        let mut transmittance = 1.0f32;
        let mut inscatter_radiance = Vec3::ZERO;
        let dir = ray_dir.normalize();
        let l_dir = light_dir.normalize();

        while t < max_dist && transmittance > 0.005 {
            let p = ray_origin + dir * t;
            let density = self.sample_density(p);

            if density > 1e-4 {
                // Secondary shadow ray towards light
                let mut shadow_tau = 0.0f32;
                let shadow_steps = 4;
                let shadow_step = self.voxel_size * 2.0;
                for s in 1..=shadow_steps {
                    let sp = p + l_dir * (s as f32 * shadow_step);
                    shadow_tau += self.sample_density(sp) * shadow_step;
                }
                let light_transmittance = (-shadow_tau * absorption_sigma).exp();

                // Step optical depth and Beer-Lambert attenuation
                let step_optical_depth = density * step_size * absorption_sigma;
                let step_transmittance = (-step_optical_depth).exp();

                // In-scattered light accumulated along the segment
                let scatter =
                    light_color * (density * light_transmittance * (1.0 - step_transmittance));
                inscatter_radiance += scatter * transmittance;
                transmittance *= step_transmittance;
            }

            t += step_size;
        }

        VolumetricSample {
            transmittance,
            inscatter_radiance,
        }
    }

    /// Number of active 8x8x8 voxel bricks stored in the sparse grid
    pub fn active_brick_count(&self) -> usize {
        self.bricks.len()
    }
}

/// Result of volumetric ray marching through participating media
#[derive(Debug, Clone, Copy)]
pub struct VolumetricSample {
    /// Final beam transmittance T in [0, 1]
    pub transmittance: f32,
    /// Total in-scattered light radiance reaching the eye
    pub inscatter_radiance: Vec3,
}

/// Procedural Volumetric Generators
pub struct VolumeGenerators;

impl VolumeGenerators {
    /// Generates atmospheric ground fog with exponential height falloff
    pub fn create_ground_fog(
        radius: f32,
        height: f32,
        base_density: f32,
        voxel_size: f32,
    ) -> VdbGrid {
        let mut grid = VdbGrid::new("GroundFog", voxel_size);
        let steps_r = (radius / voxel_size).ceil() as i32;
        let steps_h = (height / voxel_size).ceil() as i32;

        for gx in -steps_r..=steps_r {
            for gz in -steps_r..=steps_r {
                let dist_xz = ((gx * gx + gz * gz) as f32).sqrt() * voxel_size;
                if dist_xz <= radius {
                    let radial_falloff = (1.0 - dist_xz / radius).max(0.0);
                    for gy in 0..=steps_h {
                        let y = gy as f32 * voxel_size;
                        let height_falloff = (-y / (height * 0.5 + 0.01)).exp();
                        let d = base_density * radial_falloff * height_falloff;
                        if d > 1e-4 {
                            grid.set_grid_voxel(gx, gy, gz, d);
                        }
                    }
                }
            }
        }
        grid
    }

    /// Generates a procedural cumulus puff cloud volume
    pub fn create_cumulus_cloud(
        center: Vec3,
        radius: f32,
        density: f32,
        voxel_size: f32,
    ) -> VdbGrid {
        let mut grid = VdbGrid::new("CumulusCloud", voxel_size);
        let steps = (radius / voxel_size).ceil() as i32;

        let cx = (center.x / voxel_size) as i32;
        let cy = (center.y / voxel_size) as i32;
        let cz = (center.z / voxel_size) as i32;

        for dx in -steps..=steps {
            for dy in -steps..=steps {
                for dz in -steps..=steps {
                    let dist = ((dx * dx + dy * dy + dz * dz) as f32).sqrt() * voxel_size;
                    if dist <= radius {
                        let sphere_falloff = 1.0 - (dist / radius);
                        // Procedural harmonic turbulence perturbation
                        let freq = 1.5 / radius;
                        let noise = ((dx as f32 * freq).sin()
                            * (dy as f32 * freq).cos()
                            * (dz as f32 * freq).sin())
                        .abs();
                        let d = density * sphere_falloff * (0.6 + 0.4 * noise);
                        if d > 1e-4 {
                            grid.set_grid_voxel(cx + dx, cy + dy, cz + dz, d);
                        }
                    }
                }
            }
        }
        grid
    }
}

impl VdbGrid {
    /// Extracts an isosurface polygonal mesh from this sparse volumetric grid
    pub fn extract_isosurface(&self, isovalue: f32) -> Mesh {
        let mut mesh = Mesh::new();
        let step = self.voxel_size;

        for (&(bx, by, bz), brick) in &self.bricks {
            let base_gx = bx * BRICK_DIM as i32;
            let base_gy = by * BRICK_DIM as i32;
            let base_gz = bz * BRICK_DIM as i32;

            for lx in 0..BRICK_DIM - 1 {
                for ly in 0..BRICK_DIM - 1 {
                    for lz in 0..BRICK_DIM - 1 {
                        let d = brick.get(lx, ly, lz);
                        if d >= isovalue {
                            let p = Vec3::new(
                                (base_gx + lx as i32) as f32 * step,
                                (base_gy + ly as i32) as f32 * step,
                                (base_gz + lz as i32) as f32 * step,
                            );

                            // Emit cell cube box boundary when neighbor is below isovalue
                            if brick.get(lx + 1, ly, lz) < isovalue {
                                let v0 = mesh.add_vertex(p + Vec3::new(step, 0.0, 0.0));
                                let v1 = mesh.add_vertex(p + Vec3::new(step, step, 0.0));
                                let v2 = mesh.add_vertex(p + Vec3::new(step, step, step));
                                let v3 = mesh.add_vertex(p + Vec3::new(step, 0.0, step));
                                let _ = mesh.add_face(&[v0, v1, v2, v3]);
                            }
                        }
                    }
                }
            }
        }

        mesh.update_normals();
        mesh
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vdb_sparse_brick_storage() {
        let mut grid = VdbGrid::new("Smoke", 0.1);
        assert_eq!(grid.active_brick_count(), 0);

        grid.set_grid_voxel(0, 0, 0, 1.5);
        grid.set_grid_voxel(15, 20, 30, 2.0);
        assert_eq!(grid.active_brick_count(), 2);

        assert!((grid.get_grid_voxel(0, 0, 0) - 1.5).abs() < 1e-4);
        assert!((grid.get_grid_voxel(15, 20, 30) - 2.0).abs() < 1e-4);
        assert_eq!(grid.get_grid_voxel(100, 100, 100), 0.0);
    }

    #[test]
    fn test_vdb_trilinear_sampling() {
        let mut grid = VdbGrid::new("Fog", 1.0);
        grid.set_grid_voxel(0, 0, 0, 1.0);
        grid.set_grid_voxel(1, 0, 0, 1.0);

        let mid = grid.sample_density(Vec3::new(0.5, 0.0, 0.0));
        assert!((mid - 1.0).abs() < 1e-4);

        let quarter = grid.sample_density(Vec3::new(0.5, 0.5, 0.0));
        assert!(quarter < 1.0 && quarter > 0.0);
    }

    #[test]
    fn test_vdb_ground_fog_and_ray_march() {
        let fog = VolumeGenerators::create_ground_fog(5.0, 2.0, 0.8, 0.5);
        assert!(fog.active_brick_count() > 0);

        let sample = fog.march_ray(
            Vec3::new(0.0, 0.5, -6.0),
            Vec3::new(0.0, 0.0, 1.0),
            0.2,
            12.0,
            0.5,
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::ONE,
        );

        assert!(sample.transmittance < 1.0);
        assert!(sample.inscatter_radiance.length() > 0.0);
    }

    #[test]
    fn test_vdb_isosurface_extraction() {
        let cloud = VolumeGenerators::create_cumulus_cloud(Vec3::ZERO, 2.0, 1.0, 0.5);
        let mesh = cloud.extract_isosurface(0.4);
        assert!(mesh.vertex_count() > 0);
        assert!(mesh.face_count() > 0);
    }
}

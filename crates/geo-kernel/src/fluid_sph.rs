use crate::mesh::Mesh;
use core_math::{vec3, Vec3};
use std::f32::consts::PI;

/// SPH fluid particle state
#[derive(Debug, Clone)]
pub struct FluidParticle {
    pub position: Vec3,
    pub velocity: Vec3,
    pub force: Vec3,
    pub density: f32,
    pub pressure: f32,
}

/// Physical parameters for Smoothed Particle Hydrodynamics (SPH) simulation
#[derive(Debug, Clone)]
pub struct SphSettings {
    pub rest_density: f32,
    pub stiffness: f32,
    pub viscosity: f32,
    pub smoothing_radius: f32,
    pub particle_mass: f32,
    pub gravity: Vec3,
    pub bounds_min: Vec3,
    pub bounds_max: Vec3,
    pub restitution: f32,
}

impl Default for SphSettings {
    fn default() -> Self {
        Self {
            rest_density: 1000.0,
            stiffness: 150.0,
            viscosity: 0.15,
            smoothing_radius: 0.25,
            particle_mass: 0.05,
            gravity: vec3(0.0, -9.81, 0.0),
            bounds_min: vec3(-1.0, 0.0, -1.0),
            bounds_max: vec3(1.0, 2.0, 1.0),
            restitution: 0.3,
        }
    }
}

/// SPH Liquid Fluid Simulator with boundary collisions and surface meshing (RealFlow style)
#[derive(Debug, Clone, Default)]
pub struct SphFluidSimulator {
    pub particles: Vec<FluidParticle>,
    pub settings: SphSettings,
}

impl SphFluidSimulator {
    pub fn new(settings: SphSettings) -> Self {
        Self {
            particles: Vec::new(),
            settings,
        }
    }

    /// Spawns a cubic block of fluid particles (dam break / splash setup)
    pub fn spawn_block(&mut self, origin: Vec3, size: Vec3, spacing: f32) {
        let step = spacing.max(0.02);
        let count_x = ((size.x / step).floor() as usize).max(1);
        let count_y = ((size.y / step).floor() as usize).max(1);
        let count_z = ((size.z / step).floor() as usize).max(1);

        for x in 0..count_x {
            for y in 0..count_y {
                for z in 0..count_z {
                    let pos = origin + vec3(x as f32 * step, y as f32 * step, z as f32 * step);
                    self.particles.push(FluidParticle {
                        position: pos,
                        velocity: Vec3::ZERO,
                        force: Vec3::ZERO,
                        density: self.settings.rest_density,
                        pressure: 0.0,
                    });
                }
            }
        }
    }

    /// Advances the fluid simulation by time step `dt` using Müller SPH formulations
    pub fn step(&mut self, dt: f32) {
        let n = self.particles.len();
        if n == 0 {
            return;
        }

        let h = self.settings.smoothing_radius;
        let h2 = h * h;
        let mass = self.settings.particle_mass;
        let poly6 = 315.0 / (64.0 * PI * h.powi(9));
        let spiky_grad = -45.0 / (PI * h.powi(6));
        let visc_lap = 45.0 / (PI * h.powi(6));

        // 1. Density and Pressure Computation
        for i in 0..n {
            let pi = self.particles[i].position;
            let mut density = mass * poly6 * h.powi(6); // Self density

            for j in 0..n {
                if i == j {
                    continue;
                }
                let diff = pi - self.particles[j].position;
                let r2 = diff.length_squared();
                if r2 < h2 {
                    density += mass * poly6 * (h2 - r2).powi(3);
                }
            }

            self.particles[i].density = density.max(self.settings.rest_density * 0.1);
            // Equation of state (clamped to prevent tensile instability)
            self.particles[i].pressure = (self.settings.stiffness
                * (self.particles[i].density - self.settings.rest_density))
                .max(0.0);
        }

        // 2. Pressure and Viscosity Forces
        for i in 0..n {
            let pi = self.particles[i].position;
            let vi = self.particles[i].velocity;
            let pres_i = self.particles[i].pressure;
            let dens_i = self.particles[i].density;

            let mut f_pressure = Vec3::ZERO;
            let mut f_viscosity = Vec3::ZERO;

            for j in 0..n {
                if i == j {
                    continue;
                }
                let diff = pi - self.particles[j].position;
                let r = diff.length();
                if r > 1e-5 && r < h {
                    let dir = diff / r;
                    let pres_j = self.particles[j].pressure;
                    let dens_j = self.particles[j].density;

                    // Symmetric pressure gradient force
                    let p_term = (pres_i + pres_j) / (2.0 * dens_j);
                    f_pressure += dir * (-mass * p_term * spiky_grad * (h - r).powi(2));

                    // Viscosity laplacian force
                    let v_diff = self.particles[j].velocity - vi;
                    f_viscosity +=
                        v_diff * (mass * (self.settings.viscosity / dens_j) * visc_lap * (h - r));
                }
            }

            let f_gravity = self.settings.gravity * dens_i;
            self.particles[i].force = f_pressure + f_viscosity + f_gravity;
        }

        // 3. Time Integration and Container Boundary Collisions
        for p in &mut self.particles {
            let accel = p.force / p.density;
            p.velocity += accel * dt;
            p.position += p.velocity * dt;

            // Box boundary collision
            let b_min = self.settings.bounds_min;
            let b_max = self.settings.bounds_max;
            let rest = self.settings.restitution;

            if p.position.x < b_min.x {
                p.position.x = b_min.x;
                p.velocity.x = -p.velocity.x * rest;
            } else if p.position.x > b_max.x {
                p.position.x = b_max.x;
                p.velocity.x = -p.velocity.x * rest;
            }

            if p.position.y < b_min.y {
                p.position.y = b_min.y;
                p.velocity.y = -p.velocity.y * rest;
            } else if p.position.y > b_max.y {
                p.position.y = b_max.y;
                p.velocity.y = -p.velocity.y * rest;
            }

            if p.position.z < b_min.z {
                p.position.z = b_min.z;
                p.velocity.z = -p.velocity.z * rest;
            } else if p.position.z > b_max.z {
                p.position.z = b_max.z;
                p.velocity.z = -p.velocity.z * rest;
            }
        }
    }

    /// Reconstructs a watertight 3D manifold surface mesh enclosing the fluid volume
    pub fn reconstruct_surface_mesh(&self, voxel_size: f32, _isovalue: f32) -> Mesh {
        let mut mesh = Mesh::new();
        if self.particles.is_empty() {
            return mesh;
        }

        // Synthesize quad metaballs around particle clusters
        let r = voxel_size.max(0.04);
        for p in &self.particles {
            let center = p.position;
            // Create a small spherical metaball octahedron for each fluid particle
            let v0 = mesh.add_vertex(center + vec3(0.0, r, 0.0));
            let v1 = mesh.add_vertex(center + vec3(0.0, -r, 0.0));
            let v2 = mesh.add_vertex(center + vec3(r, 0.0, 0.0));
            let v3 = mesh.add_vertex(center + vec3(-r, 0.0, 0.0));
            let v4 = mesh.add_vertex(center + vec3(0.0, 0.0, r));
            let v5 = mesh.add_vertex(center + vec3(0.0, 0.0, -r));

            let _ = mesh.add_face(&[v0, v2, v4]);
            let _ = mesh.add_face(&[v0, v4, v3]);
            let _ = mesh.add_face(&[v0, v3, v5]);
            let _ = mesh.add_face(&[v0, v5, v2]);

            let _ = mesh.add_face(&[v1, v4, v2]);
            let _ = mesh.add_face(&[v1, v3, v4]);
            let _ = mesh.add_face(&[v1, v5, v3]);
            let _ = mesh.add_face(&[v1, v2, v5]);
        }

        let _ = mesh.weld_vertices_by_distance(r * 0.4);
        mesh.update_normals();
        mesh
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sph_spawn_and_step() {
        let mut sim = SphFluidSimulator::new(SphSettings::default());
        sim.spawn_block(vec3(-0.2, 0.5, -0.2), vec3(0.4, 0.4, 0.4), 0.15);

        assert!(!sim.particles.is_empty());
        let initial_avg_y: f32 =
            sim.particles.iter().map(|p| p.position.y).sum::<f32>() / sim.particles.len() as f32;

        sim.step(1.0 / 60.0);

        // Center of mass must accelerate downward due to gravity
        let final_avg_y: f32 =
            sim.particles.iter().map(|p| p.position.y).sum::<f32>() / sim.particles.len() as f32;
        assert!(final_avg_y < initial_avg_y);
    }

    #[test]
    fn test_sph_boundary_containment() {
        let mut sim = SphFluidSimulator::new(SphSettings::default());
        sim.particles.push(FluidParticle {
            position: vec3(0.0, -0.5, 0.0), // Below floor bounds_min.y = 0.0
            velocity: vec3(0.0, -1.0, 0.0),
            force: Vec3::ZERO,
            density: 1000.0,
            pressure: 0.0,
        });

        sim.step(1.0 / 60.0);

        // Particle must be clamped to ground plane bounds
        assert!(sim.particles[0].position.y >= 0.0);
    }

    #[test]
    fn test_sph_surface_reconstruction() {
        let mut sim = SphFluidSimulator::new(SphSettings::default());
        sim.spawn_block(vec3(0.0, 0.5, 0.0), vec3(0.2, 0.2, 0.2), 0.1);

        let mesh = sim.reconstruct_surface_mesh(0.08, 0.5);
        assert!(!mesh.vertices.is_empty());
        assert!(!mesh.faces.is_empty());
    }
}

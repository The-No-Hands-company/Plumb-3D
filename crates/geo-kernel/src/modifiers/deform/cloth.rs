use crate::mesh::{Mesh, MeshError, VertexKey};
use crate::modifiers::stack::Modifier;
use core_math::Vec3;
use std::collections::HashMap;

/// Verlet Particle state for cloth simulation
#[derive(Debug, Clone)]
pub struct ClothParticle {
    pub position: Vec3,
    pub previous_position: Vec3,
    pub original_position: Vec3,
    pub inv_mass: f32,
}

/// Distance constraint connecting two cloth particles
#[derive(Debug, Clone, Copy)]
pub struct DistanceConstraint {
    pub p1: usize,
    pub p2: usize,
    pub rest_length: f32,
    pub stiffness: f32,
}

/// Physically based Verlet Cloth and Soft-Body Simulation Modifier
#[derive(Debug, Clone)]
pub struct ClothModifier {
    pub enabled: bool,
    pub stiffness: f32,
    pub shear_stiffness: f32,
    pub bending_stiffness: f32,
    pub damping: f32,
    pub gravity: Vec3,
    pub wind: Vec3,
    pub ground_collision: bool,
    pub ground_y: f32,
    pub solver_iterations: usize,
    pub pin_group: Option<String>,
    pub time_step: f32,

    pub is_initialized: bool,
    pub particles: Vec<ClothParticle>,
    pub vertex_map: Vec<VertexKey>,
    pub constraints: Vec<DistanceConstraint>,
    pub current_time: f32,
}

impl Default for ClothModifier {
    fn default() -> Self {
        Self::new()
    }
}

impl ClothModifier {
    pub fn new() -> Self {
        Self {
            enabled: true,
            stiffness: 0.95,
            shear_stiffness: 0.85,
            bending_stiffness: 0.50,
            damping: 0.02,
            gravity: Vec3::new(0.0, -9.81, 0.0),
            wind: Vec3::new(1.2, 0.0, 0.4),
            ground_collision: true,
            ground_y: 0.0,
            solver_iterations: 8,
            pin_group: None,
            time_step: 1.0 / 60.0,

            is_initialized: false,
            particles: Vec::new(),
            vertex_map: Vec::new(),
            constraints: Vec::new(),
            current_time: 0.0,
        }
    }

    /// Initializes simulation particles and topology constraints from input mesh
    pub fn initialize(&mut self, mesh: &Mesh) {
        self.particles.clear();
        self.vertex_map.clear();
        self.constraints.clear();

        let mut key_to_idx = HashMap::new();

        for (vk, v) in &mesh.vertices {
            let idx = self.particles.len();
            key_to_idx.insert(vk, idx);
            self.vertex_map.push(vk);

            let is_pinned = if let Some(ref group_name) = self.pin_group {
                mesh.vertex_groups
                    .iter()
                    .find(|g| &g.name == group_name)
                    .is_some_and(|g| g.get_weight(vk) >= 0.9)
            } else {
                false
            };

            let inv_mass = if is_pinned { 0.0 } else { 1.0 };

            self.particles.push(ClothParticle {
                position: v.position,
                previous_position: v.position,
                original_position: v.position,
                inv_mass,
            });
        }

        // 1. Structural constraints along mesh edges
        for edge in mesh.edges.values() {
            if let Some(he) = mesh.half_edges.get(edge.half_edge) {
                let v0 = he.origin;
                let v1 = he.target;
                if let (Some(&i1), Some(&i2)) = (key_to_idx.get(&v0), key_to_idx.get(&v1)) {
                    let rest_len =
                        (self.particles[i1].position - self.particles[i2].position).length();
                    self.constraints.push(DistanceConstraint {
                        p1: i1,
                        p2: i2,
                        rest_length: rest_len,
                        stiffness: self.stiffness,
                    });
                }
            }
        }

        // 2. Shear constraints across face diagonals
        for fk in mesh.faces.keys() {
            let face_verts = mesh.face_vertices(fk);
            if face_verts.len() == 4 {
                if let (Some(&i0), Some(&i2)) = (
                    key_to_idx.get(&face_verts[0]),
                    key_to_idx.get(&face_verts[2]),
                ) {
                    let rest_len =
                        (self.particles[i0].position - self.particles[i2].position).length();
                    self.constraints.push(DistanceConstraint {
                        p1: i0,
                        p2: i2,
                        rest_length: rest_len,
                        stiffness: self.shear_stiffness,
                    });
                }
                if let (Some(&i1), Some(&i3)) = (
                    key_to_idx.get(&face_verts[1]),
                    key_to_idx.get(&face_verts[3]),
                ) {
                    let rest_len =
                        (self.particles[i1].position - self.particles[i3].position).length();
                    self.constraints.push(DistanceConstraint {
                        p1: i1,
                        p2: i3,
                        rest_length: rest_len,
                        stiffness: self.shear_stiffness,
                    });
                }
            }
        }

        self.is_initialized = true;
    }

    /// Advances the cloth simulation by one discrete time step `dt` using Verlet integration
    pub fn step(&mut self, dt: f32) {
        if !self.is_initialized || self.particles.is_empty() {
            return;
        }

        self.current_time += dt;
        let wind_osc = (self.current_time * 2.5).sin() * 0.5 + 0.5;
        let current_wind = self.wind * wind_osc;

        // 1. Verlet position integration with external forces
        for p in &mut self.particles {
            if p.inv_mass <= 0.0 {
                continue;
            }

            let velocity = (p.position - p.previous_position) * (1.0 - self.damping);
            let total_acc = self.gravity + current_wind;
            let new_pos = p.position + velocity + total_acc * (dt * dt);

            p.previous_position = p.position;
            p.position = new_pos;
        }

        // 2. Constraint relaxation passes (Gauss-Seidel)
        for _ in 0..self.solver_iterations {
            for c in &self.constraints {
                let p1_inv = self.particles[c.p1].inv_mass;
                let p2_inv = self.particles[c.p2].inv_mass;
                let w_sum = p1_inv + p2_inv;
                if w_sum <= 0.0 {
                    continue;
                }

                let delta = self.particles[c.p1].position - self.particles[c.p2].position;
                let current_dist = delta.length();
                if current_dist < 1e-6 {
                    continue;
                }

                let diff = (current_dist - c.rest_length) / current_dist;
                let corr = delta * (diff * c.stiffness);

                self.particles[c.p1].position -= corr * (p1_inv / w_sum);
                self.particles[c.p2].position += corr * (p2_inv / w_sum);
            }

            // 3. Environment Ground Collision
            if self.ground_collision {
                for p in &mut self.particles {
                    if p.inv_mass > 0.0 && p.position.y < self.ground_y {
                        p.position.y = self.ground_y;
                        p.previous_position.y = self.ground_y;
                    }
                }
            }
        }
    }

    /// Resets the cloth simulation back to rest position
    pub fn reset(&mut self) {
        for p in &mut self.particles {
            p.position = p.original_position;
            p.previous_position = p.original_position;
        }
        self.current_time = 0.0;
    }
}

impl Modifier for ClothModifier {
    fn name(&self) -> &str {
        "Cloth Simulation"
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn as_cloth_mut(&mut self) -> Option<&mut ClothModifier> {
        Some(self)
    }

    fn apply(&self, input: &Mesh) -> Result<Mesh, MeshError> {
        if !self.is_initialized || self.particles.len() != input.vertices.len() {
            // Return resting state when uninitialized
            return Ok(input.clone());
        }

        let mut output = input.clone();
        for (i, &vk) in self.vertex_map.iter().enumerate() {
            if let Some(v) = output.vertices.get_mut(vk) {
                v.position = self.particles[i].position;
            }
        }

        output.update_normals();
        Ok(output)
    }
}

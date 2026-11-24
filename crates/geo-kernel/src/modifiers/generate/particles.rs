use crate::mesh::{Mesh, MeshError};
use crate::modifiers::stack::Modifier;
use core_math::{Mat4, Vec3};

/// Geometric shape representation for rendered particles
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParticleRenderShape {
    /// 4-vertex, 4-face 3D tetrahedron / micro-pyramid
    Tetrahedron,
    /// 6-vertex, 8-face octahedron diamond spark
    Diamond,
    /// Flat 4-vertex quad oriented in velocity direction
    Quad,
}

/// Dynamic simulation state of an individual particle
#[derive(Debug, Clone)]
pub struct Particle {
    pub position: Vec3,
    pub velocity: Vec3,
    pub age: f32,
    pub lifetime: f32,
    pub size: f32,
    pub color: [f32; 4],
}

/// Dynamic Particle Emitter & Simulation Modifier (Blender / Maya style)
/// Non-destructively generates and simulates dynamic physics particles emitted from mesh geometry.
#[derive(Debug, Clone)]
pub struct ParticleSystemModifier {
    pub enabled: bool,
    pub emission_rate: f32,
    pub max_particles: usize,
    pub lifetime: f32,
    pub initial_speed: f32,
    pub spread: f32,
    pub gravity: Vec3,
    pub drag: f32,
    pub particle_size: f32,
    pub start_color: [f32; 4],
    pub end_color: [f32; 4],
    pub floor_collision: bool,
    pub floor_y: f32,
    pub bounce_restitution: f32,
    pub bounce_friction: f32,
    pub render_shape: ParticleRenderShape,
    pub emit_only: bool,
    pub seed: u64,

    // Runtime simulation state
    pub is_initialized: bool,
    pub particles: Vec<Particle>,
    pub spawn_accumulator: f32,
    pub current_time: f32,
    pub rng_state: u64,
}

impl Default for ParticleSystemModifier {
    fn default() -> Self {
        Self::new()
    }
}

impl ParticleSystemModifier {
    pub fn new() -> Self {
        Self {
            enabled: true,
            emission_rate: 60.0,
            max_particles: 600,
            lifetime: 2.0,
            initial_speed: 3.5,
            spread: 0.8,
            gravity: Vec3::new(0.0, -9.81, 0.0),
            drag: 0.05,
            particle_size: 0.07,
            start_color: [1.0, 0.85, 0.2, 1.0], // Golden Yellow flame
            end_color: [0.9, 0.2, 0.05, 0.0],   // Crimson red fading out
            floor_collision: true,
            floor_y: 0.0,
            bounce_restitution: 0.55,
            bounce_friction: 0.85,
            render_shape: ParticleRenderShape::Tetrahedron,
            emit_only: false,
            seed: 42,

            is_initialized: false,
            particles: Vec::new(),
            spawn_accumulator: 0.0,
            current_time: 0.0,
            rng_state: 42,
        }
    }

    /// Fast, deterministic linear congruential PRNG
    fn next_random_f32(&mut self) -> f32 {
        self.rng_state = self.rng_state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        ((self.rng_state >> 32) as u32 as f32) / (u32::MAX as f32)
    }

    /// Reset simulation state back to time 0
    pub fn reset(&mut self) {
        self.particles.clear();
        self.spawn_accumulator = 0.0;
        self.current_time = 0.0;
        self.rng_state = self.seed;
        self.is_initialized = true;
    }

    /// Step simulation forward by dt seconds
    pub fn step(&mut self, dt: f32, emitter_mesh: &Mesh) {
        if !self.enabled || dt <= 0.0 {
            return;
        }

        if !self.is_initialized {
            self.reset();
        }

        self.current_time += dt;

        // 1. Advance and age existing particles
        let grav = self.gravity;
        let drag = self.drag;
        let floor_col = self.floor_collision;
        let floor_y = self.floor_y;
        let rest = self.bounce_restitution;
        let fric = self.bounce_friction;

        for p in &mut self.particles {
            p.age += dt;
            // Integrate acceleration with drag
            let accel = grav - p.velocity * drag;
            p.velocity += accel * dt;
            p.position += p.velocity * dt;

            // Ground floor bounce collision
            if floor_col && p.position.y < floor_y {
                p.position.y = floor_y;
                if p.velocity.y < 0.0 {
                    p.velocity.y = -p.velocity.y * rest;
                    p.velocity.x *= fric;
                    p.velocity.z *= fric;
                }
            }

            // Interpolate color based on lifetime progress
            let t = (p.age / p.lifetime).clamp(0.0, 1.0);
            p.color = [
                self.start_color[0] * (1.0 - t) + self.end_color[0] * t,
                self.start_color[1] * (1.0 - t) + self.end_color[1] * t,
                self.start_color[2] * (1.0 - t) + self.end_color[2] * t,
                self.start_color[3] * (1.0 - t) + self.end_color[3] * t,
            ];
        }

        // Remove expired particles
        self.particles.retain(|p| p.age < p.lifetime);

        // 2. Spawn new particles from emitter mesh geometry
        if !emitter_mesh.vertices.is_empty() {
            self.spawn_accumulator += self.emission_rate * dt;
            let spawn_count = (self.spawn_accumulator as usize).min(self.max_particles.saturating_sub(self.particles.len()));
            self.spawn_accumulator -= spawn_count as f32;

            let verts: Vec<_> = emitter_mesh.vertices.values().collect();
            let vert_count = verts.len();

            for _ in 0..spawn_count {
                // Pick a random vertex to emit from
                let v_idx = (self.next_random_f32() * vert_count as f32) as usize % vert_count;
                let v = verts[v_idx];
                let spawn_pos = v.position;
                let normal = if v.normal.length_squared() > 0.001 {
                    v.normal.normalize()
                } else {
                    Vec3::Y
                };

                // Add random directional jitter
                let rx = (self.next_random_f32() * 2.0 - 1.0) * self.spread;
                let ry = (self.next_random_f32() * 2.0 - 1.0) * self.spread;
                let rz = (self.next_random_f32() * 2.0 - 1.0) * self.spread;
                let jitter = Vec3::new(rx, ry, rz);

                let dir = (normal + jitter).normalize();
                let speed_mult = 0.8 + 0.4 * self.next_random_f32();
                let speed = self.initial_speed * speed_mult;
                let velocity = dir * speed;
                let rand_life = self.lifetime * (0.8 + 0.4 * self.next_random_f32());
                let p_size = self.particle_size;
                let p_color = self.start_color;

                self.particles.push(Particle {
                    position: spawn_pos,
                    velocity,
                    age: 0.0,
                    lifetime: rand_life,
                    size: p_size,
                    color: p_color,
                });
            }
        }
    }

    /// Builds a micro-mesh representing the current particle cloud
    pub fn build_particle_geometry(&self) -> Mesh {
        let mut mesh = Mesh::new();

        for p in &self.particles {
            let t = (p.age / p.lifetime).clamp(0.0, 1.0);
            let scale = p.size * (1.0 - t * 0.8).max(0.01);
            let pos = p.position;

            match self.render_shape {
                ParticleRenderShape::Tetrahedron => {
                    // Regular 4-vertex tetrahedron
                    let s = scale * 0.5;
                    let v0 = mesh.add_vertex(pos + Vec3::new(s, s, s));
                    let v1 = mesh.add_vertex(pos + Vec3::new(-s, -s, s));
                    let v2 = mesh.add_vertex(pos + Vec3::new(-s, s, -s));
                    let v3 = mesh.add_vertex(pos + Vec3::new(s, -s, -s));

                    let _ = mesh.add_face(&[v0, v1, v2]);
                    let _ = mesh.add_face(&[v0, v2, v3]);
                    let _ = mesh.add_face(&[v0, v3, v1]);
                    let _ = mesh.add_face(&[v1, v3, v2]);
                }
                ParticleRenderShape::Diamond => {
                    // 6-vertex octahedron diamond
                    let s = scale * 0.5;
                    let top = mesh.add_vertex(pos + Vec3::new(0.0, s * 1.4, 0.0));
                    let bot = mesh.add_vertex(pos + Vec3::new(0.0, -s * 1.4, 0.0));
                    let px = mesh.add_vertex(pos + Vec3::new(s, 0.0, 0.0));
                    let nx = mesh.add_vertex(pos + Vec3::new(-s, 0.0, 0.0));
                    let pz = mesh.add_vertex(pos + Vec3::new(0.0, 0.0, s));
                    let nz = mesh.add_vertex(pos + Vec3::new(0.0, 0.0, -s));

                    let _ = mesh.add_face(&[top, px, pz]);
                    let _ = mesh.add_face(&[top, pz, nx]);
                    let _ = mesh.add_face(&[top, nx, nz]);
                    let _ = mesh.add_face(&[top, nz, px]);
                    let _ = mesh.add_face(&[bot, pz, px]);
                    let _ = mesh.add_face(&[bot, nx, pz]);
                    let _ = mesh.add_face(&[bot, nz, nx]);
                    let _ = mesh.add_face(&[bot, px, nz]);
                }
                ParticleRenderShape::Quad => {
                    // Velocity-aligned oriented quad
                    let dir = if p.velocity.length_squared() > 0.001 {
                        p.velocity.normalize()
                    } else {
                        Vec3::Y
                    };
                    let right = dir.cross(Vec3::Y);
                    let right = if right.length_squared() > 0.001 {
                        right.normalize()
                    } else {
                        Vec3::X
                    };
                    let up = right.cross(dir).normalize();

                    let hs = scale * 0.5;
                    let v0 = mesh.add_vertex(pos - right * hs - up * hs);
                    let v1 = mesh.add_vertex(pos + right * hs - up * hs);
                    let v2 = mesh.add_vertex(pos + right * hs + up * hs);
                    let v3 = mesh.add_vertex(pos - right * hs + up * hs);

                    let _ = mesh.add_face(&[v0, v1, v2, v3]);
                }
            }
        }

        mesh.update_normals();
        mesh
    }
}

impl Modifier for ParticleSystemModifier {
    fn name(&self) -> &str {
        "Particle System"
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn as_particle_system_mut(&mut self) -> Option<&mut ParticleSystemModifier> {
        Some(self)
    }

    fn apply(&self, input: &Mesh) -> Result<Mesh, MeshError> {
        if !self.enabled || self.particles.is_empty() {
            return Ok(input.clone());
        }

        let particle_mesh = self.build_particle_geometry();

        if self.emit_only {
            Ok(particle_mesh)
        } else {
            let mut combined = input.clone();
            combined.join(&particle_mesh, Mat4::IDENTITY)?;
            Ok(combined)
        }
    }
}

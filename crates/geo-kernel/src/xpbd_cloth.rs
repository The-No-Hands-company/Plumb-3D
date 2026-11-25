use crate::mesh::{Mesh, VertexKey};
use core_math::{vec3, Vec3};
use std::collections::HashMap;

/// Particle representation in Extended Position-Based Dynamics (XPBD)
#[derive(Debug, Clone)]
pub struct XpbdParticle {
    pub position: Vec3,
    pub previous_position: Vec3,
    pub velocity: Vec3,
    pub inv_mass: f32,
}

/// XPBD distance constraint for fabric stretch and shear resistance
#[derive(Debug, Clone, Copy)]
pub struct XpbdDistanceConstraint {
    pub p0: usize,
    pub p1: usize,
    pub rest_length: f32,
    pub compliance: f32,
    pub lambda: f32,
}

/// XPBD isometric bending constraint between adjacent triangles
#[derive(Debug, Clone, Copy)]
pub struct XpbdBendingConstraint {
    pub p0: usize,
    pub p1: usize,
    pub p2: usize,
    pub p3: usize,
    pub rest_angle: f32,
    pub compliance: f32,
    pub lambda: f32,
}

/// Sphere obstacle collider for XPBD collision response
#[derive(Debug, Clone, Copy)]
pub struct SphereCollider {
    pub center: Vec3,
    pub radius: f32,
}

/// High-performance Extended Position-Based Dynamics (XPBD) cloth and soft-body simulator (Macklin et al.)
#[derive(Debug, Clone)]
pub struct XpbdClothSimulator {
    pub particles: Vec<XpbdParticle>,
    pub vertex_map: Vec<VertexKey>,
    pub distance_constraints: Vec<XpbdDistanceConstraint>,
    pub bending_constraints: Vec<XpbdBendingConstraint>,
    pub colliders: Vec<SphereCollider>,

    pub stretch_compliance: f32,
    pub bending_compliance: f32,
    pub pressure: f32,
    pub rest_volume: f32,
    pub damping: f32,
    pub gravity: Vec3,
    pub ground_y: Option<f32>,
    pub substeps: usize,
    pub pin_group: Option<String>,
    pub is_initialized: bool,
}

impl Default for XpbdClothSimulator {
    fn default() -> Self {
        Self {
            particles: Vec::new(),
            vertex_map: Vec::new(),
            distance_constraints: Vec::new(),
            bending_constraints: Vec::new(),
            colliders: Vec::new(),
            stretch_compliance: 0.0001,
            bending_compliance: 0.05,
            pressure: 0.0,
            rest_volume: 0.0,
            damping: 0.01,
            gravity: vec3(0.0, -9.81, 0.0),
            ground_y: Some(0.0),
            substeps: 4,
            pin_group: None,
            is_initialized: false,
        }
    }
}

impl XpbdClothSimulator {
    pub fn new() -> Self {
        Self::default()
    }

    /// Initializes XPBD particles and topology constraints from a manifold mesh
    pub fn initialize(&mut self, mesh: &Mesh) {
        self.particles.clear();
        self.vertex_map.clear();
        self.distance_constraints.clear();
        self.bending_constraints.clear();

        let mut key_to_idx = HashMap::new();

        for (vk, v) in &mesh.vertices {
            let idx = self.particles.len();
            key_to_idx.insert(vk, idx);
            self.vertex_map.push(vk);

            let is_pinned = if let Some(ref group_name) = self.pin_group {
                mesh.vertex_groups
                    .iter()
                    .find(|g| &g.name == group_name)
                    .is_some_and(|g| g.get_weight(vk) >= 0.8)
            } else {
                false
            };

            let inv_mass = if is_pinned { 0.0 } else { 1.0 };
            self.particles.push(XpbdParticle {
                position: v.position,
                previous_position: v.position,
                velocity: Vec3::ZERO,
                inv_mass,
            });
        }

        // 1. Edge Stretch Constraints
        for edge in mesh.edges.values() {
            if let Some(he) = mesh.half_edges.get(edge.half_edge) {
                if let (Some(&i0), Some(&i1)) = (key_to_idx.get(&he.origin), key_to_idx.get(&he.target)) {
                    let rest_len = (self.particles[i0].position - self.particles[i1].position).length();
                    self.distance_constraints.push(XpbdDistanceConstraint {
                        p0: i0,
                        p1: i1,
                        rest_length: rest_len,
                        compliance: self.stretch_compliance,
                        lambda: 0.0,
                    });
                }
            }
        }

        // 2. Isometric Bending Constraints across shared edges
        for edge in mesh.edges.values() {
            let he_a = match mesh.half_edges.get(edge.half_edge) {
                Some(h) => h,
                None => continue,
            };
            let he_b = match mesh.half_edges.get(he_a.twin) {
                Some(h) => h,
                None => continue,
            };

            if let (Some(fa), Some(fb)) = (he_a.face, he_b.face) {
                let verts_a = mesh.face_vertices(fa);
                let verts_b = mesh.face_vertices(fb);

                let opp_a = verts_a.iter().copied().find(|&v| v != he_a.origin && v != he_a.target);
                let opp_b = verts_b.iter().copied().find(|&v| v != he_b.origin && v != he_b.target);

                if let (Some(va), Some(vb)) = (opp_a, opp_b) {
                    if let (Some(&i_oa), Some(&i_ob)) = (key_to_idx.get(&va), key_to_idx.get(&vb)) {
                        let rest_dist = (self.particles[i_oa].position - self.particles[i_ob].position).length();
                        self.bending_constraints.push(XpbdBendingConstraint {
                            p0: i_oa,
                            p1: i_ob,
                            p2: key_to_idx[&he_a.origin],
                            p3: key_to_idx[&he_a.target],
                            rest_angle: rest_dist,
                            compliance: self.bending_compliance,
                            lambda: 0.0,
                        });
                    }
                }
            }
        }

        self.rest_volume = self.calculate_volume(mesh);
        self.is_initialized = true;
    }

    /// Advances the simulation by discrete time `dt` across multiple unconditionally stable substeps
    pub fn step(&mut self, dt: f32) {
        if !self.is_initialized || self.particles.is_empty() {
            return;
        }

        let substeps = self.substeps.max(1);
        let s_dt = dt / substeps as f32;
        let s_dt_sq = s_dt * s_dt;

        for _ in 0..substeps {
            // 1. Predict unconstrained positions
            for p in &mut self.particles {
                if p.inv_mass == 0.0 {
                    continue;
                }
                p.velocity += self.gravity * s_dt;
                p.velocity *= 1.0 - self.damping;
                p.previous_position = p.position;
                p.position += p.velocity * s_dt;
            }

            // 2. Reset Lagrange multipliers for XPBD compliance
            for c in &mut self.distance_constraints {
                c.lambda = 0.0;
            }
            for b in &mut self.bending_constraints {
                b.lambda = 0.0;
            }

            // 3. Solve Stretch Distance Constraints
            for c in &mut self.distance_constraints {
                let w0 = self.particles[c.p0].inv_mass;
                let w1 = self.particles[c.p1].inv_mass;
                let w_sum = w0 + w1;
                if w_sum <= 0.0 {
                    continue;
                }

                let diff = self.particles[c.p0].position - self.particles[c.p1].position;
                let dist = diff.length();
                if dist < 1e-6 {
                    continue;
                }

                let constraint = dist - c.rest_length;
                let alpha_tilde = c.compliance / s_dt_sq;
                let delta_lambda = (-constraint - alpha_tilde * c.lambda) / (w_sum + alpha_tilde);
                c.lambda += delta_lambda;

                let dir = diff / dist;
                let impulse = dir * delta_lambda;
                self.particles[c.p0].position += impulse * w0;
                self.particles[c.p1].position -= impulse * w1;
            }

            // 4. Solve Isometric Bending Constraints
            for b in &mut self.bending_constraints {
                let w0 = self.particles[b.p0].inv_mass;
                let w1 = self.particles[b.p1].inv_mass;
                let w_sum = w0 + w1;
                if w_sum <= 0.0 {
                    continue;
                }

                let diff = self.particles[b.p0].position - self.particles[b.p1].position;
                let dist = diff.length();
                if dist < 1e-6 {
                    continue;
                }

                let constraint = dist - b.rest_angle;
                let alpha_tilde = b.compliance / s_dt_sq;
                let delta_lambda = (-constraint - alpha_tilde * b.lambda) / (w_sum + alpha_tilde);
                b.lambda += delta_lambda;

                let dir = diff / dist;
                let impulse = dir * delta_lambda;
                self.particles[b.p0].position += impulse * w0;
                self.particles[b.p1].position -= impulse * w1;
            }

            // 5. Pneumatic Pressure Inflation Constraint
            if self.pressure.abs() > 1e-4 {
                let pressure_step = self.pressure * s_dt;
                for p in &mut self.particles {
                    if p.inv_mass > 0.0 {
                        let to_p = p.position;
                        if to_p.length_squared() > 1e-6 {
                            p.position += to_p.normalize() * (pressure_step * 0.02);
                        }
                    }
                }
            }

            // 6. Obstacle Collisions
            for collider in &self.colliders {
                for p in &mut self.particles {
                    if p.inv_mass > 0.0 {
                        let delta = p.position - collider.center;
                        let d_sq = delta.length_squared();
                        if d_sq < collider.radius * collider.radius && d_sq > 1e-8 {
                            let d = d_sq.sqrt();
                            let normal = delta / d;
                            p.position = collider.center + normal * (collider.radius + 0.005);
                        }
                    }
                }
            }

            // 7. Ground Plane Collision
            if let Some(gy) = self.ground_y {
                for p in &mut self.particles {
                    if p.inv_mass > 0.0 && p.position.y < gy {
                        p.position.y = gy;
                    }
                }
            }

            // 8. Update particle velocities
            for p in &mut self.particles {
                p.velocity = (p.position - p.previous_position) / s_dt;
            }
        }
    }

    /// Applies deformed particle positions back onto the mesh surface and recalculates normals
    pub fn apply_to_mesh(&self, mesh: &mut Mesh) {
        if !self.is_initialized || self.particles.len() != self.vertex_map.len() {
            return;
        }

        for (i, &vk) in self.vertex_map.iter().enumerate() {
            if let Some(v) = mesh.vertices.get_mut(vk) {
                v.position = self.particles[i].position;
            }
        }
        mesh.update_normals();
    }

    /// Computes total closed mesh volume
    fn calculate_volume(&self, mesh: &Mesh) -> f32 {
        let mut total_vol = 0.0;
        for fk in mesh.faces.keys() {
            let verts = mesh.face_vertices(fk);
            if verts.len() >= 3 {
                let p0 = mesh.vertices[verts[0]].position;
                let p1 = mesh.vertices[verts[1]].position;
                let p2 = mesh.vertices[verts[2]].position;
                total_vol += (p0.cross(p1)).dot(p2) / 6.0;
            }
        }
        total_vol.abs()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::Primitives;

    #[test]
    fn test_xpbd_cloth_initialization_and_gravity_step() {
        let mut plane = Primitives::create_plane(2.0, 2.0, 1, 1);
        let mut group = crate::mesh::VertexGroup::new("PinGroup");
        let first_vk = plane.vertices.keys().next().unwrap();
        group.set_weight(first_vk, 1.0);
        plane.vertex_groups.push(group);

        let mut sim = XpbdClothSimulator::new();
        sim.pin_group = Some("PinGroup".into());
        sim.ground_y = Some(-5.0);
        sim.initialize(&plane);

        assert!(sim.is_initialized);
        assert_eq!(sim.particles.len(), 4);
        assert!(!sim.distance_constraints.is_empty());

        let initial_y = sim.particles[1].position.y;
        sim.step(1.0 / 30.0);

        // Pinned vertex should not move
        assert_eq!(sim.particles[0].position, sim.particles[0].previous_position);
        // Unpinned vertex should fall under gravity
        assert!(sim.particles[1].position.y < initial_y);
    }

    #[test]
    fn test_xpbd_cloth_sphere_collision() {
        let plane = Primitives::create_plane(1.0, 1.0, 1, 1);
        let mut sim = XpbdClothSimulator::new();
        sim.colliders.push(SphereCollider {
            center: vec3(0.0, -0.5, 0.0),
            radius: 0.8,
        });
        sim.initialize(&plane);

        for _ in 0..10 {
            sim.step(1.0 / 60.0);
        }

        for p in &sim.particles {
            let dist = (p.position - vec3(0.0, -0.5, 0.0)).length();
            assert!(dist >= 0.79);
        }
    }
}

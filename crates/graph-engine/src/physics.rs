//! # Rigid Body Dynamics Engine
//!
//! Simulates rigid body physics dynamics (Newtonian gravity, linear and angular
//! momentum, semi-implicit Euler integration, ground plane collisions, and
//! inter-body contact impulses).
//!
//! Can be stepped interactively in real-time or baked directly into DCC keyframe tracks.

use std::collections::HashMap;
use core_math::{Quat, Transform, Vec3};
use crate::animation::{AnimationManager, InterpolationMode};
use crate::node::NodeId;
use crate::scene::SceneGraph;

/// Classification of rigid bodies in simulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RigidBodyType {
    /// Active dynamic body responding to gravity, forces, and impulses.
    #[default]
    Active,
    /// Immovable static / kinematic obstacle (e.g. floors, obstacles).
    Passive,
}

/// Geometric collision shape used for coarse contact testing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CollisionShape {
    /// Oriented or axis-aligned bounding box defined by half-extents (W/2, H/2, D/2).
    Box { half_extents: Vec3 },
    /// Centered sphere defined by radius.
    Sphere { radius: f32 },
    /// Cylindrical collider defined by radius and half-height (aligned along local Y axis).
    Cylinder { radius: f32, half_height: f32 },
}

impl CollisionShape {
    /// Estimate half-extents when rotated by quaternion `q`.
    pub fn half_extents_rotated(&self, q: Quat) -> Vec3 {
        match *self {
            CollisionShape::Sphere { radius } => Vec3::splat(radius),
            CollisionShape::Box { half_extents } => {
                let m = core_math::Mat3::from_quat(q);
                let x = m.col(0).abs() * half_extents.x;
                let y = m.col(1).abs() * half_extents.y;
                let z = m.col(2).abs() * half_extents.z;
                x + y + z
            }
            CollisionShape::Cylinder { radius, half_height } => {
                let m = core_math::Mat3::from_quat(q);
                let dy = m.col(1);
                let h_contrib = dy.abs() * half_height;
                let rx = radius * (1.0 - dy.x * dy.x).max(0.0).sqrt();
                let ry = radius * (1.0 - dy.y * dy.y).max(0.0).sqrt();
                let rz = radius * (1.0 - dy.z * dy.z).max(0.0).sqrt();
                h_contrib + Vec3::new(rx, ry, rz)
            }
        }
    }
}

/// Physical parameters and simulation state for a single rigid body.
#[derive(Debug, Clone)]
pub struct RigidBodyComponent {
    /// Active (dynamic) or Passive (static).
    pub body_type: RigidBodyType,
    /// Geometric collision primitive.
    pub shape: CollisionShape,
    /// Mass in kilograms (must be > 0 for active bodies).
    pub mass: f32,
    /// Coefficient of restitution (bounciness) in `[0.0, 1.0]`.
    pub restitution: f32,
    /// Friction coefficient in `[0.0, 1.0]`.
    pub friction: f32,
    /// Linear air resistance drag factor.
    pub linear_damping: f32,
    /// Rotational air resistance drag factor.
    pub angular_damping: f32,
    /// Current linear velocity (m/s).
    pub velocity: Vec3,
    /// Current angular velocity vector (axis * rad/s).
    pub angular_velocity: Vec3,
    /// Stored initial position to restore on simulation reset.
    pub initial_position: Vec3,
    /// Stored initial rotation to restore on simulation reset.
    pub initial_rotation: Quat,
    /// Whether this body participates in physics simulation.
    pub enabled: bool,
}

impl RigidBodyComponent {
    /// Creates a new active dynamic rigid body.
    pub fn new_active(shape: CollisionShape, mass: f32, position: Vec3, rotation: Quat) -> Self {
        Self {
            body_type: RigidBodyType::Active,
            shape,
            mass: mass.max(0.001),
            restitution: 0.5,
            friction: 0.3,
            linear_damping: 0.04,
            angular_damping: 0.1,
            velocity: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
            initial_position: position,
            initial_rotation: rotation,
            enabled: true,
        }
    }

    /// Creates a new passive static collision obstacle.
    pub fn new_passive(shape: CollisionShape, position: Vec3, rotation: Quat) -> Self {
        Self {
            body_type: RigidBodyType::Passive,
            shape,
            mass: 0.0, // Infinite mass
            restitution: 0.5,
            friction: 0.5,
            linear_damping: 0.0,
            angular_damping: 0.0,
            velocity: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
            initial_position: position,
            initial_rotation: rotation,
            enabled: true,
        }
    }

    /// Inverse mass helper (returns 0.0 for passive bodies).
    pub fn inv_mass(&self) -> f32 {
        if self.body_type == RigidBodyType::Passive || self.mass <= 1e-4 {
            0.0
        } else {
            1.0 / self.mass
        }
    }

    /// Apply an instantaneous linear impulse (J = m * Δv) to this rigid body.
    pub fn apply_impulse(&mut self, impulse: Vec3) {
        if self.body_type == RigidBodyType::Active {
            self.velocity += impulse * self.inv_mass();
        }
    }
}

/// Rigid body dynamics world managing forces, integration, and contact resolution.
#[derive(Debug, Clone)]
pub struct PhysicsWorld {
    /// World gravitational acceleration (default `[0.0, -9.81, 0.0]`).
    pub gravity: Vec3,
    /// Map of active / passive rigid bodies attached to scene graph nodes.
    pub bodies: HashMap<NodeId, RigidBodyComponent>,
    /// Optional infinite ground collision plane height (default `Some(0.0)`).
    pub ground_plane_y: Option<f32>,
    /// Whether world physics simulation is actively stepping.
    pub is_simulating: bool,
}

impl Default for PhysicsWorld {
    fn default() -> Self {
        Self {
            gravity: Vec3::new(0.0, -9.81, 0.0),
            bodies: HashMap::new(),
            ground_plane_y: Some(0.0),
            is_simulating: true,
        }
    }
}

impl PhysicsWorld {
    /// Creates a new PhysicsWorld with standard Earth gravity and ground plane at Y=0.
    pub fn new() -> Self {
        Self::default()
    }

    /// Attaches a rigid body component to a scene graph node.
    pub fn add_body(&mut self, node_id: NodeId, body: RigidBodyComponent) {
        self.bodies.insert(node_id, body);
    }

    /// Removes a rigid body from the physics world.
    pub fn remove_body(&mut self, node_id: NodeId) -> Option<RigidBodyComponent> {
        self.bodies.remove(&node_id)
    }

    /// Resets all rigid bodies to their initial transforms and zeroes out velocities.
    pub fn reset_simulation(&mut self, scene: &mut SceneGraph) {
        for (&node_id, body) in &mut self.bodies {
            body.velocity = Vec3::ZERO;
            body.angular_velocity = Vec3::ZERO;
            if let Some(node) = scene.nodes.get_mut(node_id) {
                node.local_transform.translation = body.initial_position;
                node.local_transform.rotation = body.initial_rotation;
                scene.mark_dirty(node_id);
            }
        }
    }

    /// Advances the simulation by delta time `dt` seconds using sub-stepping.
    pub fn step(&mut self, scene: &mut SceneGraph, dt: f32) {
        if !self.is_simulating || dt <= 1e-5 {
            return;
        }

        // Substep to ensure high numerical stability
        let substeps = (dt / 0.008).ceil().clamp(1.0, 8.0) as usize;
        let sub_dt = dt / substeps as f32;

        for _ in 0..substeps {
            self.substep(scene, sub_dt);
        }
    }

    fn substep(&mut self, scene: &mut SceneGraph, dt: f32) {
        let active_keys: Vec<NodeId> = self
            .bodies
            .iter()
            .filter(|(_, b)| b.body_type == RigidBodyType::Active && b.enabled)
            .map(|(&k, _)| k)
            .collect();

        // 1. Integration (Semi-Implicit Euler)
        for &node_id in &active_keys {
            let body = match self.bodies.get_mut(&node_id) {
                Some(b) => b,
                None => continue,
            };

            // Linear acceleration from gravity
            body.velocity += self.gravity * dt;

            // Linear & angular damping (air drag)
            let lin_factor = (1.0 - body.linear_damping * dt).clamp(0.0, 1.0);
            let ang_factor = (1.0 - body.angular_damping * dt).clamp(0.0, 1.0);
            body.velocity *= lin_factor;
            body.angular_velocity *= ang_factor;

            // Update transform on scene node
            if let Some(node) = scene.nodes.get_mut(node_id) {
                node.local_transform.translation += body.velocity * dt;

                let ang_speed = body.angular_velocity.length();
                if ang_speed > 1e-4 {
                    let axis = body.angular_velocity / ang_speed;
                    let delta_rot = Quat::from_axis_angle(axis, ang_speed * dt);
                    node.local_transform.rotation = (delta_rot * node.local_transform.rotation).normalize();
                }
                scene.mark_dirty(node_id);
            }
        }

        // 2. Ground Plane Collision Resolution
        if let Some(ground_y) = self.ground_plane_y {
            for &node_id in &active_keys {
                let (half_extents, restitution, friction) = {
                    let body = &self.bodies[&node_id];
                    let rot = scene.nodes.get(node_id).map(|n| n.local_transform.rotation).unwrap_or(Quat::IDENTITY);
                    (body.shape.half_extents_rotated(rot), body.restitution, body.friction)
                };

                if let Some(node) = scene.nodes.get_mut(node_id) {
                    let bottom = node.local_transform.translation.y - half_extents.y;
                    if bottom < ground_y {
                        let penetration = ground_y - bottom;
                        node.local_transform.translation.y += penetration;
                        scene.mark_dirty(node_id);

                        let body = self.bodies.get_mut(&node_id).unwrap();
                        if body.velocity.y < 0.0 {
                            body.velocity.y = -body.velocity.y * restitution;
                            // Tangential friction
                            let fric_damping = (1.0 - friction * 0.5).clamp(0.0, 1.0);
                            body.velocity.x *= fric_damping;
                            body.velocity.z *= fric_damping;
                            body.angular_velocity *= fric_damping;

                            // Clamp resting jitter
                            if body.velocity.y.abs() < 0.05 {
                                body.velocity.y = 0.0;
                            }
                        }
                    }
                }
            }
        }

        // 3. Inter-body Pairwise Contact Impulse Resolution
        let all_keys: Vec<NodeId> = self.bodies.keys().copied().collect();
        for i in 0..all_keys.len() {
            for j in (i + 1)..all_keys.len() {
                let id_a = all_keys[i];
                let id_b = all_keys[j];

                let body_a_type = self.bodies[&id_a].body_type;
                let body_b_type = self.bodies[&id_b].body_type;

                // At least one must be active
                if body_a_type == RigidBodyType::Passive && body_b_type == RigidBodyType::Passive {
                    continue;
                }

                self.resolve_pair_collision(scene, id_a, id_b);
            }
        }
    }

    fn resolve_pair_collision(&mut self, scene: &mut SceneGraph, id_a: NodeId, id_b: NodeId) {
        let (pos_a, rot_a) = match scene.nodes.get(id_a) {
            Some(n) => (n.local_transform.translation, n.local_transform.rotation),
            None => return,
        };
        let (pos_b, rot_b) = match scene.nodes.get(id_b) {
            Some(n) => (n.local_transform.translation, n.local_transform.rotation),
            None => return,
        };

        let body_a = &self.bodies[&id_a];
        let body_b = &self.bodies[&id_b];

        let ext_a = body_a.shape.half_extents_rotated(rot_a);
        let ext_b = body_b.shape.half_extents_rotated(rot_b);

        let delta = pos_b - pos_a;
        let total_ext = ext_a + ext_b;

        let overlap_x = total_ext.x - delta.x.abs();
        let overlap_y = total_ext.y - delta.y.abs();
        let overlap_z = total_ext.z - delta.z.abs();

        if overlap_x > 0.0 && overlap_y > 0.0 && overlap_z > 0.0 {
            // Find minimum penetration axis
            let (normal, penetration) = if overlap_x <= overlap_y && overlap_x <= overlap_z {
                (Vec3::new(delta.x.signum(), 0.0, 0.0), overlap_x)
            } else if overlap_y <= overlap_x && overlap_y <= overlap_z {
                (Vec3::new(0.0, delta.y.signum(), 0.0), overlap_y)
            } else {
                (Vec3::new(0.0, 0.0, delta.z.signum()), overlap_z)
            };

            let inv_m_a = body_a.inv_mass();
            let inv_m_b = body_b.inv_mass();
            let total_inv = inv_m_a + inv_m_b;

            if total_inv <= 1e-5 {
                return;
            }

            // Position separation
            let sep_a = normal * (-penetration * (inv_m_a / total_inv));
            let sep_b = normal * (penetration * (inv_m_b / total_inv));

            if let Some(node_a) = scene.nodes.get_mut(id_a) {
                node_a.local_transform.translation += sep_a;
                scene.mark_dirty(id_a);
            }
            if let Some(node_b) = scene.nodes.get_mut(id_b) {
                node_b.local_transform.translation += sep_b;
                scene.mark_dirty(id_b);
            }

            // Velocity impulse resolution
            let vel_a = self.bodies[&id_a].velocity;
            let vel_b = self.bodies[&id_b].velocity;
            let rel_vel = vel_b - vel_a;
            let vel_along_normal = rel_vel.dot(normal);

            if vel_along_normal < 0.0 {
                let e = (body_a.restitution + body_b.restitution) * 0.5;
                let impulse_mag = -(1.0 + e) * vel_along_normal / total_inv;
                let impulse = normal * impulse_mag;

                if let Some(b_a) = self.bodies.get_mut(&id_a) {
                    b_a.velocity -= impulse * inv_m_a;
                }
                if let Some(b_b) = self.bodies.get_mut(&id_b) {
                    b_b.velocity += impulse * inv_m_b;
                }
            }
        }
    }

    /// Bakes the physics simulation across a frame range into keyframes on the animation manager.
    pub fn bake_to_keyframes(
        &mut self,
        scene: &mut SceneGraph,
        anim_manager: &mut AnimationManager,
        start_frame: u32,
        end_frame: u32,
        fps: f32,
    ) {
        let dt = 1.0 / fps.max(1.0);
        self.reset_simulation(scene);

        for frame in start_frame..=end_frame {
            let f = frame as f32;
            for &node_id in self.bodies.keys() {
                if let Some(node) = scene.nodes.get(node_id) {
                    anim_manager.insert_keyframe(
                        node_id,
                        f,
                        Transform {
                            translation: node.local_transform.translation,
                            rotation: node.local_transform.rotation,
                            scale: node.local_transform.scale,
                        },
                        InterpolationMode::Linear,
                    );
                }
            }
            self.step(scene, dt);
        }
    }
}

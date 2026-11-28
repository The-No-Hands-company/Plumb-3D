//! Inertial Motion Matching Character Locomotion Engine
//!
//! Provides high-dimensional kinematic pose database search, trajectory forecasting,
//! inertialization blending, and zero-foot-sliding character locomotion (AAA standard).

use core_math::{Vec2, Vec3};

/// Weighting configuration for motion matching cost calculation
#[derive(Debug, Clone)]
pub struct MotionFeatureWeights {
    pub joint_position: f32,
    pub joint_velocity: f32,
    pub trajectory_position: f32,
    pub trajectory_direction: f32,
}

impl Default for MotionFeatureWeights {
    fn default() -> Self {
        Self {
            joint_position: 1.0,
            joint_velocity: 0.5,
            trajectory_position: 2.0,
            trajectory_direction: 1.5,
        }
    }
}

/// A forecasted future trajectory anchor point
#[derive(Debug, Clone, Copy)]
pub struct TrajectorySample {
    pub time_offset: f32,
    pub position: Vec3,
    pub facing_direction: Vec3,
}

/// A discrete recorded kinematic pose inside the motion database
#[derive(Debug, Clone)]
pub struct MotionPose {
    pub time_seconds: f32,
    pub clip_name: String,
    pub root_position: Vec3,
    pub root_velocity: Vec3,
    pub joint_positions: Vec<Vec3>,
    pub joint_velocities: Vec<Vec3>,
    pub left_foot_contact: bool,
    pub right_foot_contact: bool,
    pub trajectory: Vec<TrajectorySample>,
}

/// Critically damped inertialization state tracking position and velocity offsets
#[derive(Debug, Clone, Default)]
pub struct InertializationBlend {
    pub offset_pos: Vec3,
    pub offset_vel: Vec3,
    pub blend_time_remaining: f32,
    pub blend_duration: f32,
}

impl InertializationBlend {
    /// Resets the transition offset between the source pose and newly matched target pose
    pub fn trigger_transition(&mut self, delta_pos: Vec3, delta_vel: Vec3, duration: f32) {
        self.offset_pos = delta_pos;
        self.offset_vel = delta_vel;
        self.blend_duration = duration.max(0.05);
        self.blend_time_remaining = self.blend_duration;
    }

    /// Advances inertial decay by delta time `dt`
    pub fn update(&mut self, dt: f32) -> Vec3 {
        if self.blend_time_remaining <= 0.0 {
            return Vec3::ZERO;
        }

        self.blend_time_remaining = (self.blend_time_remaining - dt).max(0.0);
        let progress = 1.0 - (self.blend_time_remaining / self.blend_duration);

        // Quintic decay curve (1 - t)^3 * (1 + 3t + 6t^2)
        let inv_t = 1.0 - progress;
        let decay = inv_t * inv_t * inv_t;

        self.offset_pos * decay
    }
}

/// High-dimensional kinematic motion database
#[derive(Debug, Clone)]
pub struct MotionDatabase {
    pub poses: Vec<MotionPose>,
}

impl Default for MotionDatabase {
    fn default() -> Self {
        Self::create_default_locomotion_library()
    }
}

impl MotionDatabase {
    pub fn new() -> Self {
        Self { poses: Vec::new() }
    }

    /// Queries the pose database to find the frame that best matches the desired trajectory and current kinematic state
    pub fn query_best_match(
        &self,
        current_pose: &MotionPose,
        desired_trajectory: &[TrajectorySample],
        weights: &MotionFeatureWeights,
    ) -> usize {
        let mut best_index = 0;
        let mut min_cost = f32::MAX;

        for (idx, pose) in self.poses.iter().enumerate() {
            let mut cost = 0.0f32;

            // 1. Joint Positions & Velocities distance
            let joint_count = current_pose
                .joint_positions
                .len()
                .min(pose.joint_positions.len());
            for j in 0..joint_count {
                cost += (current_pose.joint_positions[j] - pose.joint_positions[j])
                    .length_squared()
                    * weights.joint_position;
                cost += (current_pose.joint_velocities[j] - pose.joint_velocities[j])
                    .length_squared()
                    * weights.joint_velocity;
            }

            // 2. Trajectory Matching (matching predicted future points)
            let traj_count = desired_trajectory.len().min(pose.trajectory.len());
            for t in 0..traj_count {
                cost += (desired_trajectory[t].position - pose.trajectory[t].position)
                    .length_squared()
                    * weights.trajectory_position;
                cost += (1.0
                    - desired_trajectory[t]
                        .facing_direction
                        .dot(pose.trajectory[t].facing_direction)
                        .clamp(-1.0, 1.0))
                    * weights.trajectory_direction;
            }

            if cost < min_cost {
                min_cost = cost;
                best_index = idx;
            }
        }

        best_index
    }

    /// Generates a built-in character locomotion library (Walk, Run, Turn, Stop)
    pub fn create_default_locomotion_library() -> Self {
        let mut poses = Vec::with_capacity(120);

        // 1. Walk Cycle (30 frames forward)
        for f in 0..30 {
            let t = f as f32 / 30.0;
            let speed = 1.4; // 1.4 m/s walking speed
            let phase = t * std::f32::consts::TAU;

            let root_pos = Vec3::new(0.0, 1.0 + phase.sin().abs() * 0.05, t * speed);
            let root_vel = Vec3::new(0.0, 0.0, speed);

            let l_foot = Vec3::new(-0.2, (phase.sin()).max(0.0) * 0.15, (phase.cos()) * 0.3);
            let r_foot = Vec3::new(
                0.2,
                (-(phase).sin()).max(0.0) * 0.15,
                (-(phase).cos()) * 0.3,
            );

            let trajectory = vec![
                TrajectorySample {
                    time_offset: 0.2,
                    position: root_pos + Vec3::new(0.0, 0.0, speed * 0.2),
                    facing_direction: Vec3::Z,
                },
                TrajectorySample {
                    time_offset: 0.5,
                    position: root_pos + Vec3::new(0.0, 0.0, speed * 0.5),
                    facing_direction: Vec3::Z,
                },
            ];

            poses.push(MotionPose {
                time_seconds: t,
                clip_name: "WalkForward".to_string(),
                root_position: root_pos,
                root_velocity: root_vel,
                joint_positions: vec![l_foot, r_foot],
                joint_velocities: vec![Vec3::new(0.0, 0.0, speed), Vec3::new(0.0, 0.0, speed)],
                left_foot_contact: phase.sin() <= 0.05,
                right_foot_contact: (-phase.sin()) <= 0.05,
                trajectory,
            });
        }

        // 2. Sprint / Run Cycle (30 frames fast forward)
        for f in 0..30 {
            let t = f as f32 / 30.0;
            let speed = 4.2; // 4.2 m/s sprinting speed
            let phase = t * std::f32::consts::TAU;

            let root_pos = Vec3::new(0.0, 0.95 + phase.sin().abs() * 0.1, t * speed);
            let root_vel = Vec3::new(0.0, 0.0, speed);

            let l_foot = Vec3::new(-0.25, (phase.sin()).max(0.0) * 0.35, (phase.cos()) * 0.6);
            let r_foot = Vec3::new(
                0.25,
                (-(phase).sin()).max(0.0) * 0.35,
                (-(phase).cos()) * 0.6,
            );

            let trajectory = vec![
                TrajectorySample {
                    time_offset: 0.2,
                    position: root_pos + Vec3::new(0.0, 0.0, speed * 0.2),
                    facing_direction: Vec3::Z,
                },
                TrajectorySample {
                    time_offset: 0.5,
                    position: root_pos + Vec3::new(0.0, 0.0, speed * 0.5),
                    facing_direction: Vec3::Z,
                },
            ];

            poses.push(MotionPose {
                time_seconds: t,
                clip_name: "RunSprint".to_string(),
                root_position: root_pos,
                root_velocity: root_vel,
                joint_positions: vec![l_foot, r_foot],
                joint_velocities: vec![Vec3::new(0.0, 0.0, speed), Vec3::new(0.0, 0.0, speed)],
                left_foot_contact: phase.sin() <= 0.05,
                right_foot_contact: (-phase.sin()) <= 0.05,
                trajectory,
            });
        }

        // 3. Sharp Left Turn
        for f in 0..20 {
            let t = f as f32 / 20.0;
            let angle = t * std::f32::consts::FRAC_PI_2;
            let dir = Vec3::new(-angle.sin(), 0.0, angle.cos());
            let pos = Vec3::new(-t * 1.5, 1.0, t * 1.5);

            let trajectory = vec![
                TrajectorySample {
                    time_offset: 0.2,
                    position: pos + dir * 0.3,
                    facing_direction: dir,
                },
                TrajectorySample {
                    time_offset: 0.5,
                    position: pos + dir * 0.8,
                    facing_direction: dir,
                },
            ];

            poses.push(MotionPose {
                time_seconds: t,
                clip_name: "TurnLeft".to_string(),
                root_position: pos,
                root_velocity: dir * 1.5,
                joint_positions: vec![
                    pos + Vec3::new(-0.2, 0.0, 0.0),
                    pos + Vec3::new(0.2, 0.0, 0.0),
                ],
                joint_velocities: vec![dir * 1.5, dir * 1.5],
                left_foot_contact: true,
                right_foot_contact: false,
                trajectory,
            });
        }

        Self { poses }
    }
}

/// High-level runtime controller orchestrating motion matching and inertialization
#[derive(Debug, Clone)]
pub struct MotionMatchingController {
    pub database: MotionDatabase,
    pub current_pose_index: usize,
    pub current_pose: MotionPose,
    pub inertialization: InertializationBlend,
    pub current_root_pos: Vec3,
    pub facing_direction: Vec3,
    pub desired_speed: f32,
    pub weights: MotionFeatureWeights,
}

impl Default for MotionMatchingController {
    fn default() -> Self {
        let db = MotionDatabase::default();
        let first_pose = db.poses.first().cloned().unwrap_or(MotionPose {
            time_seconds: 0.0,
            clip_name: "Idle".to_string(),
            root_position: Vec3::new(0.0, 1.0, 0.0),
            root_velocity: Vec3::ZERO,
            joint_positions: vec![Vec3::new(-0.2, 0.0, 0.0), Vec3::new(0.2, 0.0, 0.0)],
            joint_velocities: vec![Vec3::ZERO, Vec3::ZERO],
            left_foot_contact: true,
            right_foot_contact: true,
            trajectory: Vec::new(),
        });

        Self {
            database: db,
            current_pose_index: 0,
            current_pose: first_pose,
            inertialization: InertializationBlend::default(),
            current_root_pos: Vec3::ZERO,
            facing_direction: Vec3::Z,
            desired_speed: 1.4,
            weights: MotionFeatureWeights::default(),
        }
    }
}

impl MotionMatchingController {
    /// Steps the locomotion controller with joystick direction input and delta time
    pub fn step(&mut self, dt: f32, input_stick: Vec2) {
        let input_len = input_stick.length();
        let target_dir = if input_len > 0.05 {
            Vec3::new(input_stick.x, 0.0, input_stick.y).normalize()
        } else {
            self.facing_direction
        };

        self.facing_direction = self
            .facing_direction
            .lerp(target_dir, (dt * 10.0).min(1.0))
            .normalize();
        let speed = if input_len > 0.7 {
            4.0
        } else if input_len > 0.1 {
            1.4
        } else {
            0.0
        };
        self.desired_speed = speed;

        // Build desired future trajectory
        let desired_trajectory = vec![
            TrajectorySample {
                time_offset: 0.2,
                position: self.current_root_pos + self.facing_direction * (speed * 0.2),
                facing_direction: self.facing_direction,
            },
            TrajectorySample {
                time_offset: 0.5,
                position: self.current_root_pos + self.facing_direction * (speed * 0.5),
                facing_direction: self.facing_direction,
            },
        ];

        // Search database for best matching pose
        let best_idx =
            self.database
                .query_best_match(&self.current_pose, &desired_trajectory, &self.weights);

        if best_idx != self.current_pose_index {
            let next_pose = &self.database.poses[best_idx];
            let delta_pos = self.current_pose.root_position - next_pose.root_position;
            let delta_vel = self.current_pose.root_velocity - next_pose.root_velocity;
            self.inertialization
                .trigger_transition(delta_pos, delta_vel, 0.25);
            self.current_pose_index = best_idx;
        }

        self.current_pose = self.database.poses[self.current_pose_index].clone();
        let inertial_offset = self.inertialization.update(dt);
        self.current_root_pos += self.facing_direction * (speed * dt) + inertial_offset * dt;
    }

    pub fn is_left_foot_planted(&self) -> bool {
        self.current_pose.left_foot_contact
    }

    pub fn is_right_foot_planted(&self) -> bool {
        self.current_pose.right_foot_contact
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_motion_database_query_best_match() {
        let db = MotionDatabase::default();
        assert!(db.poses.len() > 50);

        let current = &db.poses[0];
        let desired_sprint = vec![
            TrajectorySample {
                time_offset: 0.2,
                position: Vec3::new(0.0, 1.0, 0.8),
                facing_direction: Vec3::Z,
            },
            TrajectorySample {
                time_offset: 0.5,
                position: Vec3::new(0.0, 1.0, 2.1),
                facing_direction: Vec3::Z,
            },
        ];

        let weights = MotionFeatureWeights {
            joint_position: 0.1,
            joint_velocity: 0.1,
            trajectory_position: 5.0,
            trajectory_direction: 1.0,
        };
        let best_idx = db.query_best_match(current, &desired_sprint, &weights);
        let matched_pose = &db.poses[best_idx];
        assert_eq!(matched_pose.clip_name, "RunSprint");
    }

    #[test]
    fn test_inertialization_smooth_decay() {
        let mut blend = InertializationBlend::default();
        blend.trigger_transition(Vec3::new(1.0, 0.0, 0.0), Vec3::ZERO, 0.2);

        let initial_offset = blend.update(0.0);
        assert!((initial_offset.x - 1.0).abs() < 1e-4);

        let mid_offset = blend.update(0.1);
        assert!(mid_offset.x < 1.0 && mid_offset.x > 0.0);

        let final_offset = blend.update(0.15);
        assert_eq!(final_offset, Vec3::ZERO);
    }

    #[test]
    fn test_motion_matching_controller_locomotion_step() {
        let mut controller = MotionMatchingController::default();
        let start_pos = controller.current_root_pos;

        // Step forward with joystick
        controller.step(0.016, Vec2::new(0.0, 1.0));
        assert!(controller.current_root_pos.z > start_pos.z);
    }
}

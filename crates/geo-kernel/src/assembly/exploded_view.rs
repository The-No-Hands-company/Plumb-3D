//! Exploded View Engine & Dynamic Trail Line Generator
//!
//! Provides step-by-step assembly disassembly animations, interactive explosion
//! sliders, and 3D schematic trail lines.

use core_math::{Transform, Vec3};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// An individual step in an exploded view sequence
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExplodedStep {
    pub step_index: usize,
    pub name: String,
    /// Indices of bodies moved during this step
    pub body_ids: Vec<usize>,
    /// Unit explosion displacement direction
    pub direction: Vec3,
    /// Total travel distance at 100% explosion
    pub distance: f32,
}

impl ExplodedStep {
    pub fn new(
        step_index: usize,
        name: impl Into<String>,
        body_ids: Vec<usize>,
        direction: Vec3,
        distance: f32,
    ) -> Self {
        let dir = if direction.length_squared() > 1e-6 {
            direction.normalize()
        } else {
            Vec3::Y
        };
        Self {
            step_index,
            name: name.into(),
            body_ids,
            direction: dir,
            distance,
        }
    }
}

/// A 3D dashed guide line connecting an exploded component back to its assembled origin
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ExplodedTrailLine {
    pub body_id: usize,
    pub assembled_origin: Vec3,
    pub exploded_origin: Vec3,
}

/// Exploded view configuration and evaluation engine
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExplodedView {
    pub name: String,
    pub steps: Vec<ExplodedStep>,
    /// Global explosion factor in [0.0, 1.0] (0.0 = fully assembled, 1.0 = fully exploded)
    pub factor: f32,
    /// Whether to generate 3D trail lines
    pub show_trail_lines: bool,
}

impl Default for ExplodedView {
    fn default() -> Self {
        Self {
            name: "Default Exploded View".to_string(),
            steps: Vec::new(),
            factor: 0.0,
            show_trail_lines: true,
        }
    }
}

impl ExplodedView {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    /// Add a step to the exploded view
    pub fn add_step(&mut self, step: ExplodedStep) {
        self.steps.push(step);
    }

    /// Set explosion factor clamped to [0.0, 1.0]
    pub fn set_factor(&mut self, factor: f32) {
        self.factor = factor.clamp(0.0, 1.0);
    }

    /// Evaluate exploded transforms and trail lines given assembled transforms
    pub fn evaluate(
        &self,
        assembled: &[(usize, Transform)],
    ) -> (HashMap<usize, Transform>, Vec<ExplodedTrailLine>) {
        let mut exploded_transforms = HashMap::new();
        let mut trail_lines = Vec::new();

        // Initialize with original assembled transforms
        for &(id, tf) in assembled {
            exploded_transforms.insert(id, tf);
        }

        if self.factor <= 1e-5 || self.steps.is_empty() {
            return (exploded_transforms, trail_lines);
        }

        // Apply explosion offsets step by step
        let total_steps = self.steps.len() as f32;
        let step_window = 1.0 / total_steps;

        for (idx, step) in self.steps.iter().enumerate() {
            let step_start = idx as f32 * step_window;
            let step_end = (idx + 1) as f32 * step_window;

            let step_t = if self.factor <= step_start {
                0.0
            } else if self.factor >= step_end {
                1.0
            } else {
                (self.factor - step_start) / (step_end - step_start)
            };

            if step_t > 0.0 {
                let offset = step.direction * (step.distance * step_t);

                for &body_id in &step.body_ids {
                    if let Some(tf) = exploded_transforms.get_mut(&body_id) {
                        tf.translation += offset;
                    }
                }
            }
        }

        // Generate trail lines if enabled
        if self.show_trail_lines {
            for &(id, orig_tf) in assembled {
                if let Some(exp_tf) = exploded_transforms.get(&id) {
                    if (exp_tf.translation - orig_tf.translation).length_squared() > 1e-4 {
                        trail_lines.push(ExplodedTrailLine {
                            body_id: id,
                            assembled_origin: orig_tf.translation,
                            exploded_origin: exp_tf.translation,
                        });
                    }
                }
            }
        }

        (exploded_transforms, trail_lines)
    }
}

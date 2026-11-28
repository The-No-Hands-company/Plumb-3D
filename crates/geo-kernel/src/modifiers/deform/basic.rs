use crate::mesh::{Mesh, MeshError};
use crate::modifiers::stack::Modifier;
use core_math::{Transform, Vec3};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct TransformModifier {
    pub transform: Transform,
    pub enabled: bool,
}

impl TransformModifier {
    pub fn new(transform: Transform) -> Self {
        Self {
            transform,
            enabled: true,
        }
    }
}

impl Modifier for TransformModifier {
    fn name(&self) -> &str {
        "Transform"
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn apply(&self, input: &Mesh) -> Result<Mesh, MeshError> {
        let mut output = input.clone();
        for (_, v) in &mut output.vertices {
            v.position = self.transform.transform_point(v.position);
        }
        output.update_normals();
        Ok(output)
    }
}

/// Displace Modifier (displaces vertices along their normals via noise function)
#[derive(Debug, Clone)]
pub struct DisplaceModifier {
    pub strength: f32,
    pub frequency: f32,
    pub vertex_group: Option<String>,
    pub enabled: bool,
}

impl DisplaceModifier {
    pub fn new(strength: f32, frequency: f32) -> Self {
        Self {
            strength,
            frequency,
            vertex_group: None,
            enabled: true,
        }
    }

    pub fn with_vertex_group(mut self, group: impl Into<String>) -> Self {
        self.vertex_group = Some(group.into());
        self
    }

    /// Fast procedural 3D noise function (sinusoidal harmonic octaves)
    fn procedural_noise(p: Vec3, freq: f32) -> f32 {
        let x = p.x * freq;
        let y = p.y * freq;
        let z = p.z * freq;

        let o1 = (x.sin() + y.cos() + z.sin()) / 3.0;
        let o2 = ((x * 2.1).cos() + (y * 2.1).sin() + (z * 2.1).cos()) / 6.0;
        o1 + o2
    }
}

impl Modifier for DisplaceModifier {
    fn name(&self) -> &str {
        "Displace"
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn apply(&self, input: &Mesh) -> Result<Mesh, MeshError> {
        let mut output = input.clone();
        for (vk, v) in &mut output.vertices {
            let n = Self::procedural_noise(v.position, self.frequency);
            let weight = if let Some(ref vg) = self.vertex_group {
                input.get_vertex_weight(vg, vk)
            } else {
                1.0
            };
            v.position += v.normal * (n * self.strength * weight);
        }
        output.update_normals();
        Ok(output)
    }
}

/// Classic 3ds Max Taper Modifier (scales cross-section along an axis)
#[derive(Debug, Clone)]
pub struct TaperModifier {
    pub amount: f32,
    pub enabled: bool,
}

impl TaperModifier {
    pub fn new(amount: f32) -> Self {
        Self {
            amount,
            enabled: true,
        }
    }
}

impl Modifier for TaperModifier {
    fn name(&self) -> &str {
        "Taper"
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn apply(&self, input: &Mesh) -> Result<Mesh, MeshError> {
        let aabb = input.compute_aabb();
        let min_y = aabb.min.y;
        let height = (aabb.max.y - min_y).max(0.001);

        let mut output = input.clone();
        for (_, v) in &mut output.vertices {
            let t = ((v.position.y - min_y) / height).clamp(0.0, 1.0);
            let factor = (1.0 + self.amount * t).max(0.01);
            v.position.x *= factor;
            v.position.z *= factor;
        }
        output.update_normals();
        Ok(output)
    }
}

/// Smooth / Relax Modifier (Laplacian mesh smoothing)
#[derive(Debug, Clone)]
pub struct SmoothModifier {
    pub factor: f32, // [0.0, 1.0] relaxation weight
    pub iterations: u32,
    pub vertex_group: Option<String>,
    pub enabled: bool,
}

impl SmoothModifier {
    pub fn new(factor: f32, iterations: u32) -> Self {
        Self {
            factor: factor.clamp(0.0, 1.0),
            iterations,
            vertex_group: None,
            enabled: true,
        }
    }

    pub fn with_vertex_group(mut self, group: impl Into<String>) -> Self {
        self.vertex_group = Some(group.into());
        self
    }
}

impl Modifier for SmoothModifier {
    fn name(&self) -> &str {
        "Smooth"
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn as_smooth_mut(&mut self) -> Option<&mut SmoothModifier> {
        Some(self)
    }

    fn apply(&self, input: &Mesh) -> Result<Mesh, MeshError> {
        if !self.enabled || self.factor <= 0.0 || self.iterations == 0 || input.vertices.is_empty()
        {
            return Ok(input.clone());
        }

        let mut output = input.clone();
        let vg = self
            .vertex_group
            .as_ref()
            .and_then(|g| input.get_vertex_group(g));

        for _ in 0..self.iterations {
            let mut new_positions = HashMap::with_capacity(output.vertices.len());

            for (vk, v) in &output.vertices {
                let neighbors = output.vertex_neighbors(vk);
                if neighbors.is_empty() {
                    new_positions.insert(vk, v.position);
                    continue;
                }

                let mut neighbor_sum = Vec3::ZERO;
                for &n_key in &neighbors {
                    neighbor_sum += output.vertices[n_key].position;
                }
                let centroid = neighbor_sum / neighbors.len() as f32;

                let mask = vg.map(|g| g.get_weight(vk)).unwrap_or(1.0);
                let effective_factor = self.factor * mask;
                let smoothed_pos = v.position.lerp(centroid, effective_factor);
                new_positions.insert(vk, smoothed_pos);
            }

            for (vk, pos) in new_positions {
                output.vertices[vk].position = pos;
            }
        }

        output.update_normals();
        Ok(output)
    }
}

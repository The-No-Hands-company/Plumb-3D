use crate::mesh::{Mesh, MeshError};
use crate::modifiers::deform::types::{DeformAxis, SimpleDeformMethod};
use crate::modifiers::stack::Modifier;

#[derive(Debug, Clone)]
pub struct SimpleDeformModifier {
    pub enabled: bool,
    pub method: SimpleDeformMethod,
    pub axis: DeformAxis,
    pub factor: f32,
    pub limits: (f32, f32),
    pub vertex_group: Option<String>,
}

impl SimpleDeformModifier {
    pub fn new(method: SimpleDeformMethod, factor: f32) -> Self {
        Self {
            enabled: true,
            method,
            axis: DeformAxis::Z,
            factor,
            limits: (0.0, 1.0),
            vertex_group: None,
        }
    }
}

impl Modifier for SimpleDeformModifier {
    fn name(&self) -> &str {
        match self.method {
            SimpleDeformMethod::Twist => "Simple Deform (Twist)",
            SimpleDeformMethod::Bend => "Simple Deform (Bend)",
            SimpleDeformMethod::Taper => "Simple Deform (Taper)",
            SimpleDeformMethod::Stretch => "Simple Deform (Stretch)",
        }
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn as_simple_deform_mut(&mut self) -> Option<&mut SimpleDeformModifier> {
        Some(self)
    }

    fn apply(&self, input: &Mesh) -> Result<Mesh, MeshError> {
        if !self.enabled || input.vertices.is_empty() {
            return Ok(input.clone());
        }

        let mut output = input.clone();

        // 1. Compute bounds along deformation axis
        let mut min_val = f32::MAX;
        let mut max_val = f32::MIN;
        for v in input.vertices.values() {
            let val = match self.axis {
                DeformAxis::X => v.position.x,
                DeformAxis::Y => v.position.y,
                DeformAxis::Z => v.position.z,
            };
            min_val = min_val.min(val);
            max_val = max_val.max(val);
        }

        let range = max_val - min_val;
        if range <= 1e-5 {
            return Ok(output);
        }
        let mid_val = (min_val + max_val) * 0.5;

        // 2. Vertex group lookup
        let vg_ref = self
            .vertex_group
            .as_ref()
            .and_then(|vg_name| input.vertex_groups.iter().find(|vg| &vg.name == vg_name));

        // 3. Deform vertices
        for (vk, v) in output.vertices.iter_mut() {
            let weight = if let Some(vg) = vg_ref {
                vg.get_weight(vk)
            } else {
                1.0
            };

            if weight <= 0.0 {
                continue;
            }

            let pos = v.position;
            let (primary, ortho1, ortho2) = match self.axis {
                DeformAxis::Z => (pos.z, pos.x, pos.y),
                DeformAxis::Y => (pos.y, pos.z, pos.x),
                DeformAxis::X => (pos.x, pos.y, pos.z),
            };

            let raw_t = (primary - min_val) / range;
            let t = raw_t.clamp(self.limits.0, self.limits.1);

            let (new_primary, new_o1, new_o2) = match self.method {
                SimpleDeformMethod::Twist => {
                    let angle = self.factor * (t - 0.5);
                    let cos_a = angle.cos();
                    let sin_a = angle.sin();
                    let no1 = ortho1 * cos_a - ortho2 * sin_a;
                    let no2 = ortho1 * sin_a + ortho2 * cos_a;
                    (primary, no1, no2)
                }
                SimpleDeformMethod::Bend => {
                    let theta = self.factor;
                    if theta.abs() < 1e-4 {
                        (primary, ortho1, ortho2)
                    } else {
                        let radius = range / theta;
                        let alpha = (t - 0.5) * theta;
                        let r = radius - ortho1;
                        let no1 = radius - r * alpha.cos();
                        let np = mid_val + r * alpha.sin();
                        (np, no1, ortho2)
                    }
                }
                SimpleDeformMethod::Taper => {
                    let scale = (1.0 + (t - 0.5) * self.factor).max(0.001);
                    (primary, ortho1 * scale, ortho2 * scale)
                }
                SimpleDeformMethod::Stretch => {
                    let k = (1.0 + self.factor).max(0.01);
                    let np = mid_val + (primary - mid_val) * k;
                    let inv_sqrt = 1.0 / k.sqrt();
                    (np, ortho1 * inv_sqrt, ortho2 * inv_sqrt)
                }
            };

            let deformed_pos = match self.axis {
                DeformAxis::Z => core_math::vec3(new_o1, new_o2, new_primary),
                DeformAxis::Y => core_math::vec3(new_o2, new_primary, new_o1),
                DeformAxis::X => core_math::vec3(new_primary, new_o1, new_o2),
            };

            v.position = pos.lerp(deformed_pos, weight);
        }

        output.update_normals();
        Ok(output)
    }
}

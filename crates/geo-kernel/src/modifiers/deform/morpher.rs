use crate::mesh::{Mesh, MeshError, VertexKey};
use crate::modifiers::stack::Modifier;
use core_math::Vec3;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct MorphTarget {
    pub name: String,
    pub weight: f32,
    pub deltas: HashMap<VertexKey, Vec3>,
    pub vertex_group: Option<String>,
}

impl MorphTarget {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            weight: 0.0,
            deltas: HashMap::new(),
            vertex_group: None,
        }
    }

    pub fn with_deltas(name: impl Into<String>, deltas: HashMap<VertexKey, Vec3>) -> Self {
        Self {
            name: name.into(),
            weight: 0.0,
            deltas,
            vertex_group: None,
        }
    }

    pub fn set_delta(&mut self, vk: VertexKey, delta: Vec3) {
        if delta.length_squared() > 1e-8 {
            self.deltas.insert(vk, delta);
        } else {
            self.deltas.remove(&vk);
        }
    }
}

/// 3ds Max Morpher / Maya Blend Shape / Blender Shape Keys Modifier
#[derive(Debug, Clone, Default)]
pub struct MorpherModifier {
    pub targets: Vec<MorphTarget>,
    pub enabled: bool,
}

impl MorpherModifier {
    pub fn new() -> Self {
        Self {
            targets: Vec::new(),
            enabled: true,
        }
    }

    pub fn add_target(&mut self, target: MorphTarget) -> usize {
        self.targets.push(target);
        self.targets.len() - 1
    }

    pub fn add_target_from_mesh(
        &mut self,
        name: impl Into<String>,
        base_mesh: &Mesh,
        target_mesh: &Mesh,
    ) -> Result<usize, MeshError> {
        let mut deltas = HashMap::new();
        for (vk, base_v) in &base_mesh.vertices {
            if let Some(target_v) = target_mesh.vertices.get(vk) {
                let diff = target_v.position - base_v.position;
                if diff.length_squared() > 1e-8 {
                    deltas.insert(vk, diff);
                }
            }
        }
        let target = MorphTarget {
            name: name.into(),
            weight: 0.0,
            deltas,
            vertex_group: None,
        };
        self.targets.push(target);
        Ok(self.targets.len() - 1)
    }

    pub fn get_target(&self, name: &str) -> Option<&MorphTarget> {
        self.targets.iter().find(|t| t.name == name)
    }

    pub fn get_target_mut(&mut self, name: &str) -> Option<&mut MorphTarget> {
        self.targets.iter_mut().find(|t| t.name == name)
    }

    pub fn set_target_weight(&mut self, name: &str, weight: f32) -> bool {
        if let Some(t) = self.get_target_mut(name) {
            t.weight = weight;
            true
        } else {
            false
        }
    }
}

impl Modifier for MorpherModifier {
    fn name(&self) -> &str {
        "Morpher"
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn apply(&self, input: &Mesh) -> Result<Mesh, MeshError> {
        if self.targets.is_empty() {
            return Ok(input.clone());
        }

        let mut output = input.clone();
        let mut any_displacement = false;

        for target in &self.targets {
            if target.weight.abs() < 1e-6 {
                continue;
            }
            any_displacement = true;
            for (&vk, &delta) in &target.deltas {
                if let Some(v) = output.vertices.get_mut(vk) {
                    let mask = if let Some(ref vg_name) = target.vertex_group {
                        input.get_vertex_weight(vg_name, vk)
                    } else {
                        1.0
                    };
                    v.position += delta * (target.weight * mask);
                }
            }
        }

        if any_displacement {
            output.update_normals();
        }
        Ok(output)
    }

    fn as_morpher(&self) -> Option<&MorpherModifier> {
        Some(self)
    }

    fn as_morpher_mut(&mut self) -> Option<&mut MorpherModifier> {
        Some(self)
    }
}

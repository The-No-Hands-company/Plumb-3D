use crate::mesh::{Mesh, MeshError};
use crate::modifiers::stack::Modifier;
use core_math::{Mat4, Vec3};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct SkinBone {
    pub name: String,
    pub inverse_bind_matrix: Mat4,
    pub current_world_matrix: Mat4,
}

impl SkinBone {
    pub fn new(name: impl Into<String>, inverse_bind_matrix: Mat4, current_world_matrix: Mat4) -> Self {
        Self {
            name: name.into(),
            inverse_bind_matrix,
            current_world_matrix,
        }
    }
}

/// Linear Blend Skinning (LBS) Modifier
/// Deforms mesh vertices according to VertexGroup weights mapped to Skeletal Bones
#[derive(Debug, Clone)]
pub struct SkinModifier {
    pub bones: Vec<SkinBone>,
    pub enabled: bool,
}

impl SkinModifier {
    pub fn new(bones: Vec<SkinBone>) -> Self {
        Self {
            bones,
            enabled: true,
        }
    }

    pub fn add_bone(&mut self, bone: SkinBone) {
        self.bones.push(bone);
    }
}

impl Modifier for SkinModifier {
    fn name(&self) -> &str {
        "Skin (LBS)"
    }

    fn as_skin_mut(&mut self) -> Option<&mut SkinModifier> {
        Some(self)
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn apply(&self, input: &Mesh) -> Result<Mesh, MeshError> {
        if self.bones.is_empty() {
            return Ok(input.clone());
        }

        let mut output = input.clone();

        // Precompute bone skinning matrices: M_b * B_b^-1
        let skin_matrices: Vec<Mat4> = self.bones
            .iter()
            .map(|b| b.current_world_matrix * b.inverse_bind_matrix)
            .collect();

        let bone_indices: HashMap<&str, usize> = self.bones
            .iter()
            .enumerate()
            .map(|(i, b)| (b.name.as_str(), i))
            .collect();

        for (vk, v) in &mut output.vertices {
            let mut total_weight = 0.0_f32;
            let mut deformed_pos = Vec3::ZERO;
            let orig_pos = v.position;

            for vg in &input.vertex_groups {
                if let Some(&bone_idx) = bone_indices.get(vg.name.as_str()) {
                    let w = vg.get_weight(vk);
                    if w > 0.0 {
                        total_weight += w;
                        let transformed_p = skin_matrices[bone_idx].transform_point3(orig_pos);
                        deformed_pos += transformed_p * w;
                    }
                }
            }

            if total_weight > 1e-5 {
                v.position = deformed_pos / total_weight;
            }
        }

        output.update_normals();
        Ok(output)
    }
}

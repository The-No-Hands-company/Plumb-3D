use crate::mesh::{Mesh, MeshError};
use crate::modifiers::stack::Modifier;
use core_math::{Mat4, Quat, Vec3, Vec4};
use std::collections::HashMap;

/// Skinning algorithm to use for vertex deformation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SkinningMethod {
    #[default]
    LinearBlend,
    DualQuaternion,
}

#[derive(Debug, Clone)]
pub struct SkinBone {
    pub name: String,
    pub inverse_bind_matrix: Mat4,
    pub current_world_matrix: Mat4,
}

impl SkinBone {
    pub fn new(
        name: impl Into<String>,
        inverse_bind_matrix: Mat4,
        current_world_matrix: Mat4,
    ) -> Self {
        Self {
            name: name.into(),
            inverse_bind_matrix,
            current_world_matrix,
        }
    }
}

/// Dual Quaternion representation (real quaternion for rotation, dual quaternion for translation)
#[derive(Debug, Clone, Copy)]
struct DualQuat {
    real: Quat,
    dual: Vec4,
}

impl DualQuat {
    fn from_mat4(m: Mat4) -> Self {
        let (_, rot, trans) = m.to_scale_rotation_translation();
        let real = rot.normalize();
        let t_quat = Quat::from_xyzw(trans.x, trans.y, trans.z, 0.0);
        let d = t_quat * real;
        let dual = Vec4::new(d.x * 0.5, d.y * 0.5, d.z * 0.5, d.w * 0.5);
        Self { real, dual }
    }

    fn transform_point(&self, p: Vec3) -> Vec3 {
        let len = self.real.length();
        if len < 1e-6 {
            return p;
        }
        let inv_len = 1.0 / len;
        let r = self.real * inv_len;
        let d = self.dual * inv_len;
        let d_quat = Quat::from_xyzw(d.x, d.y, d.z, d.w);
        let t = (d_quat * r.conjugate()) * 2.0;
        let trans = Vec3::new(t.x, t.y, t.z);
        r.mul_vec3(p) + trans
    }
}

/// Linear Blend Skinning (LBS) & Dual Quaternion Skinning (DQS) Modifier
/// Deforms mesh vertices according to VertexGroup weights mapped to Skeletal Bones
#[derive(Debug, Clone)]
pub struct SkinModifier {
    pub bones: Vec<SkinBone>,
    pub method: SkinningMethod,
    pub enabled: bool,
}

impl SkinModifier {
    pub fn new(bones: Vec<SkinBone>) -> Self {
        Self {
            bones,
            method: SkinningMethod::LinearBlend,
            enabled: true,
        }
    }

    pub fn with_method(mut self, method: SkinningMethod) -> Self {
        self.method = method;
        self
    }

    pub fn add_bone(&mut self, bone: SkinBone) {
        self.bones.push(bone);
    }
}

impl Modifier for SkinModifier {
    fn name(&self) -> &str {
        match self.method {
            SkinningMethod::LinearBlend => "Skin (LBS)",
            SkinningMethod::DualQuaternion => "Skin (DQS Dual-Quaternion)",
        }
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
        let skin_matrices: Vec<Mat4> = self
            .bones
            .iter()
            .map(|b| b.current_world_matrix * b.inverse_bind_matrix)
            .collect();

        let bone_indices: HashMap<&str, usize> = self
            .bones
            .iter()
            .enumerate()
            .map(|(i, b)| (b.name.as_str(), i))
            .collect();

        match self.method {
            SkinningMethod::LinearBlend => {
                for (vk, v) in &mut output.vertices {
                    let mut total_weight = 0.0_f32;
                    let mut deformed_pos = Vec3::ZERO;
                    let orig_pos = v.position;

                    for vg in &input.vertex_groups {
                        if let Some(&bone_idx) = bone_indices.get(vg.name.as_str()) {
                            let w = vg.get_weight(vk);
                            if w > 0.0 {
                                total_weight += w;
                                let transformed_p =
                                    skin_matrices[bone_idx].transform_point3(orig_pos);
                                deformed_pos += transformed_p * w;
                            }
                        }
                    }

                    if total_weight > 1e-5 {
                        v.position = deformed_pos / total_weight;
                    }
                }
            }
            SkinningMethod::DualQuaternion => {
                let dual_quats: Vec<DualQuat> = skin_matrices
                    .iter()
                    .map(|&m| DualQuat::from_mat4(m))
                    .collect();

                for (vk, v) in &mut output.vertices {
                    let mut total_weight = 0.0_f32;
                    let mut blended_real = Quat::from_xyzw(0.0, 0.0, 0.0, 0.0);
                    let mut blended_dual = Vec4::ZERO;
                    let mut pivot_real: Option<Quat> = None;
                    let orig_pos = v.position;

                    for vg in &input.vertex_groups {
                        if let Some(&bone_idx) = bone_indices.get(vg.name.as_str()) {
                            let w = vg.get_weight(vk);
                            if w > 0.0 {
                                let dq = dual_quats[bone_idx];
                                let sign = if let Some(p) = pivot_real {
                                    if p.dot(dq.real) < 0.0 {
                                        -1.0
                                    } else {
                                        1.0
                                    }
                                } else {
                                    pivot_real = Some(dq.real);
                                    1.0
                                };

                                blended_real.x += dq.real.x * w * sign;
                                blended_real.y += dq.real.y * w * sign;
                                blended_real.z += dq.real.z * w * sign;
                                blended_real.w += dq.real.w * w * sign;
                                blended_dual += dq.dual * (w * sign);
                                total_weight += w;
                            }
                        }
                    }

                    if total_weight > 1e-5 {
                        let blended_dq = DualQuat {
                            real: blended_real,
                            dual: blended_dual,
                        };
                        v.position = blended_dq.transform_point(orig_pos);
                    }
                }
            }
        }

        output.update_normals();
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mesh::VertexGroup;
    use crate::primitives::Primitives;
    use core_math::{vec3, Mat4};

    #[test]
    fn test_lbs_and_dqs_skinning_pipeline() {
        let mut cube = Primitives::create_cube(2.0);
        let mut vg_bone1 = VertexGroup::new("Bone.001");
        let mut vg_bone2 = VertexGroup::new("Bone.002");

        for (vk, v) in &cube.vertices {
            if v.position.y > 0.0 {
                vg_bone1.set_weight(vk, 1.0);
            } else {
                vg_bone2.set_weight(vk, 1.0);
            }
        }
        cube.vertex_groups.push(vg_bone1);
        cube.vertex_groups.push(vg_bone2);

        let b1 = SkinBone::new(
            "Bone.001",
            Mat4::IDENTITY,
            Mat4::from_translation(vec3(0.0, 5.0, 0.0)),
        );
        let b2 = SkinBone::new(
            "Bone.002",
            Mat4::IDENTITY,
            Mat4::from_rotation_y(std::f32::consts::PI * 0.5),
        );

        // Test LBS
        let lbs_mod = SkinModifier::new(vec![b1.clone(), b2.clone()]);
        let lbs_res = lbs_mod.apply(&cube).expect("LBS skinning failed");
        assert_eq!(lbs_res.vertices.len(), cube.vertices.len());

        // Test DQS
        let dqs_mod = SkinModifier::new(vec![b1, b2]).with_method(SkinningMethod::DualQuaternion);
        let dqs_res = dqs_mod.apply(&cube).expect("DQS skinning failed");
        assert_eq!(dqs_res.vertices.len(), cube.vertices.len());
    }
}

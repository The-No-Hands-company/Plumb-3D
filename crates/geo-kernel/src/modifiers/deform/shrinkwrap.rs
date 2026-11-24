use crate::mesh::{Mesh, MeshError};
use crate::modifiers::deform::types::ShrinkwrapMethod;
use crate::modifiers::stack::Modifier;
use core_math::{Mat4, Vec3};

#[derive(Debug, Clone)]
pub struct ShrinkwrapModifier {
    pub target_node_name: Option<String>,
    pub target_mesh: Option<Mesh>,
    pub offset: f32,
    pub method: ShrinkwrapMethod,
    pub vertex_group: Option<String>,
    pub enabled: bool,
}

impl Default for ShrinkwrapModifier {
    fn default() -> Self {
        Self::new()
    }
}

impl ShrinkwrapModifier {
    pub fn new() -> Self {
        Self {
            target_node_name: None,
            target_mesh: None,
            offset: 0.0,
            method: ShrinkwrapMethod::NearestSurfacePoint,
            vertex_group: None,
            enabled: true,
        }
    }

    pub fn with_target(mut self, target_mesh: Mesh) -> Self {
        self.target_mesh = Some(target_mesh);
        self
    }

    pub fn with_offset(mut self, offset: f32) -> Self {
        self.offset = offset;
        self
    }

    pub fn with_vertex_group(mut self, group: impl Into<String>) -> Self {
        self.vertex_group = Some(group.into());
        self
    }
}

impl Modifier for ShrinkwrapModifier {
    fn name(&self) -> &str {
        "Shrinkwrap"
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn as_shrinkwrap_mut(&mut self) -> Option<&mut ShrinkwrapModifier> {
        Some(self)
    }

    fn apply(&self, input: &Mesh) -> Result<Mesh, MeshError> {
        if !self.enabled || input.vertices.is_empty() {
            return Ok(input.clone());
        }

        let target = match &self.target_mesh {
            Some(t) if !t.faces.is_empty() => t,
            _ => return Ok(input.clone()),
        };

        let mut output = input.clone();
        let vg = self.vertex_group.as_ref().and_then(|g| input.get_vertex_group(g));

        for (vk, v) in &mut output.vertices {
            let weight = vg.map(|g| g.get_weight(vk)).unwrap_or(1.0);
            if weight <= 0.0 {
                continue;
            }

            let projected = match self.method {
                ShrinkwrapMethod::ProjectNormal => {
                    let normal = if v.normal.length_squared() > 1e-4 {
                        v.normal.normalize()
                    } else {
                        Vec3::Y
                    };
                    let ray_fwd = core_math::Ray::new(v.position, normal);
                    let ray_bwd = core_math::Ray::new(v.position, -normal);
                    let hit_fwd = target.raycast(&ray_fwd, &Mat4::IDENTITY);
                    let hit_bwd = target.raycast(&ray_bwd, &Mat4::IDENTITY);

                    let hit = match (hit_fwd, hit_bwd) {
                        (Some(h1), Some(h2)) => {
                            if h1.distance <= h2.distance {
                                Some(h1)
                            } else {
                                Some(h2)
                            }
                        }
                        (Some(h), None) | (None, Some(h)) => Some(h),
                        (None, None) => None,
                    };

                    if let Some(h) = hit {
                        h.point + h.normal * self.offset
                    } else if let Some((pt, norm)) = target.closest_surface_point(v.position) {
                        pt + norm * self.offset
                    } else {
                        v.position
                    }
                }
                ShrinkwrapMethod::NearestSurfacePoint => {
                    if let Some((pt, norm)) = target.closest_surface_point(v.position) {
                        pt + norm * self.offset
                    } else {
                        v.position
                    }
                }
            };

            v.position = v.position.lerp(projected, weight);
        }

        output.update_normals();
        Ok(output)
    }
}

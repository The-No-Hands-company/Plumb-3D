use crate::mesh::{Mesh, MeshError};
use crate::modifiers::deform::types::DeformAxis;
use crate::modifiers::stack::Modifier;
use crate::spline::Spline;
use core_math::Vec3;

/// Deforms a mesh along a 3D Bézier spline (Blender Curve Modifier / 3ds Max PathDeform)
#[derive(Debug, Clone)]
pub struct CurveModifier {
    pub spline: Spline,
    pub deform_axis: DeformAxis,
    pub vertex_group: Option<String>,
    pub target_curve: Option<String>,
    pub enabled: bool,
}

impl CurveModifier {
    pub fn new(spline: Spline) -> Self {
        Self {
            spline,
            deform_axis: DeformAxis::Y,
            vertex_group: None,
            target_curve: None,
            enabled: true,
        }
    }
}

impl Modifier for CurveModifier {
    fn name(&self) -> &str {
        "Curve (Spline Deform)"
    }

    fn as_curve_mut(&mut self) -> Option<&mut CurveModifier> {
        Some(self)
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn apply(&self, input: &Mesh) -> Result<Mesh, MeshError> {
        let mut output = input.clone();
        if self.spline.points.len() < 2 {
            return Ok(output);
        }

        let mut min_coord = f32::INFINITY;
        let mut max_coord = f32::NEG_INFINITY;
        for (_, v) in &input.vertices {
            let c = match self.deform_axis {
                DeformAxis::X => v.position.x,
                DeformAxis::Y => v.position.y,
                DeformAxis::Z => v.position.z,
            };
            min_coord = min_coord.min(c);
            max_coord = max_coord.max(c);
        }

        let range = (max_coord - min_coord).max(1e-4);
        let vg = self
            .vertex_group
            .as_ref()
            .and_then(|name| input.get_vertex_group(name));
        let world_up = Vec3::Y;

        for (vk, v) in &mut output.vertices {
            let coord = match self.deform_axis {
                DeformAxis::X => v.position.x,
                DeformAxis::Y => v.position.y,
                DeformAxis::Z => v.position.z,
            };
            let u = ((coord - min_coord) / range).clamp(0.0, 1.0);
            let (pos, tangent) = self.spline.eval(u);

            let (offset_a, offset_b) = match self.deform_axis {
                DeformAxis::X => (v.position.y, v.position.z),
                DeformAxis::Y => (v.position.x, v.position.z),
                DeformAxis::Z => (v.position.x, v.position.y),
            };

            let forward = if tangent.length_squared() > 1e-4 {
                tangent.normalize()
            } else {
                Vec3::Z
            };

            let right = forward.cross(world_up).normalize_or_zero();
            let up = if right.length_squared() > 1e-4 {
                right.cross(forward).normalize()
            } else {
                world_up
            };
            let normal = if right.length_squared() > 1e-4 {
                right
            } else {
                Vec3::X
            };

            let deformed = pos + normal * offset_a + up * offset_b;
            let mask = vg.map(|g| g.get_weight(vk)).unwrap_or(1.0);
            v.position = v.position.lerp(deformed, mask);
        }

        output.update_normals();
        Ok(output)
    }
}

use crate::mesh::{Mesh, MeshError, VertexKey};
use crate::modifiers::stack::Modifier;
use core_math::Vec3;
use std::collections::HashMap;

/// Non-destructive Solidify Modifier (extrapolates surface skin with wall thickness)
#[derive(Debug, Clone)]
pub struct SolidifyModifier {
    pub thickness: f32,
    pub offset: f32, // -1.0 (inward) to 1.0 (outward), 0.0 (centered)
    pub rim_only: bool,
    pub enabled: bool,
}

impl SolidifyModifier {
    pub fn new(thickness: f32) -> Self {
        Self {
            thickness,
            offset: -1.0,
            rim_only: false,
            enabled: true,
        }
    }
}

impl Modifier for SolidifyModifier {
    fn name(&self) -> &str {
        "Solidify"
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn as_solidify_mut(&mut self) -> Option<&mut SolidifyModifier> {
        Some(self)
    }

    fn apply(&self, input: &Mesh) -> Result<Mesh, MeshError> {
        if !self.enabled || self.thickness.abs() < 1e-6 || input.vertices.is_empty() {
            return Ok(input.clone());
        }

        let mut output = Mesh::new();

        // 1. Calculate displacements
        let disp_front_factor = (1.0 + self.offset) * 0.5 * self.thickness;
        let disp_back_factor = (-1.0 + self.offset) * 0.5 * self.thickness;

        // 2. Create front and back vertices
        let mut front_v = HashMap::new();
        let mut back_v = HashMap::new();

        for (vk, v) in &input.vertices {
            let n = if v.normal.length_squared() > 1e-6 {
                v.normal.normalize()
            } else {
                Vec3::Y
            };

            let p_front = v.position + n * disp_front_factor;
            let p_back = v.position + n * disp_back_factor;

            let front_key = output.add_vertex_with_uv(p_front, v.uv);
            let back_key = output.add_vertex_with_uv(p_back, v.uv);

            front_v.insert(vk, front_key);
            back_v.insert(vk, back_key);
        }

        // 3. Propagate vertex groups
        for vg in &input.vertex_groups {
            let idx = output.add_vertex_group(&vg.name);
            let out_vg = &mut output.vertex_groups[idx];
            for (&vk, &w) in &vg.weights {
                if let Some(&f_key) = front_v.get(&vk) {
                    out_vg.set_weight(f_key, w);
                }
                if let Some(&b_key) = back_v.get(&vk) {
                    out_vg.set_weight(b_key, w);
                }
            }
        }

        // 4. Add front and back faces (unless rim_only is true)
        if !self.rim_only {
            for (fk, _) in &input.faces {
                let v_keys = input.face_vertices(fk);
                if v_keys.len() < 3 {
                    continue;
                }

                // Front face (original winding)
                let front_face_keys: Vec<VertexKey> = v_keys.iter().map(|k| front_v[k]).collect();
                output.add_face(&front_face_keys)?;

                // Back face (reversed winding so normals face inward/opposite)
                let mut back_face_keys: Vec<VertexKey> = v_keys.iter().map(|k| back_v[k]).collect();
                back_face_keys.reverse();
                output.add_face(&back_face_keys)?;
            }
        }

        // 5. Detect boundary edges and stitch with quad rims
        // In Half-Edge topology, a boundary half-edge has face.is_some() but twin has face.is_none().
        for (_, he) in &input.half_edges {
            if he.face.is_some() {
                let twin = &input.half_edges[he.twin];
                if twin.face.is_none() {
                    let v_start = he.origin;
                    let v_end = he.target;

                    let f_start = front_v[&v_start];
                    let f_end = front_v[&v_end];
                    let b_start = back_v[&v_start];
                    let b_end = back_v[&v_end];

                    // Quad rim connecting front to back: [f_end, f_start, b_start, b_end]
                    output.add_face(&[f_end, f_start, b_start, b_end])?;
                }
            }
        }

        output.update_normals();
        Ok(output)
    }
}

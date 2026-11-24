use crate::mesh::{Mesh, MeshError, VertexKey};
use crate::modifiers::deform::DeformAxis;
use crate::modifiers::stack::Modifier;
use std::collections::HashMap;

/// 3ds Max Lathe with Pitch / Blender Screw modifier
#[derive(Debug, Clone)]
pub struct ScrewModifier {
    pub enabled: bool,
    pub axis: DeformAxis,
    pub screw_offset: f32,
    pub angle: f32,
    pub steps: usize,
    pub iterations: usize,
}

impl ScrewModifier {
    pub fn new(screw_offset: f32, iterations: usize) -> Self {
        Self {
            enabled: true,
            axis: DeformAxis::Z,
            screw_offset,
            angle: std::f32::consts::TAU,
            steps: 16,
            iterations: iterations.max(1),
        }
    }
}

impl Modifier for ScrewModifier {
    fn name(&self) -> &str {
        "Screw (Lathe & Pitch)"
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn as_screw_mut(&mut self) -> Option<&mut ScrewModifier> {
        Some(self)
    }

    fn apply(&self, input: &Mesh) -> Result<Mesh, MeshError> {
        if !self.enabled || input.vertices.is_empty() {
            return Ok(input.clone());
        }

        let total_steps = (self.steps * self.iterations).max(3);
        let total_angle = self.angle * self.iterations as f32;
        let total_elevation = self.screw_offset * self.iterations as f32;

        let mut output = Mesh::new();

        // 1. Generate vertices for all step rings
        let mut step_v_maps: Vec<HashMap<VertexKey, VertexKey>> = Vec::with_capacity(total_steps + 1);

        for step in 0..=total_steps {
            let frac = step as f32 / total_steps as f32;
            let current_angle = total_angle * frac;
            let current_elev = total_elevation * frac;

            let rot = match self.axis {
                DeformAxis::X => core_math::Quat::from_rotation_x(current_angle),
                DeformAxis::Y => core_math::Quat::from_rotation_y(current_angle),
                DeformAxis::Z => core_math::Quat::from_rotation_z(current_angle),
            };

            let elev_vec = match self.axis {
                DeformAxis::X => core_math::vec3(current_elev, 0.0, 0.0),
                DeformAxis::Y => core_math::vec3(0.0, current_elev, 0.0),
                DeformAxis::Z => core_math::vec3(0.0, 0.0, current_elev),
            };

            let mut v_map = HashMap::new();
            for (vk, v) in &input.vertices {
                let rot_pos = rot * v.position + elev_vec;
                let new_vk = output.add_vertex_with_uv(rot_pos, v.uv);
                v_map.insert(vk, new_vk);
            }
            step_v_maps.push(v_map);
        }

        // 2. Extrude edges or boundary edges across steps
        let boundary_edges: Vec<(VertexKey, VertexKey)> = {
            let mut list = Vec::new();
            if input.faces.is_empty() {
                // Input is wire/spline edges
                for edge in input.edges.values() {
                    let he = &input.half_edges[edge.half_edge];
                    list.push((he.origin, he.target));
                }
            } else {
                // Find open boundary half-edges
                for he in input.half_edges.values() {
                    if he.face.is_some() {
                        let is_boundary = input.half_edges.get(he.twin).map(|t| t.face.is_none()).unwrap_or(true);
                        if is_boundary {
                            list.push((he.origin, he.target));
                        }
                    }
                }
            }
            list
        };

        for (v0, v1) in &boundary_edges {
            for step in 0..total_steps {
                let p00 = step_v_maps[step][v0];
                let p01 = step_v_maps[step][v1];
                let p11 = step_v_maps[step + 1][v1];
                let p10 = step_v_maps[step + 1][v0];

                let _ = output.add_face(&[p00, p01, p11, p10]);
            }
        }

        // 3. Add start and end caps if input had faces
        if !input.faces.is_empty() {
            // Start cap at step 0 (reverse winding for outward normal)
            for fk in input.faces.keys() {
                let f_verts = input.face_vertices(fk);
                let cap_verts: Vec<VertexKey> = f_verts.iter().rev().map(|k| step_v_maps[0][k]).collect();
                let _ = output.add_face(&cap_verts);
            }

            // End cap at step total_steps (forward winding)
            for fk in input.faces.keys() {
                let f_verts = input.face_vertices(fk);
                let cap_verts: Vec<VertexKey> = f_verts.iter().map(|k| step_v_maps[total_steps][k]).collect();
                let _ = output.add_face(&cap_verts);
            }
        }

        output.update_normals();
        Ok(output)
    }
}

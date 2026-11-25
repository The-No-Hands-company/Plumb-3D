use crate::mesh::{FaceKey, Mesh, MeshError, VertexKey};
use crate::modifiers::MirrorAxis;
use crate::operators::types::SymmetrizeDirection;
use core_math::{Quat, Vec2, Vec3};
use std::collections::HashMap;

impl Mesh {
    /// Bsects the mesh along a mirror plane and duplicates the retained half with reversed winding.
    pub fn symmetrize(
        &mut self,
        axis: MirrorAxis,
        direction: SymmetrizeDirection,
    ) -> Result<(), MeshError> {
        let (plane_norm, clear_pos, clear_neg) = match (axis, direction) {
            (MirrorAxis::X, SymmetrizeDirection::PositiveToNegative) => (Vec3::X, false, true),
            (MirrorAxis::X, SymmetrizeDirection::NegativeToPositive) => (Vec3::X, true, false),
            (MirrorAxis::Y, SymmetrizeDirection::PositiveToNegative) => (Vec3::Y, false, true),
            (MirrorAxis::Y, SymmetrizeDirection::NegativeToPositive) => (Vec3::Y, true, false),
            (MirrorAxis::Z, SymmetrizeDirection::PositiveToNegative) => (Vec3::Z, false, true),
            (MirrorAxis::Z, SymmetrizeDirection::NegativeToPositive) => (Vec3::Z, true, false),
        };

        // 1. Bisect across symmetry plane and remove the destination half
        self.bisect(Vec3::ZERO, plane_norm, clear_pos, clear_neg, false)?;

        let sym_tolerance = 1e-3_f32;

        // Snap vertices near the symmetry plane exactly onto the plane
        for v in self.vertices.values_mut() {
            let val = match axis {
                MirrorAxis::X => &mut v.position.x,
                MirrorAxis::Y => &mut v.position.y,
                MirrorAxis::Z => &mut v.position.z,
            };
            if val.abs() <= sym_tolerance {
                *val = 0.0;
            }
        }

        // 2. Collect existing vertices to mirror
        let existing_verts: Vec<(VertexKey, Vec3, Vec2)> = self
            .vertices
            .iter()
            .map(|(k, v)| (k, v.position, v.uv))
            .collect();

        let mut old_to_mirrored_v: HashMap<VertexKey, VertexKey> = HashMap::new();

        for (vk, pos, uv) in existing_verts {
            let is_on_seam = match axis {
                MirrorAxis::X => pos.x.abs() <= 1e-4,
                MirrorAxis::Y => pos.y.abs() <= 1e-4,
                MirrorAxis::Z => pos.z.abs() <= 1e-4,
            };

            if is_on_seam {
                old_to_mirrored_v.insert(vk, vk);
            } else {
                let mirrored_pos = match axis {
                    MirrorAxis::X => Vec3::new(-pos.x, pos.y, pos.z),
                    MirrorAxis::Y => Vec3::new(pos.x, -pos.y, pos.z),
                    MirrorAxis::Z => Vec3::new(pos.x, pos.y, -pos.z),
                };
                let mirrored_vk = self.add_vertex_with_uv(mirrored_pos, uv);
                old_to_mirrored_v.insert(vk, mirrored_vk);
            }
        }

        // 3. Mirror all faces (reversing winding order to keep outward normals)
        let existing_faces: Vec<FaceKey> = self.faces.keys().collect();
        for fk in existing_faces {
            let verts = self.face_vertices(fk);
            let mut mirrored_face_verts: Vec<VertexKey> =
                verts.iter().map(|k| old_to_mirrored_v[k]).collect();
            mirrored_face_verts.reverse();
            let _ = self.add_face(&mirrored_face_verts);
        }

        // 4. Weld any slightly off seam vertices
        let _ = self.weld_vertices_by_distance(0.001);
        self.update_normals();

        Ok(())
    }

    /// Lathes / revolves selected faces around an axis and center by angle_radians in discrete steps.
    pub fn spin(
        &mut self,
        faces_to_spin: &[FaceKey],
        axis: Vec3,
        center: Vec3,
        angle_radians: f32,
        steps: usize,
    ) -> Result<Vec<FaceKey>, MeshError> {
        if faces_to_spin.is_empty() || steps == 0 || axis.length_squared() < 1e-6 {
            return Ok(Vec::new());
        }

        let norm_axis = axis.normalize();
        let angle_step = angle_radians / steps as f32;
        let is_full_circle = (angle_radians.abs() - std::f32::consts::TAU).abs() < 1e-3;

        let mut created_faces = Vec::new();

        for &source_fk in faces_to_spin {
            let base_verts = self.face_vertices(source_fk);
            if base_verts.len() < 2 {
                continue;
            }

            let mut prev_ring: Vec<VertexKey> = base_verts.clone();
            let first_ring: Vec<VertexKey> = base_verts.clone();

            for s in 1..=steps {
                let is_last = s == steps;
                let cur_ring: Vec<VertexKey> = if is_last && is_full_circle {
                    first_ring.clone()
                } else {
                    let rot = Quat::from_axis_angle(norm_axis, angle_step * s as f32);
                    let mut ring = Vec::new();
                    for &vk in &base_verts {
                        let v = &self.vertices[vk];
                        let rotated_pos = center + rot * (v.position - center);
                        let new_vk = self.add_vertex_with_uv(rotated_pos, v.uv);
                        ring.push(new_vk);
                    }
                    ring
                };

                // Bridge prev_ring and cur_ring with quad faces
                let n = prev_ring.len();
                for i in 0..n {
                    let next_i = (i + 1) % n;
                    let v0 = prev_ring[i];
                    let v1 = prev_ring[next_i];
                    let v2 = cur_ring[next_i];
                    let v3 = cur_ring[i];

                    if let Ok(fk) = self.add_face(&[v0, v1, v2, v3]) {
                        created_faces.push(fk);
                    }
                }

                prev_ring = cur_ring;
            }

            // If not full circle, add an end cap face
            if !is_full_circle && prev_ring.len() >= 3 {
                if let Ok(fk) = self.add_face(&prev_ring) {
                    created_faces.push(fk);
                }
            }
        }

        self.update_normals();
        Ok(created_faces)
    }
}

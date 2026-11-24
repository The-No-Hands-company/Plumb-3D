use crate::mesh::{Mesh, MeshError, VertexKey};
use crate::modifiers::stack::Modifier;
use core_math::{Quat, Vec3};
use std::collections::HashMap;

/// Array replication mode: Linear translation or Radial / Spiral rotation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ArrayMode {
    #[default]
    Linear,
    Radial,
}

/// Axis of rotation for Radial Array replication.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RadialAxis {
    X,
    #[default]
    Y,
    Z,
}

impl RadialAxis {
    pub fn to_vector(self) -> Vec3 {
        match self {
            RadialAxis::X => Vec3::X,
            RadialAxis::Y => Vec3::Y,
            RadialAxis::Z => Vec3::Z,
        }
    }
}

/// Non-destructive Array Modifier (Blender / 3ds Max style)
/// Duplicates geometry N times linearly or radially around an axis (e.g. merlons, columns, spiral stairs).
#[derive(Debug, Clone)]
pub struct ArrayModifier {
    pub mode: ArrayMode,
    pub count: usize,
    pub constant_offset: Vec3,
    pub relative_offset: Vec3,
    pub scale_step: Vec3,
    pub radial_axis: RadialAxis,
    pub radial_center: Vec3,
    pub total_angle_degrees: f32,
    pub spiral_height_offset: f32,
    pub merge: bool,
    pub merge_distance: f32,
    pub enabled: bool,
}

impl ArrayModifier {
    pub fn new(count: usize) -> Self {
        Self {
            mode: ArrayMode::Linear,
            count,
            constant_offset: Vec3::ZERO,
            relative_offset: Vec3::new(1.0, 0.0, 0.0),
            scale_step: Vec3::ONE,
            radial_axis: RadialAxis::Y,
            radial_center: Vec3::ZERO,
            total_angle_degrees: 360.0,
            spiral_height_offset: 0.0,
            merge: false,
            merge_distance: 0.01,
            enabled: true,
        }
    }

    pub fn new_radial(count: usize, total_angle_degrees: f32) -> Self {
        let mut arr = Self::new(count);
        arr.mode = ArrayMode::Radial;
        arr.total_angle_degrees = total_angle_degrees;
        arr
    }
}

impl Modifier for ArrayModifier {
    fn name(&self) -> &str {
        match self.mode {
            ArrayMode::Linear => "Array (Linear)",
            ArrayMode::Radial => "Array (Radial / Spiral)",
        }
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn as_array_mut(&mut self) -> Option<&mut ArrayModifier> {
        Some(self)
    }

    fn apply(&self, input: &Mesh) -> Result<Mesh, MeshError> {
        if !self.enabled || self.count <= 1 || input.vertices.is_empty() {
            return Ok(input.clone());
        }

        let aabb = input.compute_aabb();
        let extents = aabb.extents();
        let step_offset = self.constant_offset + self.relative_offset * extents;
        let rad_axis = self.radial_axis.to_vector();
        let total_rad = self.total_angle_degrees.to_radians();

        let mut output = Mesh::new();

        // Spatial grid for merging vertices if merge is enabled
        let merge_dist_sq = self.merge_distance * self.merge_distance;
        let cell_size = self.merge_distance.max(1e-4);
        let mut grid: HashMap<(i32, i32, i32), Vec<(VertexKey, Vec3)>> = HashMap::new();

        // Pre-create vertex groups
        for vg in &input.vertex_groups {
            output.add_vertex_group(&vg.name);
        }

        for i in 0..self.count {
            let (rot_q, trans_offset, scale) = match self.mode {
                ArrayMode::Linear => {
                    let t = step_offset * (i as f32);
                    let s = Vec3::new(
                        self.scale_step.x.powi(i as i32),
                        self.scale_step.y.powi(i as i32),
                        self.scale_step.z.powi(i as i32),
                    );
                    (Quat::IDENTITY, t, s)
                }
                ArrayMode::Radial => {
                    let angle = if (self.total_angle_degrees - 360.0).abs() < 1e-3 {
                        total_rad * (i as f32) / (self.count as f32)
                    } else if self.count > 1 {
                        total_rad * (i as f32) / ((self.count - 1) as f32)
                    } else {
                        0.0
                    };
                    let q = Quat::from_axis_angle(rad_axis, angle);
                    let t = rad_axis * (self.spiral_height_offset * (i as f32));
                    (q, t, Vec3::ONE)
                }
            };

            let mut old_to_new = HashMap::new();

            for (vk, v) in &input.vertices {
                let p = match self.mode {
                    ArrayMode::Linear => v.position * scale + trans_offset,
                    ArrayMode::Radial => {
                        let rel = v.position - self.radial_center;
                        rot_q * rel + self.radial_center + trans_offset
                    }
                };

                let mut merged_key = None;
                if self.merge && self.merge_distance > 0.0 {
                    let cx = (p.x / cell_size).floor() as i32;
                    let cy = (p.y / cell_size).floor() as i32;
                    let cz = (p.z / cell_size).floor() as i32;

                    'search: for dx in -1..=1 {
                        for dy in -1..=1 {
                            for dz in -1..=1 {
                                if let Some(candidates) = grid.get(&(cx + dx, cy + dy, cz + dz)) {
                                    for &(cand_k, cand_p) in candidates {
                                        if (cand_p - p).length_squared() <= merge_dist_sq {
                                            merged_key = Some(cand_k);
                                            break 'search;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                let new_vk = match merged_key {
                    Some(k) => k,
                    None => {
                        let k = output.add_vertex_with_uv(p, v.uv);
                        if self.merge && self.merge_distance > 0.0 {
                            let cx = (p.x / cell_size).floor() as i32;
                            let cy = (p.y / cell_size).floor() as i32;
                            let cz = (p.z / cell_size).floor() as i32;
                            grid.entry((cx, cy, cz)).or_default().push((k, p));
                        }
                        k
                    }
                };

                old_to_new.insert(vk, new_vk);

                // Assign vertex weights
                for vg in &input.vertex_groups {
                    let w = vg.get_weight(vk);
                    if w > 0.0 {
                        output.set_vertex_weight(&vg.name, new_vk, w);
                    }
                }
            }

            // Add faces
            for (fk, _) in &input.faces {
                let v_keys = input.face_vertices(fk);
                let new_v_keys: Vec<VertexKey> = v_keys.iter().map(|k| old_to_new[k]).collect();

                if self.merge {
                    let mut unique_keys = Vec::with_capacity(new_v_keys.len());
                    for &vk in &new_v_keys {
                        if !unique_keys.contains(&vk) {
                            unique_keys.push(vk);
                        }
                    }
                    if unique_keys.len() >= 3 {
                        let _ = output.add_face(&unique_keys);
                    }
                } else {
                    output.add_face(&new_v_keys)?;
                }
            }
        }

        output.update_normals();
        Ok(output)
    }
}

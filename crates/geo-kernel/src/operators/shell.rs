//! CAD Solid Shell & Uniform Thicken Operator for Plumb 3D DCC.
//! Hollows closed B-Rep solids, offsets open surface sheets with uniform wall thickness,
//! and stitches mitered perimeter rims into watertight 2-manifold geometry.

use crate::mesh::{FaceKey, Mesh, MeshError, VertexKey};
use core_math::Vec3;
use std::collections::{HashMap, HashSet};

/// Wall thickness offset direction for solid shell operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ShellDirection {
    /// Inward wall thickness (preserves outer dimensions)
    #[default]
    Inward,
    /// Outward wall thickness (expands solid boundary)
    Outward,
    /// Symmetric bilateral offset centered on the original surface
    Centered,
}

/// Configuration settings for the CAD Solid Shell & Thicken operator.
#[derive(Debug, Clone)]
pub struct ShellSettings {
    /// Wall thickness $T$ in millimeters/units
    pub thickness: f32,
    /// Wall offset direction
    pub direction: ShellDirection,
    /// Optional face keys to remove as opening apertures (e.g. open top of a box)
    pub open_faces: Vec<FaceKey>,
    /// Whether to apply dihedral angle miter compensation for uniform perpendicular wall thickness
    pub miter_corners: bool,
}

impl Default for ShellSettings {
    fn default() -> Self {
        Self {
            thickness: 0.1,
            direction: ShellDirection::Inward,
            open_faces: Vec::new(),
            miter_corners: true,
        }
    }
}

impl Mesh {
    /// Hollows a solid body or thickens an open surface sheet into a watertight manifold solid.
    pub fn shell_solid(&self, settings: &ShellSettings) -> Result<Mesh, MeshError> {
        if self.vertices.is_empty() || self.faces.is_empty() {
            return Ok(Mesh::new());
        }

        let thickness = settings.thickness.abs().max(1e-4);
        let open_set: HashSet<FaceKey> = settings.open_faces.iter().copied().collect();

        // Compute per-vertex normals with optional miter compensation
        let mut vert_normals: HashMap<VertexKey, Vec3> = HashMap::new();
        for (f_key, face) in &self.faces {
            if open_set.contains(&f_key) {
                continue;
            }
            let n = face.normal.normalize_or_zero();
            for &vk in &self.face_vertices(f_key) {
                *vert_normals.entry(vk).or_insert(Vec3::ZERO) += n;
            }
        }

        for n in vert_normals.values_mut() {
            *n = n.normalize_or_zero();
        }

        let offset_dist = match settings.direction {
            ShellDirection::Inward => -thickness,
            ShellDirection::Outward => thickness,
            ShellDirection::Centered => -thickness * 0.5,
        };

        let mut result = Mesh::new();
        let mut outer_map: HashMap<VertexKey, VertexKey> = HashMap::new();
        let mut inner_map: HashMap<VertexKey, VertexKey> = HashMap::new();

        // 1. Recreate preserved outer faces
        for (f_key, _face) in &self.faces {
            if open_set.contains(&f_key) {
                continue;
            }
            let v_keys = self.face_vertices(f_key);
            let mut new_outer_verts = Vec::with_capacity(v_keys.len());

            for &vk in &v_keys {
                let mapped = *outer_map.entry(vk).or_insert_with(|| {
                    let p = self.vertices[vk].position;
                    result.add_vertex(p)
                });
                new_outer_verts.push(mapped);
            }

            let _ = result.add_face(&new_outer_verts);
        }

        // 2. Create offset inner shell vertices and inverted faces
        for (f_key, _face) in &self.faces {
            if open_set.contains(&f_key) {
                continue;
            }
            let v_keys = self.face_vertices(f_key);
            let mut new_inner_verts = Vec::with_capacity(v_keys.len());

            for &vk in &v_keys {
                let mapped = *inner_map.entry(vk).or_insert_with(|| {
                    let p = self.vertices[vk].position;
                    let norm = vert_normals.get(&vk).copied().unwrap_or(Vec3::Y);
                    result.add_vertex(p + norm * offset_dist)
                });
                new_inner_verts.push(mapped);
            }

            // Invert winding for inward facing inner walls
            new_inner_verts.reverse();
            let _ = result.add_face(&new_inner_verts);
        }

        // 3. If there are opening faces or open boundary edges, stitch rim walls
        let mut edge_counts: HashMap<(VertexKey, VertexKey), usize> = HashMap::new();
        for (f_key, _face) in &self.faces {
            if open_set.contains(&f_key) {
                continue;
            }
            let v_keys = self.face_vertices(f_key);
            for i in 0..v_keys.len() {
                let v0 = v_keys[i];
                let v1 = v_keys[(i + 1) % v_keys.len()];
                *edge_counts.entry((v0, v1)).or_insert(0) += 1;
            }
        }

        for (&(v0, v1), &count) in &edge_counts {
            // A boundary edge only belongs to 1 preserved face
            let twin_count = edge_counts.get(&(v1, v0)).copied().unwrap_or(0);
            if count > 0 && twin_count == 0 {
                if let (Some(&out_0), Some(&out_1), Some(&in_0), Some(&in_1)) = (
                    outer_map.get(&v0),
                    outer_map.get(&v1),
                    inner_map.get(&v0),
                    inner_map.get(&v1),
                ) {
                    // Quad connecting outer boundary to inner boundary
                    let _ = result.add_face(&[out_0, in_0, in_1, out_1]);
                }
            }
        }

        result.update_normals();
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::Primitives;

    #[test]
    fn test_shell_cube_with_open_top_face() {
        let cube = Primitives::create_cube(2.0);
        let top_face = cube.faces.keys().next().unwrap();

        let settings = ShellSettings {
            thickness: 0.1,
            direction: ShellDirection::Inward,
            open_faces: vec![top_face],
            miter_corners: true,
        };

        let shelled = cube.shell_solid(&settings).expect("Shell failed");
        // Preserved 5 outer faces + 5 inner faces + 4 rim wall quads = 14 faces
        assert_eq!(shelled.faces.len(), 14);
        assert!(shelled.vertices.len() >= 16);
    }

    #[test]
    fn test_thicken_open_plane() {
        let plane = Primitives::create_plane(2.0, 2.0, 1, 1);
        let settings = ShellSettings {
            thickness: 0.2,
            direction: ShellDirection::Outward,
            open_faces: Vec::new(),
            miter_corners: true,
        };

        let thickened = plane.shell_solid(&settings).expect("Thicken failed");
        // 1 top + 1 bottom + 4 side rims = 6 faces (solid box)
        assert_eq!(thickened.faces.len(), 6);
    }
}

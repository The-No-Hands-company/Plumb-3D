use crate::mesh::{FaceKey, Mesh, MeshError, VertexKey};
use crate::operators::{NgonMethod, QuadMethod};
use crate::sculpt::SculptSettings;
use core_math::Vec3;
use std::collections::{HashMap, HashSet};

/// Dynamic topology refinement mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DyntopoMode {
    /// Both split long edges and collapse short edges
    SubdivideCollapse,
    /// Only subdivide edges longer than detail size
    SubdivideOnly,
    /// Only collapse edges shorter than collapse threshold
    CollapseOnly,
}

/// Dynamic topology detail scaling mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DyntopoDetailMode {
    /// Detail size scaled relative to brush radius
    Relative,
    /// Detail size in constant absolute world units
    Constant,
}

/// Configuration settings for dynamic topology sculpting
#[derive(Debug, Clone)]
pub struct DyntopoSettings {
    pub enabled: bool,
    pub mode: DyntopoMode,
    pub detail_mode: DyntopoDetailMode,
    /// Target edge length in world units (Constant) or fraction of radius (Relative)
    pub detail_size: f32,
    /// Fraction of detail_size below which edges are collapsed (typically 0.3 - 0.5)
    pub collapse_ratio: f32,
}

impl Default for DyntopoSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            mode: DyntopoMode::SubdivideCollapse,
            detail_mode: DyntopoDetailMode::Constant,
            detail_size: 0.12,
            collapse_ratio: 0.4,
        }
    }
}

/// Statistics returned by dynamic topology operations
#[derive(Debug, Default, Clone, Copy)]
pub struct DyntopoStats {
    pub edges_split: usize,
    pub edges_collapsed: usize,
    pub vertices_sculpted: usize,
}

impl Mesh {
    /// Dynamically refines mesh topology within a spherical brush region (ZBrush Sculptris Pro / Blender Dyntopo).
    ///
    /// Long edges are subdivided and short edges are collapsed to maintain optimal triangle density
    /// under the brush stroke without requiring manual remeshing.
    pub fn apply_dyntopo(
        &mut self,
        center: Vec3,
        radius: f32,
        settings: &DyntopoSettings,
    ) -> Result<DyntopoStats, MeshError> {
        if !settings.enabled {
            return Ok(DyntopoStats::default());
        }

        let mut stats = DyntopoStats::default();
        let target_len = match settings.detail_mode {
            DyntopoDetailMode::Constant => settings.detail_size.max(0.005),
            DyntopoDetailMode::Relative => (radius * settings.detail_size).max(0.005),
        };
        let min_len = target_len * settings.collapse_ratio.clamp(0.1, 0.9);
        let max_len_sq = target_len * target_len;
        let min_len_sq = min_len * min_len;
        let r_sq = radius * radius * 1.3;

        // 1. Edge Subdivision Phase
        if settings.mode != DyntopoMode::CollapseOnly {
            let mut edges_to_split = Vec::new();
            for (ek, edge) in &self.edges {
                if let Some(he) = self.half_edges.get(edge.half_edge) {
                    if let (Some(v0), Some(v1)) = (self.vertices.get(he.origin), self.vertices.get(he.target)) {
                        let mid = (v0.position + v1.position) * 0.5;
                        if (mid - center).length_squared() <= r_sq {
                            let len_sq = (v0.position - v1.position).length_squared();
                            if len_sq > max_len_sq {
                                edges_to_split.push(ek);
                            }
                        }
                    }
                }
            }

            for ek in edges_to_split {
                if self.edges.contains_key(ek) {
                    if self.split_edge(ek, 0.5).is_ok() {
                        stats.edges_split += 1;
                    }
                }
            }

            // Triangulate faces that were split into quads/n-gons to preserve pure triangle topology
            let faces_to_triangulate: Vec<FaceKey> = self
                .faces
                .iter()
                .filter_map(|(fk, _)| {
                    let verts = self.face_vertices(fk);
                    if verts.len() > 3 {
                        let centroid: Vec3 = verts.iter().map(|&vk| self.vertices[vk].position).sum::<Vec3>()
                            / verts.len() as f32;
                        if (centroid - center).length_squared() <= r_sq {
                            Some(fk)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect();

            let _ = self.triangulate_faces(&faces_to_triangulate, QuadMethod::Beauty, NgonMethod::Beauty, 4);
        }

        // 2. Edge Collapse Phase
        if settings.mode != DyntopoMode::SubdivideOnly {
            let mut edges_to_collapse = Vec::new();
            for (_ek, edge) in &self.edges {
                if let Some(he) = self.half_edges.get(edge.half_edge) {
                    if let (Some(v0), Some(v1)) = (self.vertices.get(he.origin), self.vertices.get(he.target)) {
                        let mid = (v0.position + v1.position) * 0.5;
                        if (mid - center).length_squared() <= r_sq {
                            let len_sq = (v0.position - v1.position).length_squared();
                            if len_sq < min_len_sq {
                                edges_to_collapse.push((he.origin, he.target));
                            }
                        }
                    }
                }
            }

            if !edges_to_collapse.is_empty() {
                let mut remapped: HashMap<VertexKey, VertexKey> = HashMap::new();
                for (va, vb) in edges_to_collapse {
                    let root_a = *remapped.get(&va).unwrap_or(&va);
                    let root_b = *remapped.get(&vb).unwrap_or(&vb);
                    if root_a != root_b {
                        let mid = (self.vertices[root_a].position + self.vertices[root_b].position) * 0.5;
                        self.vertices[root_a].position = mid;
                        remapped.insert(root_b, root_a);
                        stats.edges_collapsed += 1;
                    }
                }

                if stats.edges_collapsed > 0 {
                    // Rebuild faces that reference collapsed vertices
                    let existing_faces: Vec<FaceKey> = self.faces.keys().collect();
                    for fk in existing_faces {
                        let verts = self.face_vertices(fk);
                        let mut new_verts = Vec::with_capacity(verts.len());
                        for &vk in &verts {
                            let target = *remapped.get(&vk).unwrap_or(&vk);
                            if new_verts.last() != Some(&target) {
                                new_verts.push(target);
                            }
                        }
                        if new_verts.len() > 1 && new_verts.first() == new_verts.last() {
                            new_verts.pop();
                        }
                        if new_verts.len() >= 3 {
                            let mut unique_check = HashSet::new();
                            let is_manifold = new_verts.iter().all(|v| unique_check.insert(*v));
                            if is_manifold {
                                let _ = self.add_face(&new_verts);
                            }
                        }
                        self.faces.remove(fk);
                    }
                    let _ = self.weld_vertices_by_distance(0.0005);
                }
            }
        }

        self.update_normals();
        Ok(stats)
    }

    /// Performs an integrated Dynamic Topology sculpt stroke (Refine + Deform in a single atomic pass).
    pub fn apply_dyntopo_sculpt(
        &mut self,
        hit_point: Vec3,
        hit_normal: Vec3,
        sculpt_settings: &SculptSettings,
        dyntopo: &DyntopoSettings,
        motion_delta: Option<Vec3>,
    ) -> Result<DyntopoStats, MeshError> {
        let mut stats = self.apply_dyntopo(hit_point, sculpt_settings.radius, dyntopo)?;
        stats.vertices_sculpted = self.apply_sculpt_brush(
            hit_point,
            hit_normal,
            sculpt_settings,
            motion_delta,
        );
        Ok(stats)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::Primitives;
    use crate::sculpt::{BrushFalloff, SculptBrush};

    #[test]
    fn test_dyntopo_subdivide_long_edges() {
        let mut cube = Primitives::create_cube(2.0);
        let fks: Vec<FaceKey> = cube.faces.keys().collect();
        let _ = cube.triangulate_faces(&fks, QuadMethod::Beauty, NgonMethod::Beauty, 4);
        let initial_verts = cube.vertices.len();

        let settings = DyntopoSettings {
            enabled: true,
            mode: DyntopoMode::SubdivideOnly,
            detail_mode: DyntopoDetailMode::Constant,
            detail_size: 0.8,
            collapse_ratio: 0.4,
        };

        let stats = cube
            .apply_dyntopo(core_math::vec3(0.0, 1.0, 0.0), 1.5, &settings)
            .expect("Dyntopo should succeed");

        assert!(stats.edges_split > 0);
        assert!(cube.vertices.len() > initial_verts);
    }

    #[test]
    fn test_dyntopo_sculpt_stroke_clay() {
        let mut cube = Primitives::create_cube(2.0);
        let fks: Vec<FaceKey> = cube.faces.keys().collect();
        let _ = cube.triangulate_faces(&fks, QuadMethod::Beauty, NgonMethod::Beauty, 4);

        let sculpt_settings = SculptSettings {
            brush: SculptBrush::Clay,
            radius: 1.2,
            strength: 0.5,
            falloff: BrushFalloff::Smooth,
            invert: false,
        };
        let dyntopo_settings = DyntopoSettings {
            enabled: true,
            mode: DyntopoMode::SubdivideCollapse,
            detail_mode: DyntopoDetailMode::Constant,
            detail_size: 0.5,
            collapse_ratio: 0.3,
        };

        let stats = cube
            .apply_dyntopo_sculpt(
                core_math::vec3(0.0, 1.0, 0.0),
                core_math::vec3(0.0, 1.0, 0.0),
                &sculpt_settings,
                &dyntopo_settings,
                None,
            )
            .expect("Dyntopo sculpt stroke should succeed");

        assert!(stats.vertices_sculpted > 0);
    }
}

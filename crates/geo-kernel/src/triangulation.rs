use crate::mesh::Mesh;
use bytemuck::{Pod, Zeroable};

/// GPU-compatible Vertex Buffer Layout for rendering
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct GpuVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    pub color: [f32; 4],
}

/// Material-indexed submesh index buffer range
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SubMeshRange {
    pub material_id: u32,
    pub index_start: u32,
    pub index_count: u32,
}

/// Triangulated CPU buffer ready for GPU upload
#[derive(Debug, Clone, Default)]
pub struct RenderMesh {
    pub vertices: Vec<GpuVertex>,
    pub indices: Vec<u32>,
    /// Line indices for rendering wireframe overlays (pairs of index pointers)
    pub wireframe_indices: Vec<u32>,
    /// Submesh partition ranges mapped by material slot ID
    pub submeshes: Vec<SubMeshRange>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShadingMode {
    Flat,
    Smooth,
}

impl Mesh {
    /// Convert a scalar weight [0.0, 1.0] to a standard DCC heatmap RGBA color:
    /// 0.0: Blue (0, 0, 1) -> 0.25: Cyan (0, 1, 1) -> 0.5: Green (0, 1, 0) -> 0.75: Yellow (1, 1, 0) -> 1.0: Red (1, 0, 0)
    pub fn weight_to_heatmap_rgb(weight: f32) -> [f32; 4] {
        let w = weight.clamp(0.0, 1.0);
        let (r, g, b) = if w < 0.25 {
            let t = w / 0.25;
            (0.0, t, 1.0)
        } else if w < 0.5 {
            let t = (w - 0.25) / 0.25;
            (0.0, 1.0, 1.0 - t)
        } else if w < 0.75 {
            let t = (w - 0.5) / 0.25;
            (t, 1.0, 0.0)
        } else {
            let t = (w - 0.75) / 0.25;
            (1.0, 1.0 - t, 0.0)
        };
        [r, g, b, 1.0]
    }

    /// Tessellate the Half-Edge mesh into GPU-ready vertex and index buffers.
    pub fn to_render_mesh(&self, shading: ShadingMode) -> RenderMesh {
        self.to_render_mesh_with_weights(shading, None)
    }

    /// Tessellate the Half-Edge mesh into GPU-ready vertex and index buffers,
    /// optionally evaluating heatmap vertex colors from a named VertexGroup.
    pub fn to_render_mesh_with_weights(&self, shading: ShadingMode, weight_group: Option<&str>) -> RenderMesh {
        let mut render_mesh = RenderMesh::default();
        let mut first_vk_idx: std::collections::HashMap<crate::mesh::VertexKey, u32> =
            std::collections::HashMap::new();

        let get_color = |vk: crate::mesh::VertexKey| -> [f32; 4] {
            if let Some(vg_name) = weight_group {
                let w = self.get_vertex_weight(vg_name, vk);
                Self::weight_to_heatmap_rgb(w)
            } else if let Some(v) = self.vertices.get(vk) {
                if v.sculpt_mask > 0.001 {
                    let m = v.sculpt_mask.clamp(0.0, 1.0);
                    let mask_shade = 0.22;
                    [
                        v.color[0] * (1.0 - m) + mask_shade * m,
                        v.color[1] * (1.0 - m) + mask_shade * m,
                        v.color[2] * (1.0 - m) + mask_shade * m,
                        v.color[3],
                    ]
                } else {
                    v.color
                }
            } else {
                [0.8, 0.8, 0.8, 1.0]
            }
        };

        let unique_mats = self.unique_material_ids();

        match shading {
            ShadingMode::Flat => {
                for &mat_id in &unique_mats {
                    let start_idx = render_mesh.indices.len() as u32;
                    for (f_key, face) in &self.faces {
                        if face.material_id != mat_id {
                            continue;
                        }
                        let v_keys = self.face_vertices(f_key);
                        if v_keys.len() < 3 {
                            continue;
                        }
                        let normal = face.normal.to_array();
                        let base_idx = render_mesh.vertices.len() as u32;

                        for &vk in &v_keys {
                            let v = &self.vertices[vk];
                            let idx = render_mesh.vertices.len() as u32;
                            first_vk_idx.entry(vk).or_insert(idx);

                            render_mesh.vertices.push(GpuVertex {
                                position: v.position.to_array(),
                                normal,
                                uv: v.uv.to_array(),
                                color: get_color(vk),
                            });
                        }

                        // Simple fan triangulation for N-gons: (0, i, i+1)
                        for i in 1..(v_keys.len() - 1) as u32 {
                            render_mesh.indices.push(base_idx);
                            render_mesh.indices.push(base_idx + i);
                            render_mesh.indices.push(base_idx + i + 1);
                        }
                    }
                    let count = render_mesh.indices.len() as u32 - start_idx;
                    if count > 0 {
                        render_mesh.submeshes.push(SubMeshRange {
                            material_id: mat_id,
                            index_start: start_idx,
                            index_count: count,
                        });
                    }
                }
            }
            ShadingMode::Smooth => {
                let has_sharp = self.edges.values().any(|e| e.is_sharp);
                if has_sharp {
                    for &mat_id in &unique_mats {
                        let start_idx = render_mesh.indices.len() as u32;
                        for (f_key, face) in &self.faces {
                            if face.material_id != mat_id {
                                continue;
                            }
                            let v_keys = self.face_vertices(f_key);
                            if v_keys.len() < 3 {
                                continue;
                            }
                            let base_idx = render_mesh.vertices.len() as u32;

                            for &vk in &v_keys {
                                let v = &self.vertices[vk];
                                let corner_n = self.corner_normal(f_key, vk).to_array();
                                let idx = render_mesh.vertices.len() as u32;
                                first_vk_idx.entry(vk).or_insert(idx);

                                render_mesh.vertices.push(GpuVertex {
                                    position: v.position.to_array(),
                                    normal: corner_n,
                                    uv: v.uv.to_array(),
                                    color: get_color(vk),
                                });
                            }

                            for i in 1..(v_keys.len() - 1) as u32 {
                                render_mesh.indices.push(base_idx);
                                render_mesh.indices.push(base_idx + i);
                                render_mesh.indices.push(base_idx + i + 1);
                            }
                        }
                        let count = render_mesh.indices.len() as u32 - start_idx;
                        if count > 0 {
                            render_mesh.submeshes.push(SubMeshRange {
                                material_id: mat_id,
                                index_start: start_idx,
                                index_count: count,
                            });
                        }
                    }
                } else {
                    for (v_key, v) in &self.vertices {
                        let idx = render_mesh.vertices.len() as u32;
                        render_mesh.vertices.push(GpuVertex {
                            position: v.position.to_array(),
                            normal: v.normal.to_array(),
                            uv: v.uv.to_array(),
                            color: get_color(v_key),
                        });
                        first_vk_idx.insert(v_key, idx);
                    }

                    for &mat_id in &unique_mats {
                        let start_idx = render_mesh.indices.len() as u32;
                        for (f_key, face) in &self.faces {
                            if face.material_id != mat_id {
                                continue;
                            }
                            let v_keys = self.face_vertices(f_key);
                            if v_keys.len() < 3 {
                                continue;
                            }
                            let idx0 = first_vk_idx[&v_keys[0]];
                            for i in 1..(v_keys.len() - 1) {
                                let idx1 = first_vk_idx[&v_keys[i]];
                                let idx2 = first_vk_idx[&v_keys[i + 1]];
                                render_mesh.indices.push(idx0);
                                render_mesh.indices.push(idx1);
                                render_mesh.indices.push(idx2);
                            }
                        }
                        let count = render_mesh.indices.len() as u32 - start_idx;
                        if count > 0 {
                            render_mesh.submeshes.push(SubMeshRange {
                                material_id: mat_id,
                                index_start: start_idx,
                                index_count: count,
                            });
                        }
                    }
                }
            }
        }

        if render_mesh.submeshes.is_empty() && !render_mesh.indices.is_empty() {
            render_mesh.submeshes.push(SubMeshRange {
                material_id: 0,
                index_start: 0,
                index_count: render_mesh.indices.len() as u32,
            });
        }

        // Generate wireframe line indices directly from unique edges referencing existing vertices
        for (_e_key, edge) in &self.edges {
            let he = &self.half_edges[edge.half_edge];
            if let (Some(&idx0), Some(&idx1)) =
                (first_vk_idx.get(&he.origin), first_vk_idx.get(&he.target))
            {
                render_mesh.wireframe_indices.push(idx0);
                render_mesh.wireframe_indices.push(idx1);
            }
        }

        render_mesh
    }
}

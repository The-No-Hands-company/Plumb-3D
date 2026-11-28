use crate::mesh::{EdgeKey, Mesh, MeshError, VertexKey};
use core_math::Vec3;
use std::collections::{HashMap, HashSet};

impl Mesh {
    pub fn slide_edges(&mut self, edges: &[EdgeKey], factor: f32) -> Result<(), MeshError> {
        let edge_set: HashSet<EdgeKey> = edges.iter().copied().collect();
        if edge_set.is_empty() || factor.abs() < 1e-5 {
            return Ok(());
        }

        // Collect all vertices belonging to selected edges
        let mut sliding_verts = HashSet::new();
        for &ek in &edge_set {
            if let Some(edge) = self.edges.get(ek) {
                let he = &self.half_edges[edge.half_edge];
                sliding_verts.insert(he.origin);
                sliding_verts.insert(he.target);
            }
        }

        let vert_list: Vec<VertexKey> = sliding_verts.into_iter().collect();
        self.slide_vertices_internal(&vert_list, &edge_set, factor)
    }

    /// Slides selected vertices along their incident edges by `factor` in [-1.0, 1.0].
    pub fn slide_vertices(&mut self, vertices: &[VertexKey], factor: f32) -> Result<(), MeshError> {
        let empty_edge_set = HashSet::new();
        self.slide_vertices_internal(vertices, &empty_edge_set, factor)
    }

    fn slide_vertices_internal(
        &mut self,
        vertices: &[VertexKey],
        excluded_edges: &HashSet<EdgeKey>,
        factor: f32,
    ) -> Result<(), MeshError> {
        let t = factor.clamp(-0.99, 0.99);
        if t.abs() < 1e-5 {
            return Ok(());
        }

        let mut displacements: HashMap<VertexKey, (Vec3, core_math::Vec2)> = HashMap::new();

        for &vk in vertices {
            if let Some(v) = self.vertices.get(vk) {
                let mut rail_neighbors = Vec::new();
                for &he_k in &v.half_edges {
                    let he = &self.half_edges[he_k];
                    let ek = he.edge;
                    if !excluded_edges.contains(&ek) {
                        let target_vk = if he.origin == vk {
                            he.target
                        } else {
                            he.origin
                        };
                        if target_vk != vk && !rail_neighbors.contains(&target_vk) {
                            rail_neighbors.push(target_vk);
                        }
                    }
                }

                if rail_neighbors.is_empty() {
                    continue;
                }

                // Determine transverse reference direction across the sliding edge loop
                let mut loop_tangent = Vec3::ZERO;
                for &he_k in &v.half_edges {
                    let he = &self.half_edges[he_k];
                    if excluded_edges.contains(&he.edge) {
                        let other_vk = if he.origin == vk {
                            he.target
                        } else {
                            he.origin
                        };
                        if let Some(other_v) = self.vertices.get(other_vk) {
                            let dir = (other_v.position - v.position).normalize_or_zero();
                            if loop_tangent == Vec3::ZERO {
                                loop_tangent = dir;
                            } else {
                                loop_tangent = (loop_tangent - dir).normalize_or_zero();
                            }
                        }
                    }
                }

                let transverse = v.normal.cross(loop_tangent).normalize_or_zero();

                // Select target neighbor consistently aligned with the transverse direction
                let target_neighbor = if rail_neighbors.len() >= 2
                    && transverse.length_squared() > 1e-4
                {
                    if t >= 0.0 {
                        *rail_neighbors
                            .iter()
                            .max_by(|&&a, &&b| {
                                let da = (self.vertices[a].position - v.position).dot(transverse);
                                let db = (self.vertices[b].position - v.position).dot(transverse);
                                da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
                            })
                            .unwrap()
                    } else {
                        *rail_neighbors
                            .iter()
                            .min_by(|&&a, &&b| {
                                let da = (self.vertices[a].position - v.position).dot(transverse);
                                let db = (self.vertices[b].position - v.position).dot(transverse);
                                da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
                            })
                            .unwrap()
                    }
                } else if rail_neighbors.len() >= 2 {
                    if t >= 0.0 {
                        rail_neighbors[0]
                    } else {
                        rail_neighbors[1]
                    }
                } else {
                    rail_neighbors[0]
                };

                let pos_target = self.vertices[target_neighbor].position;
                let uv_target = self.vertices[target_neighbor].uv;
                let alpha = t.abs();
                let new_pos = v.position + (pos_target - v.position) * alpha;
                let new_uv = v.uv + (uv_target - v.uv) * alpha;
                displacements.insert(vk, (new_pos, new_uv));
            }
        }

        for (vk, (new_pos, new_uv)) in displacements {
            if let Some(v) = self.vertices.get_mut(vk) {
                v.position = new_pos;
                v.uv = new_uv;
            }
        }

        self.update_normals();
        Ok(())
    }

    /// Displaces vertices along their computed vertex normals by `offset` (Blender Alt+S Shrink/Fatten).
    pub fn shrink_fatten(&mut self, vertices: &[VertexKey], offset: f32) -> Result<(), MeshError> {
        if offset.abs() < 1e-7 || vertices.is_empty() {
            return Ok(());
        }

        for &vk in vertices {
            if let Some(v) = self.vertices.get_mut(vk) {
                let n = v.normal.normalize_or_zero();
                v.position += n * offset;
            } else {
                return Err(MeshError::InvalidVertex(vk));
            }
        }

        self.update_normals();
        Ok(())
    }

    /// Applies Laplacian umbrella relaxation/smoothing on selected vertices (Laplacian Relax).
    /// `factor`: blend factor towards the centroid of 1-ring neighbors (typically 0.1 - 0.5).
    /// `iterations`: number of smoothing passes.
    pub fn relax_vertices(
        &mut self,
        vertices: &[VertexKey],
        iterations: usize,
        factor: f32,
    ) -> Result<(), MeshError> {
        if vertices.is_empty() || iterations == 0 || factor.abs() < 1e-6 {
            return Ok(());
        }

        for _ in 0..iterations {
            let mut new_positions = Vec::with_capacity(vertices.len());

            for &vk in vertices {
                let v = self.vertices.get(vk).ok_or(MeshError::InvalidVertex(vk))?;
                let neighbors = self.vertex_neighbors(vk);
                if neighbors.is_empty() {
                    continue;
                }

                let mut neighbor_sum = Vec3::ZERO;
                for &n_key in &neighbors {
                    neighbor_sum += self.vertices[n_key].position;
                }
                let centroid = neighbor_sum / (neighbors.len() as f32);
                let relaxed = v.position + (centroid - v.position) * factor.clamp(0.0, 1.0);
                new_positions.push((vk, relaxed));
            }

            for (vk, new_pos) in new_positions {
                if let Some(v) = self.vertices.get_mut(vk) {
                    v.position = new_pos;
                }
            }
        }

        self.update_normals();
        Ok(())
    }
}

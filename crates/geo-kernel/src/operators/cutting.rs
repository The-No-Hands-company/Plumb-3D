use crate::mesh::{FaceKey, Mesh, MeshError, VertexKey};
use core_math::{Vec2, Vec3};
use std::collections::{HashMap, HashSet};

use crate::operators::types::CutPoint;

impl Mesh {
    pub fn bisect(
        &mut self,
        plane_point: Vec3,
        plane_normal: Vec3,
        clear_positive: bool,
        clear_negative: bool,
        fill_cut: bool,
    ) -> Result<(), MeshError> {
        let normal = plane_normal.normalize();
        let eps = 1e-5_f32;

        #[derive(Clone, Copy, PartialEq, Eq, Hash)]
        enum PolyVert {
            Original(VertexKey),
            Cut(VertexKey, VertexKey),
        }

        let mut cut_cache: HashMap<(VertexKey, VertexKey), (Vec3, Vec2)> = HashMap::new();
        let mut cut_edges_for_cap: Vec<(VertexKey, VertexKey)> = Vec::new();

        let dist = |pos: Vec3| -> f32 {
            (pos - plane_point).dot(normal)
        };

        let get_or_calc_cut = |mesh: &Mesh,
                                   u: VertexKey,
                                   v: VertexKey,
                                   d_u: f32,
                                   d_v: f32,
                                   cut_cache: &mut HashMap<(VertexKey, VertexKey), (Vec3, Vec2)>,
                                   cap_edges: &mut Vec<(VertexKey, VertexKey)>|
         -> PolyVert {
            let key = if u < v { (u, v) } else { (v, u) };
            if !cut_cache.contains_key(&key) {
                let t = ((-d_u) / (d_v - d_u)).clamp(0.0001, 0.9999);
                let pos_u = mesh.vertices[u].position;
                let pos_v = mesh.vertices[v].position;
                let uv_u = mesh.vertices[u].uv;
                let uv_v = mesh.vertices[v].uv;
                let cut_pos = pos_u + (pos_v - pos_u) * t;
                let cut_uv = uv_u + (uv_v - uv_u) * t;
                cut_cache.insert(key, (cut_pos, cut_uv));
                cap_edges.push(key);
            }
            PolyVert::Cut(key.0, key.1)
        };

        let mut surviving_polys: Vec<Vec<PolyVert>> = Vec::new();

        for (fk, _) in &self.faces {
            let f_verts = self.face_vertices(fk);
            let m = f_verts.len();
            if m < 3 {
                continue;
            }

            let mut has_pos = false;
            let mut has_neg = false;
            let mut dists = Vec::with_capacity(m);
            for &vk in &f_verts {
                let d = dist(self.vertices[vk].position);
                dists.push(d);
                if d > eps {
                    has_pos = true;
                } else if d < -eps {
                    has_neg = true;
                }
            }

            if !has_pos && !has_neg {
                // Entire face on plane
                if !clear_positive && !clear_negative {
                    surviving_polys.push(f_verts.into_iter().map(PolyVert::Original).collect());
                }
            } else if !has_neg {
                // Pure positive
                if !clear_positive {
                    surviving_polys.push(f_verts.into_iter().map(PolyVert::Original).collect());
                }
            } else if !has_pos {
                // Pure negative
                if !clear_negative {
                    surviving_polys.push(f_verts.into_iter().map(PolyVert::Original).collect());
                }
            } else {
                // Face crosses the plane
                let mut pos_poly: Vec<PolyVert> = Vec::new();
                let mut neg_poly: Vec<PolyVert> = Vec::new();

                for i in 0..m {
                    let curr_vk = f_verts[i];
                    let next_vk = f_verts[(i + 1) % m];
                    let d_curr = dists[i];
                    let d_next = dists[(i + 1) % m];

                    if d_curr >= -eps {
                        pos_poly.push(PolyVert::Original(curr_vk));
                        if d_next < -eps {
                            let cut = get_or_calc_cut(
                                self,
                                curr_vk,
                                next_vk,
                                d_curr,
                                d_next,
                                &mut cut_cache,
                                &mut cut_edges_for_cap,
                            );
                            pos_poly.push(cut);
                        }
                    } else if d_next > eps {
                        let cut = get_or_calc_cut(
                            self,
                            curr_vk,
                            next_vk,
                            d_curr,
                            d_next,
                            &mut cut_cache,
                            &mut cut_edges_for_cap,
                        );
                        pos_poly.push(cut);
                    }

                    if d_curr <= eps {
                        neg_poly.push(PolyVert::Original(curr_vk));
                        if d_next > eps {
                            let cut = get_or_calc_cut(
                                self,
                                curr_vk,
                                next_vk,
                                d_curr,
                                d_next,
                                &mut cut_cache,
                                &mut cut_edges_for_cap,
                            );
                            neg_poly.push(cut);
                        }
                    } else if d_next < -eps {
                        let cut = get_or_calc_cut(
                            self,
                            curr_vk,
                            next_vk,
                            d_curr,
                            d_next,
                            &mut cut_cache,
                            &mut cut_edges_for_cap,
                        );
                        neg_poly.push(cut);
                    }
                }

                if !clear_positive && pos_poly.len() >= 3 {
                    surviving_polys.push(pos_poly);
                }
                if !clear_negative && neg_poly.len() >= 3 {
                    surviving_polys.push(neg_poly);
                }
            }
        }

        // Cap face for cut boundary if one side was cleared
        let mut cap_poly: Option<Vec<PolyVert>> = None;
        if fill_cut && (clear_positive ^ clear_negative) && cut_edges_for_cap.len() >= 3 {
            let mut unique_cut_keys: Vec<(VertexKey, VertexKey)> = Vec::new();
            let mut seen = HashSet::new();
            for key in cut_edges_for_cap {
                if seen.insert(key) {
                    unique_cut_keys.push(key);
                }
            }

            if unique_cut_keys.len() >= 3 {
                let mut centroid = Vec3::ZERO;
                for key in &unique_cut_keys {
                    centroid += cut_cache[key].0;
                }
                centroid /= unique_cut_keys.len() as f32;

                let arbitrary = if normal.y.abs() < 0.9 {
                    Vec3::new(0.0, 1.0, 0.0)
                } else {
                    Vec3::new(1.0, 0.0, 0.0)
                };
                let u_axis = normal.cross(arbitrary).normalize();
                let v_axis = normal.cross(u_axis).normalize();

                unique_cut_keys.sort_by(|&a, &b| {
                    let pos_a = cut_cache[&a].0 - centroid;
                    let pos_b = cut_cache[&b].0 - centroid;
                    let angle_a = (pos_a.dot(v_axis)).atan2(pos_a.dot(u_axis));
                    let angle_b = (pos_b.dot(v_axis)).atan2(pos_b.dot(u_axis));
                    angle_a.partial_cmp(&angle_b).unwrap_or(std::cmp::Ordering::Equal)
                });

                if clear_negative {
                    unique_cut_keys.reverse();
                }

                cap_poly = Some(unique_cut_keys.into_iter().map(|k| PolyVert::Cut(k.0, k.1)).collect());
            }
        }

        // Now build new_mesh with only surviving vertices and polygons
        let mut new_mesh = Mesh::new();
        let mut vert_map: HashMap<PolyVert, VertexKey> = HashMap::new();

        for poly in surviving_polys {
            let mut face_vks = Vec::with_capacity(poly.len());
            for pv in poly {
                let vk = *vert_map.entry(pv).or_insert_with(|| match pv {
                    PolyVert::Original(orig_vk) => {
                        let v = &self.vertices[orig_vk];
                        new_mesh.add_vertex_with_uv(v.position, v.uv)
                    }
                    PolyVert::Cut(u, v) => {
                        let (cut_pos, cut_uv) = cut_cache[&(u, v)];
                        new_mesh.add_vertex_with_uv(cut_pos, cut_uv)
                    }
                });
                face_vks.push(vk);
            }
            new_mesh.add_face(&face_vks)?;
        }

        if let Some(c_poly) = cap_poly {
            let mut face_vks = Vec::with_capacity(c_poly.len());
            for pv in c_poly {
                let vk = *vert_map.entry(pv).or_insert_with(|| match pv {
                    PolyVert::Original(orig_vk) => {
                        let v = &self.vertices[orig_vk];
                        new_mesh.add_vertex_with_uv(v.position, v.uv)
                    }
                    PolyVert::Cut(u, v) => {
                        let (cut_pos, cut_uv) = cut_cache[&(u, v)];
                        new_mesh.add_vertex_with_uv(cut_pos, cut_uv)
                    }
                });
                face_vks.push(vk);
            }
            let _ = new_mesh.add_face(&face_vks);
        }

        // Transfer vertex groups
        for vg in &self.vertex_groups {
            let idx = new_mesh.add_vertex_group(&vg.name);
            for (&vk, &w) in &vg.weights {
                if let Some(&new_vk) = vert_map.get(&PolyVert::Original(vk)) {
                    new_mesh.vertex_groups[idx].set_weight(new_vk, w);
                }
            }
        }

        new_mesh.update_normals();
        *self = new_mesh;
        Ok(())
    }

    /// Triangulates polygon faces in the mesh.
    /// If `faces_to_triangulate` is empty, all faces with >= min_vertices are triangulated.

    pub fn knife_cut(
        &mut self,
        face_key: FaceKey,
        start: CutPoint,
        end: CutPoint,
    ) -> Result<(FaceKey, FaceKey, VertexKey, VertexKey), MeshError> {
        let _ = self.faces.get(face_key).ok_or(MeshError::InvalidFace(face_key))?;

        // 1. Resolve start vertex
        let v_start = match start {
            CutPoint::Vertex(vk) => {
                if !self.vertices.contains_key(vk) {
                    return Err(MeshError::InvalidVertex(vk));
                }
                vk
            }
            CutPoint::Edge(ek, t) => {
                self.split_edge(ek, t.clamp(0.01, 0.99))?
            }
        };

        // 2. Resolve end vertex
        let v_end = match end {
            CutPoint::Vertex(vk) => {
                if !self.vertices.contains_key(vk) {
                    return Err(MeshError::InvalidVertex(vk));
                }
                vk
            }
            CutPoint::Edge(ek, t) => {
                self.split_edge(ek, t.clamp(0.01, 0.99))?
            }
        };

        if v_start == v_end {
            return Err(MeshError::InvariantFailed("Cannot knife cut to the same vertex".into()));
        }

        // 3. Find the face that contains both v_start and v_end
        let target_face = self.faces.iter().find_map(|(fk, _)| {
            let verts = self.face_vertices(fk);
            if verts.contains(&v_start) && verts.contains(&v_end) {
                Some(fk)
            } else {
                None
            }
        }).ok_or_else(|| MeshError::InvariantFailed("Could not find common face containing both cut vertices".into()))?;

        // 4. Connect vertices to split the face
        let (f1, f2) = self.connect_vertices(target_face, v_start, v_end)?;

        Ok((f1, f2, v_start, v_end))
    }
}

use crate::mesh::{FaceKey, Mesh, MeshError, VertexKey};
use crate::operators::types::{triangle_min_angle, NgonMethod, QuadMethod};
use core_math::{Vec2, Vec3};

impl Mesh {
    /// Triangulates polygons with configurable quad diagonal heuristics and n-gon tessellation.
    pub fn triangulate_faces(
        &mut self,
        faces_to_triangulate: &[FaceKey],
        quad_method: QuadMethod,
        ngon_method: NgonMethod,
        min_vertices: usize,
    ) -> Result<Vec<FaceKey>, MeshError> {
        let target_faces: Vec<FaceKey> = if faces_to_triangulate.is_empty() {
            self.faces.keys().collect()
        } else {
            faces_to_triangulate.to_vec()
        };

        let mut faces_to_split: Vec<(FaceKey, Vec<[VertexKey; 3]>)> = Vec::new();

        for fk in target_faces {
            if !self.faces.contains_key(fk) {
                continue;
            }
            let v_keys = self.face_vertices(fk);
            let n = v_keys.len();
            if n < min_vertices || n < 4 {
                continue;
            }

            if n == 4 {
                let v0 = v_keys[0];
                let v1 = v_keys[1];
                let v2 = v_keys[2];
                let v3 = v_keys[3];

                let p0 = self.vertices[v0].position;
                let p1 = self.vertices[v1].position;
                let p2 = self.vertices[v2].position;
                let p3 = self.vertices[v3].position;

                let use_diag_0 = match quad_method {
                    QuadMethod::Fixed => true,
                    QuadMethod::Alternate => false,
                    QuadMethod::ShortestDiagonal => {
                        (p2 - p0).length_squared() <= (p3 - p1).length_squared()
                    }
                    QuadMethod::Beauty => {
                        let a0 =
                            triangle_min_angle(p0, p1, p2).min(triangle_min_angle(p0, p2, p3));
                        let a1 =
                            triangle_min_angle(p0, p1, p3).min(triangle_min_angle(p1, p2, p3));
                        a0 >= a1
                    }
                };

                let tris = if use_diag_0 {
                    vec![[v0, v1, v2], [v0, v2, v3]]
                } else {
                    vec![[v0, v1, v3], [v1, v2, v3]]
                };
                faces_to_split.push((fk, tris));
            } else {
                // N-gon (n >= 5)
                let mut tris = Vec::with_capacity(n - 2);
                let normal = self.faces.get(fk).map(|f| f.normal).unwrap_or(Vec3::Y);

                if ngon_method == NgonMethod::Fan {
                    for i in 1..(n - 1) {
                        tris.push([v_keys[0], v_keys[i], v_keys[i + 1]]);
                    }
                } else {
                    // Ear clipping
                    let arbitrary = if normal.y.abs() < 0.9 {
                        Vec3::Y
                    } else {
                        Vec3::X
                    };
                    let u_axis = normal.cross(arbitrary).normalize();
                    let v_axis = normal.cross(u_axis).normalize();

                    let pts_2d: Vec<Vec2> = v_keys
                        .iter()
                        .map(|&vk| {
                            let p = self.vertices[vk].position;
                            Vec2::new(p.dot(u_axis), p.dot(v_axis))
                        })
                        .collect();

                    let mut area = 0.0_f32;
                    for i in 0..n {
                        let j = (i + 1) % n;
                        area += pts_2d[i].x * pts_2d[j].y - pts_2d[j].x * pts_2d[i].y;
                    }

                    let mut remaining: Vec<usize> = (0..n).collect();
                    if area < 0.0 {
                        remaining.reverse();
                    }

                    let is_point_in_tri_2d = |p: Vec2, a: Vec2, b: Vec2, c: Vec2| -> bool {
                        let cross2d = |v1: Vec2, v2: Vec2| -> f32 { v1.x * v2.y - v1.y * v2.x };
                        let cp1 = cross2d(b - a, p - a);
                        let cp2 = cross2d(c - b, p - b);
                        let cp3 = cross2d(a - c, p - c);
                        (cp1 >= -1e-6 && cp2 >= -1e-6 && cp3 >= -1e-6)
                            || (cp1 <= 1e-6 && cp2 <= 1e-6 && cp3 <= 1e-6)
                    };

                    while remaining.len() > 3 {
                        let r_len = remaining.len();
                        let mut ear_found = false;

                        for i in 0..r_len {
                            let prev_idx = remaining[(i + r_len - 1) % r_len];
                            let curr_idx = remaining[i];
                            let next_idx = remaining[(i + 1) % r_len];

                            let p_prev = pts_2d[prev_idx];
                            let p_curr = pts_2d[curr_idx];
                            let p_next = pts_2d[next_idx];

                            // Convexity test (CCW cross product > 0)
                            let cross = (p_curr.x - p_prev.x) * (p_next.y - p_curr.y)
                                - (p_curr.y - p_prev.y) * (p_next.x - p_curr.x);
                            if cross <= 1e-6 {
                                continue;
                            }

                            // Check if any other vertex lies strictly inside
                            let mut has_inside = false;
                            for &other_idx in &remaining {
                                if other_idx == prev_idx
                                    || other_idx == curr_idx
                                    || other_idx == next_idx
                                {
                                    continue;
                                }
                                if is_point_in_tri_2d(pts_2d[other_idx], p_prev, p_curr, p_next) {
                                    has_inside = true;
                                    break;
                                }
                            }

                            if !has_inside {
                                tris.push([v_keys[prev_idx], v_keys[curr_idx], v_keys[next_idx]]);
                                remaining.remove(i);
                                ear_found = true;
                                break;
                            }
                        }

                        if !ear_found {
                            for i in 1..(remaining.len() - 1) {
                                tris.push([
                                    v_keys[remaining[0]],
                                    v_keys[remaining[i]],
                                    v_keys[remaining[i + 1]],
                                ]);
                            }
                            break;
                        }
                    }

                    if remaining.len() == 3 {
                        tris.push([
                            v_keys[remaining[0]],
                            v_keys[remaining[1]],
                            v_keys[remaining[2]],
                        ]);
                    }
                }
                faces_to_split.push((fk, tris));
            }
        }

        let mut created_faces = Vec::new();

        for (fk, tris) in faces_to_split {
            let face_hes = self.face_half_edges(fk);
            for &he in &face_hes {
                self.half_edges[he].face = None;
            }
            self.faces.remove(fk);

            for tri in tris {
                let new_fk = self.add_face(&tri)?;
                created_faces.push(new_fk);
            }
        }

        self.update_normals();
        Ok(created_faces)
    }

    /// Generates a solid wireframe lattice mesh where every polygon face is replaced
    /// by border quad ribbons of `thickness`, leaving face interiors open.
    pub fn create_wireframe(&self, thickness: f32) -> Result<Mesh, MeshError> {
        let mut wire_mesh = Mesh::new();
        let t = thickness.clamp(0.01, 1.0);

        for (fk, _) in &self.faces {
            let v_keys = self.face_vertices(fk);
            let n = v_keys.len();
            if n < 3 {
                continue;
            }

            let mut center = Vec3::ZERO;
            let mut center_uv = Vec2::ZERO;
            for &vk in &v_keys {
                center += self.vertices[vk].position;
                center_uv += self.vertices[vk].uv;
            }
            center /= n as f32;
            center_uv /= n as f32;

            let factor = (1.0 - t * 0.5).clamp(0.05, 0.95);

            let mut orig_vks = Vec::with_capacity(n);
            let mut inset_vks = Vec::with_capacity(n);

            for &vk in &v_keys {
                let pos = self.vertices[vk].position;
                let uv = self.vertices[vk].uv;
                let inset_pos = center + (pos - center) * factor;
                let inset_uv = center_uv + (uv - center_uv) * factor;

                let o_vk = wire_mesh.add_vertex_with_uv(pos, uv);
                let i_vk = wire_mesh.add_vertex_with_uv(inset_pos, inset_uv);
                orig_vks.push(o_vk);
                inset_vks.push(i_vk);
            }

            for i in 0..n {
                let next = (i + 1) % n;
                let quad = [orig_vks[i], orig_vks[next], inset_vks[next], inset_vks[i]];
                wire_mesh.add_face(&quad)?;
            }
        }

        let _ = wire_mesh.weld_vertices_by_distance(0.001);
        wire_mesh.update_normals();
        Ok(wire_mesh)
    }

    /// Detects boundary loops and fills open holes with polygon caps.
    pub fn cap_holes(&mut self, triangulate: bool) -> Result<Vec<FaceKey>, MeshError> {
        let b_loops = self.boundary_loops();
        if b_loops.is_empty() {
            return Ok(Vec::new());
        }

        let mut created_faces = Vec::new();
        for b_loop in b_loops {
            if b_loop.len() < 3 {
                continue;
            }

            let mut avg_v_normal = Vec3::ZERO;
            for &vk in &b_loop {
                if let Some(v) = self.vertices.get(vk) {
                    avg_v_normal += v.normal;
                }
            }
            avg_v_normal = avg_v_normal.normalize_or_zero();

            let mut loop_vks = b_loop.clone();
            let cap_norm = self.compute_face_normal_newell(&loop_vks);
            if avg_v_normal.length_squared() > 1e-4 && cap_norm.dot(avg_v_normal) < 0.0 {
                loop_vks.reverse();
            }

            if let Ok(fk) = self.add_face(&loop_vks) {
                created_faces.push(fk);
            }
        }

        if triangulate && !created_faces.is_empty() {
            let tri_faces = self.triangulate_faces(
                &created_faces,
                QuadMethod::Beauty,
                NgonMethod::EarClipping,
                4,
            )?;
            self.update_normals();
            Ok(tri_faces)
        } else {
            self.update_normals();
            Ok(created_faces)
        }
    }

    /// Fills an even-length boundary loop with an interior (P x Q) all-quad mesh patch (Blender `Grid Fill`).
    /// Uses transfinite Coons patch interpolation between the four boundary spans.
    pub fn grid_fill(
        &mut self,
        boundary_loop: &[VertexKey],
        span_count: usize,
    ) -> Result<Vec<FaceKey>, MeshError> {
        let n = boundary_loop.len();
        if n < 4 || !n.is_multiple_of(2) {
            return Err(MeshError::DegenerateFace(n));
        }

        let p = if span_count == 0 || span_count >= n / 2 {
            (n / 4).max(1)
        } else {
            span_count
        };
        let q = (n / 2).saturating_sub(p).max(1);

        if 2 * (p + q) != n {
            return Err(MeshError::DegenerateFace(n));
        }

        let mut south = Vec::with_capacity(p + 1);
        for i in 0..=p {
            south.push(boundary_loop[i]);
        }

        let mut east = Vec::with_capacity(q + 1);
        for j in 0..=q {
            east.push(boundary_loop[p + j]);
        }

        let mut north = Vec::with_capacity(p + 1);
        for i in 0..=p {
            north.push(boundary_loop[p + q + (p - i)]);
        }

        let mut west = Vec::with_capacity(q + 1);
        for j in 0..=q {
            west.push(boundary_loop[(2 * p + q + (q - j)) % n]);
        }

        let c00 = self.vertices[south[0]].position;
        let c10 = self.vertices[south[p]].position;
        let c11 = self.vertices[east[q]].position;
        let c01 = self.vertices[north[0]].position;

        let uv00 = self.vertices[south[0]].uv;
        let uv10 = self.vertices[south[p]].uv;
        let uv11 = self.vertices[east[q]].uv;
        let uv01 = self.vertices[north[0]].uv;

        let mut grid: Vec<Vec<VertexKey>> = vec![vec![south[0]; q + 1]; p + 1];

        for i in 0..=p {
            for j in 0..=q {
                if j == 0 {
                    grid[i][j] = south[i];
                } else if j == q {
                    grid[i][j] = north[i];
                } else if i == 0 {
                    grid[i][j] = west[j];
                } else if i == p {
                    grid[i][j] = east[j];
                } else {
                    let u = i as f32 / p as f32;
                    let v = j as f32 / q as f32;

                    let p_south = self.vertices[south[i]].position;
                    let p_north = self.vertices[north[i]].position;
                    let p_west = self.vertices[west[j]].position;
                    let p_east = self.vertices[east[j]].position;

                    let p_corner = (1.0 - u) * (1.0 - v) * c00
                        + u * (1.0 - v) * c10
                        + u * v * c11
                        + (1.0 - u) * v * c01;

                    let pos = (1.0 - v) * p_south + v * p_north + (1.0 - u) * p_west + u * p_east
                        - p_corner;
                    let uv = (1.0 - u) * (1.0 - v) * uv00
                        + u * (1.0 - v) * uv10
                        + u * v * uv11
                        + (1.0 - u) * v * uv01;

                    let new_vk = self.add_vertex_with_uv(pos, uv);
                    grid[i][j] = new_vk;
                }
            }
        }

        let mut created_faces = Vec::new();
        for i in 0..p {
            for j in 0..q {
                let v0 = grid[i][j];
                let v1 = grid[i + 1][j];
                let v2 = grid[i + 1][j + 1];
                let v3 = grid[i][j + 1];

                if let Ok(fk) = self.add_face(&[v0, v1, v2, v3]) {
                    created_faces.push(fk);
                }
            }
        }

        self.update_normals();
        Ok(created_faces)
    }
}

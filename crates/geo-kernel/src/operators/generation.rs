use crate::mesh::{FaceKey, Mesh, MeshError, VertexKey};
use crate::modifiers::MirrorAxis;
use crate::operators::types::{triangle_min_angle, NgonMethod, QuadMethod, SymmetrizeDirection};
use core_math::{Quat, Vec2, Vec3};
use std::collections::{HashMap, HashSet};

impl Mesh {
    pub fn bridge_edge_loops(&mut self, loop_a: &[VertexKey], loop_b: &[VertexKey]) -> Result<Vec<FaceKey>, MeshError> {
        let n = loop_a.len();
        if n < 3 || loop_b.len() != n {
            return Err(MeshError::InvariantFailed("Bridging requires two loops of equal length >= 3".into()));
        }

        let mut created_faces = Vec::with_capacity(n);
        for i in 0..n {
            let a_curr = loop_a[i];
            let a_next = loop_a[(i + 1) % n];
            let b_next = loop_b[(i + 1) % n];
            let b_curr = loop_b[i];

            let fk = self.add_face(&[a_curr, a_next, b_next, b_curr])?;
            created_faces.push(fk);
        }

        self.update_normals();
        Ok(created_faces)
    }

    /// Poke a polygon face by inserting a centroid vertex and splitting into triangles (Blender Alt+P).
    /// Offset pushes the center vertex outward along the face normal.


    pub fn flip_face_normals(&mut self, faces_to_flip: &[FaceKey]) -> Result<(), MeshError> {
        let flip_set: HashSet<FaceKey> = if faces_to_flip.is_empty() {
            self.faces.keys().collect()
        } else {
            faces_to_flip.iter().copied().collect()
        };

        let mut new_mesh = Mesh::new();
        let mut old_to_new_v = std::collections::HashMap::new();

        for (vk, v) in &self.vertices {
            let new_vk = new_mesh.add_vertex_with_uv(v.position, v.uv);
            old_to_new_v.insert(vk, new_vk);
        }

        for (fk, _) in &self.faces {
            let v_keys = self.face_vertices(fk);
            let mut new_v_keys: Vec<VertexKey> = v_keys.iter().map(|k| old_to_new_v[k]).collect();
            if flip_set.contains(&fk) {
                new_v_keys.reverse();
            }
            new_mesh.add_face(&new_v_keys)?;
        }

        for vg in &self.vertex_groups {
            let idx = new_mesh.add_vertex_group(&vg.name);
            for (&vk, &w) in &vg.weights {
                if let Some(&new_vk) = old_to_new_v.get(&vk) {
                    new_mesh.vertex_groups[idx].set_weight(new_vk, w);
                }
            }
        }

        new_mesh.update_normals();
        *self = new_mesh;
        Ok(())
    }

    /// Computes axis-aligned bounding box (min, max) in local mesh space

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
            faces_to_triangulate.iter().copied().collect()
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
                        let a0 = triangle_min_angle(p0, p1, p2).min(triangle_min_angle(p0, p2, p3));
                        let a1 = triangle_min_angle(p0, p1, p3).min(triangle_min_angle(p1, p2, p3));
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
                    let arbitrary = if normal.y.abs() < 0.9 { Vec3::Y } else { Vec3::X };
                    let u_axis = normal.cross(arbitrary).normalize();
                    let v_axis = normal.cross(u_axis).normalize();

                    let pts_2d: Vec<core_math::Vec2> = v_keys.iter().map(|&vk| {
                        let p = self.vertices[vk].position;
                        core_math::Vec2::new(p.dot(u_axis), p.dot(v_axis))
                    }).collect();

                    let mut area = 0.0_f32;
                    for i in 0..n {
                        let j = (i + 1) % n;
                        area += pts_2d[i].x * pts_2d[j].y - pts_2d[j].x * pts_2d[i].y;
                    }

                    let mut remaining: Vec<usize> = (0..n).collect();
                    if area < 0.0 {
                        remaining.reverse();
                    }

                    let is_point_in_tri_2d = |p: core_math::Vec2, a: core_math::Vec2, b: core_math::Vec2, c: core_math::Vec2| -> bool {
                        let cross2d = |v1: core_math::Vec2, v2: core_math::Vec2| -> f32 {
                            v1.x * v2.y - v1.y * v2.x
                        };
                        let cp1 = cross2d(b - a, p - a);
                        let cp2 = cross2d(c - b, p - b);
                        let cp3 = cross2d(a - c, p - c);
                        (cp1 >= -1e-6 && cp2 >= -1e-6 && cp3 >= -1e-6) || (cp1 <= 1e-6 && cp2 <= 1e-6 && cp3 <= 1e-6)
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
                            let cross = (p_curr.x - p_prev.x) * (p_next.y - p_curr.y) - (p_curr.y - p_prev.y) * (p_next.x - p_curr.x);
                            if cross <= 1e-6 {
                                continue;
                            }

                            // Check if any other vertex lies strictly inside
                            let mut has_inside = false;
                            for &other_idx in &remaining {
                                if other_idx == prev_idx || other_idx == curr_idx || other_idx == next_idx {
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
                            // Fallback to fan for remaining
                            for i in 1..(remaining.len() - 1) {
                                tris.push([v_keys[remaining[0]], v_keys[remaining[i]], v_keys[remaining[i + 1]]]);
                            }
                            break;
                        }
                    }

                    if remaining.len() == 3 {
                        tris.push([v_keys[remaining[0]], v_keys[remaining[1]], v_keys[remaining[2]]]);
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

            // Compute face centroid
            let mut center = Vec3::ZERO;
            let mut center_uv = core_math::Vec2::ZERO;
            for &vk in &v_keys {
                center += self.vertices[vk].position;
                center_uv += self.vertices[vk].uv;
            }
            center /= n as f32;
            center_uv /= n as f32;

            // Inset factor based on thickness
            let factor = (1.0 - t * 0.5).clamp(0.05, 0.95);

            // Add original border vertices and inset vertices for this face
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

            // Create quad ribbon border for each edge
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

    /// Bevels (chamfers) vertices by cutting each incident edge at distance `offset`
    /// and inserting a cap polygon at each truncated corner.


    pub fn symmetrize(&mut self, axis: MirrorAxis, direction: SymmetrizeDirection) -> Result<(), MeshError> {
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
        let existing_verts: Vec<(VertexKey, Vec3, Vec2)> = self.vertices.iter()
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
            let mut mirrored_face_verts: Vec<VertexKey> = verts.iter()
                .map(|k| old_to_mirrored_v[k])
                .collect();
            mirrored_face_verts.reverse();
            let _ = self.add_face(&mirrored_face_verts);
        }

        // 4. Weld any slightly off seam vertices
        let _ = self.weld_vertices_by_distance(0.001);
        self.update_normals();

        Ok(())
    }

    /// Cleans degenerate geometry: merges vertices within `min_edge_len`,
    /// removes degenerate sliver faces with area below `min_area`,
    /// and ensures 2-manifold topology without zero-area elements.

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

    /// Returns true if the edge is on an open mesh boundary.

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
        if n < 4 || n % 2 != 0 {
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

                    let pos = (1.0 - v) * p_south + v * p_north + (1.0 - u) * p_west + u * p_east - p_corner;
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

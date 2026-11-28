use crate::mesh::{Mesh, MeshError, VertexKey};
use core_math::{vec2, vec3, Vec2, Vec3};
use std::f32::consts::PI;

use super::types::Spline;

impl Spline {
    /// Revolve (Lathe) a 2D/3D profile around the Y-axis.
    /// `profile_points`: list of (radius, height / Y) coordinates along profile.
    /// `segments`: number of radial slices around the revolution axis.
    /// `angle_radians`: total sweep angle (2*PI for full 360-degree closed revolution).
    pub fn lathe(
        profile_points: &[(f32, f32)],
        segments: u32,
        angle_radians: f32,
    ) -> Result<Mesh, MeshError> {
        if profile_points.len() < 2 {
            return Err(MeshError::DegenerateFace(profile_points.len()));
        }
        let segs = segments.max(3) as usize;
        let is_full_circle = (angle_radians - 2.0 * PI).abs() < 1e-4;
        let slice_count = if is_full_circle { segs } else { segs + 1 };

        let mut mesh = Mesh::new();
        let n_profile = profile_points.len();

        // 1. Generate vertices: grid of [slice_index][profile_index]
        let mut vert_grid: Vec<Vec<VertexKey>> = Vec::with_capacity(slice_count);

        for s in 0..slice_count {
            let theta = (s as f32 / segs as f32) * angle_radians;
            let cos_t = theta.cos();
            let sin_t = theta.sin();
            let u = s as f32 / segs as f32;

            let mut column = Vec::with_capacity(n_profile);
            for (p_idx, &(r, y)) in profile_points.iter().enumerate() {
                let pos = vec3(r * cos_t, y, r * sin_t);
                let v = p_idx as f32 / (n_profile - 1) as f32;
                let vk = mesh.add_vertex_with_uv(pos, vec2(u, v));
                column.push(vk);
            }
            vert_grid.push(column);
        }

        // 2. Generate quad faces between adjacent slices
        let num_slice_quads = if is_full_circle { segs } else { segs };
        for s in 0..num_slice_quads {
            let s_next = (s + 1) % slice_count;
            for p in 0..(n_profile - 1) {
                let v00 = vert_grid[s][p];
                let v01 = vert_grid[s][p + 1];
                let v10 = vert_grid[s_next][p];
                let v11 = vert_grid[s_next][p + 1];

                // Check for degenerate poles (radius == 0.0)
                let r0 = profile_points[p].0.abs();
                let r1 = profile_points[p + 1].0.abs();

                if r0 < 1e-6 {
                    // Top / bottom triangle fan at axis
                    let _ = mesh.add_face(&[v01, v11, v00]);
                } else if r1 < 1e-6 {
                    let _ = mesh.add_face(&[v00, v01, v10]);
                } else {
                    // Standard quad face (counter-clockwise outward)
                    let _ = mesh.add_face(&[v00, v01, v11, v10]);
                }
            }
        }

        // 3. If lathe is not a full 360-degree circle, generate start and end pie caps
        if !is_full_circle {
            // Start pie wedge (reverse winding for outward normal)
            let mut start_cap = vert_grid[0].clone();
            start_cap.reverse();
            let _ = mesh.add_face(&start_cap);

            // End pie wedge (forward winding)
            let end_cap = vert_grid[slice_count - 1].clone();
            let _ = mesh.add_face(&end_cap);
        }

        mesh.update_normals();
        Ok(mesh)
    }

    /// Revolve (Lathe) a 3D Bézier Spline around the Y-axis.
    /// Samples the spline into 2D profile points `(radius, y)` and executes `Spline::lathe`.
    pub fn lathe_spline(
        &self,
        samples_per_segment: usize,
        segments: u32,
        angle_radians: f32,
    ) -> Result<Mesh, MeshError> {
        let sampled = self.sample(samples_per_segment.max(3));
        let profile_pts: Vec<(f32, f32)> = sampled
            .iter()
            .map(|p| {
                let r = (p.x * p.x + p.z * p.z).sqrt();
                (r, p.y)
            })
            .collect();
        Self::lathe(&profile_pts, segments, angle_radians)
    }

    /// Extrudes/Sweeps a 2D cross-section along a 3D path using Rotation Minimizing Frames (RMF / Bishop Frame).
    /// Prevents artificial twisting, gimbal lock, and distortion common with standard Frenet-Serret frames.
    pub fn sweep(
        cross_section: &[Vec2],
        path_points: &[Vec3],
        is_path_closed: bool,
    ) -> Result<Mesh, MeshError> {
        let k = cross_section.len();
        let m = path_points.len();

        if k < 3 {
            return Err(MeshError::DegenerateFace(k));
        }
        if m < 2 {
            return Err(MeshError::DegenerateFace(m));
        }

        let mut mesh = Mesh::new();

        // 1. Compute Rotation Minimizing Frames (RMF) along path points using Double Reflection method
        let mut tangents = Vec::with_capacity(m);
        for i in 0..m {
            let t = if i == 0 {
                if is_path_closed {
                    (path_points[1] - path_points[m - 1]).normalize_or_zero()
                } else {
                    (path_points[1] - path_points[0]).normalize_or_zero()
                }
            } else if i == m - 1 {
                if is_path_closed {
                    (path_points[0] - path_points[m - 2]).normalize_or_zero()
                } else {
                    (path_points[m - 1] - path_points[m - 2]).normalize_or_zero()
                }
            } else {
                (path_points[i + 1] - path_points[i - 1]).normalize_or_zero()
            };
            tangents.push(if t.length_squared() > 1e-4 {
                t
            } else {
                Vec3::Z
            });
        }

        // Initial reference frame at index 0
        let mut normals = Vec::with_capacity(m);
        let mut binormals = Vec::with_capacity(m);

        let t0 = tangents[0];
        let arbitrary_up = if t0.y.abs() < 0.9 { Vec3::Y } else { Vec3::X };
        let mut n0 = t0.cross(arbitrary_up).normalize_or_zero();
        if n0.length_squared() < 1e-4 {
            n0 = Vec3::X;
        }
        let b0 = t0.cross(n0).normalize_or_zero();

        normals.push(n0);
        binormals.push(b0);

        // Propagate frames using double reflection (Wang et al., 2008)
        for i in 0..(m - 1) {
            let p_curr = path_points[i];
            let p_next = path_points[i + 1];
            let t_curr = tangents[i];
            let t_next = tangents[i + 1];

            let v1 = p_next - p_curr;
            let c1 = v1.length_squared();

            if c1 > 1e-8 {
                // First reflection across bisecting plane of points (p_curr -> p_next)
                let n_l = normals[i] - v1 * (2.0 * v1.dot(normals[i]) / c1);
                let t_l = t_curr - v1 * (2.0 * v1.dot(t_curr) / c1);

                // Second reflection across bisecting plane of tangents (t_l -> t_next)
                let v2 = t_next - t_l;
                let c2 = v2.length_squared();

                let n_next = if c2 > 1e-8 {
                    n_l - v2 * (2.0 * v2.dot(n_l) / c2)
                } else {
                    n_l
                };

                let n_next_norm = n_next.normalize_or_zero();
                let b_next_norm = t_next.cross(n_next_norm).normalize_or_zero();
                normals.push(n_next_norm);
                binormals.push(b_next_norm);
            } else {
                normals.push(normals[i]);
                binormals.push(binormals[i]);
            }
        }

        // 2. Instantiate cross-section along path to generate vertex grid
        let mut ring_verts = Vec::with_capacity(m);

        // Calculate total arc length for V coordinate
        let mut arc_lengths = Vec::with_capacity(m);
        let mut total_len = 0.0;
        arc_lengths.push(0.0);
        for i in 1..m {
            total_len += (path_points[i] - path_points[i - 1]).length();
            arc_lengths.push(total_len);
        }
        let max_len = if total_len > 1e-5 { total_len } else { 1.0 };

        for i in 0..m {
            let origin = path_points[i];
            let n_axis = normals[i];
            let b_axis = binormals[i];
            let v_coord = arc_lengths[i] / max_len;

            let mut ring = Vec::with_capacity(k);
            for (j, &cs) in cross_section.iter().enumerate() {
                let pos = origin + n_axis * cs.x + b_axis * cs.y;
                let u_coord = j as f32 / k as f32;
                let vk = mesh.add_vertex_with_uv(pos, vec2(u_coord, v_coord));
                ring.push(vk);
            }
            ring_verts.push(ring);
        }

        // 3. Connect rings with quad faces
        let rings_to_connect = if is_path_closed { m } else { m - 1 };
        for i in 0..rings_to_connect {
            let next_i = (i + 1) % m;
            for j in 0..k {
                let next_j = (j + 1) % k;

                let v00 = ring_verts[i][j];
                let v01 = ring_verts[i][next_j];
                let v11 = ring_verts[next_i][next_j];
                let v10 = ring_verts[next_i][j];

                let _ = mesh.add_face(&[v00, v10, v11, v01]);
            }
        }

        // 4. Cap end caps if path is open
        if !is_path_closed {
            // Start cap: traverses j -> j+1 (twin goes j+1 -> j)
            let start_cap = ring_verts[0].clone();
            let _ = mesh.add_face(&start_cap);

            // End cap: traverses j+1 -> j (twin goes j -> j+1)
            let mut end_cap = ring_verts[m - 1].clone();
            end_cap.reverse();
            let _ = mesh.add_face(&end_cap);
        }

        mesh.update_normals();
        Ok(mesh)
    }

    /// Two-Rail Curve Sweep (Plasticity / Rhino / CAD style).
    /// Sweeps a 2D profile between two guiding rails (`rail1` spine and `rail2` guide rail).
    /// Dynamically scales and rotates the profile cross-section so it spans from rail 1 to rail 2.
    pub fn two_rail_sweep(
        rail1_points: &[Vec3],
        rail2_points: &[Vec3],
        profile_shape: &[Vec2],
        is_closed: bool,
    ) -> Result<Mesh, MeshError> {
        let m = rail1_points.len().min(rail2_points.len());
        let k = profile_shape.len();

        if m < 2 {
            return Err(MeshError::DegenerateFace(m));
        }
        if k < 3 {
            return Err(MeshError::DegenerateFace(k));
        }

        let mut mesh = Mesh::new();
        let mut ring_verts = Vec::with_capacity(m);

        // Normalize profile width to [-0.5, 0.5] if needed, or assume [0, 1] across rails
        let min_x = profile_shape
            .iter()
            .map(|p| p.x)
            .fold(f32::INFINITY, f32::min);
        let max_x = profile_shape
            .iter()
            .map(|p| p.x)
            .fold(f32::NEG_INFINITY, f32::max);
        let span_x = if (max_x - min_x).abs() > 1e-4 {
            max_x - min_x
        } else {
            1.0
        };

        for i in 0..m {
            let p1 = rail1_points[i];
            let p2 = rail2_points[i];
            let across = p2 - p1;
            let width = across.length();
            let x_axis = if width > 1e-5 {
                across / width
            } else {
                Vec3::X
            };

            let t = if i == 0 {
                if is_closed {
                    (rail1_points[1] - rail1_points[m - 1]).normalize_or_zero()
                } else {
                    (rail1_points[1] - rail1_points[0]).normalize_or_zero()
                }
            } else if i == m - 1 {
                if is_closed {
                    (rail1_points[0] - rail1_points[m - 2]).normalize_or_zero()
                } else {
                    (rail1_points[m - 1] - rail1_points[m - 2]).normalize_or_zero()
                }
            } else {
                (rail1_points[i + 1] - rail1_points[i - 1]).normalize_or_zero()
            };

            let tangent = if t.length_squared() > 1e-4 {
                t
            } else {
                Vec3::Z
            };
            let mut y_axis = tangent.cross(x_axis).normalize_or_zero();
            if y_axis.length_squared() < 1e-4 {
                y_axis = Vec3::Y;
            }

            let v_coord = i as f32 / (m - 1).max(1) as f32;
            let mut ring = Vec::with_capacity(k);

            for (j, &ps) in profile_shape.iter().enumerate() {
                // Map normalized x [0.0, 1.0] from p1 to p2
                let norm_u = (ps.x - min_x) / span_x;
                let height = ps.y * width;
                let world_pos = p1 + x_axis * (norm_u * width) + y_axis * height;
                let u_coord = j as f32 / k as f32;
                let vk = mesh.add_vertex_with_uv(world_pos, vec2(u_coord, v_coord));
                ring.push(vk);
            }
            ring_verts.push(ring);
        }

        // Connect rings with quad faces
        let rings_to_connect = if is_closed { m } else { m - 1 };
        for i in 0..rings_to_connect {
            let next_i = (i + 1) % m;
            for j in 0..k {
                let next_j = (j + 1) % k;

                let v00 = ring_verts[i][j];
                let v01 = ring_verts[i][next_j];
                let v11 = ring_verts[next_i][next_j];
                let v10 = ring_verts[next_i][j];

                let _ = mesh.add_face(&[v00, v10, v11, v01]);
            }
        }

        // Cap ends if open
        if !is_closed {
            let start_cap = ring_verts[0].clone();
            let _ = mesh.add_face(&start_cap);

            let mut end_cap = ring_verts[m - 1].clone();
            end_cap.reverse();
            let _ = mesh.add_face(&end_cap);
        }

        mesh.update_normals();
        Ok(mesh)
    }

    /// Multi-Profile Curve Loft: Generates a smooth B-rep surface passing through a sequence
    /// of N >= 2 cross-sectional profile curves.
    /// Each profile curve must contain the same number of vertices K >= 2.
    /// `closed_u`: whether the cross sections themselves are closed loops.
    /// `closed_v`: whether the loft loops back from the last profile to the first profile.
    pub fn loft(profiles: &[Vec<Vec3>], closed_u: bool, closed_v: bool) -> Result<Mesh, MeshError> {
        if profiles.len() < 2 {
            return Err(MeshError::InvariantFailed(
                "Loft requires at least 2 profile curves".into(),
            ));
        }
        let k = profiles[0].len();
        if k < 2 {
            return Err(MeshError::InvariantFailed(
                "Loft profiles must contain at least 2 points".into(),
            ));
        }
        for (idx, p) in profiles.iter().enumerate() {
            if p.len() != k {
                return Err(MeshError::InvariantFailed(format!(
                    "Profile {} has {} points, expected {}",
                    idx,
                    p.len(),
                    k
                )));
            }
        }

        let num_profiles = profiles.len();
        let mut mesh = Mesh::new();
        let mut grid: Vec<Vec<VertexKey>> = Vec::with_capacity(num_profiles);

        // 1. Generate vertices with normalized (u, v) parameterization
        for (i, prof) in profiles.iter().enumerate() {
            let v_coord = i as f32 / (num_profiles - 1).max(1) as f32;
            let mut row = Vec::with_capacity(k);
            for (j, &pt) in prof.iter().enumerate() {
                let u_coord = j as f32 / (k - 1).max(1) as f32;
                let vk = mesh.add_vertex_with_uv(pt, vec2(u_coord, v_coord));
                row.push(vk);
            }
            grid.push(row);
        }

        // 2. Generate quad patches between adjacent profiles
        let num_v_segments = if closed_v {
            num_profiles
        } else {
            num_profiles - 1
        };
        let num_u_segments = if closed_u { k } else { k - 1 };

        for i in 0..num_v_segments {
            let next_i = (i + 1) % num_profiles;
            for j in 0..num_u_segments {
                let next_j = (j + 1) % k;

                let v00 = grid[i][j];
                let v01 = grid[i][next_j];
                let v11 = grid[next_i][next_j];
                let v10 = grid[next_i][j];

                let _ = mesh.add_face(&[v00, v10, v11, v01]);
            }
        }

        // 3. Cap planar ends if profiles are closed loops but loft along V is open
        if closed_u && !closed_v && k >= 3 {
            let start_cap = grid[0].clone();
            let _ = mesh.add_face(&start_cap);

            let mut end_cap = grid[num_profiles - 1].clone();
            end_cap.reverse();
            let _ = mesh.add_face(&end_cap);
        }

        mesh.update_normals();
        Ok(mesh)
    }

    /// Multi-Profile Curve Loft for Spline objects: automatically resamples each profile spline
    /// to a uniform vertex count along its arc length before skinning, ensuring topological
    /// compatibility even when input curves have varying control knot counts.
    pub fn loft_splines(
        splines: &[Spline],
        samples_per_spline: usize,
        closed_v: bool,
    ) -> Result<Mesh, MeshError> {
        if splines.len() < 2 {
            return Err(MeshError::InvariantFailed(
                "Loft requires at least 2 profile splines".into(),
            ));
        }

        let samples = samples_per_spline.max(4);
        let closed_u = splines[0].is_closed;

        let mut sampled_profiles = Vec::with_capacity(splines.len());
        for s in splines {
            let pts = s.sample(samples);
            sampled_profiles.push(pts);
        }

        Self::loft(&sampled_profiles, closed_u, closed_v)
    }

    /// Offsets a 2D polyline or polygon by distance `distance`.
    /// Positive distance expands outward (or left of curve direction), negative offsets inward.
    pub fn offset_polygon_2d(points: &[Vec2], distance: f32, is_closed: bool) -> Vec<Vec2> {
        if points.len() < 2 {
            return points.to_vec();
        }
        let n = points.len();
        let mut result = Vec::with_capacity(n);

        // Compute edge directions and left-pointing normals
        let mut normals = Vec::with_capacity(n);
        let num_edges = if is_closed { n } else { n - 1 };
        for i in 0..num_edges {
            let next_i = (i + 1) % n;
            let dir = (points[next_i] - points[i]).normalize_or_zero();
            let normal = vec2(-dir.y, dir.x); // 90 degrees CCW
            normals.push(normal);
        }

        if is_closed {
            for i in 0..n {
                let prev_edge = (i + n - 1) % n;
                let n1 = normals[prev_edge];
                let n2 = normals[i];
                let bisector = (n1 + n2).normalize_or_zero();
                let cos_half = bisector.dot(n1).max(0.1);
                let miter_len =
                    (distance / cos_half).clamp(-distance.abs() * 3.0, distance.abs() * 3.0);
                result.push(points[i] + bisector * miter_len);
            }
        } else {
            // First point
            result.push(points[0] + normals[0] * distance);
            // Intermediate vertices
            for i in 1..(n - 1) {
                let n1 = normals[i - 1];
                let n2 = normals[i];
                let bisector = (n1 + n2).normalize_or_zero();
                let cos_half = bisector.dot(n1).max(0.1);
                let miter_len =
                    (distance / cos_half).clamp(-distance.abs() * 3.0, distance.abs() * 3.0);
                result.push(points[i] + bisector * miter_len);
            }
            // Last point
            result.push(points[n - 1] + normals[n - 2] * distance);
        }

        result
    }
}

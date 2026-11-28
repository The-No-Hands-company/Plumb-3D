use core_math::{Vec2, Vec3, Workplane};
use geo_kernel::{Mesh, MeshError, MirrorAxis};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CutterShape {
    #[default]
    Box,
    Circle,
    Ngon,
}

pub fn build_box_cutter(
    wp: &Workplane,
    start_uv: Vec2,
    curr_uv: Vec2,
    depth: f32,
) -> Result<Mesh, MeshError> {
    let min_u = start_uv.x.min(curr_uv.x);
    let max_u = start_uv.x.max(curr_uv.x);
    let min_v = start_uv.y.min(curr_uv.y);
    let max_v = start_uv.y.max(curr_uv.y);

    let n = wp.normal;
    let d = depth.abs().max(0.01);
    let eps = 0.05;

    let p00 = wp.uv_to_world(Vec2::new(min_u, min_v)) - n * d;
    let p10 = wp.uv_to_world(Vec2::new(max_u, min_v)) - n * d;
    let p11 = wp.uv_to_world(Vec2::new(max_u, max_v)) - n * d;
    let p01 = wp.uv_to_world(Vec2::new(min_u, max_v)) - n * d;

    let t00 = wp.uv_to_world(Vec2::new(min_u, min_v)) + n * eps;
    let t10 = wp.uv_to_world(Vec2::new(max_u, min_v)) + n * eps;
    let t11 = wp.uv_to_world(Vec2::new(max_u, max_v)) + n * eps;
    let t01 = wp.uv_to_world(Vec2::new(min_u, max_v)) + n * eps;

    let mut mesh = Mesh::new();
    let v0 = mesh.add_vertex(p00);
    let v1 = mesh.add_vertex(p10);
    let v2 = mesh.add_vertex(p11);
    let v3 = mesh.add_vertex(p01);

    let v4 = mesh.add_vertex(t00);
    let v5 = mesh.add_vertex(t10);
    let v6 = mesh.add_vertex(t11);
    let v7 = mesh.add_vertex(t01);

    mesh.add_face(&[v0, v3, v2, v1])?;
    mesh.add_face(&[v4, v5, v6, v7])?;
    mesh.add_face(&[v0, v1, v5, v4])?;
    mesh.add_face(&[v1, v2, v6, v5])?;
    mesh.add_face(&[v2, v3, v7, v6])?;
    mesh.add_face(&[v3, v0, v4, v7])?;

    mesh.update_normals();
    Ok(mesh)
}

pub fn build_circle_cutter(
    wp: &Workplane,
    center_uv: Vec2,
    radius: f32,
    depth: f32,
    segments: usize,
) -> Result<Mesh, MeshError> {
    let n = wp.normal;
    let d = depth.abs().max(0.01);
    let eps = 0.05;
    let r = radius.max(0.01);
    let segs = segments.max(16);

    let mut mesh = Mesh::new();
    let mut bot_verts = Vec::with_capacity(segs);
    let mut top_verts = Vec::with_capacity(segs);

    for i in 0..segs {
        let theta = (i as f32 / segs as f32) * std::f32::consts::TAU;
        let uv = center_uv + Vec2::new(theta.cos() * r, theta.sin() * r);
        let pt_bot = wp.uv_to_world(uv) - n * d;
        let pt_top = wp.uv_to_world(uv) + n * eps;
        bot_verts.push(mesh.add_vertex(pt_bot));
        top_verts.push(mesh.add_vertex(pt_top));
    }

    let mut bot_rev = bot_verts.clone();
    bot_rev.reverse();
    mesh.add_face(&bot_rev)?;
    mesh.add_face(&top_verts)?;

    for i in 0..segs {
        let next = (i + 1) % segs;
        mesh.add_face(&[bot_verts[i], bot_verts[next], top_verts[next], top_verts[i]])?;
    }

    mesh.update_normals();
    Ok(mesh)
}

pub fn build_ngon_cutter(
    wp: &Workplane,
    polygon_uv: &[Vec2],
    depth: f32,
) -> Result<Mesh, MeshError> {
    let n_pts = polygon_uv.len();
    if n_pts < 3 {
        return Err(MeshError::DegenerateFace(n_pts));
    }

    let norm = wp.normal;
    let d = depth.abs().max(0.01);
    let eps = 0.05;

    let mut mesh = Mesh::new();
    let mut bot_verts = Vec::with_capacity(n_pts);
    let mut top_verts = Vec::with_capacity(n_pts);

    for &uv in polygon_uv {
        let pt_bot = wp.uv_to_world(uv) - norm * d;
        let pt_top = wp.uv_to_world(uv) + norm * eps;
        bot_verts.push(mesh.add_vertex(pt_bot));
        top_verts.push(mesh.add_vertex(pt_top));
    }

    // Bottom and top cap polygons
    let mut bot_rev = bot_verts.clone();
    bot_rev.reverse();
    mesh.add_face(&bot_rev)?;
    mesh.add_face(&top_verts)?;

    // Extruded side walls
    for i in 0..n_pts {
        let next = (i + 1) % n_pts;
        mesh.add_face(&[bot_verts[i], bot_verts[next], top_verts[next], top_verts[i]])?;
    }

    mesh.update_normals();
    Ok(mesh)
}

pub fn build_compound_cutter(
    wp: &Workplane,
    shape: CutterShape,
    start_uv: Vec2,
    current_uv: Vec2,
    polygon_points: &[Vec2],
    depth: f32,
    radial_count: usize,
    symmetry_mirror: Option<MirrorAxis>,
) -> Result<Mesh, MeshError> {
    let mut base_cutter = if radial_count <= 1 {
        match shape {
            CutterShape::Box => build_box_cutter(wp, start_uv, current_uv, depth)?,
            CutterShape::Circle => {
                let radius = (current_uv - start_uv).length();
                build_circle_cutter(wp, start_uv, radius, depth, 24)?
            }
            CutterShape::Ngon => {
                let mut pts = polygon_points.to_vec();
                if pts.len() < 3 {
                    // Fallback to triangle
                    pts = vec![start_uv, Vec2::new(current_uv.x, start_uv.y), current_uv];
                }
                build_ngon_cutter(wp, &pts, depth)?
            }
        }
    } else {
        let radius = (current_uv - start_uv).length();
        let r = if radius < 1e-4 { 0.2 } else { radius };
        let n = radial_count;
        let mut compound = Mesh::new();

        let elem_r = (r * 0.22).clamp(0.01, 10.0);
        let box_half = (r * 0.16).clamp(0.01, 10.0);

        for k in 0..n {
            let theta = (k as f32 / n as f32) * std::f32::consts::TAU;
            let v0 = current_uv - start_uv;
            let rotated_offset = if v0.length_squared() > 1e-6 {
                let cos_t = theta.cos();
                let sin_t = theta.sin();
                Vec2::new(v0.x * cos_t - v0.y * sin_t, v0.x * sin_t + v0.y * cos_t)
            } else {
                Vec2::new(theta.cos() * r, theta.sin() * r)
            };
            let elem_center = start_uv + rotated_offset;

            let instance = match shape {
                CutterShape::Circle => build_circle_cutter(wp, elem_center, elem_r, depth, 16)?,
                CutterShape::Box => build_box_cutter(
                    wp,
                    elem_center - Vec2::new(box_half, box_half),
                    elem_center + Vec2::new(box_half, box_half),
                    depth,
                )?,
                CutterShape::Ngon => {
                    let mut pts = Vec::with_capacity(polygon_points.len());
                    for &p in polygon_points {
                        pts.push(elem_center + (p - start_uv));
                    }
                    if pts.len() < 3 {
                        pts = vec![
                            elem_center - Vec2::new(box_half, box_half),
                            elem_center + Vec2::new(box_half, -box_half),
                            elem_center + Vec2::new(0.0, box_half),
                        ];
                    }
                    build_ngon_cutter(wp, &pts, depth)?
                }
            };

            let mut vert_map = HashMap::new();
            for (vk, v) in &instance.vertices {
                let n_vk = compound.add_vertex_with_uv(v.position, v.uv);
                vert_map.insert(vk, n_vk);
            }
            for (fk, _) in &instance.faces {
                let old_f_verts = instance.face_vertices(fk);
                let new_f_verts: Vec<_> = old_f_verts.iter().map(|v| vert_map[v]).collect();
                let _ = compound.add_face(&new_f_verts);
            }
        }

        compound.update_normals();
        compound
    };

    // Apply symmetry mirror if active
    if let Some(axis) = symmetry_mirror {
        let orig_vertices: Vec<(geo_kernel::VertexKey, Vec3, Vec2)> = base_cutter
            .vertices
            .iter()
            .map(|(k, v)| (k, v.position, v.uv))
            .collect();
        let orig_faces: Vec<geo_kernel::FaceKey> = base_cutter.faces.keys().collect();

        let mut mirrored_map = HashMap::new();
        for (vk, pos, uv) in orig_vertices {
            let mirrored_pos = match axis {
                MirrorAxis::X => Vec3::new(-pos.x, pos.y, pos.z),
                MirrorAxis::Y => Vec3::new(pos.x, -pos.y, pos.z),
                MirrorAxis::Z => Vec3::new(pos.x, pos.y, -pos.z),
            };
            let m_vk = base_cutter.add_vertex_with_uv(mirrored_pos, uv);
            mirrored_map.insert(vk, m_vk);
        }

        for fk in orig_faces {
            let verts = base_cutter.face_vertices(fk);
            let mut mirrored_verts: Vec<_> = verts.iter().map(|k| mirrored_map[k]).collect();
            mirrored_verts.reverse(); // Reverse winding for outward normal
            let _ = base_cutter.add_face(&mirrored_verts);
        }
        base_cutter.update_normals();
    }

    Ok(base_cutter)
}

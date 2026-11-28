//! Parametric Curved 3D Primitives (Sphere, Cylinder, Cone, Torus, Capsule, IcoSphere).

use crate::mesh::Mesh;
use core_math::{vec2, vec3, Vec3};
use std::f32::consts::PI;

/// Cap closure style for cylindrical and conical primitives.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CapFillType {
    #[default]
    Ngon,
    TriangleFan,
    None,
}

pub fn create_uv_sphere(radius: f32, segments: u32, rings: u32) -> Mesh {
    let mut mesh = Mesh::new();
    let segs = segments.max(3);
    let rgs = rings.max(2);

    let north_pole = mesh.add_vertex_with_uv(vec3(0.0, radius, 0.0), vec2(0.5, 1.0));
    let mut ring_vertices = Vec::new();

    for r in 1..rgs {
        let phi = PI * (r as f32) / (rgs as f32);
        let y = radius * phi.cos();
        let ring_r = radius * phi.sin();

        let mut current_ring = Vec::new();
        for s in 0..segs {
            let theta = 2.0 * PI * (s as f32) / (segs as f32);
            let x = ring_r * theta.sin();
            let z = ring_r * theta.cos();
            let u = (s as f32) / (segs as f32);
            let v = 1.0 - (r as f32) / (rgs as f32);

            let vk = mesh.add_vertex_with_uv(vec3(x, y, z), vec2(u, v));
            current_ring.push(vk);
        }
        ring_vertices.push(current_ring);
    }

    let south_pole = mesh.add_vertex_with_uv(vec3(0.0, -radius, 0.0), vec2(0.5, 0.0));

    // North pole triangular fan
    for s in 0..segs as usize {
        let next_s = (s + 1) % (segs as usize);
        let v1 = ring_vertices[0][s];
        let v2 = ring_vertices[0][next_s];
        let _ = mesh.add_face(&[north_pole, v1, v2]);
    }

    // Quad bands between rings
    for r in 0..(rgs - 2) as usize {
        for s in 0..segs as usize {
            let next_s = (s + 1) % (segs as usize);
            let v00 = ring_vertices[r][s];
            let v10 = ring_vertices[r][next_s];
            let v11 = ring_vertices[r + 1][next_s];
            let v01 = ring_vertices[r + 1][s];
            let _ = mesh.add_face(&[v00, v01, v11, v10]);
        }
    }

    // South pole triangular fan
    let last_ring = (rgs - 2) as usize;
    for s in 0..segs as usize {
        let next_s = (s + 1) % (segs as usize);
        let v1 = ring_vertices[last_ring][s];
        let v2 = ring_vertices[last_ring][next_s];
        let _ = mesh.add_face(&[south_pole, v2, v1]);
    }

    mesh.update_normals();
    mesh
}

pub fn create_cylinder(radius: f32, height: f32, segments: u32) -> Mesh {
    create_cylinder_parametric(radius, height, segments, CapFillType::Ngon)
}

pub fn create_cylinder_parametric(
    radius: f32,
    height: f32,
    segments: u32,
    cap_fill: CapFillType,
) -> Mesh {
    let mut mesh = Mesh::new();
    let segs = segments.max(3);
    let half_h = height * 0.5;

    let mut top_ring = Vec::with_capacity(segs as usize);
    let mut bottom_ring = Vec::with_capacity(segs as usize);

    for s in 0..segs {
        let theta = 2.0 * PI * (s as f32) / (segs as f32);
        let x = radius * theta.sin();
        let z = radius * theta.cos();
        let u = (s as f32) / (segs as f32);

        let v_top = mesh.add_vertex_with_uv(vec3(x, half_h, z), vec2(u, 1.0));
        let v_bottom = mesh.add_vertex_with_uv(vec3(x, -half_h, z), vec2(u, 0.0));

        top_ring.push(v_top);
        bottom_ring.push(v_bottom);
    }

    // Side quad faces
    for s in 0..segs as usize {
        let next_s = (s + 1) % (segs as usize);
        let _ = mesh.add_face(&[
            top_ring[s],
            bottom_ring[s],
            bottom_ring[next_s],
            top_ring[next_s],
        ]);
    }

    match cap_fill {
        CapFillType::Ngon => {
            let _ = mesh.add_face(&top_ring);
            let mut rev = bottom_ring.clone();
            rev.reverse();
            let _ = mesh.add_face(&rev);
        }
        CapFillType::TriangleFan => {
            let top_center = mesh.add_vertex_with_uv(vec3(0.0, half_h, 0.0), vec2(0.5, 0.5));
            let bot_center = mesh.add_vertex_with_uv(vec3(0.0, -half_h, 0.0), vec2(0.5, 0.5));
            for s in 0..segs as usize {
                let next_s = (s + 1) % (segs as usize);
                let _ = mesh.add_face(&[top_center, top_ring[s], top_ring[next_s]]);
                let _ = mesh.add_face(&[bot_center, bottom_ring[next_s], bottom_ring[s]]);
            }
        }
        CapFillType::None => {}
    }

    mesh.update_normals();
    mesh
}

pub fn create_cone(radius: f32, height: f32, segments: u32) -> Mesh {
    create_cone_parametric(radius, 0.0, height, segments, CapFillType::Ngon)
}

pub fn create_cone_parametric(
    radius_bottom: f32,
    radius_top: f32,
    height: f32,
    segments: u32,
    cap_fill: CapFillType,
) -> Mesh {
    let mut mesh = Mesh::new();
    let segs = segments.max(3) as usize;
    let half_h = height * 0.5;

    let mut base_ring = Vec::with_capacity(segs);
    let mut top_ring = Vec::with_capacity(segs);

    for s in 0..segs {
        let theta = 2.0 * PI * (s as f32) / (segs as f32);
        let cos_t = theta.cos();
        let sin_t = theta.sin();
        let u = (s as f32) / (segs as f32);

        let vb = mesh.add_vertex_with_uv(
            vec3(radius_bottom * cos_t, -half_h, radius_bottom * sin_t),
            vec2(u, 0.0),
        );
        base_ring.push(vb);

        if radius_top > 1e-4 {
            let vt = mesh.add_vertex_with_uv(
                vec3(radius_top * cos_t, half_h, radius_top * sin_t),
                vec2(u, 1.0),
            );
            top_ring.push(vt);
        }
    }

    if radius_top <= 1e-4 {
        let apex = mesh.add_vertex_with_uv(vec3(0.0, half_h, 0.0), vec2(0.5, 1.0));
        for s in 0..segs {
            let next_s = (s + 1) % segs;
            let _ = mesh.add_face(&[apex, base_ring[s], base_ring[next_s]]);
        }
    } else {
        for s in 0..segs {
            let next_s = (s + 1) % segs;
            let _ = mesh.add_face(&[
                top_ring[s],
                base_ring[s],
                base_ring[next_s],
                top_ring[next_s],
            ]);
        }
        if cap_fill == CapFillType::Ngon {
            let _ = mesh.add_face(&top_ring);
        } else if cap_fill == CapFillType::TriangleFan {
            let top_center = mesh.add_vertex_with_uv(vec3(0.0, half_h, 0.0), vec2(0.5, 0.5));
            for s in 0..segs {
                let next_s = (s + 1) % segs;
                let _ = mesh.add_face(&[top_center, top_ring[s], top_ring[next_s]]);
            }
        }
    }

    if cap_fill == CapFillType::Ngon {
        let mut rev = base_ring.clone();
        rev.reverse();
        let _ = mesh.add_face(&rev);
    } else if cap_fill == CapFillType::TriangleFan {
        let bot_center = mesh.add_vertex_with_uv(vec3(0.0, -half_h, 0.0), vec2(0.5, 0.5));
        for s in 0..segs {
            let next_s = (s + 1) % segs;
            let _ = mesh.add_face(&[bot_center, base_ring[next_s], base_ring[s]]);
        }
    }

    mesh.update_normals();
    mesh
}

pub fn create_torus(
    major_radius: f32,
    minor_radius: f32,
    major_segments: u32,
    minor_segments: u32,
) -> Mesh {
    let mut mesh = Mesh::new();
    let maj_segs = major_segments.max(3) as usize;
    let min_segs = minor_segments.max(3) as usize;
    let mut grid = Vec::with_capacity(maj_segs);

    for i in 0..maj_segs {
        let theta = 2.0 * PI * (i as f32) / (maj_segs as f32);
        let cos_theta = theta.cos();
        let sin_theta = theta.sin();
        let u = (i as f32) / (maj_segs as f32);

        let mut ring = Vec::with_capacity(min_segs);
        for j in 0..min_segs {
            let phi = 2.0 * PI * (j as f32) / (min_segs as f32);
            let cos_phi = phi.cos();
            let sin_phi = phi.sin();
            let v = (j as f32) / (min_segs as f32);

            let x = (major_radius + minor_radius * cos_phi) * cos_theta;
            let y = minor_radius * sin_phi;
            let z = (major_radius + minor_radius * cos_phi) * sin_theta;

            let vk = mesh.add_vertex_with_uv(vec3(x, y, z), vec2(u, v));
            ring.push(vk);
        }
        grid.push(ring);
    }

    for i in 0..maj_segs {
        let next_i = (i + 1) % maj_segs;
        for j in 0..min_segs {
            let next_j = (j + 1) % min_segs;
            let v00 = grid[i][j];
            let v01 = grid[i][next_j];
            let v11 = grid[next_i][next_j];
            let v10 = grid[next_i][j];
            let _ = mesh.add_face(&[v00, v10, v11, v01]);
        }
    }

    mesh.update_normals();
    mesh
}

pub fn create_capsule(
    radius: f32,
    cylinder_height: f32,
    segments: u32,
    hemisphere_rings: u32,
) -> Mesh {
    let mut mesh = Mesh::new();
    let segs = segments.max(3) as usize;
    let h_rings = hemisphere_rings.max(2) as usize;
    let half_h = cylinder_height * 0.5;

    let top_pole = mesh.add_vertex_with_uv(vec3(0.0, half_h + radius, 0.0), vec2(0.5, 1.0));
    let mut all_rings = Vec::new();

    for r in 1..h_rings {
        let phi = (r as f32 / h_rings as f32) * (std::f32::consts::FRAC_PI_2);
        let y = half_h + radius * phi.cos();
        let ring_r = radius * phi.sin();
        let v_coord = 0.5 + 0.5 * phi.cos();

        let mut ring = Vec::with_capacity(segs);
        for s in 0..segs {
            let theta = 2.0 * PI * (s as f32) / (segs as f32);
            let vk = mesh.add_vertex_with_uv(
                vec3(ring_r * theta.cos(), y, ring_r * theta.sin()),
                vec2(s as f32 / segs as f32, v_coord),
            );
            ring.push(vk);
        }
        all_rings.push(ring);
    }

    for &h_offset in &[half_h, -half_h] {
        let mut ring = Vec::with_capacity(segs);
        for s in 0..segs {
            let theta = 2.0 * PI * (s as f32) / (segs as f32);
            let vk = mesh.add_vertex_with_uv(
                vec3(radius * theta.cos(), h_offset, radius * theta.sin()),
                vec2(s as f32 / segs as f32, 0.5),
            );
            ring.push(vk);
        }
        all_rings.push(ring);
    }

    for r in 1..h_rings {
        let phi = (r as f32 / h_rings as f32) * (std::f32::consts::FRAC_PI_2);
        let y = -half_h - radius * phi.sin();
        let ring_r = radius * phi.cos();
        let v_coord = 0.5 * (1.0 - phi.sin());

        let mut ring = Vec::with_capacity(segs);
        for s in 0..segs {
            let theta = 2.0 * PI * (s as f32) / (segs as f32);
            let vk = mesh.add_vertex_with_uv(
                vec3(ring_r * theta.cos(), y, ring_r * theta.sin()),
                vec2(s as f32 / segs as f32, v_coord),
            );
            ring.push(vk);
        }
        all_rings.push(ring);
    }

    let bottom_pole = mesh.add_vertex_with_uv(vec3(0.0, -half_h - radius, 0.0), vec2(0.5, 0.0));

    let first_ring = &all_rings[0];
    for s in 0..segs {
        let next_s = (s + 1) % segs;
        let _ = mesh.add_face(&[top_pole, first_ring[s], first_ring[next_s]]);
    }

    for r in 0..(all_rings.len() - 1) {
        let r0 = &all_rings[r];
        let r1 = &all_rings[r + 1];
        for s in 0..segs {
            let next_s = (s + 1) % segs;
            let _ = mesh.add_face(&[r0[s], r1[s], r1[next_s], r0[next_s]]);
        }
    }

    let last_ring = all_rings.last().unwrap();
    for s in 0..segs {
        let next_s = (s + 1) % segs;
        let _ = mesh.add_face(&[bottom_pole, last_ring[next_s], last_ring[s]]);
    }

    mesh.update_normals();
    mesh
}

pub fn create_icosphere(radius: f32, subdivisions: u32) -> Mesh {
    let mut mesh = Mesh::new();
    let phi = (1.0 + (5.0_f32).sqrt()) * 0.5;

    let raw_verts = [
        vec3(-1.0, phi, 0.0),
        vec3(1.0, phi, 0.0),
        vec3(-1.0, -phi, 0.0),
        vec3(1.0, -phi, 0.0),
        vec3(0.0, -1.0, phi),
        vec3(0.0, 1.0, phi),
        vec3(0.0, -1.0, -phi),
        vec3(0.0, 1.0, -phi),
        vec3(phi, 0.0, -1.0),
        vec3(phi, 0.0, 1.0),
        vec3(-phi, 0.0, -1.0),
        vec3(-phi, 0.0, 1.0),
    ];
    let mut positions: Vec<Vec3> = raw_verts.iter().map(|v| v.normalize() * radius).collect();

    let mut faces: Vec<[usize; 3]> = vec![
        [0, 11, 5],
        [0, 5, 1],
        [0, 1, 7],
        [0, 7, 10],
        [0, 10, 11],
        [1, 5, 9],
        [5, 11, 4],
        [11, 10, 2],
        [10, 7, 6],
        [7, 1, 8],
        [3, 9, 4],
        [3, 4, 2],
        [3, 2, 6],
        [3, 6, 8],
        [3, 8, 9],
        [4, 9, 5],
        [2, 4, 11],
        [6, 2, 10],
        [8, 6, 7],
        [9, 8, 1],
    ];

    let mut midpoint_cache = std::collections::HashMap::new();
    let subs = subdivisions.min(4);

    for _ in 0..subs {
        let mut new_faces = Vec::with_capacity(faces.len() * 4);
        midpoint_cache.clear();

        for tri in faces {
            let mut get_midpoint = |i1: usize, i2: usize, positions: &mut Vec<Vec3>| -> usize {
                let key = if i1 < i2 { (i1, i2) } else { (i2, i1) };
                if let Some(&idx) = midpoint_cache.get(&key) {
                    return idx;
                }
                let mid = ((positions[i1] + positions[i2]) * 0.5).normalize() * radius;
                let idx = positions.len();
                positions.push(mid);
                midpoint_cache.insert(key, idx);
                idx
            };

            let a = get_midpoint(tri[0], tri[1], &mut positions);
            let b = get_midpoint(tri[1], tri[2], &mut positions);
            let c = get_midpoint(tri[2], tri[0], &mut positions);

            new_faces.push([tri[0], a, c]);
            new_faces.push([tri[1], b, a]);
            new_faces.push([tri[2], c, b]);
            new_faces.push([a, b, c]);
        }
        faces = new_faces;
    }

    let vertex_keys: Vec<_> = positions
        .iter()
        .map(|&p| {
            let n = p.normalize();
            let u = 0.5 + n.z.atan2(n.x) / (2.0 * PI);
            let v = 0.5 - n.y.asin() / PI;
            mesh.add_vertex_with_uv(p, vec2(u, v))
        })
        .collect();

    for tri in faces {
        let _ = mesh.add_face(&[
            vertex_keys[tri[0]],
            vertex_keys[tri[1]],
            vertex_keys[tri[2]],
        ]);
    }

    mesh.update_normals();
    mesh
}

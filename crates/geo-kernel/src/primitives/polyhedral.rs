//! Polyhedral & Mascot Primitives (Pyramid, Plumb Bob).

use crate::mesh::Mesh;
use core_math::{vec2, vec3};
use std::f32::consts::PI;

pub fn create_pyramid(base_size: f32, height: f32) -> Mesh {
    create_pyramid_parametric(base_size, base_size, height)
}

pub fn create_pyramid_parametric(base_width: f32, base_depth: f32, height: f32) -> Mesh {
    let mut mesh = Mesh::new();
    let hw = base_width * 0.5;
    let hd = base_depth * 0.5;
    let hh = height * 0.5;

    let apex = mesh.add_vertex_with_uv(vec3(0.0, hh, 0.0), vec2(0.5, 1.0));
    let v0 = mesh.add_vertex_with_uv(vec3(-hw, -hh, hd), vec2(0.0, 0.0));
    let v1 = mesh.add_vertex_with_uv(vec3(hw, -hh, hd), vec2(1.0, 0.0));
    let v2 = mesh.add_vertex_with_uv(vec3(hw, -hh, -hd), vec2(1.0, 1.0));
    let v3 = mesh.add_vertex_with_uv(vec3(-hw, -hh, -hd), vec2(0.0, 1.0));

    let _ = mesh.add_face(&[apex, v0, v1]);
    let _ = mesh.add_face(&[apex, v1, v2]);
    let _ = mesh.add_face(&[apex, v2, v3]);
    let _ = mesh.add_face(&[apex, v3, v0]);
    let _ = mesh.add_face(&[v3, v2, v1, v0]);

    mesh.update_normals();
    mesh
}

pub fn create_plumb_bob() -> Mesh {
    create_plumb_bob_detailed(24)
}

pub fn create_plumb_bob_detailed(segments: usize) -> Mesh {
    let mut mesh = Mesh::new();
    let segments = segments.max(8);
    let profile: &[(f32, f32)] = &[
        (0.00, 2.30),
        (0.12, 2.22),
        (0.18, 2.05),
        (0.12, 1.95),
        (0.35, 1.80),
        (0.28, 1.68),
        (0.40, 1.55),
        (0.70, 1.25),
        (1.00, 0.70),
        (0.92, 0.20),
        (0.70, -0.60),
        (0.42, -1.30),
        (0.18, -1.80),
        (0.00, -2.25),
    ];

    let m = profile.len();
    let mut rings: Vec<Vec<crate::mesh::VertexKey>> = Vec::with_capacity(m);

    for (j, &(r, y)) in profile.iter().enumerate() {
        let v = (j as f32) / ((m - 1) as f32);
        let mut ring = Vec::with_capacity(segments);
        if r <= 1e-4 {
            let vk = mesh.add_vertex_with_uv(vec3(0.0, y, 0.0), vec2(0.5, v));
            ring.push(vk);
        } else {
            for i in 0..segments {
                let theta = 2.0 * PI * (i as f32) / (segments as f32);
                let x = r * theta.sin();
                let z = r * theta.cos();
                let u = (i as f32) / (segments as f32);
                let vk = mesh.add_vertex_with_uv(vec3(x, y, z), vec2(u, v));
                ring.push(vk);
            }
        }
        rings.push(ring);
    }

    for j in 0..(m - 1) {
        let r0_is_pole = profile[j].0 <= 1e-4;
        let r1_is_pole = profile[j + 1].0 <= 1e-4;

        if r0_is_pole {
            let north_pole = rings[j][0];
            for s in 0..segments {
                let next_s = (s + 1) % segments;
                let v1 = rings[j + 1][s];
                let v2 = rings[j + 1][next_s];
                let _ = mesh.add_face(&[north_pole, v1, v2]);
            }
        } else if r1_is_pole {
            let south_pole = rings[j + 1][0];
            for s in 0..segments {
                let next_s = (s + 1) % segments;
                let v1 = rings[j][s];
                let v2 = rings[j][next_s];
                let _ = mesh.add_face(&[south_pole, v2, v1]);
            }
        } else {
            for s in 0..segments {
                let next_s = (s + 1) % segments;
                let v00 = rings[j][s];
                let v10 = rings[j][next_s];
                let v11 = rings[j + 1][next_s];
                let v01 = rings[j + 1][s];
                let _ = mesh.add_face(&[v00, v01, v11, v10]);
            }
        }
    }

    mesh.update_normals();
    mesh
}

//! Architectural Geodesic Domes and Precision Helical Pipe Thread Primitives.

use crate::mesh::Mesh;
use core_math::{vec2, vec3, Vec3};
use std::collections::HashMap;

/// Creates an architectural geodesic dome of given radius and icosahedral frequency (1V, 2V, 3V, 4V).
///
/// Truncates the lower hemisphere at y >= 0, creating a grounded geodesic dome with a flat perimeter foundation ring.
pub fn create_geodesic_dome(radius: f32, frequency: u32) -> Mesh {
    let freq = frequency.clamp(1, 6) as usize;
    let mut mesh = Mesh::new();

    // Golden ratio for icosahedron vertices
    let phi = (1.0 + 5.0_f32.sqrt()) * 0.5;
    let ico_verts = [
        vec3(-1.0, phi, 0.0), vec3(1.0, phi, 0.0), vec3(-1.0, -phi, 0.0), vec3(1.0, -phi, 0.0),
        vec3(0.0, -1.0, phi), vec3(0.0, 1.0, phi), vec3(0.0, -1.0, -phi), vec3(0.0, 1.0, -phi),
        vec3(phi, 0.0, -1.0), vec3(phi, 0.0, 1.0), vec3(-phi, 0.0, -1.0), vec3(-phi, 0.0, 1.0),
    ];

    let ico_faces = [
        [0, 11, 5], [0, 5, 1], [0, 1, 7], [0, 7, 10], [0, 10, 11],
        [1, 5, 9], [5, 11, 4], [11, 10, 2], [10, 7, 6], [7, 1, 8],
        [3, 9, 4], [3, 4, 2], [3, 2, 6], [3, 6, 8], [3, 8, 9],
        [4, 9, 5], [2, 4, 11], [6, 2, 10], [8, 6, 7], [9, 8, 1],
    ];

    let mut vert_map: HashMap<[i32; 3], crate::mesh::VertexKey> = HashMap::new();
    let mut get_or_add_vert = |mesh: &mut Mesh, p: Vec3| -> crate::mesh::VertexKey {
        let norm_p = p.normalize() * radius;
        let key = [
            (norm_p.x * 1000.0).round() as i32,
            (norm_p.y * 1000.0).round() as i32,
            (norm_p.z * 1000.0).round() as i32,
        ];
        if let Some(&vk) = vert_map.get(&key) {
            vk
        } else {
            let u = 0.5 + norm_p.z.atan2(norm_p.x) / (2.0 * std::f32::consts::PI);
            let v = 0.5 - (norm_p.y / radius).clamp(-1.0, 1.0).asin() / std::f32::consts::PI;
            let vk = mesh.add_vertex_with_uv(norm_p, vec2(u, v));
            vert_map.insert(key, vk);
            vk
        }
    };

    for face in &ico_faces {
        let v0 = ico_verts[face[0]];
        let v1 = ico_verts[face[1]];
        let v2 = ico_verts[face[2]];

        let mut grid = Vec::with_capacity(freq + 1);
        for i in 0..=freq {
            let mut row = Vec::with_capacity(freq + 1 - i);
            let ti = i as f32 / freq as f32;
            for j in 0..=(freq - i) {
                let tj = j as f32 / freq as f32;
                let tk = 1.0 - ti - tj;
                let pt = v0 * ti + v1 * tj + v2 * tk;
                let vk = get_or_add_vert(&mut mesh, pt);
                row.push(vk);
            }
            grid.push(row);
        }

        for i in 0..freq {
            for j in 0..(freq - i) {
                let p00 = grid[i][j];
                let p10 = grid[i + 1][j];
                let p01 = grid[i][j + 1];

                let y00 = mesh.vertices[p00].position.y;
                let y10 = mesh.vertices[p10].position.y;
                let y01 = mesh.vertices[p01].position.y;

                if y00 >= -0.05 || y10 >= -0.05 || y01 >= -0.05 {
                    let _ = mesh.add_face(&[p00, p10, p01]);
                }

                if j + 1 < freq - i {
                    let p11 = grid[i + 1][j + 1];
                    let y11 = mesh.vertices[p11].position.y;
                    if y10 >= -0.05 || y11 >= -0.05 || y01 >= -0.05 {
                        let _ = mesh.add_face(&[p10, p11, p01]);
                    }
                }
            }
        }
    }

    // Clamp bottom ring vertices to ground plane y = 0.0
    for v in mesh.vertices.values_mut() {
        if v.position.y < 0.08 && v.position.y >= -0.2 {
            v.position.y = 0.0;
        }
    }

    mesh.update_normals();
    mesh
}

/// Creates a precision ISO metric screw / pipe thread cylinder primitive.
///
/// Generates helical 60-degree V-threads with solid circular end caps.
pub fn create_pipe_thread(
    major_diameter: f32,
    pitch: f32,
    thread_length: f32,
    segments_per_turn: u32,
) -> Mesh {
    let mut mesh = Mesh::new();
    let segs = segments_per_turn.clamp(8, 64) as usize;
    let r_major = (major_diameter * 0.5).max(0.1);
    let p = pitch.max(0.05);
    let len = thread_length.max(p);
    let turns = (len / p).ceil() as usize;
    let total_steps = turns * segs;

    // ISO 60-degree metric thread geometry
    // Thread depth H = 0.866025 * P; actual engaged depth ~ 5/8 * H = 0.54127 * P
    let thread_depth = 0.5413 * p;
    let r_root = (r_major - thread_depth).max(0.05);

    let mut flank_top_keys = Vec::with_capacity(total_steps + 1);
    let mut crest_keys = Vec::with_capacity(total_steps + 1);
    let mut flank_bot_keys = Vec::with_capacity(total_steps + 1);

    for s in 0..=total_steps {
        let frac = s as f32 / total_steps as f32;
        let y_base = frac * len;
        let theta = (s as f32 / segs as f32) * std::f32::consts::TAU;
        let cos_t = theta.cos();
        let sin_t = theta.sin();

        let u = (s % segs) as f32 / segs as f32;
        let v = frac;

        let pt_root_bot = vec3(r_root * cos_t, (y_base - p * 0.25).max(0.0), r_root * sin_t);
        let pt_crest = vec3(r_major * cos_t, y_base, r_major * sin_t);
        let pt_root_top = vec3(r_root * cos_t, (y_base + p * 0.25).min(len), r_root * sin_t);

        let k_bot = mesh.add_vertex_with_uv(pt_root_bot, vec2(u, v));
        let k_crest = mesh.add_vertex_with_uv(pt_crest, vec2(u, v));
        let k_top = mesh.add_vertex_with_uv(pt_root_top, vec2(u, v));

        flank_bot_keys.push(k_bot);
        crest_keys.push(k_crest);
        flank_top_keys.push(k_top);
    }

    // Helical quad strips for bottom and top flanks
    for s in 0..total_steps {
        let b0 = flank_bot_keys[s];
        let b1 = flank_bot_keys[s + 1];
        let c0 = crest_keys[s];
        let c1 = crest_keys[s + 1];
        let t0 = flank_top_keys[s];
        let t1 = flank_top_keys[s + 1];

        // Lower thread flank (root to crest)
        let _ = mesh.add_face(&[b0, b1, c1, c0]);
        // Upper thread flank (crest to root)
        let _ = mesh.add_face(&[c0, c1, t1, t0]);
    }

    // Bottom and top cap circles
    let bottom_center = mesh.add_vertex_with_uv(vec3(0.0, 0.0, 0.0), vec2(0.5, 0.5));
    let top_center = mesh.add_vertex_with_uv(vec3(0.0, len, 0.0), vec2(0.5, 0.5));

    for i in 0..segs {
        let next_i = (i + 1) % segs;
        let b0 = flank_bot_keys[i];
        let b1 = flank_bot_keys[next_i];
        let _ = mesh.add_face(&[bottom_center, b1, b0]);

        let top_start = total_steps.saturating_sub(segs);
        let t0 = flank_top_keys[top_start + i];
        let t1 = flank_top_keys[top_start + next_i];
        let _ = mesh.add_face(&[top_center, t0, t1]);
    }

    mesh.update_normals();
    mesh
}

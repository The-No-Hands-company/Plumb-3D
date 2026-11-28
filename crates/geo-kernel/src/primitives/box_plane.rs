//! Parametric Box & Planar Grid primitive generators.

use crate::mesh::Mesh;
use core_math::{vec2, vec3};

/// Generates a standard uniform cube of given size (8 vertices, 12 edges, 6 quad faces).
pub fn create_cube(size: f32) -> Mesh {
    create_box_parametric(size, size, size, 1, 1, 1)
}

/// Generates a fully parametric 3D box with custom dimensions and face subdivisions.
pub fn create_box_parametric(
    width: f32,
    height: f32,
    depth: f32,
    seg_x: u32,
    seg_y: u32,
    seg_z: u32,
) -> Mesh {
    let sx = seg_x.max(1);
    let sy = seg_y.max(1);
    let sz = seg_z.max(1);
    let hw = width * 0.5;
    let hh = height * 0.5;
    let hd = depth * 0.5;

    // Fast path: standard 8-vertex, 6-quad closed 2-manifold box
    if sx == 1 && sy == 1 && sz == 1 {
        let mut mesh = Mesh::new();
        let v0 = mesh.add_vertex_with_uv(vec3(-hw, -hh, -hd), vec2(0.0, 0.0));
        let v1 = mesh.add_vertex_with_uv(vec3(hw, -hh, -hd), vec2(1.0, 0.0));
        let v2 = mesh.add_vertex_with_uv(vec3(hw, hh, -hd), vec2(1.0, 1.0));
        let v3 = mesh.add_vertex_with_uv(vec3(-hw, hh, -hd), vec2(0.0, 1.0));

        let v4 = mesh.add_vertex_with_uv(vec3(-hw, -hh, hd), vec2(0.0, 0.0));
        let v5 = mesh.add_vertex_with_uv(vec3(hw, -hh, hd), vec2(1.0, 0.0));
        let v6 = mesh.add_vertex_with_uv(vec3(hw, hh, hd), vec2(1.0, 1.0));
        let v7 = mesh.add_vertex_with_uv(vec3(-hw, hh, hd), vec2(0.0, 1.0));

        let _ = mesh.add_face(&[v4, v5, v6, v7]);
        let _ = mesh.add_face(&[v1, v0, v3, v2]);
        let _ = mesh.add_face(&[v3, v7, v6, v2]);
        let _ = mesh.add_face(&[v0, v1, v5, v4]);
        let _ = mesh.add_face(&[v5, v1, v2, v6]);
        let _ = mesh.add_face(&[v0, v4, v7, v3]);

        mesh.update_normals();
        return mesh;
    }

    // Subdivided box: generate 6 face grids and weld boundary vertices
    let mut mesh = Mesh::new();
    let mut add_quad_face =
        |origin: [f32; 3], u_axis: [f32; 3], v_axis: [f32; 3], u_segs: u32, v_segs: u32| {
            let mut grid = Vec::with_capacity(((u_segs + 1) * (v_segs + 1)) as usize);

            for vi in 0..=v_segs {
                for ui in 0..=u_segs {
                    let u_frac = (ui as f32) / (u_segs as f32);
                    let v_frac = (vi as f32) / (v_segs as f32);

                    let pos = vec3(
                        origin[0] + u_axis[0] * u_frac + v_axis[0] * v_frac,
                        origin[1] + u_axis[1] * u_frac + v_axis[1] * v_frac,
                        origin[2] + u_axis[2] * u_frac + v_axis[2] * v_frac,
                    );
                    let uv = vec2(u_frac, v_frac);
                    let vk = mesh.add_vertex_with_uv(pos, uv);
                    grid.push(vk);
                }
            }

            for vi in 0..v_segs {
                for ui in 0..u_segs {
                    let row_width = (u_segs + 1) as usize;
                    let v00 = grid[vi as usize * row_width + ui as usize];
                    let v10 = grid[vi as usize * row_width + (ui + 1) as usize];
                    let v11 = grid[(vi + 1) as usize * row_width + (ui + 1) as usize];
                    let v01 = grid[(vi + 1) as usize * row_width + ui as usize];

                    let _ = mesh.add_face(&[v00, v10, v11, v01]);
                }
            }
        };

    // 1. Front (Z+), 2. Back (Z-), 3. Top (Y+), 4. Bottom (Y-), 5. Right (X+), 6. Left (X-)
    add_quad_face(
        [-hw, -hh, hd],
        [width, 0.0, 0.0],
        [0.0, height, 0.0],
        sx,
        sy,
    );
    add_quad_face(
        [hw, -hh, -hd],
        [-width, 0.0, 0.0],
        [0.0, height, 0.0],
        sx,
        sy,
    );
    add_quad_face([-hw, hh, hd], [width, 0.0, 0.0], [0.0, 0.0, -depth], sx, sz);
    add_quad_face(
        [-hw, -hh, -hd],
        [width, 0.0, 0.0],
        [0.0, 0.0, depth],
        sx,
        sz,
    );
    add_quad_face(
        [hw, -hh, hd],
        [0.0, 0.0, -depth],
        [0.0, height, 0.0],
        sz,
        sy,
    );
    add_quad_face(
        [-hw, -hh, -hd],
        [0.0, 0.0, depth],
        [0.0, height, 0.0],
        sz,
        sy,
    );

    if let Ok(welded) = mesh.weld_vertices_by_distance(1e-4) {
        mesh = welded;
    }
    mesh.update_normals();
    mesh
}

/// Create a subdivided planar grid on the XZ ground plane.
pub fn create_plane(width: f32, length: f32, sub_x: u32, sub_z: u32) -> Mesh {
    let mut mesh = Mesh::new();
    let sx = sub_x.max(1);
    let sz = sub_z.max(1);

    let mut grid = Vec::with_capacity(((sx + 1) * (sz + 1)) as usize);

    let start_x = -width * 0.5;
    let start_z = -length * 0.5;
    let step_x = width / (sx as f32);
    let step_z = length / (sz as f32);

    for z in 0..=sz {
        for x in 0..=sx {
            let px = start_x + (x as f32) * step_x;
            let pz = start_z + (z as f32) * step_z;
            let u = (x as f32) / (sx as f32);
            let v = (z as f32) / (sz as f32);

            let vk = mesh.add_vertex_with_uv(vec3(px, 0.0, pz), vec2(u, v));
            grid.push(vk);
        }
    }

    for z in 0..sz {
        for x in 0..sx {
            let row = (sx + 1) as usize;
            let idx00 = (z as usize) * row + (x as usize);
            let idx10 = idx00 + 1;
            let idx01 = idx00 + row;
            let idx11 = idx01 + 1;

            let _ = mesh.add_face(&[grid[idx00], grid[idx01], grid[idx11], grid[idx10]]);
        }
    }

    mesh.update_normals();
    mesh
}

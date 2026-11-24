use crate::mesh::Mesh;
use core_math::{vec2, vec3, Vec3};
use std::f32::consts::PI;

pub struct Primitives;

impl Primitives {
    /// Create a standard 3D Box/Cube with 8 vertices and 6 quad faces.
    /// Topological invariants: V = 8, E = 12, F = 6, Euler characteristic = 2.
    pub fn create_cube(size: f32) -> Mesh {
        let mut mesh = Mesh::new();
        let h = size * 0.5;

        // 8 vertices of cube
        let v0 = mesh.add_vertex_with_uv(vec3(-h, -h, -h), vec2(0.0, 0.0));
        let v1 = mesh.add_vertex_with_uv(vec3(h, -h, -h), vec2(1.0, 0.0));
        let v2 = mesh.add_vertex_with_uv(vec3(h, h, -h), vec2(1.0, 1.0));
        let v3 = mesh.add_vertex_with_uv(vec3(-h, h, -h), vec2(0.0, 1.0));

        let v4 = mesh.add_vertex_with_uv(vec3(-h, -h, h), vec2(0.0, 0.0));
        let v5 = mesh.add_vertex_with_uv(vec3(h, -h, h), vec2(1.0, 0.0));
        let v6 = mesh.add_vertex_with_uv(vec3(h, h, h), vec2(1.0, 1.0));
        let v7 = mesh.add_vertex_with_uv(vec3(-h, h, h), vec2(0.0, 1.0));

        // 6 quad faces with counter-clockwise outward winding
        // Front (Z+)
        mesh.add_face(&[v4, v5, v6, v7]).unwrap();
        // Back (Z-)
        mesh.add_face(&[v1, v0, v3, v2]).unwrap();
        // Top (Y+)
        mesh.add_face(&[v3, v7, v6, v2]).unwrap();
        // Bottom (Y-)
        mesh.add_face(&[v0, v1, v5, v4]).unwrap();
        // Right (X+)
        mesh.add_face(&[v5, v1, v2, v6]).unwrap();
        // Left (X-)
        mesh.add_face(&[v0, v4, v7, v3]).unwrap();

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

        let stride = (sx + 1) as usize;
        for z in 0..sz as usize {
            for x in 0..sx as usize {
                let v00 = grid[z * stride + x];
                let v10 = grid[z * stride + (x + 1)];
                let v11 = grid[(z + 1) * stride + (x + 1)];
                let v01 = grid[(z + 1) * stride + x];

                // CCW winding facing upward (Y+)
                mesh.add_face(&[v00, v01, v11, v10]).unwrap();
            }
        }

        mesh.update_normals();
        mesh
    }

    /// Create a UV Sphere with quad rings and polar caps.
    pub fn create_uv_sphere(radius: f32, segments: u32, rings: u32) -> Mesh {
        let mut mesh = Mesh::new();
        let segs = segments.max(3);
        let rgs = rings.max(2);

        // North pole
        let north_pole = mesh.add_vertex_with_uv(vec3(0.0, radius, 0.0), vec2(0.5, 1.0));

        // Intermediate rings
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

        // South pole
        let south_pole = mesh.add_vertex_with_uv(vec3(0.0, -radius, 0.0), vec2(0.5, 0.0));

        // North pole triangular fan
        for s in 0..segs as usize {
            let next_s = (s + 1) % (segs as usize);
            let v1 = ring_vertices[0][s];
            let v2 = ring_vertices[0][next_s];
            mesh.add_face(&[north_pole, v1, v2]).unwrap();
        }

        // Quad bands between rings
        for r in 0..(rgs - 2) as usize {
            for s in 0..segs as usize {
                let next_s = (s + 1) % (segs as usize);
                let v00 = ring_vertices[r][s];
                let v10 = ring_vertices[r][next_s];
                let v11 = ring_vertices[r + 1][next_s];
                let v01 = ring_vertices[r + 1][s];

                mesh.add_face(&[v00, v01, v11, v10]).unwrap();
            }
        }

        // South pole triangular fan
        let last_ring = (rgs - 2) as usize;
        for s in 0..segs as usize {
            let next_s = (s + 1) % (segs as usize);
            let v1 = ring_vertices[last_ring][s];
            let v2 = ring_vertices[last_ring][next_s];
            mesh.add_face(&[south_pole, v2, v1]).unwrap();
        }

        mesh.update_normals();
        mesh
    }

    /// Create a 3D Cylinder with quad body panels and N-gon top and bottom caps.
    pub fn create_cylinder(radius: f32, height: f32, segments: u32) -> Mesh {
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
            let t0 = top_ring[s];
            let t1 = top_ring[next_s];
            let b0 = bottom_ring[s];
            let b1 = bottom_ring[next_s];

            mesh.add_face(&[t0, b0, b1, t1]).unwrap();
        }

        // Top cap N-gon (CCW when viewed from above)
        mesh.add_face(&top_ring).unwrap();

        // Bottom cap N-gon (reversed CCW when viewed from below)
        let mut reversed_bottom = bottom_ring.clone();
        reversed_bottom.reverse();
        mesh.add_face(&reversed_bottom).unwrap();

        mesh.update_normals();
        mesh
    }

    /// Create a 3D Torus (Donut) with major radius R, minor tube radius r,
    /// and major/minor segment tessellation.
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

        // Connect rings into quads
        for i in 0..maj_segs {
            let next_i = (i + 1) % maj_segs;
            for j in 0..min_segs {
                let next_j = (j + 1) % min_segs;

                let v00 = grid[i][j];
                let v01 = grid[i][next_j];
                let v11 = grid[next_i][next_j];
                let v10 = grid[next_i][j];

                mesh.add_face(&[v00, v10, v11, v01]).unwrap();
            }
        }

        mesh.update_normals();
        mesh
    }

    /// Create a 3D Cone with apex vertex, circular base ring, and base cap.
    pub fn create_cone(radius: f32, height: f32, segments: u32) -> Mesh {
        let mut mesh = Mesh::new();
        let segs = segments.max(3) as usize;
        let half_h = height * 0.5;

        // Apex top vertex
        let apex = mesh.add_vertex_with_uv(vec3(0.0, half_h, 0.0), vec2(0.5, 1.0));

        // Base ring vertices
        let mut base_ring = Vec::with_capacity(segs);
        for s in 0..segs {
            let theta = 2.0 * PI * (s as f32) / (segs as f32);
            let x = radius * theta.cos();
            let z = radius * theta.sin();
            let u = (s as f32) / (segs as f32);

            let vk = mesh.add_vertex_with_uv(vec3(x, -half_h, z), vec2(u, 0.0));
            base_ring.push(vk);
        }

        // Side triangle faces connecting base ring to apex
        for s in 0..segs {
            let next_s = (s + 1) % segs;
            let b0 = base_ring[s];
            let b1 = base_ring[next_s];

            mesh.add_face(&[apex, b0, b1]).unwrap();
        }

        // Bottom cap N-gon
        let mut reversed_base = base_ring.clone();
        reversed_base.reverse();
        mesh.add_face(&reversed_base).unwrap();

        mesh.update_normals();
        mesh
    }

    /// Create a 3D Pyramid with apex vertex, square base, and bottom cap.
    pub fn create_pyramid(base_size: f32, height: f32) -> Mesh {
        let mut mesh = Mesh::new();
        let half_b = base_size * 0.5;
        let half_h = height * 0.5;

        // Apex top vertex
        let apex = mesh.add_vertex_with_uv(vec3(0.0, half_h, 0.0), vec2(0.5, 1.0));

        // Base 4 vertices (CCW when viewed from above)
        let v0 = mesh.add_vertex_with_uv(vec3(-half_b, -half_h, half_b), vec2(0.0, 0.0));
        let v1 = mesh.add_vertex_with_uv(vec3(half_b, -half_h, half_b), vec2(1.0, 0.0));
        let v2 = mesh.add_vertex_with_uv(vec3(half_b, -half_h, -half_b), vec2(1.0, 1.0));
        let v3 = mesh.add_vertex_with_uv(vec3(-half_b, -half_h, -half_b), vec2(0.0, 1.0));

        // 4 side triangles
        mesh.add_face(&[apex, v0, v1]).unwrap();
        mesh.add_face(&[apex, v1, v2]).unwrap();
        mesh.add_face(&[apex, v2, v3]).unwrap();
        mesh.add_face(&[apex, v3, v0]).unwrap();

        // Bottom square cap (facing downwards)
        mesh.add_face(&[v3, v2, v1, v0]).unwrap();

        mesh.update_normals();
        mesh
    }

    /// Create a 3D Capsule / Pill mesh with cylindrical body and two hemispherical caps.
    pub fn create_capsule(radius: f32, cylinder_height: f32, segments: u32, hemisphere_rings: u32) -> Mesh {
        let mut mesh = Mesh::new();
        let segs = segments.max(3) as usize;
        let h_rings = hemisphere_rings.max(2) as usize;
        let half_h = cylinder_height * 0.5;

        // Top pole vertex
        let top_pole = mesh.add_vertex_with_uv(vec3(0.0, half_h + radius, 0.0), vec2(0.5, 1.0));

        // Generate rings from top hemisphere down to bottom hemisphere
        let mut all_rings = Vec::new();

        // 1. Top hemisphere rings (excluding pole and equator)
        for r in 1..h_rings {
            let phi = (r as f32 / h_rings as f32) * (std::f32::consts::FRAC_PI_2);
            let y = half_h + radius * phi.cos();
            let ring_r = radius * phi.sin();
            let v_coord = 0.5 + 0.5 * phi.cos();

            let mut ring = Vec::with_capacity(segs);
            for s in 0..segs {
                let theta = 2.0 * PI * (s as f32) / (segs as f32);
                let x = ring_r * theta.cos();
                let z = ring_r * theta.sin();
                let u_coord = (s as f32) / (segs as f32);
                let vk = mesh.add_vertex_with_uv(vec3(x, y, z), vec2(u_coord, v_coord));
                ring.push(vk);
            }
            all_rings.push(ring);
        }

        // 2. Cylinder top rim (at +half_h)
        {
            let mut ring = Vec::with_capacity(segs);
            for s in 0..segs {
                let theta = 2.0 * PI * (s as f32) / (segs as f32);
                let x = radius * theta.cos();
                let z = radius * theta.sin();
                let u_coord = (s as f32) / (segs as f32);
                let vk = mesh.add_vertex_with_uv(vec3(x, half_h, z), vec2(u_coord, 0.5));
                ring.push(vk);
            }
            all_rings.push(ring);
        }

        // 3. Cylinder bottom rim (at -half_h)
        {
            let mut ring = Vec::with_capacity(segs);
            for s in 0..segs {
                let theta = 2.0 * PI * (s as f32) / (segs as f32);
                let x = radius * theta.cos();
                let z = radius * theta.sin();
                let u_coord = (s as f32) / (segs as f32);
                let vk = mesh.add_vertex_with_uv(vec3(x, -half_h, z), vec2(u_coord, 0.5));
                ring.push(vk);
            }
            all_rings.push(ring);
        }

        // 4. Bottom hemisphere rings
        for r in 1..h_rings {
            let phi = (r as f32 / h_rings as f32) * (std::f32::consts::FRAC_PI_2);
            let y = -half_h - radius * phi.sin();
            let ring_r = radius * phi.cos();
            let v_coord = 0.5 * (1.0 - phi.sin());

            let mut ring = Vec::with_capacity(segs);
            for s in 0..segs {
                let theta = 2.0 * PI * (s as f32) / (segs as f32);
                let x = ring_r * theta.cos();
                let z = ring_r * theta.sin();
                let u_coord = (s as f32) / (segs as f32);
                let vk = mesh.add_vertex_with_uv(vec3(x, y, z), vec2(u_coord, v_coord));
                ring.push(vk);
            }
            all_rings.push(ring);
        }

        // Bottom pole vertex
        let bottom_pole = mesh.add_vertex_with_uv(vec3(0.0, -half_h - radius, 0.0), vec2(0.5, 0.0));

        // Connect top pole to first ring with triangles
        let first_ring = &all_rings[0];
        for s in 0..segs {
            let next_s = (s + 1) % segs;
            let _ = mesh.add_face(&[top_pole, first_ring[s], first_ring[next_s]]);
        }

        // Connect adjacent rings with quads
        for r in 0..(all_rings.len() - 1) {
            let r0 = &all_rings[r];
            let r1 = &all_rings[r + 1];
            for s in 0..segs {
                let next_s = (s + 1) % segs;
                let _ = mesh.add_face(&[r0[s], r1[s], r1[next_s], r0[next_s]]);
            }
        }

        // Connect last ring to bottom pole with triangles
        let last_ring = all_rings.last().unwrap();
        for s in 0..segs {
            let next_s = (s + 1) % segs;
            let _ = mesh.add_face(&[bottom_pole, last_ring[next_s], last_ring[s]]);
        }

        mesh.update_normals();
        mesh
    }

    /// Create an IcoSphere with uniform equilateral geodesic triangles (no polar distortion).
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

        let raw_faces: Vec<[usize; 3]> = vec![
            [0, 11, 5], [0, 5, 1], [0, 1, 7], [0, 7, 10], [0, 10, 11],
            [1, 5, 9], [5, 11, 4], [11, 10, 2], [10, 7, 6], [7, 1, 8],
            [3, 9, 4], [3, 4, 2], [3, 2, 6], [3, 6, 8], [3, 8, 9],
            [4, 9, 5], [2, 4, 11], [6, 2, 10], [8, 6, 7], [9, 8, 1],
        ];

        let mut faces = raw_faces;

        // Subdivide recursively
        let mut midpoint_cache = std::collections::HashMap::new();
        let subs = subdivisions.min(4); // limit to prevent memory explosion

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

        // Add all positions to mesh
        let vertex_keys: Vec<_> = positions
            .iter()
            .map(|&p| {
                let n = p.normalize();
                let u = 0.5 + n.z.atan2(n.x) / (2.0 * PI);
                let v = 0.5 - n.y.asin() / PI;
                mesh.add_vertex_with_uv(p, vec2(u, v))
            })
            .collect();

        // Add faces
        for tri in faces {
            let _ = mesh.add_face(&[vertex_keys[tri[0]], vertex_keys[tri[1]], vertex_keys[tri[2]]]);
        }

        mesh.update_normals();
        mesh
    }

    /// Create the signature mascot primitive for Plumb 3D: a high-precision architectural Plumb Bob.
    /// Features an ornamental suspension finial, brass knurled collar, conical body, and precision needle tip.
    pub fn create_plumb_bob() -> Mesh {
        Self::create_plumb_bob_detailed(24)
    }

    /// Create high-precision Plumb Bob mascot with custom circumferential tessellation.
    pub fn create_plumb_bob_detailed(segments: usize) -> Mesh {
        let mut mesh = Mesh::new();
        let segments = segments.max(8);
        let profile: &[(f32, f32)] = &[
            (0.00, 2.30),  // Top finial peak (North pole)
            (0.12, 2.22),  // Suspension eye ring top
            (0.18, 2.05),  // Suspension cap rim
            (0.12, 1.95),  // Threaded neck
            (0.35, 1.80),  // Upper brass collar
            (0.28, 1.68),  // Recessed grip groove
            (0.40, 1.55),  // Collar shoulder
            (0.70, 1.25),  // Body upper chamfer
            (1.00, 0.70),  // Widest plumb bob diameter
            (0.92, 0.20),  // Mid taper
            (0.70, -0.60), // Lower conical taper
            (0.42, -1.30), // Tip collar
            (0.18, -1.80), // Steel tip shank
            (0.00, -2.25), // Precision needle tip (South pole, true vertical origin)
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

        // Generate faces between consecutive rings
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
}

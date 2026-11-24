//! 3D Voxel Remesher (Dual-Contouring / Marching Cubes SDF).
//! Converts arbitrary meshes into uniform, watertight, 2-manifold BMesh surfaces.

use crate::mesh::{Mesh, MeshError};
use crate::operators::remesh_tables::{CORNER_OFFSETS, EDGE_TABLE, EDGE_VERTICES, TRI_TABLE};
use core_math::{closest_point_on_triangle, Vec3};
use rayon::prelude::*;

/// A simple planar triangle representation for SDF computation.
#[derive(Debug, Clone, Copy)]
struct SimpTri {
    v0: Vec3,
    v1: Vec3,
    v2: Vec3,
    normal: Vec3,
}

impl Mesh {
    /// Voxel Remesh (DynaMesh): Converts the input mesh into a clean, uniform manifold quad/triangle mesh
    /// by sampling an exact Signed Distance Field (SDF) and extracting the zero-level isosurface via Marching Cubes.
    pub fn voxel_remesh(&self, voxel_size: f32) -> Result<Mesh, MeshError> {
        if self.vertices.is_empty() || self.faces.is_empty() {
            return Ok(self.clone());
        }

        let vs = voxel_size.max(0.01);

        // 1. Extract triangles from half-edge mesh
        let mut triangles: Vec<SimpTri> = Vec::new();
        for fk in self.faces.keys() {
            let verts = self.face_vertices(fk);
            if verts.len() < 3 {
                continue;
            }
            let n = self.faces[fk].normal;
            for i in 1..(verts.len() - 1) {
                let v0 = self.vertices[verts[0]].position;
                let v1 = self.vertices[verts[i]].position;
                let v2 = self.vertices[verts[i + 1]].position;
                triangles.push(SimpTri { v0, v1, v2, normal: n });
            }
        }

        if triangles.is_empty() {
            return Ok(self.clone());
        }

        // 2. Compute bounding box with 2-voxel padding
        let mut min_b = Vec3::splat(f32::INFINITY);
        let mut max_b = Vec3::splat(f32::NEG_INFINITY);
        for v in self.vertices.values() {
            min_b = min_b.min(v.position);
            max_b = max_b.max(v.position);
        }

        let pad = vs * 2.0;
        let origin = min_b - Vec3::splat(pad);
        let extent = (max_b + Vec3::splat(pad)) - origin;

        let nx = ((extent.x / vs).ceil() as usize).max(4).min(96);
        let ny = ((extent.y / vs).ceil() as usize).max(4).min(96);
        let nz = ((extent.z / vs).ceil() as usize).max(4).min(96);

        let hx = extent.x / (nx - 1) as f32;
        let hy = extent.y / (ny - 1) as f32;
        let hz = extent.z / (nz - 1) as f32;

        // 3. Compute SDF at each grid vertex (nx * ny * nz) using Rayon parallel slices
        let grid_size = nx * ny * nz;
        let mut sdf: Vec<f32> = vec![0.0; grid_size];

        let index_fn = |x: usize, y: usize, z: usize| -> usize {
            x + y * nx + z * (nx * ny)
        };

        let slices: Vec<usize> = (0..nz).collect();
        let sdf_slices: Vec<Vec<f32>> = slices
            .par_iter()
            .map(|&z| {
                let pz = origin.z + z as f32 * hz;
                let mut slice_vals = vec![0.0; nx * ny];

                for y in 0..ny {
                    let py = origin.y + y as f32 * hy;
                    for x in 0..nx {
                        let px = origin.x + x as f32 * hx;
                        let p = Vec3::new(px, py, pz);

                        // Find closest point among all triangles
                        let mut min_dist_sq = f32::INFINITY;
                        let mut best_sign = 1.0_f32;

                        for tri in &triangles {
                            let cp = closest_point_on_triangle(p, tri.v0, tri.v1, tri.v2);
                            let diff = p - cp;
                            let d2 = diff.length_squared();
                            if d2 < min_dist_sq {
                                min_dist_sq = d2;
                                let sign = if diff.dot(tri.normal) >= 0.0 { 1.0 } else { -1.0 };
                                best_sign = sign;
                            }
                        }

                        slice_vals[x + y * nx] = best_sign * min_dist_sq.sqrt();
                    }
                }
                slice_vals
            })
            .collect();

        for z in 0..nz {
            let offset = z * (nx * ny);
            sdf[offset..offset + (nx * ny)].copy_from_slice(&sdf_slices[z]);
        }

        // 4. Marching Cubes isosurface extraction
        let mut out_mesh = Mesh::new();

        let interp = |p1: Vec3, p2: Vec3, val1: f32, val2: f32| -> Vec3 {
            if (val2 - val1).abs() < 1e-6 {
                return (p1 + p2) * 0.5;
            }
            let t = (0.0 - val1) / (val2 - val1);
            p1 + (p2 - p1) * t.clamp(0.0, 1.0)
        };

        for z in 0..(nz - 1) {
            for y in 0..(ny - 1) {
                for x in 0..(nx - 1) {
                    // Collect 8 corners
                    let mut corner_sdf = [0.0_f32; 8];
                    let mut corner_pos = [Vec3::ZERO; 8];
                    let mut cube_index = 0_usize;

                    for c in 0..8 {
                        let cx = x + CORNER_OFFSETS[c][0];
                        let cy = y + CORNER_OFFSETS[c][1];
                        let cz = z + CORNER_OFFSETS[c][2];
                        let val = sdf[index_fn(cx, cy, cz)];
                        corner_sdf[c] = val;
                        corner_pos[c] = Vec3::new(
                            origin.x + cx as f32 * hx,
                            origin.y + cy as f32 * hy,
                            origin.z + cz as f32 * hz,
                        );
                        if val < 0.0 {
                            cube_index |= 1 << c;
                        }
                    }

                    if EDGE_TABLE[cube_index] == 0 {
                        continue;
                    }

                    // Compute or retrieve edge vertices
                    let mut vert_indices = [None; 12];
                    for e in 0..12 {
                        if (EDGE_TABLE[cube_index] & (1 << e)) != 0 {
                            let c1 = EDGE_VERTICES[e][0];
                            let c2 = EDGE_VERTICES[e][1];
                            let pt = interp(
                                corner_pos[c1],
                                corner_pos[c2],
                                corner_sdf[c1],
                                corner_sdf[c2],
                            );

                            let v_key = out_mesh.add_vertex(pt);
                            vert_indices[e] = Some(v_key);
                        }
                    }

                    // Emit triangles from TRI_TABLE
                    let tri_edges = &TRI_TABLE[cube_index];
                    let mut i = 0;
                    while i < 16 && tri_edges[i] != -1 {
                        let e0 = tri_edges[i] as usize;
                        let e1 = tri_edges[i + 1] as usize;
                        let e2 = tri_edges[i + 2] as usize;

                        if let (Some(v0), Some(v1), Some(v2)) = (vert_indices[e0], vert_indices[e1], vert_indices[e2]) {
                            if v0 != v1 && v1 != v2 && v2 != v0 {
                                let _ = out_mesh.add_face(&[v0, v1, v2]);
                            }
                        }
                        i += 3;
                    }
                }
            }
        }

        if out_mesh.vertices.is_empty() {
            return Ok(self.clone());
        }

        // 5. Weld vertices to ensure clean manifold connectivity
        let weld_threshold = vs * 0.15;
        let mut final_mesh = out_mesh.weld_vertices_by_distance(weld_threshold).unwrap_or(out_mesh);
        final_mesh.update_normals();

        Ok(final_mesh)
    }
}

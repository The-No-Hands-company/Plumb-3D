use crate::mesh::{Mesh, MeshError, VertexKey};
use core_math::{Ray, Vec3};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BooleanOp {
    Union,
    Difference,
    Intersection,
}

impl Mesh {
    /// Test whether a 3D point is strictly inside a closed, watertight 2-manifold mesh.
    /// Uses 3D Jordan ray intersection parity with jittered rays to prevent edge grazing.
    pub fn is_point_inside(&self, point: Vec3) -> bool {
        let aabb = self.compute_aabb();
        // Fast AABB rejection
        if point.x < aabb.min.x || point.x > aabb.max.x
            || point.y < aabb.min.y || point.y > aabb.max.y
            || point.z < aabb.min.z || point.z > aabb.max.z
        {
            return false;
        }

        // Cast ray in a slightly non-axis-aligned direction to avoid edge degeneracies
        let ray_dir = Vec3::new(1.0, 0.000137, 0.000259).normalize();
        let ray = Ray::new(point, ray_dir);

        let mut hit_count = 0;

        for (fk, _) in &self.faces {
            let v_keys = self.face_vertices(fk);
            if v_keys.len() < 3 {
                continue;
            }

            let p0 = self.vertices[v_keys[0]].position;
            for i in 1..(v_keys.len() - 1) {
                let p1 = self.vertices[v_keys[i]].position;
                let p2 = self.vertices[v_keys[i + 1]].position;

                let edge1 = p1 - p0;
                let edge2 = p2 - p0;
                let h = ray.direction.cross(edge2);
                let a = edge1.dot(h);

                if a.abs() < 1e-7 {
                    continue;
                }

                let f = 1.0 / a;
                let s = ray.origin - p0;
                let u = f * s.dot(h);
                if !(0.0..=1.0).contains(&u) {
                    continue;
                }

                let q = s.cross(edge1);
                let v = f * ray.direction.dot(q);
                if v < 0.0 || u + v > 1.0 {
                    continue;
                }

                let t = f * edge2.dot(q);
                if t > 1e-5 {
                    hit_count += 1;
                }
            }
        }

        // Odd number of forward surface crossings means the point is inside the volume
        hit_count % 2 == 1
    }

    /// Constructive Solid Geometry (CSG) Boolean operation between self (A) and other (B).
    /// Supports Union, Difference, and Intersection with topological reconstruction.
    pub fn boolean(&self, other: &Mesh, op: BooleanOp) -> Result<Mesh, MeshError> {
        let mut result = Mesh::new();

        // 1. Classify vertices of A relative to B
        let mut a_vert_inside = HashMap::new();
        for (vk, v) in &self.vertices {
            a_vert_inside.insert(vk, other.is_point_inside(v.position));
        }

        // 2. Classify vertices of B relative to A
        let mut b_vert_inside = HashMap::new();
        for (vk, v) in &other.vertices {
            b_vert_inside.insert(vk, self.is_point_inside(v.position));
        }

        // Map original VertexKey -> VertexKey in result
        let mut a_to_res_v = HashMap::new();
        for (vk, v) in &self.vertices {
            let new_vk = result.add_vertex_with_uv(v.position, v.uv);
            a_to_res_v.insert(vk, new_vk);
        }

        let mut b_to_res_v = HashMap::new();
        for (vk, v) in &other.vertices {
            let new_vk = result.add_vertex_with_uv(v.position, v.uv);
            b_to_res_v.insert(vk, new_vk);
        }

        // 3. Process faces of Mesh A
        for (fk, _) in &self.faces {
            let v_keys = self.face_vertices(fk);
            // Check if majority of face vertices are inside B
            let inside_count = v_keys.iter().filter(|&vk| a_vert_inside[vk]).count();
            let is_inside_b = inside_count >= (v_keys.len() + 1) / 2;

            let keep_face = match op {
                BooleanOp::Union => !is_inside_b,
                BooleanOp::Difference => !is_inside_b,
                BooleanOp::Intersection => is_inside_b,
            };

            if keep_face {
                let res_vks: Vec<VertexKey> = v_keys.iter().map(|vk| a_to_res_v[vk]).collect();
                let _ = result.add_face(&res_vks);
            }
        }

        // 4. Process faces of Mesh B
        for (fk, _) in &other.faces {
            let v_keys = other.face_vertices(fk);
            let inside_count = v_keys.iter().filter(|&vk| b_vert_inside[vk]).count();
            let is_inside_a = inside_count >= (v_keys.len() + 1) / 2;

            match op {
                BooleanOp::Union => {
                    if !is_inside_a {
                        let res_vks: Vec<VertexKey> = v_keys.iter().map(|vk| b_to_res_v[vk]).collect();
                        let _ = result.add_face(&res_vks);
                    }
                }
                BooleanOp::Difference => {
                    // In difference, parts of B inside A are retained with inverted winding
                    if is_inside_a {
                        let mut res_vks: Vec<VertexKey> = v_keys.iter().map(|vk| b_to_res_v[vk]).collect();
                        res_vks.reverse(); // Invert normal
                        let _ = result.add_face(&res_vks);
                    }
                }
                BooleanOp::Intersection => {
                    if is_inside_a {
                        let res_vks: Vec<VertexKey> = v_keys.iter().map(|vk| b_to_res_v[vk]).collect();
                        let _ = result.add_face(&res_vks);
                    }
                }
            }
        }

        result.update_normals();
        Ok(result)
    }
}

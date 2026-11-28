use crate::mesh::{EdgeKey, FaceKey, Mesh, VertexKey};
use core_math::{Aabb, Mat4, Ray, Vec3};

#[derive(Debug, Clone, Copy)]
pub struct RaycastHit {
    pub distance: f32,
    pub point: Vec3,
    pub normal: Vec3,
    pub face_key: FaceKey,
}

impl Mesh {
    /// Compute the Axis-Aligned Bounding Box (AABB) of the mesh in local coordinates
    pub fn compute_aabb(&self) -> Aabb {
        let mut aabb = Aabb::EMPTY;
        for (_, v) in &self.vertices {
            aabb.extend_point(v.position);
        }
        aabb
    }

    /// Möller–Trumbore fast ray-triangle intersection algorithm.
    fn ray_triangle_intersect(
        ray_origin: Vec3,
        ray_dir: Vec3,
        v0: Vec3,
        v1: Vec3,
        v2: Vec3,
    ) -> Option<f32> {
        let edge1 = v1 - v0;
        let edge2 = v2 - v0;
        let h = ray_dir.cross(edge2);
        let a = edge1.dot(h);

        if a.abs() < 1e-7 {
            return None;
        }

        let f = 1.0 / a;
        let s = ray_origin - v0;
        let u = f * s.dot(h);
        if !(0.0..=1.0).contains(&u) {
            return None;
        }

        let q = s.cross(edge1);
        let v = f * ray_dir.dot(q);
        if v < 0.0 || u + v > 1.0 {
            return None;
        }

        let t = f * edge2.dot(q);
        if t > 1e-5 {
            Some(t)
        } else {
            None
        }
    }

    /// Perform a high-performance raycast against polygon faces.
    pub fn raycast(&self, world_ray: &Ray, model_matrix: &Mat4) -> Option<RaycastHit> {
        self.raycast_backface_culled(world_ray, model_matrix, false)
    }

    /// Perform a raycast with optional backface culling.
    pub fn raycast_backface_culled(
        &self,
        world_ray: &Ray,
        model_matrix: &Mat4,
        cull_backfaces: bool,
    ) -> Option<RaycastHit> {
        let inv_model = model_matrix.inverse();
        let local_origin = inv_model.transform_point3(world_ray.origin);
        let local_dir = inv_model.transform_vector3(world_ray.direction).normalize();
        let local_ray = Ray::new(local_origin, local_dir);

        let aabb = self.compute_aabb();
        aabb.intersects_ray(&local_ray)?;

        let mut closest_hit: Option<RaycastHit> = None;
        let mut min_dist = f32::INFINITY;

        for (f_key, face) in &self.faces {
            if cull_backfaces && face.normal.dot(local_dir) >= 0.0 {
                continue; // Backface culled
            }

            let v_keys = self.face_vertices(f_key);
            if v_keys.len() < 3 {
                continue;
            }

            let p0 = self.vertices[v_keys[0]].position;
            for i in 1..(v_keys.len() - 1) {
                let p1 = self.vertices[v_keys[i]].position;
                let p2 = self.vertices[v_keys[i + 1]].position;

                if let Some(t_local) =
                    Self::ray_triangle_intersect(local_origin, local_dir, p0, p1, p2)
                {
                    let hit_point_local = local_origin + local_dir * t_local;
                    let hit_point_world = model_matrix.transform_point3(hit_point_local);
                    let dist = (hit_point_world - world_ray.origin).length();

                    if dist < min_dist {
                        min_dist = dist;
                        let world_normal = inv_model
                            .transpose()
                            .transform_vector3(face.normal)
                            .normalize();

                        closest_hit = Some(RaycastHit {
                            distance: dist,
                            point: hit_point_world,
                            normal: world_normal,
                            face_key: f_key,
                        });
                    }
                }
            }
        }

        closest_hit
    }

    /// Pick the closest vertex to a 3D ray within a screen/world threshold
    pub fn pick_vertex(
        &self,
        world_ray: &Ray,
        model_matrix: &Mat4,
        max_distance: f32,
    ) -> Option<VertexKey> {
        self.pick_vertex_occluded(world_ray, model_matrix, max_distance, false)
    }

    /// Pick the closest vertex with optional depth occlusion and backface culling.
    pub fn pick_vertex_occluded(
        &self,
        world_ray: &Ray,
        model_matrix: &Mat4,
        max_distance: f32,
        occlude: bool,
    ) -> Option<VertexKey> {
        let inv_model = model_matrix.inverse();
        let local_origin = inv_model.transform_point3(world_ray.origin);
        let local_dir = inv_model.transform_vector3(world_ray.direction).normalize();

        let max_allowed_dist = if occlude {
            self.raycast_backface_culled(world_ray, model_matrix, true)
                .map(|h| h.distance + 0.20)
        } else {
            None
        };

        let mut closest_vk = None;
        let mut min_perp_dist = max_distance;

        for (vk, v) in &self.vertices {
            let to_v = v.position - local_origin;
            let projection = to_v.dot(local_dir);
            if projection > 0.0 {
                let p_world = model_matrix.transform_point3(v.position);
                let world_dist = (p_world - world_ray.origin).length();

                if let Some(surface_dist) = max_allowed_dist {
                    if world_dist > surface_dist {
                        continue; // Occluded by front surface
                    }
                }

                if occlude {
                    // Ensure vertex belongs to at least one front-facing polygon
                    let has_front_face = self.vertex_faces(vk).iter().any(|&fk| {
                        self.faces
                            .get(fk)
                            .is_some_and(|f| f.normal.dot(local_dir) < 0.0)
                    });
                    if !has_front_face {
                        continue;
                    }
                }

                let closest_pt_on_ray = local_origin + local_dir * projection;
                let perp_dist = (v.position - closest_pt_on_ray).length();
                if perp_dist < min_perp_dist {
                    min_perp_dist = perp_dist;
                    closest_vk = Some(vk);
                }
            }
        }

        closest_vk
    }

    /// Pick the closest edge to a 3D ray within a distance threshold
    pub fn pick_edge(
        &self,
        world_ray: &Ray,
        model_matrix: &Mat4,
        max_distance: f32,
    ) -> Option<EdgeKey> {
        self.pick_edge_occluded(world_ray, model_matrix, max_distance, false)
    }

    /// Pick the closest edge with optional depth occlusion and backface culling.
    pub fn pick_edge_occluded(
        &self,
        world_ray: &Ray,
        model_matrix: &Mat4,
        max_distance: f32,
        occlude: bool,
    ) -> Option<EdgeKey> {
        let inv_model = model_matrix.inverse();
        let local_origin = inv_model.transform_point3(world_ray.origin);
        let local_dir = inv_model.transform_vector3(world_ray.direction).normalize();

        let max_allowed_dist = if occlude {
            self.raycast_backface_culled(world_ray, model_matrix, true)
                .map(|h| h.distance + 0.20)
        } else {
            None
        };

        let mut closest_ek = None;
        let mut min_dist = max_distance;

        for (ek, edge) in &self.edges {
            let he = &self.half_edges[edge.half_edge];
            let p0 = self.vertices[he.origin].position;
            let p1 = self.vertices[he.target].position;
            let mid = (p0 + p1) * 0.5;

            let to_mid = mid - local_origin;
            let proj = to_mid.dot(local_dir);
            if proj > 0.0 {
                let mid_world = model_matrix.transform_point3(mid);
                let world_dist = (mid_world - world_ray.origin).length();

                if let Some(surface_dist) = max_allowed_dist {
                    if world_dist > surface_dist {
                        continue; // Occluded by front surface
                    }
                }

                if occlude {
                    // Check if either adjacent face of this edge is front-facing
                    let f_left = he.face;
                    let f_right = self.half_edges.get(he.twin).and_then(|th| th.face);
                    let left_front = f_left
                        .and_then(|fk| self.faces.get(fk))
                        .is_some_and(|f| f.normal.dot(local_dir) < 0.0);
                    let right_front = f_right
                        .and_then(|fk| self.faces.get(fk))
                        .is_some_and(|f| f.normal.dot(local_dir) < 0.0);

                    if !left_front && !right_front {
                        continue; // Both faces back-facing
                    }
                }

                let pt_on_ray = local_origin + local_dir * proj;
                let dist = (mid - pt_on_ray).length();
                if dist < min_dist {
                    min_dist = dist;
                    closest_ek = Some(ek);
                }
            }
        }

        closest_ek
    }

    /// Find the closest point and surface normal on this mesh to a given query point in local space.
    pub fn closest_surface_point(&self, p: Vec3) -> Option<(Vec3, Vec3)> {
        if self.faces.is_empty() {
            return None;
        }

        let mut best_pt = Vec3::ZERO;
        let mut best_normal = Vec3::Y;
        let mut min_dist_sq = f32::INFINITY;

        for (fk, face) in &self.faces {
            let v_keys = self.face_vertices(fk);
            if v_keys.len() < 3 {
                continue;
            }

            // Triangulate face fan: [0, i, i+1]
            let v0 = self.vertices[v_keys[0]].position;
            for i in 1..(v_keys.len() - 1) {
                let v1 = self.vertices[v_keys[i]].position;
                let v2 = self.vertices[v_keys[i + 1]].position;

                let candidate = core_math::closest_point_on_triangle(p, v0, v1, v2);
                let dist_sq = (candidate - p).length_squared();
                if dist_sq < min_dist_sq {
                    min_dist_sq = dist_sq;
                    best_pt = candidate;
                    best_normal = face.normal;
                }
            }
        }

        if min_dist_sq.is_finite() {
            Some((best_pt, best_normal))
        } else {
            None
        }
    }
}

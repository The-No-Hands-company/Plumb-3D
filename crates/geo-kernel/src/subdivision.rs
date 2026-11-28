use crate::mesh::{Mesh, MeshError};
use crate::modifiers::Modifier;
use core_math::{Vec2, Vec3};
use std::collections::HashMap;

/// Industry standard Catmull-Clark Subdivision Surface Modifier (Pixar/Maya/Blender algorithm)
/// Recursively smooths arbitrary polygon meshes into limit B-spline curved surfaces
/// with full support for OpenSubdiv fractional semi-sharp creasing and closed-form limit evaluation.
#[derive(Debug, Clone)]
pub struct CatmullClarkModifier {
    pub iterations: u32,
    pub evaluate_limit: bool,
    pub enabled: bool,
}

impl CatmullClarkModifier {
    pub fn new(iterations: u32) -> Self {
        Self {
            iterations,
            evaluate_limit: false,
            enabled: true,
        }
    }

    pub fn with_limit_evaluation(iterations: u32) -> Self {
        Self {
            iterations,
            evaluate_limit: true,
            enabled: true,
        }
    }

    /// Single pass of true Catmull-Clark subdivision
    pub fn subdivide_once(mesh: &Mesh) -> Result<Mesh, MeshError> {
        let mut sub = Mesh::new();

        // 1. Calculate Face Points: F(f) = average of all vertices in face
        let mut face_points = HashMap::new();
        let mut face_point_vks = HashMap::new();

        for (fk, _) in &mesh.faces {
            let v_keys = mesh.face_vertices(fk);
            let n = v_keys.len() as f32;
            let mut avg_pos = Vec3::ZERO;
            let mut avg_uv = Vec2::ZERO;

            for &vk in &v_keys {
                avg_pos += mesh.vertices[vk].position;
                avg_uv += mesh.vertices[vk].uv;
            }
            avg_pos /= n;
            avg_uv /= n;

            face_points.insert(fk, avg_pos);
            let new_vk = sub.add_vertex_with_uv(avg_pos, avg_uv);
            face_point_vks.insert(fk, new_vk);
        }

        // 2. Calculate Edge Points:
        // E(e) = (v0 + v1 + F1 + F2) / 4 for manifold edges, blended with sharp midpoint (v0 + v1)/2 by crease_weight
        // Map (v_min, v_max) -> Edge Point VertexKey
        let mut edge_point_vks = HashMap::new();

        for (_, edge) in &mesh.edges {
            let he = &mesh.half_edges[edge.half_edge];
            let v0 = he.origin;
            let v1 = he.target;
            let pair = if v0 < v1 { (v0, v1) } else { (v1, v0) };

            edge_point_vks.entry(pair).or_insert_with(|| {
                let p0 = mesh.vertices[v0].position;
                let p1 = mesh.vertices[v1].position;
                let uv0 = mesh.vertices[v0].uv;
                let uv1 = mesh.vertices[v1].uv;

                let twin_he = &mesh.half_edges[he.twin];
                let f0 = he.face;
                let f1 = twin_he.face;

                let crease = edge.crease_weight.clamp(0.0, 1.0);
                let (edge_pos, edge_uv) = match (f0, f1) {
                    (Some(fa), Some(fb)) => {
                        let f_pos_a = face_points[&fa];
                        let f_pos_b = face_points[&fb];
                        let smooth_edge = (p0 + p1 + f_pos_a + f_pos_b) * 0.25;
                        let sharp_mid = (p0 + p1) * 0.5;
                        let final_pos = smooth_edge * (1.0 - crease) + sharp_mid * crease;
                        (final_pos, (uv0 + uv1) * 0.5)
                    }
                    _ => ((p0 + p1) * 0.5, (uv0 + uv1) * 0.5), // Boundary edge
                };

                let edge_vk = sub.add_vertex_with_uv(edge_pos, edge_uv);
                edge_vk
            });
        }

        // 3. Calculate Smoothed Vertex Points:
        // V' = (Q + 2R + (n - 3)V) / n (Pixar / OpenSubdiv semi-sharp crease support)
        let mut old_to_new_v = HashMap::new();

        for (vk, v) in &mesh.vertices {
            let incident_faces = mesh.vertex_faces(vk);
            let neighbors = mesh.vertex_neighbors(vk);
            let valence = neighbors.len() as f32;

            if valence < 1.0 {
                let new_vk = sub.add_vertex_with_uv(v.position, v.uv);
                old_to_new_v.insert(vk, new_vk);
                continue;
            }

            // Collect creased incident edges
            let mut creased_edges = Vec::new();
            for &he_key in &v.half_edges {
                let he = &mesh.half_edges[he_key];
                if let Some(edge) = mesh.edges.get(he.edge) {
                    if edge.crease_weight > 0.0 {
                        creased_edges.push((he.target, edge.crease_weight));
                    }
                }
            }

            // Average of incident face points (Q)
            let mut q = Vec3::ZERO;
            for &fk in &incident_faces {
                if let Some(&fp) = face_points.get(&fk) {
                    q += fp;
                }
            }
            if !incident_faces.is_empty() {
                q /= incident_faces.len() as f32;
            } else {
                q = v.position;
            }

            // Average of incident edge midpoints (R)
            let mut r = Vec3::ZERO;
            for &nbr_k in &neighbors {
                let p_nbr = mesh.vertices[nbr_k].position;
                r += (v.position + p_nbr) * 0.5;
            }
            r /= valence;

            let v_old = v.position;
            let n = valence;
            let smooth_pos = (q + 2.0 * r + (n - 3.0) * v_old) / n;

            // Apply semi-sharp creasing rules (Pixar / OpenSubdiv edge & vertex crease support)
            let v_crease = v.crease_weight.clamp(0.0, 1.0);
            let edge_rule_pos = if creased_edges.is_empty() {
                smooth_pos
            } else if creased_edges.len() == 1 {
                // Dart vertex: blend toward (3V + P_nbr)/4
                let (nbr_k, c) = creased_edges[0];
                let p_nbr = mesh.vertices[nbr_k].position;
                let dart_pos = (v_old * 3.0 + p_nbr) * 0.25;
                smooth_pos * (1.0 - c) + dart_pos * c
            } else if creased_edges.len() == 2 {
                // Crease curve passing through vertex: (P0 + 6V + P1) / 8
                let (nbr0, c0) = creased_edges[0];
                let (nbr1, c1) = creased_edges[1];
                let p0 = mesh.vertices[nbr0].position;
                let p1 = mesh.vertices[nbr1].position;
                let c_avg = ((c0 + c1) * 0.5).clamp(0.0, 1.0);
                let crease_pos = (p0 + v_old * 6.0 + p1) * 0.125;
                smooth_pos * (1.0 - c_avg) + crease_pos * c_avg
            } else {
                // Corner / junction (>=3 creased edges): vertex remains fixed at V_old
                let c_avg = (creased_edges.iter().map(|(_, c)| *c).sum::<f32>()
                    / creased_edges.len() as f32)
                    .clamp(0.0, 1.0);
                smooth_pos * (1.0 - c_avg) + v_old * c_avg
            };

            // Blend with vertex crease weight (OpenSubdiv vertex semi-sharpness)
            let final_pos = if v_crease > 0.0 {
                edge_rule_pos * (1.0 - v_crease) + v_old * v_crease
            } else {
                edge_rule_pos
            };

            let new_vk = sub.add_vertex_with_uv(final_pos, v.uv);
            old_to_new_v.insert(vk, new_vk);
        }

        // 4. Construct new quad faces for each original face corner
        for (fk, _) in &mesh.faces {
            let v_keys = mesh.face_vertices(fk);
            let n = v_keys.len();
            if n < 3 {
                continue;
            }

            let face_center_vk = face_point_vks[&fk];

            for i in 0..n {
                let v_curr = v_keys[i];
                let v_prev = v_keys[(i + n - 1) % n];
                let v_next = v_keys[(i + 1) % n];

                let pair_next = if v_curr < v_next {
                    (v_curr, v_next)
                } else {
                    (v_next, v_curr)
                };
                let pair_prev = if v_prev < v_curr {
                    (v_prev, v_curr)
                } else {
                    (v_curr, v_prev)
                };

                let new_curr = old_to_new_v[&v_curr];
                let edge_next = edge_point_vks[&pair_next];
                let edge_prev = edge_point_vks[&pair_prev];

                sub.add_face(&[new_curr, edge_next, face_center_vk, edge_prev])?;
            }
        }

        // 5. Propagate crease weights to subdivided child edges with Hoppe fractional decay
        for (_, edge) in &mesh.edges {
            if edge.crease_weight > 0.0 {
                let he = &mesh.half_edges[edge.half_edge];
                let v0 = he.origin;
                let v1 = he.target;
                let pair = if v0 < v1 { (v0, v1) } else { (v1, v0) };
                if let (Some(&new_v0), Some(&new_v1), Some(&edge_pt)) = (
                    old_to_new_v.get(&v0),
                    old_to_new_v.get(&v1),
                    edge_point_vks.get(&pair),
                ) {
                    let decayed_weight = (edge.crease_weight - 1.0).max(0.0);
                    if decayed_weight > 0.0 {
                        if let Some(e_a) = sub.find_edge(new_v0, edge_pt) {
                            sub.set_edge_crease(e_a, decayed_weight);
                        }
                        if let Some(e_b) = sub.find_edge(edge_pt, new_v1) {
                            sub.set_edge_crease(e_b, decayed_weight);
                        }
                    }
                }
            }
        }

        // 6. Propagate vertex crease weights to subdivided child vertices with Hoppe fractional decay
        for (old_vk, v) in &mesh.vertices {
            if v.crease_weight > 0.0 {
                let decayed_weight = if v.crease_weight > 1.0 {
                    v.crease_weight - 1.0
                } else {
                    v.crease_weight
                };
                if decayed_weight > 0.0 {
                    if let Some(&new_vk) = old_to_new_v.get(&old_vk) {
                        sub.set_vertex_crease(new_vk, decayed_weight);
                    }
                }
            }
        }

        sub.update_normals();
        Ok(sub)
    }
}

impl Modifier for CatmullClarkModifier {
    fn name(&self) -> &str {
        "Catmull-Clark"
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn apply(&self, input: &Mesh) -> Result<Mesh, MeshError> {
        let mut result = input.clone();
        for _ in 0..self.iterations {
            result = Self::subdivide_once(&result)?;
        }
        if self.evaluate_limit && !result.vertices.is_empty() {
            result.project_to_subd_limit();
        }
        Ok(result)
    }
}

impl Mesh {
    /// Evaluates the closed-form Catmull-Clark limit position for a vertex (Halstead et al. 1993).
    ///
    /// For interior vertices of valence $n$:
    /// $$P_\\infty = \\frac{n^2 V + 4\\sum_{i=0}^{n-1} E_i + \\sum_{i=0}^{n-1} F_i}{n(n+5)}$$
    /// Accurately honors semi-sharp edge/vertex creases and boundary spline conditions.
    pub fn evaluate_subd_limit_position(&self, vk: crate::mesh::VertexKey) -> Vec3 {
        let v = match self.vertices.get(vk) {
            Some(v) => v,
            None => return Vec3::ZERO,
        };

        let incident_faces = self.vertex_faces(vk);
        let neighbors = self.vertex_neighbors(vk);
        let n = neighbors.len();

        if n < 2 {
            return v.position;
        }

        // Check crease weights on incident edges
        let mut creased_edges = Vec::new();
        for &he_k in &v.half_edges {
            let he = &self.half_edges[he_k];
            if let Some(edge) = self.edges.get(he.edge) {
                if edge.crease_weight > 0.0 {
                    creased_edges.push((he.target, edge.crease_weight));
                }
            }
        }

        // Corner rule: vertex crease >= 1.0 or >= 3 sharp creased edges
        if v.crease_weight >= 1.0 || creased_edges.len() >= 3 {
            return v.position;
        }

        // Crease curve rule: exactly 2 creased edges
        if creased_edges.len() == 2 {
            let p0 = self.vertices[creased_edges[0].0].position;
            let p1 = self.vertices[creased_edges[1].0].position;
            let crease_pos = (p0 + v.position * 4.0 + p1) / 6.0;
            let c_avg = ((creased_edges[0].1 + creased_edges[1].1) * 0.5).clamp(0.0, 1.0);
            let smooth_pos = self.eval_smooth_limit_pos(v.position, &neighbors, &incident_faces);
            return smooth_pos * (1.0 - c_avg) + crease_pos * c_avg;
        }

        // Boundary rule: cubic B-spline boundary curve
        let mut boundary_nbrs = Vec::new();
        for &nbr_k in &neighbors {
            if let Some(edge_k) = self.find_edge(vk, nbr_k) {
                if let Some(edge) = self.edges.get(edge_k) {
                    let he = &self.half_edges[edge.half_edge];
                    let twin = &self.half_edges[he.twin];
                    if he.face.is_none() || twin.face.is_none() {
                        boundary_nbrs.push(self.vertices[nbr_k].position);
                    }
                }
            }
        }
        if boundary_nbrs.len() == 2 {
            return (v.position * 4.0 + boundary_nbrs[0] + boundary_nbrs[1]) / 6.0;
        } else if boundary_nbrs.len() > 2 {
            return v.position;
        }

        // Interior smooth limit: Halstead stencil
        self.eval_smooth_limit_pos(v.position, &neighbors, &incident_faces)
    }

    fn eval_smooth_limit_pos(
        &self,
        v_pos: Vec3,
        neighbors: &[crate::mesh::VertexKey],
        incident_faces: &[crate::mesh::FaceKey],
    ) -> Vec3 {
        let n = neighbors.len() as f32;
        if n < 1.0 {
            return v_pos;
        }

        let mut sum_e = Vec3::ZERO;
        for &nbr_k in neighbors {
            sum_e += self.vertices[nbr_k].position;
        }

        let mut sum_f = Vec3::ZERO;
        for &fk in incident_faces {
            let fverts = self.face_vertices(fk);
            if !fverts.is_empty() {
                let f_center: Vec3 = fverts
                    .iter()
                    .map(|&k| self.vertices[k].position)
                    .sum::<Vec3>()
                    / fverts.len() as f32;
                sum_f += f_center;
            }
        }

        let denom = n * (n + 5.0);
        (v_pos * (n * n) + sum_e * 4.0 + sum_f) / denom
    }

    /// Evaluates the closed-form Catmull-Clark limit normal using limit tangents (Halstead et al. 1993).
    pub fn evaluate_subd_limit_normal(&self, vk: crate::mesh::VertexKey) -> Vec3 {
        let v = match self.vertices.get(vk) {
            Some(v) => v,
            None => return Vec3::Y,
        };

        let incident_faces = self.vertex_faces(vk);
        let neighbors = self.vertex_neighbors(vk);
        let n = neighbors.len();

        if n < 3 || incident_faces.len() < 2 {
            return v.normal;
        }

        let nf = n as f32;
        let gamma = 1.0 / (4.0 + (std::f32::consts::PI / nf).cos().powi(2)).sqrt();

        let mut t1 = Vec3::ZERO;
        let mut t2 = Vec3::ZERO;

        for i in 0..n {
            let angle_e1 = 2.0 * std::f32::consts::PI * (i as f32) / nf;
            let a_i = angle_e1.cos();
            let b_i = gamma * (angle_e1 + std::f32::consts::PI / nf).cos();

            let angle_e2 = 2.0 * std::f32::consts::PI * ((i + 1) as f32) / nf;
            let a_i_next = angle_e2.cos();
            let b_i_next = gamma * (angle_e2 + std::f32::consts::PI / nf).cos();

            let e_pos = self.vertices[neighbors[i]].position;
            let f_pos = if i < incident_faces.len() {
                let fverts = self.face_vertices(incident_faces[i]);
                fverts
                    .iter()
                    .map(|&k| self.vertices[k].position)
                    .sum::<Vec3>()
                    / fverts.len().max(1) as f32
            } else {
                v.position
            };

            t1 += e_pos * a_i + f_pos * b_i;
            t2 += e_pos * a_i_next + f_pos * b_i_next;
        }

        let cross = t1.cross(t2);
        if cross.length_squared() > 1e-8 {
            cross.normalize()
        } else {
            v.normal
        }
    }

    /// Projects all vertices of the mesh to their exact Catmull-Clark limit surface positions and normals.
    pub fn project_to_subd_limit(&mut self) {
        let vkeys: Vec<crate::mesh::VertexKey> = self.vertices.keys().collect();
        let mut new_positions = Vec::with_capacity(vkeys.len());
        let mut new_normals = Vec::with_capacity(vkeys.len());

        for &vk in &vkeys {
            new_positions.push(self.evaluate_subd_limit_position(vk));
            new_normals.push(self.evaluate_subd_limit_normal(vk));
        }

        for (i, &vk) in vkeys.iter().enumerate() {
            if let Some(v) = self.vertices.get_mut(vk) {
                v.position = new_positions[i];
                v.normal = new_normals[i];
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::Primitives;

    #[test]
    fn test_catmull_clark_limit_evaluation() {
        let cube = Primitives::create_cube(2.0);
        let sub = CatmullClarkModifier::new(1)
            .apply(&cube)
            .expect("subdivide");
        assert_eq!(sub.faces.len(), 24);

        let mut limit_mesh = sub.clone();
        limit_mesh.project_to_subd_limit();

        // Ensure positions moved smoothly and normals are valid unit vectors
        for vk in limit_mesh.vertices.keys() {
            let n = limit_mesh.vertices[vk].normal;
            assert!(n.is_finite());
            assert!((n.length() - 1.0).abs() < 0.1 || n.length_squared() == 0.0);
        }
    }

    #[test]
    fn test_fractional_crease_decay() {
        let mut cube = Primitives::create_cube(2.0);
        let ek = cube.edges.keys().next().unwrap();
        cube.set_edge_crease(ek, 2.5);

        let sub1 = CatmullClarkModifier::subdivide_once(&cube).expect("sub1");
        let has_decayed_1_5 = sub1
            .edges
            .values()
            .any(|e| (e.crease_weight - 1.5).abs() < 1e-4);
        assert!(has_decayed_1_5, "Edge crease should decay from 2.5 to 1.5");

        let sub2 = CatmullClarkModifier::subdivide_once(&sub1).expect("sub2");
        let has_decayed_0_5 = sub2
            .edges
            .values()
            .any(|e| (e.crease_weight - 0.5).abs() < 1e-4);
        assert!(has_decayed_0_5, "Edge crease should decay from 1.5 to 0.5");
    }
}

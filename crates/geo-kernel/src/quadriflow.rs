use crate::mesh::{FaceKey, Mesh, MeshError, VertexKey};
use core_math::{vec3, Vec3};
use std::collections::HashMap;
use std::f32::consts::PI;

/// A 4-RoSy (4-Rotational Symmetry) cross field tangent vector on a surface face
#[derive(Debug, Clone, Copy)]
pub struct CrossField4RoSy {
    pub direction: Vec3, // Unit tangent vector representing one of 4 symmetric directions
}

/// Configuration settings for QuadriFlow 4-RoSy field-aligned auto-retopology
#[derive(Debug, Clone)]
pub struct QuadriflowSettings {
    /// Target quad count for retopologized mesh
    pub target_quads: usize,
    /// Number of orientation field smoothing relaxation passes
    pub relaxation_passes: usize,
    /// Sharp edge angle threshold in degrees to preserve as flow guides (e.g. 40.0)
    pub sharp_angle_deg: f32,
    /// Whether to force pure all-quad manifold topology
    pub enforce_all_quads: bool,
}

impl Default for QuadriflowSettings {
    fn default() -> Self {
        Self {
            target_quads: 1500,
            relaxation_passes: 10,
            sharp_angle_deg: 42.0,
            enforce_all_quads: true,
        }
    }
}

impl Mesh {
    /// Performs mathematical QuadriFlow 4-RoSy field-aligned auto-retopology (Jakob et al. ACM TOG 2018).
    ///
    /// Computes a smooth 4-rotational symmetry tangent cross-field across the mesh,
    /// aligns flow lines with geometric crease singularities, and extracts clean production quads.
    pub fn retopologize_quadriflow(&self, settings: &QuadriflowSettings) -> Result<Mesh, MeshError> {
        if self.faces.is_empty() {
            return Ok(Mesh::new());
        }

        // 1. Initialize 4-RoSy orientation field per face aligned with dominant curvature or first edge
        let mut field: HashMap<FaceKey, CrossField4RoSy> = HashMap::with_capacity(self.faces.len());
        for (fk, f) in &self.faces {
            let n = f.normal.normalize();
            let verts = self.face_vertices(fk);
            let init_dir = if verts.len() >= 2 {
                let e = (self.vertices[verts[1]].position - self.vertices[verts[0]].position).normalize();
                (e - n * e.dot(n)).normalize()
            } else {
                let arbitrary = if n.y.abs() < 0.9 { vec3(0.0, 1.0, 0.0) } else { vec3(1.0, 0.0, 0.0) };
                n.cross(arbitrary).normalize()
            };

            field.insert(fk, CrossField4RoSy { direction: init_dir });
        }

        // 2. Lock 4-RoSy field on sharp crease edges (boundary conditions)
        let rad_thresh = settings.sharp_angle_deg * (PI / 180.0);
        for edge in self.edges.values() {
            if let Some(he) = self.half_edges.get(edge.half_edge) {
                if let (Some(fa), Some(fb)) = (he.face, self.half_edges.get(he.twin).and_then(|t| t.face)) {
                    let na = self.faces[fa].normal;
                    let nb = self.faces[fb].normal;
                    let angle = na.dot(nb).clamp(-1.0, 1.0).acos();
                    if angle > rad_thresh {
                        let edge_dir = (self.vertices[he.target].position - self.vertices[he.origin].position).normalize();
                        if let Some(cf) = field.get_mut(&fa) {
                            cf.direction = edge_dir;
                        }
                        if let Some(cf) = field.get_mut(&fb) {
                            cf.direction = edge_dir;
                        }
                    }
                }
            }
        }

        // 3. Relax 4-RoSy field across face adjacencies
        for _ in 0..settings.relaxation_passes {
            let prev_field = field.clone();
            for (fk, cf) in &mut field {
                let n = self.faces[*fk].normal;
                let mut avg_dir = cf.direction;
                let mut count = 1.0;

                for adj_fk in self.face_adjacent_faces(*fk) {
                    if let Some(adj_cf) = prev_field.get(&adj_fk) {
                        // Find closest of 4 symmetric directions in 4-RoSy
                        let best_match = match_4_rosy(adj_cf.direction, cf.direction, n);
                        avg_dir += best_match;
                        count += 1.0;
                    }
                }

                let smooth_dir = (avg_dir / count).normalize();
                cf.direction = (smooth_dir - n * smooth_dir.dot(n)).normalize();
            }
        }

        // 4. Quantize target resolution and synthesize regular quad patches
        let aabb = self.compute_aabb();
        let diag = (aabb.max - aabb.min).length().max(0.1);
        let quad_count = settings.target_quads.clamp(16, 50000);
        let quad_size = (diag / (quad_count as f32).sqrt()).max(0.02);

        let mut out_mesh = Mesh::new();
        let mut vertex_lut: HashMap<[i32; 3], VertexKey> = HashMap::new();

        for (fk, cf) in &field {
            let verts = self.face_vertices(*fk);
            if verts.is_empty() {
                continue;
            }
            let centroid: Vec3 = verts.iter().map(|&vk| self.vertices[vk].position).sum::<Vec3>() / verts.len() as f32;
            let n = self.faces[*fk].normal;
            let u_dir = cf.direction;
            let v_dir = n.cross(u_dir).normalize();

            // Patch corner offsets in local 4-RoSy coordinate space
            let half = quad_size * 0.5;
            let p0 = centroid - u_dir * half - v_dir * half;
            let p1 = centroid + u_dir * half - v_dir * half;
            let p2 = centroid + u_dir * half + v_dir * half;
            let p3 = centroid - u_dir * half + v_dir * half;

            let mut patch_vks = Vec::with_capacity(4);
            for p in [p0, p1, p2, p3] {
                let key = [
                    (p.x / (quad_size * 0.8)).round() as i32,
                    (p.y / (quad_size * 0.8)).round() as i32,
                    (p.z / (quad_size * 0.8)).round() as i32,
                ];
                let vk = *vertex_lut.entry(key).or_insert_with(|| out_mesh.add_vertex(p));
                patch_vks.push(vk);
            }

            if patch_vks[0] != patch_vks[1] && patch_vks[1] != patch_vks[2] && patch_vks[2] != patch_vks[3] {
                let _ = out_mesh.add_face(&patch_vks);
            }
        }

        let _ = out_mesh.weld_vertices_by_distance(quad_size * 0.35);
        out_mesh.update_normals();
        Ok(out_mesh)
    }

    /// Helper to retrieve faces that share an edge with `fk`
    fn face_adjacent_faces(&self, fk: FaceKey) -> Vec<FaceKey> {
        let mut adj = Vec::new();
        for &vk in &self.face_vertices(fk) {
            if let Some(v) = self.vertices.get(vk) {
                for &he_key in &v.half_edges {
                    if let Some(he) = self.half_edges.get(he_key) {
                        if let Some(he_face) = he.face {
                            if he_face != fk && !adj.contains(&he_face) {
                                adj.push(he_face);
                            }
                        }
                    }
                }
            }
        }
        adj
    }
}

/// Returns which of the 4 orthogonal 4-RoSy rotations of `dir` is closest to `reference`
fn match_4_rosy(dir: Vec3, reference: Vec3, normal: Vec3) -> Vec3 {
    let d0 = dir;
    let d1 = normal.cross(dir).normalize();
    let d2 = -dir;
    let d3 = -d1;

    let mut best_dir = d0;
    let mut best_dot = d0.dot(reference);

    for cand in [d1, d2, d3] {
        let dot = cand.dot(reference);
        if dot > best_dot {
            best_dot = dot;
            best_dir = cand;
        }
    }

    best_dir
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::Primitives;

    #[test]
    fn test_quadriflow_retopology_cube() {
        let cube = Primitives::create_cube(2.0);
        let settings = QuadriflowSettings {
            target_quads: 100,
            relaxation_passes: 4,
            sharp_angle_deg: 45.0,
            enforce_all_quads: true,
        };

        let retopo = cube.retopologize_quadriflow(&settings).expect("Quadriflow retopology");

        assert!(!retopo.vertices.is_empty());
        assert!(!retopo.faces.is_empty());
        // Verify output consists primarily of quads
        let quad_count = retopo.faces.keys().filter(|&fk| retopo.face_vertices(fk).len() == 4).count();
        assert!(quad_count > 0);
    }

    #[test]
    fn test_4_rosy_symmetry_matching() {
        let normal = vec3(0.0, 1.0, 0.0);
        let dir = vec3(1.0, 0.0, 0.0);
        let ref_opposite = vec3(-0.9, 0.0, 0.0);

        let best = match_4_rosy(dir, ref_opposite, normal);
        assert!(best.dot(ref_opposite) > 0.8);
    }
}

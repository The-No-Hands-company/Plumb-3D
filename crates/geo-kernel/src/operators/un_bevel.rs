use crate::mesh::{FaceKey, Mesh, MeshError, VertexKey};
use std::collections::HashMap;

impl Mesh {
    /// Collapses and dissolves a chamfer or bevel quad strip, reconstructing the original crisp sharp edge (MeshMachine / HardOps Alt+Shift+B).
    pub fn un_bevel_faces(&mut self, bevel_faces: &[FaceKey]) -> Result<usize, MeshError> {
        if bevel_faces.is_empty() {
            return Ok(0);
        }

        let mut collapsed_count = 0;
        let mut vertex_remap: HashMap<VertexKey, VertexKey> = HashMap::new();

        for &fk in bevel_faces {
            if !self.faces.contains_key(fk) {
                continue;
            }
            let fverts = self.face_vertices(fk);
            if fverts.len() == 4 {
                let p0 = self.vertices[fverts[0]].position;
                let p1 = self.vertices[fverts[1]].position;
                let p2 = self.vertices[fverts[2]].position;
                let _p3 = self.vertices[fverts[3]].position;

                let d01 = (p0 - p1).length_squared();
                let d12 = (p1 - p2).length_squared();

                let (va0, vb0, va1, vb1) = if d01 < d12 {
                    (fverts[0], fverts[1], fverts[3], fverts[2])
                } else {
                    (fverts[1], fverts[2], fverts[0], fverts[3])
                };

                let mid0 = (self.vertices[va0].position + self.vertices[vb0].position) * 0.5;
                let mid1 = (self.vertices[va1].position + self.vertices[vb1].position) * 0.5;

                self.vertices[va0].position = mid0;
                self.vertices[va1].position = mid1;

                vertex_remap.insert(vb0, va0);
                vertex_remap.insert(vb1, va1);

                collapsed_count += 1;
            }
        }

        if collapsed_count > 0 {
            // Collect remapped vertex loops for all non-bevel faces
            let mut new_face_vertex_loops: Vec<Vec<VertexKey>> = Vec::new();
            for (fk, _) in &self.faces {
                if bevel_faces.contains(&fk) {
                    continue;
                }
                let fverts = self.face_vertices(fk);
                let remapped: Vec<VertexKey> = fverts
                    .iter()
                    .map(|&vk| *vertex_remap.get(&vk).unwrap_or(&vk))
                    .collect();

                let mut clean_verts = Vec::new();
                for &vk in &remapped {
                    if clean_verts.last() != Some(&vk) {
                        clean_verts.push(vk);
                    }
                }
                if clean_verts.len() > 1 && clean_verts.first() == clean_verts.last() {
                    clean_verts.pop();
                }
                if clean_verts.len() >= 3 {
                    new_face_vertex_loops.push(clean_verts);
                }
            }

            // Clear old topological elements while preserving vertex coordinates
            self.faces.clear();
            self.edges.clear();
            self.half_edges.clear();
            for v in self.vertices.values_mut() {
                v.half_edges.clear();
            }

            // Reconstruct manifold faces
            for vloop in new_face_vertex_loops {
                let _ = self.add_face(&vloop);
            }

            if let Ok(welded) = self.weld_vertices_by_distance(0.001) {
                *self = welded;
            }
            self.update_face_weighted_normals(true);
        }

        Ok(collapsed_count)
    }
}

#[cfg(test)]
mod tests {
    use crate::primitives::Primitives;

    #[test]
    fn test_un_bevel_chamfer_face() {
        let mut cube = Primitives::create_cube(2.0);
        let edge = cube.edges.keys().next().unwrap();
        let bevel_faces = cube.bevel_edge_with_segments(edge, 0.2, 1).expect("bevel edge");
        assert_eq!(bevel_faces.len(), 1);

        let un_beveled = cube.un_bevel_faces(&bevel_faces).expect("un-bevel faces");
        assert_eq!(un_beveled, 1);
        assert_eq!(cube.faces.len(), 6);
        assert!(cube.validate().is_ok());
    }
}

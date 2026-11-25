use crate::mesh::{FaceKey, Mesh, MeshError, VertexKey};
use std::collections::{HashMap, HashSet};

impl Mesh {
    /// Inverts the vertex winding order and face normals of the specified faces (or entire mesh if empty).
    pub fn flip_face_normals(&mut self, faces_to_flip: &[FaceKey]) -> Result<(), MeshError> {
        let flip_set: HashSet<FaceKey> = if faces_to_flip.is_empty() {
            self.faces.keys().collect()
        } else {
            faces_to_flip.iter().copied().collect()
        };

        let mut new_mesh = Mesh::new();
        let mut old_to_new_v = HashMap::new();

        for (vk, v) in &self.vertices {
            let new_vk = new_mesh.add_vertex_with_uv(v.position, v.uv);
            old_to_new_v.insert(vk, new_vk);
        }

        for (fk, _) in &self.faces {
            let v_keys = self.face_vertices(fk);
            let mut new_v_keys: Vec<VertexKey> = v_keys.iter().map(|k| old_to_new_v[k]).collect();
            if flip_set.contains(&fk) {
                new_v_keys.reverse();
            }
            new_mesh.add_face(&new_v_keys)?;
        }

        for vg in &self.vertex_groups {
            let idx = new_mesh.add_vertex_group(&vg.name);
            for (&vk, &w) in &vg.weights {
                if let Some(&new_vk) = old_to_new_v.get(&vk) {
                    new_mesh.vertex_groups[idx].set_weight(new_vk, w);
                }
            }
        }

        new_mesh.update_normals();
        *self = new_mesh;
        Ok(())
    }
}

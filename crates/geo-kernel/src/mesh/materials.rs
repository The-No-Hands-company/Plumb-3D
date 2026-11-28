use super::types::{FaceKey, Mesh, MeshError};

impl Mesh {
    /// Set the material slot index for a face.
    pub fn set_face_material(&mut self, face: FaceKey, mat_id: u32) -> Result<(), MeshError> {
        let f = self
            .faces
            .get_mut(face)
            .ok_or(MeshError::InvalidFace(face))?;
        f.material_id = mat_id;
        Ok(())
    }

    /// Get the material slot index assigned to a face.
    pub fn get_face_material(&self, face: FaceKey) -> Result<u32, MeshError> {
        let f = self.faces.get(face).ok_or(MeshError::InvalidFace(face))?;
        Ok(f.material_id)
    }

    /// Set the material slot index for a list of faces.
    pub fn set_faces_material(&mut self, faces: &[FaceKey], mat_id: u32) {
        for &fk in faces {
            if let Some(f) = self.faces.get_mut(fk) {
                f.material_id = mat_id;
            }
        }
    }

    /// Find all faces with the specified material slot index.
    pub fn faces_with_material(&self, mat_id: u32) -> Vec<FaceKey> {
        self.faces
            .iter()
            .filter_map(|(fk, f)| {
                if f.material_id == mat_id {
                    Some(fk)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Returns a sorted list of unique material slot indices used by faces in this mesh.
    pub fn unique_material_ids(&self) -> Vec<u32> {
        let mut ids: Vec<u32> = self.faces.values().map(|f| f.material_id).collect();
        ids.sort_unstable();
        ids.dedup();
        if ids.is_empty() {
            vec![0]
        } else {
            ids
        }
    }
}

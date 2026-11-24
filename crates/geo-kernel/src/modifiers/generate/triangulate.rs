use crate::mesh::{Mesh, MeshError};
use crate::modifiers::stack::Modifier;
use crate::operators::{NgonMethod, QuadMethod};

/// Non-destructive Triangulate Modifier (Blender / 3ds Max style)
#[derive(Debug, Clone)]
pub struct TriangulateModifier {
    pub name: String,
    pub enabled: bool,
    pub quad_method: QuadMethod,
    pub ngon_method: NgonMethod,
    pub min_vertices: usize,
}

impl TriangulateModifier {
    pub fn new() -> Self {
        Self {
            name: "Triangulate".to_string(),
            enabled: true,
            quad_method: QuadMethod::Beauty,
            ngon_method: NgonMethod::EarClipping,
            min_vertices: 4,
        }
    }
}

impl Default for TriangulateModifier {
    fn default() -> Self {
        Self::new()
    }
}

impl Modifier for TriangulateModifier {
    fn name(&self) -> &str {
        &self.name
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn apply(&self, input: &Mesh) -> Result<Mesh, MeshError> {
        let mut output = input.clone();
        output.triangulate_faces(&[], self.quad_method, self.ngon_method, self.min_vertices)?;
        output.update_normals();
        Ok(output)
    }

    fn as_triangulate_mut(&mut self) -> Option<&mut TriangulateModifier> {
        Some(self)
    }
}

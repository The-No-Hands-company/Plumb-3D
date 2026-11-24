use crate::mesh::{Mesh, MeshError};
use crate::modifiers::stack::Modifier;

/// Non-destructive Wireframe Modifier (Blender / 3ds Max Lattice style)
#[derive(Debug, Clone)]
pub struct WireframeModifier {
    pub name: String,
    pub enabled: bool,
    pub thickness: f32,
}

impl WireframeModifier {
    pub fn new(thickness: f32) -> Self {
        Self {
            name: "Wireframe".to_string(),
            enabled: true,
            thickness: thickness.max(0.01),
        }
    }
}

impl Default for WireframeModifier {
    fn default() -> Self {
        Self::new(0.05)
    }
}

impl Modifier for WireframeModifier {
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
        input.create_wireframe(self.thickness)
    }

    fn as_wireframe_mut(&mut self) -> Option<&mut WireframeModifier> {
        Some(self)
    }
}

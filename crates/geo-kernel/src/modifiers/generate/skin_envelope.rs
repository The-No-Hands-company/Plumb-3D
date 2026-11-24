use crate::mesh::{Mesh, MeshError};
use crate::modifiers::stack::Modifier;

/// Procedural Skin Envelope Modifier (Blender Skin / ZBrush ZSpheres).
/// Generates a manifold 3D quad hull around wireframe edge/vertex armatures.
#[derive(Debug, Clone)]
pub struct SkinEnvelopeModifier {
    pub name: String,
    pub enabled: bool,
    pub default_radius: f32,
    pub smooth_iterations: usize,
}

impl SkinEnvelopeModifier {
    pub fn new(radius: f32) -> Self {
        Self {
            name: "Skin Envelope".to_string(),
            enabled: true,
            default_radius: radius.max(0.01),
            smooth_iterations: 1,
        }
    }
}

impl Default for SkinEnvelopeModifier {
    fn default() -> Self {
        Self::new(0.25)
    }
}

impl Modifier for SkinEnvelopeModifier {
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
        input.generate_skin_envelope(self.default_radius, self.smooth_iterations)
    }

    fn as_skin_envelope_mut(&mut self) -> Option<&mut SkinEnvelopeModifier> {
        Some(self)
    }
}

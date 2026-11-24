use crate::mesh::{Mesh, MeshError};
use crate::modifiers::stack::Modifier;

/// Voxel Remesh Modifier (DynaMesh / Blender Voxel Remesher).
/// Re-topologizes arbitrary meshes into uniform, watertight, 2-manifold surfaces.
#[derive(Debug, Clone)]
pub struct RemeshModifier {
    pub name: String,
    pub enabled: bool,
    pub voxel_size: f32,
    pub smooth_shading: bool,
}

impl RemeshModifier {
    pub fn new(voxel_size: f32) -> Self {
        Self {
            name: "Voxel Remesh".to_string(),
            enabled: true,
            voxel_size: voxel_size.max(0.01),
            smooth_shading: true,
        }
    }
}

impl Default for RemeshModifier {
    fn default() -> Self {
        Self::new(0.1)
    }
}

impl Modifier for RemeshModifier {
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
        input.voxel_remesh(self.voxel_size)
    }

    fn as_remesh_mut(&mut self) -> Option<&mut RemeshModifier> {
        Some(self)
    }
}

use crate::decimate::DecimateModifier;
use crate::mesh::{Mesh, MeshError};
use crate::modifiers::deform::*;
use crate::modifiers::generate::*;
use crate::modifiers::topology::*;

/// Trait defining a non-destructive parametric geometric modifier (3ds Max style)
pub trait Modifier: std::fmt::Debug + Send + Sync {
    fn name(&self) -> &str;
    fn is_enabled(&self) -> bool;
    fn set_enabled(&mut self, enabled: bool);
    fn apply(&self, input: &Mesh) -> Result<Mesh, MeshError>;

    fn as_skin_mut(&mut self) -> Option<&mut SkinModifier> {
        None
    }

    fn as_skin_envelope_mut(&mut self) -> Option<&mut SkinEnvelopeModifier> {
        None
    }

    fn as_remesh_mut(&mut self) -> Option<&mut RemeshModifier> {
        None
    }

    fn as_curve_mut(&mut self) -> Option<&mut CurveModifier> {
        None
    }

    fn as_boolean_mut(&mut self) -> Option<&mut BooleanModifier> {
        None
    }

    fn as_solidify_mut(&mut self) -> Option<&mut SolidifyModifier> {
        None
    }

    fn as_array_mut(&mut self) -> Option<&mut ArrayModifier> {
        None
    }

    fn as_smooth_mut(&mut self) -> Option<&mut SmoothModifier> {
        None
    }

    fn as_shrinkwrap_mut(&mut self) -> Option<&mut ShrinkwrapModifier> {
        None
    }

    fn as_simple_deform_mut(&mut self) -> Option<&mut SimpleDeformModifier> {
        None
    }

    fn as_screw_mut(&mut self) -> Option<&mut ScrewModifier> {
        None
    }

    fn as_triangulate_mut(&mut self) -> Option<&mut TriangulateModifier> {
        None
    }

    fn as_wireframe_mut(&mut self) -> Option<&mut WireframeModifier> {
        None
    }

    fn as_weighted_normal_mut(&mut self) -> Option<&mut WeightedNormalModifier> {
        None
    }

    fn as_decimate_mut(&mut self) -> Option<&mut DecimateModifier> {
        None
    }

    fn as_morpher(&self) -> Option<&MorpherModifier> {
        None
    }

    fn as_morpher_mut(&mut self) -> Option<&mut MorpherModifier> {
        None
    }

    fn as_cloth_mut(&mut self) -> Option<&mut ClothModifier> {
        None
    }

    fn as_particle_system_mut(&mut self) -> Option<&mut ParticleSystemModifier> {
        None
    }
}




/// 3ds Max style Modifier Stack evaluation manager
#[derive(Debug, Default)]
pub struct ModifierStack {
    modifiers: Vec<Box<dyn Modifier>>,
    dirty: bool,
    cached_output: Option<Mesh>,
}

impl ModifierStack {
    pub fn new() -> Self {
        Self {
            modifiers: Vec::new(),
            dirty: true,
            cached_output: None,
        }
    }

    pub fn add_modifier(&mut self, modifier: Box<dyn Modifier>) {
        self.modifiers.push(modifier);
        self.dirty = true;
    }

    pub fn remove_modifier(&mut self, index: usize) -> Box<dyn Modifier> {
        self.dirty = true;
        self.modifiers.remove(index)
    }

    pub fn clear(&mut self) {
        self.modifiers.clear();
        self.cached_output = None;
        self.dirty = true;
    }

    pub fn modifiers(&self) -> &[Box<dyn Modifier>] {
        &self.modifiers
    }

    pub fn modifiers_mut(&mut self) -> &mut [Box<dyn Modifier>] {
        self.dirty = true;
        &mut self.modifiers
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Evaluate the stack sequentially from base mesh to final deformed mesh.
    pub fn evaluate(&mut self, base_mesh: &Mesh) -> Result<&Mesh, MeshError> {
        if !self.dirty && self.cached_output.is_some() {
            return Ok(self.cached_output.as_ref().unwrap());
        }

        let mut current_mesh = base_mesh.clone();
        for modifier in &self.modifiers {
            if modifier.is_enabled() {
                current_mesh = modifier.apply(&current_mesh)?;
            }
        }

        self.cached_output = Some(current_mesh);
        self.dirty = false;
        Ok(self.cached_output.as_ref().unwrap())
    }
}

use crate::boolean::BooleanOp;
use crate::mesh::{Mesh, MeshError, VertexKey};
use crate::modifiers::stack::Modifier;
use std::collections::HashMap;

/// Non-destructive Mirror Modifier (symmetrically reflects mesh across X, Y, or Z plane)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MirrorAxis {
    X,
    Y,
    Z,
}

#[derive(Debug, Clone)]
pub struct MirrorModifier {
    pub axis: MirrorAxis,
    pub enabled: bool,
}

impl MirrorModifier {
    pub fn new(axis: MirrorAxis) -> Self {
        Self { axis, enabled: true }
    }
}

impl Modifier for MirrorModifier {
    fn name(&self) -> &str {
        "Mirror"
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn apply(&self, input: &Mesh) -> Result<Mesh, MeshError> {
        let mut mirrored = input.clone();

        // Map original VertexKey -> mirrored VertexKey
        let mut old_to_mirrored_v = HashMap::new();
        for (vk, v) in &input.vertices {
            let mut pos = v.position;
            match self.axis {
                MirrorAxis::X => pos.x = -pos.x,
                MirrorAxis::Y => pos.y = -pos.y,
                MirrorAxis::Z => pos.z = -pos.z,
            }
            let new_vk = mirrored.add_vertex_with_uv(pos, v.uv);
            old_to_mirrored_v.insert(vk, new_vk);
        }

        // Add mirrored faces with reversed winding so normals point outwards
        for (fk, _) in &input.faces {
            let v_keys = input.face_vertices(fk);
            let mut mirrored_v_keys: Vec<VertexKey> = v_keys
                .iter()
                .map(|vk| old_to_mirrored_v[vk])
                .collect();
            mirrored_v_keys.reverse(); // Reverse winding for reflection

            mirrored.add_face(&mirrored_v_keys)?;
        }

        mirrored.update_normals();
        Ok(mirrored)
    }
}

/// Non-destructive Procedural Displacement Modifier (displaces vertices along normals)
#[derive(Debug, Clone)]



pub struct BooleanModifier {
    pub operand: Mesh,
    pub operation: BooleanOp,
    pub target_node_name: Option<String>,
    pub enabled: bool,
}

impl BooleanModifier {
    pub fn new(operand: Mesh, operation: BooleanOp) -> Self {
        Self {
            operand,
            operation,
            target_node_name: None,
            enabled: true,
        }
    }

    pub fn with_target(operand: Mesh, operation: BooleanOp, target_node_name: impl Into<String>) -> Self {
        Self {
            operand,
            operation,
            target_node_name: Some(target_node_name.into()),
            enabled: true,
        }
    }
}

impl Modifier for BooleanModifier {
    fn name(&self) -> &str {
        match self.operation {
            BooleanOp::Union => "Boolean Union",
            BooleanOp::Difference => "Boolean Difference",
            BooleanOp::Intersection => "Boolean Intersection",
        }
    }

    fn as_boolean_mut(&mut self) -> Option<&mut BooleanModifier> {
        Some(self)
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn apply(&self, input: &Mesh) -> Result<Mesh, MeshError> {
        input.boolean(&self.operand, self.operation)
    }
}

/// Non-destructive Vertex Weld Modifier (3ds Max Vertex Weld / Blender Merge by Distance)
#[derive(Debug, Clone)]


pub struct WeldModifier {
    pub distance: f32,
    pub enabled: bool,
}

impl WeldModifier {
    pub fn new(distance: f32) -> Self {
        Self {
            distance,
            enabled: true,
        }
    }
}

impl Modifier for WeldModifier {
    fn name(&self) -> &str {
        "Vertex Weld"
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn apply(&self, input: &Mesh) -> Result<Mesh, MeshError> {
        input.weld_vertices_by_distance(self.distance)
    }
}

/// Weighting algorithm for WeightedNormalModifier (3ds Max / Blender)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeightedNormalMode {
    FaceArea,
    CornerAngle,
    AreaAndAngle,
}

/// Non-destructive Weighted Normal Modifier (3ds Max / Blender)
/// Adjusts vertex / split corner normals based on face area and corner angles,
/// providing flat shading on large polygons and smooth shading across bevels.
#[derive(Debug, Clone)]
pub struct WeightedNormalModifier {
    pub name: String,
    pub enabled: bool,
    pub mode: WeightedNormalMode,
    pub weight: f32,
    pub keep_sharp: bool,
}

impl WeightedNormalModifier {
    pub fn new(mode: WeightedNormalMode) -> Self {
        Self {
            name: "Weighted Normal".to_string(),
            enabled: true,
            mode,
            weight: 1.0,
            keep_sharp: true,
        }
    }
}

impl Default for WeightedNormalModifier {
    fn default() -> Self {
        Self::new(WeightedNormalMode::AreaAndAngle)
    }
}

impl Modifier for WeightedNormalModifier {
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
        output.compute_weighted_normals(self.mode, self.weight);
        Ok(output)
    }

    fn as_weighted_normal_mut(&mut self) -> Option<&mut WeightedNormalModifier> {
        Some(self)
    }
}

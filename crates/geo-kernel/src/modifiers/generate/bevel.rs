use crate::mesh::{Mesh, MeshError};
use crate::modifiers::stack::Modifier;

#[derive(Debug, Clone)]
pub struct BevelModifier {
    pub width: f32,
    pub min_angle_radians: f32,
    pub enabled: bool,
}

impl BevelModifier {
    pub fn new(width: f32) -> Self {
        Self {
            width,
            min_angle_radians: 0.5,
            enabled: true,
        }
    }

    pub fn with_angle(mut self, radians: f32) -> Self {
        self.min_angle_radians = radians;
        self
    }
}

impl Modifier for BevelModifier {
    fn name(&self) -> &str {
        "Bevel"
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn apply(&self, input: &Mesh) -> Result<Mesh, MeshError> {
        let mut output = input.clone();
        let edge_keys: Vec<_> = output.edges.keys().collect();

        for ek in edge_keys {
            if let Some(edge) = output.edges.get(ek) {
                let h1 = edge.half_edge;
                let h2 = output.half_edges[h1].twin;
                if let (Some(f1), Some(f2)) = (output.half_edges[h1].face, output.half_edges[h2].face) {
                    let n1 = output.faces[f1].normal;
                    let n2 = output.faces[f2].normal;
                    let angle = n1.dot(n2).clamp(-1.0, 1.0).acos();
                    if angle >= self.min_angle_radians {
                        let _ = output.bevel_edge(ek, self.width);
                    }
                }
            }
        }

        output.update_normals();
        Ok(output)
    }
}

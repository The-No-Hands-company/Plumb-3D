use crate::graph::GeoNodeGraph;
use crate::nodes::GroupInputNode;
use geo_kernel::{modifiers::Modifier, Mesh, MeshError};

/// Non-destructive procedural modifier that evaluates a `GeoNodeGraph`
/// dynamically inside a scene object's `ModifierStack`.
#[derive(Debug, Clone)]
pub struct GeometryNodesModifier {
    pub name: String,
    pub enabled: bool,
    pub graph: GeoNodeGraph,
}

impl GeometryNodesModifier {
    pub fn new(name: impl Into<String>, graph: GeoNodeGraph) -> Self {
        Self {
            name: name.into(),
            enabled: true,
            graph,
        }
    }
}

impl Modifier for GeometryNodesModifier {
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
        let mut graph_copy = self.graph.clone();

        // Inject input mesh into any GroupInputNode present in the graph
        for (_, node) in &mut graph_copy.nodes {
            if let Some(any) = node.op.as_any_mut() {
                if let Some(group_in) = any.downcast_mut::<GroupInputNode>() {
                    group_in.input_mesh = Some(input.clone());
                }
            }
        }

        match graph_copy.evaluate() {
            Ok(Some(mesh)) => Ok(mesh),
            Ok(None) => Ok(input.clone()),
            Err(e) => Err(MeshError::InvariantFailed(format!("GeoNodes evaluation failed: {:?}", e))),
        }
    }
}

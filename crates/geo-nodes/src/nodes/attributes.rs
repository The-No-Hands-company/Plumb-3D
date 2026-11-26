use crate::nodes::traits::GeoNodeOp;
use crate::types::*;

/// Captures an evaluated attribute field onto geometry elements (vertices/faces/points)
#[derive(Debug, Clone)]
pub struct CaptureAttributeNode {
    pub attribute_name: String,
    pub domain: AttributeDomain,
}

impl Default for CaptureAttributeNode {
    fn default() -> Self {
        Self {
            attribute_name: "Attribute".to_string(),
            domain: AttributeDomain::Vertex,
        }
    }
}

impl GeoNodeOp for CaptureAttributeNode {
    fn name(&self) -> &str {
        "Capture Attribute"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::GeometryOps
    }

    fn inputs(&self) -> Vec<NodeSocket> {
        vec![
            NodeSocket::geometry("Geometry"),
            NodeSocket::float("Value", 1.0),
        ]
    }

    fn outputs(&self) -> Vec<NodeSocket> {
        vec![
            NodeSocket::geometry("Geometry"),
            NodeSocket::field("Attribute"),
        ]
    }

    fn execute(&self, inputs: &[SocketValue]) -> Result<Vec<SocketValue>, NodeGraphError> {
        let mesh = match inputs.first() {
            Some(SocketValue::Geometry(Some(m))) => m.clone(),
            _ => return Err(NodeGraphError::ExecutionError("Capture Attribute requires valid input geometry".into())),
        };

        let val = inputs.get(1).map(|s| s.as_float()).unwrap_or(1.0);
        let vert_count = mesh.vertices.len();
        let field_vals = vec![val; vert_count];

        // Store into mesh as a vertex group
        let mut modified_mesh = mesh;
        let g_idx = modified_mesh.add_vertex_group(&self.attribute_name);
        for vk in modified_mesh.vertices.keys() {
            modified_mesh.vertex_groups[g_idx].set_weight(vk, val);
        }

        let container = AttributeContainer::new_float(&self.attribute_name, self.domain, field_vals);

        Ok(vec![
            SocketValue::Geometry(Some(modified_mesh)),
            SocketValue::Field(container),
        ])
    }

    fn clone_box(&self) -> Box<dyn GeoNodeOp> {
        Box::new(self.clone())
    }
}

/// Transfers / samples attributes from a source geometry to a target geometry based on nearest surface proximity
#[derive(Debug, Clone, Default)]
pub struct TransferAttributeNode {
    pub attribute_name: String,
}

impl GeoNodeOp for TransferAttributeNode {
    fn name(&self) -> &str {
        "Transfer Attribute"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::GeometryOps
    }

    fn inputs(&self) -> Vec<NodeSocket> {
        vec![
            NodeSocket::geometry("Target Mesh"),
            NodeSocket::geometry("Source Mesh"),
            NodeSocket::field("Source Field"),
        ]
    }

    fn outputs(&self) -> Vec<NodeSocket> {
        vec![
            NodeSocket::geometry("Geometry"),
            NodeSocket::field("Attribute"),
        ]
    }

    fn execute(&self, inputs: &[SocketValue]) -> Result<Vec<SocketValue>, NodeGraphError> {
        let mut target_mesh = match inputs.first() {
            Some(SocketValue::Geometry(Some(m))) => m.clone(),
            _ => return Err(NodeGraphError::ExecutionError("Target geometry required for attribute transfer".into())),
        };

        let source_mesh = match inputs.get(1) {
            Some(SocketValue::Geometry(Some(m))) => m,
            _ => return Err(NodeGraphError::ExecutionError("Source geometry required for attribute transfer".into())),
        };

        let source_field = inputs.get(2).and_then(|s| s.as_field());

        // For each vertex in target mesh, find closest vertex in source mesh and transfer value
        let mut transferred_floats = Vec::with_capacity(target_mesh.vertices.len());
        let g_name = source_field.map(|f| f.name.as_str()).unwrap_or("Transferred");
        let g_idx = target_mesh.add_vertex_group(g_name);

        for (tgt_vk, tgt_v) in &target_mesh.vertices {
            let mut nearest_idx = 0;
            let mut best_dist_sq = f32::INFINITY;

            for (src_idx, (_, src_v)) in source_mesh.vertices.iter().enumerate() {
                let d_sq = (tgt_v.position - src_v.position).length_squared();
                if d_sq < best_dist_sq {
                    best_dist_sq = d_sq;
                    nearest_idx = src_idx;
                }
            }

            let val = source_field.and_then(|f| f.get_float(nearest_idx)).unwrap_or(0.0);
            target_mesh.vertex_groups[g_idx].set_weight(tgt_vk, val);
            transferred_floats.push(val);
        }

        let container = AttributeContainer::new_float(g_name, AttributeDomain::Vertex, transferred_floats);

        Ok(vec![
            SocketValue::Geometry(Some(target_mesh)),
            SocketValue::Field(container),
        ])
    }

    fn clone_box(&self) -> Box<dyn GeoNodeOp> {
        Box::new(self.clone())
    }
}

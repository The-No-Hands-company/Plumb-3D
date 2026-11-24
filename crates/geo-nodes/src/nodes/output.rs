use crate::nodes::traits::GeoNodeOp;
use crate::types::*;

#[derive(Debug, Clone)]
pub struct GroupOutputNode;

impl GeoNodeOp for GroupOutputNode {
    fn name(&self) -> &str {
        "Group Output"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::Output
    }

    fn inputs(&self) -> Vec<NodeSocket> {
        vec![NodeSocket::geometry("Geometry")]
    }

    fn outputs(&self) -> Vec<NodeSocket> {
        vec![]
    }

    fn execute(&self, inputs: &[SocketValue]) -> Result<Vec<SocketValue>, NodeGraphError> {
        let geo = inputs.first().cloned().unwrap_or(SocketValue::Geometry(None));
        Ok(vec![geo])
    }

    fn clone_box(&self) -> Box<dyn GeoNodeOp> {
        Box::new(self.clone())
    }
}

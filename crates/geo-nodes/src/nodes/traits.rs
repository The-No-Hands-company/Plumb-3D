use crate::types::*;
use std::any::Any;

/// Trait implemented by any procedural geometry node
pub trait GeoNodeOp: std::fmt::Debug + Send + Sync {
    fn name(&self) -> &str;
    fn category(&self) -> NodeCategory;
    fn inputs(&self) -> Vec<NodeSocket>;
    fn outputs(&self) -> Vec<NodeSocket>;
    fn execute(&self, inputs: &[SocketValue]) -> Result<Vec<SocketValue>, NodeGraphError>;

    fn as_any_mut(&mut self) -> Option<&mut dyn Any> {
        None
    }

    fn clone_box(&self) -> Box<dyn GeoNodeOp>;
}

impl Clone for Box<dyn GeoNodeOp> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

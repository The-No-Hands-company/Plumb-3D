use crate::nodes::*;
use crate::types::*;
use geo_kernel::Mesh;
use slotmap::SlotMap;
use std::collections::{HashMap, VecDeque};

/// An instantiated visual node with 2D visual position and dynamic socket states
#[derive(Debug, Clone)]
pub struct GeoNode {
    pub op: Box<dyn GeoNodeOp>,
    pub position: [f32; 2],
    pub input_sockets: Vec<NodeSocket>,
    pub output_sockets: Vec<NodeSocket>,
}

impl GeoNode {
    pub fn new(op: Box<dyn GeoNodeOp>, position: [f32; 2]) -> Self {
        let input_sockets = op.inputs();
        let output_sockets = op.outputs();
        Self {
            op,
            position,
            input_sockets,
            output_sockets,
        }
    }
}

/// Visual Geometry Node Graph (Blender Geo Nodes / Unreal Blueprints style)
#[derive(Debug, Clone)]
pub struct GeoNodeGraph {
    pub nodes: SlotMap<NodeKey, GeoNode>,
    pub links: Vec<NodeLink>,
    pub output_node: Option<NodeKey>,
    dirty: bool,
    cached_mesh: Option<Mesh>,
}

impl Default for GeoNodeGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl GeoNodeGraph {
    /// Creates a new node graph initialized with a default Cube -> Group Output pipeline
    pub fn new() -> Self {
        let mut graph = Self {
            nodes: SlotMap::with_key(),
            links: Vec::new(),
            output_node: None,
            dirty: true,
            cached_mesh: None,
        };

        let cube_key = graph.add_node(Box::new(CubeNode), [-200.0, 100.0]);
        let out_key = graph.add_node(Box::new(GroupOutputNode), [200.0, 100.0]);
        graph.output_node = Some(out_key);

        let _ = graph.link(cube_key, 0, out_key, 0);
        graph
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn add_node(&mut self, op: Box<dyn GeoNodeOp>, position: [f32; 2]) -> NodeKey {
        self.dirty = true;
        let is_output = op.category() == NodeCategory::Output;
        let key = self.nodes.insert(GeoNode::new(op, position));
        if is_output && self.output_node.is_none() {
            self.output_node = Some(key);
        }
        key
    }

    pub fn remove_node(&mut self, key: NodeKey) -> Option<GeoNode> {
        self.dirty = true;
        self.links
            .retain(|l| l.from_node != key && l.to_node != key);
        if self.output_node == Some(key) {
            self.output_node = None;
        }
        self.nodes.remove(key)
    }

    /// Link an output socket to an input socket
    pub fn link(
        &mut self,
        from_node: NodeKey,
        from_socket: usize,
        to_node: NodeKey,
        to_socket: usize,
    ) -> Result<(), NodeGraphError> {
        if from_node == to_node {
            return Err(NodeGraphError::CyclicDependency);
        }

        // Validate socket indices
        let Some(from_n) = self.nodes.get(from_node) else {
            return Err(NodeGraphError::ExecutionError("Source node missing".into()));
        };
        if from_socket >= from_n.output_sockets.len() {
            return Err(NodeGraphError::InvalidSocket {
                node_name: from_n.op.name().to_string(),
                index: from_socket,
            });
        }

        let Some(to_n) = self.nodes.get(to_node) else {
            return Err(NodeGraphError::ExecutionError(
                "Destination node missing".into(),
            ));
        };
        if to_socket >= to_n.input_sockets.len() {
            return Err(NodeGraphError::InvalidSocket {
                node_name: to_n.op.name().to_string(),
                index: to_socket,
            });
        }

        // Disconnect existing link into target input socket (single input connection rule)
        self.links
            .retain(|l| !(l.to_node == to_node && l.to_socket == to_socket));

        let link = NodeLink {
            from_node,
            from_socket,
            to_node,
            to_socket,
        };
        self.links.push(link);

        // Verify DAG invariant (cycle detection)
        if self.topological_sort().is_err() {
            self.links.pop();
            return Err(NodeGraphError::CyclicDependency);
        }

        self.dirty = true;
        Ok(())
    }

    /// Unlink all connections going into `(to_node, to_socket)`
    pub fn unlink(&mut self, to_node: NodeKey, to_socket: usize) {
        let prev_len = self.links.len();
        self.links
            .retain(|l| !(l.to_node == to_node && l.to_socket == to_socket));
        if self.links.len() != prev_len {
            self.dirty = true;
        }
    }

    /// Computes a topological ordering of node execution using Kahn's algorithm
    pub fn topological_sort(&self) -> Result<Vec<NodeKey>, NodeGraphError> {
        let mut in_degree: HashMap<NodeKey, usize> = HashMap::new();
        let mut adj: HashMap<NodeKey, Vec<NodeKey>> = HashMap::new();

        for key in self.nodes.keys() {
            in_degree.insert(key, 0);
            adj.insert(key, Vec::new());
        }

        for link in &self.links {
            if self.nodes.contains_key(link.from_node) && self.nodes.contains_key(link.to_node) {
                *in_degree.entry(link.to_node).or_insert(0) += 1;
                adj.entry(link.from_node).or_default().push(link.to_node);
            }
        }

        let mut queue: VecDeque<NodeKey> = in_degree
            .iter()
            .filter(|&(_, &deg)| deg == 0)
            .map(|(&k, _)| k)
            .collect();

        let mut sorted: Vec<NodeKey> = Vec::with_capacity(self.nodes.len());

        while let Some(u) = queue.pop_front() {
            sorted.push(u);
            if let Some(neighbors) = adj.get(&u) {
                for &v in neighbors {
                    if let Some(deg) = in_degree.get_mut(&v) {
                        *deg -= 1;
                        if *deg == 0 {
                            queue.push_back(v);
                        }
                    }
                }
            }
        }

        if sorted.len() != self.nodes.len() {
            return Err(NodeGraphError::CyclicDependency);
        }

        Ok(sorted)
    }

    /// Evaluates the entire node graph and produces the output geometry
    pub fn evaluate(&mut self) -> Result<Option<Mesh>, NodeGraphError> {
        if !self.dirty && self.cached_mesh.is_some() {
            return Ok(self.cached_mesh.clone());
        }

        let out_key = self.output_node.ok_or(NodeGraphError::MissingOutput)?;
        let sorted_keys = self.topological_sort()?;

        // Map: (NodeKey, socket_idx) -> SocketValue
        let mut socket_values: HashMap<(NodeKey, usize), SocketValue> = HashMap::new();

        for node_key in sorted_keys {
            let Some(node) = self.nodes.get(node_key) else {
                continue;
            };

            // Collect input values
            let mut inputs = Vec::with_capacity(node.input_sockets.len());
            for (idx, socket) in node.input_sockets.iter().enumerate() {
                // Check if an incoming link feeds this input socket
                let incoming = self
                    .links
                    .iter()
                    .find(|l| l.to_node == node_key && l.to_socket == idx);
                let val = if let Some(link) = incoming {
                    socket_values
                        .get(&(link.from_node, link.from_socket))
                        .cloned()
                        .unwrap_or_else(|| socket.value.clone())
                } else {
                    socket.value.clone()
                };
                inputs.push(val);
            }

            // Execute node operator
            let outputs = node.op.execute(&inputs)?;

            // Store outputs
            for (out_idx, out_val) in outputs.into_iter().enumerate() {
                socket_values.insert((node_key, out_idx), out_val);
            }
        }

        let output_mesh = socket_values
            .remove(&(out_key, 0))
            .and_then(|v| v.to_geometry());

        self.cached_mesh = output_mesh.clone();
        self.dirty = false;
        Ok(output_mesh)
    }
}

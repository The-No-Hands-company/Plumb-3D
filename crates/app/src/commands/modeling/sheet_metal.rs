//! Undoable Commands for Sheet Metal Design and CNC Flat Pattern Unfolding.

use geo_kernel::{
    BaseFlangeSettings, EdgeFlangeSettings, EdgeKey, FlatPattern, Mesh, SheetMetalBuilder,
    SheetMetalParams,
};
use graph_engine::{Command, NodeData, NodeId, SceneGraph};

/// Undoable command to create a base sheet metal plate.
#[derive(Debug)]
pub struct CreateBaseFlangeCommand {
    node_id: NodeId,
    settings: BaseFlangeSettings,
    old_mesh: Option<Mesh>,
}

impl CreateBaseFlangeCommand {
    pub fn new(node_id: NodeId, settings: BaseFlangeSettings) -> Self {
        Self {
            node_id,
            settings,
            old_mesh: None,
        }
    }
}

impl Command for CreateBaseFlangeCommand {
    fn name(&self) -> &str {
        "Create Base Flange"
    }

    fn execute(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        let node = scene
            .nodes
            .get_mut(self.node_id)
            .ok_or_else(|| format!("Node {:?} not found", self.node_id))?;

        let current_mesh = match &mut node.data {
            NodeData::Mesh { base_mesh, .. } => base_mesh,
            _ => return Err(format!("Node {:?} is not a mesh", self.node_id)),
        };

        self.old_mesh = Some(current_mesh.clone());
        let new_mesh = SheetMetalBuilder::create_base_flange(&self.settings);
        *current_mesh = new_mesh;

        Ok(())
    }

    fn undo(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        if let Some(old) = self.old_mesh.take() {
            let node = scene
                .nodes
                .get_mut(self.node_id)
                .ok_or_else(|| format!("Node {:?} not found", self.node_id))?;

            if let NodeData::Mesh { base_mesh, .. } = &mut node.data {
                *base_mesh = old;
            }
        }
        Ok(())
    }
}

/// Undoable command to bend an edge flange onto a sheet metal part.
#[derive(Debug)]
pub struct AddEdgeFlangeCommand {
    node_id: NodeId,
    edge: EdgeKey,
    settings: EdgeFlangeSettings,
    params: SheetMetalParams,
    old_mesh: Option<Mesh>,
}

impl AddEdgeFlangeCommand {
    pub fn new(
        node_id: NodeId,
        edge: EdgeKey,
        settings: EdgeFlangeSettings,
        params: SheetMetalParams,
    ) -> Self {
        Self {
            node_id,
            edge,
            settings,
            params,
            old_mesh: None,
        }
    }
}

impl Command for AddEdgeFlangeCommand {
    fn name(&self) -> &str {
        "Add Edge Flange"
    }

    fn execute(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        let node = scene
            .nodes
            .get_mut(self.node_id)
            .ok_or_else(|| format!("Node {:?} not found", self.node_id))?;

        let mesh = match &mut node.data {
            NodeData::Mesh { base_mesh, .. } => base_mesh,
            _ => return Err(format!("Node {:?} is not a mesh", self.node_id)),
        };

        self.old_mesh = Some(mesh.clone());

        let (edge_x, edge_len) = if let Some(edge_data) = mesh.edges.get(self.edge) {
            if let Some(he) = mesh.half_edges.get(edge_data.half_edge) {
                let p0 = mesh.vertices.get(he.origin).map(|v| v.position).unwrap_or_default();
                let p1 = mesh.vertices.get(he.target).map(|v| v.position).unwrap_or_default();
                ((p0.x + p1.x) * 0.5, (p0 - p1).length().max(1.0))
            } else {
                (0.0, 50.0)
            }
        } else {
            (0.0, 50.0)
        };

        let (new_mesh, _bend) = SheetMetalBuilder::add_edge_flange(
            mesh,
            &self.settings,
            &self.params,
            edge_x,
            edge_len,
        );
        *mesh = new_mesh;

        Ok(())
    }

    fn undo(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        if let Some(old) = self.old_mesh.take() {
            let node = scene
                .nodes
                .get_mut(self.node_id)
                .ok_or_else(|| format!("Node {:?} not found", self.node_id))?;

            if let NodeData::Mesh { base_mesh, .. } = &mut node.data {
                *base_mesh = old;
            }
        }
        Ok(())
    }
}

/// Command to unfold a 3D folded sheet metal part into a 2D CNC flat pattern.
#[derive(Debug)]
pub struct UnfoldSheetMetalCommand {
    node_id: NodeId,
    params: SheetMetalParams,
    pub flat_pattern: Option<FlatPattern>,
    pub dxf_output: Option<String>,
    pub svg_output: Option<String>,
}

impl UnfoldSheetMetalCommand {
    pub fn new(node_id: NodeId, params: SheetMetalParams) -> Self {
        Self {
            node_id,
            params,
            flat_pattern: None,
            dxf_output: None,
            svg_output: None,
        }
    }
}

impl Command for UnfoldSheetMetalCommand {
    fn name(&self) -> &str {
        "Unfold Sheet Metal (Flat Pattern)"
    }

    fn execute(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        let node = scene
            .nodes
            .get(self.node_id)
            .ok_or_else(|| format!("Node {:?} not found", self.node_id))?;

        let mesh = match &node.data {
            NodeData::Mesh { base_mesh, .. } => base_mesh,
            _ => return Err(format!("Node {:?} is not a mesh", self.node_id)),
        };

        let pattern = FlatPattern::unfold_mesh(mesh, &self.params)?;
        self.dxf_output = Some(pattern.export_dxf());
        self.svg_output = Some(pattern.export_svg());
        self.flat_pattern = Some(pattern);

        Ok(())
    }

    fn undo(&mut self, _scene: &mut SceneGraph) -> Result<(), String> {
        self.flat_pattern = None;
        self.dxf_output = None;
        self.svg_output = None;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core_math::Transform;
    use geo_kernel::{BendDirection, EdgeFlangeSettings, Primitives, SheetMetalParams};
    use graph_engine::{NodeData, SceneGraph, SceneNode};

    #[test]
    fn test_create_base_flange_command_and_undo() {
        let mut scene = SceneGraph::new();
        let initial_mesh = Primitives::create_cube(1.0);
        let node_id = scene.add_node(SceneNode::new_mesh(
            "SheetPart",
            Transform::IDENTITY,
            initial_mesh,
        ));

        let settings = BaseFlangeSettings {
            width: 100.0,
            length: 80.0,
            params: SheetMetalParams::new(1.5, 2.0, 0.44),
        };

        let mut cmd = CreateBaseFlangeCommand::new(node_id, settings);
        cmd.execute(&mut scene).expect("Base flange creation should succeed");

        let node = scene.nodes.get(node_id).unwrap();
        if let NodeData::Mesh { base_mesh, .. } = &node.data {
            let (min, max) = base_mesh.bounding_box();
            assert!((max.x - min.x - 100.0).abs() < 1e-3);
            assert!((max.y - min.y - 80.0).abs() < 1e-3);
        }

        // Undo
        cmd.undo(&mut scene).expect("Undo should succeed");
        let node_undo = scene.nodes.get(node_id).unwrap();
        if let NodeData::Mesh { base_mesh, .. } = &node_undo.data {
            let (min, max) = base_mesh.bounding_box();
            assert!((max.x - min.x - 1.0).abs() < 1e-3);
        }
    }

    #[test]
    fn test_add_edge_flange_and_unfold_pipeline() {
        let mut scene = SceneGraph::new();
        let base_plate = SheetMetalBuilder::create_base_flange(&BaseFlangeSettings {
            width: 80.0,
            length: 60.0,
            params: SheetMetalParams::new(2.0, 2.5, 0.44),
        });
        let edge_key = base_plate.edges.keys().next().unwrap();

        let node_id = scene.add_node(SceneNode::new_mesh(
            "FoldedBracket",
            Transform::IDENTITY,
            base_plate,
        ));

        let flange_settings = EdgeFlangeSettings {
            length: 25.0,
            angle_deg: 90.0,
            bend_radius: Some(2.5),
            direction: BendDirection::Up,
        };
        let params = SheetMetalParams::new(2.0, 2.5, 0.44);

        let mut flange_cmd = AddEdgeFlangeCommand::new(node_id, edge_key, flange_settings, params);
        flange_cmd.execute(&mut scene).expect("Edge flange addition should succeed");

        // Now unfold the sheet metal bracket
        let mut unfold_cmd = UnfoldSheetMetalCommand::new(node_id, params);
        unfold_cmd.execute(&mut scene).expect("Unfolding should succeed");

        assert!(unfold_cmd.flat_pattern.is_some());
        let dxf = unfold_cmd.dxf_output.expect("DXF should be generated");
        assert!(dxf.contains("SECTION"));
        assert!(dxf.contains("EOF"));

        let svg = unfold_cmd.svg_output.expect("SVG should be generated");
        assert!(svg.contains("<svg"));
        assert!(svg.contains("</svg>"));
    }
}


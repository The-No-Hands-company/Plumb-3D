//! Undoable Command for Generating Technical 2D Engineering Drawing Sheets.

use geo_kernel::{DrawingSheet, ProjectionStandard, SheetFormat};
use graph_engine::{Command, NodeData, NodeId, SceneGraph};

#[derive(Debug)]
pub struct GenerateDrawingSheetCommand {
    node_id: NodeId,
    format: SheetFormat,
    projection: ProjectionStandard,
    scale: f32,
    pub generated_sheet: Option<DrawingSheet>,
    pub svg_output: Option<String>,
    pub dxf_output: Option<String>,
}

impl GenerateDrawingSheetCommand {
    pub fn new(
        node_id: NodeId,
        format: SheetFormat,
        projection: ProjectionStandard,
        scale: f32,
    ) -> Self {
        Self {
            node_id,
            format,
            projection,
            scale,
            generated_sheet: None,
            svg_output: None,
            dxf_output: None,
        }
    }
}

impl Command for GenerateDrawingSheetCommand {
    fn name(&self) -> &str {
        "Generate 2D Drawing Sheet"
    }

    fn execute(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        let node = scene
            .nodes
            .get(self.node_id)
            .ok_or_else(|| format!("Node {:?} not found", self.node_id))?;

        let base_mesh = match &node.data {
            NodeData::Mesh { base_mesh, .. } => base_mesh,
            _ => return Err(format!("Node {:?} is not a polygon mesh", self.node_id)),
        };

        let mut sheet = DrawingSheet::new(self.format, self.projection);
        sheet.title_block.title = node.name.to_uppercase();
        sheet.generate_multi_view(base_mesh, self.scale);

        self.svg_output = Some(sheet.export_svg());
        self.dxf_output = Some(sheet.export_dxf());
        self.generated_sheet = Some(sheet);

        Ok(())
    }

    fn undo(&mut self, _scene: &mut SceneGraph) -> Result<(), String> {
        self.generated_sheet = None;
        self.svg_output = None;
        self.dxf_output = None;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core_math::Transform;
    use geo_kernel::Primitives;
    use graph_engine::SceneNode;

    #[test]
    fn test_generate_drawing_sheet_command_execution() {
        let mut scene = SceneGraph::new();

        let cube = Primitives::create_cube(25.0);
        let n0 = scene.add_node(SceneNode::new_mesh(
            "BearingBlock",
            Transform::IDENTITY,
            cube,
        ));

        let mut cmd = GenerateDrawingSheetCommand::new(
            n0,
            SheetFormat::IsoA3,
            ProjectionStandard::ThirdAngle,
            1.0,
        );

        assert!(cmd.execute(&mut scene).is_ok());

        let sheet = cmd.generated_sheet.as_ref().unwrap();
        assert_eq!(sheet.views.len(), 4);
        assert!(cmd.svg_output.as_ref().unwrap().contains("BEARINGBLOCK"));
        assert!(cmd.dxf_output.as_ref().unwrap().contains("SECTION\n2\nHEADER"));

        assert!(cmd.undo(&mut scene).is_ok());
        assert!(cmd.generated_sheet.is_none());
    }
}

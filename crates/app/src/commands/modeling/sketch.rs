//! 2D Parametric CAD Constraint Sketcher & Live Dimension Commands.
//!
//! Provides undoable commands for sketch creation, entity placement,
//! constraint definition, dynamic dimension adjustments, and 3D BMesh extrusion.

use core_math::{Transform, Vec2};
use geo_kernel::{
    extrude_sketch, solve_sketch, ConstraintId, ConstraintType, Sketch, SketchConstraint,
    SketchEntityId, SolverConfig, Workplane,
};
use graph_engine::{Command, NodeData, NodeId, SceneGraph, SceneNode};

/// Command to create a new 2D parametric sketch hosted on a workplane.
#[derive(Debug)]
pub struct CreateSketchNodeCommand {
    pub name: String,
    pub workplane: Workplane,
    pub created_node_id: Option<NodeId>,
}

impl Command for CreateSketchNodeCommand {
    fn name(&self) -> &str {
        "Create CAD Sketch Node"
    }

    fn execute(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        let sketch = Sketch::new(&self.name, self.workplane);
        let node = SceneNode::new_sketch(&self.name, Transform::IDENTITY, sketch);
        let id = scene.add_node(node);
        self.created_node_id = Some(id);
        Ok(())
    }

    fn undo(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        if let Some(id) = self.created_node_id.take() {
            scene.remove_node(id);
            Ok(())
        } else {
            Err("No sketch node to remove".into())
        }
    }
}

/// Command to add an entity (line, arc, circle, rectangle) to an active sketch.
#[derive(Debug)]
pub struct AddSketchLineCommand {
    pub node_id: NodeId,
    pub p1: Vec2,
    pub p2: Vec2,
    pub added_line_id: Option<SketchEntityId>,
    old_sketch: Option<Sketch>,
}

impl AddSketchLineCommand {
    pub fn new(node_id: NodeId, p1: Vec2, p2: Vec2) -> Self {
        Self {
            node_id,
            p1,
            p2,
            added_line_id: None,
            old_sketch: None,
        }
    }
}

impl Command for AddSketchLineCommand {
    fn name(&self) -> &str {
        "Add Sketch Line"
    }

    fn execute(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        let node = scene.nodes.get_mut(self.node_id).ok_or("Node not found")?;
        if let NodeData::Sketch {
            sketch,
            cached_render_mesh,
        } = &mut node.data
        {
            self.old_sketch = Some(sketch.clone());
            let pt_a = sketch.add_point(self.p1);
            let pt_b = sketch.add_point(self.p2);
            let line_id = sketch.add_line(pt_a, pt_b);
            self.added_line_id = Some(line_id);
            *cached_render_mesh = None;
            scene.mark_dirty(self.node_id);
            Ok(())
        } else {
            Err("Node is not a CAD Sketch".into())
        }
    }

    fn undo(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        if let Some(old) = self.old_sketch.take() {
            let node = scene.nodes.get_mut(self.node_id).ok_or("Node not found")?;
            if let NodeData::Sketch {
                sketch,
                cached_render_mesh,
            } = &mut node.data
            {
                *sketch = old;
                *cached_render_mesh = None;
                scene.mark_dirty(self.node_id);
                Ok(())
            } else {
                Err("Node is not a CAD Sketch".into())
            }
        } else {
            Err("No previous sketch state to restore".into())
        }
    }
}

/// Command to add a geometric or dimensional constraint to an active sketch and re-solve.
#[derive(Debug)]
pub struct AddSketchConstraintCommand {
    pub node_id: NodeId,
    pub constraint_type: ConstraintType,
    pub added_id: Option<ConstraintId>,
    old_sketch: Option<Sketch>,
}

impl AddSketchConstraintCommand {
    pub fn new(node_id: NodeId, constraint_type: ConstraintType) -> Self {
        Self {
            node_id,
            constraint_type,
            added_id: None,
            old_sketch: None,
        }
    }
}

impl Command for AddSketchConstraintCommand {
    fn name(&self) -> &str {
        "Add Sketch Constraint"
    }

    fn execute(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        let node = scene.nodes.get_mut(self.node_id).ok_or("Node not found")?;
        if let NodeData::Sketch {
            sketch,
            cached_render_mesh,
        } = &mut node.data
        {
            self.old_sketch = Some(sketch.clone());
            let next_id = ConstraintId(sketch.constraints.len());
            sketch
                .constraints
                .push(SketchConstraint::new(next_id, self.constraint_type));
            self.added_id = Some(next_id);

            // Solve constraint system
            let _ = solve_sketch(sketch, SolverConfig::default());
            *cached_render_mesh = None;
            scene.mark_dirty(self.node_id);
            Ok(())
        } else {
            Err("Node is not a CAD Sketch".into())
        }
    }

    fn undo(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        if let Some(old) = self.old_sketch.take() {
            let node = scene.nodes.get_mut(self.node_id).ok_or("Node not found")?;
            if let NodeData::Sketch {
                sketch,
                cached_render_mesh,
            } = &mut node.data
            {
                *sketch = old;
                *cached_render_mesh = None;
                scene.mark_dirty(self.node_id);
                Ok(())
            } else {
                Err("Node is not a CAD Sketch".into())
            }
        } else {
            Err("No previous sketch state to restore".into())
        }
    }
}

/// Command to dynamically adjust the value of an existing dimensional constraint and re-solve.
#[derive(Debug)]
pub struct SetDimensionValueCommand {
    pub node_id: NodeId,
    pub constraint_id: ConstraintId,
    pub new_value: f32,
    old_value: Option<f32>,
    old_sketch: Option<Sketch>,
}

impl SetDimensionValueCommand {
    pub fn new(node_id: NodeId, constraint_id: ConstraintId, new_value: f32) -> Self {
        Self {
            node_id,
            constraint_id,
            new_value,
            old_value: None,
            old_sketch: None,
        }
    }
}

impl Command for SetDimensionValueCommand {
    fn name(&self) -> &str {
        "Edit Live Dimension Value"
    }

    fn execute(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        let node = scene.nodes.get_mut(self.node_id).ok_or("Node not found")?;
        if let NodeData::Sketch {
            sketch,
            cached_render_mesh,
        } = &mut node.data
        {
            self.old_sketch = Some(sketch.clone());
            let c = sketch
                .constraints
                .iter_mut()
                .find(|c| c.id == self.constraint_id)
                .ok_or("Constraint not found")?;

            match &mut c.kind {
                ConstraintType::Distance { distance, .. }
                | ConstraintType::HorizontalDistance { distance, .. }
                | ConstraintType::VerticalDistance { distance, .. } => {
                    self.old_value = Some(*distance);
                    *distance = self.new_value;
                }
                ConstraintType::Angle { angle_rad, .. } => {
                    self.old_value = Some(*angle_rad);
                    *angle_rad = self.new_value;
                }
                _ => return Err("Constraint is not a dimensional value".into()),
            }

            let _ = solve_sketch(sketch, SolverConfig::default());
            *cached_render_mesh = None;
            scene.mark_dirty(self.node_id);
            Ok(())
        } else {
            Err("Node is not a CAD Sketch".into())
        }
    }

    fn undo(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        if let Some(old) = self.old_sketch.take() {
            let node = scene.nodes.get_mut(self.node_id).ok_or("Node not found")?;
            if let NodeData::Sketch {
                sketch,
                cached_render_mesh,
            } = &mut node.data
            {
                *sketch = old;
                *cached_render_mesh = None;
                scene.mark_dirty(self.node_id);
                Ok(())
            } else {
                Err("Node is not a CAD Sketch".into())
            }
        } else {
            Err("No previous sketch state to restore".into())
        }
    }
}

/// Command to extrude a closed 2D CAD sketch profile into a solid 3D BMesh.
#[derive(Debug)]
pub struct ExtrudeSketchCommand {
    pub sketch_node_id: NodeId,
    pub depth: f32,
    pub created_mesh_node_id: Option<NodeId>,
}

impl ExtrudeSketchCommand {
    pub fn new(sketch_node_id: NodeId, depth: f32) -> Self {
        Self {
            sketch_node_id,
            depth,
            created_mesh_node_id: None,
        }
    }
}

impl Command for ExtrudeSketchCommand {
    fn name(&self) -> &str {
        "Extrude CAD Sketch to Solid"
    }

    fn execute(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        let (mesh, name) = {
            let node = scene
                .nodes
                .get(self.sketch_node_id)
                .ok_or("Node not found")?;
            if let NodeData::Sketch { sketch, .. } = &node.data {
                let mesh = extrude_sketch(sketch, self.depth)
                    .map_err(|e| format!("Extrusion failed: {}", e))?;
                (mesh, format!("{}_Solid", node.name))
            } else {
                return Err("Node is not a CAD Sketch".into());
            }
        };

        let mesh_node = SceneNode::new_mesh(name, Transform::IDENTITY, mesh);
        let id = scene.add_node(mesh_node);
        self.created_mesh_node_id = Some(id);
        Ok(())
    }

    fn undo(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        if let Some(id) = self.created_mesh_node_id.take() {
            scene.remove_node(id);
            Ok(())
        } else {
            Err("No extruded solid node to remove".into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core_math::vec2;
    use geo_kernel::SketchPointId;

    #[test]
    fn test_sketch_command_lifecycle() {
        let mut scene = SceneGraph::new();
        let mut create_cmd = CreateSketchNodeCommand {
            name: "Part1_Sketch".into(),
            workplane: Workplane::xy(),
            created_node_id: None,
        };

        assert!(create_cmd.execute(&mut scene).is_ok());
        let sketch_id = create_cmd.created_node_id.unwrap();

        // Add line
        let mut line_cmd = AddSketchLineCommand::new(sketch_id, vec2(0.0, 0.0), vec2(30.0, 10.0));
        assert!(line_cmd.execute(&mut scene).is_ok());

        // Add horizontal constraint
        let mut constr_cmd = AddSketchConstraintCommand::new(
            sketch_id,
            ConstraintType::Horizontal {
                pt_a: SketchPointId(0),
                pt_b: SketchPointId(1),
            },
        );
        assert!(constr_cmd.execute(&mut scene).is_ok());

        // Verify line is horizontal (points have identical Y coordinates)
        if let NodeData::Sketch { sketch, .. } = &scene.nodes[sketch_id].data {
            let p1 = sketch.get_point(SketchPointId(0)).unwrap().pos;
            let p2 = sketch.get_point(SketchPointId(1)).unwrap().pos;
            assert!(
                (p1.y - p2.y).abs() < 1e-3,
                "Points must have identical Y for horizontal line"
            );
        } else {
            panic!("Expected sketch node");
        }

        // Test undo constraint
        assert!(constr_cmd.undo(&mut scene).is_ok());
        // Test undo line
        assert!(line_cmd.undo(&mut scene).is_ok());
        // Test undo create sketch
        assert!(create_cmd.undo(&mut scene).is_ok());
        assert!(!scene.nodes.contains_key(sketch_id));
    }

    #[test]
    fn test_extrude_sketch_command_creates_solid_mesh() {
        let mut scene = SceneGraph::new();
        let mut create_cmd = CreateSketchNodeCommand {
            name: "Bracket_Profile".into(),
            workplane: Workplane::xy(),
            created_node_id: None,
        };
        assert!(create_cmd.execute(&mut scene).is_ok());
        let sketch_id = create_cmd.created_node_id.unwrap();

        // Add 4 lines forming closed rectangle profile
        if let NodeData::Sketch { sketch, .. } = &mut scene.nodes.get_mut(sketch_id).unwrap().data {
            sketch.add_rectangle(vec2(0.0, 0.0), vec2(40.0, 20.0));
        }

        // Extrude to solid 3D BMesh
        let mut extrude_cmd = ExtrudeSketchCommand::new(sketch_id, 30.0);
        assert!(extrude_cmd.execute(&mut scene).is_ok());

        let solid_id = extrude_cmd.created_mesh_node_id.unwrap();
        if let NodeData::Mesh { base_mesh, .. } = &scene.nodes[solid_id].data {
            assert_eq!(base_mesh.vertex_count(), 8);
            assert_eq!(base_mesh.face_count(), 6);
            assert_eq!(base_mesh.euler_characteristic(), 2);
        } else {
            panic!("Expected solid mesh node");
        }

        // Undo extrusion
        assert!(extrude_cmd.undo(&mut scene).is_ok());
        assert!(!scene.nodes.contains_key(solid_id));
    }

    #[test]
    fn test_set_dimension_value_command_and_solve() {
        let mut scene = SceneGraph::new();
        let mut create_cmd = CreateSketchNodeCommand {
            name: "Parametric_Line".into(),
            workplane: Workplane::xy(),
            created_node_id: None,
        };
        assert!(create_cmd.execute(&mut scene).is_ok());
        let sketch_id = create_cmd.created_node_id.unwrap();

        // Add line
        let mut line_cmd = AddSketchLineCommand::new(sketch_id, vec2(0.0, 0.0), vec2(10.0, 0.0));
        assert!(line_cmd.execute(&mut scene).is_ok());

        // Add Distance constraint = 10.0
        let mut constr_cmd = AddSketchConstraintCommand::new(
            sketch_id,
            ConstraintType::Distance {
                pt_a: SketchPointId(0),
                pt_b: SketchPointId(1),
                distance: 10.0,
            },
        );
        assert!(constr_cmd.execute(&mut scene).is_ok());
        let cid = constr_cmd.added_id.unwrap();

        // Adjust dimension to 50.0 mm
        let mut set_dim_cmd = SetDimensionValueCommand::new(sketch_id, cid, 50.0);
        assert!(set_dim_cmd.execute(&mut scene).is_ok());

        // Verify distance was updated and solved
        if let NodeData::Sketch { sketch, .. } = &scene.nodes[sketch_id].data {
            let p1 = sketch.get_point(SketchPointId(0)).unwrap().pos;
            let p2 = sketch.get_point(SketchPointId(1)).unwrap().pos;
            let dist = (p2 - p1).length();
            assert!((dist - 50.0).abs() < 1e-2, "Distance must solve to 50.0 mm");
        }

        // Undo dimension edit
        assert!(set_dim_cmd.undo(&mut scene).is_ok());
        if let NodeData::Sketch { sketch, .. } = &scene.nodes[sketch_id].data {
            let p1 = sketch.get_point(SketchPointId(0)).unwrap().pos;
            let p2 = sketch.get_point(SketchPointId(1)).unwrap().pos;
            let dist = (p2 - p1).length();
            assert!(
                (dist - 10.0).abs() < 1e-2,
                "Distance must restore to 10.0 mm"
            );
        }
    }
}

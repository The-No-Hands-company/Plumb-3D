//! Undoable Command for Scrubbing and Evaluating Exploded Views.

use core_math::Transform;
use geo_kernel::{ExplodedTrailLine, ExplodedView};
use graph_engine::{Command, NodeId, SceneGraph};
use std::collections::HashMap;

#[derive(Debug)]
pub struct ScrubExplodedViewCommand {
    exploded_view: ExplodedView,
    factor: f32,
    node_mapping: Vec<(usize, NodeId)>,
    before_transforms: Option<HashMap<NodeId, Transform>>,
    pub trail_lines: Vec<ExplodedTrailLine>,
}

impl ScrubExplodedViewCommand {
    pub fn new(
        mut exploded_view: ExplodedView,
        factor: f32,
        node_mapping: Vec<(usize, NodeId)>,
    ) -> Self {
        exploded_view.set_factor(factor);
        Self {
            exploded_view,
            factor,
            node_mapping,
            before_transforms: None,
            trail_lines: Vec::new(),
        }
    }

    pub fn factor(&self) -> f32 {
        self.factor
    }
}

impl Command for ScrubExplodedViewCommand {
    fn name(&self) -> &str {
        "Scrub Exploded View"
    }

    fn execute(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        let mut assembled = Vec::new();
        let mut before = HashMap::new();

        for &(body_id, node_id) in &self.node_mapping {
            let node = scene
                .nodes
                .get(node_id)
                .ok_or_else(|| format!("Node {:?} not found", node_id))?;
            assembled.push((body_id, node.local_transform));
            before.insert(node_id, node.local_transform);
        }

        if self.before_transforms.is_none() {
            self.before_transforms = Some(before);
        }

        let (exploded_map, trails) = self.exploded_view.evaluate(&assembled);
        self.trail_lines = trails;

        for &(body_id, node_id) in &self.node_mapping {
            if let Some(exp_tf) = exploded_map.get(&body_id) {
                if let Some(node) = scene.nodes.get_mut(node_id) {
                    node.local_transform = *exp_tf;
                    scene.mark_dirty(node_id);
                }
            }
        }

        Ok(())
    }

    fn undo(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        let before = self
            .before_transforms
            .as_ref()
            .ok_or("No previous transforms available for undo")?;

        for &(body_id, node_id) in &self.node_mapping {
            let _ = body_id;
            if let Some(orig_tf) = before.get(&node_id) {
                if let Some(node) = scene.nodes.get_mut(node_id) {
                    node.local_transform = *orig_tf;
                    scene.mark_dirty(node_id);
                }
            }
        }
        self.trail_lines.clear();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core_math::Vec3;
    use geo_kernel::{ExplodedStep, Primitives};
    use graph_engine::SceneNode;

    #[test]
    fn test_exploded_view_command_and_undo() {
        let mut scene = SceneGraph::new();

        let cube1 = Primitives::create_cube(2.0);
        let n1 = scene.add_node(SceneNode::new_mesh(
            "Base",
            Transform::IDENTITY,
            cube1,
        ));

        let cube2 = Primitives::create_cube(2.0);
        let n2 = scene.add_node(SceneNode::new_mesh(
            "Part",
            Transform::from_translation(Vec3::new(0.0, 5.0, 0.0)),
            cube2,
        ));

        let mut exp = ExplodedView::new("TestExplosion");
        exp.add_step(ExplodedStep::new(0, "MovePart", vec![1], Vec3::Y, 20.0));

        let mapping = vec![(0, n1), (1, n2)];
        let mut cmd = ScrubExplodedViewCommand::new(exp, 1.0, mapping);

        assert!(cmd.execute(&mut scene).is_ok());

        let part = scene.nodes.get(n2).unwrap();
        assert!((part.local_transform.translation - Vec3::new(0.0, 25.0, 0.0)).length() < 1e-4);
        assert_eq!(cmd.trail_lines.len(), 1);

        assert!(cmd.undo(&mut scene).is_ok());

        let part_restored = scene.nodes.get(n2).unwrap();
        assert!((part_restored.local_transform.translation - Vec3::new(0.0, 5.0, 0.0)).length() < 1e-4);
    }
}

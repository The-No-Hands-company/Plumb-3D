//! Undoable Commands for Spawning Parametric Mechanical Parts
//!
//! Generates and places ISO/ANSI Fasteners (bolts, screws, nuts, washers),
//! Involute Gears (Spur, Helical, Bevel), and Roller Chain Sprockets.

use core_math::Transform;
use geo_kernel::{
    FastenerBuilder, FastenerParams, GearParams, InvoluteGearBuilder, SprocketBuilder,
    SprocketParams,
};
use graph_engine::{Command, NodeId, SceneGraph, SceneNode};

/// Undoable command to generate and spawn an ISO/ANSI mechanical fastener.
#[derive(Debug)]
pub struct SpawnFastenerCommand {
    params: FastenerParams,
    pub created_node: Option<NodeId>,
}

impl SpawnFastenerCommand {
    pub fn new(params: FastenerParams) -> Self {
        Self {
            params,
            created_node: None,
        }
    }
}

impl Command for SpawnFastenerCommand {
    fn name(&self) -> &str {
        "Spawn Mechanical Fastener"
    }

    fn execute(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        let mesh = FastenerBuilder::generate_mesh(&self.params);
        let name = format!("{:?}_{:?}", self.params.fastener_type, self.params.size);
        let node_id = scene.add_node(SceneNode::new_mesh(&name, Transform::IDENTITY, mesh));
        self.created_node = Some(node_id);
        Ok(())
    }

    fn undo(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        if let Some(id) = self.created_node.take() {
            scene.remove_node(id);
        }
        Ok(())
    }
}

/// Undoable command to generate and spawn an analytical involute gear.
#[derive(Debug)]
pub struct SpawnGearCommand {
    params: GearParams,
    pub created_node: Option<NodeId>,
}

impl SpawnGearCommand {
    pub fn new(params: GearParams) -> Self {
        Self {
            params,
            created_node: None,
        }
    }
}

impl Command for SpawnGearCommand {
    fn name(&self) -> &str {
        "Spawn Involute Gear"
    }

    fn execute(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        let mesh = InvoluteGearBuilder::generate_mesh(&self.params);
        let name = format!("{:?}_M{:.1}_Z{}", self.params.gear_type, self.params.module, self.params.teeth);
        let node_id = scene.add_node(SceneNode::new_mesh(&name, Transform::IDENTITY, mesh));
        self.created_node = Some(node_id);
        Ok(())
    }

    fn undo(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        if let Some(id) = self.created_node.take() {
            scene.remove_node(id);
        }
        Ok(())
    }
}

/// Undoable command to generate and spawn a roller chain sprocket.
#[derive(Debug)]
pub struct SpawnSprocketCommand {
    params: SprocketParams,
    pub created_node: Option<NodeId>,
}

impl SpawnSprocketCommand {
    pub fn new(params: SprocketParams) -> Self {
        Self {
            params,
            created_node: None,
        }
    }
}

impl Command for SpawnSprocketCommand {
    fn name(&self) -> &str {
        "Spawn Roller Chain Sprocket"
    }

    fn execute(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        let mesh = SprocketBuilder::generate_mesh(&self.params);
        let name = format!("{:?}_Z{}", self.params.chain_size, self.params.teeth);
        let node_id = scene.add_node(SceneNode::new_mesh(&name, Transform::IDENTITY, mesh));
        self.created_node = Some(node_id);
        Ok(())
    }

    fn undo(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        if let Some(id) = self.created_node.take() {
            scene.remove_node(id);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use geo_kernel::{FastenerType, GearType, MetricThreadSize};

    #[test]
    fn test_spawn_fastener_command_lifecycle() {
        let mut scene = SceneGraph::new();
        let params = FastenerParams {
            size: MetricThreadSize::M10,
            fastener_type: FastenerType::HexBolt,
            length: 40.0,
            thread_mode: geo_kernel::ThreadMode::Cosmetic,
        };

        let mut cmd = SpawnFastenerCommand::new(params);
        assert!(cmd.execute(&mut scene).is_ok());

        let node_id = cmd.created_node.expect("Node ID should exist");
        assert!(scene.nodes.contains_key(node_id));

        assert!(cmd.undo(&mut scene).is_ok());
        assert!(!scene.nodes.contains_key(node_id));
    }

    #[test]
    fn test_spawn_gear_and_sprocket_commands() {
        let mut scene = SceneGraph::new();
        let gear_params = GearParams {
            module: 2.5,
            teeth: 24,
            pressure_angle_deg: 20.0,
            face_width: 20.0,
            helix_angle_deg: 0.0,
            bore_diameter: 14.0,
            gear_type: GearType::Spur,
        };

        let mut gear_cmd = SpawnGearCommand::new(gear_params);
        assert!(gear_cmd.execute(&mut scene).is_ok());
        let gear_node = gear_cmd.created_node.unwrap();
        assert!(scene.nodes.contains_key(gear_node));

        let sprocket_params = SprocketParams::default();
        let mut sp_cmd = SpawnSprocketCommand::new(sprocket_params);
        assert!(sp_cmd.execute(&mut scene).is_ok());
        let sp_node = sp_cmd.created_node.unwrap();
        assert!(scene.nodes.contains_key(sp_node));

        assert!(sp_cmd.undo(&mut scene).is_ok());
        assert!(!scene.nodes.contains_key(sp_node));
    }
}

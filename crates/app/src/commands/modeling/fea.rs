//! Undoable Command for Executing FEA Structural Simulation
//!
//! Solves linear elasticity on a 3D solid mesh and records Von Mises stress,
//! displacement fields, and Factor of Safety (FoS) analysis.

use geo_kernel::{FeaConfig, FeaReport, FeaSolver, Mesh, TetMesh};
use graph_engine::{Command, NodeData, NodeId, SceneGraph};

/// Undoable command to execute FEA structural simulation on an object
#[derive(Debug)]
pub struct RunFeaSimulationCommand {
    node_id: NodeId,
    config: FeaConfig,
    pub report: Option<FeaReport>,
    old_mesh: Option<Mesh>,
}

impl RunFeaSimulationCommand {
    pub fn new(node_id: NodeId, config: FeaConfig) -> Self {
        Self {
            node_id,
            config,
            report: None,
            old_mesh: None,
        }
    }
}

impl Command for RunFeaSimulationCommand {
    fn name(&self) -> &str {
        "Run FEA Structural Simulation"
    }

    fn execute(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        let node = scene
            .nodes
            .get_mut(self.node_id)
            .ok_or_else(|| format!("Node {:?} not found", self.node_id))?;

        let base_mesh = match &mut node.data {
            NodeData::Mesh { base_mesh, .. } => base_mesh,
            _ => return Err(format!("Node {:?} is not a polygon mesh", self.node_id)),
        };

        self.old_mesh = Some(base_mesh.clone());

        // 1. Build volumetric tetrahedral mesh from surface geometry
        let tet_mesh = TetMesh::from_surface_mesh(base_mesh);
        if tet_mesh.elements.is_empty() {
            return Err("Mesh cannot be decomposed into volumetric tetrahedra".to_string());
        }

        // 2. Solve linear elasticity system
        let report = FeaSolver::solve(&tet_mesh, &self.config)?;

        // 3. Update deformed surface mesh vertices if deformation_scale > 0
        if self.config.deformation_scale > 0.0 {
            let mut deformed = base_mesh.clone();
            let mut v_idx = 0;
            for (_, vert) in &mut deformed.vertices {
                if v_idx < report.deformed_nodes.len() {
                    vert.position = report.deformed_nodes[v_idx];
                }
                v_idx += 1;
            }
            *base_mesh = deformed;
        }

        self.report = Some(report);
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
        self.report = None;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core_math::{Transform, Vec3};
    use geo_kernel::{AppliedLoad, FeaMaterial, Primitives, SupportConstraint};
    use graph_engine::SceneNode;

    #[test]
    fn test_fea_command_execution_and_undo() {
        let mut scene = SceneGraph::new();
        let cube = Primitives::create_cube(10.0);
        let node_id = scene.add_node(SceneNode::new_mesh("TestBracket", Transform::IDENTITY, cube));

        let config = FeaConfig {
            material: FeaMaterial::structural_steel(),
            supports: vec![SupportConstraint::new_fixed(vec![0, 1, 2, 3])],
            loads: vec![AppliedLoad::new(Vec3::new(0.0, 0.0, -100.0), vec![4, 5, 6, 7])],
            include_gravity: false,
            deformation_scale: 50.0,
        };

        let mut cmd = RunFeaSimulationCommand::new(node_id, config);
        assert!(cmd.execute(&mut scene).is_ok());

        let report = cmd.report.as_ref().expect("Report should be generated");
        assert!(report.max_displacement_mm > 0.0);
        assert!(report.max_von_mises_mpa > 0.0);

        assert!(cmd.undo(&mut scene).is_ok());
        assert!(cmd.report.is_none());
    }
}

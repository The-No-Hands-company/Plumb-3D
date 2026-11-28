//! Finite Element Analysis (FEA) Structural Simulation & Stress Tensor Heatmaps
//!
//! Provides linear elasticity solvers, 4-node tetrahedral meshing, material models,
//! and Von Mises stress tensor recovery with Factor of Safety (FoS) analysis.

pub mod boundary;
pub mod material;
pub mod mesh_tet;
pub mod solver;

pub use boundary::{AppliedLoad, FeaConfig, SupportConstraint, SupportType};
pub use material::{FeaMaterial, MaterialPreset};
pub use mesh_tet::{Tet4Element, TetMesh};
pub use solver::{FeaReport, FeaSolver};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::Primitives;
    use core_math::Vec3;

    #[test]
    fn test_fea_material_lame_parameters_and_constitutive_matrix() {
        let steel = FeaMaterial::structural_steel();
        assert_eq!(steel.youngs_modulus_gpa, 210.0);
        assert_eq!(steel.poissons_ratio, 0.30);

        let d = steel.constitutive_matrix_6x6();
        // Check symmetry of constitutive tensor
        assert_eq!(d[0][1], d[1][0]);
        assert_eq!(d[0][2], d[2][0]);
        assert!(d[0][0] > 0.0);
        assert!(d[3][3] > 0.0); // Shear modulus G > 0
    }

    #[test]
    fn test_tetrahedral_volume_and_stiffness_matrix() {
        let nodes = vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(10.0, 0.0, 0.0),
            Vec3::new(0.0, 10.0, 0.0),
            Vec3::new(0.0, 0.0, 10.0),
        ];

        let tet = Tet4Element::new(0, 1, 2, 3);
        let vol = tet.volume(&nodes);
        // Canonical tetrahedron volume = (1/6) * 10 * 10 * 10 = 1000 / 6 = 166.667 mm^3
        assert!((vol - 166.6667).abs() < 1e-2);

        let steel = FeaMaterial::structural_steel();
        let ke = tet.element_stiffness_12x12(&nodes, &steel);

        // Check stiffness symmetry k_ij = k_ji
        for i in 0..12 {
            for j in 0..12 {
                assert!((ke[i][j] - ke[j][i]).abs() < 1e-1);
            }
        }
    }

    #[test]
    fn test_fea_cantilever_beam_bending_simulation() {
        // Create a 3D rectangular solid beam
        let beam_mesh = Primitives::create_cube(20.0);
        let tet_mesh = TetMesh::from_surface_mesh(&beam_mesh);

        assert!(!tet_mesh.elements.is_empty());
        assert!(!tet_mesh.nodes.is_empty());

        // Find bottom nodes (Z <= -9.0) to fix as support
        let mut fixed_nodes = Vec::new();
        let mut top_nodes = Vec::new();

        for (i, p) in tet_mesh.nodes.iter().enumerate() {
            if p.z < -9.0 {
                fixed_nodes.push(i);
            } else if p.z > 9.0 {
                top_nodes.push(i);
            }
        }

        let config = FeaConfig {
            material: FeaMaterial::structural_steel(),
            supports: vec![SupportConstraint::new_fixed(fixed_nodes)],
            loads: vec![AppliedLoad::new(Vec3::new(0.0, 0.0, -500.0), top_nodes)], // 500 N downward
            include_gravity: true,
            deformation_scale: 100.0,
        };

        let report = FeaSolver::solve(&tet_mesh, &config).expect("FEA solve should succeed");

        assert!(report.max_displacement_mm > 0.0);
        assert!(report.max_von_mises_mpa > 0.0);
        assert!(report.min_factor_of_safety > 0.0);
        assert_eq!(report.node_displacements.len(), tet_mesh.nodes.len());
        assert_eq!(report.deformed_nodes.len(), tet_mesh.nodes.len());
    }
}

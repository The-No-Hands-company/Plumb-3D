//! Finite Element Linear Elasticity Solver & Von Mises Stress Recovery
//!
//! Assembles global stiffness matrix [K], solves [K]{U} = {F} via Conjugate Gradient,
//! and recovers Cauchy stress tensors, Von Mises equivalent stress, and Factor of Safety (FoS).

use core_math::Vec3;
use serde::{Deserialize, Serialize};

use super::boundary::FeaConfig;
use super::mesh_tet::TetMesh;

/// Results and Mechanical Integrity Report from FEA Simulation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FeaReport {
    /// Displacement vector in millimeters at each node
    pub node_displacements: Vec<Vec3>,
    /// Von Mises equivalent stress in Megapascals (MPa) at each node
    pub node_von_mises_mpa: Vec<f32>,
    /// Peak equivalent stress in structure in MPa
    pub max_von_mises_mpa: f32,
    /// Minimum Factor of Safety (FoS = Yield Strength / Max Von Mises)
    pub min_factor_of_safety: f32,
    /// Maximum absolute displacement magnitude in millimeters
    pub max_displacement_mm: f32,
    /// Deformed 3D node positions magnified by deformation_scale
    pub deformed_nodes: Vec<Vec3>,
    /// Whether maximum stress exceeds material yield strength (plastic deformation)
    pub is_yield_exceeded: bool,
}

/// Linear Elastic Finite Element Solver
pub struct FeaSolver;

impl FeaSolver {
    /// Solve the linear structural system [K]{U} = {F} for the given tetrahedral mesh
    pub fn solve(tet_mesh: &TetMesh, config: &FeaConfig) -> Result<FeaReport, String> {
        let num_nodes = tet_mesh.nodes.len();
        if num_nodes == 0 || tet_mesh.elements.is_empty() {
            return Err("Cannot solve FEA on empty tetrahedral mesh".to_string());
        }

        let num_dofs = num_nodes * 3;

        // 1. Build Global Force Vector {F} in Newtons
        let mut f_global = vec![0.0f32; num_dofs];

        // Applied external mechanical forces
        for load in &config.loads {
            if load.node_indices.is_empty() {
                continue;
            }
            let per_node_force = load.force_vector_n / load.node_indices.len() as f32;
            for &node_idx in &load.node_indices {
                if node_idx < num_nodes {
                    f_global[node_idx * 3] += per_node_force.x;
                    f_global[node_idx * 3 + 1] += per_node_force.y;
                    f_global[node_idx * 3 + 2] += per_node_force.z;
                }
            }
        }

        // Gravitational body forces if enabled (-Z direction)
        if config.include_gravity {
            let rho = config.material.density_kg_m3;
            let g = -9.81f32;
            for elem in &tet_mesh.elements {
                let vol_m3 = elem.volume(&tet_mesh.nodes) * 1e-9; // mm^3 to m^3
                let elem_mass = rho * vol_m3;
                let node_grav_force = (elem_mass * g) / 4.0;
                for &n in &elem.nodes {
                    f_global[n * 3 + 2] += node_grav_force;
                }
            }
        }

        // 2. Identify Constrained Degrees of Freedom
        let mut fixed_dofs = vec![false; num_dofs];
        for sup in &config.supports {
            for &n in &sup.node_indices {
                if n < num_nodes {
                    fixed_dofs[n * 3] = true;
                    fixed_dofs[n * 3 + 1] = true;
                    fixed_dofs[n * 3 + 2] = true;
                }
            }
        }

        // If no supports specified, anchor the lowest node(s) along Z to prevent rigid body motion
        let has_support = fixed_dofs.iter().any(|&f| f);
        if !has_support {
            let mut min_z = f32::MAX;
            let mut lowest_idx = 0;
            for (i, p) in tet_mesh.nodes.iter().enumerate() {
                if p.z < min_z {
                    min_z = p.z;
                    lowest_idx = i;
                }
            }
            fixed_dofs[lowest_idx * 3] = true;
            fixed_dofs[lowest_idx * 3 + 1] = true;
            fixed_dofs[lowest_idx * 3 + 2] = true;
        }

        // 3. Assemble Global Stiffness Matrix in Sparse Coordinate Format
        // For efficiency on interactive CAD meshes, assemble into diagonal preconditioner and matrix-vector operator
        let mut diag = vec![1e-6f32; num_dofs];
        let mut elem_ke = Vec::with_capacity(tet_mesh.elements.len());

        for elem in &tet_mesh.elements {
            let ke = elem.element_stiffness_12x12(&tet_mesh.nodes, &config.material);
            for i in 0..4 {
                let ni = elem.nodes[i];
                for dof_i in 0..3 {
                    let global_dof = ni * 3 + dof_i;
                    diag[global_dof] += ke[i * 3 + dof_i][i * 3 + dof_i];
                }
            }
            elem_ke.push(ke);
        }

        // Apply boundary constraints to diagonal
        for d in 0..num_dofs {
            if fixed_dofs[d] {
                diag[d] = 1.0e15; // Penalty constraint
                f_global[d] = 0.0;
            }
        }

        // 4. Solve [K]{U} = {F} using Preconditioned Conjugate Gradient (PCG)
        let mut u = vec![0.0f32; num_dofs];
        let mut r = f_global.clone();
        let mut p = vec![0.0f32; num_dofs];

        // p = M^-1 * r
        for i in 0..num_dofs {
            p[i] = r[i] / diag[i];
        }

        let mut rs_old: f32 = r.iter().zip(&p).map(|(a, b)| a * b).sum();
        let max_iters = (num_dofs).min(200);

        for _ in 0..max_iters {
            if rs_old.abs() < 1e-12 {
                break;
            }

            // Compute Ap = [K] * p by accumulating element-level products
            let mut ap = vec![0.0f32; num_dofs];
            for (elem_idx, elem) in tet_mesh.elements.iter().enumerate() {
                let ke = &elem_ke[elem_idx];
                let mut pe = [0.0f32; 12];
                for i in 0..4 {
                    let n = elem.nodes[i];
                    pe[i * 3] = p[n * 3];
                    pe[i * 3 + 1] = p[n * 3 + 1];
                    pe[i * 3 + 2] = p[n * 3 + 2];
                }

                for r_idx in 0..12 {
                    let mut sum = 0.0;
                    for c_idx in 0..12 {
                        sum += ke[r_idx][c_idx] * pe[c_idx];
                    }
                    let n_idx = elem.nodes[r_idx / 3];
                    let dof_local = r_idx % 3;
                    ap[n_idx * 3 + dof_local] += sum;
                }
            }

            // Enforce boundary penalty on Ap
            for d in 0..num_dofs {
                if fixed_dofs[d] {
                    ap[d] = p[d] * 1.0e15;
                }
            }

            let p_ap: f32 = p.iter().zip(&ap).map(|(a, b)| a * b).sum();
            if p_ap.abs() < 1e-18 {
                break;
            }

            let alpha = rs_old / p_ap;
            for i in 0..num_dofs {
                u[i] += alpha * p[i];
                r[i] -= alpha * ap[i];
            }

            // Precondition z = M^-1 * r
            let mut z = vec![0.0f32; num_dofs];
            for i in 0..num_dofs {
                z[i] = r[i] / diag[i];
            }

            let rs_new: f32 = r.iter().zip(&z).map(|(a, b)| a * b).sum();
            let beta = rs_new / rs_old;
            for i in 0..num_dofs {
                p[i] = z[i] + beta * p[i];
            }
            rs_old = rs_new;
        }

        // 5. Stress Recovery & Von Mises Equivalent Stress
        let mut node_von_mises = vec![0.0f32; num_nodes];
        let mut node_contribs = vec![0usize; num_nodes];
        let d_matrix = config.material.constitutive_matrix_6x6();

        for elem in &tet_mesh.elements {
            let (b_matrix, _) = elem.strain_displacement_matrix(&tet_mesh.nodes);
            let mut u_elem = [0.0f32; 12];
            for i in 0..4 {
                let n = elem.nodes[i];
                u_elem[i * 3] = u[n * 3];
                u_elem[i * 3 + 1] = u[n * 3 + 1];
                u_elem[i * 3 + 2] = u[n * 3 + 2];
            }

            // Strain eps = [B] * {u_e}
            let mut eps = [0.0f32; 6];
            for r in 0..6 {
                let mut sum = 0.0;
                for c in 0..12 {
                    sum += b_matrix[r][c] * u_elem[c];
                }
                eps[r] = sum;
            }

            // Stress sigma = [D] * eps (in Pa -> convert to MPa: / 1e6)
            let mut sigma = [0.0f32; 6];
            for r in 0..6 {
                let mut sum = 0.0;
                for c in 0..6 {
                    sum += d_matrix[r][c] * eps[c];
                }
                sigma[r] = sum * 1e-6; // MPa
            }

            // Von Mises stress formula:
            // sigma_v = sqrt( 0.5 * ((s_xx - s_yy)^2 + (s_yy - s_zz)^2 + (s_zz - s_xx)^2 + 6*(t_xy^2 + t_yz^2 + t_zx^2)) )
            let (sxx, syy, szz) = (sigma[0], sigma[1], sigma[2]);
            let (txy, tyz, tzx) = (sigma[3], sigma[4], sigma[5]);

            let vm = (0.5 * ((sxx - syy).powi(2) + (syy - szz).powi(2) + (szz - sxx).powi(2) + 6.0 * (txy.powi(2) + tyz.powi(2) + tzx.powi(2)))).sqrt();

            for &n in &elem.nodes {
                node_von_mises[n] += vm;
                node_contribs[n] += 1;
            }
        }

        let mut max_vm = 0.0f32;
        for i in 0..num_nodes {
            if node_contribs[i] > 0 {
                node_von_mises[i] /= node_contribs[i] as f32;
            }
            max_vm = max_vm.max(node_von_mises[i]);
        }

        // 6. Format Node Displacements and Deformed Positions
        let mut node_displacements = Vec::with_capacity(num_nodes);
        let mut deformed_nodes = Vec::with_capacity(num_nodes);
        let mut max_disp = 0.0f32;

        let scale = config.deformation_scale;

        for i in 0..num_nodes {
            let disp = Vec3::new(u[i * 3], u[i * 3 + 1], u[i * 3 + 2]);
            max_disp = max_disp.max(disp.length());
            node_displacements.push(disp);
            deformed_nodes.push(tet_mesh.nodes[i] + disp * scale);
        }

        let fos = config.material.yield_strength_mpa / max_vm.max(1e-4);
        let is_yield_exceeded = max_vm >= config.material.yield_strength_mpa;

        Ok(FeaReport {
            node_displacements,
            node_von_mises_mpa: node_von_mises,
            max_von_mises_mpa: max_vm,
            min_factor_of_safety: fos,
            max_displacement_mm: max_disp,
            deformed_nodes,
            is_yield_exceeded,
        })
    }
}

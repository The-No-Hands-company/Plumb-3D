//! Linear 4-Node Tetrahedral Element (Tet4) Formulation & Meshing
//!
//! Evaluates element volume, strain-displacement matrix [B], and local 12x12
//! stiffness matrix [k_e] = [B]^T [D] [B] * V_e.

use core_math::Vec3;
use serde::{Deserialize, Serialize};

use super::material::FeaMaterial;
use crate::mesh::Mesh;

/// A linear 4-node tetrahedral element with node indices into node array
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tet4Element {
    pub nodes: [usize; 4],
}

impl Tet4Element {
    pub fn new(n0: usize, n1: usize, n2: usize, n3: usize) -> Self {
        Self {
            nodes: [n0, n1, n2, n3],
        }
    }

    /// Compute volume of tetrahedron using 3D determinant
    pub fn volume(&self, node_coords: &[Vec3]) -> f32 {
        let p0 = node_coords[self.nodes[0]];
        let p1 = node_coords[self.nodes[1]];
        let p2 = node_coords[self.nodes[2]];
        let p3 = node_coords[self.nodes[3]];

        let v1 = p1 - p0;
        let v2 = p2 - p0;
        let v3 = p3 - p0;

        (v1.cross(v2).dot(v3)).abs() / 6.0
    }

    /// Compute centroid of tetrahedron
    pub fn centroid(&self, node_coords: &[Vec3]) -> Vec3 {
        let p0 = node_coords[self.nodes[0]];
        let p1 = node_coords[self.nodes[1]];
        let p2 = node_coords[self.nodes[2]];
        let p3 = node_coords[self.nodes[3]];
        (p0 + p1 + p2 + p3) * 0.25
    }

    /// Compute shape function spatial gradients [b_alpha, c_alpha, d_alpha] / (6 * V)
    /// for alpha in 0..4
    pub fn shape_gradients(&self, node_coords: &[Vec3]) -> ([[f32; 3]; 4], f32) {
        let p = [
            node_coords[self.nodes[0]],
            node_coords[self.nodes[1]],
            node_coords[self.nodes[2]],
            node_coords[self.nodes[3]],
        ];

        let vol = self.volume(node_coords).max(1e-12);
        let det6 = vol * 6.0;

        // Shape function coefficients from standard cofactor expansions
        let mut grads = [[0.0f32; 3]; 4];

        for i in 0..4 {
            let (j, m, n) = match i {
                0 => (1, 2, 3),
                1 => (0, 3, 2),
                2 => (0, 1, 3),
                3 => (0, 2, 1),
                _ => unreachable!(),
            };

            let v1 = p[m] - p[j];
            let v2 = p[n] - p[j];
            let normal = v1.cross(v2);

            grads[i][0] = normal.x / det6;
            grads[i][1] = normal.y / det6;
            grads[i][2] = normal.z / det6;
        }

        (grads, vol)
    }

    /// Compute 6x12 strain-displacement matrix [B]
    /// relating node displacements [u0,v0,w0, u1,v1,w1, u2,v2,w2, u3,v3,w3]^T to strain
    pub fn strain_displacement_matrix(&self, node_coords: &[Vec3]) -> ([[f32; 12]; 6], f32) {
        let (grads, vol) = self.shape_gradients(node_coords);
        let mut b = [[0.0f32; 12]; 6];

        for i in 0..4 {
            let col = i * 3;
            let [bx, by, bz] = grads[i];

            // eps_xx
            b[0][col] = bx;
            // eps_yy
            b[1][col + 1] = by;
            // eps_zz
            b[2][col + 2] = bz;
            // gamma_xy
            b[3][col] = by;
            b[3][col + 1] = bx;
            // gamma_yz
            b[4][col + 1] = bz;
            b[4][col + 2] = by;
            // gamma_zx
            b[5][col] = bz;
            b[5][col + 2] = bx;
        }

        (b, vol)
    }

    /// Compute local 12x12 element stiffness matrix [k_e] = [B]^T [D] [B] * V_e
    pub fn element_stiffness_12x12(
        &self,
        node_coords: &[Vec3],
        material: &FeaMaterial,
    ) -> [[f32; 12]; 12] {
        let (b, vol) = self.strain_displacement_matrix(node_coords);
        let d = material.constitutive_matrix_6x6();

        // 1. Compute [DB] = [D] * [B] (6x12)
        let mut db = [[0.0f32; 12]; 6];
        for r in 0..6 {
            for c in 0..12 {
                let mut sum = 0.0;
                for k in 0..6 {
                    sum += d[r][k] * b[k][c];
                }
                db[r][c] = sum;
            }
        }

        // 2. Compute [k_e] = [B]^T * [DB] * vol (12x12)
        let mut ke = [[0.0f32; 12]; 12];
        for r in 0..12 {
            for c in 0..12 {
                let mut sum = 0.0;
                for k in 0..6 {
                    sum += b[k][r] * db[k][c];
                }
                ke[r][c] = sum * vol;
            }
        }

        ke
    }
}

/// Tetrahedral FEA Mesh Container
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TetMesh {
    pub nodes: Vec<Vec3>,
    pub elements: Vec<Tet4Element>,
}

impl TetMesh {
    /// Generate a volumetric tetrahedral mesh from a solid surface BMesh
    pub fn from_surface_mesh(mesh: &Mesh) -> Self {
        let mut nodes = Vec::new();
        let mut node_map = std::collections::HashMap::new();

        for (vk, v) in &mesh.vertices {
            let idx = nodes.len();
            nodes.push(v.position);
            node_map.insert(vk, idx);
        }

        // Compute mesh centroid to form interior anchor node
        let centroid = if !nodes.is_empty() {
            let sum: Vec3 = nodes.iter().copied().sum();
            sum / nodes.len() as f32
        } else {
            Vec3::ZERO
        };

        let centroid_idx = nodes.len();
        nodes.push(centroid);

        let mut elements = Vec::new();

        // Tessellate each polygonal face into triangles with centroid to form Tet4
        for (fk, _) in &mesh.faces {
            let face_verts = mesh.face_vertices(fk);
            if face_verts.len() < 3 {
                continue;
            }

            let v0 = node_map[&face_verts[0]];
            for i in 1..(face_verts.len() - 1) {
                let v1 = node_map[&face_verts[i]];
                let v2 = node_map[&face_verts[i + 1]];

                let tet = Tet4Element::new(v0, v1, v2, centroid_idx);
                if tet.volume(&nodes) > 1e-9 {
                    elements.push(tet);
                } else {
                    // Try reverse orientation if negative volume
                    let tet_rev = Tet4Element::new(v0, v2, v1, centroid_idx);
                    if tet_rev.volume(&nodes) > 1e-9 {
                        elements.push(tet_rev);
                    }
                }
            }
        }

        Self { nodes, elements }
    }

    /// Compute total volume of all tetrahedral elements in cubic millimeters
    pub fn total_volume(&self) -> f32 {
        self.elements.iter().map(|e| e.volume(&self.nodes)).sum()
    }
}

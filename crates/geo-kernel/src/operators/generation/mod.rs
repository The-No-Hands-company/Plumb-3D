pub mod bridge;
pub mod fill_triangulate;
pub mod normals;
pub mod spin_symmetrize;

#[cfg(test)]
mod tests {
    use crate::mesh::Mesh;
    use core_math::Vec3;

    #[test]
    fn test_bridge_edge_loops_advanced_and_cuts() {
        let mut mesh = Mesh::new();

        // Loop A at Y = 0
        let a0 = mesh.add_vertex(Vec3::new(-1.0, 0.0, -1.0));
        let a1 = mesh.add_vertex(Vec3::new(1.0, 0.0, -1.0));
        let a2 = mesh.add_vertex(Vec3::new(1.0, 0.0, 1.0));
        let a3 = mesh.add_vertex(Vec3::new(-1.0, 0.0, 1.0));

        // Loop B at Y = 2
        let b0 = mesh.add_vertex(Vec3::new(-1.0, 2.0, -1.0));
        let b1 = mesh.add_vertex(Vec3::new(1.0, 2.0, -1.0));
        let b2 = mesh.add_vertex(Vec3::new(1.0, 2.0, 1.0));
        let b3 = mesh.add_vertex(Vec3::new(-1.0, 2.0, 1.0));

        let loop_a = vec![a0, a1, a2, a3];
        let loop_b = vec![b0, b1, b2, b3];

        // Bridge with 2 intermediate cuts
        let faces = mesh
            .bridge_edge_loops_advanced(&loop_a, &loop_b, 0, 2)
            .expect("Bridge should succeed");

        // 4 quads per segment ring * 3 rings = 12 quads
        assert_eq!(faces.len(), 12);
        assert_eq!(mesh.faces.len(), 12);
    }
}

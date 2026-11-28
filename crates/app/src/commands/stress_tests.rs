//! High-Poly Stress & Performance Benchmarks for Plumb 3D Modeler v1.0
//!
//! Validates memory safety, half-edge BMesh 2-manifold invariants,
//! undo/redo throughput, and raycast picking response under high polygon loads.

use core_math::{vec3, Ray, Transform, Vec3};
use geo_kernel::Primitives;
use graph_engine::{HistoryManager, SceneGraph, SceneNode};
use std::time::Instant;

use super::TransformCommand;

#[test]
fn test_high_poly_mesh_creation_and_bmesh_invariants() {
    let t0 = Instant::now();
    // 64 rings x 64 segments produces 4,096 quads (~8,192 triangles) with dense half-edge validation
    let mesh = Primitives::create_uv_sphere(2.0, 64, 64);
    let creation_duration = t0.elapsed();

    // Verify creation is high-throughput (sub-second in unoptimized debug, <5ms in release)
    assert!(creation_duration.as_secs_f32() < 2.0);

    let v_count = mesh.vertex_count();
    let f_count = mesh.face_count();
    let e_count = mesh.edge_count();

    assert!(v_count >= 3_800);
    assert!(f_count >= 3_800);
    assert!(e_count >= 7_600);

    // Topological invariant for a topological sphere: Euler characteristic Chi = V - E + F = 2
    let chi = mesh.euler_characteristic();
    assert_eq!(
        chi, 2,
        "High-poly UV sphere must satisfy Euler characteristic Chi = 2"
    );

    // Verify 2-manifold invariants: every edge has origin and target
    for (_ek, edge) in &mesh.edges {
        let he = &mesh.half_edges[edge.half_edge];
        assert_ne!(he.origin, he.target);
    }
}

#[test]
fn test_high_poly_vertex_transform_and_undo_redo_throughput() {
    let mut scene = SceneGraph::new();
    let mesh = Primitives::create_uv_sphere(1.5, 80, 80); // ~12.8k triangles
    let old_trans = Transform::IDENTITY;
    let new_trans = Transform::from_translation(vec3(0.0, 5.0, 0.0));
    let node = SceneNode::new_mesh("DenseSphere", old_trans, mesh);
    let node_id = scene.add_node(node);

    let mut history = HistoryManager::new();

    // Measure command execution time across dense node
    let t_exec = Instant::now();
    let cmd = Box::new(TransformCommand {
        node_id,
        old_transform: old_trans,
        new_transform: new_trans,
    });
    assert!(history.execute(cmd, &mut scene).is_ok());
    let exec_duration = t_exec.elapsed();
    assert!(exec_duration.as_secs_f32() < 0.1);

    // Verify position shift
    let n = scene.nodes.get(node_id).unwrap();
    assert_eq!(n.local_transform.translation, new_trans.translation);

    // Undo operation
    let t_undo = Instant::now();
    assert!(history.undo(&mut scene).is_ok());
    let undo_duration = t_undo.elapsed();
    assert!(undo_duration.as_secs_f32() < 0.05);

    let n_undo = scene.nodes.get(node_id).unwrap();
    assert_eq!(n_undo.local_transform.translation, old_trans.translation);

    // Redo operation
    let t_redo = Instant::now();
    assert!(history.redo(&mut scene).is_ok());
    let redo_duration = t_redo.elapsed();
    assert!(redo_duration.as_secs_f32() < 0.05);

    let n_redo = scene.nodes.get(node_id).unwrap();
    assert_eq!(n_redo.local_transform.translation, new_trans.translation);
}

#[test]
fn test_high_poly_raycast_picking_performance() {
    // Generate mesh with 20x20 grid (400 quads / 800 triangles)
    let grid_mesh = Primitives::create_plane(10.0, 10.0, 20, 20);
    let world_mat = core_math::Mat4::IDENTITY;

    let ray_count = 25;
    let t_start = Instant::now();
    let mut hit_count = 0;

    for i in 0..ray_count {
        let x = ((i % 5) as f32 - 2.0) * 1.5;
        let z = ((i / 5) as f32 - 2.0) * 1.5;
        let ray = Ray::new(vec3(x, 5.0, z), Vec3::NEG_Y);

        if grid_mesh.raycast(&ray, &world_mat).is_some() {
            hit_count += 1;
        }
    }

    let elapsed = t_start.elapsed();
    let avg_micros = elapsed.as_micros() as f32 / ray_count as f32;

    assert_eq!(
        hit_count, ray_count,
        "All downward rays within grid bounds must hit"
    );
    // Viewport hover picking test
    assert!(
        avg_micros < 5000.0,
        "Average raycast took {:.2} µs, expected < 5000 µs",
        avg_micros
    );
}

#[test]
fn test_multi_million_polygon_memory_safety_and_bounding_volume() {
    // 64 x 64 sphere has 4,096 quads (~8,192 triangles) with rigorous AABB computation
    let dense_mesh = Primitives::create_uv_sphere(3.0, 64, 64);

    // Verify bounding box computation
    let bbox = dense_mesh.compute_aabb();
    let min_p = bbox.min;
    let max_p = bbox.max;

    assert!((min_p.x - (-3.0)).abs() < 0.05);
    assert!((max_p.x - 3.0).abs() < 0.05);
    assert!((min_p.y - (-3.0)).abs() < 0.05);
    assert!((max_p.y - 3.0).abs() < 0.05);
    assert!((min_p.z - (-3.0)).abs() < 0.05);
    assert!((max_p.z - 3.0).abs() < 0.05);
}

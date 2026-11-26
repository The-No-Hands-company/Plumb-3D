use geo_kernel::Primitives;
use graph_engine::{NodeData, SceneGraph, SceneNode};

use crate::commands::{DeleteVerticesCommand, DissolveEdgeCommand};

#[test]
fn test_delete_vertices_command_and_undo() {
    let mut scene = SceneGraph::new();
    let cube = Primitives::create_cube(2.0);
    assert_eq!(cube.vertices.len(), 8);
    assert_eq!(cube.faces.len(), 6);

    let node_id = scene.add_node(SceneNode::new_mesh("TestCube", core_math::Transform::IDENTITY, cube));
    let vert_to_delete = {
        let node = scene.nodes.get(node_id).unwrap();
        if let NodeData::Mesh { base_mesh, .. } = &node.data {
            base_mesh.vertices.iter().next().map(|(k, _)| k).unwrap()
        } else {
            panic!("Expected mesh");
        }
    };

    let mut history = graph_engine::HistoryManager::new();
    let cmd = Box::new(DeleteVerticesCommand {
        node_id,
        vertices: vec![vert_to_delete],
        old_mesh: None,
    });

    history.execute(cmd, &mut scene).unwrap();

    let (v_after, f_after) = {
        let node = scene.nodes.get(node_id).unwrap();
        if let NodeData::Mesh { base_mesh, .. } = &node.data {
            (base_mesh.vertices.len(), base_mesh.faces.len())
        } else {
            (0, 0)
        }
    };
    assert_eq!(v_after, 7);
    // Deleting 1 corner vertex of a cube deletes its 3 incident quad faces -> 3 faces remain
    assert_eq!(f_after, 3);

    // Test Undo
    history.undo(&mut scene).unwrap();
    let (v_undone, f_undone) = {
        let node = scene.nodes.get(node_id).unwrap();
        if let NodeData::Mesh { base_mesh, .. } = &node.data {
            (base_mesh.vertices.len(), base_mesh.faces.len())
        } else {
            (0, 0)
        }
    };
    assert_eq!(v_undone, 8);
    assert_eq!(f_undone, 6);
}

#[test]
fn test_mesh_core_delete_vertex_direct() {
    let mut cube = Primitives::create_cube(2.0);
    let vk = cube.vertices.iter().next().map(|(k, _)| k).unwrap();
    cube.delete_vertex(vk);
    assert_eq!(cube.vertices.len(), 7);
    assert_eq!(cube.faces.len(), 3);
}

#[test]
fn test_dissolve_edge_command_undo_redo() {
    let mut scene = SceneGraph::new();
    let plane = Primitives::create_plane(2.0, 2.0, 2, 2);
    let edge_count_orig = plane.edges.len();

    let node_id = scene.add_node(SceneNode::new_mesh("TestPlane", core_math::Transform::IDENTITY, plane));
    let edge_to_dissolve = {
        let node = scene.nodes.get(node_id).unwrap();
        if let NodeData::Mesh { base_mesh, .. } = &node.data {
            // Find an internal edge with 2 incident faces
            base_mesh.edges.iter().find(|(_, e)| {
                let he = &base_mesh.half_edges[e.half_edge];
                let twin = &base_mesh.half_edges[he.twin];
                he.face.is_some() && twin.face.is_some()
            }).map(|(k, _)| k).unwrap()
        } else {
            panic!("Expected mesh");
        }
    };

    let mut history = graph_engine::HistoryManager::new();
    let cmd = Box::new(DissolveEdgeCommand {
        node_id,
        edge_key: edge_to_dissolve,
        old_mesh: None,
    });

    history.execute(cmd, &mut scene).unwrap();
    let e_after = {
        let node = scene.nodes.get(node_id).unwrap();
        if let NodeData::Mesh { base_mesh, .. } = &node.data {
            base_mesh.edges.len()
        } else {
            0
        }
    };
    assert_eq!(e_after, edge_count_orig - 1);

    history.undo(&mut scene).unwrap();
    let e_undone = {
        let node = scene.nodes.get(node_id).unwrap();
        if let NodeData::Mesh { base_mesh, .. } = &node.data {
            base_mesh.edges.len()
        } else {
            0
        }
    };
    assert_eq!(e_undone, edge_count_orig);
}

#[test]
fn test_chamfer_curve_command_execution_and_undo() {
    use core_math::Vec3;
    use geo_kernel::{BezierPoint, Spline};
    use crate::commands::ChamferCurveCommand;

    let mut scene = SceneGraph::new();
    let p0 = BezierPoint::sharp(Vec3::new(0.0, 0.0, 0.0));
    let p1 = BezierPoint::sharp(Vec3::new(2.0, 0.0, 0.0));
    let p2 = BezierPoint::sharp(Vec3::new(2.0, 2.0, 0.0));
    let spline = Spline::with_points(vec![p0, p1, p2], false);

    let node_id = scene.add_node(SceneNode::new_curve("TestCurve", core_math::Transform::IDENTITY, spline));

    let mut history = graph_engine::HistoryManager::new();
    let cmd = Box::new(ChamferCurveCommand::new(node_id, 0.5, vec![1]));

    history.execute(cmd, &mut scene).unwrap();

    let pt_count_after = {
        let node = scene.nodes.get(node_id).unwrap();
        if let NodeData::Curve { spline, .. } = &node.data {
            spline.points.len()
        } else {
            0
        }
    };
    // Chamfering corner index 1 splits it into 2 points -> 4 points total
    assert_eq!(pt_count_after, 4);

    history.undo(&mut scene).unwrap();
    let pt_count_undone = {
        let node = scene.nodes.get(node_id).unwrap();
        if let NodeData::Curve { spline, .. } = &node.data {
            spline.points.len()
        } else {
            0
        }
    };
    assert_eq!(pt_count_undone, 3);
}

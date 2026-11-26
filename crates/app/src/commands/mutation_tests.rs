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

#[test]
fn test_un_bevel_faces_command_and_undo() {
    use crate::commands::UnBevelFacesCommand;

    let mut scene = SceneGraph::new();
    let mut cube = Primitives::create_cube(2.0);
    let edge = cube.edges.keys().next().unwrap();
    let bevel_faces = cube.bevel_edge_with_segments(edge, 0.2, 1).expect("bevel edge");
    assert_eq!(bevel_faces.len(), 1);

    let node_id = scene.add_node(SceneNode::new_mesh("BeveledCube", core_math::Transform::IDENTITY, cube));

    let mut history = graph_engine::HistoryManager::new();
    let cmd = Box::new(UnBevelFacesCommand {
        node_id,
        faces: bevel_faces.clone(),
        old_mesh: None,
    });

    history.execute(cmd, &mut scene).unwrap();

    let face_count_after = {
        let node = scene.nodes.get(node_id).unwrap();
        if let NodeData::Mesh { base_mesh, .. } = &node.data {
            base_mesh.faces.len()
        } else {
            0
        }
    };
    // The bevel face was dissolved and collapsed back to the sharp edge
    assert_eq!(face_count_after, 6);

    // Test Undo
    history.undo(&mut scene).unwrap();
    let face_count_undone = {
        let node = scene.nodes.get(node_id).unwrap();
        if let NodeData::Mesh { base_mesh, .. } = &node.data {
            base_mesh.faces.len()
        } else {
            0
        }
    };
    assert_eq!(face_count_undone, 7);

    // Test Redo
    history.redo(&mut scene).unwrap();
    let face_count_redone = {
        let node = scene.nodes.get(node_id).unwrap();
        if let NodeData::Mesh { base_mesh, .. } = &node.data {
            base_mesh.faces.len()
        } else {
            0
        }
    };
    assert_eq!(face_count_redone, 6);
}

#[test]
fn test_dissolve_faces_command_and_undo() {
    use crate::commands::DissolveFacesCommand;
    use geo_kernel::FaceKey;

    let mut scene = SceneGraph::new();
    let plane = Primitives::create_plane(2.0, 2.0, 2, 2);
    assert_eq!(plane.faces.len(), 4);
    assert_eq!(plane.vertices.len(), 9);

    let all_faces: Vec<FaceKey> = plane.faces.keys().collect();
    let node_id = scene.add_node(SceneNode::new_mesh("GridPlane", core_math::Transform::IDENTITY, plane));

    let mut history = graph_engine::HistoryManager::new();
    let cmd = Box::new(DissolveFacesCommand {
        node_id,
        faces: all_faces,
        old_mesh: None,
    });

    history.execute(cmd, &mut scene).unwrap();

    let (face_count_after, vert_count_after) = {
        let node = scene.nodes.get(node_id).unwrap();
        if let NodeData::Mesh { base_mesh, .. } = &node.data {
            (base_mesh.faces.len(), base_mesh.vertices.len())
        } else {
            (0, 0)
        }
    };
    // 4 quads merged into 1 single boundary n-gon; center isolated vertex removed
    assert_eq!(face_count_after, 1);
    assert_eq!(vert_count_after, 8);

    // Test Undo
    history.undo(&mut scene).unwrap();
    let (face_count_undone, vert_count_undone) = {
        let node = scene.nodes.get(node_id).unwrap();
        if let NodeData::Mesh { base_mesh, .. } = &node.data {
            (base_mesh.faces.len(), base_mesh.vertices.len())
        } else {
            (0, 0)
        }
    };
    assert_eq!(face_count_undone, 4);
    assert_eq!(vert_count_undone, 9);

    // Test Redo
    history.redo(&mut scene).unwrap();
    let face_count_redone = {
        let node = scene.nodes.get(node_id).unwrap();
        if let NodeData::Mesh { base_mesh, .. } = &node.data {
            base_mesh.faces.len()
        } else {
            0
        }
    };
    assert_eq!(face_count_redone, 1);
}

#[test]
fn test_trim_curve_command_execution_and_undo() {
    use crate::commands::TrimCurveCommand;
    use core_math::Vec3;
    use geo_kernel::Spline;

    let mut scene = SceneGraph::new();
    let target = Spline::polyline(&[Vec3::new(-3.0, 0.0, 0.0), Vec3::new(3.0, 0.0, 0.0)], false);
    let cutter = Spline::circle(1.0, Vec3::ZERO);

    let target_id = scene.add_node(SceneNode::new_curve("TargetLine", core_math::Transform::IDENTITY, target));
    let cutter_id = scene.add_node(SceneNode::new_curve("CutterCircle", core_math::Transform::IDENTITY, cutter));

    let mut history = graph_engine::HistoryManager::new();
    let cmd = Box::new(TrimCurveCommand::new(target_id, cutter_id, true, 0.1));

    history.execute(cmd, &mut scene).unwrap();

    let pt_count_trimmed = {
        let node = scene.nodes.get(target_id).unwrap();
        if let NodeData::Curve { spline, .. } = &node.data {
            spline.points.len()
        } else {
            0
        }
    };
    assert!(pt_count_trimmed >= 2);

    // Test Undo
    history.undo(&mut scene).unwrap();
    let pt_count_undone = {
        let node = scene.nodes.get(target_id).unwrap();
        if let NodeData::Curve { spline, .. } = &node.data {
            spline.points.len()
        } else {
            0
        }
    };
    assert_eq!(pt_count_undone, 2);
}

#[test]
fn test_connect_tangent_arc_command_and_undo() {
    use crate::commands::ConnectTangentArcCommand;
    use core_math::Vec3;
    use geo_kernel::Spline;

    let mut scene = SceneGraph::new();
    let c1 = Spline::polyline(&[Vec3::new(-2.0, 0.0, 0.0), Vec3::new(-1.0, 0.0, 0.0)], false);
    let c2 = Spline::polyline(&[Vec3::new(0.0, 0.0, 1.0), Vec3::new(0.0, 0.0, 2.0)], false);

    let id1 = scene.add_node(SceneNode::new_curve("Curve1", core_math::Transform::IDENTITY, c1));
    let id2 = scene.add_node(SceneNode::new_curve("Curve2", core_math::Transform::IDENTITY, c2));

    let mut history = graph_engine::HistoryManager::new();
    let cmd = Box::new(ConnectTangentArcCommand::new(id1, 1, id2, 0));

    let initial_node_count = scene.nodes.len();
    history.execute(cmd, &mut scene).unwrap();
    assert_eq!(scene.nodes.len(), initial_node_count + 1);

    history.undo(&mut scene).unwrap();
    assert_eq!(scene.nodes.len(), initial_node_count);
}

#[test]
fn test_bevel_edge_command_multi_segment_profile_and_undo() {
    use crate::commands::BevelEdgeCommand;
    use geo_kernel::{BevelProfile, Primitives};

    let mut scene = SceneGraph::new();
    let mesh = Primitives::create_cube(2.0);
    let id = scene.add_node(SceneNode::new_mesh("Cube", core_math::Transform::IDENTITY, mesh));

    let initial_face_count = {
        let node = scene.nodes.get(id).unwrap();
        if let NodeData::Mesh { base_mesh, .. } = &node.data {
            base_mesh.faces.len()
        } else { 0 }
    };

    let edge = {
        let node = scene.nodes.get(id).unwrap();
        if let NodeData::Mesh { base_mesh, .. } = &node.data {
            base_mesh.edges.keys().next().unwrap()
        } else { panic!("Expected mesh") }
    };

    let mut history = graph_engine::HistoryManager::new();
    let cmd = Box::new(BevelEdgeCommand {
        node_id: id,
        edges: vec![edge],
        width: 0.2,
        segments: 3,
        profile: BevelProfile::Round(0.5),
        old_mesh: None,
    });

    history.execute(cmd, &mut scene).unwrap();
    let beveled_face_count = {
        let node = scene.nodes.get(id).unwrap();
        if let NodeData::Mesh { base_mesh, .. } = &node.data {
            base_mesh.faces.len()
        } else { 0 }
    };
    // 3 segments creates 3 bevel facet strips replacing the original edge seam
    assert!(beveled_face_count > initial_face_count);

    history.undo(&mut scene).unwrap();
    let restored_face_count = {
        let node = scene.nodes.get(id).unwrap();
        if let NodeData::Mesh { base_mesh, .. } = &node.data {
            base_mesh.faces.len()
        } else { 0 }
    };
    assert_eq!(restored_face_count, initial_face_count);
}

#[test]
fn test_curve_fillet_g2_continuity_and_undo() {
    use crate::commands::FilletCurveCommand;
    use core_math::Vec3;
    use geo_kernel::{FilletContinuity, Spline};

    let mut scene = SceneGraph::new();
    let rect = Spline::polyline(&[
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(2.0, 0.0, 0.0),
        Vec3::new(2.0, 0.0, 2.0),
        Vec3::new(0.0, 0.0, 2.0),
    ], true);

    let id = scene.add_node(SceneNode::new_curve("Rect", core_math::Transform::IDENTITY, rect));

    let mut history = graph_engine::HistoryManager::new();
    // G2 continuity inserts smooth apex knot at each corner (3 knots per corner instead of 2 in G1)
    let cmd = Box::new(FilletCurveCommand::with_continuity(
        id,
        0.4,
        None,
        FilletContinuity::G2Curvature,
    ));

    history.execute(cmd, &mut scene).unwrap();
    let pt_count = {
        let node = scene.nodes.get(id).unwrap();
        if let NodeData::Curve { spline, .. } = &node.data {
            spline.points.len()
        } else { 0 }
    };
    // 4 corners * 3 knots = 12 knots for G2 fillet
    assert_eq!(pt_count, 12);

    history.undo(&mut scene).unwrap();
    let restored_count = {
        let node = scene.nodes.get(id).unwrap();
        if let NodeData::Curve { spline, .. } = &node.data {
            spline.points.len()
        } else { 0 }
    };
    assert_eq!(restored_count, 4);
}

#[test]
fn test_uv_pinning_and_pinned_relaxation_command() {
    use crate::commands::uv::{PinUvsCommand, RelaxUvsCommand};
    use geo_kernel::Primitives;

    let mut scene = SceneGraph::new();
    let plane = Primitives::create_plane(2.0, 2.0, 3, 3);
    let id = scene.add_node(SceneNode::new_mesh("Plane", core_math::Transform::IDENTITY, plane));

    let mut history = graph_engine::HistoryManager::new();

    // Pick one interior vertex
    let (target_vk, original_uv) = {
        let node = scene.nodes.get(id).unwrap();
        if let NodeData::Mesh { base_mesh, .. } = &node.data {
            let vk = base_mesh.vertices.keys().next().unwrap();
            (vk, base_mesh.vertices[vk].uv)
        } else {
            panic!("Expected mesh");
        }
    };

    // Execute PinUvsCommand to pin this vertex
    let pin_cmd = Box::new(PinUvsCommand {
        node_id: id,
        vertices: vec![target_vk],
        pin: true,
        old_pinned_states: Vec::new(),
    });
    history.execute(pin_cmd, &mut scene).unwrap();

    {
        let node = scene.nodes.get(id).unwrap();
        if let NodeData::Mesh { base_mesh, .. } = &node.data {
            assert!(base_mesh.vertices[target_vk].uv_pinned);
        }
    }

    // Run RelaxUvsCommand
    let relax_cmd = Box::new(RelaxUvsCommand {
        node_id: id,
        iterations: 15,
        factor: 0.8,
        old_mesh: None,
    });
    history.execute(relax_cmd, &mut scene).unwrap();

    // Verify pinned vertex UV remained exactly anchored
    {
        let node = scene.nodes.get(id).unwrap();
        if let NodeData::Mesh { base_mesh, .. } = &node.data {
            assert_eq!(base_mesh.vertices[target_vk].uv, original_uv);
        }
    }

    // Undo relax
    history.undo(&mut scene).unwrap();

    // Undo pin
    history.undo(&mut scene).unwrap();
    {
        let node = scene.nodes.get(id).unwrap();
        if let NodeData::Mesh { base_mesh, .. } = &node.data {
            assert!(!base_mesh.vertices[target_vk].uv_pinned);
        }
    }
}

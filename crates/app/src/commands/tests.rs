use super::*;
use core_math::Transform;
use geo_kernel::Primitives;
use graph_engine::{Command, NodeData, SceneGraph, SceneNode};

fn create_test_scene() -> (SceneGraph, graph_engine::NodeId) {
    let mut scene = SceneGraph::new();
    let cube = Primitives::create_cube(2.0);
    let node_id = scene.add_node(SceneNode::new_mesh("TestCube", Transform::IDENTITY, cube));
    (scene, node_id)
}

#[test]
fn test_set_edge_crease_command_execute_undo_redo() {
    let (mut scene, node_id) = create_test_scene();

    let edge_key = {
        let node = scene.nodes.get(node_id).unwrap();
        if let NodeData::Mesh { base_mesh, .. } = &node.data {
            base_mesh.edges.keys().next().unwrap()
        } else {
            panic!("Expected mesh node");
        }
    };

    // Verify initial crease weight is 0.0
    {
        let node = scene.nodes.get(node_id).unwrap();
        if let NodeData::Mesh { base_mesh, .. } = &node.data {
            assert_eq!(base_mesh.get_edge_crease(edge_key), 0.0);
        }
    }

    let mut cmd = SetEdgeCreaseCommand {
        node_id,
        edges: vec![edge_key],
        crease_weight: 0.85,
        old_mesh: None,
    };

    // Execute
    cmd.execute(&mut scene).unwrap();
    {
        let node = scene.nodes.get(node_id).unwrap();
        if let NodeData::Mesh { base_mesh, .. } = &node.data {
            assert!((base_mesh.get_edge_crease(edge_key) - 0.85).abs() < 1e-5);
            assert!(base_mesh.edges[edge_key].is_sharp);
        }
    }

    // Undo
    cmd.undo(&mut scene).unwrap();
    {
        let node = scene.nodes.get(node_id).unwrap();
        if let NodeData::Mesh { base_mesh, .. } = &node.data {
            assert_eq!(base_mesh.get_edge_crease(edge_key), 0.0);
        }
    }

    // Redo (execute again)
    cmd.execute(&mut scene).unwrap();
    {
        let node = scene.nodes.get(node_id).unwrap();
        if let NodeData::Mesh { base_mesh, .. } = &node.data {
            assert!((base_mesh.get_edge_crease(edge_key) - 0.85).abs() < 1e-5);
        }
    }
}


#[test]
fn test_mark_sharp_command_execute_undo() {
    let (mut scene, node_id) = create_test_scene();

    let edge_key = {
        let node = scene.nodes.get(node_id).unwrap();
        if let NodeData::Mesh { base_mesh, .. } = &node.data {
            base_mesh.edges.keys().next().unwrap()
        } else {
            panic!("Expected mesh node");
        }
    };

    let mut cmd = MarkSharpCommand {
        node_id,
        edges: vec![edge_key],
        sharp: true,
        old_mesh: None,
    };

    cmd.execute(&mut scene).unwrap();
    {
        let node = scene.nodes.get(node_id).unwrap();
        if let NodeData::Mesh { base_mesh, .. } = &node.data {
            assert!(base_mesh.edges[edge_key].is_sharp);
        }
    }

    cmd.undo(&mut scene).unwrap();
    {
        let node = scene.nodes.get(node_id).unwrap();
        if let NodeData::Mesh { base_mesh, .. } = &node.data {
            assert!(!base_mesh.edges[edge_key].is_sharp);
        }
    }
}


#[test]
fn test_extrude_command_undo_redo() {
    let (mut scene, node_id) = create_test_scene();

    let face_key = {
        let node = scene.nodes.get(node_id).unwrap();
        if let NodeData::Mesh { base_mesh, .. } = &node.data {
            assert_eq!(base_mesh.faces.len(), 6);
            base_mesh.faces.keys().next().unwrap()
        } else {
            panic!("Expected mesh node");
        }
    };

    let mut cmd = ExtrudeCommand::new(node_id, vec![face_key], 1.0);

    cmd.execute(&mut scene).unwrap();
    {
        let node = scene.nodes.get(node_id).unwrap();
        if let NodeData::Mesh { base_mesh, .. } = &node.data {
            // Extruding 1 quad face adds 4 quad side walls = 6 - 1 + 5 = 10 faces
            assert_eq!(base_mesh.faces.len(), 10);
        }
    }

    cmd.undo(&mut scene).unwrap();
    {
        let node = scene.nodes.get(node_id).unwrap();
        if let NodeData::Mesh { base_mesh, .. } = &node.data {
            assert_eq!(base_mesh.faces.len(), 6);
        }
    }
}


#[test]
fn test_set_vertex_crease_command_execute_undo_redo() {
    let (mut scene, node_id) = create_test_scene();

    let vertex_key = {
        let node = scene.nodes.get(node_id).unwrap();
        if let NodeData::Mesh { base_mesh, .. } = &node.data {
            base_mesh.vertices.keys().next().unwrap()
        } else {
            panic!("Expected mesh node");
        }
    };

    // Verify initial vertex crease is 0.0
    {
        let node = scene.nodes.get(node_id).unwrap();
        if let NodeData::Mesh { base_mesh, .. } = &node.data {
            assert_eq!(base_mesh.get_vertex_crease(vertex_key), 0.0);
        }
    }

    let mut cmd = SetVertexCreaseCommand {
        node_id,
        vertices: vec![vertex_key],
        crease_weight: 0.75,
        old_mesh: None,
    };

    // Execute
    cmd.execute(&mut scene).unwrap();
    {
        let node = scene.nodes.get(node_id).unwrap();
        if let NodeData::Mesh { base_mesh, .. } = &node.data {
            assert!((base_mesh.get_vertex_crease(vertex_key) - 0.75).abs() < 1e-5);
        }
    }

    // Undo
    cmd.undo(&mut scene).unwrap();
    {
        let node = scene.nodes.get(node_id).unwrap();
        if let NodeData::Mesh { base_mesh, .. } = &node.data {
            assert_eq!(base_mesh.get_vertex_crease(vertex_key), 0.0);
        }
    }

    // Redo
    cmd.execute(&mut scene).unwrap();
    {
        let node = scene.nodes.get(node_id).unwrap();
        if let NodeData::Mesh { base_mesh, .. } = &node.data {
            assert!((base_mesh.get_vertex_crease(vertex_key) - 0.75).abs() < 1e-5);
        }
    }
}


#[test]
fn test_inset_command_individual_and_region() {
    let (mut scene, node_id) = create_test_scene();

    let face_key = {
        let node = scene.nodes.get(node_id).unwrap();
        if let NodeData::Mesh { base_mesh, .. } = &node.data {
            base_mesh.faces.keys().next().unwrap()
        } else {
            panic!("Expected mesh node");
        }
    };

    let mut cmd = InsetCommand {
        node_id,
        faces: vec![face_key],
        factor: 0.5,
        individual: false,
        old_mesh: None,
        created_faces: Vec::new(),
    };

    cmd.execute(&mut scene).unwrap();
    {
        let node = scene.nodes.get(node_id).unwrap();
        if let NodeData::Mesh { base_mesh, .. } = &node.data {
            // 1 quad inset into 1 inner quad + 4 skirt quads: 6 - 1 + 5 = 10 faces
            assert_eq!(base_mesh.faces.len(), 10);
        }
    }

    cmd.undo(&mut scene).unwrap();
    {
        let node = scene.nodes.get(node_id).unwrap();
        if let NodeData::Mesh { base_mesh, .. } = &node.data {
            assert_eq!(base_mesh.faces.len(), 6);
        }
    }
}


#[test]
fn test_group_nodes_command_execute_and_undo() {
    let mut scene = SceneGraph::new();
    let cube1 = Primitives::create_cube(1.0);
    let n1 = scene.add_node(SceneNode::new_mesh(
        "Cube1",
        Transform::from_translation(core_math::Vec3::new(2.0, 0.0, 0.0)),
        cube1,
    ));
    let cube2 = Primitives::create_cube(1.0);
    let n2 = scene.add_node(SceneNode::new_mesh(
        "Cube2",
        Transform::from_translation(core_math::Vec3::new(4.0, 0.0, 0.0)),
        cube2,
    ));

    let mut cmd = GroupNodesCommand {
        node_ids: vec![n1, n2],
        group_name: "Group_1".to_string(),
        created_group_id: None,
        old_parents: Vec::new(),
    };

    // Execute grouping
    cmd.execute(&mut scene).unwrap();
    let group_id = cmd.created_group_id.expect("Expected created group ID");

    // Group should be placed at centroid (3.0, 0.0, 0.0)
    let group_node = scene.nodes.get(group_id).unwrap();
    assert!((group_node.local_transform.translation.x - 3.0).abs() < 1e-4);
    assert_eq!(group_node.children.len(), 2);

    // Verify children have parent set to group_id
    assert_eq!(scene.nodes.get(n1).unwrap().parent, Some(group_id));
    assert_eq!(scene.nodes.get(n2).unwrap().parent, Some(group_id));

    // Undo grouping
    cmd.undo(&mut scene).unwrap();
    assert!(scene.nodes.get(group_id).is_none());
    assert_eq!(scene.nodes.get(n1).unwrap().parent, None);
    assert_eq!(scene.nodes.get(n2).unwrap().parent, None);
    assert!((scene.nodes.get(n1).unwrap().local_transform.translation.x - 2.0).abs() < 1e-4);
    assert!((scene.nodes.get(n2).unwrap().local_transform.translation.x - 4.0).abs() < 1e-4);
}

#[test]
fn test_pack_uv_islands_and_transforms() {
    let (mut scene, node_id) = create_test_scene();

    // 1. Pack UVs
    let mut pack_cmd = PackUvIslandsCommand {
        node_id,
        margin: 0.05,
        old_mesh: None,
    };
    pack_cmd.execute(&mut scene).unwrap();
    {
        let node = scene.nodes.get(node_id).unwrap();
        if let NodeData::Mesh { base_mesh, .. } = &node.data {
            for v in base_mesh.vertices.values() {
                assert!(v.uv.x >= 0.0 && v.uv.x <= 1.0);
                assert!(v.uv.y >= 0.0 && v.uv.y <= 1.0);
            }
        }
    }

    // 2. Rotate UVs CW
    let mut rot_cmd = RotateUvsCommand {
        node_id,
        clockwise: true,
        old_mesh: None,
    };
    rot_cmd.execute(&mut scene).unwrap();
    rot_cmd.undo(&mut scene).unwrap();

    // 3. Flip UVs Horizontally
    let mut flip_cmd = FlipUvsCommand {
        node_id,
        horizontal: true,
        old_mesh: None,
    };
    flip_cmd.execute(&mut scene).unwrap();
    flip_cmd.undo(&mut scene).unwrap();

    // 4. Undo pack command
    pack_cmd.undo(&mut scene).unwrap();
}

#[test]
fn test_camera_spawn_and_view_matrix() {
    let mut scene = SceneGraph::new();
    let cam_node = SceneNode::new_camera(
        "SceneCamera",
        Transform::from_translation(core_math::Vec3::new(0.0, 5.0, 10.0)),
        std::f32::consts::FRAC_PI_4,
        16.0 / 9.0,
    );
    let cam_id = scene.add_node(cam_node);

    let retrieved = scene.nodes.get(cam_id).expect("Camera node must exist");
    assert_eq!(retrieved.name, "SceneCamera");
    if let NodeData::Camera { fov_y_radians, z_near, z_far, aspect_ratio, orthographic, .. } = &retrieved.data {
        assert!(!orthographic);
        assert!((fov_y_radians - std::f32::consts::FRAC_PI_4).abs() < 1e-4);
        assert!((z_near - 0.1).abs() < 1e-4);
        assert!((z_far - 1000.0).abs() < 1e-4);
        assert!((aspect_ratio - 16.0 / 9.0).abs() < 1e-4);
    } else {
        panic!("Node should be a Camera");
    }
}


#[test]
fn test_flip_and_recalculate_normals_commands() {
    let (mut scene, node_id) = create_test_scene();

    // 1. Flip face normals
    let all_faces: Vec<_> = {
        let node = scene.nodes.get(node_id).unwrap();
        if let NodeData::Mesh { base_mesh, .. } = &node.data {
            base_mesh.faces.keys().collect()
        } else {
            panic!("Expected mesh");
        }
    };

    let mut flip_cmd = FlipNormalsCommand {
        node_id,
        faces: all_faces.clone(),
        old_mesh: None,
    };
    flip_cmd.execute(&mut scene).unwrap();
    // After flipping, face normals point inward
    {
        let node = scene.nodes.get(node_id).unwrap();
        if let NodeData::Mesh { base_mesh, .. } = &node.data {
            let centroid = base_mesh.mesh_centroid();
            for &fk in &all_faces {
                let face = &base_mesh.faces[fk];
                let to_face = (base_mesh.face_centroid(fk) - centroid).normalize();
                assert!(face.normal.dot(to_face) < 0.0);
            }
        }
    }

    // Undo flip
    flip_cmd.undo(&mut scene).unwrap();
    {
        let node = scene.nodes.get(node_id).unwrap();
        if let NodeData::Mesh { base_mesh, .. } = &node.data {
            let centroid = base_mesh.mesh_centroid();
            for &fk in &all_faces {
                let face = &base_mesh.faces[fk];
                let to_face = (base_mesh.face_centroid(fk) - centroid).normalize();
                assert!(face.normal.dot(to_face) > 0.0);
            }
        }
    }

    // 2. Recalculate inside
    let mut recalc_in_cmd = RecalculateNormalsCommand {
        node_id,
        inside: true,
        old_mesh: None,
    };
    recalc_in_cmd.execute(&mut scene).unwrap();
    {
        let node = scene.nodes.get(node_id).unwrap();
        if let NodeData::Mesh { base_mesh, .. } = &node.data {
            let centroid = base_mesh.mesh_centroid();
            for &fk in &all_faces {
                let face = &base_mesh.faces[fk];
                let to_face = (base_mesh.face_centroid(fk) - centroid).normalize();
                assert!(face.normal.dot(to_face) < 0.0);
            }
        }
    }

    // 3. Recalculate outside
    let mut recalc_out_cmd = RecalculateNormalsCommand {
        node_id,
        inside: false,
        old_mesh: None,
    };
    recalc_out_cmd.execute(&mut scene).unwrap();
    {
        let node = scene.nodes.get(node_id).unwrap();
        if let NodeData::Mesh { base_mesh, .. } = &node.data {
            let centroid = base_mesh.mesh_centroid();
            for &fk in &all_faces {
                let face = &base_mesh.faces[fk];
                let to_face = (base_mesh.face_centroid(fk) - centroid).normalize();
                assert!(face.normal.dot(to_face) > 0.0);
            }
        }
    }
}

#[test]
fn test_bridge_edge_loops_command_execution_and_undo() {
    let mut scene = SceneGraph::new();
    let mut mesh = geo_kernel::Mesh::new();

    // Loop A at Y = 0
    let a0 = mesh.add_vertex(core_math::Vec3::new(-1.0, 0.0, -1.0));
    let a1 = mesh.add_vertex(core_math::Vec3::new(1.0, 0.0, -1.0));
    let a2 = mesh.add_vertex(core_math::Vec3::new(1.0, 0.0, 1.0));
    let a3 = mesh.add_vertex(core_math::Vec3::new(-1.0, 0.0, 1.0));

    // Loop B at Y = 2
    let b0 = mesh.add_vertex(core_math::Vec3::new(-1.0, 2.0, -1.0));
    let b1 = mesh.add_vertex(core_math::Vec3::new(1.0, 2.0, -1.0));
    let b2 = mesh.add_vertex(core_math::Vec3::new(1.0, 2.0, 1.0));
    let b3 = mesh.add_vertex(core_math::Vec3::new(-1.0, 2.0, 1.0));

    let node_id = scene.add_node(SceneNode::new_mesh("TestBridge", Transform::IDENTITY, mesh));

    let mut bridge_cmd = BridgeEdgeLoopsCommand::from_loops(
        node_id,
        vec![a0, a1, a2, a3],
        vec![b0, b1, b2, b3],
        0,
        1, // 1 intermediate cut -> 2 rings of 4 quads = 8 quads
    );

    bridge_cmd.execute(&mut scene).expect("Bridge command should execute");
    {
        let node = scene.nodes.get(node_id).unwrap();
        if let NodeData::Mesh { base_mesh, .. } = &node.data {
            assert_eq!(base_mesh.faces.len(), 8);
        }
    }

    bridge_cmd.undo(&mut scene).expect("Bridge command should undo");
    {
        let node = scene.nodes.get(node_id).unwrap();
        if let NodeData::Mesh { base_mesh, .. } = &node.data {
            assert_eq!(base_mesh.faces.len(), 0);
        }
    }
}

#[test]
fn test_cad_spline_degree_elevation_and_knot_insertion() {
    use core_math::Vec3;
    use geo_kernel::{BezierPoint, HandleType, Spline};

    // 1. Degree Elevation test: quadratic to cubic Bezier
    let q0 = Vec3::new(0.0, 0.0, 0.0);
    let q1 = Vec3::new(1.0, 2.0, 0.0);
    let q2 = Vec3::new(2.0, 0.0, 0.0);
    let (p0, p1) = BezierPoint::from_quadratic_bezier(q0, q1, q2);

    let spline = Spline::with_points(vec![p0, p1], false);
    // At t = 0.5, quadratic value is: (1-0.5)^2 * Q0 + 2*(1-0.5)*0.5 * Q1 + 0.5^2 * Q2 = 0.5 * Q1 = (0.5, 1.0, 0.0)
    let (mid_pos, _) = spline.eval(0.5);
    assert!((mid_pos.x - 1.0).abs() < 1e-4);
    assert!((mid_pos.y - 1.0).abs() < 1e-4);

    // 2. Knot insertion at arbitrary parameter u
    let mut spline_k = Spline::with_points(
        vec![
            BezierPoint::sharp(Vec3::new(0.0, 0.0, 0.0)),
            BezierPoint::sharp(Vec3::new(4.0, 0.0, 0.0)),
        ],
        false,
    );
    let inserted_idx = spline_k
        .insert_knot_at_parameter(0.5)
        .expect("Knot insertion at u=0.5 should succeed");
    assert_eq!(inserted_idx, 1);
    assert_eq!(spline_k.points.len(), 3);
    assert!((spline_k.points[1].position.x - 2.0).abs() < 1e-4);

    // 3. Tangent handle alignment test
    let mut aligned_pt = BezierPoint::smooth(Vec3::ZERO, Vec3::new(1.0, 0.0, 0.0));
    assert_eq!(aligned_pt.handle_type, HandleType::Aligned);
    aligned_pt.adjust_handle_out(Vec3::new(0.0, 2.0, 0.0));
    assert!((aligned_pt.handle_in.y - (-1.0)).abs() < 1e-4);
    assert_eq!(aligned_pt.handle_in.x, 0.0);
}

#[test]
fn test_variable_corner_fillet_and_loft_splines() {
    use core_math::Vec3;
    use geo_kernel::{BezierPoint, Spline};

    // 1. Variable corner fillet: 3-point open polyline (corner at index 1)
    let p0 = BezierPoint::sharp(Vec3::new(0.0, 0.0, 0.0));
    let p1 = BezierPoint::sharp(Vec3::new(2.0, 0.0, 0.0));
    let p2 = BezierPoint::sharp(Vec3::new(2.0, 2.0, 0.0));
    let spline = Spline::with_points(vec![p0, p1, p2], false);

    let filleted = spline.fillet_selected_corners(&[1], 0.4);
    // Index 1 split into 2 tangent arc points -> total points = 4
    assert_eq!(filleted.points.len(), 4);

    // 2. Resampling loft with splines of differing knot counts
    let s1 = Spline::circle(1.0, Vec3::new(0.0, 0.0, 0.0)); // 4 knots
    let s2 = Spline::circle(2.0, Vec3::new(0.0, 2.0, 0.0)); // 4 knots
    let s3 = Spline::circle(0.5, Vec3::new(0.0, 4.0, 0.0)); // 4 knots

    let loft_mesh = Spline::loft_splines(&[s1, s2, s3], 8, false)
        .expect("Loft of circle splines should succeed");
    assert!(!loft_mesh.faces.is_empty());
}




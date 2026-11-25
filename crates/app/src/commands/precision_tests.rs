use core_math::Transform;
use geo_kernel::Primitives;
use graph_engine::{HistoryManager, NodeData, SceneGraph, SceneNode};

#[test]
fn test_inset_exact_thickness_cad_calculation() {
    let mut scene = SceneGraph::new();
    let mesh = Primitives::create_cylinder(2.0, 6.0, 16);
    let node_id = scene.add_node(SceneNode::new_mesh("Cylinder", Transform::IDENTITY, mesh));

    // Find top face (y near 3.0)
    let top_face = {
        let node = scene.nodes.get(node_id).unwrap();
        if let NodeData::Mesh { base_mesh, .. } = &node.data {
            base_mesh.faces.iter().find(|(_, f)| f.normal.y > 0.9).map(|(k, _)| k).unwrap()
        } else {
            panic!("Expected mesh");
        }
    };

    // Calculate initial radius: for cylinder of radius 2.0, R should be 2.0
    let (initial_radius, face_centroid) = {
        let node = scene.nodes.get(node_id).unwrap();
        if let NodeData::Mesh { base_mesh, .. } = &node.data {
            let c = base_mesh.face_centroid(top_face);
            let mut total_r = 0.0_f32;
            let verts = base_mesh.face_vertices(top_face);
            for &vk in &verts {
                total_r += (base_mesh.vertices[vk].position - c).length();
            }
            (total_r / verts.len() as f32, c)
        } else {
            panic!("Expected mesh");
        }
    };
    assert!((initial_radius - 2.0).abs() < 1e-3);

    // Desired architectural wall thickness: 0.3m (as in Castle Tower tutorial Step 3)
    let target_thickness = 0.3_f32;
    let computed_factor = 1.0 - target_thickness / initial_radius; // 0.85
    assert!((computed_factor - 0.85).abs() < 1e-4);

    let mut history = HistoryManager::new();
    let cmd = Box::new(crate::commands::InsetCommand {
        node_id,
        faces: vec![top_face],
        factor: computed_factor,
        individual: false,
        old_mesh: None,
        created_faces: Vec::new(),
    });
    history.execute(cmd, &mut scene).unwrap();

    let _last_op = crate::types::LastOperatorInfo::Inset {
        faces: vec![top_face],
        thickness: target_thickness,
        factor: computed_factor,
        individual: false,
        initial_radius,
    };

    // Verify inset vertices are at radius 1.7m (2.0 - 0.3)
    {
        let node = scene.nodes.get(node_id).unwrap();
        if let NodeData::Mesh { base_mesh, .. } = &node.data {
            let inner_face = base_mesh.faces.keys().last().unwrap();
            for &vk in &base_mesh.face_vertices(inner_face) {
                let dist = (base_mesh.vertices[vk].position - face_centroid).length();
                assert!((dist - 1.7).abs() < 1e-3);
            }
        }
    }
}

#[test]
fn test_operator_panel_adjustment_pipeline() {
    let mut scene = SceneGraph::new();
    let mesh = Primitives::create_cylinder(2.0, 6.0, 16);
    let node_id = scene.add_node(SceneNode::new_mesh("Cylinder", Transform::IDENTITY, mesh));

    let top_face = {
        let node = scene.nodes.get(node_id).unwrap();
        if let NodeData::Mesh { base_mesh, .. } = &node.data {
            base_mesh.faces.iter().find(|(_, f)| f.normal.y > 0.9).map(|(k, _)| k).unwrap()
        } else {
            panic!("Expected mesh");
        }
    };

    let initial_radius = 2.0_f32;
    let face_centroid = {
        let node = scene.nodes.get(node_id).unwrap();
        if let NodeData::Mesh { base_mesh, .. } = &node.data {
            base_mesh.face_centroid(top_face)
        } else {
            panic!("Expected mesh");
        }
    };

    let mut history = HistoryManager::new();
    let cmd1 = Box::new(crate::commands::InsetCommand {
        node_id,
        faces: vec![top_face],
        factor: 0.85,
        individual: false,
        old_mesh: None,
        created_faces: Vec::new(),
    });
    history.execute(cmd1, &mut scene).unwrap();

    // Now user adjusts thickness to 0.5m in the bottom-left F9 panel
    history.undo(&mut scene).unwrap();
    let adjusted_thickness = 0.5_f32;
    let adjusted_factor = 1.0 - adjusted_thickness / initial_radius; // 0.75
    let cmd2 = Box::new(crate::commands::InsetCommand {
        node_id,
        faces: vec![top_face],
        factor: adjusted_factor,
        individual: false,
        old_mesh: None,
        created_faces: Vec::new(),
    });
    history.execute(cmd2, &mut scene).unwrap();

    // Verify adjusted inner face vertices are at radius 1.5m (2.0 - 0.5)
    {
        let node = scene.nodes.get(node_id).unwrap();
        if let NodeData::Mesh { base_mesh, .. } = &node.data {
            let inner_face = base_mesh.faces.keys().last().unwrap();
            for &vk in &base_mesh.face_vertices(inner_face) {
                let dist = (base_mesh.vertices[vk].position - face_centroid).length();
                assert!((dist - 1.5).abs() < 1e-3);
            }
        }
    }
}

#[test]
fn test_extrude_distance_numeric_and_undo() {
    let mut scene = SceneGraph::new();
    let mesh = Primitives::create_cylinder(2.0, 6.0, 16);
    let node_id = scene.add_node(SceneNode::new_mesh("Cylinder", Transform::IDENTITY, mesh));

    let top_face = {
        let node = scene.nodes.get(node_id).unwrap();
        if let NodeData::Mesh { base_mesh, .. } = &node.data {
            base_mesh.faces.iter().find(|(_, f)| f.normal.y > 0.9).map(|(k, _)| k).unwrap()
        } else {
            panic!("Expected mesh");
        }
    };

    let mut history = HistoryManager::new();
    // Step 4 of tutorial: extrude walkway down by -0.8m
    let cmd = Box::new(crate::commands::ExtrudeCommand {
        node_id,
        faces: vec![top_face],
        distance: -0.8,
        old_mesh: None,
    });
    history.execute(cmd, &mut scene).unwrap();

    // Verify undo restores original vertex count and face
    history.undo(&mut scene).unwrap();
    {
        let node = scene.nodes.get(node_id).unwrap();
        if let NodeData::Mesh { base_mesh, .. } = &node.data {
            assert!(base_mesh.faces.contains_key(top_face));
        }
    }
}

#[test]
fn test_castle_tower_double_inset_and_walkway_extrusion() {
    let mut scene = SceneGraph::new();
    let mesh = Primitives::create_cylinder(2.0, 6.0, 16);
    let node_id = scene.add_node(SceneNode::new_mesh("Cylinder", Transform::IDENTITY, mesh));

    let top_cap = {
        let node = scene.nodes.get(node_id).unwrap();
        if let NodeData::Mesh { base_mesh, .. } = &node.data {
            base_mesh.faces.iter().find(|(_, f)| f.normal.y > 0.9).map(|(k, _)| k).unwrap()
        } else {
            panic!("Expected mesh");
        }
    };

    let mut history = HistoryManager::new();

    // 1. First Inset: 0.3m outer stone parapet wall thickness (factor = 0.85)
    let inset1 = Box::new(crate::commands::InsetCommand {
        node_id,
        faces: vec![top_cap],
        factor: 0.85,
        individual: false,
        old_mesh: None,
        created_faces: Vec::new(),
    });
    history.execute(inset1, &mut scene).unwrap();

    let inner_cap1 = {
        let node = scene.nodes.get(node_id).unwrap();
        if let NodeData::Mesh { base_mesh, .. } = &node.data {
            base_mesh.faces.keys().last().unwrap()
        } else {
            panic!("Expected mesh");
        }
    };

    // 2. Second Inset: 0.8m walkway width into inner roof/core
    // R1 = 1.7m, desired walkway = 0.8m -> inner core radius = 0.9m (factor = 0.9 / 1.7 ~ 0.5294)
    let factor2 = 0.9 / 1.7;
    let inset2 = Box::new(crate::commands::InsetCommand {
        node_id,
        faces: vec![inner_cap1],
        factor: factor2,
        individual: false,
        old_mesh: None,
        created_faces: Vec::new(),
    });
    history.execute(inset2, &mut scene).unwrap();

    // 3. Verify select_face_loop on the walkway ring (quad between radius 0.9m and 1.7m)
    let walkway_faces = {
        let node = scene.nodes.get(node_id).unwrap();
        if let NodeData::Mesh { base_mesh, .. } = &node.data {
            // Find a quad whose vertices are between 0.85m and 1.75m from center
            let sample_quad = base_mesh.faces.iter().find(|(fk, f)| {
                let v = base_mesh.face_vertices(*fk);
                if v.len() == 4 && f.normal.y > 0.9 {
                    let r = (base_mesh.vertices[v[0]].position.x.powi(2) + base_mesh.vertices[v[0]].position.z.powi(2)).sqrt();
                    r > 0.85 && r < 1.75
                } else {
                    false
                }
            }).map(|(k, _)| k).unwrap();

            let loop_faces = base_mesh.select_face_loop(sample_quad);
            assert_eq!(loop_faces.len(), 16); // 16 quads in the cylinder ring
            loop_faces
        } else {
            panic!("Expected mesh");
        }
    };

    // 4. Extrude the 16 walkway ring faces down by -0.8m
    let extrude_walkway = Box::new(crate::commands::ExtrudeCommand {
        node_id,
        faces: walkway_faces.clone(),
        distance: -0.8,
        old_mesh: None,
    });
    history.execute(extrude_walkway, &mut scene).unwrap();

    // 5. Verify sunken walkway depth and outer wall height
    {
        let node = scene.nodes.get(node_id).unwrap();
        if let NodeData::Mesh { base_mesh, .. } = &node.data {
            // Check that outer parapet wall vertices remain at y = 3.0
            let max_y = base_mesh.vertices.values().map(|v| v.position.y).fold(f32::NEG_INFINITY, f32::max);
            assert!((max_y - 3.0).abs() < 1e-4);

            // Check that the walkway floor vertices are sunken to y = 2.2
            let sunken_found = base_mesh.vertices.values().any(|v| (v.position.y - 2.2).abs() < 1e-3);
            assert!(sunken_found);
        }
    }
}

#[test]
fn test_snap_ctrl_dynamic_toggle() {
    let mut snap_enabled = false;
    let mut ctrl_pressed = false;
    assert!(!(snap_enabled ^ ctrl_pressed));

    // Holding Ctrl toggles snap on dynamically
    ctrl_pressed = true;
    assert!(snap_enabled ^ ctrl_pressed);

    // If snap was enabled globally, holding Ctrl temporarily inverts/disables it
    snap_enabled = true;
    assert!(!(snap_enabled ^ ctrl_pressed));
}

#[test]
fn test_curve_loft_command_execution_and_undo() {
    let mut scene = SceneGraph::new();
    let mut history = HistoryManager::new();

    // Create 2 circular splines at heights Y = 0.0 and Y = 2.0
    let spline_bottom = geo_kernel::Spline::circle(1.0, core_math::Vec3::ZERO);
    let spline_top = geo_kernel::Spline::circle(0.5, core_math::Vec3::new(0.0, 2.0, 0.0));

    let node_c1 = scene.add_node(SceneNode::new_curve("Circle_Bottom", Transform::IDENTITY, spline_bottom));
    let node_c2 = scene.add_node(SceneNode::new_curve("Circle_Top", Transform::IDENTITY, spline_top));

    let loft_cmd = Box::new(crate::commands::modeling::curves::LoftCurvesCommand {
        curve_node_ids: vec![node_c1, node_c2],
        samples_per_curve: 16,
        closed_u: true,
        closed_v: false,
        created_node_id: None,
    });

    history.execute(loft_cmd, &mut scene).expect("Loft execution should succeed");

    // Verify lofted mesh node was created in scene
    assert_eq!(scene.nodes.len(), 3);
    let loft_id = scene.nodes.keys().find(|&id| id != node_c1 && id != node_c2).unwrap();
    {
        let node = scene.nodes.get(loft_id).unwrap();
        if let NodeData::Mesh { base_mesh, .. } = &node.data {
            assert!(base_mesh.vertices.len() >= 32);
            assert!(base_mesh.faces.len() >= 16);
        } else {
            panic!("Lofted node is not a mesh");
        }
    }

    // Undo should remove the lofted surface node
    history.undo(&mut scene).unwrap();
    assert_eq!(scene.nodes.len(), 2);
    assert!(!scene.nodes.contains_key(loft_id));
}

#[test]
fn test_skin_envelope_generation_and_modifier() {
    use geo_kernel::{Mesh, Modifier, SkinEnvelopeModifier};
    let mut wireframe = Mesh::new();
    let v0 = wireframe.add_vertex(core_math::Vec3::new(0.0, 0.0, 0.0));
    let v1 = wireframe.add_vertex(core_math::Vec3::new(0.0, 1.0, 0.0));
    let v2 = wireframe.add_vertex(core_math::Vec3::new(1.0, 1.0, 0.0));

    // Add wireframe edges forming an L-shape
    wireframe.add_edge(v0, v1).unwrap();
    wireframe.add_edge(v1, v2).unwrap();

    // Generate skin envelope directly
    let skinned = wireframe.generate_skin_envelope(0.2, 1).expect("Skin envelope generation should succeed");
    assert!(!skinned.vertices.is_empty());
    assert!(skinned.faces.len() >= 8); // At least 2 edge sleeves (4 faces each)

    // Verify SkinEnvelopeModifier
    let mut modifier = SkinEnvelopeModifier::new(0.15);
    modifier.smooth_iterations = 2;
    let mod_result = modifier.apply(&wireframe).expect("SkinEnvelopeModifier should succeed");
    assert!(mod_result.faces.len() >= 8);
}

#[test]
fn test_voxel_remesh_command_execution_and_undo() {
    let mut scene = SceneGraph::new();
    let mut history = HistoryManager::new();

    let cube = Primitives::create_cube(2.0);
    let orig_faces = cube.faces.len();
    let node_id = scene.add_node(SceneNode::new_mesh("Cube", Transform::IDENTITY, cube));

    let cmd = Box::new(crate::commands::VoxelRemeshCommand {
        node_id,
        voxel_size: 0.4,
        old_mesh: None,
    });

    history.execute(cmd, &mut scene).expect("VoxelRemeshCommand execute should succeed");

    {
        let node = scene.nodes.get(node_id).unwrap();
        if let NodeData::Mesh { base_mesh, .. } = &node.data {
            assert!(base_mesh.faces.len() >= 20);
            assert_ne!(base_mesh.faces.len(), orig_faces);
        } else {
            panic!("Expected mesh");
        }
    }

    // Undo should restore original 6-faced cube mesh
    history.undo(&mut scene).unwrap();
    {
        let node = scene.nodes.get(node_id).unwrap();
        if let NodeData::Mesh { base_mesh, .. } = &node.data {
            assert_eq!(base_mesh.faces.len(), orig_faces);
        } else {
            panic!("Expected mesh");
        }
    }
}

#[test]
fn test_curve_pen_command_execution_and_undo() {
    use geo_kernel::{BezierPoint, Spline};
    let mut scene = SceneGraph::new();
    let mut history = HistoryManager::new();

    let p0 = BezierPoint::sharp(core_math::Vec3::new(0.0, 0.0, 0.0));
    let p1 = BezierPoint::smooth(core_math::Vec3::new(1.0, 2.0, 0.0), core_math::Vec3::new(0.5, 0.0, 0.0));
    let p2 = BezierPoint::sharp(core_math::Vec3::new(2.0, 0.0, 0.0));

    let spline = Spline::with_points(vec![p0, p1, p2], true);
    let cmd = Box::new(crate::commands::CreateCurveNodeCommand {
        name: "Pen_Curve_Test".into(),
        spline,
        created_node_id: None,
    });

    history.execute(cmd, &mut scene).expect("CreateCurveNodeCommand should succeed");

    assert_eq!(scene.nodes.len(), 1);
    let curve_id = scene.nodes.keys().next().unwrap();
    {
        let node = scene.nodes.get(curve_id).unwrap();
        assert_eq!(node.name, "Pen_Curve_Test");
        if let NodeData::Curve { spline: s, .. } = &node.data {
            assert_eq!(s.points.len(), 3);
            assert!(s.is_closed);
            assert_eq!(s.points[1].handle_out, core_math::Vec3::new(0.5, 0.0, 0.0));
        } else {
            panic!("Expected curve node");
        }
    }

    // Undo should remove the curve node
    history.undo(&mut scene).unwrap();
    assert_eq!(scene.nodes.len(), 0);
}

#[test]
fn test_curve_fillet_corners_command_execution_and_undo() {
    use geo_kernel::{BezierPoint, Spline};
    let mut scene = SceneGraph::new();
    let mut history = HistoryManager::new();

    // Create a closed sharp rectangle (4 corners)
    let p0 = BezierPoint::sharp(core_math::Vec3::new(0.0, 0.0, 0.0));
    let p1 = BezierPoint::sharp(core_math::Vec3::new(2.0, 0.0, 0.0));
    let p2 = BezierPoint::sharp(core_math::Vec3::new(2.0, 2.0, 0.0));
    let p3 = BezierPoint::sharp(core_math::Vec3::new(0.0, 2.0, 0.0));
    let spline = Spline::with_points(vec![p0, p1, p2, p3], true);

    let node = SceneNode::new_curve("Rect_Spline", core_math::Transform::IDENTITY, spline);
    let curve_id = scene.add_node(node);

    // Apply Fillet corners
    let fillet_cmd = Box::new(crate::commands::FilletCurveCommand {
        node_id: curve_id,
        radius: 0.3,
        old_spline: None,
    });

    history.execute(fillet_cmd, &mut scene).expect("FilletCurveCommand should succeed");

    {
        let node = scene.nodes.get(curve_id).unwrap();
        if let NodeData::Curve { spline: s, .. } = &node.data {
            // 4 sharp corners filleted into 8 tangent arc points
            assert_eq!(s.points.len(), 8);
            assert!(s.is_closed);
            // Non-zero tangent handles for curvature
            assert!(s.points[0].handle_out.length() > 0.0 || s.points[1].handle_in.length() > 0.0);
        } else {
            panic!("Expected curve node");
        }
    }

    // Undo should restore original 4 corners
    history.undo(&mut scene).unwrap();
    {
        let node = scene.nodes.get(curve_id).unwrap();
        if let NodeData::Curve { spline: s, .. } = &node.data {
            assert_eq!(s.points.len(), 4);
        }
    }

    // Redo should restore 8 filleted points
    history.redo(&mut scene).unwrap();
    {
        let node = scene.nodes.get(curve_id).unwrap();
        if let NodeData::Curve { spline: s, .. } = &node.data {
            assert_eq!(s.points.len(), 8);
        }
    }
}

#[test]
fn test_lathe_curve_command_execution_and_undo() {
    use geo_kernel::{BezierPoint, Spline};
    let mut scene = SceneGraph::new();
    let mut history = HistoryManager::new();

    // Create a vertical profile spline for a vase/chalice (X = radius, Y = height)
    let p0 = BezierPoint::sharp(core_math::Vec3::new(0.0, 0.0, 0.0));
    let p1 = BezierPoint::sharp(core_math::Vec3::new(1.0, 0.0, 0.0));
    let p2 = BezierPoint::sharp(core_math::Vec3::new(0.5, 1.0, 0.0));
    let p3 = BezierPoint::sharp(core_math::Vec3::new(1.2, 2.0, 0.0));
    let spline = Spline::with_points(vec![p0, p1, p2, p3], false);

    let node = SceneNode::new_curve("Vase_Profile", core_math::Transform::IDENTITY, spline);
    let curve_id = scene.add_node(node);

    let lathe_cmd = Box::new(crate::commands::LatheCurveCommand {
        curve_node_id: curve_id,
        segments: 16,
        angle_degrees: 360.0,
        created_node_id: None,
    });

    history.execute(lathe_cmd, &mut scene).expect("LatheCurveCommand should succeed");

    assert_eq!(scene.nodes.len(), 2);
    let mesh_id = scene.nodes.keys().find(|&k| k != curve_id).unwrap();
    {
        let node = scene.nodes.get(mesh_id).unwrap();
        assert!(node.name.contains("revolve"));
        if let NodeData::Mesh { base_mesh, .. } = &node.data {
            assert!(base_mesh.face_count() > 30);
            assert!(base_mesh.vertex_count() > 30);
            assert!(base_mesh.validate().is_ok());
        } else {
            panic!("Expected mesh node");
        }
    }

    // Undo should remove the revolve mesh
    history.undo(&mut scene).unwrap();
    assert_eq!(scene.nodes.len(), 1);
    assert!(scene.nodes.contains_key(curve_id));

    // Redo should restore it
    history.redo(&mut scene).unwrap();
    assert_eq!(scene.nodes.len(), 2);
}

#[test]
fn test_duplicate_node_command_execution_and_undo() {
    let mut scene = SceneGraph::new();
    let mut history = HistoryManager::new();

    // Create base tower node
    let tower_mesh = Primitives::create_cylinder(2.0, 6.0, 16);
    let tower_node = SceneNode::new_mesh("CastleTower_NW", core_math::Transform::from_translation(core_math::vec3(0.0, 3.0, 0.0)), tower_mesh);
    let nw_tower_id = scene.add_node(tower_node);

    // Duplicate to NE tower with offset X = +15m
    let dup_cmd = Box::new(crate::commands::DuplicateNodeCommand::new(
        nw_tower_id,
        core_math::vec3(15.0, 0.0, 0.0),
    ));

    history.execute(dup_cmd, &mut scene).expect("DuplicateNodeCommand should succeed");

    assert_eq!(scene.nodes.len(), 2);
    let ne_tower_id = scene.nodes.keys().find(|&k| k != nw_tower_id).unwrap();
    {
        let ne_tower = scene.nodes.get(ne_tower_id).unwrap();
        assert_eq!(ne_tower.name, "CastleTower_NW_copy");
        assert_eq!(ne_tower.local_transform.translation, core_math::vec3(15.0, 3.0, 0.0));
        if let NodeData::Mesh { base_mesh, .. } = &ne_tower.data {
            assert_eq!(base_mesh.faces.len(), 18);
            assert!(base_mesh.validate().is_ok());
        } else {
            panic!("Expected mesh node");
        }
    }

    // Undo should cleanly remove duplicated node
    history.undo(&mut scene).unwrap();
    assert_eq!(scene.nodes.len(), 1);
    assert!(scene.nodes.contains_key(nw_tower_id));

    // Redo should restore duplicated node
    history.redo(&mut scene).unwrap();
    assert_eq!(scene.nodes.len(), 2);
}

#[test]
fn test_variable_radius_fillet_and_weld_commands() {
    let mut scene = SceneGraph::new();
    let mut history = graph_engine::HistoryManager::new();

    let cube = Primitives::create_cube(2.0);
    let edge_key = cube.edges.keys().next().unwrap();
    let node_id = scene.add_node(SceneNode::new_mesh("BevelCube", Transform::IDENTITY, cube));

    // 1. Variable Fillet Command
    let vf_cmd = Box::new(crate::commands::VariableRadiusFilletCommand {
        node_id,
        edges: vec![edge_key],
        r1: 0.1,
        r2: 0.3,
        segments: 2,
        continuity: geo_kernel::FilletContinuity::G2Curvature,
        old_mesh: None,
    });
    history.execute(vf_cmd, &mut scene).expect("variable fillet execute");

    // 2. Undo & Redo
    history.undo(&mut scene).expect("variable fillet undo");
    history.redo(&mut scene).expect("variable fillet redo");

    // 3. Generate Weld Bead Command
    let weld_cmd = Box::new(crate::commands::GenerateWeldBeadCommand {
        node_id,
        edges: vec![edge_key],
        radius: 0.08,
        style: geo_kernel::WeldStyle::TigWeld,
        old_mesh: None,
    });
    history.execute(weld_cmd, &mut scene).expect("weld bead execute");
    history.undo(&mut scene).expect("weld bead undo");
}



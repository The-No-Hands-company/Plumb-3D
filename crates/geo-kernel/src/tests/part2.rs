#[allow(unused_imports)]
use super::*;

    #[test]
    fn test_vertex_group_masked_displacement() {
        let mut plane = Primitives::create_plane(2.0, 2.0, 2, 2);
        let center_vk = plane.vertices.keys().next().unwrap();

        // Assign weight 1.0 to center_vk in group "Mask", all other vertices have 0.0 default weight
        plane.set_vertex_weight("Mask", center_vk, 1.0);
        assert_eq!(plane.get_vertex_weight("Mask", center_vk), 1.0);

        let displace = DisplaceModifier::new(2.0, 1.0).with_vertex_group("Mask");
        let displaced = displace.apply(&plane).unwrap();

        for (vk, v) in &displaced.vertices {
            let orig_pos = plane.vertices[vk].position;
            if vk == center_vk {
                // The masked vertex should have moved
                let diff = (v.position - orig_pos).length();
                assert!(diff > 0.0);
            } else {
                // Unweighted vertices should remain unchanged
                let diff = (v.position - orig_pos).length();
                assert!(diff < 1e-5);
            }
        }
    }

    #[test]
    fn test_weld_vertices_by_distance_and_modifier() {
        let mut mesh = Mesh::new();
        // Create two disconnected quads right next to each other sharing two coincident vertices along X=0
        let v0 = mesh.add_vertex(Vec3::new(-1.0, 0.0, 0.0));
        let v1 = mesh.add_vertex(Vec3::new(0.0, 0.0, 0.0));
        let v2 = mesh.add_vertex(Vec3::new(0.0, 1.0, 0.0));
        let v3 = mesh.add_vertex(Vec3::new(-1.0, 1.0, 0.0));
        mesh.add_face(&[v0, v1, v2, v3]).unwrap();

        // Duplicate coincident vertices at (0, 0, 0) and (0, 1, 0) with a tiny 0.01 offset
        let v4 = mesh.add_vertex(Vec3::new(0.005, 0.0, 0.0));
        let v5 = mesh.add_vertex(Vec3::new(1.0, 0.0, 0.0));
        let v6 = mesh.add_vertex(Vec3::new(1.0, 1.0, 0.0));
        let v7 = mesh.add_vertex(Vec3::new(0.005, 1.0, 0.0));
        mesh.add_face(&[v4, v5, v6, v7]).unwrap();

        assert_eq!(mesh.vertex_count(), 8);

        let weld_mod = WeldModifier::new(0.05);
        let welded = weld_mod.apply(&mesh).unwrap();

        // The 2 duplicate pairs should have merged into 2 single vertices, resulting in 6 vertices total
        assert_eq!(welded.vertex_count(), 6);
        assert_eq!(welded.face_count(), 2);
        assert!(welded.validate().is_ok());
    }

    #[test]
    fn test_bridge_edge_loops() {
        let mut mesh = Mesh::new();
        // Loop A: Triangle at Y = 0
        let a0 = mesh.add_vertex(Vec3::new(0.0, 0.0, 1.0));
        let a1 = mesh.add_vertex(Vec3::new(1.0, 0.0, -0.5));
        let a2 = mesh.add_vertex(Vec3::new(-1.0, 0.0, -0.5));

        // Loop B: Triangle at Y = 2
        let b0 = mesh.add_vertex(Vec3::new(0.0, 2.0, 1.0));
        let b1 = mesh.add_vertex(Vec3::new(1.0, 2.0, -0.5));
        let b2 = mesh.add_vertex(Vec3::new(-1.0, 2.0, -0.5));

        let created_quads = mesh.bridge_edge_loops(&[a0, a1, a2], &[b0, b1, b2]).unwrap();
        assert_eq!(created_quads.len(), 3);
        assert_eq!(mesh.face_count(), 3);
        assert!(mesh.validate().is_ok());
    }

    #[test]
    fn test_skin_modifier_linear_blend_skinning() {
        use core_math::Mat4;
        let mut mesh = Primitives::create_cube(2.0);

        // Create two bone vertex groups: "Root" and "Tip"
        let root_idx = mesh.add_vertex_group("Root");
        let tip_idx = mesh.add_vertex_group("Tip");

        // Vertices with Y <= 0 weighted 1.0 to Root, vertices with Y > 0 weighted 1.0 to Tip
        for (vk, v) in &mesh.vertices {
            if v.position.y <= 0.0 {
                mesh.vertex_groups[root_idx].set_weight(vk, 1.0);
            } else {
                mesh.vertex_groups[tip_idx].set_weight(vk, 1.0);
            }
        }

        // Bone 1: Root stays at origin
        let bone_root = SkinBone::new("Root", Mat4::IDENTITY, Mat4::IDENTITY);
        // Bone 2: Tip translated by +5 along Y
        let tip_world = Mat4::from_translation(Vec3::new(0.0, 5.0, 0.0));
        let bone_tip = SkinBone::new("Tip", Mat4::IDENTITY, tip_world);

        let skin_mod = SkinModifier::new(vec![bone_root, bone_tip]);
        let skinned = skin_mod.apply(&mesh).unwrap();

        // Verify root vertices didn't move, while tip vertices moved up by 5 units
        for (vk, v) in &skinned.vertices {
            let orig = mesh.vertices[vk].position;
            if orig.y <= 0.0 {
                assert!((v.position - orig).length() < 1e-4);
            } else {
                assert!((v.position.y - (orig.y + 5.0)).abs() < 1e-4);
            }
        }
    }

    #[test]
    fn test_apply_weight_brush() {
        let mut plane = Primitives::create_plane(4.0, 4.0, 4, 4);
        let center_pos = Vec3::ZERO;

        let affected = plane.apply_weight_brush("TestGroup", center_pos, 0.8, 1.5, 1.0, BrushFalloff::Smooth);
        assert!(affected > 0);

        // Center vertex should now have weight close to 0.8
        for (_, v) in &plane.vertices {
            if v.position.length() < 0.2 {
                let w = plane.get_vertex_weight("TestGroup", plane.vertices.iter().find(|(_, vt)| vt.position == v.position).unwrap().0);
                assert!((w - 0.8).abs() < 0.1);
            }
        }
    }

    #[test]
    fn test_lattice_and_curve_modifiers() {
        let cube = Primitives::create_cube(2.0);

        // 1. Test Lattice (FFD) Modifier
        let mut lattice = LatticeModifier::from_mesh_bounds(&cube, (3, 3, 3));
        // Displace top-center control point upwards
        lattice.displace_control_point(1, 2, 1, vec3(0.0, 1.5, 0.0));
        let deformed = lattice.apply(&cube).unwrap();
        assert_eq!(deformed.vertex_count(), cube.vertex_count());
        assert!(deformed.validate().is_ok());

        // Test volume-preserving squash and stretch
        let mut ffd_squash = LatticeModifier::from_mesh_bounds(&cube, (2, 2, 2));
        ffd_squash.squash_and_stretch(1.5);
        let squashed = ffd_squash.apply(&cube).unwrap();
        assert!(squashed.validate().is_ok());

        // 2. Test Curve (Spline Deform) Modifier
        let mut spline = Spline::new();
        spline.add_point(BezierPoint::sharp(vec3(0.0, 0.0, 0.0)));
        spline.add_point(BezierPoint::sharp(vec3(0.0, 5.0, 3.0)));

        let curve_mod = CurveModifier::new(spline);
        let bent = curve_mod.apply(&cube).unwrap();
        assert_eq!(bent.vertex_count(), cube.vertex_count());
        assert!(bent.validate().is_ok());
    }

    #[test]
    fn test_topological_selection_operators() {
        let plane = Primitives::create_plane(4.0, 4.0, 4, 4);

        // Edge loop selection on quad grid plane
        let first_edge = plane.edges.keys().next().unwrap();
        let loop_edges = plane.select_edge_loop(first_edge);
        assert!(!loop_edges.is_empty());

        let ring_edges = plane.select_edge_ring(first_edge);
        assert!(!ring_edges.is_empty());

        // Cylinder edge loop and edge ring verification
        let cyl = Primitives::create_cylinder(1.0, 2.0, 16);
        let cyl_edge = cyl.edges.keys().next().unwrap();
        let cyl_loop = cyl.select_edge_loop(cyl_edge);
        assert!(!cyl_loop.is_empty());
        let cyl_ring = cyl.select_edge_ring(cyl_edge);
        assert!(!cyl_ring.is_empty());

        // Selection growing and shrinking - Vertices
        let first_v = plane.vertices.keys().next().unwrap();
        let mut initial_verts = std::collections::HashSet::new();
        initial_verts.insert(first_v);

        let grown_verts = plane.grow_vertex_selection(&initial_verts);
        assert!(grown_verts.len() > initial_verts.len());

        let shrunk_verts = plane.shrink_vertex_selection(&grown_verts);
        assert!(shrunk_verts.len() < grown_verts.len());

        // Selection growing and shrinking - Edges
        let mut initial_edges = std::collections::HashSet::new();
        initial_edges.insert(first_edge);
        let grown_edges = plane.grow_edge_selection(&initial_edges);
        assert!(grown_edges.len() > initial_edges.len());
        let shrunk_edges = plane.shrink_edge_selection(&grown_edges);
        assert!(shrunk_edges.len() < grown_edges.len());

        // Face selection growing and shrinking
        let first_f = plane.faces.keys().next().unwrap();
        let mut initial_faces = std::collections::HashSet::new();
        initial_faces.insert(first_f);

        let grown_faces = plane.grow_face_selection(&initial_faces);
        assert!(grown_faces.len() > initial_faces.len());
        let shrunk_faces = plane.shrink_face_selection(&grown_faces);
        assert!(shrunk_faces.len() < grown_faces.len());

        // Coplanar face selection on plane (all faces share Y normal)
        let coplanar = plane.select_coplanar_faces(first_f, 1.0);
        assert_eq!(coplanar.len(), plane.faces.len());

        // Manifold report on plane vs cube
        let plane_report = plane.check_manifold();
        assert!(!plane_report.is_watertight);
        assert!(plane_report.boundary_edge_count > 0);

        let cube = Primitives::create_cube(2.0);
        let cube_report = cube.check_manifold();
        assert!(cube_report.is_watertight);
        assert!(cube_report.is_2_manifold);
        assert_eq!(cube_report.boundary_edge_count, 0);
        assert_eq!(cube_report.euler_characteristic, 2);
    }

    #[test]
    fn test_auto_smooth_normals_and_split_corners() {
        let mut cube = Primitives::create_cube(2.0);

        // Before auto smooth, no edges are marked sharp
        assert!(cube.edges.values().all(|e| !e.is_sharp));

        // Auto smooth with threshold 45 degrees (pi/4 radians)
        // All cube faces meet at 90 degrees (pi/2 radians > pi/4), so all 12 edges should be marked sharp!
        cube.auto_smooth_normals(std::f32::consts::FRAC_PI_4);
        assert!(cube.edges.values().all(|e| e.is_sharp));

        // When tessellating cube with auto-smooth sharp edges, corner normals should match face normals exactly
        let render_mesh = cube.to_render_mesh(ShadingMode::Smooth);
        assert_eq!(render_mesh.vertices.len(), 24); // 6 faces * 4 vertices = 24 split corner vertices!

        // Cylinder auto smooth: mantle edges (< 45 deg) stay smooth, cap rim edges (90 deg) become sharp
        let mut cylinder = Primitives::create_cylinder(1.0, 2.0, 16);
        cylinder.auto_smooth_normals(std::f32::consts::FRAC_PI_4);
        let sharp_count = cylinder.edges.values().filter(|e| e.is_sharp).count();
        assert!(sharp_count > 0);
        let smooth_render = cylinder.to_render_mesh(ShadingMode::Smooth);
        assert!(smooth_render.vertices.len() > 0);
    }

    #[test]
    fn test_catmull_clark_semi_sharp_creasing() {
        let mut cube = Primitives::create_cube(2.0);

        // Mark all edges fully creased (weight = 1.0)
        for ek in cube.edges.keys().collect::<Vec<_>>() {
            cube.set_edge_crease(ek, 1.0);
        }

        let modifier = CatmullClarkModifier::new(1);
        let subdivided = modifier.apply(&cube).unwrap();

        // With crease_weight = 1.0, all original cube vertices must retain their exact position [-1, 1]!
        for v in subdivided.vertices.values() {
            // All coordinates must be within the original cube bounds [-1.0, 1.0]
            assert!(v.position.x >= -1.001 && v.position.x <= 1.001);
            assert!(v.position.y >= -1.001 && v.position.y <= 1.001);
            assert!(v.position.z >= -1.001 && v.position.z <= 1.001);
        }

        // Subdivided mesh has 24 faces and valid 2-manifold topology
        assert_eq!(subdivided.face_count(), 24);
        assert!(subdivided.validate().is_ok());
    }

    #[test]
    fn test_solidify_modifier_plane() {
        // A single quad plane has 1 face, 4 vertices, and 4 boundary half-edges.
        let plane = Primitives::create_plane(2.0, 2.0, 1, 1);
        assert_eq!(plane.face_count(), 1);
        assert_eq!(plane.vertex_count(), 4);

        let solidify = SolidifyModifier::new(0.5);
        let solidified = solidify.apply(&plane).unwrap();

        // Solidifying a single quad produces 6 faces (1 front + 1 back + 4 rims) and 8 vertices!
        assert_eq!(solidified.face_count(), 6);
        assert_eq!(solidified.vertex_count(), 8);

        // Resulting mesh is a closed 2-manifold box!
        assert!(solidified.validate().is_ok());
        assert_eq!(solidified.euler_characteristic(), 2);
    }

    #[test]
    fn test_array_modifier_cube() {
        let cube = Primitives::create_cube(2.0);
        let mut array_mod = ArrayModifier::new(3);
        array_mod.relative_offset = Vec3::new(1.5, 0.0, 0.0);

        let arrayed = array_mod.apply(&cube).unwrap();

        // 3 cubes: 3 * 8 = 24 vertices, 3 * 6 = 18 faces
        assert_eq!(arrayed.face_count(), 18);
        assert_eq!(arrayed.vertex_count(), 24);
        assert!(arrayed.validate().is_ok());
    }

    #[test]
    fn test_radial_array_modifier_castle_merlons() {
        // Create 1 merlon block at radius X = 2.0m on top rim of tower
        let mut merlon = Primitives::create_cube(0.5);
        for v in merlon.vertices.values_mut() {
            v.position.x += 2.0; // Place at tower radius
            v.position.y += 3.0; // Tower rim height
        }
        merlon.update_normals();

        // Radial array with 8 copies over 360 degrees
        let radial_mod = ArrayModifier::new_radial(8, 360.0);
        let battlements = radial_mod.apply(&merlon).expect("Radial array should succeed");

        // 8 copies * 6 faces = 48 faces, 8 * 8 verts = 64 verts
        assert_eq!(battlements.face_count(), 48);
        assert_eq!(battlements.vertex_count(), 64);
        assert!(battlements.validate().is_ok());

        // Check radial distribution: bounds should be symmetric along X and Z
        let aabb = battlements.compute_aabb();
        assert!((aabb.min.x.abs() - aabb.max.x.abs()).abs() < 1e-3);
        assert!((aabb.min.z.abs() - aabb.max.z.abs()).abs() < 1e-3);
        assert!((aabb.max.x - 2.25).abs() < 1e-3);
    }

    #[test]
    fn test_smooth_modifier_and_shrinkwrap() {
        let cube = Primitives::create_cube(2.0);
        let smooth = SmoothModifier::new(0.5, 3);
        let smoothed = smooth.apply(&cube).unwrap();

        assert_eq!(smoothed.vertex_count(), cube.vertex_count());
        assert_eq!(smoothed.face_count(), cube.face_count());
        assert!(smoothed.validate().is_ok());

        // Shrinkwrap a larger plane onto the cube
        let plane = Primitives::create_plane(4.0, 4.0, 4, 4);
        let shrinkwrap = ShrinkwrapModifier::new()
            .with_target(cube.clone())
            .with_offset(0.1);
        let wrapped = shrinkwrap.apply(&plane).unwrap();

        assert_eq!(wrapped.vertex_count(), plane.vertex_count());
        assert!(wrapped.validate().is_ok());
    }

    #[test]
    fn test_poke_face_and_dissolve_edge() {
        let mut plane = Primitives::create_plane(2.0, 2.0, 1, 1);
        assert_eq!(plane.face_count(), 1);
        assert_eq!(plane.vertex_count(), 4);

        // Poke the quad face: splits 1 quad into 4 triangles and adds 1 centroid vertex
        let fk = plane.faces.keys().next().unwrap();
        let tris = plane.poke_face(fk, 0.5).unwrap();
        assert_eq!(tris.len(), 4);
        assert_eq!(plane.face_count(), 4);
        assert_eq!(plane.vertex_count(), 5);
        assert!(plane.validate().is_ok());

        // Dissolve an internal edge between two triangles
        let internal_edge = plane.edges.iter().find_map(|(ek, _)| {
            let edge_faces = plane.edge_faces(ek);
            if edge_faces.len() == 2 {
                Some(ek)
            } else {
                None
            }
        }).unwrap();

        let merged_fk = plane.dissolve_edge(internal_edge).unwrap();
        assert!(plane.faces.contains_key(merged_fk));
        assert_eq!(plane.face_count(), 3); // 4 - 2 + 1 = 3
        if let Err(e) = plane.validate() {
            panic!("Validation error: {:?}", e);
        }
    }

    #[test]
    fn test_join_and_separate_faces() {
        let mut cube = Primitives::create_cube(2.0);
        assert_eq!(cube.face_count(), 6);

        let faces_to_extract: Vec<FaceKey> = cube.faces.keys().take(2).collect();
        let extracted = cube.separate_faces(&faces_to_extract).unwrap();

        assert_eq!(extracted.face_count(), 2);
        assert_eq!(cube.face_count(), 4);
        assert!(extracted.validate().is_ok());
        assert!(cube.validate().is_ok());

        // Join extracted back into cube
        cube.join(&extracted, core_math::Mat4::IDENTITY).unwrap();
        assert_eq!(cube.face_count(), 6);
        assert!(cube.validate().is_ok());
    }

    #[test]
    fn test_simple_deform_and_screw_modifiers() {
        let cube = Primitives::create_cube(2.0);

        // Twist test
        let twist = SimpleDeformModifier::new(SimpleDeformMethod::Twist, std::f32::consts::FRAC_PI_2);
        let twisted = twist.apply(&cube).unwrap();
        assert_eq!(twisted.vertex_count(), 8);
        assert!(twisted.validate().is_ok());

        // Bend test on a cylinder
        let cyl = Primitives::create_cylinder(0.5, 3.0, 12);
        let bend = SimpleDeformModifier::new(SimpleDeformMethod::Bend, std::f32::consts::PI);
        let bent = bend.apply(&cyl).unwrap();
        assert_eq!(bent.vertex_count(), cyl.vertex_count());
        assert!(bent.validate().is_ok());

        // Stretch test
        let stretch = SimpleDeformModifier::new(SimpleDeformMethod::Stretch, 0.5);
        let stretched = stretch.apply(&cube).unwrap();
        assert_eq!(stretched.vertex_count(), 8);
        assert!(stretched.validate().is_ok());

        // Screw modifier on a plane
        let plane = Primitives::create_plane(1.0, 1.0, 1, 1);
        let screw = ScrewModifier::new(2.0, 2);
        let screwed = screw.apply(&plane).unwrap();
        assert!(screwed.face_count() > 0);
        assert!(screwed.validate().is_ok());
    }

    #[test]
    fn test_flip_normals_and_bounding_dimensions() {
        let mut cube = Primitives::create_cube(2.0);
        let dims = cube.bounding_dimensions();
        assert!((dims.x - 2.0).abs() < 1e-4);
        assert!((dims.y - 2.0).abs() < 1e-4);
        assert!((dims.z - 2.0).abs() < 1e-4);

        let first_fk = cube.faces.keys().next().unwrap();
        let orig_normal = cube.faces[first_fk].normal;

        // Flipping all faces reverses the orientation of the closed manifold
        cube.flip_face_normals(&[]).unwrap();
        let flipped_normal = cube.faces[first_fk].normal;
        assert!((flipped_normal + orig_normal).length() < 1e-4);
        assert!(cube.validate().is_ok());

        // A single face on a plane (open boundary) can be flipped safely
        let mut plane = Primitives::create_plane(2.0, 2.0, 1, 1);
        let plane_fk = plane.faces.keys().next().unwrap();
        let plane_norm = plane.faces[plane_fk].normal;
        plane.flip_face_normals(&[plane_fk]).unwrap();
        let plane_flipped_norm = plane.faces[plane_fk].normal;
        assert!((plane_flipped_norm + plane_norm).length() < 1e-4);
        assert!(plane.validate().is_ok());
    }

    #[test]
    fn test_bisect_plane() {
        // 1. Cut plane without clearing
        let mut cube = Primitives::create_cube(2.0);
        cube.bisect(Vec3::ZERO, Vec3::new(0.0, 1.0, 0.0), false, false, false).unwrap();
        assert_eq!(cube.faces.len(), 10);
        assert_eq!(cube.vertices.len(), 12);
        assert!(cube.validate().is_ok());

        // 2. Cut plane with clear positive and cap
        let mut cube_capped = Primitives::create_cube(2.0);
        cube_capped.bisect(Vec3::ZERO, Vec3::new(0.0, 1.0, 0.0), true, false, true).unwrap();
        assert_eq!(cube_capped.faces.len(), 6);
        for v in cube_capped.vertices.values() {
            assert!(v.position.y <= 1e-4);
        }
        assert!(cube_capped.validate().is_ok());
    }

    #[test]
    fn test_triangulate_and_wireframe_modifiers() {
        // 1. Direct triangulation
        let mut cube = Primitives::create_cube(2.0);
        let created = cube.triangulate_faces(&[], QuadMethod::Beauty, NgonMethod::EarClipping, 4).unwrap();
        assert_eq!(created.len(), 12);
        assert_eq!(cube.faces.len(), 12);
        for fk in cube.faces.keys() {
            assert_eq!(cube.face_vertices(fk).len(), 3);
        }
        assert!(cube.validate().is_ok());

        // 2. Triangulate Modifier
        let cube2 = Primitives::create_cube(2.0);
        let tri_mod = TriangulateModifier::new();
        let tri_mesh = tri_mod.apply(&cube2).unwrap();
        assert_eq!(tri_mesh.faces.len(), 12);
        assert!(tri_mesh.validate().is_ok());

        // 3. Wireframe Modifier
        let cube3 = Primitives::create_cube(2.0);
        let wire_mod = WireframeModifier::new(0.1);
        let wire_mesh = wire_mod.apply(&cube3).unwrap();
        // 6 faces * 4 quad struts = 24 faces
        assert_eq!(wire_mesh.faces.len(), 24);
        assert!(wire_mesh.validate().is_ok());
    }


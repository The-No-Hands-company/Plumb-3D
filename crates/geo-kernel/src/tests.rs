    use super::*;
    use core_math::{vec3, Transform, Vec3};

    #[test]
    fn test_cube_topology_and_euler_characteristic() {
        let cube = Primitives::create_cube(2.0);
        assert_eq!(cube.vertex_count(), 8);
        assert_eq!(cube.edge_count(), 12);
        assert_eq!(cube.face_count(), 6);
        assert_eq!(cube.half_edge_count(), 24);

        // Topological invariants must pass
        assert!(cube.validate().is_ok());

        // Euler characteristic chi = V - E + F = 8 - 12 + 6 = 2
        assert_eq!(cube.euler_characteristic(), 2);
    }

    #[test]
    fn test_plane_primitive() {
        let plane = Primitives::create_plane(10.0, 10.0, 4, 4);
        assert_eq!(plane.face_count(), 16); // 4x4 quads
        assert_eq!(plane.vertex_count(), 25); // 5x5 vertices
        assert!(plane.validate().is_ok());
    }

    #[test]
    fn test_uv_sphere_primitive() {
        let sphere = Primitives::create_uv_sphere(1.0, 8, 8);
        assert!(sphere.validate().is_ok());
        assert_eq!(sphere.euler_characteristic(), 2);
    }

    #[test]
    fn test_cylinder_primitive() {
        let cylinder = Primitives::create_cylinder(1.0, 2.0, 8);
        assert!(cylinder.validate().is_ok());
        assert_eq!(cylinder.euler_characteristic(), 2);
    }

    #[test]
    fn test_subdivision_modifier() {
        let cube = Primitives::create_cube(1.0);
        let sub_mod = SubdivideModifier::new(1);
        let subdivided = sub_mod.apply(&cube).expect("Subdivision should succeed");

        // After 1 subdivision of a cube:
        // Faces: 6 quads * 4 = 24 quads
        // Vertices: 8 original + 12 edge midpoints + 6 face centers = 26
        // Edges: 12 * 2 + 6 * 4 = 48
        assert_eq!(subdivided.face_count(), 24);
        assert_eq!(subdivided.vertex_count(), 26);
        assert_eq!(subdivided.edge_count(), 48);

        assert!(subdivided.validate().is_ok());
        assert_eq!(subdivided.euler_characteristic(), 2);
    }

    #[test]
    fn test_modifier_stack() {
        let base_cube = Primitives::create_cube(1.0);
        let mut stack = ModifierStack::new();

        stack.add_modifier(Box::new(TransformModifier::new(Transform::from_translation(vec3(0.0, 5.0, 0.0)))));
        stack.add_modifier(Box::new(SubdivideModifier::new(1)));

        let result = stack.evaluate(&base_cube).unwrap();
        assert_eq!(result.face_count(), 24);
        assert!(result.vertices.values().all(|v| v.position.y >= 4.0 && v.position.y <= 6.0));
    }

    #[test]
    fn test_triangulation_render_mesh() {
        let cube = Primitives::create_cube(1.0);
        let flat_render = cube.to_render_mesh(ShadingMode::Flat);
        // 6 faces * 2 triangles per quad * 3 indices = 36 indices
        assert_eq!(flat_render.indices.len(), 36);
        // Wireframe edges: 12 edges * 2 indices = 24 indices
        assert_eq!(flat_render.wireframe_indices.len(), 24);

        let smooth_render = cube.to_render_mesh(ShadingMode::Smooth);
        assert_eq!(smooth_render.indices.len(), 36);
    }

    #[test]
    fn test_raycast_cube() {
        use core_math::{Mat4, Ray};

        let cube = Primitives::create_cube(2.0);
        let ray = Ray::new(vec3(0.0, 0.0, 5.0), vec3(0.0, 0.0, -1.0));
        let hit = cube.raycast(&ray, &Mat4::IDENTITY);
        assert!(hit.is_some());
        let h = hit.unwrap();
        assert!((h.point.z - 1.0).abs() < 1e-4);
        assert!((h.distance - 4.0).abs() < 1e-4);
    }

    #[test]
    fn test_face_extrusion_topology() {
        let mut cube = Primitives::create_cube(2.0);
        let initial_faces = cube.face_count();
        let target_face = cube.faces.keys().next().unwrap();

        let top_face = cube.extrude_face(target_face, 1.5).expect("Extrusion should succeed");
        // Extruding 1 quad face adds 4 side quads, net +4 faces: 6 + 4 = 10 faces
        assert_eq!(cube.face_count(), initial_faces + 4);
        // Euler characteristic must be strictly preserved: chi = 2
        assert_eq!(cube.euler_characteristic(), 2);
        assert!(cube.validate().is_ok());
        assert!(cube.faces.contains_key(top_face));

        // Test multi-face contiguous region extrusion on a plane
        let mut plane = Primitives::create_plane(4.0, 4.0, 3, 3);
        let f_keys: Vec<_> = plane.faces.keys().take(2).collect();
        let region_tops = plane.extrude_faces(&f_keys, 1.0, false, None).expect("Region extrude should succeed");
        assert_eq!(region_tops.len(), 2);
        assert!(plane.validate().is_ok());
    }

    #[test]
    fn test_face_bevel_topology() {
        let mut cube = Primitives::create_cube(2.0);
        let target_face = cube.faces.keys().next().unwrap();

        let beveled_face = cube.bevel_face(target_face, 0.2, 0.2).expect("Bevel should succeed");
        assert_eq!(cube.face_count(), 10);
        assert_eq!(cube.euler_characteristic(), 2);
        assert!(cube.validate().is_ok());
        assert!(cube.faces.contains_key(beveled_face));
    }

    #[test]
    fn test_displace_and_taper_modifiers() {
        let cube = Primitives::create_cube(2.0);

        let displace = DisplaceModifier::new(0.5, 2.0);
        let displaced = displace.apply(&cube).expect("Displacement should succeed");
        assert_eq!(displaced.vertex_count(), 8);
        assert_eq!(displaced.face_count(), 6);

        let taper = TaperModifier::new(0.5);
        let tapered = taper.apply(&cube).expect("Taper should succeed");
        assert_eq!(tapered.vertex_count(), 8);
        assert_eq!(tapered.face_count(), 6);
    }

    #[test]
    fn test_split_quad_face() {
        let mut cube = Primitives::create_cube(2.0);
        let target_face = cube.faces.keys().next().unwrap();
        let (f_a, f_b) = cube.split_quad_face(target_face, 0, 0.5).expect("Split quad should succeed");

        assert_eq!(cube.face_count(), 7); // 6 - 1 + 2 = 7 faces
        assert_eq!(cube.vertex_count(), 10); // 8 + 2 new midpoints = 10 vertices
        assert!(cube.faces.contains_key(f_a));
        assert!(cube.faces.contains_key(f_b));
        assert!(cube.validate().is_ok());
    }

    #[test]
    fn test_point_in_mesh_and_boolean() {
        let cube_a = Primitives::create_cube(2.0); // [-1, 1] on all axes
        assert!(cube_a.is_point_inside(vec3(0.0, 0.0, 0.0)));
        assert!(!cube_a.is_point_inside(vec3(3.0, 0.0, 0.0)));

        let cube_b = Primitives::create_cube(1.0); // [-0.5, 0.5] nested entirely inside A
        let union_result = cube_a.boolean(&cube_b, BooleanOp::Union).expect("Boolean Union should succeed");
        // All faces of B are inside A, so Union retains external faces of A
        assert_eq!(union_result.face_count(), 6);

        let diff_result = cube_a.boolean(&cube_b, BooleanOp::Difference).expect("Boolean Difference should succeed");
        // Difference adds the 6 inverted internal cavity faces
        assert_eq!(diff_result.face_count(), 12);
    }

    #[test]
    fn test_hard_surface_boolean_intersecting_cut() {
        let cube_a = Primitives::create_cube(2.0); // [-1, 1]
        let mut cutter = Primitives::create_cube(1.0); // [-0.5, 0.5]
        for v in cutter.vertices.values_mut() {
            v.position.x += 0.8;
        }

        let pocket = cube_a.boolean(&cutter, BooleanOp::Difference).expect("Boolean cut should succeed");
        assert!(pocket.face_count() >= 6, "Carved mesh should have faces from cut");
        assert!(pocket.vertex_count() >= 8, "Carved mesh should have vertices");

        let union_res = cube_a.boolean(&cutter, BooleanOp::Union).expect("Boolean union should succeed");
        assert!(union_res.face_count() >= 6, "Union mesh should have faces");
    }

    #[test]
    fn test_mirror_modifier() {
        let cube = Primitives::create_cube(2.0);
        let mirror = MirrorModifier::new(MirrorAxis::X);
        let mirrored = mirror.apply(&cube).expect("Mirror should succeed");
        // Initial 6 faces + 6 mirrored faces = 12 faces
        assert_eq!(mirrored.face_count(), 12);
        assert_eq!(mirrored.vertex_count(), 16);
    }

    #[test]
    fn test_obj_export_import_roundtrip() {
        let original_cube = Primitives::create_cube(2.0);
        let obj_text = original_cube.to_obj("TestCube");
        assert!(obj_text.contains("v -1.000000 -1.000000 -1.000000"));
        assert!(obj_text.contains("f "));

        let imported_mesh = Mesh::from_obj(&obj_text).expect("OBJ Import should succeed");
        assert_eq!(imported_mesh.vertex_count(), 8);
        assert_eq!(imported_mesh.face_count(), 6);
        assert_eq!(imported_mesh.edge_count(), 12);
        assert_eq!(imported_mesh.euler_characteristic(), 2);
    }

    #[test]
    fn test_catmull_clark_subdivision() {
        let cube = Primitives::create_cube(2.0);
        let cc = CatmullClarkModifier::new(1);
        let smoothed = cc.apply(&cube).expect("Catmull-Clark should succeed");

        // 6 quad faces subdivided -> 24 quad faces
        assert_eq!(smoothed.face_count(), 24);
        assert_eq!(smoothed.vertex_count(), 26);
        assert_eq!(smoothed.euler_characteristic(), 2);
        assert!(smoothed.validate().is_ok());

        // Corner vertices should be pulled inward toward the center (smoothing effect)
        let max_dist = smoothed
            .vertices
            .values()
            .map(|v| v.position.length())
            .fold(0.0_f32, f32::max);
        // Original cube corner distance = sqrt(1 + 1 + 1) = 1.732
        // Smoothed corner distance should be significantly less than 1.732
        assert!(max_dist < 1.70);
    }

    #[test]
    fn test_connect_vertices_and_split_edge() {
        let mut plane = Primitives::create_plane(2.0, 2.0, 1, 1);
        assert_eq!(plane.face_count(), 1);
        assert_eq!(plane.vertex_count(), 4);
        assert_eq!(plane.edge_count(), 4);

        let face_key = plane.faces.keys().next().unwrap();
        let verts = plane.face_vertices(face_key);
        assert_eq!(verts.len(), 4);

        // Connect opposite vertices v0 and v2: splits 1 quad into 2 triangles
        let (f_a, f_b) = plane
            .connect_vertices(face_key, verts[0], verts[2])
            .expect("Connect vertices should succeed");
        assert_eq!(plane.face_count(), 2);
        assert_eq!(plane.face_vertices(f_a).len(), 3);
        assert_eq!(plane.face_vertices(f_b).len(), 3);
        assert_eq!(plane.edge_count(), 5);

        // Split one of the edges
        let edge_key = plane.edges.keys().next().unwrap();
        let new_v = plane.split_edge(edge_key, 0.5).expect("Split edge should succeed");
        assert!(plane.vertices.contains_key(new_v));
    }

    #[test]
    fn test_loop_cut_operator() {
        let mut cube = Primitives::create_cube(2.0);
        let first_edge = cube.edges.keys().next().unwrap();
        let preview = cube.get_loop_cut_preview(first_edge, 0.5);
        assert_eq!(preview.len(), 4);

        let new_faces = cube.loop_cut(first_edge, 0.5).expect("Loop cut should succeed");
        assert!(!new_faces.is_empty());
        assert!(cube.face_count() > 6);
        assert!(cube.validate().is_ok());
    }

    #[test]
    fn test_uv_projections() {
        let mut cube = Primitives::create_cube(2.0);
        cube.generate_planar_uv(ProjectionAxis::Y);
        for v in cube.vertices.values() {
            assert!(v.uv.x >= 0.0 && v.uv.x <= 1.0);
            assert!(v.uv.y >= 0.0 && v.uv.y <= 1.0);
        }

        cube.generate_cylindrical_uv(ProjectionAxis::Y);
        for v in cube.vertices.values() {
            assert!(v.uv.x >= 0.0 && v.uv.x <= 1.0);
            assert!(v.uv.y >= 0.0 && v.uv.y <= 1.0);
        }

        cube.generate_spherical_uv();
        for v in cube.vertices.values() {
            assert!(v.uv.x >= 0.0 && v.uv.x <= 1.0);
            assert!(v.uv.y >= 0.0 && v.uv.y <= 1.0);
        }

        cube.generate_box_uv();
        for v in cube.vertices.values() {
            assert!(v.uv.x >= 0.0 && v.uv.x <= 1.0);
            assert!(v.uv.y >= 0.0 && v.uv.y <= 1.0);
        }
    }

    #[test]
    fn test_gltf_export_and_import() {
        let cube = Primitives::create_cube(2.0);
        let gltf_json = cube.to_gltf("CubeAsset");
        assert!(gltf_json.contains(r#""version": "2.0""#));
        assert!(gltf_json.contains(r#""name": "CubeAsset""#));
        assert!(gltf_json.contains(r#""POSITION""#));
        assert!(gltf_json.contains(r#""NORMAL""#));
        assert!(gltf_json.contains(r#""TEXCOORD_0""#));
        assert!(gltf_json.contains("data:application/octet-stream;base64,"));

        let imported = Mesh::from_gltf(&gltf_json).expect("glTF import should succeed");
        // Smooth Cube: 12 triangle faces sharing 8 vertices
        assert_eq!(imported.face_count(), 12);
        assert_eq!(imported.vertex_count(), 8);
    }

    #[test]
    fn test_mesh_decimation() {
        let sphere = Primitives::create_uv_sphere(1.0, 16, 16);
        let orig_faces = sphere.face_count();
        let orig_verts = sphere.vertex_count();

        // Decimate to 50%
        let decimated = sphere.decimate(0.5).expect("Decimation should succeed");
        assert!(decimated.face_count() < orig_faces);
        assert!(decimated.vertex_count() < orig_verts);
        assert!(decimated.validate().is_ok());

        // Test DecimateModifier in modifier stack
        let mut stack = ModifierStack::new();
        stack.add_modifier(Box::new(DecimateModifier::new(0.4)));
        let output = stack.evaluate(&sphere).expect("Modifier evaluation should succeed");
        assert!(output.face_count() < orig_faces);
        assert!(output.validate().is_ok());
    }

    #[test]
    fn test_torus_and_cone_primitives() {
        let torus = Primitives::create_torus(2.0, 0.5, 12, 8);
        assert_eq!(torus.face_count(), 12 * 8); // 96 quads
        assert_eq!(torus.vertex_count(), 12 * 8); // 96 vertices
        assert!(torus.validate().is_ok());
        // Torus genus 1: chi = V - E + F = 96 - 192 + 96 = 0
        assert_eq!(torus.euler_characteristic(), 0);

        let cone = Primitives::create_cone(1.5, 3.0, 10);
        assert_eq!(cone.face_count(), 10 + 1); // 10 side triangles + 1 bottom n-gon cap
        assert_eq!(cone.vertex_count(), 1 + 10); // apex + 10 base vertices
        assert!(cone.validate().is_ok());
        assert_eq!(cone.euler_characteristic(), 2);

        // Pyramid (5 vertices, 8 edges, 5 faces -> chi = 2)
        let pyramid = Primitives::create_pyramid(2.0, 3.0);
        assert_eq!(pyramid.vertex_count(), 5);
        assert_eq!(pyramid.edge_count(), 8);
        assert_eq!(pyramid.face_count(), 5);
        assert!(pyramid.validate().is_ok());
        assert_eq!(pyramid.euler_characteristic(), 2);

        // Capsule (chi = 2)
        let capsule = Primitives::create_capsule(1.0, 2.0, 8, 4);
        assert!(capsule.validate().is_ok());
        assert_eq!(capsule.euler_characteristic(), 2);

        // IcoSphere (chi = 2)
        let icosphere = Primitives::create_icosphere(1.5, 1);
        assert!(icosphere.validate().is_ok());
        assert_eq!(icosphere.euler_characteristic(), 2);
    }

    #[test]
    fn test_spline_lathe_and_sweep() {
        use std::f32::consts::PI;

        // 1. Test Lathe / Revolve
        // Wine glass / Vase profile
        let profile = vec![
            (0.0, -1.0), // Base center (r=0)
            (0.8, -1.0), // Base rim
            (0.15, -0.2), // Stem bottom
            (0.15, 0.5),  // Stem top
            (0.9, 1.2),  // Bowl rim
            (0.0, 1.2),  // Top center (r=0)
        ];
        let vase = Spline::lathe(&profile, 16, 2.0 * PI).expect("Lathe should succeed");
        assert!(vase.face_count() > 0);
        assert!(vase.vertex_count() > 0);
        assert!(vase.validate().is_ok());

        // 2. Test Sweep / Extrude along 3D Path
        let circle = Spline::circle_cross_section(0.2, 8);
        // S-shaped curved 3D path
        let mut spline = Spline::new();
        spline.add_point(BezierPoint::smooth(vec3(0.0, 0.0, 0.0), vec3(0.0, 1.0, 0.0)));
        spline.add_point(BezierPoint::smooth(vec3(1.0, 2.0, 1.0), vec3(1.0, 0.0, 0.0)));
        spline.add_point(BezierPoint::smooth(vec3(3.0, 1.0, -1.0), vec3(0.0, -1.0, 0.0)));
        let path = spline.sample(10);
        assert!(path.len() >= 20);

        let pipe = Spline::sweep(&circle, &path, false).expect("Sweep should succeed");
        assert!(pipe.face_count() > 0);
        assert!(pipe.vertex_count() > 0);
        assert!(pipe.validate().is_ok());
        assert_eq!(pipe.euler_characteristic(), 2);

        // 3. Test procedural Circle and Helix splines
        let circ_spline = Spline::circle(5.0, Vec3::ZERO);
        assert_eq!(circ_spline.points.len(), 4);
        assert!(circ_spline.is_closed);
        let (c_pos, _) = circ_spline.eval(0.0);
        assert!((c_pos.length() - 5.0).abs() < 1e-3);

        let helix_spline = Spline::helix(3.0, 10.0, 2.0, 8);
        assert!(helix_spline.points.len() >= 16);
        let (h_start, _) = helix_spline.eval(0.0);
        let (h_end, _) = helix_spline.eval(1.0);
        assert_eq!(h_start.y, 0.0);
        assert!((h_end.y - 10.0).abs() < 1e-3);
    }

    #[test]
    fn test_sculpt_brushes() {
        let mut plane = Primitives::create_plane(4.0, 4.0, 8, 8);
        let orig_pos: Vec<Vec3> = plane.vertices.values().map(|v| v.position).collect();

        // 1. Test Draw brush (pushes center vertices upward)
        let settings = SculptSettings {
            brush: SculptBrush::Draw,
            radius: 1.0,
            strength: 0.5,
            falloff: BrushFalloff::Smooth,
            invert: false,
        };
        let affected = plane.apply_sculpt_brush(Vec3::ZERO, Vec3::Y, &settings, None);
        assert!(affected > 0);
        assert!(plane.validate().is_ok());

        // Center vertex should have moved in +Y
        let mut moved_up = false;
        for (i, v) in plane.vertices.values().enumerate() {
            if (v.position.x.abs() < 0.1) && (v.position.z.abs() < 0.1) {
                if v.position.y > orig_pos[i].y + 0.01 {
                    moved_up = true;
                }
            }
        }
        assert!(moved_up);

        // 2. Test Smooth brush (dampens the deformation back towards neighbors)
        let smooth_settings = SculptSettings {
            brush: SculptBrush::Smooth,
            radius: 1.2,
            strength: 0.8,
            falloff: BrushFalloff::Smooth,
            invert: false,
        };
        let smoothed_count = plane.apply_sculpt_brush(Vec3::ZERO, Vec3::Y, &smooth_settings, None);
        assert!(smoothed_count > 0);
        assert!(plane.validate().is_ok());

        // 3. Test Flatten brush (flattens back onto Y=0 plane)
        let flatten_settings = SculptSettings {
            brush: SculptBrush::Flatten,
            radius: 1.5,
            strength: 1.0,
            falloff: BrushFalloff::Linear,
            invert: false,
        };
        plane.apply_sculpt_brush(Vec3::ZERO, Vec3::Y, &flatten_settings, None);
        assert!(plane.validate().is_ok());

        // 4. Test Pinch brush (pulls vertices toward center)
        let pinch_settings = SculptSettings {
            brush: SculptBrush::Pinch,
            radius: 1.5,
            strength: 0.8,
            falloff: BrushFalloff::Smooth,
            invert: false,
        };
        let pinched = plane.apply_sculpt_brush(Vec3::ZERO, Vec3::Y, &pinch_settings, None);
        assert!(pinched > 0);
        assert!(plane.validate().is_ok());

        // 5. Test Crease brush (carves valley and pinches)
        let crease_settings = SculptSettings {
            brush: SculptBrush::Crease,
            radius: 1.0,
            strength: 0.7,
            falloff: BrushFalloff::Sharp,
            invert: false,
        };
        let creased = plane.apply_sculpt_brush(Vec3::ZERO, Vec3::Y, &crease_settings, None);
        assert!(creased > 0);
        assert!(plane.validate().is_ok());

        // 6. Test Snake Hook brush (drags along motion vector)
        let snake_settings = SculptSettings {
            brush: SculptBrush::SnakeHook,
            radius: 1.2,
            strength: 0.9,
            falloff: BrushFalloff::Smooth,
            invert: false,
        };
        let snaked = plane.apply_sculpt_brush(Vec3::ZERO, Vec3::Y, &snake_settings, Some(Vec3::new(0.5, 0.2, 0.0)));
        assert!(snaked > 0);
        assert!(plane.validate().is_ok());

        // 7. Test Clay brush (builds up volume towards offset plane)
        let clay_settings = SculptSettings {
            brush: SculptBrush::Clay,
            radius: 1.5,
            strength: 0.6,
            falloff: BrushFalloff::Smooth,
            invert: false,
        };
        let clayed = plane.apply_sculpt_brush(Vec3::ZERO, Vec3::Y, &clay_settings, None);
        assert!(clayed > 0);
        assert!(plane.validate().is_ok());
    }

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

    #[test]
    fn test_bevel_vertices() {
        let mut cube = Primitives::create_cube(2.0);
        let v0 = cube.vertices.keys().next().unwrap();
        let caps = cube.bevel_vertices(&[v0], 0.3).unwrap();
        assert_eq!(caps.len(), 1);
        assert_eq!(cube.faces.len(), 7); // 6 original (now 3 quads, 3 pentagons) + 1 cap
        assert_eq!(cube.vertices.len(), 10); // 8 - 1 + 3 = 10
        assert!(cube.validate().is_ok());
    }

    #[test]
    fn test_merge_vertices_at_center() {
        let mut plane = Primitives::create_plane(2.0, 2.0, 1, 1);
        assert_eq!(plane.vertices.len(), 4);
        assert_eq!(plane.faces.len(), 1);

        // Merge two vertices of the quad
        let vks: Vec<VertexKey> = plane.vertices.keys().take(2).collect();
        let merged = plane.merge_vertices_at_center(&vks).unwrap();
        assert!(merged.is_some());
        assert_eq!(plane.vertices.len(), 3);
        assert_eq!(plane.faces.len(), 1);
        let fk = plane.faces.keys().next().unwrap();
        assert_eq!(plane.face_vertices(fk).len(), 3);
        assert!(plane.validate().is_ok());
    }

    #[test]
    fn test_slide_edges_and_vertices() {
        let mut plane = Primitives::create_plane(4.0, 4.0, 2, 2);
        assert_eq!(plane.vertices.len(), 9);
        assert_eq!(plane.faces.len(), 4);

        // Slide an edge
        let first_ek = plane.edges.keys().next().unwrap();
        plane.slide_edges(&[first_ek], 0.3).unwrap();
        assert_eq!(plane.vertices.len(), 9);
        assert_eq!(plane.faces.len(), 4);
        assert!(plane.validate().is_ok());

        // Slide a vertex
        let center_vk = plane.vertices.keys().find(|&vk| plane.vertices[vk].position.length() < 0.1).unwrap();
        let orig_pos = plane.vertices[center_vk].position;
        plane.slide_vertices(&[center_vk], 0.5).unwrap();
        let new_pos = plane.vertices[center_vk].position;
        assert!((new_pos - orig_pos).length() > 0.01);
        assert!(plane.validate().is_ok());
    }

    #[test]
    fn test_separate_loose_parts() {
        let mut cube1 = Primitives::create_cube(2.0);
        let mut cube2 = Primitives::create_cube(2.0);
        // Translate cube2 away along X
        for v in cube2.vertices.values_mut() {
            v.position.x += 10.0;
        }

        cube1.join(&cube2, core_math::Mat4::IDENTITY).unwrap();
        assert_eq!(cube1.vertex_count(), 16);
        assert_eq!(cube1.face_count(), 12);

        let parts = cube1.separate_loose_parts().unwrap();
        assert_eq!(parts.len(), 2);
        for part in &parts {
            assert_eq!(part.vertex_count(), 8);
            assert_eq!(part.face_count(), 6);
            assert!(part.validate().is_ok());
        }
    }

    #[test]
    fn test_separate_by_material() {
        let mut cube = Primitives::create_cube(2.0);
        let faces: Vec<FaceKey> = cube.faces.keys().collect();
        // Assign material 1 to the first 2 faces, and material 2 to the remaining 4 faces
        cube.set_face_material(faces[0], 1).unwrap();
        cube.set_face_material(faces[1], 1).unwrap();
        for &fk in &faces[2..] {
            cube.set_face_material(fk, 2).unwrap();
        }

        let mat_parts = cube.separate_by_material().unwrap();
        assert_eq!(mat_parts.len(), 2);
        assert_eq!(mat_parts[0].0, 1);
        assert_eq!(mat_parts[0].1.face_count(), 2);
        assert!(mat_parts[0].1.validate().is_ok());

        assert_eq!(mat_parts[1].0, 2);
        assert_eq!(mat_parts[1].1.face_count(), 4);
        assert!(mat_parts[1].1.validate().is_ok());
    }

    #[test]
    fn test_symmetrize_operator() {
        let mut cube = Primitives::create_cube(2.0);
        // Translate slightly along +X so the cut at X=0 slices cleanly
        for v in cube.vertices.values_mut() {
            v.position.x += 0.5;
        }

        cube.symmetrize(MirrorAxis::X, SymmetrizeDirection::PositiveToNegative).unwrap();
        assert!(cube.validate().is_ok());
        assert!(cube.face_count() >= 6);

        // Verify bounding box symmetry across X
        let (min, max) = cube.bounding_box();
        assert!((min.x.abs() - max.x.abs()).abs() < 0.05);
    }

    #[test]
    fn test_clean_degenerate_geometry() {
        let mut plane = Primitives::create_plane(2.0, 2.0, 1, 1);
        // Add a redundant duplicate vertex near an existing vertex
        let near_pos = plane.vertices.values().next().unwrap().position + Vec3::new(1e-4, 1e-4, 1e-4);
        plane.add_vertex(near_pos);
        assert_eq!(plane.vertex_count(), 5);

        let cleaned = plane.clean_degenerate_geometry(1e-4, 0.01).unwrap();
        assert!(cleaned > 0);
        assert_eq!(plane.vertex_count(), 4);
        assert!(plane.validate().is_ok());
    }

    #[test]
    fn test_spin_revolve_operator() {
        let mut plane = Primitives::create_plane(1.0, 1.0, 1, 1);
        // Translate away from center along X
        for v in plane.vertices.values_mut() {
            v.position.x += 3.0;
        }
        let fk = plane.faces.keys().next().unwrap();

        let new_faces = plane.spin(&[fk], Vec3::Y, Vec3::ZERO, std::f32::consts::TAU, 8).unwrap();
        assert!(!new_faces.is_empty());
        assert!(plane.validate().is_ok());
    }

    #[test]
    fn test_boundary_loops_and_cap_holes() {
        // Delete a face from a cube to create an open hole
        let mut cube = Primitives::create_cube(2.0);
        let top_fk = cube.faces.keys().next().unwrap();
        cube.delete_face(top_fk);

        let loops = cube.boundary_loops();
        assert_eq!(loops.len(), 1);
        assert_eq!(loops[0].len(), 4);

        let caps = cube.cap_holes(false).unwrap();
        assert_eq!(caps.len(), 1);
        assert_eq!(cube.face_count(), 6);
        assert!(cube.validate().is_ok());
    }

    #[test]
    fn test_grid_fill_operator() {
        // Build a square boundary loop of 8 vertices (P=2, Q=2)
        let mut mesh = Mesh::new();
        let v0 = mesh.add_vertex(Vec3::new(0.0, 0.0, 0.0));
        let v1 = mesh.add_vertex(Vec3::new(1.0, 0.0, 0.0));
        let v2 = mesh.add_vertex(Vec3::new(2.0, 0.0, 0.0));
        let v3 = mesh.add_vertex(Vec3::new(2.0, 0.0, 1.0));
        let v4 = mesh.add_vertex(Vec3::new(2.0, 0.0, 2.0));
        let v5 = mesh.add_vertex(Vec3::new(1.0, 0.0, 2.0));
        let v6 = mesh.add_vertex(Vec3::new(0.0, 0.0, 2.0));
        let v7 = mesh.add_vertex(Vec3::new(0.0, 0.0, 1.0));

        let boundary = vec![v0, v1, v2, v3, v4, v5, v6, v7];
        let faces = mesh.grid_fill(&boundary, 2).unwrap();
        assert_eq!(faces.len(), 4);
        assert_eq!(mesh.vertex_count(), 9); // 8 boundary + 1 center interior
        assert!(mesh.validate().is_ok());
    }

    #[test]
    fn test_shortest_path_dijkstra() {
        let plane = Primitives::create_plane(4.0, 4.0, 2, 2);
        let v_keys: Vec<VertexKey> = plane.vertices.keys().collect();
        let start = v_keys[0];
        let end = v_keys[8];

        let path = plane.shortest_path(start, end);
        assert!(path.is_some());
        let p = path.unwrap();
        assert_eq!(p.first().copied(), Some(start));
        assert_eq!(p.last().copied(), Some(end));
        assert!(p.len() >= 3);
    }

    #[test]
    fn test_weighted_normal_modifier() {
        let cylinder = Primitives::create_cylinder(1.0, 2.0, 16);
        let modifier = WeightedNormalModifier::new(WeightedNormalMode::AreaAndAngle);
        let weighted = modifier.apply(&cylinder).unwrap();

        assert_eq!(weighted.vertex_count(), cylinder.vertex_count());
        for v in weighted.vertices.values() {
            assert!(v.normal.length() > 0.99 && v.normal.length() < 1.01);
        }
        assert!(weighted.validate().is_ok());
    }

    #[test]
    fn test_vertex_colors_and_color_brush() {
        let mut plane = Primitives::create_plane(2.0, 2.0, 2, 2);
        // Default color is white [1, 1, 1, 1]
        for v in plane.vertices.values() {
            assert_eq!(v.color, [1.0, 1.0, 1.0, 1.0]);
        }

        // Test fill_vertex_colors
        plane.fill_vertex_colors([0.1, 0.2, 0.3, 1.0]);
        for v in plane.vertices.values() {
            assert_eq!(v.color, [0.1, 0.2, 0.3, 1.0]);
        }

        // Test apply_color_brush at center
        let red = [1.0, 0.0, 0.0, 1.0];
        let affected = plane.apply_color_brush(Vec3::ZERO, red, 0.8, 1.0, crate::sculpt::BrushFalloff::Linear);
        assert!(affected > 0);

        // Center vertex should be closer to red
        let center_vk = plane.vertices.iter().find(|(_, v)| v.position.length() < 0.01).map(|(k, _)| k).unwrap();
        let center_color = plane.get_vertex_color(center_vk);
        assert!(center_color[0] > 0.8);

        // RenderMesh contains the updated colors
        let render_mesh = plane.to_render_mesh(ShadingMode::Smooth);
        assert!(render_mesh.vertices.iter().any(|gv| gv.color[0] > 0.8));
    }

    #[test]
    fn test_extract_curve_and_to_pipe_mesh() {
        let cube = Primitives::create_cube(2.0);
        // Find top 4 edges forming a closed ring on top of the cube (y > 0.99)
        let top_edges: Vec<EdgeKey> = cube.edges
            .iter()
            .filter(|(_, edge)| {
                let he = &cube.half_edges[edge.half_edge];
                cube.vertices[he.origin].position.y > 0.99 && cube.vertices[he.target].position.y > 0.99
            })
            .map(|(k, _)| k)
            .collect();
        assert_eq!(top_edges.len(), 4);

        let spline = cube.extract_curve_from_edges(&top_edges).unwrap();
        assert!(spline.is_closed);
        assert_eq!(spline.points.len(), 4);

        // Convert the spline to a 3D pipe mesh
        let pipe = spline.to_pipe_mesh(0.1, 8, 4).unwrap();
        assert!(pipe.vertex_count() > 0);
        assert!(pipe.face_count() > 0);
        assert!(pipe.validate().is_ok());
    }

    #[test]
    fn test_morpher_modifier_blendshapes() {
        let base_cube = Primitives::create_cube(2.0);
        let mut target_cube = base_cube.clone();

        // Displace top 4 vertices up by +1.5 units on target
        for v in target_cube.vertices.values_mut() {
            if v.position.y > 0.99 {
                v.position.y += 1.5;
            }
        }

        let mut morpher = MorpherModifier::new();
        morpher.add_target_from_mesh("TopStretch", &base_cube, &target_cube).unwrap();

        // At weight 0.0, output matches base_cube
        let eval_0 = morpher.apply(&base_cube).unwrap();
        for (vk, v) in &eval_0.vertices {
            assert!((v.position - base_cube.vertices[vk].position).length() < 1e-5);
        }

        // At weight 1.0, top vertices should be at y = 2.5
        morpher.set_target_weight("TopStretch", 1.0);
        let eval_1 = morpher.apply(&base_cube).unwrap();
        for (vk, v) in &eval_1.vertices {
            if base_cube.vertices[vk].position.y > 0.99 {
                assert!((v.position.y - 2.5).abs() < 1e-4);
            }
        }

        // At weight 0.5, top vertices should be at y = 1.75
        morpher.set_target_weight("TopStretch", 0.5);
        let eval_half = morpher.apply(&base_cube).unwrap();
        for (vk, v) in &eval_half.vertices {
            if base_cube.vertices[vk].position.y > 0.99 {
                assert!((v.position.y - 1.75).abs() < 1e-4);
            }
        }
    }

    #[test]
    fn test_shrink_fatten_and_relax_vertices() {
        let mut sphere = Primitives::create_uv_sphere(1.0, 8, 8);
        let all_verts: Vec<VertexKey> = sphere.vertices.keys().collect();

        // Test Shrink/Fatten (Alt+S)
        let r0 = sphere.vertices.values().next().unwrap().position.length();
        assert!((r0 - 1.0).abs() < 0.05);

        sphere.shrink_fatten(&all_verts, 0.5).unwrap();
        let r1 = sphere.vertices.values().next().unwrap().position.length();
        assert!(r1 > 1.4);

        // Test Relax Vertices (Laplacian smooth)
        let mut plane = Primitives::create_plane(2.0, 2.0, 2, 2);
        // Perturb center vertex
        let center_vk = plane.vertices.iter().find(|(_, v)| v.position.length() < 0.01).map(|(k, _)| k).unwrap();
        plane.vertices[center_vk].position.y = 2.0;

        // Relax center vertex towards flat plane
        plane.relax_vertices(&[center_vk], 5, 0.5).unwrap();
        assert!(plane.vertices[center_vk].position.y < 0.2);
    }

    #[test]
    fn test_face_materials_and_submeshes() {
        let mut cube = Primitives::create_cube(1.0);
        let faces: Vec<FaceKey> = cube.faces.keys().collect();
        assert_eq!(faces.len(), 6);

        // Assign material 0 to first 4 faces, material 1 to last 2 faces
        cube.set_faces_material(&faces[0..4], 0);
        cube.set_faces_material(&faces[4..6], 1);

        assert_eq!(cube.get_face_material(faces[0]).unwrap(), 0);
        assert_eq!(cube.get_face_material(faces[5]).unwrap(), 1);
        assert_eq!(cube.faces_with_material(0).len(), 4);
        assert_eq!(cube.faces_with_material(1).len(), 2);
        assert_eq!(cube.unique_material_ids(), vec![0, 1]);

        // Tessellate Flat Shading
        let flat_rm = cube.to_render_mesh(ShadingMode::Flat);
        assert_eq!(flat_rm.submeshes.len(), 2);
        assert_eq!(flat_rm.submeshes[0].material_id, 0);
        assert_eq!(flat_rm.submeshes[0].index_count, 24); // 4 faces * 2 tris * 3 = 24
        assert_eq!(flat_rm.submeshes[1].material_id, 1);
        assert_eq!(flat_rm.submeshes[1].index_count, 12); // 2 faces * 2 tris * 3 = 12

        // Tessellate Smooth Shading
        let smooth_rm = cube.to_render_mesh(ShadingMode::Smooth);
        assert_eq!(smooth_rm.submeshes.len(), 2);
        assert_eq!(smooth_rm.submeshes[0].material_id, 0);
        assert_eq!(smooth_rm.submeshes[0].index_count, 24);
        assert_eq!(smooth_rm.submeshes[1].material_id, 1);
        assert_eq!(smooth_rm.submeshes[1].index_count, 12);
    }

    #[test]
    fn test_rip_vertices_operator() {
        let mut plane = Primitives::create_plane(2.0, 2.0, 2, 2);
        let initial_vert_count = plane.vertices.len();
        let initial_face_count = plane.faces.len();

        // Find center vertex shared by 4 quads
        let center_vk = plane.vertices.iter().find(|(_, v)| v.position.length() < 0.01).map(|(k, _)| k).unwrap();

        let ripped = plane.rip_vertices(&[center_vk]).unwrap();
        assert_eq!(ripped.len(), 1);
        assert_eq!(plane.vertices.len(), initial_vert_count + 1);
        assert_eq!(plane.faces.len(), initial_face_count);

        // Verify manifold invariants
        assert!(plane.validate().is_ok());
    }

    #[test]
    fn test_profile_shapes_and_extrusion() {
        let star_profile = ProfileShape::Star;
        let star_points = star_profile.generate_cross_section(1.0, 10);
        assert_eq!(star_points.len(), 10);

        let hex_profile = ProfileShape::Hexagon;
        let hex_points = hex_profile.generate_cross_section(1.0, 6);
        assert_eq!(hex_points.len(), 6);

        let spline = Spline::circle(2.0, Vec3::ZERO);
        let mesh = spline.to_extruded_mesh(star_profile, 0.5, 10, 8).unwrap();
        assert!(!mesh.vertices.is_empty());
        assert!(!mesh.faces.is_empty());
        assert!(mesh.validate().is_ok());
    }

    #[test]
    fn test_vertex_group_mirror_and_normalize() {
        let mut plane = Primitives::create_plane(2.0, 2.0, 2, 2);
        let left_vks: Vec<VertexKey> = plane
            .vertices
            .iter()
            .filter(|(_, v)| v.position.x < -0.01)
            .map(|(k, _)| k)
            .collect();
        assert!(!left_vks.is_empty());

        for &vk in &left_vks {
            plane.set_vertex_weight("Arm.L", vk, 0.8);
        }

        let mapped = plane
            .mirror_vertex_group("Arm.L", "Arm.R", crate::modifiers::MirrorAxis::X, 0.01)
            .unwrap();
        assert_eq!(mapped, left_vks.len());

        let right_group = plane.get_vertex_group("Arm.R").unwrap();
        assert_eq!(right_group.weights.len(), left_vks.len());

        // Test normalization
        let all_vks: Vec<VertexKey> = plane.vertices.keys().collect();
        for vk in all_vks {
            plane.set_vertex_weight("Body", vk, 0.4);
        }
        plane.normalize_vertex_groups(&["Arm.L".into(), "Body".into()]);

        for &vk in &left_vks {
            let w_arm = plane.get_vertex_group("Arm.L").unwrap().get_weight(vk);
            let w_body = plane.get_vertex_group("Body").unwrap().get_weight(vk);
            let sum = w_arm + w_body;
            assert!((sum - 1.0).abs() < 1e-4);
        }
    }

    #[test]
    fn test_knife_cut_operator() {
        let mut plane = Primitives::create_plane(2.0, 2.0, 1, 1);
        assert_eq!(plane.faces.len(), 1);
        let face_key = plane.faces.iter().next().map(|(k, _)| k).unwrap();

        let f_hes = plane.face_half_edges(face_key);
        assert_eq!(f_hes.len(), 4);

        // Knife cut across opposite edges
        let e1 = plane.half_edges[f_hes[0]].edge;
        let e2 = plane.half_edges[f_hes[2]].edge;

        let cut_res = plane.knife_cut(
            face_key,
            operators::CutPoint::Edge(e1, 0.5),
            operators::CutPoint::Edge(e2, 0.5),
        );
        assert!(cut_res.is_ok());
        assert_eq!(plane.faces.len(), 2);
        assert!(plane.validate().is_ok());
    }

    #[test]
    fn test_weight_gradient() {
        let mut plane = Primitives::create_plane(2.0, 2.0, 2, 2);
        let updated = plane.apply_weight_gradient(
            "Grad",
            vec3(-1.0, 0.0, 0.0),
            vec3(1.0, 0.0, 0.0),
            0.0,
            1.0,
        );
        assert_eq!(updated, plane.vertices.len());

        for (vk, v) in &plane.vertices {
            let expected_w = ((v.position.x - (-1.0)) / 2.0).clamp(0.0, 1.0);
            let actual_w = plane.get_vertex_weight("Grad", vk);
            assert!((actual_w - expected_w).abs() < 1e-4);
        }
    }

    #[test]
    fn test_edge_bevel_multi_segment() {
        let mut cube = Primitives::create_cube(2.0);
        let edge_key = cube.edges.keys().next().unwrap();

        // Single segment bevel (standard chamfer)
        let bevel_faces_1 = cube.bevel_edge_with_segments(edge_key, 0.2, 1).expect("1-segment bevel");
        assert_eq!(bevel_faces_1.len(), 1);
        assert!(cube.validate().is_ok());

        // Multi-segment bevel (3 segments rounded fillet)
        let mut cube2 = Primitives::create_cube(2.0);
        let edge_key2 = cube2.edges.keys().next().unwrap();
        let bevel_faces_3 = cube2.bevel_edge_with_segments(edge_key2, 0.2, 3).expect("3-segment bevel");
        assert_eq!(bevel_faces_3.len(), 3);
        assert!(cube2.validate().is_ok());
    }

    #[test]
    fn test_uv_seams_and_smart_unwrap() {
        let mut cube = Primitives::create_cube(2.0);

        // Verify seam markings
        assert_eq!(cube.get_seams().len(), 0);
        let first_edge = cube.edges.keys().next().unwrap();
        cube.set_edge_seam(first_edge, true);
        assert!(cube.is_edge_seam(first_edge));
        assert_eq!(cube.get_seams().len(), 1);

        cube.clear_all_seams();
        assert_eq!(cube.get_seams().len(), 0);

        // Smart UV Project on Cube (threshold 66 deg will split 90 deg cube faces into 6 charts)
        cube.smart_uv_project(66.0, 0.02);
        assert!(cube.get_seams().len() > 0);

        // Verify all vertices have valid UV coordinates within [0, 1]
        for v in cube.vertices.values() {
            assert!(v.uv.x >= 0.0 && v.uv.x <= 1.0, "U out of bounds: {}", v.uv.x);
            assert!(v.uv.y >= 0.0 && v.uv.y <= 1.0, "V out of bounds: {}", v.uv.y);
        }

        // Test seam unwrap on cylinder
        let mut cylinder = Primitives::create_cylinder(1.0, 2.0, 16);
        let seam_edge = cylinder.edges.keys().next().unwrap();
        cylinder.set_edge_seam(seam_edge, true);
        cylinder.unwrap_seams(0.02);

        for v in cylinder.vertices.values() {
            assert!(v.uv.x >= 0.0 && v.uv.x <= 1.0, "Cylinder U out of bounds: {}", v.uv.x);
            assert!(v.uv.y >= 0.0 && v.uv.y <= 1.0, "Cylinder V out of bounds: {}", v.uv.y);
        }
    }

    #[test]
    fn test_uv_relax_laplacian_diffusion() {
        // Create a 2x2 grid (9 vertices, 4 quads)
        let mut plane = Primitives::create_plane(2.0, 2.0, 2, 2);
        assert_eq!(plane.vertices.len(), 9);

        // Find the central interior vertex (not on boundary)
        let center_vk = plane.vertices.iter()
            .find(|(_, v)| v.position.x.abs() < 1e-4 && v.position.z.abs() < 1e-4)
            .map(|(k, _)| k)
            .unwrap();

        // Artificially distort center vertex UV
        plane.vertices[center_vk].uv = core_math::vec2(0.9, 0.9);

        // Relax UVs with 10 iterations and factor 0.8
        plane.relax_uvs(10, 0.8);

        // Center vertex UV should have relaxed toward center (0.5, 0.5)
        let relaxed_uv = plane.vertices[center_vk].uv;
        assert!(relaxed_uv.x < 0.7, "Distorted UV should relax toward center: {}", relaxed_uv.x);
        assert!(relaxed_uv.y < 0.7, "Distorted UV should relax toward center: {}", relaxed_uv.y);
    }

    #[test]
    fn test_inset_faces_contiguous_region_vs_individual() {
        // Create a cube and pick two strictly adjacent faces that share an edge
        let mut cube_reg = Primitives::create_cube(2.0);
        let f0 = cube_reg.faces.keys().next().unwrap();
        let f0_verts = cube_reg.face_vertices(f0);
        let f_adj = cube_reg.faces.keys().find(|&fk| {
            if fk == f0 { return false; }
            let f_verts = cube_reg.face_vertices(fk);
            let shared = f0_verts.iter().filter(|v| f_verts.contains(v)).count();
            shared >= 2
        }).expect("Adjacent face must exist on cube");
        let faces = vec![f0, f_adj];

        // 1. Inset contiguous region (individual = false)
        let reg_result = cube_reg.inset_faces(&faces, 0.5, false).expect("Region inset should succeed");
        assert_eq!(reg_result.len(), 2);
        // Region inset creates 2 inner faces + skirts on perimeter (perimeter has 6 boundary edges)
        // 6 original faces - 2 removed + 2 inner + 6 skirts = 12 faces
        assert_eq!(cube_reg.faces.len(), 12);

        // 2. Inset individually (individual = true)
        let mut cube_ind = Primitives::create_cube(2.0);
        let ind_faces = faces.clone();
        let ind_result = cube_ind.inset_faces(&ind_faces, 0.5, true).expect("Individual inset should succeed");
        assert_eq!(ind_result.len(), 2);
        // Each quad gets 4 skirts: 6 - 2 + 2 + 4 + 4 = 14 faces
        assert_eq!(cube_ind.faces.len(), 14);
    }

    #[test]
    fn test_catmull_clark_vertex_creasing() {
        use crate::subdivision::CatmullClarkModifier;
        use crate::modifiers::stack::Modifier;

        let mut cube = Primitives::create_cube(2.0);
        let sharp_vk = cube.vertices.keys().next().unwrap();
        let sharp_orig_pos = cube.vertices[sharp_vk].position;

        // Mark only one vertex fully creased (1.0 = sharp corner)
        cube.set_vertex_crease(sharp_vk, 1.0);

        let sub_mod = CatmullClarkModifier::new(1);
        let subdivided = sub_mod.apply(&cube).expect("Subdivision should succeed");

        // Find the subdivided vertex matching the sharp corner position
        let mut found_sharp = false;
        for (_, v) in &subdivided.vertices {
            if (v.position - sharp_orig_pos).length() < 1e-4 {
                found_sharp = true;
                assert_eq!(v.crease_weight, 1.0);
                break;
            }
        }
        assert!(found_sharp, "Sharp creased vertex must retain its exact corner position after subdivision!");
    }

    #[test]
    fn test_spline_de_casteljau_subdivision_and_arc_length() {
        use crate::spline::{BezierPoint, Spline};
        use core_math::{vec3, Vec3};

        // Straight line from (0,0,0) to (10,0,0) with handles along X
        let p0 = BezierPoint::new(vec3(0.0, 0.0, 0.0), Vec3::ZERO, vec3(3.0, 0.0, 0.0));
        let p1 = BezierPoint::new(vec3(10.0, 0.0, 0.0), vec3(-3.0, 0.0, 0.0), Vec3::ZERO);
        let mut spline = Spline::with_points(vec![p0, p1], false);

        let initial_length = spline.arc_length(16);
        assert!((initial_length - 10.0).abs() < 0.05);

        // Subdivide segment at t=0.5
        let new_idx = spline.subdivide_segment(0, 0.5).expect("Subdivision should succeed");
        assert_eq!(new_idx, 1);
        assert_eq!(spline.points.len(), 3);

        // Position of new midpoint should be at (5, 0, 0)
        let mid_pos = spline.points[1].position;
        assert!((mid_pos - vec3(5.0, 0.0, 0.0)).length() < 1e-4);

        // Arc length after De Casteljau subdivision should remain 10.0
        let subdivided_length = spline.arc_length(16);
        assert!((subdivided_length - 10.0).abs() < 0.05);

        // Evaluate at distance = 7.5
        let (pos_at_d, tan_at_d) = spline.eval_at_distance(7.5, 16);
        assert!((pos_at_d.x - 7.5).abs() < 0.1);
        assert!((tan_at_d - vec3(1.0, 0.0, 0.0)).length() < 0.05);

        // Reverse spline
        spline.reverse();
        assert_eq!(spline.points.first().unwrap().position, vec3(10.0, 0.0, 0.0));
        assert_eq!(spline.points.last().unwrap().position, vec3(0.0, 0.0, 0.0));
    }

    #[test]
    fn test_cloth_simulation_modifier() {
        use crate::modifiers::deform::ClothModifier;
        use crate::modifiers::stack::Modifier;
        use crate::mesh::VertexGroup;

        let mut mesh = Primitives::create_plane(2.0, 2.0, 4, 4);
        // Plane is created in XZ plane with Y=0. Let's elevate it to Y=5.0
        for (_, v) in &mut mesh.vertices {
            v.position.y += 5.0;
        }

        // Create a pinning group for vertices with z < 0.0
        let mut pin_group = VertexGroup::new("PinGroup");
        let mut pinned_count = 0;
        for (vk, v) in &mesh.vertices {
            if v.position.z < 0.0 {
                pin_group.set_weight(vk, 1.0);
                pinned_count += 1;
            }
        }
        assert!(pinned_count > 0);
        mesh.vertex_groups.push(pin_group);

        let mut cloth = ClothModifier::new();
        cloth.pin_group = Some("PinGroup".into());
        cloth.gravity = vec3(0.0, -9.81, 0.0);
        cloth.initialize(&mesh);

        assert!(cloth.is_initialized);
        assert_eq!(cloth.particles.len(), mesh.vertices.len());
        assert!(!cloth.constraints.is_empty());

        // Step simulation for 10 frames
        let dt = 1.0 / 60.0;
        for _ in 0..10 {
            cloth.step(dt);
        }

        // Verify unpinned vertices fell due to gravity, while pinned remained at Y=5.0
        for p in &cloth.particles {
            if p.inv_mass == 0.0 {
                assert!((p.position.y - 5.0).abs() < 1e-4);
            } else {
                assert!(p.position.y < 5.0);
            }
        }

        // Apply modifier to mesh
        let deformed = cloth.apply(&mesh).expect("Cloth modifier application should succeed");
        assert_eq!(deformed.vertices.len(), mesh.vertices.len());
        assert_eq!(deformed.edges.len(), mesh.edges.len());

        // Reset
        cloth.reset();
        for p in &cloth.particles {
            assert_eq!(p.position, p.original_position);
        }
    }

    #[test]
    fn test_stl_ascii_and_binary_roundtrip() {
        let cube = Primitives::create_cube(2.0);

        // 1. ASCII STL export & import
        let ascii_stl = cube.to_stl_ascii("TestCube");
        assert!(ascii_stl.starts_with("solid TestCube"));
        assert!(ascii_stl.ends_with("endsolid TestCube\n"));
        let imported_ascii = Mesh::from_stl(ascii_stl.as_bytes()).expect("ASCII STL import failed");
        assert_eq!(imported_ascii.faces.len(), 12); // 6 quad faces triangulated = 12 triangles

        // 2. Binary STL export & import
        let binary_stl = cube.to_stl_binary();
        assert_eq!(binary_stl.len(), 84 + 12 * 50); // 84 bytes header + 12 triangles * 50 bytes = 684 bytes
        let imported_binary = Mesh::from_stl(&binary_stl).expect("Binary STL import failed");
        assert_eq!(imported_binary.faces.len(), 12);
    }

    #[test]
    fn test_ply_ascii_and_binary_roundtrip() {
        let cube = Primitives::create_cube(2.0);

        // 1. ASCII PLY export & import
        let ascii_ply = cube.to_ply_ascii();
        assert!(ascii_ply.starts_with("ply\nformat ascii 1.0"));
        let imported_ascii = Mesh::from_ply(ascii_ply.as_bytes()).expect("ASCII PLY import failed");
        assert_eq!(imported_ascii.vertices.len(), 8);
        assert_eq!(imported_ascii.faces.len(), 6);

        // 2. Binary PLY export & import
        let binary_ply = cube.to_ply_binary();
        let imported_binary = Mesh::from_ply(&binary_ply).expect("Binary PLY import failed");
        assert_eq!(imported_binary.faces.len(), 12); // Triangulated 12 faces
    }

    #[test]
    fn test_particle_system_simulation_and_mesh_generation() {
        let sphere = Primitives::create_uv_sphere(1.0, 8, 8);
        let mut particles = ParticleSystemModifier::new();
        particles.emission_rate = 120.0;
        particles.lifetime = 1.0;
        particles.floor_collision = true;
        particles.floor_y = -2.0;

        // Initially no particles
        assert_eq!(particles.particles.len(), 0);

        // Step simulation by 0.1s: should spawn 12 particles
        particles.step(0.1, &sphere);
        assert!(particles.particles.len() >= 10, "Expected particles spawned, got {}", particles.particles.len());

        // Step through multiple frames: particles accelerate down under gravity
        for _ in 0..10 {
            particles.step(0.05, &sphere);
        }
        for p in &particles.particles {
            // Check that floor collision kept particle above floor_y
            assert!(p.position.y >= particles.floor_y - 0.001);
        }

        // Test mesh generation for Tetrahedron shape
        particles.render_shape = ParticleRenderShape::Tetrahedron;
        let p_mesh_tetra = particles.build_particle_geometry();
        assert!(!p_mesh_tetra.vertices.is_empty());
        assert!(!p_mesh_tetra.faces.is_empty());

        // Test mesh generation for Diamond shape
        particles.render_shape = ParticleRenderShape::Diamond;
        let p_mesh_diamond = particles.build_particle_geometry();
        assert_eq!(p_mesh_diamond.faces.len(), particles.particles.len() * 8);

        // Test Modifier trait apply
        particles.emit_only = false;
        let combined = particles.apply(&sphere).expect("Particle apply failed");
        assert!(combined.vertices.len() > sphere.vertices.len());
        assert!(combined.faces.len() > sphere.faces.len());

        // Reset
        particles.reset();
        assert_eq!(particles.particles.len(), 0);
    }

    #[test]
    fn test_two_rail_sweep() {
        let rail1 = vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 2.0),
            Vec3::new(0.0, 0.0, 4.0),
            Vec3::new(0.0, 0.0, 6.0),
        ];
        let rail2 = vec![
            Vec3::new(2.0, 0.0, 0.0),
            Vec3::new(3.0, 0.0, 2.0),
            Vec3::new(3.0, 0.0, 4.0),
            Vec3::new(1.0, 0.0, 6.0),
        ];
        let profile = vec![
            core_math::Vec2::new(0.0, 0.0),
            core_math::Vec2::new(0.5, 0.8),
            core_math::Vec2::new(1.0, 0.0),
        ];

        let mesh = Spline::two_rail_sweep(&rail1, &rail2, &profile, false).expect("Two rail sweep failed");
        assert_eq!(mesh.vertices.len(), 12);
        assert_eq!(mesh.faces.len(), 11);
        assert!(mesh.validate().is_ok());
    }

    #[test]
    fn test_spline_loft() {
        // Create 3 concentric circular profiles stacked along Y
        let p0: Vec<Vec3> = (0..8).map(|i| {
            let a = i as f32 * std::f32::consts::TAU / 8.0;
            Vec3::new(a.cos() * 1.0, 0.0, a.sin() * 1.0)
        }).collect();
        let p1: Vec<Vec3> = (0..8).map(|i| {
            let a = i as f32 * std::f32::consts::TAU / 8.0;
            Vec3::new(a.cos() * 2.0, 2.0, a.sin() * 2.0)
        }).collect();
        let p2: Vec<Vec3> = (0..8).map(|i| {
            let a = i as f32 * std::f32::consts::TAU / 8.0;
            Vec3::new(a.cos() * 0.8, 4.0, a.sin() * 0.8)
        }).collect();

        let loft_mesh = Spline::loft(&[p0, p1, p2], true, false).expect("Loft should succeed");
        assert_eq!(loft_mesh.vertices.len(), 24);
        assert_eq!(loft_mesh.faces.len(), 18); // 16 quads + 2 end caps
        assert!(loft_mesh.validate().is_ok());
    }

    #[test]
    fn test_offset_polygon_2d() {
        let quad = vec![
            core_math::Vec2::new(0.0, 0.0),
            core_math::Vec2::new(2.0, 0.0),
            core_math::Vec2::new(2.0, 2.0),
            core_math::Vec2::new(0.0, 2.0),
        ];

        let offset = Spline::offset_polygon_2d(&quad, 0.2, true);
        assert_eq!(offset.len(), 4);
        // Centroid of offset should match centroid of original (1.0, 1.0)
        let orig_c = (quad[0] + quad[1] + quad[2] + quad[3]) * 0.25;
        let off_c = (offset[0] + offset[1] + offset[2] + offset[3]) * 0.25;
        assert!((orig_c - off_c).length() < 1e-4);
    }

    #[test]
    fn test_sculpt_mask_and_brush_pinning() {
        let mut mesh = Primitives::create_cube(2.0);
        let center = Vec3::new(1.0, 1.0, 1.0);

        // 1. Paint mask onto the corner vertex
        let mask_settings = crate::sculpt::SculptSettings {
            brush: crate::sculpt::SculptBrush::Mask,
            radius: 1.5,
            strength: 1.0,
            falloff: crate::sculpt::BrushFalloff::Smooth,
            invert: false,
        };
        let affected = mesh.apply_sculpt_brush(center, Vec3::Y, &mask_settings, None);
        assert!(affected > 0);

        // Verify corner vertex is masked
        let masked_vks: Vec<_> = mesh.vertices.iter().filter(|(_, v)| v.sculpt_mask > 0.5).map(|(vk, _)| vk).collect();
        assert!(!masked_vks.is_empty());

        // Record initial positions of masked vertices
        let initial_positions: Vec<_> = masked_vks.iter().map(|&vk| (vk, mesh.vertices[vk].position)).collect();

        // 2. Apply Inflate brush directly on the masked area
        let inflate_settings = crate::sculpt::SculptSettings {
            brush: crate::sculpt::SculptBrush::Inflate,
            radius: 1.5,
            strength: 0.8,
            falloff: crate::sculpt::BrushFalloff::Smooth,
            invert: false,
        };
        mesh.apply_sculpt_brush(center, Vec3::Y, &inflate_settings, None);

        // Masked vertices (weight > 0.99) should barely have moved or not moved at all
        for (vk, orig_pos) in initial_positions {
            let v = &mesh.vertices[vk];
            if v.sculpt_mask >= 0.95 {
                assert!((v.position - orig_pos).length() < 0.02);
            }
        }

        // 3. Test Invert Mask
        mesh.invert_sculpt_mask();
        for &vk in &masked_vks {
            assert!(mesh.vertices[vk].sculpt_mask < 0.5);
        }

        // 4. Test Clear Mask
        mesh.clear_sculpt_mask();
        for v in mesh.vertices.values() {
            assert_eq!(v.sculpt_mask, 0.0);
        }
    }

    #[test]
    fn test_voxel_remesh_operator_and_modifier() {
        let cube = Primitives::create_cube(2.0);
        assert_eq!(cube.faces.len(), 6);

        // Voxel remesh at 0.4m spacing
        let remeshed = cube.voxel_remesh(0.4).expect("Voxel remesh should succeed");
        assert!(remeshed.faces.len() >= 20);
        assert!(remeshed.vertices.len() >= 20);
        assert!(remeshed.validate().is_ok());

        // Test non-destructive RemeshModifier
        let modifier = crate::RemeshModifier::new(0.4);
        let mod_result = modifier.apply(&cube).expect("RemeshModifier should succeed");
        assert!(mod_result.faces.len() >= 20);
    }

    #[test]
    fn test_3d_print_mesh_analysis_and_autofix() {
        let mut cube = Primitives::create_cube(2.0);
        let report = cube.analyze_for_3d_printing(45.0);

        assert!(report.is_watertight);
        assert!((report.volume_m3 - 8.0).abs() < 1e-3);
        assert!((report.volume_cm3 - 8_000_000.0).abs() < 1e2);
        assert!((report.surface_area_m2 - 24.0).abs() < 1e-3);
        assert_eq!(report.open_boundary_edges, 0);
        assert_eq!(report.degenerate_faces, 0);
        assert_eq!(report.overhang_faces, 1); // bottom face
        assert!((report.dimensions_mm.x - 2000.0).abs() < 1e-2);

        // Delete top face to create an open hole
        let top_fk = cube.faces.iter().find(|(_, f)| f.normal.y > 0.9).map(|(k, _)| k).unwrap();
        cube.delete_face(top_fk);

        let broken_report = cube.analyze_for_3d_printing(45.0);
        assert!(!broken_report.is_watertight);
        assert_eq!(broken_report.open_boundary_edges, 4);

        // Auto-fix: cap hole and restore watertightness
        let fixed = cube.auto_fix_for_3d_printing().unwrap();
        assert!(fixed > 0);
        let fixed_report = cube.analyze_for_3d_printing(45.0);
        assert!(fixed_report.is_watertight);
        assert_eq!(fixed_report.open_boundary_edges, 0);
    }

    #[test]
    fn test_create_plumb_bob_signature_mascot() {
        let plumb = Primitives::create_plumb_bob();
        assert!(plumb.validate().is_ok(), "Plumb Bob mesh must satisfy all half-edge 2-manifold invariants");

        // 14 profile rings, 24 segments
        // Ring 0 is pole (1 vert), rings 1..12 have 24 verts, ring 13 is tip pole (1 vert) -> 2 + 12*24 = 290 verts
        assert_eq!(plumb.vertices.len(), 2 + 12 * 24);
        // Top fan: 24 tris, Bottom fan: 24 tris, 11 quad rings * 24 = 264 quads -> total 312 faces
        assert_eq!(plumb.faces.len(), 24 + 24 + 11 * 24);

        // Check watertightness
        let report = plumb.analyze_for_3d_printing(45.0);
        assert!(report.is_watertight, "Plumb Bob must be completely watertight (0 boundary edges)");
        assert_eq!(report.open_boundary_edges, 0);

        // Check bounds
        let aabb = plumb.compute_aabb();
        assert!((aabb.min.y - (-2.25)).abs() < 1e-4, "Needle tip must touch true vertical Y = -2.25");
        assert!((aabb.max.y - 2.30).abs() < 1e-4, "Finial apex must reach Y = 2.30");
        assert!((aabb.max.x - 1.00).abs() < 0.05, "Widest radius must be approximately 1.0");
    }

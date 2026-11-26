#[allow(unused_imports)]
use super::*;

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
            ..Default::default()
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
            ..Default::default()
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
            ..Default::default()
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
            ..Default::default()
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
            ..Default::default()
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
            ..Default::default()
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
            ..Default::default()
        };
        let clayed = plane.apply_sculpt_brush(Vec3::ZERO, Vec3::Y, &clay_settings, None);
        assert!(clayed > 0);
        assert!(plane.validate().is_ok());
    }


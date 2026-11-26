#[allow(unused_imports)]
use super::*;

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
            ..Default::default()
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
            ..Default::default()
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

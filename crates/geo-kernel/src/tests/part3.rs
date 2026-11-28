#[allow(unused_imports)]
use super::*;

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
    let center_vk = plane
        .vertices
        .keys()
        .find(|&vk| plane.vertices[vk].position.length() < 0.1)
        .unwrap();
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

    cube.symmetrize(MirrorAxis::X, SymmetrizeDirection::PositiveToNegative)
        .unwrap();
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

    let new_faces = plane
        .spin(&[fk], Vec3::Y, Vec3::ZERO, std::f32::consts::TAU, 8)
        .unwrap();
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
    let affected = plane.apply_color_brush(
        Vec3::ZERO,
        red,
        0.8,
        1.0,
        crate::sculpt::BrushFalloff::Linear,
    );
    assert!(affected > 0);

    // Center vertex should be closer to red
    let center_vk = plane
        .vertices
        .iter()
        .find(|(_, v)| v.position.length() < 0.01)
        .map(|(k, _)| k)
        .unwrap();
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
    let top_edges: Vec<EdgeKey> = cube
        .edges
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
    morpher
        .add_target_from_mesh("TopStretch", &base_cube, &target_cube)
        .unwrap();

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
    let center_vk = plane
        .vertices
        .iter()
        .find(|(_, v)| v.position.length() < 0.01)
        .map(|(k, _)| k)
        .unwrap();
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
    let center_vk = plane
        .vertices
        .iter()
        .find(|(_, v)| v.position.length() < 0.01)
        .map(|(k, _)| k)
        .unwrap();

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
    let updated =
        plane.apply_weight_gradient("Grad", vec3(-1.0, 0.0, 0.0), vec3(1.0, 0.0, 0.0), 0.0, 1.0);
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
    let bevel_faces_1 = cube
        .bevel_edge_with_segments(edge_key, 0.2, 1)
        .expect("1-segment bevel");
    assert_eq!(bevel_faces_1.len(), 1);
    assert!(cube.validate().is_ok());

    // Multi-segment bevel (3 segments rounded fillet)
    let mut cube2 = Primitives::create_cube(2.0);
    let edge_key2 = cube2.edges.keys().next().unwrap();
    let bevel_faces_3 = cube2
        .bevel_edge_with_segments(edge_key2, 0.2, 3)
        .expect("3-segment bevel");
    assert_eq!(bevel_faces_3.len(), 3);
    assert!(cube2.validate().is_ok());
}

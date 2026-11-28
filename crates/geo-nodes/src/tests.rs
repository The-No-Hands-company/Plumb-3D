#[cfg(test)]
mod tests {
    use crate::graph::GeoNodeGraph;
    use crate::nodes::*;
    use crate::types::*;

    #[test]
    fn test_default_graph_evaluation() {
        let mut graph = GeoNodeGraph::new();
        let mesh = graph
            .evaluate()
            .expect("Evaluation should succeed")
            .expect("Should produce mesh");
        assert_eq!(mesh.faces.len(), 6);
        assert_eq!(mesh.vertices.len(), 8);
    }

    #[test]
    fn test_procedural_extrude_subdivide_graph() {
        let mut graph = GeoNodeGraph::new();

        // 1. Clear default nodes
        let keys: Vec<_> = graph.nodes.keys().collect();
        for k in keys {
            graph.remove_node(k);
        }

        // 2. Add Cylinder -> Extrude -> Subdivide -> Group Output
        let cyl_key = graph.add_node(Box::new(CylinderNode), [-300.0, 0.0]);
        let extrude_key = graph.add_node(Box::new(ExtrudeNode), [-100.0, 0.0]);
        let subd_key = graph.add_node(Box::new(SubdivideNode), [100.0, 0.0]);
        let out_key = graph.add_node(Box::new(GroupOutputNode), [300.0, 0.0]);
        graph.output_node = Some(out_key);

        // 3. Connect nodes
        assert!(graph.link(cyl_key, 0, extrude_key, 0).is_ok());
        assert!(graph.link(extrude_key, 0, subd_key, 0).is_ok());
        assert!(graph.link(subd_key, 0, out_key, 0).is_ok());

        // 4. Evaluate procedural result
        let mesh = graph
            .evaluate()
            .expect("Graph evaluation should succeed")
            .expect("Should produce mesh");
        assert!(mesh.faces.len() > 20);
        assert!(mesh.vertices.len() > 20);
        assert!(mesh.validate().is_ok());
    }

    #[test]
    fn test_procedural_boolean_graph() {
        let mut graph = GeoNodeGraph::new();

        // Clear default
        let keys: Vec<_> = graph.nodes.keys().collect();
        for k in keys {
            graph.remove_node(k);
        }

        let cube_key = graph.add_node(Box::new(CubeNode), [-300.0, -100.0]);
        let sphere_key = graph.add_node(Box::new(SphereNode), [-300.0, 100.0]);
        let bool_key = graph.add_node(Box::new(BooleanNode), [0.0, 0.0]);
        let out_key = graph.add_node(Box::new(GroupOutputNode), [250.0, 0.0]);
        graph.output_node = Some(out_key);

        // Cube as Mesh A, Sphere as Mesh B, operation difference (0)
        assert!(graph.link(cube_key, 0, bool_key, 0).is_ok());
        assert!(graph.link(sphere_key, 0, bool_key, 1).is_ok());
        assert!(graph.link(bool_key, 0, out_key, 0).is_ok());

        let mesh = graph
            .evaluate()
            .expect("Boolean graph should succeed")
            .expect("Should produce mesh");
        assert!(mesh.faces.len() > 6);
        assert!(mesh.validate().is_ok());
    }

    #[test]
    fn test_cycle_prevention() {
        let mut graph = GeoNodeGraph::new();

        let node_a = graph.add_node(Box::new(SubdivideNode), [0.0, 0.0]);
        let node_b = graph.add_node(Box::new(SubdivideNode), [100.0, 0.0]);

        assert!(graph.link(node_a, 0, node_b, 0).is_ok());
        // Attempting to link B back into A should return CyclicDependency error
        let cycle_res = graph.link(node_b, 0, node_a, 0);
        assert!(matches!(cycle_res, Err(NodeGraphError::CyclicDependency)));
    }

    #[test]
    fn test_math_and_transform_pipeline() {
        let mut graph = GeoNodeGraph::new();

        let keys: Vec<_> = graph.nodes.keys().collect();
        for k in keys {
            graph.remove_node(k);
        }

        let cube_key = graph.add_node(Box::new(CubeNode), [-400.0, 0.0]);
        let val_x = graph.add_node(Box::new(ValueNode { value: 5.0 }), [-400.0, 150.0]);
        let combine_key = graph.add_node(Box::new(CombineXyzNode), [-200.0, 150.0]);
        let transform_key = graph.add_node(Box::new(TransformGeometryNode), [0.0, 0.0]);
        let out_key = graph.add_node(Box::new(GroupOutputNode), [200.0, 0.0]);
        graph.output_node = Some(out_key);

        // Connect val_x -> CombineXyz (X), CombineXyz -> Transform (Translation)
        assert!(graph.link(val_x, 0, combine_key, 0).is_ok());
        assert!(graph.link(combine_key, 0, transform_key, 1).is_ok()); // Translation socket
        assert!(graph.link(cube_key, 0, transform_key, 0).is_ok()); // Geometry socket
        assert!(graph.link(transform_key, 0, out_key, 0).is_ok());

        let mesh = graph
            .evaluate()
            .expect("Transform graph should succeed")
            .expect("Should produce mesh");
        let center = mesh.bounding_center();
        assert!((center.x - 5.0).abs() < 1e-4);
    }

    #[test]
    fn test_procedural_scattering_and_instancing() {
        let mut graph = GeoNodeGraph::new();
        let keys: Vec<_> = graph.nodes.keys().collect();
        for k in keys {
            graph.remove_node(k);
        }

        let base_plane = graph.add_node(Box::new(PlaneNode), [-400.0, 0.0]);
        let small_cube = graph.add_node(Box::new(CubeNode), [-400.0, 150.0]);
        let scatter = graph.add_node(Box::new(DistributePointsOnFacesNode), [-200.0, 0.0]);
        let instance = graph.add_node(Box::new(InstanceOnPointsNode), [0.0, 0.0]);
        let out_key = graph.add_node(Box::new(GroupOutputNode), [200.0, 0.0]);
        graph.output_node = Some(out_key);

        // Plane -> Scatter (Geometry)
        assert!(graph.link(base_plane, 0, scatter, 0).is_ok());
        // Scatter (Points) -> Instance (Points)
        assert!(graph.link(scatter, 0, instance, 0).is_ok());
        // Small Cube -> Instance (Instance)
        assert!(graph.link(small_cube, 0, instance, 1).is_ok());
        // Instance -> Output
        assert!(graph.link(instance, 0, out_key, 0).is_ok());

        let mesh = graph
            .evaluate()
            .expect("Scattering should succeed")
            .expect("Should produce mesh");
        // Verify multiple instances were scattered onto the plane surface
        assert!(mesh.vertex_count() >= 8 * 2);
        assert!(mesh.face_count() >= 6 * 2);
    }

    #[test]
    fn test_procedural_noise_displacement() {
        let mut graph = GeoNodeGraph::new();
        let keys: Vec<_> = graph.nodes.keys().collect();
        for k in keys {
            graph.remove_node(k);
        }

        let cube_key = graph.add_node(Box::new(CubeNode), [-300.0, 0.0]);
        let noise_key = graph.add_node(Box::new(NoiseTextureNode), [-300.0, 150.0]);
        let displace_key = graph.add_node(Box::new(DisplaceGeometryNode), [-100.0, 0.0]);
        let out_key = graph.add_node(Box::new(GroupOutputNode), [100.0, 0.0]);
        graph.output_node = Some(out_key);

        // Cube -> Displace (Geometry)
        assert!(graph.link(cube_key, 0, displace_key, 0).is_ok());
        // Noise (Fac) -> Displace (Displacement)
        assert!(graph.link(noise_key, 0, displace_key, 1).is_ok());
        // Displace -> Output
        assert!(graph.link(displace_key, 0, out_key, 0).is_ok());

        let mesh = graph
            .evaluate()
            .expect("Displacement should succeed")
            .expect("Should produce mesh");
        assert_eq!(mesh.vertex_count(), 8);
        // Vertices should be displaced outward along normals
        let dims = mesh.bounding_dimensions();
        assert!(dims.x > 2.0);
    }

    #[test]
    fn test_geometry_nodes_modifier() {
        use crate::modifier::GeometryNodesModifier;
        use geo_kernel::{modifiers::Modifier, Primitives};

        let mut graph = GeoNodeGraph::new();
        let keys: Vec<_> = graph.nodes.keys().collect();
        for k in keys {
            graph.remove_node(k);
        }

        let group_in = graph.add_node(Box::new(GroupInputNode::default()), [-300.0, 0.0]);
        let subdivide = graph.add_node(Box::new(SubdivideNode), [-100.0, 0.0]);
        let out_key = graph.add_node(Box::new(GroupOutputNode), [100.0, 0.0]);
        graph.output_node = Some(out_key);

        assert!(graph.link(group_in, 0, subdivide, 0).is_ok());
        assert!(graph.link(subdivide, 0, out_key, 0).is_ok());

        let modifier = GeometryNodesModifier::new("ProceduralGeoNodes", graph);
        let base_cube = Primitives::create_cube(2.0);
        let result = modifier
            .apply(&base_cube)
            .expect("Modifier evaluation should succeed");

        // Base cube (6 faces) subdivided -> 24 faces
        assert_eq!(result.face_count(), 24);
        assert_eq!(result.vertex_count(), 26);
    }

    #[test]
    fn test_composition_and_material_nodes() {
        let mut graph = GeoNodeGraph::new();
        let keys: Vec<_> = graph.nodes.keys().collect();
        for k in keys {
            graph.remove_node(k);
        }

        // Cube + Plane -> Join Geometry -> Set Material -> Group Output
        let cube_key = graph.add_node(Box::new(CubeNode), [-300.0, -100.0]);
        let plane_key = graph.add_node(Box::new(PlaneNode), [-300.0, 100.0]);
        let join_key = graph.add_node(Box::new(JoinGeometryNode), [-100.0, 0.0]);
        let mat_key = graph.add_node(Box::new(SetMaterialNode), [100.0, 0.0]);
        let out_key = graph.add_node(Box::new(GroupOutputNode), [300.0, 0.0]);
        graph.output_node = Some(out_key);

        assert!(graph.link(cube_key, 0, join_key, 0).is_ok());
        assert!(graph.link(plane_key, 0, join_key, 1).is_ok());
        assert!(graph.link(join_key, 0, mat_key, 0).is_ok());
        assert!(graph.link(mat_key, 0, out_key, 0).is_ok());

        // Connect a constant value 3 to material slot
        let val_key = graph.add_node(Box::new(ValueNode { value: 3.0 }), [-100.0, 150.0]);
        assert!(graph.link(val_key, 0, mat_key, 1).is_ok());

        let mesh = graph
            .evaluate()
            .expect("Graph should evaluate")
            .expect("Should produce mesh");
        // Cube (6 faces) + Plane (4x4 = 16 faces) = 22 faces
        assert_eq!(mesh.faces.len(), 22);
        for f in mesh.faces.values() {
            assert_eq!(f.material_id, 3);
        }

        // Test MeshToPointsNode
        let m2p = MeshToPointsNode;
        let points_res = m2p
            .execute(&[SocketValue::Geometry(Some(mesh))])
            .expect("MeshToPoints execution should succeed");
        if let SocketValue::Points(pts) = &points_res[0] {
            // Cube has 8 vertices, 4x4 plane has 25 vertices -> 33 points
            assert_eq!(pts.len(), 33);
        } else {
            panic!("Expected SocketValue::Points");
        }
    }

    #[test]
    fn test_procedural_voxel_remesh_node() {
        let mut graph = GeoNodeGraph::new();

        // Clear default nodes
        let keys: Vec<_> = graph.nodes.keys().collect();
        for k in keys {
            graph.remove_node(k);
        }

        // Cube -> Voxel Remesh -> Group Output
        let cube_key = graph.add_node(Box::new(CubeNode), [-200.0, 0.0]);
        let remesh_key = graph.add_node(Box::new(VoxelRemeshNode), [0.0, 0.0]);
        let out_key = graph.add_node(Box::new(GroupOutputNode), [200.0, 0.0]);
        graph.output_node = Some(out_key);

        assert!(graph.link(cube_key, 0, remesh_key, 0).is_ok());
        assert!(graph.link(remesh_key, 0, out_key, 0).is_ok());

        let mesh = graph
            .evaluate()
            .expect("Graph should evaluate")
            .expect("Should produce mesh");
        assert!(mesh.faces.len() > 10);
        assert!(mesh.vertices.len() > 10);
        assert!(mesh.validate().is_ok());
    }

    #[test]
    fn test_procedural_curve_fillet_and_revolve_nodes() {
        // Test CurveFilletNode
        let fillet_node = CurveFilletNode;
        let fillet_res = fillet_node
            .execute(&[SocketValue::Float(0.3), SocketValue::Float(0.1)])
            .expect("CurveFilletNode execution should succeed");

        if let SocketValue::Geometry(Some(m)) = &fillet_res[0] {
            assert!(m.face_count() > 20);
            assert!(m.validate().is_ok());
        } else {
            panic!("Expected Geometry output");
        }

        // Test CurveRevolveNode
        let revolve_node = CurveRevolveNode;
        let revolve_res = revolve_node
            .execute(&[
                SocketValue::Float(1.5),
                SocketValue::Float(3.0),
                SocketValue::Int(16),
                SocketValue::Float(360.0),
            ])
            .expect("CurveRevolveNode execution should succeed");

        if let SocketValue::Geometry(Some(m)) = &revolve_res[0] {
            assert!(m.face_count() > 30);
            assert!(m.validate().is_ok());
        } else {
            panic!("Expected Geometry output");
        }
    }

    #[test]
    fn test_mirror_and_bevel_nodes() {
        use geo_kernel::Primitives;

        let cube = Primitives::create_cube(2.0);
        assert_eq!(cube.faces.len(), 6);

        // Test MirrorNode across X axis
        let mirror_node = MirrorNode;
        let mirror_res = mirror_node
            .execute(&[
                SocketValue::Geometry(Some(cube.clone())),
                SocketValue::Int(0), // X axis
            ])
            .expect("MirrorNode should succeed");

        if let SocketValue::Geometry(Some(m)) = &mirror_res[0] {
            assert_eq!(m.faces.len(), 12);
        } else {
            panic!("Expected Geometry output from MirrorNode");
        }

        // Test BevelNode
        let bevel_node = BevelNode;
        let bevel_res = bevel_node
            .execute(&[SocketValue::Geometry(Some(cube)), SocketValue::Float(0.1)])
            .expect("BevelNode should succeed");

        if let SocketValue::Geometry(Some(m)) = &bevel_res[0] {
            assert!(m.faces.len() > 6);
        } else {
            panic!("Expected Geometry output from BevelNode");
        }
    }

    #[test]
    fn test_capture_and_transfer_attribute_nodes() {
        use geo_kernel::Primitives;

        let cube1 = Primitives::create_cube(2.0);
        let cube2 = Primitives::create_cube(2.0);

        // 1. Capture Attribute onto cube1
        let capture_node = CaptureAttributeNode {
            attribute_name: "Density".to_string(),
            domain: AttributeDomain::Vertex,
        };
        let cap_res = capture_node
            .execute(&[SocketValue::Geometry(Some(cube1)), SocketValue::Float(0.75)])
            .expect("CaptureAttributeNode should succeed");

        assert_eq!(cap_res.len(), 2);
        let captured_field = match &cap_res[1] {
            SocketValue::Field(f) => {
                assert_eq!(f.name, "Density");
                assert_eq!(f.len(), 8);
                assert_eq!(f.get_float(0), Some(0.75));
                f.clone()
            }
            _ => panic!("Expected Field output"),
        };

        // 2. Transfer Attribute from cube1 to cube2
        let transfer_node = TransferAttributeNode {
            attribute_name: "Density".to_string(),
        };
        let trans_res = transfer_node
            .execute(&[
                SocketValue::Geometry(Some(cube2)),
                cap_res[0].clone(),
                SocketValue::Field(captured_field),
            ])
            .expect("TransferAttributeNode should succeed");

        assert_eq!(trans_res.len(), 2);
        if let SocketValue::Geometry(Some(tgt_m)) = &trans_res[0] {
            let vg = tgt_m
                .vertex_groups
                .iter()
                .find(|g| g.name == "Density")
                .expect("Transferred group must exist");
            for vk in tgt_m.vertices.keys() {
                assert!((vg.get_weight(vk) - 0.75).abs() < 1e-4);
            }
        } else {
            panic!("Expected Geometry output");
        }
    }
}

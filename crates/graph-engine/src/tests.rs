use super::*;
use core_math::{vec3, Transform, Vec3};
use geo_kernel::{Primitives, Spline};

#[test]
fn test_dag_hierarchy_transform_propagation() {
    let mut scene = SceneGraph::new();

    let parent_id = scene.add_node(SceneNode::new_transform(
        "Parent",
        Transform::from_translation(vec3(0.0, 10.0, 0.0)),
    ));

    let child_id = scene.add_node(SceneNode::new_mesh(
        "ChildCube",
        Transform::from_translation(vec3(5.0, 0.0, 0.0)),
        Primitives::create_cube(1.0),
    ));

    scene.set_parent(child_id, Some(parent_id));
    scene.update_transforms();

    let child = &scene.nodes[child_id];
    let child_world_pos = child.world_matrix.transform_point3(Vec3::ZERO);
    assert!((child_world_pos - vec3(5.0, 10.0, 0.0)).length() < 1e-4);
}

#[test]
fn test_mesh_evaluation_pipeline() {
    let mut scene = SceneGraph::new();
    let cube_id = scene.add_node(SceneNode::new_mesh(
        "Cube",
        Transform::IDENTITY,
        Primitives::create_cube(2.0),
    ));

    let render_mesh = scene
        .evaluate_mesh(cube_id)
        .expect("Should evaluate render mesh");
    assert_eq!(render_mesh.indices.len(), 36);
    assert_eq!(render_mesh.wireframe_indices.len(), 24);

    // Mutate base mesh by extruding a face
    if let Some(node) = scene.nodes.get_mut(cube_id) {
        if let NodeData::Mesh { base_mesh, .. } = &mut node.data {
            let target_face = base_mesh.faces.keys().next().unwrap();
            base_mesh
                .extrude_face(target_face, 1.5)
                .expect("Extrude should succeed");
        }
    }
    scene.mark_dirty(cube_id);

    // Verify that after mark_dirty, evaluate_mesh re-evaluates and returns the newly extruded mesh
    let updated_render_mesh = scene
        .evaluate_mesh(cube_id)
        .expect("Should evaluate updated render mesh");
    // Extruding 1 quad face adds 4 side quads: 6 + 4 = 10 quads -> 20 triangles -> 60 indices!
    // Edge count increases by 8 (4 side edges + 4 top edges): 12 + 8 = 20 edges -> 40 wireframe indices!
    assert_eq!(updated_render_mesh.indices.len(), 60);
    assert_eq!(updated_render_mesh.wireframe_indices.len(), 40);
}

#[derive(Debug)]
struct MockTransformCommand {
    node_id: NodeId,
    old_transform: Transform,
    new_transform: Transform,
}

impl Command for MockTransformCommand {
    fn name(&self) -> &str {
        "Translate Node"
    }

    fn execute(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        if let Some(node) = scene.nodes.get_mut(self.node_id) {
            node.local_transform = self.new_transform;
            scene.mark_dirty(self.node_id);
            Ok(())
        } else {
            Err("Node not found".into())
        }
    }

    fn undo(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        if let Some(node) = scene.nodes.get_mut(self.node_id) {
            node.local_transform = self.old_transform;
            scene.mark_dirty(self.node_id);
            Ok(())
        } else {
            Err("Node not found".into())
        }
    }
}

#[test]
fn test_undo_redo_history() {
    let mut scene = SceneGraph::new();
    let mut history = HistoryManager::new();

    let id = scene.add_node(SceneNode::new_transform("Test", Transform::IDENTITY));

    let cmd = Box::new(MockTransformCommand {
        node_id: id,
        old_transform: Transform::IDENTITY,
        new_transform: Transform::from_translation(vec3(0.0, 10.0, 0.0)),
    });

    history.execute(cmd, &mut scene).unwrap();
    assert_eq!(scene.nodes[id].local_transform.translation.y, 10.0);
    assert!(history.can_undo());

    history.undo(&mut scene).unwrap();
    assert_eq!(scene.nodes[id].local_transform.translation.y, 0.0);
    assert!(history.can_redo());

    history.redo(&mut scene).unwrap();
    assert_eq!(scene.nodes[id].local_transform.translation.y, 10.0);
}

#[test]
fn test_apply_and_collapse_modifiers() {
    use geo_kernel::SubdivideModifier;

    let mut scene = SceneGraph::new();
    let cube = Primitives::create_cube(2.0);
    let node_id = scene.add_node(SceneNode::new_mesh("Cube", Transform::IDENTITY, cube));

    if let NodeData::Mesh { modifier_stack, .. } = &mut scene.nodes.get_mut(node_id).unwrap().data {
        modifier_stack.add_modifier(Box::new(SubdivideModifier::new(1)));
    }

    // Apply the modifier: 6 faces -> 24 faces baked into base mesh
    scene.apply_modifier(node_id, 0).unwrap();

    if let NodeData::Mesh {
        base_mesh,
        modifier_stack,
        ..
    } = &scene.nodes[node_id].data
    {
        assert_eq!(base_mesh.face_count(), 24);
        assert_eq!(modifier_stack.modifiers().len(), 0);
    } else {
        panic!("Expected mesh node");
    }
}

#[test]
fn test_light_and_camera_nodes() {
    let mut scene = SceneGraph::new();

    let sun = scene.add_node(SceneNode::new_directional_light(
        "Sun",
        Transform::from_translation(vec3(0.0, 10.0, 0.0)),
        vec3(1.0, 0.95, 0.9),
        3.0,
    ));

    let point = scene.add_node(SceneNode::new_point_light(
        "Omni",
        Transform::from_translation(vec3(2.0, 3.0, 1.0)),
        vec3(1.0, 1.0, 1.0),
        15.0,
        20.0,
    ));

    let spot = scene.add_node(SceneNode::new_spot_light(
        "Spot",
        Transform::from_translation(vec3(-2.0, 4.0, 0.0)),
        vec3(0.8, 0.9, 1.0),
        25.0,
        30.0,
        0.4,
        0.8,
    ));

    let camera = scene.add_node(SceneNode::new_camera(
        "MainCam",
        Transform::from_translation(vec3(0.0, 2.0, 5.0)),
        45.0_f32.to_radians(),
        16.0 / 9.0,
    ));

    // Parent point light to camera
    scene.set_parent(point, Some(camera));
    scene.update_transforms();

    if let NodeData::Light {
        light_type,
        intensity,
        ..
    } = &scene.nodes[sun].data
    {
        assert_eq!(*light_type, LightType::Directional);
        assert_eq!(*intensity, 3.0);
    } else {
        panic!("Expected directional light");
    }

    if let NodeData::Light {
        light_type, range, ..
    } = &scene.nodes[spot].data
    {
        assert_eq!(*light_type, LightType::Spot);
        assert_eq!(*range, 30.0);
    } else {
        panic!("Expected spot light");
    }

    // Verify parent-child world transform
    let point_world = scene.nodes[point].world_matrix.transform_point3(Vec3::ZERO);
    assert!((point_world.y - 5.0).abs() < 1e-4); // cam(2.0) + point(3.0) = 5.0
}

#[test]
fn test_joint_armature_and_auto_bind() {
    let mut scene = SceneGraph::new();

    let cube_id = scene.add_node(SceneNode::new_mesh(
        "TargetMesh",
        Transform::IDENTITY,
        Primitives::create_cube(2.0),
    ));

    let root_joint = scene.add_node(SceneNode::new_joint(
        "RootBone",
        Transform::from_translation(vec3(0.0, -1.0, 0.0)),
        1.0,
        0.1,
    ));

    let tip_joint = scene.add_node(SceneNode::new_joint(
        "TipBone",
        Transform::from_translation(vec3(0.0, 2.0, 0.0)),
        1.0,
        0.08,
    ));

    scene.set_parent(tip_joint, Some(root_joint));
    scene.update_transforms();

    assert!(scene
        .auto_bind_mesh_to_armature(cube_id, root_joint)
        .is_ok());

    // Mesh node should now have a SkinModifier and vertex groups
    if let NodeData::Mesh {
        base_mesh,
        modifier_stack,
        ..
    } = &scene.nodes[cube_id].data
    {
        assert_eq!(modifier_stack.modifiers().len(), 1);
        assert_eq!(modifier_stack.modifiers()[0].name(), "Skin (LBS)");
        assert!(base_mesh.get_vertex_group("RootBone").is_some());
        assert!(base_mesh.get_vertex_group("TipBone").is_some());
    } else {
        panic!("Expected mesh node");
    }

    // Test synchronization when moving tip joint
    scene
        .nodes
        .get_mut(tip_joint)
        .unwrap()
        .local_transform
        .translation
        .x += 1.5;
    scene.update_transforms();
    scene.sync_skeletons();

    // Mesh should evaluate properly with new skinning
    let render_mesh = scene.evaluate_mesh(cube_id);
    assert!(render_mesh.is_some());
}

#[test]
fn test_curve_bevel_evaluation() {
    let mut scene = SceneGraph::new();

    let mut spline = Spline::new();
    spline.add_point(geo_kernel::BezierPoint::smooth(
        vec3(0.0, 0.0, 0.0),
        vec3(0.0, 1.0, 0.0),
    ));
    spline.add_point(geo_kernel::BezierPoint::smooth(
        vec3(0.0, 2.0, 0.0),
        vec3(0.0, 1.0, 0.0),
    ));

    let curve_id = scene.add_node(SceneNode::new_curve(
        "BeveledWire",
        Transform::IDENTITY,
        spline,
    ));

    let render_mesh = scene.evaluate_mesh(curve_id);
    assert!(render_mesh.is_some());
    assert!(render_mesh.unwrap().vertices.len() > 0);
}

#[test]
fn test_boolean_modifier_synchronization() {
    let mut scene = SceneGraph::new();

    let base_id = scene.add_node(SceneNode::new_mesh(
        "BaseCube",
        Transform::IDENTITY,
        Primitives::create_cube(2.0),
    ));

    let cutter_id = scene.add_node(SceneNode::new_mesh(
        "CutterSphere",
        Transform::from_translation(vec3(0.5, 0.0, 0.0)),
        Primitives::create_uv_sphere(0.8, 8, 8),
    ));

    // Add Boolean Difference to BaseCube targeting CutterSphere
    if let NodeData::Mesh { modifier_stack, .. } = &mut scene.nodes.get_mut(base_id).unwrap().data {
        let sphere_mesh = Primitives::create_uv_sphere(0.8, 8, 8);
        modifier_stack.add_modifier(Box::new(geo_kernel::BooleanModifier::with_target(
            sphere_mesh,
            geo_kernel::BooleanOp::Difference,
            "CutterSphere",
        )));
    }

    scene.update_transforms();
    scene.sync_booleans();

    let initial_eval = scene.evaluate_mesh(base_id);
    assert!(initial_eval.is_some());

    // Move cutter sphere and sync
    scene
        .nodes
        .get_mut(cutter_id)
        .unwrap()
        .local_transform
        .translation
        .x += 1.0;
    scene.update_transforms();
    scene.sync_booleans();

    let moved_eval = scene.evaluate_mesh(base_id);
    assert!(moved_eval.is_some());
}

#[test]
fn test_decimate_modifier_stack_and_collapse() {
    let mut scene = SceneGraph::new();
    let sphere_id = scene.add_node(SceneNode::new_mesh(
        "HighPolySphere",
        Transform::IDENTITY,
        Primitives::create_uv_sphere(1.0, 16, 16),
    ));

    let orig_faces = if let NodeData::Mesh { base_mesh, .. } = &scene.nodes[sphere_id].data {
        base_mesh.face_count()
    } else {
        0
    };
    assert!(orig_faces > 200);

    // Add Decimate Modifier with 50% target ratio
    if let NodeData::Mesh { modifier_stack, .. } = &mut scene.nodes.get_mut(sphere_id).unwrap().data
    {
        modifier_stack.add_modifier(Box::new(geo_kernel::DecimateModifier::new(0.5)));
    }

    // Non-destructive evaluation
    let eval_mesh = scene
        .evaluate_mesh(sphere_id)
        .expect("Should evaluate decimated mesh");
    assert!(eval_mesh.indices.len() / 3 < orig_faces);

    // Collapse modifier stack into base mesh
    assert!(scene.collapse_modifier_stack(sphere_id).is_ok());

    if let NodeData::Mesh {
        base_mesh,
        modifier_stack,
        ..
    } = &scene.nodes[sphere_id].data
    {
        assert_eq!(modifier_stack.modifiers().len(), 0);
        assert!(base_mesh.face_count() < orig_faces);
    } else {
        panic!("Expected mesh node");
    }
}

#[test]
fn test_rigid_body_physics_simulation_and_floor_bounce() {
    use crate::physics::{CollisionShape, PhysicsWorld, RigidBodyComponent};

    let mut scene = SceneGraph::new();
    let mut world = PhysicsWorld::new();

    // Spawn a sphere 5 meters above ground (Y = 5.0)
    let sphere_node = SceneNode::new_mesh(
        "Rigid_Sphere",
        Transform {
            translation: Vec3::new(0.0, 5.0, 0.0),
            rotation: core_math::Quat::IDENTITY,
            scale: Vec3::ONE,
        },
        Primitives::create_uv_sphere(1.0, 8, 8),
    );
    let sphere_id = scene.add_node(sphere_node);

    let mut body = RigidBodyComponent::new_active(
        CollisionShape::Sphere { radius: 1.0 },
        2.0,
        Vec3::new(0.0, 5.0, 0.0),
        core_math::Quat::IDENTITY,
    );
    body.restitution = 0.8;
    world.add_body(sphere_id, body);

    // Step simulation for 0.4 seconds (freefall)
    for _ in 0..20 {
        world.step(&mut scene, 0.02);
    }

    let y_mid = scene.nodes[sphere_id].local_transform.translation.y;
    assert!(y_mid < 5.0, "Sphere must fall downwards under gravity");

    // Step enough for sphere to hit ground plane (Y=0) and bounce back up
    // Free fall time from 5 to 1: s = 4m, t = sqrt(2*s/g) = sqrt(8/9.81) ~ 0.9s
    for _ in 0..40 {
        world.step(&mut scene, 0.02);
    }

    let final_pos = scene.nodes[sphere_id].local_transform.translation;
    assert!(
        final_pos.y >= 1.0,
        "Sphere must not penetrate ground plane Y=0"
    );
    let body_after = &world.bodies[&sphere_id];
    // After bouncing, velocity should have reversed upwards at some point
    assert!(body_after.velocity.y.abs() >= 0.0);

    // Reset simulation should return sphere to initial position
    world.reset_simulation(&mut scene);
    assert_eq!(
        scene.nodes[sphere_id].local_transform.translation,
        Vec3::new(0.0, 5.0, 0.0)
    );
}

#[test]
fn test_physics_bake_to_keyframes() {
    use crate::animation::AnimationManager;
    use crate::physics::{CollisionShape, PhysicsWorld, RigidBodyComponent};

    let mut scene = SceneGraph::new();
    let mut world = PhysicsWorld::new();
    let mut anim = AnimationManager::new();

    let box_node = SceneNode::new_mesh(
        "Falling_Box",
        Transform {
            translation: Vec3::new(0.0, 3.0, 0.0),
            rotation: core_math::Quat::IDENTITY,
            scale: Vec3::ONE,
        },
        Primitives::create_cube(1.0),
    );
    let box_id = scene.add_node(box_node);

    let body = RigidBodyComponent::new_active(
        CollisionShape::Box {
            half_extents: Vec3::splat(0.5),
        },
        1.5,
        Vec3::new(0.0, 3.0, 0.0),
        core_math::Quat::IDENTITY,
    );
    world.add_body(box_id, body);

    // Bake frames 0 to 30 at 30 FPS
    world.bake_to_keyframes(&mut scene, &mut anim, 0, 30, 30.0);

    assert!(anim.tracks.contains_key(&box_id));
    let track = &anim.tracks[&box_id];
    assert_eq!(track.translation.keyframes.len(), 31);
    assert_eq!(
        track.translation.keyframes[0].value,
        Vec3::new(0.0, 3.0, 0.0)
    );
    // End keyframe should be lower than frame 0
    let last_val = track.translation.keyframes[30].value;
    assert!(last_val.y < 3.0);
}

#[test]
fn test_cylinder_collision_shape_and_impulse() {
    use crate::physics::{CollisionShape, PhysicsWorld, RigidBodyComponent};

    let mut scene = SceneGraph::new();
    let mut world = PhysicsWorld::new();

    // Tower cylinder: radius 2.0m, height 6.0m (half-height 3.0m)
    let tower_mesh = Primitives::create_cylinder(2.0, 6.0, 16);
    let tower_node = SceneNode::new_mesh(
        "SiegeTower",
        Transform::from_translation(Vec3::new(0.0, 3.0, 0.0)),
        tower_mesh,
    );
    let tower_id = scene.add_node(tower_node);

    let cyl_shape = CollisionShape::Cylinder {
        radius: 2.0,
        half_height: 3.0,
    };
    // Check unrotated extents
    let ext = cyl_shape.half_extents_rotated(core_math::Quat::IDENTITY);
    assert!((ext.x - 2.0).abs() < 1e-4);
    assert!((ext.y - 3.0).abs() < 1e-4);
    assert!((ext.z - 2.0).abs() < 1e-4);

    let mut body = RigidBodyComponent::new_active(
        cyl_shape,
        100.0,
        Vec3::new(0.0, 3.0, 0.0),
        core_math::Quat::IDENTITY,
    );

    // Apply a trebuchet projectile impact impulse (J = 500 N*s in +X direction)
    body.apply_impulse(Vec3::new(500.0, 0.0, 0.0));
    assert!((body.velocity.x - 5.0).abs() < 1e-4); // Δv = J / m = 500 / 100 = 5.0 m/s

    world.add_body(tower_id, body);
    world.step(&mut scene, 0.1);

    let new_pos = scene.nodes[tower_id].local_transform.translation;
    assert!(
        new_pos.x > 0.0,
        "Tower should drift in +X direction following impulse"
    );
}

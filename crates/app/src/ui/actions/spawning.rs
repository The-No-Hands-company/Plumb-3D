use core_math::{vec3, Transform, Vec3};
use geo_kernel::{BezierPoint, Spline};
use graph_engine::{LightType, NodeData, SceneNode};

use super::UiActions;
use crate::state::AppState;

pub fn handle_spawning(state: &mut AppState, actions: &mut UiActions) {
    if let Some((name, mesh)) = actions.spawn_mesh.take() {
        state.selected_vertices.clear();
        state.selected_edges.clear();
        state.selected_faces.clear();
        if let Some(first_v) = mesh.vertices.keys().next() {
            state.selected_vertices.insert(first_v);
        }
        if let Some(first_e) = mesh.edges.keys().next() {
            state.selected_edges.insert(first_e);
        }
        if let Some(first_f) = mesh.faces.keys().next() {
            state.selected_faces.insert(first_f);
        }
        let id = state.scene.add_node(SceneNode::new_mesh(name, Transform::IDENTITY, mesh));
        state.selected_node = Some(id);
        state.object_colors.insert(id, [0.72, 0.73, 0.75]);
        state.object_materials.insert(id, [0.0, 0.5, 0.0, 0.0]);

        // Connect newly spawned primitive to the interactive F9 operator panel
        let primitive_params = match name {
            "Cube" => Some(crate::types::PrimitiveParams::Box { width: 2.0, height: 2.0, depth: 2.0, seg_x: 1, seg_y: 1, seg_z: 1 }),
            "Plane" => Some(crate::types::PrimitiveParams::Plane { width: 4.0, length: 4.0, sub_x: 4, sub_z: 4 }),
            "Sphere" | "UVSphere" => Some(crate::types::PrimitiveParams::UvSphere { radius: 1.2, segments: 16, rings: 16 }),
            "Cylinder" => Some(crate::types::PrimitiveParams::Cylinder { radius: 1.0, height: 2.5, segments: 16, cap_fill: geo_kernel::CapFillType::Ngon }),
            "Cone" => Some(crate::types::PrimitiveParams::Cone { radius_bottom: 1.2, radius_top: 0.0, height: 2.5, segments: 16, cap_fill: geo_kernel::CapFillType::Ngon }),
            "Torus" => Some(crate::types::PrimitiveParams::Torus { major_radius: 1.5, minor_radius: 0.4, major_segments: 24, minor_segments: 16 }),
            "Pyramid" => Some(crate::types::PrimitiveParams::Pyramid { base_width: 2.0, base_depth: 2.0, height: 2.5 }),
            "Capsule" => Some(crate::types::PrimitiveParams::Capsule { radius: 0.8, cylinder_height: 1.8, segments: 16, hemisphere_rings: 6 }),
            "IcoSphere" => Some(crate::types::PrimitiveParams::IcoSphere { radius: 1.2, subdivisions: 2 }),
            "PlumbBob" => Some(crate::types::PrimitiveParams::PlumbBob { segments: 24 }),
            _ => None,
        };

        if let Some(params) = primitive_params {
            state.last_operator = Some(crate::types::LastOperatorInfo::AddPrimitive {
                node_id: id,
                params,
            });
            state.show_last_operator_panel = true;
        }
    }

    // Handle Spawn Light
    if let Some((name, l_type)) = actions.spawn_light.take() {
        state.selected_vertices.clear();
        state.selected_edges.clear();
        state.selected_faces.clear();
        let trans = Transform::from_translation(Vec3::new(2.0, 4.0, 3.0));
        let node = match l_type {
            LightType::Directional => SceneNode::new_directional_light(name, trans, Vec3::new(1.0, 0.98, 0.95), 2.5),
            LightType::Point => SceneNode::new_point_light(name, trans, Vec3::new(1.0, 0.92, 0.8), 20.0, 25.0),
            LightType::Spot => SceneNode::new_spot_light(name, trans, Vec3::new(1.0, 1.0, 1.0), 30.0, 30.0, 25.0_f32.to_radians(), 45.0_f32.to_radians()),
        };
        let id = state.scene.add_node(node);
        state.selected_node = Some(id);
    }

    // Handle Spawn Camera
    if actions.spawn_camera {
        state.selected_vertices.clear();
        state.selected_edges.clear();
        state.selected_faces.clear();
        let trans = Transform::from_translation(Vec3::new(0.0, 2.5, 6.0));
        let node = SceneNode::new_camera("Camera", trans, 45.0_f32.to_radians(), state.camera.aspect);
        let id = state.scene.add_node(node);
        state.selected_node = Some(id);
    }

    // Handle Spawn Skeleton Joint / Bone
    if actions.spawn_joint {
        state.selected_vertices.clear();
        state.selected_edges.clear();
        state.selected_faces.clear();
        let trans = Transform::from_translation(Vec3::new(0.0, 0.0, 0.0));
        let joint_count = state.scene.nodes.values().filter(|n| matches!(n.data, NodeData::Joint { .. })).count();
        let name = format!("Bone_{}", joint_count + 1);
        let node = SceneNode::new_joint(name, trans, 2.0, 0.12);
        let id = state.scene.add_node(node);
        state.selected_node = Some(id);
    }

    if let Some((parent_joint_id, length)) = actions.add_child_joint.take() {
        state.selected_vertices.clear();
        state.selected_edges.clear();
        state.selected_faces.clear();
        let child_trans = Transform::from_translation(Vec3::new(0.0, length, 0.0));
        let joint_count = state.scene.nodes.values().filter(|n| matches!(n.data, NodeData::Joint { .. })).count();
        let name = format!("Bone_{}", joint_count + 1);
        let child_node = SceneNode::new_joint(name, child_trans, (length * 0.8).max(0.5), 0.1);
        let child_id = state.scene.add_node(child_node);
        state.scene.set_parent(child_id, Some(parent_joint_id));
        state.selected_node = Some(child_id);
        state.scene.update_transforms();
    }

    if let Some((mesh_id, root_joint_id)) = actions.bind_armature_request.take() {
        if let Err(e) = state.scene.auto_bind_mesh_to_armature(mesh_id, root_joint_id) {
            eprintln!("Armature binding error: {}", e);
        }
    }

    // Handle Spawn Bezier Curve
    if let Some(curve_type) = actions.spawn_curve.take() {
        state.selected_vertices.clear();
        state.selected_edges.clear();
        state.selected_faces.clear();
        let trans = Transform::IDENTITY;
        let curve_count = state.scene.nodes.values().filter(|n| matches!(n.data, NodeData::Curve { .. })).count();
        let (name, spline) = match curve_type {
            0 => {
                let mut s = Spline::new();
                s.add_point(BezierPoint::smooth(vec3(-2.0, 0.0, 0.0), vec3(0.0, 1.0, 0.0)));
                s.add_point(BezierPoint::smooth(vec3(0.0, 1.5, 0.0), vec3(1.0, 0.0, 0.0)));
                s.add_point(BezierPoint::smooth(vec3(2.0, 0.0, 0.0), vec3(0.0, -1.0, 0.0)));
                (format!("BezierCurve_{}", curve_count + 1), s)
            }
            1 => {
                (format!("BezierCircle_{}", curve_count + 1), Spline::circle(1.5, Vec3::ZERO))
            }
            2 => {
                (format!("HelicalSpiral_{}", curve_count + 1), Spline::helix(1.0, 3.0, 3.0, 12))
            }
            _ => (format!("Curve_{}", curve_count + 1), Spline::new()),
        };
        let node = SceneNode::new_curve(name, trans, spline);
        let id = state.scene.add_node(node);
        state.selected_node = Some(id);
    }

    // Handle Spawn Group
    if actions.spawn_group {
        let group_count = state.scene.nodes.values().filter(|n| matches!(n.data, NodeData::Transform)).count();
        let group_name = format!("Group_{}", group_count + 1);
        let group_id = state.scene.add_node(SceneNode::new_transform(group_name, Transform::IDENTITY));
        if let Some(sel_id) = state.selected_node {
            state.scene.set_parent(sel_id, Some(group_id));
        }
        state.selected_node = Some(group_id);
    }
}

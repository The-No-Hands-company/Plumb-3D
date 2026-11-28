use core_math::{vec3, EulerRot, Quat, Transform};
use geo_kernel::Primitives;
use graph_engine::{Material, NodeData, NodeId, SceneGraph, SceneNode};
use render_viewport::{CameraProjection, ViewportCamera, ViewportShadingMode};

use crate::state::AppState;
use crate::types::*;
use crate::ui::overlays::walkthrough::WalkthroughState;

/// Builds a finished, fully rendered Sci-Fi Storage Crate showcase scene.
pub fn build_scifi_crate_finished_graph() -> (SceneGraph, ViewportCamera, NodeId) {
    let mut scene = SceneGraph::new();

    // 1. Materials Palette
    let mat_gunmetal = Material::new(
        "IndustrialGunmetal",
        [0.18, 0.20, 0.24, 1.0],
        0.88,
        0.32,
        0.0,
        0,
    );
    let mat_floor = Material::new(
        "HangarDeckPlate",
        [0.12, 0.14, 0.16, 1.0],
        0.30,
        0.65,
        0.0,
        2,
    );

    // 2. Base Crate Mesh (Box modeled with insets, extrusions, and live bevel modifier)
    let cube_mesh = Primitives::create_cube(2.0);
    let mut crate_node = SceneNode::new_mesh(
        "SciFi_Cargo_Crate",
        Transform::from_translation(vec3(0.0, 1.0, 0.0)),
        cube_mesh,
    );
    if let NodeData::Mesh {
        ref mut materials,
        ref mut modifier_stack,
        ..
    } = crate_node.data
    {
        materials[0] = mat_gunmetal;
        let bevel = geo_kernel::BevelModifier::new(0.04);
        modifier_stack.add_modifier(Box::new(bevel));
    }
    let hero_crate_id = scene.add_node(crate_node);

    // 3. Studio Hangar Floor
    let floor_mesh = Primitives::create_plane(16.0, 16.0, 1, 1);
    let mut floor_node = SceneNode::new_mesh(
        "Hangar_Floor_Plate",
        Transform::from_translation(vec3(0.0, 0.0, 0.0)),
        floor_mesh,
    );
    if let NodeData::Mesh {
        ref mut materials, ..
    } = floor_node.data
    {
        materials[0] = mat_floor;
    }
    scene.add_node(floor_node);

    // 4. Studio 3-Point Lighting
    let key_light = SceneNode::new_directional_light(
        "Key_SpotLight",
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.65, 0.85, 0.0)),
        vec3(1.0, 0.96, 0.90),
        2.8,
    );
    scene.add_node(key_light);

    let fill_light = SceneNode::new_directional_light(
        "Fill_CoolLight",
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.35, -1.20, 0.0)),
        vec3(0.70, 0.85, 1.0),
        1.2,
    );
    scene.add_node(fill_light);

    let rim_light = SceneNode::new_directional_light(
        "Rim_EdgeLight",
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, 0.50, 2.90, 0.0)),
        vec3(1.0, 0.88, 0.70),
        2.2,
    );
    scene.add_node(rim_light);

    // 5. Cinematic 3/4 Viewport Camera
    let camera = ViewportCamera {
        target: vec3(0.0, 1.0, 0.0),
        distance: 5.2,
        pitch: 0.32,
        yaw: 0.65,
        projection: CameraProjection::Perspective,
        ..Default::default()
    };

    (scene, camera, hero_crate_id)
}

/// Initializes an empty scene canvas for the hands-on "Sci-Fi Storage Crate" box modeling tutorial from scratch.
pub fn build_scifi_crate_tutorial_blank(state: &mut AppState) {
    state.scene = SceneGraph::new();
    state.gpu_mesh_cache.clear();
    state.object_colors.clear();
    state.object_materials.clear();
    state.selected_vertices.clear();
    state.selected_edges.clear();
    state.selected_faces.clear();
    state.selected_node = None;
    state.active_camera_node = None;

    state.camera.target = vec3(0.0, 1.0, 0.0);
    state.camera.distance = 5.2;
    state.camera.pitch = 0.32;
    state.camera.yaw = 0.65;
    state.camera.fov_y = 45.0_f32.to_radians();

    state.interaction_mode = InteractionMode::Object;
    state.viewport_shading = ViewportShadingMode::Studio;
    state.show_wireframe = true;
    state.show_grid = true;
    state.show_outliner = true;
    state.show_inspector = true;
    state.inspector_tab = InspectorTab::Modifiers;

    state.walkthrough = WalkthroughState::scifi_crate();
    state.walkthrough.current_step = 0;
    state.sync_gpu_meshes();
    state.set_status_message(
        "📦 Sci-Fi Storage Crate Tutorial started: Click Step 1 to spawn your default cube!",
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scifi_crate_finished_graph() {
        let (scene, camera, hero_id) = build_scifi_crate_finished_graph();
        assert!(scene.nodes.len() >= 5);
        assert_eq!(camera.projection, CameraProjection::Perspective);

        let hero = scene.nodes.get(hero_id).expect("hero node exists");
        assert_eq!(hero.name, "SciFi_Cargo_Crate");
        if let NodeData::Mesh {
            ref modifier_stack, ..
        } = hero.data
        {
            assert_eq!(modifier_stack.modifiers().len(), 1);
            assert_eq!(modifier_stack.modifiers()[0].name(), "Bevel");
        } else {
            panic!("expected mesh node");
        }
    }

    #[test]
    fn test_scifi_crate_box_modeling_pipeline() {
        let mut scene = SceneGraph::new();
        let cube = Primitives::create_cube(2.0);
        let node = SceneNode::new_mesh("TestCrate", Transform::IDENTITY, cube);
        let id = scene.add_node(node);

        // Find side faces
        let side_faces: Vec<_> = {
            let node = scene.nodes.get(id).unwrap();
            if let NodeData::Mesh { base_mesh, .. } = &node.data {
                base_mesh
                    .faces
                    .iter()
                    .filter(|(_, f)| f.normal.y.abs() < 0.2)
                    .map(|(k, _)| k)
                    .collect()
            } else {
                Vec::new()
            }
        };
        assert_eq!(side_faces.len(), 4);

        // Execute Inset
        let mut history = graph_engine::HistoryManager::new();
        let inset_cmd = Box::new(crate::commands::InsetCommand {
            node_id: id,
            faces: side_faces.clone(),
            factor: 0.82,
            individual: true,
            old_mesh: None,
            created_faces: Vec::new(),
        });
        assert!(history.execute(inset_cmd, &mut scene).is_ok());

        // Execute Extrude
        let extrude_cmd = Box::new(crate::commands::ExtrudeCommand::new(id, side_faces, -0.12));
        assert!(history.execute(extrude_cmd, &mut scene).is_ok());

        // Check undo and redo
        assert!(history.undo(&mut scene).is_ok());
        assert!(history.redo(&mut scene).is_ok());
    }
}

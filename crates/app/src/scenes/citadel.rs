use core_math::{vec3, EulerRot, Quat, Transform};
use geo_kernel::Primitives;
use graph_engine::{Material, NodeData, NodeId, SceneGraph, SceneNode};
use render_viewport::{CameraProjection, ViewportCamera, ViewportShadingMode};

use crate::state::AppState;
use crate::types::*;

/// Builds a fully realized Medieval Citadel & Fortress architectural scene graph and camera setup.
pub fn build_medieval_citadel_graph() -> (SceneGraph, ViewportCamera, NodeId) {
    let mut scene = SceneGraph::new();

    // 1. Materials Palette
    let mat_stone = Material::new("WeatheredGranite", [0.42, 0.44, 0.48, 1.0], 0.15, 0.82, 0.0, 2); // Micro-grid stone pattern
    let mat_timber = Material::new("FortressDarkOak", [0.24, 0.16, 0.10, 1.0], 0.05, 0.68, 0.0, 0);
    let mat_iron = Material::new("ForgedSteel", [0.18, 0.19, 0.22, 1.0], 0.92, 0.35, 0.0, 0);
    let mat_ground = Material::new("CourtyardCobble", [0.12, 0.13, 0.15, 1.0], 0.20, 0.75, 0.0, 2);
    let mat_torch_flame = Material::new("TorchFlame", [1.0, 0.65, 0.18, 1.0], 0.0, 0.0, 6.0, 0);

    // 2. Main Donjon Keep Tower
    // Talus Battered Plinth Base
    let base_mesh = Primitives::create_cylinder(3.2, 2.0, 24);
    let mut base_node = SceneNode::new_mesh(
        "Keep_Talus_Base",
        Transform::from_translation(vec3(0.0, 1.0, 0.0)),
        base_mesh,
    );
    if let NodeData::Mesh { ref mut materials, .. } = base_node.data {
        materials[0] = mat_stone.clone();
    }
    scene.add_node(base_node);

    // Main Cylindrical Tower Shaft
    let tower_mesh = Primitives::create_cylinder(2.6, 7.5, 24);
    let mut tower_node = SceneNode::new_mesh(
        "Keep_Main_Tower",
        Transform::from_translation(vec3(0.0, 5.75, 0.0)),
        tower_mesh,
    );
    if let NodeData::Mesh { ref mut materials, ref mut modifier_stack, .. } = tower_node.data {
        materials[0] = mat_stone.clone();
        // Add non-destructive Bevel modifier to showcase DCC modifier pipeline
        modifier_stack.add_modifier(Box::new(geo_kernel::BevelModifier::new(0.05)));
    }
    let hero_tower_id = scene.add_node(tower_node);

    // Corbel Machicolation Flared Ring (Norman Defensive Projection)
    let corbel_mesh = Primitives::create_cylinder(3.2, 0.8, 24);
    let mut corbel_node = SceneNode::new_mesh(
        "Keep_Machicolation_Ring",
        Transform::from_translation(vec3(0.0, 9.8, 0.0)),
        corbel_mesh,
    );
    if let NodeData::Mesh { ref mut materials, .. } = corbel_node.data {
        materials[0] = mat_stone.clone();
    }
    scene.add_node(corbel_node);

    // Crenellated Battlements & Walkway Platform
    let battlement_mesh = Primitives::create_cylinder(3.25, 0.9, 16);
    let mut battlement_node = SceneNode::new_mesh(
        "Keep_Crenels_Parapet",
        Transform::from_translation(vec3(0.0, 10.65, 0.0)),
        battlement_mesh,
    );
    if let NodeData::Mesh { ref mut materials, .. } = battlement_node.data {
        materials[0] = mat_stone.clone();
    }
    scene.add_node(battlement_node);

    // Conical Timber Watchtower Roof
    let roof_mesh = Primitives::create_cone(3.4, 3.5, 24);
    let mut roof_node = SceneNode::new_mesh(
        "Keep_Timber_Roof",
        Transform::from_translation(vec3(0.0, 12.8, 0.0)),
        roof_mesh,
    );
    if let NodeData::Mesh { ref mut materials, .. } = roof_node.data {
        materials[0] = mat_timber.clone();
    }
    scene.add_node(roof_node);

    // Weather-Vane Iron Finial Peak
    let finial_mesh = Primitives::create_cylinder(0.04, 1.2, 8);
    let mut finial_node = SceneNode::new_mesh(
        "Roof_Iron_Finial",
        Transform::from_translation(vec3(0.0, 15.0, 0.0)),
        finial_mesh,
    );
    if let NodeData::Mesh { ref mut materials, .. } = finial_node.data {
        materials[0] = mat_iron.clone();
    }
    scene.add_node(finial_node);

    // 3. Defensive Curtain Walls
    // West Rampart Wall
    let wall_west_mesh = Primitives::create_cube(1.0);
    let mut wall_west_node = SceneNode::new_mesh(
        "Curtain_Wall_West",
        Transform {
            translation: vec3(-5.5, 2.5, 0.0),
            rotation: Quat::IDENTITY,
            scale: vec3(7.0, 5.0, 1.4),
        },
        wall_west_mesh,
    );
    if let NodeData::Mesh { ref mut materials, .. } = wall_west_node.data {
        materials[0] = mat_stone.clone();
    }
    scene.add_node(wall_west_node);

    // East Rampart Wall
    let wall_east_mesh = Primitives::create_cube(1.0);
    let mut wall_east_node = SceneNode::new_mesh(
        "Curtain_Wall_East",
        Transform {
            translation: vec3(5.5, 2.5, 0.0),
            rotation: Quat::IDENTITY,
            scale: vec3(7.0, 5.0, 1.4),
        },
        wall_east_mesh,
    );
    if let NodeData::Mesh { ref mut materials, .. } = wall_east_node.data {
        materials[0] = mat_stone.clone();
    }
    scene.add_node(wall_east_node);

    // 4. Corner Bastion Turret
    let bastion_mesh = Primitives::create_cylinder(1.8, 6.5, 20);
    let mut bastion_node = SceneNode::new_mesh(
        "Bastion_East_Turret",
        Transform::from_translation(vec3(9.2, 3.25, 0.0)),
        bastion_mesh,
    );
    if let NodeData::Mesh { ref mut materials, .. } = bastion_node.data {
        materials[0] = mat_stone.clone();
    }
    scene.add_node(bastion_node);

    // 5. Fortified Gatehouse Arch & Portcullis Grille
    let gate_arch_mesh = Primitives::create_cube(1.0);
    let mut gate_arch_node = SceneNode::new_mesh(
        "Gatehouse_Archway",
        Transform {
            translation: vec3(0.0, 2.2, 3.0),
            rotation: Quat::IDENTITY,
            scale: vec3(3.6, 4.4, 1.8),
        },
        gate_arch_mesh,
    );
    if let NodeData::Mesh { ref mut materials, .. } = gate_arch_node.data {
        materials[0] = mat_stone.clone();
    }
    scene.add_node(gate_arch_node);

    let portcullis_mesh = Primitives::create_cube(1.0);
    let mut portcullis_node = SceneNode::new_mesh(
        "Portcullis_Iron_Grille",
        Transform {
            translation: vec3(0.0, 1.8, 3.8),
            rotation: Quat::IDENTITY,
            scale: vec3(1.8, 3.2, 0.15),
        },
        portcullis_mesh,
    );
    if let NodeData::Mesh { ref mut materials, .. } = portcullis_node.data {
        materials[0] = mat_iron.clone();
    }
    scene.add_node(portcullis_node);

    // 6. Wide Courtyard Cobblestone Ground Plinth
    let ground_mesh = Primitives::create_cylinder(22.0, 0.3, 48);
    let mut ground_node = SceneNode::new_mesh(
        "Citadel_Courtyard_Ground",
        Transform::from_translation(vec3(0.0, -0.15, 0.0)),
        ground_mesh,
    );
    if let NodeData::Mesh { ref mut materials, .. } = ground_node.data {
        materials[0] = mat_ground;
    }
    scene.add_node(ground_node);

    // 7. Architectural Emissive Torch Sconces (Left & Right of Gate)
    let torch_l_mesh = Primitives::create_cylinder(0.12, 0.35, 12);
    let mut torch_l_node = SceneNode::new_mesh(
        "Torch_Sconce_Gate_L",
        Transform::from_translation(vec3(-2.1, 2.8, 4.0)),
        torch_l_mesh,
    );
    if let NodeData::Mesh { ref mut materials, .. } = torch_l_node.data {
        materials[0] = mat_torch_flame.clone();
    }
    scene.add_node(torch_l_node);

    let torch_r_mesh = Primitives::create_cylinder(0.12, 0.35, 12);
    let mut torch_r_node = SceneNode::new_mesh(
        "Torch_Sconce_Gate_R",
        Transform::from_translation(vec3(2.1, 2.8, 4.0)),
        torch_r_mesh,
    );
    if let NodeData::Mesh { ref mut materials, .. } = torch_r_node.data {
        materials[0] = mat_torch_flame;
    }
    scene.add_node(torch_r_node);

    // 8. Cinematic Multi-Point Studio & Atmospheric Lighting Rig
    // Moon Key Light (Casting dramatic long shadows across battlements and courtyard)
    let moonlight = SceneNode::new_directional_light(
        "Moonlight_Key",
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.65, 0.85, 0.0)),
        vec3(0.55, 0.72, 0.95),
        2.8,
    );
    scene.add_node(moonlight);

    // Warm Gate Torch Light Left
    let torch_light_l = SceneNode::new_point_light(
        "Torch_Light_Left",
        Transform::from_translation(vec3(-2.1, 3.2, 4.2)),
        vec3(1.0, 0.65, 0.20),
        24.0,
        22.0,
    );
    scene.add_node(torch_light_l);

    // Warm Gate Torch Light Right
    let torch_light_r = SceneNode::new_point_light(
        "Torch_Light_Right",
        Transform::from_translation(vec3(2.1, 3.2, 4.2)),
        vec3(1.0, 0.65, 0.20),
        24.0,
        22.0,
    );
    scene.add_node(torch_light_r);

    // Upper Parapet Walkway Accent Torch
    let parapet_light = SceneNode::new_point_light(
        "Parapet_Torch_Accent",
        Transform::from_translation(vec3(0.0, 11.2, 2.2)),
        vec3(1.0, 0.72, 0.25),
        18.0,
        20.0,
    );
    scene.add_node(parapet_light);

    // Cool Sky Ambient Fill
    let sky_fill = SceneNode::new_point_light(
        "Sky_Ambient_Fill",
        Transform::from_translation(vec3(0.0, 16.0, 0.0)),
        vec3(0.30, 0.42, 0.65),
        10.0,
        60.0,
    );
    scene.add_node(sky_fill);

    // 9. Camera framing
    let camera = ViewportCamera {
        target: vec3(0.0, 5.0, 0.0),
        distance: 22.0,
        pitch: 0.22,
        yaw: 0.65,
        projection: CameraProjection::Perspective,
        ..Default::default()
    };

    (scene, camera, hero_tower_id)
}

/// Builds a fully realized Medieval Citadel & Fortress architectural scene directly in the editor.
/// Showcases authentic multi-mesh hierarchy, stone/wood/iron PBR materials, torchlight lighting,
/// parametric modifier stacks, and heroic cinematic camera framing.
pub fn build_medieval_citadel_scene(state: &mut AppState) {
    let (scene, camera, hero_tower_id) = build_medieval_citadel_graph();

    state.scene = scene;
    state.camera = camera;
    state.gpu_mesh_cache.clear();
    state.object_colors.clear();
    state.object_materials.clear();
    state.selected_vertices.clear();
    state.selected_edges.clear();
    state.selected_faces.clear();

    state.selected_node = Some(hero_tower_id);
    state.interaction_mode = InteractionMode::Edit;
    state.sub_element_level = SubElementLevel::Face;
    state.viewport_shading = ViewportShadingMode::Studio;
    state.show_wireframe = true;
    state.show_grid = true;
    state.show_outliner = true;
    state.inspector_tab = InspectorTab::Modifiers;

    // Configure Production Render Dialog default preset for Castle
    state.production_render.export_path = "docs/images/castle_citadel_hero.png".to_string();
    state.production_render.post_process.enabled = true;
    state.production_render.post_process.bloom_enabled = true;
    state.production_render.post_process.bloom_intensity = 0.45;
    state.production_render.post_process.bloom_threshold = 1.1;
    state.production_render.post_process.vignette_intensity = 0.32;
    state.production_render.post_process.contrast = 1.10;
    state.production_render.post_process.saturation = 1.15;
    state.production_render.post_process.filmic_tonemap = true;

    state.sync_gpu_meshes();
}

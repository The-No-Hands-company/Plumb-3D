use core_math::{vec3, EulerRot, Quat, Transform};
use geo_kernel::Primitives;
use graph_engine::{Material, NodeData, NodeId, SceneGraph, SceneNode};
use render_viewport::{CameraProjection, ViewportCamera, ViewportShadingMode};

use crate::state::AppState;
use crate::types::*;
use crate::ui::overlays::walkthrough::WalkthroughState;

/// Builds the clean, beginner-friendly Lunar Night Sky scene graph and telephoto camera setup.
pub fn build_lunar_night_sky_graph() -> (SceneGraph, ViewportCamera, NodeId, NodeId) {
    let mut scene = SceneGraph::new();

    // 1. Lunar Regolith PBR Material
    // Authentic lunar regolith: low albedo, high roughness (0.85), procedural basalt maria & crater pattern
    let mat_regolith = Material::new(
        "LunarRegolith",
        [0.85, 0.86, 0.88, 1.0],
        0.0,  // Pure dielectric dust
        0.85, // Matte surface
        0.0,  // Non-emissive (reflects sunlight)
        5,    // Marble / turbulent crater noise pattern
    );

    // 2. The Celestial Body: Clean UV Sphere (No complex sculpting required)
    let moon_mesh = Primitives::create_uv_sphere(3.0, 36, 36);
    let mut moon_node = SceneNode::new_mesh(
        "Moon_Body",
        Transform::IDENTITY,
        moon_mesh,
    );
    if let NodeData::Mesh { ref mut materials, .. } = moon_node.data {
        materials[0] = mat_regolith;
    }
    let moon_id = scene.add_node(moon_node);

    // 3. Directional Sun Key Light (Casts solar illumination and grazing terminator shadows)
    let sun_node = SceneNode::new_directional_light(
        "Sun_KeyLight",
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.2, 0.65, 0.0)),
        vec3(1.0, 0.98, 0.95),
        12.0,
    );
    scene.add_node(sun_node);

    // 4. Subtle Earthshine Ambient Fill (Soft nocturnal blue reflection from Earth)
    let earthshine_node = SceneNode::new_directional_light(
        "Earthshine_Fill",
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, 0.1, -0.8, 0.0)),
        vec3(0.12, 0.18, 0.32),
        0.8,
    );
    scene.add_node(earthshine_node);

    // 5. Astrophotography Camera Node (Dedicated Scene Camera)
    let cam_node = SceneNode::new_camera(
        "Astrophotography_Camera",
        Transform::from_translation(vec3(0.0, 0.0, 8.5)),
        40.0_f32.to_radians(),
        16.0 / 9.0,
    );
    let cam_id = scene.add_node(cam_node);

    // 6. Viewport Telephoto Camera Setup (Framed colossally against deep space)
    let camera = ViewportCamera {
        target: vec3(0.0, 0.0, 0.0),
        distance: 8.5,
        pitch: 0.0,
        yaw: 0.0,
        fov_y: 40.0_f32.to_radians(),
        projection: CameraProjection::Perspective,
        ..Default::default()
    };

    (scene, camera, moon_id, cam_id)
}

/// Populates and activates the complete Lunar Night Sky beginner journey in the app state.
pub fn build_lunar_night_sky_scene(state: &mut AppState) {
    let (scene, camera, moon_id, cam_id) = build_lunar_night_sky_graph();
    state.scene = scene;
    state.camera = camera;
    state.selected_node = Some(moon_id);
    state.active_camera_node = Some(cam_id);
    state.interaction_mode = InteractionMode::Object;
    state.viewport_shading = ViewportShadingMode::Pbr;
    state.show_wireframe = false;
    state.show_outliner = true;
    state.show_inspector = true;
    state.inspector_tab = InspectorTab::Materials;

    // Configure Optics & Atmosphere
    state.dof.enabled = true;
    state.dof.focus_distance = 8.5;
    state.dof.f_stop = 4.0;
    state.ssr.enabled = false;

    // Default Production Render Settings (960x540 64 SPP with glowing lunar bloom)
    state.production_render.settings.width = 960;
    state.production_render.settings.height = 540;
    state.production_render.settings.target_samples = 64;
    state.production_render.focus_distance = 8.5;
    state.production_render.use_scene_camera = true;
    state.production_render.post_process.bloom_enabled = true;
    state.production_render.post_process.bloom_threshold = 0.85;
    state.production_render.post_process.bloom_intensity = 0.45;

    // Launch the friendly In-Viewport Guided Walkthrough
    state.walkthrough = WalkthroughState::lunar_night_sky();

    state.sync_gpu_meshes();
    state.set_status_message("🌕 Lunar Night Sky Scene initialized. Follow the in-viewport guide!");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lunar_night_sky_graph_initialization() {
        let (scene, camera, moon_id, cam_id) = build_lunar_night_sky_graph();
        assert_eq!(scene.nodes.len(), 4); // Moon + Sun + Earthshine + Camera
        assert!(scene.nodes.get(moon_id).is_some());
        assert!(scene.nodes.get(cam_id).is_some());
        assert_eq!(camera.target, core_math::Vec3::ZERO);
        assert!((camera.fov_y.to_degrees() - 40.0).abs() < 1e-3);

        let pt_scene = render_pathtracer::PathTracerScene::from_scene_graph(&scene);
        assert!(!pt_scene.directional_lights.is_empty());

        let pt_cam = render_pathtracer::PathtracerCamera::new(
            core_math::vec3(0.0, 0.0, 8.5),
            core_math::Vec3::ZERO,
            core_math::Vec3::Y,
            40.0_f32.to_radians(),
            16.0 / 9.0,
        );
        let center_ray = pt_cam.get_ray(0.5, 0.5, || 0.5);
        let hit = pt_scene.intersect(&center_ray, 1e-4, 1e5);
        assert!(hit.is_some(), "Center ray must hit the moon");

        let mut rng = || 0.5_f32;
        let color = render_pathtracer::integrator::trace_path(&pt_scene, &center_ray, 4, &mut rng);
        assert!(color.length() > 1.0, "Moon surface must be brightly illuminated by sunlight: {:?}", color);

        // Render 160x90 test frame to verify complete render output
        let mut buffer = render_pathtracer::RenderBuffer::new(160, 90);
        buffer.render_sample_pass(&pt_scene, &pt_cam, 4, None);
        assert!(buffer.samples_rendered == 1);
        let has_lit_pixels = buffer.accumulated_radiance.iter().any(|p| p.length() > 1.0);
        assert!(has_lit_pixels, "Render buffer must contain illuminated moon pixels");
    }
}

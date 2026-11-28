use core_math::{vec3, Transform};
use geo_kernel::Primitives;
use graph_engine::{Material, NodeData, SceneGraph, SceneNode};
use render_pathtracer::{camera::PathtracerCamera, renderer::RenderBuffer, scene::PathTracerScene};
use std::time::Instant;

fn main() {
    println!("🎨 Rendering Plumb Bob Hero Splash Art (1280x720 ACES Filmic Studio Render)...");
    let start_time = Instant::now();

    let mut scene = SceneGraph::new();

    // 1. Plumb Bob Mascot (64-segment precision architectural lathe mesh)
    let plumb_mesh = Primitives::create_plumb_bob_detailed(64);
    let mut plumb_node = SceneNode::new_mesh(
        "PlumbBob_Hero",
        Transform::from_translation(vec3(0.0, 0.0, 0.0)),
        plumb_mesh,
    );
    if let NodeData::Mesh {
        ref mut materials, ..
    } = plumb_node.data
    {
        // Rich architectural imperial brass / gold with 88% metallic and 0.20 roughness for silky, luxurious highlights
        materials[0] = Material::new("PlumbGold", [1.0, 0.78, 0.28, 1.0], 0.88, 0.20, 0.0, 0);
    }
    scene.add_node(plumb_node);

    // 2. Suspended Plumb Line (Braided Anthracite Cord)
    let cord_mesh = Primitives::create_cylinder(0.018, 4.0, 24);
    let mut cord_node = SceneNode::new_mesh(
        "PlumbCord",
        Transform::from_translation(vec3(0.0, 4.25, 0.0)),
        cord_mesh,
    );
    if let NodeData::Mesh {
        ref mut materials, ..
    } = cord_node.data
    {
        materials[0] = Material::new("AnthraciteCord", [0.15, 0.16, 0.18, 1.0], 0.1, 0.7, 0.0, 0);
    }
    scene.add_node(cord_node);

    // 3. Dark Reflective Studio Pedestal Floor
    let floor_mesh = Primitives::create_cylinder(8.0, 0.2, 64);
    let mut floor_node = SceneNode::new_mesh(
        "StudioFloor",
        Transform::from_translation(vec3(0.0, -2.45, 0.0)),
        floor_mesh,
    );
    if let NodeData::Mesh {
        ref mut materials, ..
    } = floor_node.data
    {
        materials[0] = Material::new(
            "StudioFloorMat",
            [0.05, 0.06, 0.08, 1.0],
            0.80,
            0.18,
            0.0,
            2,
        );
    }
    scene.add_node(floor_node);

    // 4. Studio Lighting Rig
    // Directional Sun Light (Smooth, noise-free studio key illumination without mesh markers)
    let sun = SceneNode::new_directional_light(
        "StudioSun",
        Transform::from_rotation(core_math::Quat::from_euler(
            core_math::EulerRot::XYZ,
            -0.65,
            0.85,
            0.0,
        )),
        vec3(1.0, 0.94, 0.82),
        4.5,
    );
    scene.add_node(sun);

    // Warm Key Light (shining on front-right chamfer)
    let key_light = SceneNode::new_point_light(
        "KeyLight",
        Transform::from_translation(vec3(4.0, 4.2, 4.2)),
        vec3(1.0, 0.95, 0.86),
        36.0,
        50.0,
    );
    scene.add_node(key_light);

    // Front Studio Fill Light
    let fill_light = SceneNode::new_point_light(
        "FillLight",
        Transform::from_translation(vec3(1.0, 2.0, 5.5)),
        vec3(0.95, 0.90, 0.82),
        18.0,
        50.0,
    );
    scene.add_node(fill_light);

    // Electric Cyan Rim Light (Placed directly behind the widest body of the plumb bob so it is occluded by the mascot)
    let rim_light = SceneNode::new_point_light(
        "RimLight",
        Transform::from_translation(vec3(-0.1, 0.3, -1.8)),
        vec3(0.3, 0.85, 1.0),
        24.0,
        50.0,
    );
    scene.add_node(rim_light);

    // Build ray tracing scene & configure dark moody DCC studio environment
    let pt_scene = PathTracerScene::from_scene_graph(&scene).with_background(
        vec3(0.040, 0.048, 0.072),
        vec3(0.024, 0.028, 0.040),
        vec3(0.012, 0.014, 0.020),
    );
    println!(
        "✓ BVH constructed successfully (emissive triangles: {})",
        pt_scene.emissive_triangles.len()
    );

    // 5. Camera: 1280x720 16:9 Cinematic Widescreen Framing
    let width = 1280;
    let height = 720;
    let look_from = vec3(4.2, 1.4, 6.2);
    let look_at = vec3(0.0, 0.15, 0.0);
    let vup = vec3(0.0, 1.0, 0.0);
    let vfov = 38.0_f32.to_radians();
    let aspect = width as f32 / height as f32;
    let focus_dist = (look_from - look_at).length();
    let aperture = 0.005; // Sharp focal plane with creamy background bokeh

    let camera = PathtracerCamera::new_thin_lens(
        look_from, look_at, vup, vfov, aspect, aperture, focus_dist,
    );

    let mut buffer = RenderBuffer::new(width, height);
    let target_samples = 144;

    for s in 0..target_samples {
        buffer.render_sample_pass(&pt_scene, &camera, 5, None);
        if (s + 1) % 16 == 0 || s + 1 == target_samples {
            print!(
                "\rRendering pass: {:3}/{} samples ({:5.1}%)",
                s + 1,
                target_samples,
                ((s + 1) as f32 / target_samples as f32) * 100.0
            );
            std::io::Write::flush(&mut std::io::stdout()).unwrap();
        }
    }
    println!();

    let output_path = "docs/images/plumb_bob_splash.png";
    buffer
        .save_png_filmic(output_path, true)
        .expect("Save PNG render with ACES Filmic tone mapping");
    println!(
        "✨ Ultra-high-end hero render saved successfully to: {} (Elapsed: {:.2?})",
        output_path,
        start_time.elapsed()
    );
}

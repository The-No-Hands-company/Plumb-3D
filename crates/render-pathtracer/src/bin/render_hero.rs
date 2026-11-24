use core_math::{vec3, EulerRot, Quat, Transform};
use geo_kernel::Primitives;
use graph_engine::{Material, NodeData, SceneGraph, SceneNode};
use render_pathtracer::{
    camera::PathtracerCamera,
    renderer::RenderBuffer,
    scene::PathTracerScene,
};
use std::time::Instant;

fn main() {
    println!("🎨 Rendering Plumb Bob Hero Splash Art with Monte Carlo Path Tracer...");
    let start_time = Instant::now();

    let mut scene = SceneGraph::new();

    // 1. Plumb Bob Mascot (Architectural Brass / Gold)
    let plumb_mesh = Primitives::create_plumb_bob();
    let mut plumb_node = SceneNode::new_mesh(
        "PlumbBob_Hero",
        Transform::from_translation(vec3(0.0, 0.2, 0.0)),
        plumb_mesh,
    );
    if let NodeData::Mesh { ref mut materials, .. } = plumb_node.data {
        let mat = Material::new("PlumbGold", [1.0, 0.80, 0.38, 1.0], 0.95, 0.16, 0.0, 0);
        materials.push(mat);
    }
    scene.add_node(plumb_node);

    // 2. Suspended Plumb Line (Braided Cord)
    let string_mesh = Primitives::create_cylinder(0.02, 3.0, 12);
    let mut string_node = SceneNode::new_mesh(
        "PlumbLine",
        Transform::from_translation(vec3(0.0, 3.8, 0.0)),
        string_mesh,
    );
    if let NodeData::Mesh { ref mut materials, .. } = string_node.data {
        let mat = Material::new("BraidedCord", [0.2, 0.22, 0.25, 1.0], 0.05, 0.8, 0.0, 0);
        materials.push(mat);
    }
    scene.add_node(string_node);

    // 3. Studio Pedestal Base
    let base_mesh = Primitives::create_cylinder(3.5, 0.3, 32);
    let mut base_node = SceneNode::new_mesh(
        "Pedestal",
        Transform::from_translation(vec3(0.0, -2.2, 0.0)),
        base_mesh,
    );
    if let NodeData::Mesh { ref mut materials, .. } = base_node.data {
        let mat = Material::new("DarkPedestal", [0.08, 0.09, 0.12, 1.0], 0.3, 0.25, 0.0, 3);
        materials.push(mat);
    }
    scene.add_node(base_node);

    // 4. Studio 3-Point Lighting
    // Key Light (Warm Sunlight)
    let key_light = SceneNode::new_point_light(
        "KeyLight",
        Transform::from_translation(vec3(4.0, 5.0, 4.0)),
        vec3(1.0, 0.94, 0.86),
        18.0,
        50.0,
    );
    scene.add_node(key_light);

    // Fill Light (Cool Soft Sky)
    let fill_light = SceneNode::new_point_light(
        "FillLight",
        Transform::from_translation(vec3(-5.0, 3.0, 2.0)),
        vec3(0.65, 0.8, 1.0),
        8.0,
        50.0,
    );
    scene.add_node(fill_light);

    // Rim / Edge Light (High intensity backlight for gold silhouette)
    let rim_light = SceneNode::new_point_light(
        "RimLight",
        Transform::from_translation(vec3(0.0, 4.0, -4.5)),
        vec3(1.0, 0.85, 0.5),
        25.0,
        50.0,
    );
    scene.add_node(rim_light);

    // Physical Sun
    let sun = SceneNode::new_directional_light(
        "Sun",
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.8, 0.6, 0.0)),
        vec3(1.0, 0.95, 0.85),
        2.5,
    );
    scene.add_node(sun);

    // Build ray tracing scene & BVH
    let pt_scene = PathTracerScene::from_scene_graph(&scene);
    println!("✓ BVH constructed successfully for hero scene");

    // Configure camera: dramatic 3/4 perspective looking at the plumb bob
    let width = 720;
    let height = 480;
    let look_from = vec3(2.8, 0.8, 4.2);
    let look_at = vec3(0.0, 0.1, 0.0);
    let vup = vec3(0.0, 1.0, 0.0);
    let vfov = 38.0_f32.to_radians();
    let aspect = width as f32 / height as f32;
    let focus_dist = (look_from - look_at).length();
    let aperture = 0.025; // Subtle bokeh

    let camera = PathtracerCamera::new_thin_lens(
        look_from,
        look_at,
        vup,
        vfov,
        aspect,
        aperture,
        focus_dist,
    );

    let mut buffer = RenderBuffer::new(width, height);
    let target_samples = 96;

    for s in 0..target_samples {
        buffer.render_sample_pass(&pt_scene, &camera, 4, None);
        if (s + 1) % 16 == 0 || s + 1 == target_samples {
            print!("\rRendering pass: {}/{} samples ({:.1}%)", s + 1, target_samples, ((s + 1) as f32 / target_samples as f32) * 100.0);
            std::io::Write::flush(&mut std::io::stdout()).unwrap();
        }
    }
    println!();

    let output_path = "docs/images/plumb_bob_splash.png";
    buffer.save_png(output_path).expect("Save PNG render");
    println!("✨ Hero render saved successfully to: {} (Elapsed: {:.2?})", output_path, start_time.elapsed());
}

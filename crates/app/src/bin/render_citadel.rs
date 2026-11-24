use app::scenes::citadel::build_medieval_citadel_graph;
use core_math::vec3;
use render_pathtracer::{
    camera::PathtracerCamera,
    post_process::PostProcessSettings,
    renderer::RenderBuffer,
    scene::PathTracerScene,
};
use std::time::Instant;

fn main() {
    println!("🏰 Rendering Medieval Citadel & Fortress Architectural Scene...");
    let start_time = Instant::now();

    let (mut scene, camera_state, _) = build_medieval_citadel_graph();

    // 1. Evaluate all mesh modifiers (e.g. Bevel modifier on Keep_Main_Tower)
    let node_ids: Vec<_> = scene.nodes.keys().collect();
    for id in node_ids {
        scene.evaluate_mesh(id);
    }

    // 2. Build BVH with atmospheric dusk/night sky environment
    let pt_scene = PathTracerScene::from_scene_graph(&scene)
        .with_background(
            vec3(0.018, 0.032, 0.065), // Sapphire twilight zenith
            vec3(0.035, 0.045, 0.080), // Moody dusk horizon
            vec3(0.010, 0.012, 0.018), // Deep courtyard shadow ground
        );

    println!(
        "✓ BVH constructed successfully (emissive triangles: {})",
        pt_scene.emissive_triangles.len()
    );

    // 3. Cinematic Ultrawide Banner Camera (1600x480, 3.33:1 panoramic aspect ratio)
    let width = 1600;
    let height = 480;
    let look_from = camera_state.eye_position();
    let look_at = camera_state.target;
    let vup = vec3(0.0, 1.0, 0.0);
    let vfov = camera_state.fov_y;
    let aspect = width as f32 / height as f32;
    let focus_dist = (look_from - look_at).length();
    let aperture = 0.008; // Crisp architectural depth across wide panoramic frame

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
    let target_samples = 128;

    for s in 0..target_samples {
        buffer.render_sample_pass(&pt_scene, &camera, 6, None);
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

    // 4. Post-Production LookDev Pipeline
    let post_process = PostProcessSettings {
        enabled: true,
        bloom_enabled: true,
        bloom_intensity: 0.42,
        bloom_threshold: 1.05,
        bloom_radius: 12,
        vignette_intensity: 0.32,
        exposure: 1.15,
        contrast: 1.12,
        saturation: 1.18,
        filmic_tonemap: true,
    };

    let output_path = "docs/images/castle_citadel_hero.png";
    buffer
        .save_png_post_processed(output_path, &post_process)
        .expect("Save post-processed PNG render");

    println!(
        "✨ Medieval Citadel hero render saved to: {} (Elapsed: {:.2?})",
        output_path,
        start_time.elapsed()
    );
}

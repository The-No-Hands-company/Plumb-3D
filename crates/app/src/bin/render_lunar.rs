use app::scenes::lunar_night_sky::build_lunar_night_sky_graph;
use core_math::vec3;
use render_pathtracer::{
    camera::PathtracerCamera, post_process::PostProcessSettings, renderer::RenderBuffer,
    scene::PathTracerScene,
};
use std::time::Instant;

fn main() {
    println!("🌕 Rendering Lunar Night Sky & Full Moon Astrophotography Scene...");
    let start_time = Instant::now();

    let (scene, camera_state, _, _) = build_lunar_night_sky_graph();

    let pt_scene = PathTracerScene::from_scene_graph(&scene).with_background(
        vec3(0.001, 0.002, 0.004),
        vec3(0.002, 0.003, 0.006),
        vec3(0.0, 0.0, 0.0),
    );

    println!(
        "✓ BVH constructed successfully (directional lights: {})",
        pt_scene.directional_lights.len()
    );

    let width = 960;
    let height = 540;
    let look_from = camera_state.eye_position();
    let look_at = camera_state.target;
    let vup = vec3(0.0, 1.0, 0.0);
    let vfov = camera_state.fov_y;
    let aspect = width as f32 / height as f32;

    let camera = PathtracerCamera::new(look_from, look_at, vup, vfov, aspect);

    let mut buffer = RenderBuffer::new(width, height);
    let target_samples = 32;

    for s in 0..target_samples {
        buffer.render_sample_pass(&pt_scene, &camera, 4, None);
        if (s + 1) % 8 == 0 || s + 1 == target_samples {
            print!(
                "\rRendering pass: {:2}/{} samples ({:5.1}%)",
                s + 1,
                target_samples,
                ((s + 1) as f32 / target_samples as f32) * 100.0
            );
            std::io::Write::flush(&mut std::io::stdout()).unwrap();
        }
    }
    println!();

    let post_process = PostProcessSettings {
        enabled: true,
        bloom_enabled: true,
        bloom_intensity: 0.15,
        bloom_threshold: 1.35,
        bloom_radius: 8,
        vignette_intensity: 0.25,
        chromatic_aberration: 0.0,
        temperature: -0.02,
        exposure: 0.85,
        contrast: 1.15,
        saturation: 1.05,
        filmic_tonemap: true,
        ..Default::default()
    };

    let output_path = "render_output.png";
    buffer
        .save_png_post_processed(output_path, &post_process)
        .expect("Save calibrated render PNG");

    println!(
        "✨ Calibrated Lunar render saved to: {} (Elapsed: {:.2?})",
        output_path,
        start_time.elapsed()
    );
}

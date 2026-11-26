#[cfg(test)]
mod tests {
    use crate::bvh::{BvhNode, Triangle};
    use crate::camera::PathtracerCamera;
    use crate::material::Material;
    use crate::renderer::RenderBuffer;
    use crate::scene::PathTracerScene;
    use core_math::{Ray, Vec2, Vec3};

    #[test]
    fn test_ray_triangle_intersection() {
        let tri = Triangle::new(
            Vec3::new(-1.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 2.0, 0.0),
            Vec3::Z,
            Vec3::Z,
            Vec3::Z,
            Vec2::ZERO,
            Vec2::X,
            Vec2::Y,
            0,
        );

        let ray_hit = Ray::new(Vec3::new(0.0, 0.5, 5.0), -Vec3::Z);
        let hit = tri.intersect(&ray_hit, 0.001, 100.0);
        assert!(hit.is_some());
        let h = hit.unwrap();
        assert!((h.t - 5.0).abs() < 1e-4);
        assert_eq!(h.material_id, 0);

        let ray_miss = Ray::new(Vec3::new(5.0, 5.0, 5.0), -Vec3::Z);
        assert!(tri.intersect(&ray_miss, 0.001, 100.0).is_none());
    }

    #[test]
    fn test_bvh_construction_and_traversal() {
        let mut triangles = Vec::new();
        // Create 10 offset triangles
        for i in 0..10 {
            let offset = i as f32 * 2.0;
            triangles.push(Triangle::new(
                Vec3::new(offset, 0.0, 0.0),
                Vec3::new(offset + 1.0, 0.0, 0.0),
                Vec3::new(offset, 1.0, 0.0),
                Vec3::Z,
                Vec3::Z,
                Vec3::Z,
                Vec2::ZERO,
                Vec2::X,
                Vec2::Y,
                i,
            ));
        }

        let bvh = BvhNode::build(triangles, 2);
        // Shoot ray at triangle 4 (x = 8.2, y = 0.2)
        let ray = Ray::new(Vec3::new(8.2, 0.2, 5.0), -Vec3::Z);
        let hit = bvh.intersect(&ray, 0.001, 100.0);
        assert!(hit.is_some());
        assert_eq!(hit.unwrap().material_id, 4);
    }

    #[test]
    fn test_progressive_render_convergence() {
        let tri1 = Triangle::new(
            Vec3::new(-2.0, -1.0, -2.0),
            Vec3::new(2.0, -1.0, -2.0),
            Vec3::new(2.0, -1.0, 2.0),
            Vec3::Y,
            Vec3::Y,
            Vec3::Y,
            Vec2::ZERO,
            Vec2::X,
            Vec2::ONE,
            0,
        );
        let tri2 = Triangle::new(
            Vec3::new(-2.0, -1.0, -2.0),
            Vec3::new(2.0, -1.0, 2.0),
            Vec3::new(-2.0, -1.0, 2.0),
            Vec3::Y,
            Vec3::Y,
            Vec3::Y,
            Vec2::ZERO,
            Vec2::ONE,
            Vec2::Y,
            0,
        );

        let materials = vec![
            Material::lambertian(Vec3::new(0.8, 0.2, 0.2)),
            Material::Emissive { radiance: Vec3::splat(10.0) },
        ];

        let scene = PathTracerScene::new(vec![tri1, tri2], materials);
        let camera = PathtracerCamera::new(
            Vec3::new(0.0, 2.0, 5.0),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::Y,
            std::f32::consts::PI / 4.0,
            1.0,
        );

        let mut buffer = RenderBuffer::new(32, 32);
        for _ in 0..4 {
            let ok = buffer.render_sample_pass(&scene, &camera, 3, None);
            assert!(ok);
        }

        assert_eq!(buffer.samples_rendered, 4);
        let rgba = buffer.to_rgba8();
        assert_eq!(rgba.len(), 32 * 32 * 4);
        assert!(rgba.iter().any(|&b| b > 0));
    }

    #[test]
    fn test_thin_lens_depth_of_field() {
        let look_from = Vec3::new(0.0, 0.0, 5.0);
        let look_at = Vec3::ZERO;
        let vup = Vec3::Y;
        let cam_pinhole = PathtracerCamera::new(look_from, look_at, vup, 0.785, 1.0);
        assert_eq!(cam_pinhole.lens_radius, 0.0);

        let ray1 = cam_pinhole.get_ray(0.5, 0.5, || 0.5);
        assert_eq!(ray1.origin, look_from);

        let cam_dof = PathtracerCamera::new_thin_lens(look_from, look_at, vup, 0.785, 1.0, 0.5, 5.0);
        assert!((cam_dof.lens_radius - 0.25).abs() < 1e-5);
        assert_eq!(cam_dof.focus_distance, 5.0);

        let ray_dof = cam_dof.get_ray(0.5, 0.5, || 0.75);
        // Thin-lens ray origin is jittered on the lens aperture disk
        assert!((ray_dof.origin - look_from).length() > 0.0);
    }

    #[test]
    fn test_procedural_texture_evaluation() {
        use crate::material::TextureSource;

        let col_a = Vec3::new(1.0, 0.0, 0.0);
        let col_b = Vec3::new(0.0, 0.0, 1.0);
        let checker = TextureSource::Checkerboard {
            color_a: col_a,
            color_b: col_b,
            scale: 2.0,
        };

        // UV (0.1, 0.1) -> index (0, 0) -> sum 0 -> color_a
        assert_eq!(checker.evaluate(Vec2::new(0.1, 0.1), Vec3::ZERO, Vec3::Y), col_a);
        // UV (0.6, 0.1) -> index (1, 0) -> sum 1 -> color_b
        assert_eq!(checker.evaluate(Vec2::new(0.6, 0.1), Vec3::ZERO, Vec3::Y), col_b);

        let rings = TextureSource::Rings {
            color_a: col_a,
            color_b: col_b,
            scale: 4.0,
        };
        let eval_ring = rings.evaluate(Vec2::splat(0.5), Vec3::ZERO, Vec3::Y);
        // Center of rings (distance 0) should evaluate to mix based on sin(0) = 0.5
        assert!((eval_ring.x - 0.5).abs() < 1e-4);
        assert!((eval_ring.z - 0.5).abs() < 1e-4);
    }

    #[test]
    fn test_texture_map_baking_normal_and_ao() {
        use geo_kernel::Primitives;
        use crate::baker::{bake_texture, BakeMapType, BakeSettings};

        let low_poly = Primitives::create_cube(2.0);
        let high_poly = Primitives::create_uv_sphere(1.0, 16, 16);

        let settings = BakeSettings {
            map_type: BakeMapType::TangentNormal,
            width: 64,
            height: 64,
            cage_distance: 0.5,
            ao_samples: 8,
        };

        let normal_img = bake_texture(&low_poly, &high_poly, &settings);
        assert_eq!(normal_img.width(), 64);
        assert_eq!(normal_img.height(), 64);
        assert_eq!(normal_img.pixels().count(), 64 * 64);

        // Test AO baking
        let mut ao_settings = settings;
        ao_settings.map_type = BakeMapType::AmbientOcclusion;
        let ao_img = bake_texture(&low_poly, &high_poly, &ao_settings);
        assert_eq!(ao_img.width(), 64);
    }

    #[test]
    fn test_principled_bsdf_evaluation_and_sampling() {
        use crate::material::Material;
        use core_math::Ray;

        // 1. Metallic Principled Material
        let gold_mat = Material::principled(
            Vec3::new(1.0, 0.76, 0.33),
            1.0,  // Metallic
            0.05, // Smooth roughness
            1.5,
            0.0,
            Vec3::ZERO,
        );
        assert!(!gold_mat.is_emissive());
        assert_eq!(gold_mat.emission(), Vec3::ZERO);

        let mut seed = 123456789_u32;
        let mut rng = move || {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            (seed >> 8) as f32 / 16777216.0
        };
        let ray_in = Ray::new(Vec3::new(-1.0, 1.0, 0.0), Vec3::new(1.0, -1.0, 0.0).normalize());
        let sample = gold_mat.sample(&ray_in, Vec3::ZERO, Vec3::Y, Vec2::ZERO, &mut rng);
        assert!(sample.is_some());
        let bsdf = sample.unwrap();
        assert!(bsdf.is_specular);
        assert!(bsdf.scattered_ray.direction.y > 0.0);
        // Attenuation should retain golden tint
        assert!(bsdf.attenuation.x > bsdf.attenuation.z);

        // 2. Emissive Principled Material
        let neon_mat = Material::principled(
            Vec3::ONE,
            0.0,
            0.5,
            1.5,
            0.0,
            Vec3::new(5.0, 10.0, 2.0),
        );
        assert!(neon_mat.is_emissive());
        assert_eq!(neon_mat.emission(), Vec3::new(5.0, 10.0, 2.0));

        // 3. Glass / Refractive Principled Material (Transmission = 1.0)
        let glass_mat = Material::principled(
            Vec3::ONE,
            0.0,
            0.0,
            1.52,
            1.0, // Full transmission
            Vec3::ZERO,
        );
        let ray_straight_down = Ray::new(Vec3::new(0.0, 1.0, 0.0), Vec3::new(0.0, -1.0, 0.0));
        let glass_sample = glass_mat.sample(&ray_straight_down, Vec3::ZERO, Vec3::Y, Vec2::ZERO, &mut rng);
        assert!(glass_sample.is_some());
        let glass_bsdf = glass_sample.unwrap();
        assert!(glass_bsdf.is_specular);
    }

    #[test]
    fn test_physical_sky_and_sun_disk_radiance() {
        let scene = PathTracerScene::new(Vec::new(), Vec::new());
        // Default sky: zenith is blue, horizon is brighter, ground is dark
        let zenith_rad = scene.sky_radiance(Vec3::Y);
        let ground_rad = scene.sky_radiance(-Vec3::Y);
        assert!(zenith_rad.z > zenith_rad.x); // Blue tint
        assert!(zenith_rad.length() > ground_rad.length());

        // Sky with golden hour Sun
        let sun_dir = Vec3::new(1.0, 0.2, 0.0).normalize();
        let sun_scene = scene.with_sun(sun_dir, Vec3::new(1.0, 0.8, 0.5), 2.0);

        // Looking directly into solar disk
        let sun_disk_rad = sun_scene.sky_radiance(sun_dir);
        assert!(sun_disk_rad.length() > 20.0); // Intense solar disk

        // Forward haze aureole
        let near_sun_dir = (sun_dir + Vec3::new(0.0, 0.05, 0.0)).normalize();
        let aureole_rad = sun_scene.sky_radiance(near_sun_dir);
        assert!(aureole_rad.length() > zenith_rad.length());
    }

    #[test]
    fn test_compositor_chromatic_aberration_and_temperature() {
        use crate::post_process::{apply_post_processing, PostProcessSettings};

        let width = 16;
        let height = 16;
        let white = Vec3::splat(1.0);
        let radiance = vec![white; (width * height) as usize];

        // 1. Warm grading test
        let mut warm_settings = PostProcessSettings::default();
        warm_settings.bloom_enabled = false;
        warm_settings.vignette_intensity = 0.0;
        warm_settings.temperature = 1.0; // Warm shift
        let warm_bytes = apply_post_processing(&radiance, width, height, 1, &warm_settings);
        let center_idx = (8 * 16 + 8) * 4;
        let warm_r = warm_bytes[center_idx];
        let warm_b = warm_bytes[center_idx + 2];
        assert!(warm_r > warm_b, "Warm grading should boost red over blue: r={}, b={}", warm_r, warm_b);

        // 2. Cool grading test
        let mut cool_settings = PostProcessSettings::default();
        cool_settings.bloom_enabled = false;
        cool_settings.vignette_intensity = 0.0;
        cool_settings.temperature = -1.0; // Cool shift
        let cool_bytes = apply_post_processing(&radiance, width, height, 1, &cool_settings);
        let cool_r = cool_bytes[center_idx];
        let cool_b = cool_bytes[center_idx + 2];
        assert!(cool_b > cool_r, "Cool grading should boost blue over red: r={}, b={}", cool_r, cool_b);

        // 3. Chromatic aberration test
        // Place a bright pixel at coordinate (14, 8) near right edge
        let mut pattern_radiance = vec![Vec3::ZERO; (width * height) as usize];
        pattern_radiance[8 * 16 + 14] = Vec3::splat(10.0);

        let mut ca_settings = PostProcessSettings::default();
        ca_settings.bloom_enabled = false;
        ca_settings.vignette_intensity = 0.0;
        ca_settings.filmic_tonemap = false;
        ca_settings.chromatic_aberration = 1.0;
        let ca_bytes = apply_post_processing(&pattern_radiance, width, height, 1, &ca_settings);
        assert_eq!(ca_bytes.len(), (width * height * 4) as usize);
    }
}

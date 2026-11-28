use crate::material::Material;
use crate::scene::PathTracerScene;
use core_math::{Ray, Vec3};

/// Evaluates radiance along a ray through Monte Carlo path tracing.
pub fn trace_path(
    scene: &PathTracerScene,
    initial_ray: &Ray,
    max_bounces: u32,
    rng: &mut impl FnMut() -> f32,
) -> Vec3 {
    let mut ray = *initial_ray;
    let mut throughput = Vec3::ONE;
    let mut accumulated_radiance = Vec3::ZERO;

    for bounce in 0..max_bounces {
        let Some(mut hit) = scene.intersect(&ray, 1e-4, 1e5) else {
            // Missed geometry -> add ambient sky radiance
            accumulated_radiance += throughput * scene.sky_radiance(ray.direction);
            break;
        };

        if scene.bevel_radius > 0.0 {
            hit.normal = crate::bevel_normal::compute_beveled_normal(
                scene,
                hit.position,
                hit.normal,
                scene.bevel_radius,
            );
        }

        let material = &scene.materials[hit.material_id % scene.materials.len()];

        // 1. Direct surface emission
        let emission = material.emission();
        if emission.length_squared() > 0.0 {
            accumulated_radiance += throughput * emission;
            break; // Emissive surfaces terminate the path
        }

        // 2. Next Event Estimation (Direct Area Light, Directional Sunlight & Point Lights)
        // 2a. Emissive Meshes (Area Lights)
        if !scene.emissive_triangles.is_empty() {
            let light_idx = ((rng() * scene.emissive_triangles.len() as f32) as usize)
                .min(scene.emissive_triangles.len() - 1);
            let light_tri = &scene.emissive_triangles[light_idx];

            // Sample point on triangle surface
            let r1 = rng().sqrt();
            let r2 = rng();
            let u = 1.0 - r1;
            let v = r2 * r1;
            let w = 1.0 - u - v;
            let light_pos = light_tri.v0 * u + light_tri.v1 * v + light_tri.v2 * w;

            let to_light = light_pos - hit.position;
            let dist_sq = to_light.length_squared();
            let dist = dist_sq.sqrt();
            let light_dir = to_light / dist.max(1e-4);

            let cos_theta = hit.normal.dot(light_dir);
            let light_normal =
                (light_tri.n0 * u + light_tri.n1 * v + light_tri.n2 * w).normalize_or_zero();
            let cos_light = (-light_dir).dot(light_normal);

            if cos_theta > 0.0 && cos_light > 0.0 {
                let shadow_ray = Ray::new(hit.position + hit.normal * 1e-4, light_dir);
                // Test shadow occlusion
                if let Some(shadow_hit) = scene.intersect(&shadow_ray, 1e-4, dist - 1e-3) {
                    if shadow_hit.material_id == light_tri.material_id {
                        let light_mat =
                            &scene.materials[light_tri.material_id % scene.materials.len()];
                        let light_emission = light_mat.emission();

                        // Geometry term: G = cos_theta * cos_light / dist_sq weighted by triangle area
                        let edge1 = light_tri.v1 - light_tri.v0;
                        let edge2 = light_tri.v2 - light_tri.v0;
                        let area = 0.5 * edge1.cross(edge2).length();

                        let g = (cos_theta * cos_light / dist_sq.max(1e-4)).min(2.0);
                        let light_pdf = 1.0 / scene.emissive_triangles.len() as f32;
                        let albedo = material.albedo(hit.uv, hit.position, hit.normal);
                        accumulated_radiance +=
                            throughput * albedo * light_emission * (g * area) / light_pdf.max(1e-4);
                    }
                }
            }
        }

        let eff_normal = material.perturbed_normal(hit.uv, hit.position, hit.normal);

        // 2b. Directional Lights (Solar / Celestial Illumination)
        for dir_light in &scene.directional_lights {
            let cos_theta = eff_normal.dot(dir_light.to_light);
            if cos_theta > 0.0 {
                let shadow_ray = Ray::new(hit.position + hit.normal * 1e-4, dir_light.to_light);
                // Offset threshold (0.02) eliminates shadow terminator facet self-occlusion
                if scene.intersect(&shadow_ray, 0.02, 1e5).is_none() {
                    let albedo = material.albedo(hit.uv, hit.position, eff_normal);

                    // Micro-facet Oren-Nayar diffuse for rough surfaces (lunar regolith, dust)
                    let rough = match material {
                        Material::Principled { roughness, .. } => *roughness,
                        _ => 0.0,
                    };
                    let diffuse_term = if rough > 0.1 {
                        let sigma2 = rough * rough;
                        let a = 1.0 - 0.5 * (sigma2 / (sigma2 + 0.33));
                        let b = 0.45 * (sigma2 / (sigma2 + 0.09));
                        let view_dir = -ray.direction.normalize();
                        let v_dot_n = hit.normal.dot(view_dir).max(0.0);
                        let l_dot_n = cos_theta;
                        let cos_phi = (view_dir - hit.normal * v_dot_n)
                            .normalize_or_zero()
                            .dot((dir_light.to_light - hit.normal * l_dot_n).normalize_or_zero());
                        let (sin_alpha, tan_beta) = if l_dot_n < v_dot_n {
                            (
                                (1.0 - l_dot_n * l_dot_n).max(0.0).sqrt(),
                                (1.0 - v_dot_n * v_dot_n).max(0.0).sqrt() / v_dot_n.max(1e-4),
                            )
                        } else {
                            (
                                (1.0 - v_dot_n * v_dot_n).max(0.0).sqrt(),
                                (1.0 - l_dot_n * l_dot_n).max(0.0).sqrt() / l_dot_n.max(1e-4),
                            )
                        };
                        cos_theta * (a + b * cos_phi.max(0.0) * sin_alpha * tan_beta)
                    } else {
                        cos_theta
                    };

                    accumulated_radiance += throughput
                        * albedo
                        * dir_light.color
                        * (dir_light.intensity * diffuse_term);
                }
            }
        }

        // 2c. Point Lights (Omni Sources)
        for pt_light in &scene.point_lights {
            let to_light = pt_light.position - hit.position;
            let dist_sq = to_light.length_squared();
            let dist = dist_sq.sqrt();
            let light_dir = to_light / dist.max(1e-4);
            let cos_theta = eff_normal.dot(light_dir);
            if cos_theta > 0.0 {
                let shadow_ray = Ray::new(hit.position + hit.normal * 1e-4, light_dir);
                if scene.intersect(&shadow_ray, 1e-4, dist - 1e-3).is_none() {
                    let d_over_r = (dist / pt_light.range.max(0.1)).clamp(0.0, 1.0);
                    let win = (1.0 - d_over_r * d_over_r * d_over_r * d_over_r).clamp(0.0, 1.0);
                    let att = (win * win) / (dist_sq + 1.0);
                    let albedo = material.albedo(hit.uv, hit.position, eff_normal);
                    accumulated_radiance += throughput
                        * albedo
                        * pt_light.color
                        * (pt_light.intensity * att * cos_theta);
                }
            }
        }

        // 3. Scatter indirect ray using BSDF
        let Some(sample) = material.sample(&ray, hit.position, hit.normal, hit.uv, rng) else {
            break;
        };

        throughput *= sample.attenuation;
        ray = sample.scattered_ray;

        // 4. Russian Roulette termination after 3 bounces
        if bounce >= 3 {
            let p = throughput
                .x
                .max(throughput.y)
                .max(throughput.z)
                .clamp(0.05, 0.95);
            if rng() > p {
                break;
            }
            throughput /= p;
        }
    }

    accumulated_radiance
}

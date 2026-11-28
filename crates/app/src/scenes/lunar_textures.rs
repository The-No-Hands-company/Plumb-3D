//! Procedural high-resolution Lunar Surface Cartography and Texture Generator.
//! Synthesizes authentic 2K equirectangular Albedo (diffuse) and Normal (bump relief)
//! maps featuring major lunar maria, highland anorthosite terrae, and ray impact craters
//! (Tycho, Copernicus, Kepler, Aristarchus).

use std::f32::consts::{PI, TAU};
use std::path::PathBuf;

/// Ensures authentic lunar surface textures exist on disk, generating them if absent.
/// Returns (albedo_texture_path, normal_texture_path).
pub fn ensure_lunar_textures_exist() -> (String, String) {
    let candidates = [
        PathBuf::from("assets/textures/lunar"),
        PathBuf::from("crates/app/assets/textures/lunar"),
    ];

    for dir in &candidates {
        let nasa_photo = dir.join("nasa_moon_photographic.jpg");
        let alb = if nasa_photo.exists() {
            nasa_photo
        } else {
            dir.join("moon_albedo.png")
        };
        let nrm = dir.join("moon_normal.png");
        if alb.exists() && nrm.exists() {
            return (
                alb.to_string_lossy().to_string(),
                nrm.to_string_lossy().to_string(),
            );
        }
    }

    let base_dir = PathBuf::from("assets/textures/lunar");
    if let Err(e) = std::fs::create_dir_all(&base_dir) {
        eprintln!("[Plumb 3D] Failed to create lunar texture directory: {}", e);
    }

    let (albedo_img, normal_img) = generate_lunar_maps(2048, 1024);

    let albedo_path = base_dir.join("moon_albedo.png");
    let normal_path = base_dir.join("moon_normal.png");

    if let Err(e) = albedo_img.save(&albedo_path) {
        eprintln!("[Plumb 3D] Failed to save lunar albedo map: {}", e);
    }
    if let Err(e) = normal_img.save(&normal_path) {
        eprintln!("[Plumb 3D] Failed to save lunar normal map: {}", e);
    }

    (
        albedo_path.to_string_lossy().to_string(),
        normal_path.to_string_lossy().to_string(),
    )
}

struct LunarCrater {
    lon: f32,    // Longitude (radians, -PI to PI)
    lat: f32,    // Latitude (radians, -PI/2 to PI/2)
    radius: f32, // Angular radius (radians)
    depth: f32,
    rim_height: f32,
    has_rays: bool,
    ray_count: usize,
    ray_length: f32,
}

fn generate_lunar_maps(width: u32, height: u32) -> (image::RgbaImage, image::RgbaImage) {
    let mut albedo_img = image::RgbaImage::new(width, height);
    let mut normal_img = image::RgbaImage::new(width, height);

    let craters = get_lunar_craters();
    let w_f = width as f32;
    let h_f = height as f32;

    // 1. First pass: compute height / elevation grid for normal calculation
    let mut elevation_grid = vec![0.0_f32; (width * height) as usize];
    let mut albedo_grid = vec![[0u8; 3]; (width * height) as usize];

    for y in 0..height {
        let v = y as f32 / h_f;
        let lat = (0.5 - v) * PI;
        let cos_lat = lat.cos();
        let sin_lat = lat.sin();

        for x in 0..width {
            let u = x as f32 / w_f;
            let lon = (u - 0.5) * TAU;

            // 3D Cartesian coordinates on unit sphere
            let p_x = cos_lat * lon.sin();
            let p_y = sin_lat;
            let p_z = cos_lat * lon.cos();

            // Evaluate mare basin factor (dark volcanic plains on near side)
            let mare_factor = evaluate_maria(lon, lat);

            // Multiscale fractal noise for highland terrain & basalt texture
            let n1 = smooth_noise(p_x * 4.0, p_y * 4.0, p_z * 4.0);
            let n2 = smooth_noise(p_x * 12.0 + 3.1, p_y * 12.0 + 1.4, p_z * 12.0 + 7.2);
            let n3 = smooth_noise(p_x * 32.0 + 5.7, p_y * 32.0 + 8.2, p_z * 32.0 + 2.9);
            let n4 = smooth_noise(p_x * 96.0 + 1.9, p_y * 96.0 + 4.3, p_z * 96.0 + 6.1);
            let fbm = n1 * 0.48 + n2 * 0.28 + n3 * 0.16 + n4 * 0.08;

            let mut elev = 0.65 + fbm * 0.18 - mare_factor * 0.22;
            let mut ray_brightness = 0.0_f32;

            // Evaluate major impact craters
            for c in &craters {
                let dist = angular_distance(lon, lat, c.lon, c.lat);
                if dist < c.radius * 2.5 {
                    let d_norm = dist / c.radius;
                    if d_norm < 1.0 {
                        // Crater bowl
                        let bowl = (1.0 - d_norm * d_norm).sqrt();
                        elev -= c.depth * bowl;
                        // Central peak
                        if d_norm < 0.22 {
                            let peak = (1.0 - d_norm / 0.22).powi(2);
                            elev += c.depth * 0.45 * peak;
                        }
                    } else if d_norm < 1.45 {
                        // Raised crater rim
                        let rim_t = (d_norm - 1.0) / 0.45;
                        let rim_shape = (rim_t * PI).sin();
                        elev += c.rim_height * rim_shape;
                    }
                }

                // Ray system (e.g. Tycho, Copernicus)
                if c.has_rays && dist < c.ray_length {
                    let d_ratio = dist / c.ray_length;
                    let angle = (lat - c.lat).atan2(lon - c.lon);
                    let ray_pattern = ((angle * c.ray_count as f32).sin().abs()).powi(6);
                    let ray_str = ray_pattern * (1.0 - d_ratio).powi(2) * 0.45;
                    ray_brightness = ray_brightness.max(ray_str);
                }
            }

            let idx = (y * width + x) as usize;
            elevation_grid[idx] = elev;

            // Compute albedo color
            let base_albedo = if mare_factor > 0.35 {
                // Basalt Mare (Dark gray / charcoal)
                let t = ((mare_factor - 0.35) / 0.65).clamp(0.0, 1.0);
                let col_a = [42.0, 44.0, 48.0];
                let col_b = [62.0, 66.0, 72.0];
                lerp3(col_b, col_a, t)
            } else {
                // Anorthosite Highlands (Bright silvery-tan)
                let t = (fbm * 1.2).clamp(0.0, 1.0);
                let col_a = [135.0, 138.0, 144.0];
                let col_b = [178.0, 180.0, 186.0];
                lerp3(col_a, col_b, t)
            };

            // Blend in crater ray ejecta brightness
            let mut final_col = [
                (base_albedo[0] * (1.0 + ray_brightness * 1.4)).min(245.0) as u8,
                (base_albedo[1] * (1.0 + ray_brightness * 1.4)).min(248.0) as u8,
                (base_albedo[2] * (1.0 + ray_brightness * 1.4)).min(255.0) as u8,
            ];

            // Local micro-crater freckling
            if fbm > 0.72 {
                final_col[0] = final_col[0].saturating_add(15);
                final_col[1] = final_col[1].saturating_add(15);
                final_col[2] = final_col[2].saturating_add(18);
            }

            albedo_grid[idx] = final_col;
        }
    }

    // 2. Second pass: compute Sobel tangent normals and populate images
    let bump_scale = 3.5_f32;
    for y in 0..height {
        let ym = if y == 0 { height - 1 } else { y - 1 };
        let yp = if y + 1 >= height { 0 } else { y + 1 };

        for x in 0..width {
            let xm = if x == 0 { width - 1 } else { x - 1 };
            let xp = if x + 1 >= width { 0 } else { x + 1 };

            let idx = (y * width + x) as usize;
            let col = albedo_grid[idx];
            albedo_img.put_pixel(x, y, image::Rgba([col[0], col[1], col[2], 255]));

            let h_l = elevation_grid[(y * width + xm) as usize];
            let h_r = elevation_grid[(y * width + xp) as usize];
            let h_u = elevation_grid[(ym * width + x) as usize];
            let h_d = elevation_grid[(yp * width + x) as usize];

            let dx = (h_r - h_l) * bump_scale;
            let dy = (h_d - h_u) * bump_scale;

            let n_raw = core_math::vec3(-dx, -dy, 1.0).normalize_or_zero();
            let nx = ((n_raw.x * 0.5 + 0.5) * 255.0).clamp(0.0, 255.0) as u8;
            let ny = ((n_raw.y * 0.5 + 0.5) * 255.0).clamp(0.0, 255.0) as u8;
            let nz = ((n_raw.z * 0.5 + 0.5) * 255.0).clamp(0.0, 255.0) as u8;

            normal_img.put_pixel(x, y, image::Rgba([nx, ny, nz, 255]));
        }
    }

    (albedo_img, normal_img)
}

fn get_lunar_craters() -> Vec<LunarCrater> {
    vec![
        // Tycho (prominent southern impact with colossal ray system)
        LunarCrater {
            lon: -11.2_f32.to_radians(),
            lat: -43.3_f32.to_radians(),
            radius: 0.048,
            depth: 0.16,
            rim_height: 0.08,
            has_rays: true,
            ray_count: 14,
            ray_length: 1.15,
        },
        // Copernicus (Oceanus Procellarum terraced crater)
        LunarCrater {
            lon: -20.1_f32.to_radians(),
            lat: 9.6_f32.to_radians(),
            radius: 0.052,
            depth: 0.14,
            rim_height: 0.07,
            has_rays: true,
            ray_count: 12,
            ray_length: 0.75,
        },
        // Kepler
        LunarCrater {
            lon: -38.0_f32.to_radians(),
            lat: 8.1_f32.to_radians(),
            radius: 0.032,
            depth: 0.11,
            rim_height: 0.05,
            has_rays: true,
            ray_count: 8,
            ray_length: 0.55,
        },
        // Aristarchus (brightest feature on the Moon)
        LunarCrater {
            lon: -47.4_f32.to_radians(),
            lat: 23.7_f32.to_radians(),
            radius: 0.038,
            depth: 0.13,
            rim_height: 0.06,
            has_rays: true,
            ray_count: 9,
            ray_length: 0.48,
        },
        // Plato (dark floor crater in northern highlands)
        LunarCrater {
            lon: -9.3_f32.to_radians(),
            lat: 51.6_f32.to_radians(),
            radius: 0.055,
            depth: 0.10,
            rim_height: 0.06,
            has_rays: false,
            ray_count: 0,
            ray_length: 0.0,
        },
        // Clavius (giant southern crater)
        LunarCrater {
            lon: -14.4_f32.to_radians(),
            lat: -58.4_f32.to_radians(),
            radius: 0.095,
            depth: 0.12,
            rim_height: 0.05,
            has_rays: false,
            ray_count: 0,
            ray_length: 0.0,
        },
    ]
}

fn evaluate_maria(lon: f32, lat: f32) -> f32 {
    // Maria are located almost entirely on the near side (Z > 0, lon between -90 and +90 deg)
    let mut mare = 0.0_f32;

    // Oceanus Procellarum & Mare Imbrium (Western expanse)
    let d_proc = angular_distance(lon, lat, -25.0_f32.to_radians(), 22.0_f32.to_radians());
    if d_proc < 0.65 {
        mare = mare.max(1.0 - (d_proc / 0.65).powi(2));
    }

    // Mare Serenitatis
    let d_seren = angular_distance(lon, lat, 17.5_f32.to_radians(), 28.0_f32.to_radians());
    if d_seren < 0.28 {
        mare = mare.max(1.0 - (d_seren / 0.28).powi(2));
    }

    // Mare Tranquillitatis (Apollo 11 landing site)
    let d_tranq = angular_distance(lon, lat, 31.4_f32.to_radians(), 8.5_f32.to_radians());
    if d_tranq < 0.33 {
        mare = mare.max(1.0 - (d_tranq / 0.33).powi(2));
    }

    // Mare Crisium (Distinct circular dark sea)
    let d_cris = angular_distance(lon, lat, 59.1_f32.to_radians(), 17.0_f32.to_radians());
    if d_cris < 0.20 {
        mare = mare.max(1.0 - (d_cris / 0.20).powi(2));
    }

    // Mare Nubium & Humorum
    let d_nub = angular_distance(lon, lat, -20.0_f32.to_radians(), -21.0_f32.to_radians());
    if d_nub < 0.32 {
        mare = mare.max(1.0 - (d_nub / 0.32).powi(2));
    }

    // Mare Fecunditatis
    let d_fec = angular_distance(lon, lat, 51.0_f32.to_radians(), -7.8_f32.to_radians());
    if d_fec < 0.26 {
        mare = mare.max(1.0 - (d_fec / 0.26).powi(2));
    }

    mare.clamp(0.0, 1.0)
}

fn angular_distance(lon1: f32, lat1: f32, lon2: f32, lat2: f32) -> f32 {
    let d_lon = lon1 - lon2;
    let d_lat = lat1 - lat2;
    let a = (d_lat * 0.5).sin().powi(2) + lat1.cos() * lat2.cos() * (d_lon * 0.5).sin().powi(2);
    2.0 * a.clamp(0.0, 1.0).sqrt().asin()
}

fn lerp3(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

fn smooth_noise(x: f32, y: f32, z: f32) -> f32 {
    let ix = x.floor() as i32;
    let iy = y.floor() as i32;
    let iz = z.floor() as i32;
    let fx = x.fract().abs();
    let fy = y.fract().abs();
    let fz = z.fract().abs();

    let ux = fx * fx * (3.0 - 2.0 * fx);
    let uy = fy * fy * (3.0 - 2.0 * fy);
    let uz = fz * fz * (3.0 - 2.0 * fz);

    let hash = |px: i32, py: i32, pz: i32| -> f32 {
        let mut n =
            px.wrapping_mul(374761393) ^ py.wrapping_mul(668265263) ^ pz.wrapping_mul(951234191);
        n = (n ^ (n >> 13)).wrapping_mul(1274126177);
        ((n as u32 >> 8) as f32) / 16777216.0
    };

    let c000 = hash(ix, iy, iz);
    let c100 = hash(ix + 1, iy, iz);
    let c010 = hash(ix, iy + 1, iz);
    let c110 = hash(ix + 1, iy + 1, iz);
    let c001 = hash(ix, iy, iz + 1);
    let c101 = hash(ix + 1, iy, iz + 1);
    let c011 = hash(ix, iy + 1, iz + 1);
    let c111 = hash(ix + 1, iy + 1, iz + 1);

    let x00 = c000 + ux * (c100 - c000);
    let x10 = c010 + ux * (c110 - c010);
    let x01 = c001 + ux * (c101 - c001);
    let x11 = c011 + ux * (c111 - c011);

    let y0 = x00 + uy * (x10 - x00);
    let y1 = x01 + uy * (x11 - x01);

    y0 + uz * (y1 - y0)
}

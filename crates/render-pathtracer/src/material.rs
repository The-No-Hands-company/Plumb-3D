use core_math::{Ray, Vec2, Vec3};

/// Procedural or constant texture source for PBR surfaces
#[derive(Debug, Clone, PartialEq)]
pub enum TextureSource {
    /// Constant solid RGB color
    Constant(Vec3),
    /// Loaded RGBA image texture (PNG/JPG)
    Image {
        width: u32,
        height: u32,
        data: std::sync::Arc<Vec<u8>>,
    },
    /// Procedural 2D UV checkerboard
    Checkerboard {
        color_a: Vec3,
        color_b: Vec3,
        scale: f32,
    },
    /// Procedural 2D UV coordinate wire grid
    Grid {
        line_color: Vec3,
        cell_color: Vec3,
        scale: f32,
    },
    /// Procedural 3D World Triplanar grid lines
    Triplanar {
        line_color: Vec3,
        cell_color: Vec3,
        scale: f32,
    },
    /// Concentric circular rings around UV center
    Rings {
        color_a: Vec3,
        color_b: Vec3,
        scale: f32,
    },
    /// Procedural marble / turbulent noise pattern
    Marble {
        color_a: Vec3,
        color_b: Vec3,
        scale: f32,
    },
}

impl From<Vec3> for TextureSource {
    fn from(v: Vec3) -> Self {
        TextureSource::Constant(v)
    }
}

impl Default for TextureSource {
    fn default() -> Self {
        TextureSource::Constant(Vec3::splat(0.75))
    }
}

impl TextureSource {
    /// Evaluates the surface color at given UV coordinates, world position, and geometric normal
    pub fn evaluate(&self, uv: Vec2, world_pos: Vec3, normal: Vec3) -> Vec3 {
        match self {
            TextureSource::Constant(c) => *c,
            TextureSource::Image { width, height, data } => {
                if *width == 0 || *height == 0 || data.is_empty() {
                    return Vec3::ONE;
                }
                let u = uv.x.rem_euclid(1.0);
                let v = (1.0 - uv.y.rem_euclid(1.0)).clamp(0.0, 1.0);
                let px = ((u * (*width as f32 - 1.0)).round() as usize).min(*width as usize - 1);
                let py = ((v * (*height as f32 - 1.0)).round() as usize).min(*height as usize - 1);
                let idx = (py * *width as usize + px) * 4;
                if idx + 3 <= data.len() {
                    Vec3::new(
                        data[idx] as f32 / 255.0,
                        data[idx + 1] as f32 / 255.0,
                        data[idx + 2] as f32 / 255.0,
                    )
                } else {
                    Vec3::ONE
                }
            }
            TextureSource::Checkerboard { color_a, color_b, scale } => {
                let u_idx = (uv.x * scale).floor() as i32;
                let v_idx = (uv.y * scale).floor() as i32;
                let check = ((u_idx + v_idx) % 2).abs();
                if check == 0 {
                    *color_a
                } else {
                    *color_b
                }
            }
            TextureSource::Grid { line_color, cell_color, scale } => {
                let fu = (uv.x * scale).fract().abs();
                let fv = (uv.y * scale).fract().abs();
                let edge = fu.min(fv);
                if edge < 0.06 {
                    *line_color
                } else {
                    *cell_color
                }
            }
            TextureSource::Triplanar { line_color, cell_color, scale } => {
                let abs_n = normal.abs();
                let tp_uv = if abs_n.x > abs_n.y && abs_n.x > abs_n.z {
                    Vec2::new(world_pos.y, world_pos.z)
                } else if abs_n.z > abs_n.x && abs_n.z > abs_n.y {
                    Vec2::new(world_pos.x, world_pos.y)
                } else {
                    Vec2::new(world_pos.x, world_pos.z)
                };
                let fu = (tp_uv.x * scale).fract().abs();
                let fv = (tp_uv.y * scale).fract().abs();
                let edge = fu.min(fv);
                if edge < 0.04 {
                    *line_color
                } else {
                    *cell_color
                }
            }
            TextureSource::Rings { color_a, color_b, scale } => {
                let center = Vec2::splat(0.5);
                let dist = (uv - center).length() * scale;
                let s = (dist * std::f32::consts::PI * 2.0).sin() * 0.5 + 0.5;
                *color_a * s + *color_b * (1.0 - s)
            }
            TextureSource::Marble { color_a, color_b, scale } => {
                let p = uv * *scale;
                let n = ((p.x * 12.9898 + p.y * 78.233).sin() * 43758.5453).fract().abs();
                let m = ((p.x + p.y + n * 2.0) * std::f32::consts::PI).sin() * 0.5 + 0.5;
                *color_a * (1.0 - m * 0.7) + *color_b * (m * 0.7)
            }
        }
    }
}

/// PBR Material model for physically based path tracing
#[derive(Debug, Clone)]
pub enum Material {
    /// Ideal diffuse Lambertian reflection
    Lambertian {
        albedo: TextureSource,
    },
    /// Metallic specular reflection with microfacet roughness
    Metal {
        albedo: TextureSource,
        roughness: f32,
    },
    /// Dielectric glass/water with Snell's law refraction and Schlick Fresnel reflection
    Dielectric {
        ior: f32,
        albedo: TextureSource,
    },
    /// Radiance emitter (mesh area light / studio light)
    Emissive {
        radiance: Vec3,
    },
    /// Disney / Unreal Engine Principled PBR BSDF with microfacet reflection, refraction, and emission
    Principled {
        base_color: TextureSource,
        metallic: f32,
        roughness: f32,
        ior: f32,
        transmission: f32,
        emission: Vec3,
    },
}

impl Default for Material {
    fn default() -> Self {
        Self::Lambertian {
            albedo: TextureSource::Constant(Vec3::splat(0.75)),
        }
    }
}

/// Result of scattering a ray off a surface BSDF
#[derive(Debug, Clone, Copy)]
pub struct BsdfSample {
    pub scattered_ray: Ray,
    pub attenuation: Vec3,
    pub is_specular: bool,
}

impl Material {
    pub fn lambertian(albedo: impl Into<TextureSource>) -> Self {
        Self::Lambertian { albedo: albedo.into() }
    }

    pub fn metal(albedo: impl Into<TextureSource>, roughness: f32) -> Self {
        Self::Metal {
            albedo: albedo.into(),
            roughness,
        }
    }

    pub fn dielectric(ior: f32, albedo: impl Into<TextureSource>) -> Self {
        Self::Dielectric {
            ior,
            albedo: albedo.into(),
        }
    }

    pub fn principled(
        base_color: impl Into<TextureSource>,
        metallic: f32,
        roughness: f32,
        ior: f32,
        transmission: f32,
        emission: Vec3,
    ) -> Self {
        Self::Principled {
            base_color: base_color.into(),
            metallic: metallic.clamp(0.0, 1.0),
            roughness: roughness.clamp(0.001, 1.0),
            ior: ior.max(1.0),
            transmission: transmission.clamp(0.0, 1.0),
            emission,
        }
    }

    pub fn is_emissive(&self) -> bool {
        match self {
            Material::Emissive { .. } => true,
            Material::Principled { emission, .. } => emission.length_squared() > 1e-4,
            _ => false,
        }
    }

    pub fn emission(&self) -> Vec3 {
        match self {
            Material::Emissive { radiance } => *radiance,
            Material::Principled { emission, .. } => *emission,
            _ => Vec3::ZERO,
        }
    }

    /// Evaluates the BSDF and scatters a ray
    pub fn sample(
        &self,
        ray_in: &Ray,
        hit_pos: Vec3,
        normal: Vec3,
        uv: Vec2,
        rng: &mut impl FnMut() -> f32,
    ) -> Option<BsdfSample> {
        match self {
            Material::Lambertian { albedo } => {
                let albedo_eval = albedo.evaluate(uv, hit_pos, normal);
                let random_unit = random_in_unit_sphere(rng);
                let mut scatter_dir = normal + random_unit;

                if scatter_dir.length_squared() < 1e-6 {
                    scatter_dir = normal;
                } else {
                    scatter_dir = scatter_dir.normalize();
                }

                Some(BsdfSample {
                    scattered_ray: Ray::new(hit_pos + normal * 1e-4, scatter_dir),
                    attenuation: albedo_eval,
                    is_specular: false,
                })
            }
            Material::Metal { albedo, roughness } => {
                let albedo_eval = albedo.evaluate(uv, hit_pos, normal);
                let r = roughness.clamp(0.0, 1.0);
                let reflected = reflect(ray_in.direction.normalize(), normal);
                let scatter_dir = reflected + random_in_unit_sphere(rng) * r;

                if scatter_dir.dot(normal) > 0.0 {
                    Some(BsdfSample {
                        scattered_ray: Ray::new(hit_pos + normal * 1e-4, scatter_dir.normalize()),
                        attenuation: albedo_eval,
                        is_specular: true,
                    })
                } else {
                    None
                }
            }
            Material::Dielectric { ior, albedo } => {
                let albedo_eval = albedo.evaluate(uv, hit_pos, normal);
                let unit_dir = ray_in.direction.normalize();
                let cos_theta = (-unit_dir).dot(normal).clamp(0.0, 1.0);
                let sin_theta = (1.0 - cos_theta * cos_theta).max(0.0).sqrt();

                let refraction_ratio = if unit_dir.dot(normal) > 0.0 {
                    *ior
                } else {
                    1.0 / *ior
                };

                let cannot_refract = refraction_ratio * sin_theta > 1.0;
                let direction = if cannot_refract || reflectance(cos_theta, refraction_ratio) > rng() {
                    reflect(unit_dir, normal)
                } else {
                    refract(unit_dir, normal, refraction_ratio)
                };

                let normal_offset = if direction.dot(normal) > 0.0 {
                    normal * 1e-4
                } else {
                    -normal * 1e-4
                };

                Some(BsdfSample {
                    scattered_ray: Ray::new(hit_pos + normal_offset, direction.normalize()),
                    attenuation: albedo_eval,
                    is_specular: true,
                })
            }
            Material::Principled {
                base_color,
                metallic,
                roughness,
                ior,
                transmission,
                ..
            } => {
                let albedo_eval = base_color.evaluate(uv, hit_pos, normal);
                let unit_dir = ray_in.direction.normalize();

                // 1. Specular Transmission (Refraction / Glass)
                if *transmission > 0.001 && rng() < *transmission {
                    let cos_theta = (-unit_dir).dot(normal).clamp(0.0, 1.0);
                    let sin_theta = (1.0 - cos_theta * cos_theta).max(0.0).sqrt();
                    let refraction_ratio = if unit_dir.dot(normal) > 0.0 { *ior } else { 1.0 / *ior };
                    let cannot_refract = refraction_ratio * sin_theta > 1.0;
                    let direction = if cannot_refract || reflectance(cos_theta, refraction_ratio) > rng() {
                        reflect(unit_dir, normal)
                    } else {
                        refract(unit_dir, normal, refraction_ratio)
                    };
                    let normal_offset = if direction.dot(normal) > 0.0 {
                        normal * 1e-4
                    } else {
                        -normal * 1e-4
                    };
                    return Some(BsdfSample {
                        scattered_ray: Ray::new(hit_pos + normal_offset, direction.normalize()),
                        attenuation: albedo_eval,
                        is_specular: true,
                    });
                }

                // 2. Metallic vs Dielectric (Cook-Torrance Microfacet Model)
                let f0 = 0.04 * (1.0 - *metallic) + *metallic;
                let is_specular_bounce = rng() < f0;

                if is_specular_bounce {
                    let r = roughness.clamp(0.001, 1.0);
                    let reflected = reflect(unit_dir, normal);
                    let scatter_dir = reflected + random_in_unit_sphere(rng) * r;
                    if scatter_dir.dot(normal) > 0.0 {
                        let tint = albedo_eval * *metallic + Vec3::ONE * (1.0 - *metallic);
                        Some(BsdfSample {
                            scattered_ray: Ray::new(hit_pos + normal * 1e-4, scatter_dir.normalize()),
                            attenuation: tint,
                            is_specular: true,
                        })
                    } else {
                        None
                    }
                } else {
                    // Diffuse Lambertian
                    let random_unit = random_in_unit_sphere(rng);
                    let mut scatter_dir = normal + random_unit;
                    if scatter_dir.length_squared() < 1e-6 {
                        scatter_dir = normal;
                    } else {
                        scatter_dir = scatter_dir.normalize();
                    }
                    Some(BsdfSample {
                        scattered_ray: Ray::new(hit_pos + normal * 1e-4, scatter_dir),
                        attenuation: albedo_eval * (1.0 - *metallic),
                        is_specular: false,
                    })
                }
            }
            Material::Emissive { .. } => None,
        }
    }
}

fn reflect(v: Vec3, n: Vec3) -> Vec3 {
    v - n * (2.0 * v.dot(n))
}

fn refract(uv: Vec3, n: Vec3, etai_over_etat: f32) -> Vec3 {
    let cos_theta = (-uv).dot(n).min(1.0);
    let r_out_perp = (uv + n * cos_theta) * etai_over_etat;
    let r_out_parallel = -n * (1.0 - r_out_perp.length_squared()).abs().sqrt();
    r_out_perp + r_out_parallel
}

fn reflectance(cosine: f32, ref_idx: f32) -> f32 {
    let mut r0 = (1.0 - ref_idx) / (1.0 + ref_idx);
    r0 = r0 * r0;
    r0 + (1.0 - r0) * (1.0 - cosine).powi(5)
}

fn random_in_unit_sphere(rng: &mut impl FnMut() -> f32) -> Vec3 {
    loop {
        let p = Vec3::new(
            rng() * 2.0 - 1.0,
            rng() * 2.0 - 1.0,
            rng() * 2.0 - 1.0,
        );
        let len_sq = p.length_squared();
        if len_sq > 1e-6 && len_sq < 1.0 {
            return p / len_sq.sqrt();
        }
    }
}

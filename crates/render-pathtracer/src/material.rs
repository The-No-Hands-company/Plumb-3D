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
                    // Decode sRGB texture bytes into physical linear radiance
                    let to_linear = |b: u8| -> f32 {
                        let c = b as f32 / 255.0;
                        if c <= 0.04045 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) }
                    };
                    Vec3::new(
                        to_linear(data[idx]),
                        to_linear(data[idx + 1]),
                        to_linear(data[idx + 2]),
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
                let coord = if world_pos.length_squared() > 1e-4 {
                    world_pos * (*scale * 0.45)
                } else {
                    core_math::vec3(uv.x * *scale, uv.y * *scale, 0.5)
                };
                let n1 = smooth_noise3d(coord);
                let n2 = smooth_noise3d(coord * 2.08 + core_math::vec3(1.7, 4.3, 2.1));
                let n3 = smooth_noise3d(coord * 4.15 + core_math::vec3(7.2, 1.1, 5.8));
                let fbm = n1 * 0.53 + n2 * 0.31 + n3 * 0.16;
                // Soft turbulent threshold for planetary basalt maria and highland plains
                let m = (fbm * std::f32::consts::PI * 1.5).sin() * 0.5 + 0.5;
                *color_a * (0.35 + 0.65 * m) + *color_b * (0.05 * (1.0 - m))
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
        normal_map: Option<TextureSource>,
        metallic: f32,
        roughness: f32,
        ior: f32,
        transmission: f32,
        emission: Vec3,
    },
    /// Subsurface scattering (BSSRDF dipole/random-walk approximation for skin, marble, jade, wax)
    Subsurface {
        albedo: TextureSource,
        subsurface_color: TextureSource,
        radius: Vec3,
        ior: f32,
        roughness: f32,
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
        Self::principled_with_normal(base_color, None, metallic, roughness, ior, transmission, emission)
    }

    pub fn principled_with_normal(
        base_color: impl Into<TextureSource>,
        normal_map: Option<TextureSource>,
        metallic: f32,
        roughness: f32,
        ior: f32,
        transmission: f32,
        emission: Vec3,
    ) -> Self {
        Self::Principled {
            base_color: base_color.into(),
            normal_map,
            metallic: metallic.clamp(0.0, 1.0),
            roughness: roughness.clamp(0.001, 1.0),
            ior: ior.max(1.0),
            transmission: transmission.clamp(0.0, 1.0),
            emission,
        }
    }

    /// Evaluates perturbed tangent-space normal vector if normal map is active
    pub fn perturbed_normal(&self, uv: Vec2, hit_pos: Vec3, normal: Vec3) -> Vec3 {
        match self {
            Material::Principled { normal_map: Some(n_map), .. } => {
                let sample = n_map.evaluate(uv, hit_pos, normal);
                let tangent_normal = sample * 2.0 - Vec3::ONE;
                let (tangent, bitangent) = create_tangent_basis(normal);
                let perturbed = tangent * tangent_normal.x + bitangent * tangent_normal.y + normal * tangent_normal.z;
                if perturbed.length_squared() > 1e-4 {
                    perturbed.normalize()
                } else {
                    normal
                }
            }
            _ => normal,
        }
    }

    pub fn subsurface(
        albedo: impl Into<TextureSource>,
        subsurface_color: impl Into<TextureSource>,
        radius: Vec3,
        ior: f32,
        roughness: f32,
    ) -> Self {
        Self::Subsurface {
            albedo: albedo.into(),
            subsurface_color: subsurface_color.into(),
            radius: radius.max(Vec3::splat(1e-4)),
            ior: ior.max(1.0),
            roughness: roughness.clamp(0.001, 1.0),
        }
    }

    /// Evaluates base albedo color for lighting integration
    pub fn albedo(&self, uv: Vec2, hit_pos: Vec3, normal: Vec3) -> Vec3 {
        match self {
            Material::Lambertian { albedo } => albedo.evaluate(uv, hit_pos, normal),
            Material::Metal { albedo, .. } => albedo.evaluate(uv, hit_pos, normal),
            Material::Dielectric { albedo, .. } => albedo.evaluate(uv, hit_pos, normal),
            Material::Principled { base_color, .. } => base_color.evaluate(uv, hit_pos, normal),
            Material::Subsurface { albedo, .. } => albedo.evaluate(uv, hit_pos, normal),
            Material::Emissive { .. } => Vec3::ZERO,
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
                let eff_normal = self.perturbed_normal(uv, hit_pos, normal);
                let albedo_eval = base_color.evaluate(uv, hit_pos, eff_normal);
                let unit_dir = ray_in.direction.normalize();

                // 1. Specular Transmission (Refraction / Glass)
                if *transmission > 0.001 && rng() < *transmission {
                    let cos_theta = (-unit_dir).dot(eff_normal).clamp(0.0, 1.0);
                    let sin_theta = (1.0 - cos_theta * cos_theta).max(0.0).sqrt();
                    let refraction_ratio = if unit_dir.dot(eff_normal) > 0.0 { *ior } else { 1.0 / *ior };
                    let cannot_refract = refraction_ratio * sin_theta > 1.0;
                    let direction = if cannot_refract || reflectance(cos_theta, refraction_ratio) > rng() {
                        reflect(unit_dir, eff_normal)
                    } else {
                        refract(unit_dir, eff_normal, refraction_ratio)
                    };
                    let normal_offset = if direction.dot(eff_normal) > 0.0 {
                        eff_normal * 1e-4
                    } else {
                        -eff_normal * 1e-4
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
                    let reflected = reflect(unit_dir, eff_normal);
                    let scatter_dir = reflected + random_in_unit_sphere(rng) * r;
                    if scatter_dir.dot(eff_normal) > 0.0 {
                        let tint = albedo_eval * *metallic + Vec3::ONE * (1.0 - *metallic);
                        Some(BsdfSample {
                            scattered_ray: Ray::new(hit_pos + eff_normal * 1e-4, scatter_dir.normalize()),
                            attenuation: tint,
                            is_specular: true,
                        })
                    } else {
                        None
                    }
                } else {
                    // Diffuse Lambertian
                    let random_unit = random_in_unit_sphere(rng);
                    let mut scatter_dir = eff_normal + random_unit;
                    if scatter_dir.length_squared() < 1e-6 {
                        scatter_dir = eff_normal;
                    } else {
                        scatter_dir = scatter_dir.normalize();
                    }
                    Some(BsdfSample {
                        scattered_ray: Ray::new(hit_pos + eff_normal * 1e-4, scatter_dir),
                        attenuation: albedo_eval * (1.0 - *metallic),
                        is_specular: false,
                    })
                }
            }
            Material::Subsurface {
                albedo,
                subsurface_color,
                radius,
                ior,
                roughness,
            } => {
                let albedo_eval = albedo.evaluate(uv, hit_pos, normal);
                let sss_eval = subsurface_color.evaluate(uv, hit_pos, normal);
                let unit_dir = ray_in.direction.normalize();

                let cos_theta = (-unit_dir).dot(normal).clamp(0.0, 1.0);
                let f0 = reflectance(cos_theta, *ior);

                if rng() < f0 {
                    // Surface specular boundary reflection
                    let r = roughness.clamp(0.001, 1.0);
                    let reflected = reflect(unit_dir, normal);
                    let scatter_dir = reflected + random_in_unit_sphere(rng) * r;
                    if scatter_dir.dot(normal) > 0.0 {
                        Some(BsdfSample {
                            scattered_ray: Ray::new(hit_pos + normal * 1e-4, scatter_dir.normalize()),
                            attenuation: Vec3::ONE,
                            is_specular: true,
                        })
                    } else {
                        None
                    }
                } else {
                    // Internal volume diffusion: sample radial displacement along local tangent disk
                    let mfp = (radius.x + radius.y + radius.z) / 3.0;
                    let xi1 = rng().clamp(1e-4, 0.9999);
                    let r = -mfp * (1.0 - xi1).ln();
                    let phi = rng() * std::f32::consts::TAU;

                    let (tangent, bitangent) = create_tangent_basis(normal);
                    let exit_pos = hit_pos + tangent * (r * phi.cos()) + bitangent * (r * phi.sin());

                    let random_unit = random_in_unit_sphere(rng);
                    let mut scatter_dir = normal + random_unit;
                    if scatter_dir.length_squared() < 1e-6 {
                        scatter_dir = normal;
                    } else {
                        scatter_dir = scatter_dir.normalize();
                    }

                    let absorption = Vec3::new(
                        (-r / radius.x.max(1e-3)).exp(),
                        (-r / radius.y.max(1e-3)).exp(),
                        (-r / radius.z.max(1e-3)).exp(),
                    );
                    let attenuation = albedo_eval * sss_eval * absorption;

                    Some(BsdfSample {
                        scattered_ray: Ray::new(exit_pos + normal * 1e-4, scatter_dir),
                        attenuation,
                        is_specular: false,
                    })
                }
            }
            Material::Emissive { .. } => None,
        }
    }
}

fn create_tangent_basis(n: Vec3) -> (Vec3, Vec3) {
    let t = if n.y.abs() < 0.999 {
        Vec3::new(n.z, 0.0, -n.x).normalize()
    } else {
        Vec3::X
    };
    let b = n.cross(t).normalize();
    (t, b)
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
    let u = rng();
    let v = rng();
    let theta = u * std::f32::consts::TAU;
    let z = (v * 2.0 - 1.0).clamp(-1.0, 1.0);
    let r = (1.0 - z * z).max(0.0).sqrt();
    let radius = rng().cbrt().max(0.01);
    Vec3::new(radius * r * theta.cos(), radius * r * theta.sin(), radius * z)
}

fn smooth_noise3d(p: Vec3) -> f32 {
    let ix = p.x.floor() as i32;
    let iy = p.y.floor() as i32;
    let iz = p.z.floor() as i32;
    let fx = p.x.fract().abs();
    let fy = p.y.fract().abs();
    let fz = p.z.fract().abs();

    let ux = fx * fx * (3.0 - 2.0 * fx);
    let uy = fy * fy * (3.0 - 2.0 * fy);
    let uz = fz * fz * (3.0 - 2.0 * fz);

    let hash = |x: i32, y: i32, z: i32| -> f32 {
        let mut n = x.wrapping_mul(374761393)
            ^ y.wrapping_mul(668265263)
            ^ z.wrapping_mul(951234191);
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

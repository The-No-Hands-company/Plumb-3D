use crate::bvh::{BvhNode, RayHit, Triangle};
use crate::material::{Material, TextureSource};
use core_math::{Mat4, Ray, Vec2, Vec3};
use geo_kernel::{Mesh, RenderMesh};
use graph_engine::{LightType, NodeData, SceneGraph};

/// Directional light source for physically based solar illumination in the path tracer
#[derive(Debug, Clone, Copy)]
pub struct DirectionalLightSource {
    pub to_light: Vec3,
    pub color: Vec3,
    pub intensity: f32,
}

/// Point light source for local omnidirectional illumination in the path tracer
#[derive(Debug, Clone, Copy)]
pub struct PointLightSource {
    pub position: Vec3,
    pub color: Vec3,
    pub intensity: f32,
    pub range: f32,
}

/// Production Ray Tracing Scene holding the BVH geometry, materials, and light sources
pub struct PathTracerScene {
    pub bvh: BvhNode,
    pub materials: Vec<Material>,
    pub emissive_triangles: Vec<Triangle>,
    pub directional_lights: Vec<DirectionalLightSource>,
    pub point_lights: Vec<PointLightSource>,
    pub sun_direction: Option<Vec3>,
    pub sun_color: Vec3,
    pub sun_intensity: f32,
    pub background_zenith: Option<Vec3>,
    pub background_horizon: Option<Vec3>,
    pub background_ground: Option<Vec3>,
}

impl PathTracerScene {
    pub fn new(triangles: Vec<Triangle>, materials: Vec<Material>) -> Self {
        let emissive_triangles: Vec<Triangle> = triangles
            .iter()
            .copied()
            .filter(|t| materials.get(t.material_id).is_some_and(|m| m.is_emissive()))
            .collect();

        let bvh = BvhNode::build(triangles, 4);

        Self {
            bvh,
            materials,
            emissive_triangles,
            directional_lights: Vec::new(),
            point_lights: Vec::new(),
            sun_direction: None,
            sun_color: Vec3::new(1.0, 0.95, 0.8),
            sun_intensity: 1.0,
            background_zenith: None,
            background_horizon: None,
            background_ground: None,
        }
    }

    /// Sets custom background environment gradient (zenith, horizon, ground)
    pub fn with_background(mut self, zenith: Vec3, horizon: Vec3, ground: Vec3) -> Self {
        self.background_zenith = Some(zenith);
        self.background_horizon = Some(horizon);
        self.background_ground = Some(ground);
        self
    }

    /// Configures the scene for deep celestial nocturnal space with zero daytime glare.
    pub fn with_night_sky(mut self) -> Self {
        self.background_zenith = Some(Vec3::new(0.001, 0.002, 0.004));
        self.background_horizon = Some(Vec3::new(0.002, 0.003, 0.006));
        self.background_ground = Some(Vec3::splat(0.0005));
        self
    }

    /// Sets the physical sun direction, color, and intensity for atmospheric sky evaluation.
    pub fn with_sun(mut self, direction: Vec3, color: Vec3, intensity: f32) -> Self {
        self.sun_direction = Some(direction.normalize_or_zero());
        self.sun_color = color;
        self.sun_intensity = intensity.max(0.0);
        self
    }

    /// Intersects a ray against the scene BVH
    pub fn intersect(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<RayHit> {
        self.bvh.intersect(ray, t_min, t_max)
    }

    /// Builds a path tracer scene directly from the DCC SceneGraph
    pub fn from_scene_graph(scene: &SceneGraph) -> Self {
        let mut triangles = Vec::new();
        let mut materials = Vec::new();
        let mut directional_lights = Vec::new();
        let mut point_lights = Vec::new();
        let mut sun_direction = None;
        let mut sun_color = Vec3::new(1.0, 0.95, 0.85);
        let mut sun_intensity = 1.0_f32;

        // Default studio materials
        let mat_white = materials.len();
        materials.push(Material::lambertian(Vec3::splat(0.75)));
        let _mat_gold = materials.len();
        materials.push(Material::metal(Vec3::new(1.0, 0.78, 0.34), 0.2));

        // Iterate all mesh, curve, and light nodes in the scene graph
        for node in scene.nodes.values() {
            let world_mat = node.world_matrix;
            let normal_mat = world_mat.inverse().transpose();

            match &node.data {
                NodeData::Mesh { base_mesh, cached_render_mesh, materials: node_mats, .. } => {
                    let mut mat_offset_map = Vec::new();
                    if node_mats.is_empty() {
                        mat_offset_map.push(mat_white);
                    } else {
                        for m in node_mats {
                            let idx = materials.len();
                            let base_col = Vec3::new(m.base_color[0], m.base_color[1], m.base_color[2]);
                            let tex_src = if let Some(ref path) = m.albedo_texture_path {
                                if let Ok(img) = image::open(path) {
                                    let rgba = img.to_rgba8();
                                    TextureSource::Image {
                                        width: rgba.width(),
                                        height: rgba.height(),
                                        data: std::sync::Arc::new(rgba.into_raw()),
                                    }
                                } else {
                                    TextureSource::Constant(base_col)
                                }
                            } else {
                                match m.texture_pattern {
                                    1 => TextureSource::Checkerboard {
                                        color_a: base_col,
                                        color_b: base_col * 0.35,
                                        scale: 16.0,
                                    },
                                    2 => TextureSource::Grid {
                                        line_color: Vec3::splat(1.0),
                                        cell_color: base_col,
                                        scale: 10.0,
                                    },
                                    3 => TextureSource::Triplanar {
                                        line_color: Vec3::new(0.9, 0.95, 1.0),
                                        cell_color: base_col,
                                        scale: 2.0,
                                    },
                                    4 => TextureSource::Rings {
                                        color_a: base_col,
                                        color_b: base_col * 0.25,
                                        scale: 16.0,
                                    },
                                    5 => TextureSource::Marble {
                                        color_a: base_col,
                                        color_b: Vec3::splat(0.95),
                                        scale: 8.0,
                                    },
                                    _ => TextureSource::Constant(base_col),
                                }
                            };

                            let transmission = if m.base_color[3] < 0.99 { 1.0 - m.base_color[3] } else { 0.0 };
                            let emission = if m.emissive > 0.01 { base_col * (m.emissive * 10.0) } else { Vec3::ZERO };

                            let norm_src = if let Some(ref path) = m.normal_texture_path {
                                if let Ok(img) = image::open(path) {
                                    let rgba = img.to_rgba8();
                                    Some(TextureSource::Image {
                                        width: rgba.width(),
                                        height: rgba.height(),
                                        data: std::sync::Arc::new(rgba.into_raw()),
                                    })
                                } else {
                                    None
                                }
                            } else {
                                None
                            };

                            materials.push(Material::principled_with_normal(
                                tex_src,
                                norm_src,
                                m.metallic,
                                m.roughness,
                                1.5,
                                transmission,
                                emission,
                            ));
                            mat_offset_map.push(idx);
                        }
                    }

                    if let Some(render_mesh) = cached_render_mesh {
                        append_render_mesh_triangles(render_mesh, world_mat, normal_mat, &mat_offset_map, &mut triangles);
                    } else {
                        append_mesh_triangles(base_mesh, world_mat, normal_mat, &mat_offset_map, &mut triangles);
                    }
                }
                NodeData::Curve { cached_render_mesh: Some(render_mesh), .. } => {
                    let mat_offset_map = vec![_mat_gold];
                    append_render_mesh_triangles(render_mesh, world_mat, normal_mat, &mat_offset_map, &mut triangles);
                }
                NodeData::Light { light_type, color, intensity, .. } => {
                    let light_pos = world_mat.transform_point3(Vec3::ZERO);
                    if *light_type == LightType::Directional {
                        // Light shines along local -Z. The vector pointing FROM surface TOWARDS light:
                        let fwd = -world_mat.transform_vector3(Vec3::Z).normalize_or_zero();
                        let to_light = if fwd.length_squared() > 1e-4 { -fwd } else { Vec3::new(0.5, 0.8, 0.3).normalize() };
                        directional_lights.push(DirectionalLightSource {
                            to_light,
                            color: *color,
                            intensity: *intensity,
                        });
                        if sun_direction.is_none() {
                            sun_direction = Some(to_light);
                            sun_color = *color;
                            sun_intensity = *intensity;
                        }
                    } else if *light_type == LightType::Point {
                        point_lights.push(PointLightSource {
                            position: light_pos,
                            color: *color,
                            intensity: *intensity,
                            range: 100.0,
                        });
                    } else {
                        let light_mat = materials.len();
                        materials.push(Material::Emissive { radiance: *color * (*intensity).max(1.0) });
                        append_light_geometry(light_pos, *light_type, light_mat, &mut triangles);
                    }
                }
                _ => {}
            }
        }

        // Only add soft ceiling area light if NO emissive meshes AND NO directional lights exist
        let has_emissive = triangles.iter().any(|t| materials.get(t.material_id).is_some_and(|m| m.is_emissive()));
        let has_illumination = has_emissive || !directional_lights.is_empty();
        if !has_illumination {
            let light_y = 6.0_f32;
            let light_size = 2.0_f32;
            let mat_light = materials.len();
            materials.push(Material::Emissive { radiance: Vec3::splat(12.0) });

            let p0 = Vec3::new(-light_size, light_y, -light_size);
            let p1 = Vec3::new( light_size, light_y, -light_size);
            let p2 = Vec3::new( light_size, light_y,  light_size);
            let p3 = Vec3::new(-light_size, light_y,  light_size);
            let n_down = -Vec3::Y;

            triangles.push(Triangle::new(p0, p1, p2, n_down, n_down, n_down, Vec2::ZERO, Vec2::X, Vec2::ONE, mat_light));
            triangles.push(Triangle::new(p0, p2, p3, n_down, n_down, n_down, Vec2::ZERO, Vec2::ONE, Vec2::Y, mat_light));
        }

        let mut pt_scene = Self::new(triangles, materials);
        pt_scene.directional_lights = directional_lights;
        pt_scene.point_lights = point_lights;
        if let Some(sun_dir) = sun_direction {
            pt_scene = pt_scene.with_sun(sun_dir, sun_color, sun_intensity);
        }

        // Automatic celestial night-sky detection for lunar or nocturnal scenes
        let is_night_scene = scene.nodes.values().any(|n| {
            let name = n.name.to_lowercase();
            name.contains("moon") || name.contains("lunar") || name.contains("space") || name.contains("night")
        });
        if is_night_scene {
            pt_scene = pt_scene.with_night_sky();
        }

        pt_scene
    }

    /// Computes atmospheric studio sky radiance with physical sun disk & Mie forward scattering
    pub fn sky_radiance(&self, ray_dir: Vec3) -> Vec3 {
        let y = ray_dir.y.clamp(-1.0, 1.0);
        let t = 0.5 * (y + 1.0);
        let is_custom_bg = self.background_zenith.is_some();
        let zenith = self.background_zenith.unwrap_or(Vec3::new(0.35, 0.55, 0.95) * 0.9);
        let horizon = self.background_horizon.unwrap_or(Vec3::new(0.85, 0.88, 0.95));
        let ground = self.background_ground.unwrap_or(Vec3::new(0.18, 0.19, 0.22) * 0.5);

        let mut ambient = if y >= 0.0 {
            horizon * (1.0 - t) + zenith * t
        } else {
            ground * (-y) + horizon * (1.0 + y)
        };

        if is_custom_bg {
            // In deep celestial space: no atmospheric daylight haze; render pinpoint stars
            let sp = (ray_dir * 140.0).fract().abs();
            let sval = ((sp.x * 12.9898 + sp.y * 78.233 + sp.z * 45.164).sin() * 43_758.547).fract().abs();
            if sval > 0.9982 {
                let star_glow = ((sval - 0.9982) / 0.0018).powi(2) * 1.5;
                ambient += Vec3::splat(star_glow);
            }

            if let Some(sun_dir) = self.sun_direction {
                let cos_angle = ray_dir.dot(sun_dir);
                if cos_angle > 0.9995 {
                    ambient += self.sun_color * (self.sun_intensity * 20.0);
                }
            }
        } else if let Some(sun_dir) = self.sun_direction {
            let cos_angle = ray_dir.dot(sun_dir);
            if cos_angle > 0.0 {
                // Atmospheric Mie forward scattering haze glow
                let glow = (cos_angle.powf(32.0) * 0.4 + cos_angle.powf(128.0) * 1.0) * self.sun_intensity;
                ambient += self.sun_color * glow;

                // Solar disk
                if cos_angle > 0.9995 {
                    ambient += self.sun_color * (self.sun_intensity * 20.0);
                }
            }
        }

        ambient
    }
}

fn append_render_mesh_triangles(
    render_mesh: &RenderMesh,
    world_mat: Mat4,
    normal_mat: Mat4,
    mat_map: &[usize],
    triangles: &mut Vec<Triangle>,
) {
    let default_mat = *mat_map.first().unwrap_or(&0);
    for chunk in render_mesh.indices.as_chunks::<3>().0 {
        let i0 = chunk[0] as usize;
        let i1 = chunk[1] as usize;
        let i2 = chunk[2] as usize;

        if i0 >= render_mesh.vertices.len() || i1 >= render_mesh.vertices.len() || i2 >= render_mesh.vertices.len() {
            continue;
        }

        let v0 = &render_mesh.vertices[i0];
        let v1 = &render_mesh.vertices[i1];
        let v2 = &render_mesh.vertices[i2];

        let p0 = world_mat.transform_point3(Vec3::from_slice(&v0.position));
        let p1 = world_mat.transform_point3(Vec3::from_slice(&v1.position));
        let p2 = world_mat.transform_point3(Vec3::from_slice(&v2.position));

        let n0 = normal_mat.transform_vector3(Vec3::from_slice(&v0.normal)).normalize_or_zero();
        let n1 = normal_mat.transform_vector3(Vec3::from_slice(&v1.normal)).normalize_or_zero();
        let n2 = normal_mat.transform_vector3(Vec3::from_slice(&v2.normal)).normalize_or_zero();

        let uv0 = Vec2::from_slice(&v0.uv);
        let uv1 = Vec2::from_slice(&v1.uv);
        let uv2 = Vec2::from_slice(&v2.uv);

        let mat_id = mat_map.first().copied().unwrap_or(default_mat);
        triangles.push(Triangle::new(p0, p1, p2, n0, n1, n2, uv0, uv1, uv2, mat_id));
    }
}

fn append_mesh_triangles(
    mesh: &Mesh,
    world_mat: Mat4,
    normal_mat: Mat4,
    mat_map: &[usize],
    triangles: &mut Vec<Triangle>,
) {
    let default_mat = *mat_map.first().unwrap_or(&0);
    for fk in mesh.faces.keys() {
        let v_keys = mesh.face_vertices(fk);
        if v_keys.len() < 3 {
            continue;
        }

        let face_mat_slot = mesh.faces.get(fk).map_or(0, |f| f.material_id as usize);
        let mat_id = mat_map.get(face_mat_slot).copied().unwrap_or(default_mat);

        let v0_key = v_keys[0];
        let p0 = world_mat.transform_point3(mesh.vertices[v0_key].position);
        let n0 = normal_mat.transform_vector3(mesh.vertices[v0_key].normal).normalize_or_zero();
        let uv0 = mesh.vertices[v0_key].uv;

        for i in 1..(v_keys.len() - 1) {
            let v1_key = v_keys[i];
            let v2_key = v_keys[i + 1];

            let p1 = world_mat.transform_point3(mesh.vertices[v1_key].position);
            let n1 = normal_mat.transform_vector3(mesh.vertices[v1_key].normal).normalize_or_zero();
            let uv1 = mesh.vertices[v1_key].uv;

            let p2 = world_mat.transform_point3(mesh.vertices[v2_key].position);
            let n2 = normal_mat.transform_vector3(mesh.vertices[v2_key].normal).normalize_or_zero();
            let uv2 = mesh.vertices[v2_key].uv;

            triangles.push(Triangle::new(p0, p1, p2, n0, n1, n2, uv0, uv1, uv2, mat_id));
        }
    }
}

fn append_light_geometry(
    pos: Vec3,
    _light_type: LightType,
    mat_id: usize,
    triangles: &mut Vec<Triangle>,
) {
    let s = 0.25_f32;
    // Emissive octahedral area light marker
    let top = pos + Vec3::Y * s;
    let btm = pos - Vec3::Y * s;
    let px = pos + Vec3::X * s;
    let nx = pos - Vec3::X * s;
    let pz = pos + Vec3::Z * s;
    let nz = pos - Vec3::Z * s;

    let pts = [(top, px, pz), (top, pz, nx), (top, nx, nz), (top, nz, px),
               (btm, pz, px), (btm, nx, pz), (btm, nz, nx), (btm, px, nz)];

    for (p0, p1, p2) in pts {
        let n = ((p1 - p0).cross(p2 - p0)).normalize_or_zero();
        triangles.push(Triangle::new(p0, p1, p2, n, n, n, Vec2::ZERO, Vec2::X, Vec2::ONE, mat_id));
    }
}

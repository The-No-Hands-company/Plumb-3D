/// Light emitter types supported in the DCC viewport and scene graph
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightType {
    Directional,
    Point,
    Spot,
}

/// Standard DCC PBR Material with Albedo, Metallic, Roughness, Emissive, and Procedural Patterns
#[derive(Debug, Clone, PartialEq)]
pub struct Material {
    pub name: String,
    pub base_color: [f32; 4],
    pub metallic: f32,
    pub roughness: f32,
    pub emissive: f32,
    pub texture_pattern: u32,
    pub albedo_texture_path: Option<String>,
    pub normal_texture_path: Option<String>,
    pub roughness_metallic_texture_path: Option<String>,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            name: "Material.001".into(),
            base_color: [0.72, 0.73, 0.75, 1.0],
            metallic: 0.0,
            roughness: 0.5,
            emissive: 0.0,
            texture_pattern: 0,
            albedo_texture_path: None,
            normal_texture_path: None,
            roughness_metallic_texture_path: None,
        }
    }
}

impl Material {
    pub fn new(
        name: impl Into<String>,
        base_color: [f32; 4],
        metallic: f32,
        roughness: f32,
        emissive: f32,
        texture_pattern: u32,
    ) -> Self {
        Self {
            name: name.into(),
            base_color,
            metallic: metallic.clamp(0.0, 1.0),
            roughness: roughness.clamp(0.04, 1.0),
            emissive: emissive.max(0.0),
            texture_pattern,
            albedo_texture_path: None,
            normal_texture_path: None,
            roughness_metallic_texture_path: None,
        }
    }

    pub fn with_albedo_texture(mut self, path: impl Into<String>) -> Self {
        self.albedo_texture_path = Some(path.into());
        self
    }

    pub fn with_normal_texture(mut self, path: impl Into<String>) -> Self {
        self.normal_texture_path = Some(path.into());
        self
    }

    pub fn with_roughness_metallic_texture(mut self, path: impl Into<String>) -> Self {
        self.roughness_metallic_texture_path = Some(path.into());
        self
    }

    pub fn gold(name: impl Into<String>) -> Self {
        Self::new(name, [1.0, 0.766, 0.336, 1.0], 1.0, 0.25, 0.0, 0)
    }

    pub fn copper(name: impl Into<String>) -> Self {
        Self::new(name, [0.955, 0.637, 0.538, 1.0], 1.0, 0.30, 0.0, 0)
    }

    pub fn chrome(name: impl Into<String>) -> Self {
        Self::new(name, [0.95, 0.95, 0.95, 1.0], 1.0, 0.06, 0.0, 0)
    }

    pub fn plastic(name: impl Into<String>) -> Self {
        Self::new(name, [0.2, 0.55, 0.85, 1.0], 0.0, 0.35, 0.0, 0)
    }

    pub fn rubber(name: impl Into<String>) -> Self {
        Self::new(name, [0.12, 0.12, 0.12, 1.0], 0.0, 0.85, 0.0, 0)
    }

    pub fn clay(name: impl Into<String>) -> Self {
        Self::new(name, [0.73, 0.45, 0.35, 1.0], 0.0, 0.65, 0.0, 0)
    }

    pub fn glass(name: impl Into<String>) -> Self {
        Self::new(name, [0.9, 0.95, 1.0, 0.4], 0.1, 0.05, 0.0, 0)
    }

    pub fn ruby(name: impl Into<String>) -> Self {
        Self::new(name, [0.85, 0.05, 0.15, 1.0], 0.3, 0.1, 0.1, 0)
    }

    pub fn emerald(name: impl Into<String>) -> Self {
        Self::new(name, [0.08, 0.75, 0.35, 1.0], 0.2, 0.15, 0.05, 0)
    }
}

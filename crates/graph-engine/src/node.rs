use core_math::{Mat4, Transform, Vec3};
use geo_kernel::{Mesh, ModifierStack, ProfileShape, RenderMesh, ShadingMode, Spline};
use slotmap::new_key_type;

use crate::material::{LightType, Material};

new_key_type! {
    pub struct NodeId;
}

/// Type of entity represented in the scene DAG
#[derive(Debug)]
pub enum NodeData {
    Transform,
    Mesh {
        base_mesh: Mesh,
        modifier_stack: ModifierStack,
        shading_mode: ShadingMode,
        cached_render_mesh: Option<RenderMesh>,
        materials: Vec<Material>,
    },
    Camera {
        fov_y_radians: f32,
        z_near: f32,
        z_far: f32,
        aspect_ratio: f32,
        orthographic: bool,
        ortho_size: f32,
    },
    Light {
        light_type: LightType,
        color: Vec3,
        intensity: f32,
        range: f32,
        spot_inner_angle_radians: f32,
        spot_outer_angle_radians: f32,
    },
    Joint {
        length: f32,
        radius: f32,
    },
    Curve {
        spline: Spline,
        bevel_radius: f32,
        bevel_resolution: usize,
        cached_render_mesh: Option<RenderMesh>,
        profile_shape: ProfileShape,
    },
}

/// A node in the Maya-style Directed Acyclic Graph (DAG) scene hierarchy
#[derive(Debug)]
pub struct SceneNode {
    pub name: String,
    pub local_transform: Transform,
    pub world_matrix: Mat4,
    pub parent: Option<NodeId>,
    pub children: Vec<NodeId>,
    pub data: NodeData,
    /// Maya-style dirty bit for caching world transforms and geometry evaluation
    pub is_dirty: bool,
    pub visible: bool,
}

impl SceneNode {
    pub fn new_transform(name: impl Into<String>, transform: Transform) -> Self {
        Self {
            name: name.into(),
            local_transform: transform,
            world_matrix: transform.to_matrix(),
            parent: None,
            children: Vec::new(),
            data: NodeData::Transform,
            is_dirty: true,
            visible: true,
        }
    }

    pub fn new_mesh(name: impl Into<String>, transform: Transform, base_mesh: Mesh) -> Self {
        Self {
            name: name.into(),
            local_transform: transform,
            world_matrix: transform.to_matrix(),
            parent: None,
            children: Vec::new(),
            data: NodeData::Mesh {
                base_mesh,
                modifier_stack: ModifierStack::new(),
                shading_mode: ShadingMode::Flat,
                cached_render_mesh: None,
                materials: vec![Material::default()],
            },
            is_dirty: true,
            visible: true,
        }
    }

    pub fn new_camera(
        name: impl Into<String>,
        transform: Transform,
        fov_y: f32,
        aspect: f32,
    ) -> Self {
        Self {
            name: name.into(),
            local_transform: transform,
            world_matrix: transform.to_matrix(),
            parent: None,
            children: Vec::new(),
            data: NodeData::Camera {
                fov_y_radians: fov_y,
                z_near: 0.1,
                z_far: 1000.0,
                aspect_ratio: aspect,
                orthographic: false,
                ortho_size: 10.0,
            },
            is_dirty: true,
            visible: true,
        }
    }

    pub fn new_directional_light(
        name: impl Into<String>,
        transform: Transform,
        color: Vec3,
        intensity: f32,
    ) -> Self {
        Self {
            name: name.into(),
            local_transform: transform,
            world_matrix: transform.to_matrix(),
            parent: None,
            children: Vec::new(),
            data: NodeData::Light {
                light_type: LightType::Directional,
                color,
                intensity,
                range: 100.0,
                spot_inner_angle_radians: 0.0,
                spot_outer_angle_radians: 0.0,
            },
            is_dirty: true,
            visible: true,
        }
    }

    pub fn new_point_light(
        name: impl Into<String>,
        transform: Transform,
        color: Vec3,
        intensity: f32,
        range: f32,
    ) -> Self {
        Self {
            name: name.into(),
            local_transform: transform,
            world_matrix: transform.to_matrix(),
            parent: None,
            children: Vec::new(),
            data: NodeData::Light {
                light_type: LightType::Point,
                color,
                intensity,
                range,
                spot_inner_angle_radians: 0.0,
                spot_outer_angle_radians: 0.0,
            },
            is_dirty: true,
            visible: true,
        }
    }

    pub fn new_spot_light(
        name: impl Into<String>,
        transform: Transform,
        color: Vec3,
        intensity: f32,
        range: f32,
        inner_angle: f32,
        outer_angle: f32,
    ) -> Self {
        Self {
            name: name.into(),
            local_transform: transform,
            world_matrix: transform.to_matrix(),
            parent: None,
            children: Vec::new(),
            data: NodeData::Light {
                light_type: LightType::Spot,
                color,
                intensity,
                range,
                spot_inner_angle_radians: inner_angle,
                spot_outer_angle_radians: outer_angle,
            },
            is_dirty: true,
            visible: true,
        }
    }

    pub fn new_joint(
        name: impl Into<String>,
        transform: Transform,
        length: f32,
        radius: f32,
    ) -> Self {
        Self {
            name: name.into(),
            local_transform: transform,
            world_matrix: transform.to_matrix(),
            parent: None,
            children: Vec::new(),
            data: NodeData::Joint {
                length: length.max(0.01),
                radius: radius.max(0.005),
            },
            is_dirty: true,
            visible: true,
        }
    }

    pub fn new_curve(name: impl Into<String>, transform: Transform, spline: Spline) -> Self {
        Self {
            name: name.into(),
            local_transform: transform,
            world_matrix: transform.to_matrix(),
            parent: None,
            children: Vec::new(),
            data: NodeData::Curve {
                spline,
                bevel_radius: 0.05,
                bevel_resolution: 8,
                cached_render_mesh: None,
                profile_shape: ProfileShape::Circle,
            },
            is_dirty: true,
            visible: true,
        }
    }

    /// Duplicate this node with a new name and a translational offset.
    pub fn duplicate(&self, name: impl Into<String>, offset: Vec3) -> Self {
        let mut new_transform = self.local_transform;
        new_transform.translation += offset;
        let data = match &self.data {
            NodeData::Transform => NodeData::Transform,
            NodeData::Mesh {
                base_mesh,
                shading_mode,
                materials,
                ..
            } => NodeData::Mesh {
                base_mesh: base_mesh.clone(),
                modifier_stack: ModifierStack::new(),
                shading_mode: *shading_mode,
                cached_render_mesh: None,
                materials: materials.clone(),
            },
            NodeData::Camera {
                fov_y_radians,
                z_near,
                z_far,
                aspect_ratio,
                orthographic,
                ortho_size,
            } => NodeData::Camera {
                fov_y_radians: *fov_y_radians,
                z_near: *z_near,
                z_far: *z_far,
                aspect_ratio: *aspect_ratio,
                orthographic: *orthographic,
                ortho_size: *ortho_size,
            },
            NodeData::Light {
                light_type,
                color,
                intensity,
                range,
                spot_inner_angle_radians,
                spot_outer_angle_radians,
            } => NodeData::Light {
                light_type: *light_type,
                color: *color,
                intensity: *intensity,
                range: *range,
                spot_inner_angle_radians: *spot_inner_angle_radians,
                spot_outer_angle_radians: *spot_outer_angle_radians,
            },
            NodeData::Joint { length, radius } => NodeData::Joint {
                length: *length,
                radius: *radius,
            },
            NodeData::Curve {
                spline,
                bevel_radius,
                bevel_resolution,
                profile_shape,
                ..
            } => NodeData::Curve {
                spline: spline.clone(),
                bevel_radius: *bevel_radius,
                bevel_resolution: *bevel_resolution,
                cached_render_mesh: None,
                profile_shape: *profile_shape,
            },
        };

        Self {
            name: name.into(),
            local_transform: new_transform,
            world_matrix: new_transform.to_matrix(),
            parent: self.parent,
            children: Vec::new(),
            data,
            is_dirty: true,
            visible: self.visible,
        }
    }
}

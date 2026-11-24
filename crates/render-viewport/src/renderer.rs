use crate::camera::ViewportCamera;
use crate::pipeline;
pub use crate::types::*;
use geo_kernel::RenderMesh;
use std::sync::Arc;
use wgpu::util::DeviceExt;

pub struct ViewportRenderer {
    pub device: Arc<wgpu::Device>,
    pub queue: Arc<wgpu::Queue>,
    mesh_pipeline: wgpu::RenderPipeline,
    wireframe_pipeline: wgpu::RenderPipeline,
    pub(crate) gizmo_pipeline: wgpu::RenderPipeline,
    shadow_pipeline: wgpu::RenderPipeline,
    grid_pipeline: wgpu::RenderPipeline,
    _camera_bind_group_layout: wgpu::BindGroupLayout,
    pub(crate) model_bind_group_layout: wgpu::BindGroupLayout,
    pub texture_bind_group_layout: wgpu::BindGroupLayout,
    pub default_texture_bind_group: wgpu::BindGroup,
    camera_buffer: wgpu::Buffer,
    pub(crate) camera_bind_group: wgpu::BindGroup,
    lights_buffer: wgpu::Buffer,
    lights_bind_group: wgpu::BindGroup,
    grid_vertex_buffer: wgpu::Buffer,
    _shadow_texture: wgpu::Texture,
    shadow_view: wgpu::TextureView,
}

impl ViewportRenderer {
    pub fn new(
        device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>,
        surface_format: wgpu::TextureFormat,
    ) -> Self {
        let (camera_layout, camera_buffer, camera_bind_group) =
            pipeline::create_camera_resources(&device);
        let model_bind_group_layout = pipeline::create_model_layout(&device);
        let texture_bind_group_layout = pipeline::create_texture_layout(&device);
        let vertex_layout = pipeline::create_vertex_layout();

        // High-resolution 2048x2048 shadow depth map & PCF comparison sampler
        let shadow_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Shadow Map Texture"),
            size: wgpu::Extent3d {
                width: 2048,
                height: 2048,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let shadow_view = shadow_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let shadow_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Shadow Comparison Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: Some(wgpu::CompareFunction::LessEqual),
            ..Default::default()
        });

        let (lights_layout, lights_buffer, lights_bind_group) =
            pipeline::create_lights_resources(&device, &shadow_view, &shadow_sampler);

        let mesh_pipeline = pipeline::create_mesh_pipeline(
            &device,
            surface_format,
            &camera_layout,
            &model_bind_group_layout,
            &lights_layout,
            &texture_bind_group_layout,
            &vertex_layout,
        );

        let wireframe_pipeline = pipeline::create_wireframe_pipeline(
            &device,
            surface_format,
            &camera_layout,
            &model_bind_group_layout,
            &vertex_layout,
        );

        let gizmo_pipeline = pipeline::create_gizmo_pipeline(
            &device,
            surface_format,
            &camera_layout,
            &model_bind_group_layout,
            &vertex_layout,
        );

        let shadow_pipeline = pipeline::create_shadow_pipeline(
            &device,
            &camera_layout,
            &model_bind_group_layout,
            &vertex_layout,
        );

        let (grid_pipeline, grid_vertex_buffer) =
            pipeline::create_grid_pipeline(&device, surface_format, &camera_layout);

        // Create 1x1 default textures: white albedo, flat normal [128, 128, 255, 255], and white roughness/metallic
        let albedo_tex = device.create_texture_with_data(
            &queue,
            &wgpu::TextureDescriptor {
                label: Some("Default Albedo Texture"),
                size: wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            },
            wgpu::util::TextureDataOrder::LayerMajor,
            &[255, 255, 255, 255],
        );
        let albedo_view = albedo_tex.create_view(&wgpu::TextureViewDescriptor::default());

        let normal_tex = device.create_texture_with_data(
            &queue,
            &wgpu::TextureDescriptor {
                label: Some("Default Normal Texture"),
                size: wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            },
            wgpu::util::TextureDataOrder::LayerMajor,
            &[128, 128, 255, 255],
        );
        let normal_view = normal_tex.create_view(&wgpu::TextureViewDescriptor::default());

        let rm_tex = device.create_texture_with_data(
            &queue,
            &wgpu::TextureDescriptor {
                label: Some("Default Roughness Metallic Texture"),
                size: wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            },
            wgpu::util::TextureDataOrder::LayerMajor,
            &[255, 255, 255, 255],
        );
        let rm_view = rm_tex.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Default Texture Sampler"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let default_texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Default Texture Bind Group"),
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&albedo_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&normal_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(&rm_view),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        Self {
            device,
            queue,
            mesh_pipeline,
            wireframe_pipeline,
            gizmo_pipeline,
            shadow_pipeline,
            grid_pipeline,
            _camera_bind_group_layout: camera_layout,
            model_bind_group_layout,
            texture_bind_group_layout,
            default_texture_bind_group,
            camera_buffer,
            camera_bind_group,
            lights_buffer,
            lights_bind_group,
            grid_vertex_buffer,
            _shadow_texture: shadow_texture,
            shadow_view,
        }
    }

    /// Upload a CPU RenderMesh into GPU buffers
    pub fn upload_mesh(&self, render_mesh: &RenderMesh) -> GpuMesh {
        let vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Mesh Vertex Buffer"),
            contents: bytemuck::cast_slice(&render_mesh.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Mesh Index Buffer"),
            contents: bytemuck::cast_slice(&render_mesh.indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let wireframe_index_buffer =
            self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Wireframe Index Buffer"),
                contents: bytemuck::cast_slice(&render_mesh.wireframe_indices),
                usage: wgpu::BufferUsages::INDEX,
            });

        GpuMesh {
            vertex_buffer,
            index_buffer,
            index_count: render_mesh.indices.len() as u32,
            wireframe_index_buffer,
            wireframe_index_count: render_mesh.wireframe_indices.len() as u32,
            submeshes: render_mesh.submeshes.clone(),
        }
    }

    /// Render the viewport scene
    pub fn render_scene(
        &self,
        target_view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        camera: &ViewportCamera,
        meshes: &[MeshDrawCall],
        lights: &[GpuLight],
        show_grid: bool,
        shading_mode: ViewportShadingMode,
        exposure: f32,
        tonemapping: TonemappingMode,
        cavity_intensity: f32,
    ) {
        // Compute directional light view-projection matrix for shadows
        let sun_dir = lights
            .iter()
            .find(|l| l.position[3] == 0.0 && l.spot_params[2] > 0.0)
            .map(|l| core_math::Vec3::new(l.direction[0], l.direction[1], l.direction[2]).normalize_or_zero())
            .filter(|d| d.length_squared() > 0.1)
            .map(|d| if d.y > 0.0 { d } else { -d })
            .unwrap_or_else(|| core_math::Vec3::new(0.577, 0.707, 0.408).normalize());

        let center = camera.target;
        let light_pos = center + sun_dir * 30.0;
        let light_view = core_math::Mat4::look_at_rh(light_pos, center, core_math::Vec3::Y);
        let light_proj = core_math::Mat4::orthographic_rh(-20.0, 20.0, -20.0, 20.0, 1.0, 70.0);
        let light_view_proj = light_proj * light_view;

        // Update camera uniform
        let eye = camera.eye_position();
        let camera_data = CameraUniform {
            view_proj: camera.view_projection_matrix().to_cols_array(),
            eye_pos: [eye.x, eye.y, eye.z, 1.0],
            viewport_params: [
                shading_mode.as_shader_id(),
                exposure.max(0.01),
                tonemapping.as_shader_id(),
                cavity_intensity.max(0.0),
            ],
            light_view_proj: light_view_proj.to_cols_array(),
        };
        self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::bytes_of(&camera_data));

        // Update lights uniform
        let mut lights_data = LightsUniform::default();
        let num_lights = lights.len().min(MAX_SCENE_LIGHTS);
        lights_data.light_count[0] = num_lights as u32;
        for (i, &l) in lights.iter().take(MAX_SCENE_LIGHTS).enumerate() {
            lights_data.lights[i] = l;
        }
        self.queue.write_buffer(&self.lights_buffer, 0, bytemuck::bytes_of(&lights_data));

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Viewport Render Encoder"),
        });

        // 1. Directional Shadow Pass (depth-only)
        {
            let mut shadow_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Viewport Shadow Pass"),
                color_attachments: &[],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.shadow_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            shadow_pass.set_pipeline(&self.shadow_pipeline);
            shadow_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            for item in meshes {
                let model_data = ModelUniform {
                    model: item.model_matrix.to_cols_array(),
                    color: item.color,
                    material: item.material,
                };
                let model_buffer =
                    self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Shadow Model Buffer"),
                        contents: bytemuck::bytes_of(&model_data),
                        usage: wgpu::BufferUsages::UNIFORM,
                    });
                let model_bind_group =
                    self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("Shadow Model Bind Group"),
                        layout: &self.model_bind_group_layout,
                        entries: &[wgpu::BindGroupEntry {
                            binding: 0,
                            resource: model_buffer.as_entire_binding(),
                        }],
                    });
                shadow_pass.set_bind_group(1, &model_bind_group, &[]);
                shadow_pass.set_vertex_buffer(0, item.mesh.vertex_buffer.slice(..));
                shadow_pass.set_index_buffer(item.mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                shadow_pass.draw_indexed(0..item.mesh.index_count, 0, 0..1);
            }
        }

        // 2. Main Color Pass
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Viewport Main Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.18,
                            g: 0.19,
                            b: 0.21,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            // 1. Render Ground Grid
            if show_grid {
                pass.set_pipeline(&self.grid_pipeline);
                pass.set_bind_group(0, &self.camera_bind_group, &[]);
                pass.set_vertex_buffer(0, self.grid_vertex_buffer.slice(..));
                pass.draw(0..6, 0..1);
            }

            // 2. Render Meshes
            for item in meshes {
                let mesh = item.mesh;
                let model_matrix = item.model_matrix;

                // Shaded Pass
                pass.set_pipeline(&self.mesh_pipeline);
                pass.set_bind_group(0, &self.camera_bind_group, &[]);
                pass.set_bind_group(2, &self.lights_bind_group, &[]);
                let tex_bg = item.texture_bind_group.unwrap_or(&self.default_texture_bind_group);
                pass.set_bind_group(3, tex_bg, &[]);
                pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

                if !mesh.submeshes.is_empty() && !item.submesh_materials.is_empty() {
                    for submesh in &mesh.submeshes {
                        let sm = item
                            .submesh_materials
                            .get(submesh.material_id as usize)
                            .copied()
                            .unwrap_or(SubMeshMaterial {
                                color: item.color,
                                material: item.material,
                            });

                        let model_data = ModelUniform {
                            model: model_matrix.to_cols_array(),
                            color: sm.color,
                            material: sm.material,
                        };
                        let model_buffer =
                            self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                                label: Some("Temp SubMesh Model Buffer"),
                                contents: bytemuck::bytes_of(&model_data),
                                usage: wgpu::BufferUsages::UNIFORM,
                            });
                        let model_bind_group =
                            self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                                label: Some("Temp SubMesh Model Bind Group"),
                                layout: &self.model_bind_group_layout,
                                entries: &[wgpu::BindGroupEntry {
                                    binding: 0,
                                    resource: model_buffer.as_entire_binding(),
                                }],
                            });
                        pass.set_bind_group(1, &model_bind_group, &[]);
                        pass.draw_indexed(
                            submesh.index_start..(submesh.index_start + submesh.index_count),
                            0,
                            0..1,
                        );
                    }
                } else {
                    let model_data = ModelUniform {
                        model: model_matrix.to_cols_array(),
                        color: item.color,
                        material: item.material,
                    };
                    let model_buffer =
                        self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Temp Model Buffer"),
                            contents: bytemuck::bytes_of(&model_data),
                            usage: wgpu::BufferUsages::UNIFORM,
                        });
                    let model_bind_group =
                        self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                            label: Some("Temp Model Bind Group"),
                            layout: &self.model_bind_group_layout,
                            entries: &[wgpu::BindGroupEntry {
                                binding: 0,
                                resource: model_buffer.as_entire_binding(),
                            }],
                        });
                    pass.set_bind_group(1, &model_bind_group, &[]);
                    pass.draw_indexed(0..mesh.index_count, 0, 0..1);
                }

                // Wireframe Pass
                if item.show_wireframe && mesh.wireframe_index_count > 0 {
                    let wire_data = ModelUniform {
                        model: model_matrix.to_cols_array(),
                        color: [0.15, 0.15, 0.15, 1.0], // Dark wireframe lines
                        material: [0.0, 1.0, 0.0, 0.0],
                    };
                    let wire_buffer =
                        self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Wireframe Model Buffer"),
                            contents: bytemuck::bytes_of(&wire_data),
                            usage: wgpu::BufferUsages::UNIFORM,
                        });
                    let wire_bind_group =
                        self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                            label: Some("Wireframe Model Bind Group"),
                            layout: &self.model_bind_group_layout,
                            entries: &[wgpu::BindGroupEntry {
                                binding: 0,
                                resource: wire_buffer.as_entire_binding(),
                            }],
                        });

                    pass.set_pipeline(&self.wireframe_pipeline);
                    pass.set_bind_group(0, &self.camera_bind_group, &[]);
                    pass.set_bind_group(1, &wire_bind_group, &[]);
                    pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                    pass.set_index_buffer(
                        mesh.wireframe_index_buffer.slice(..),
                        wgpu::IndexFormat::Uint32,
                    );
                    pass.draw_indexed(0..mesh.wireframe_index_count, 0, 0..1);
                }
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
    }

    /// Uploads RGBA8 pixel data to a 2D GPU texture and returns the texture and its default view.
    pub fn create_texture_from_rgba(
        &self,
        width: u32,
        height: u32,
        rgba: &[u8],
        label: &str,
    ) -> (wgpu::Texture, wgpu::TextureView) {
        let size = wgpu::Extent3d {
            width: width.max(1),
            height: height.max(1),
            depth_or_array_layers: 1,
        };
        let texture = self.device.create_texture_with_data(
            &self.queue,
            &wgpu::TextureDescriptor {
                label: Some(label),
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            },
            wgpu::util::TextureDataOrder::LayerMajor,
            rgba,
        );
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        (texture, view)
    }

    /// Creates a PBR texture bind group for custom albedo, normal, and roughness/metallic views.
    pub fn create_pbr_texture_bind_group(
        &self,
        albedo_view: &wgpu::TextureView,
        normal_view: &wgpu::TextureView,
        rm_view: &wgpu::TextureView,
    ) -> wgpu::BindGroup {
        let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("PBR Material Texture Sampler"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Custom PBR Texture Bind Group"),
            layout: &self.texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(albedo_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(normal_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(rm_view),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        })
    }
}

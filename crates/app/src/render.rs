use core_math::Vec3;
use egui_wgpu::ScreenDescriptor;
use graph_engine::{Constraint, LightType, NodeData, NodeId};
use render_viewport::{GpuLight, MeshDrawCall, SubMeshMaterial};
use std::collections::HashMap;
use std::time::Instant;

use crate::state::AppState;
use crate::types::{InteractionMode, SubElementLevel};

pub fn render_frame(state: &mut AppState) {
    let now = Instant::now();
    let delta = now.duration_since(state.last_frame_time).as_secs_f32();
    state.last_frame_time = now;
    state.fps = state.fps * 0.9 + (1.0 / delta.max(0.001)) * 0.1;

    if state.animation.is_playing {
        state.animation.step(delta, &mut state.scene);
        let frame = state.animation.current_frame;
        let total = (state.animation.end_frame - state.animation.start_frame).max(1.0);
        let u = ((frame - state.animation.start_frame) / total).rem_euclid(1.0);
        for c in &mut state.constraints.constraints {
            if let Constraint::Path(p) = c {
                p.u = u;
            }
        }

        // Advance dynamic physics simulation modifiers (Cloth, Particles)
        let mut dirty_sim_nodes = Vec::new();
        for (node_id, node) in &mut state.scene.nodes {
            if let NodeData::Mesh { modifier_stack, base_mesh, .. } = &mut node.data {
                let mut stepped_sim = false;
                for m in modifier_stack.modifiers_mut() {
                    if let Some(cloth) = m.as_cloth_mut() {
                        if !cloth.is_initialized {
                            cloth.initialize(base_mesh);
                        }
                        cloth.step(delta.min(0.05));
                        stepped_sim = true;
                    }
                    if let Some(particles) = m.as_particle_system_mut() {
                        if !particles.is_initialized {
                            particles.reset();
                        }
                        particles.step(delta.min(0.05), base_mesh);
                        stepped_sim = true;
                    }
                }
                if stepped_sim {
                    dirty_sim_nodes.push(node_id);
                }
            }
        }
        for node_id in dirty_sim_nodes {
            state.scene.mark_dirty(node_id);
        }

        // Advance Rigid Body Dynamics World
        if state.physics.is_simulating && !state.physics.bodies.is_empty() {
            state.physics.step(&mut state.scene, delta.min(0.05));
        }
    }

    state.constraints.evaluate(&mut state.scene);

    state.sync_gpu_meshes();

    let output = match state.surface.get_current_texture() {
        Ok(tex) => tex,
        Err(wgpu::SurfaceError::Lost) => {
            state.surface.configure(&state.device, &state.surface_config);
            return;
        }
        Err(wgpu::SurfaceError::OutOfMemory) => {
            eprintln!("Out of memory");
            return;
        }
        Err(e) => {
            eprintln!("Surface error: {:?}", e);
            return;
        }
    };

    let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

    // Update active scene camera override
    if let Some(cam_id) = state.active_camera_node {
        if let Some(cam_node) = state.scene.nodes.get(cam_id) {
            if let NodeData::Camera { fov_y_radians, z_near, z_far, aspect_ratio, orthographic, ortho_size } = cam_node.data {
                let eye = cam_node.world_matrix.transform_point3(Vec3::ZERO);
                let view_mat = cam_node.world_matrix.inverse();
                let proj = if orthographic {
                    let h = ortho_size * 0.5;
                    let w = h * aspect_ratio;
                    core_math::Mat4::orthographic_rh(-w, w, -h, h, z_near, z_far)
                } else {
                    core_math::Mat4::perspective_rh(fov_y_radians, aspect_ratio, z_near, z_far)
                };
                state.camera.custom_view_proj = Some((proj * view_mat, eye));
            } else {
                state.camera.custom_view_proj = None;
            }
        } else {
            state.camera.custom_view_proj = None;
        }
    } else {
        state.camera.custom_view_proj = None;
    }

    let mut submesh_materials_storage: HashMap<NodeId, Vec<SubMeshMaterial>> = HashMap::new();
    for (node_id, node) in &state.scene.nodes {
        if let NodeData::Mesh { materials, .. } = &node.data {
            let sm_mats: Vec<SubMeshMaterial> = materials
                .iter()
                .map(|m| SubMeshMaterial {
                    color: m.base_color,
                    material: [m.metallic, m.roughness, m.emissive, m.texture_pattern as f32],
                })
                .collect();
            submesh_materials_storage.insert(node_id, sm_mats);
        }
    }

    // Ensure GPU textures are created and cached for nodes with custom textures
    let mut custom_texture_requests = Vec::new();
    for (_node_id, node) in &state.scene.nodes {
        if let NodeData::Mesh { materials, .. } = &node.data {
            for m in materials {
                if m.albedo_texture_path.is_some() || m.normal_texture_path.is_some() || m.roughness_metallic_texture_path.is_some() {
                    let key = format!("{:?}|{:?}|{:?}", m.albedo_texture_path, m.normal_texture_path, m.roughness_metallic_texture_path);
                    if !state.gpu_texture_bind_groups.contains_key(&key) {
                        custom_texture_requests.push((key, m.albedo_texture_path.clone(), m.normal_texture_path.clone(), m.roughness_metallic_texture_path.clone()));
                    }
                }
            }
        }
    }

    for (key, albedo_p, norm_p, rm_p) in custom_texture_requests {
        let (_albedo_tex, albedo_view) = if let Some(p) = albedo_p.and_then(|p| image::open(p).ok()) {
            let rgba = p.to_rgba8();
            state.renderer.create_texture_from_rgba(rgba.width(), rgba.height(), &rgba, "Custom Albedo")
        } else {
            state.renderer.create_texture_from_rgba(1, 1, &[255, 255, 255, 255], "White Albedo")
        };

        let (_norm_tex, norm_view) = if let Some(p) = norm_p.and_then(|p| image::open(p).ok()) {
            let rgba = p.to_rgba8();
            state.renderer.create_texture_from_rgba(rgba.width(), rgba.height(), &rgba, "Custom Normal")
        } else {
            state.renderer.create_texture_from_rgba(1, 1, &[128, 128, 255, 255], "Flat Normal")
        };

        let (_rm_tex, rm_view) = if let Some(p) = rm_p.and_then(|p| image::open(p).ok()) {
            let rgba = p.to_rgba8();
            state.renderer.create_texture_from_rgba(rgba.width(), rgba.height(), &rgba, "Custom RM")
        } else {
            state.renderer.create_texture_from_rgba(1, 1, &[255, 255, 255, 255], "White RM")
        };

        let bg = state.renderer.create_pbr_texture_bind_group(&albedo_view, &norm_view, &rm_view);
        state.gpu_texture_bind_groups.insert(key, bg);
    }

    let mut meshes_to_render = Vec::new();
    let mut scene_gpu_lights = Vec::new();
    let mut light_glyphs = Vec::new();
    let mut camera_glyphs = Vec::new();
    let mut joint_glyphs = Vec::new();
    let mut curve_glyphs = Vec::new();
    let gizmo_pos = state.get_gizmo_world_position();

    for (node_id, node) in &state.scene.nodes {
        if !node.visible {
            continue;
        }
        let is_active = Some(node_id) == state.selected_node;
        let is_selected = is_active || state.selected_nodes.contains(&node_id);
        match &node.data {
            NodeData::Mesh { modifier_stack, .. } => {
                if let Some(gpu_mesh) = state.gpu_mesh_cache.get(&node_id) {
                    let has_subdiv = modifier_stack.modifiers().iter().any(|m| {
                        m.is_enabled() && (m.name() == "Subdivide" || m.name() == "Catmull-Clark")
                    });
                    let show_wf = if state.optimal_display && has_subdiv {
                        false
                    } else {
                        state.show_wireframe || is_selected
                    };
                    let base_rgb = state.object_colors.get(&node_id).copied().unwrap_or([0.72, 0.73, 0.75]);
                    let color = if is_active {
                        if state.interaction_mode == InteractionMode::Edit {
                            [0.92, 0.52, 0.32, 1.0]
                        } else {
                            [0.92, 0.72, 0.35, 1.0]
                        }
                    } else if is_selected {
                        [0.85, 0.50, 0.22, 1.0]
                    } else {
                        [base_rgb[0], base_rgb[1], base_rgb[2], 1.0]
                    };
                    let base_mat = state.object_materials.get(&node_id).copied().unwrap_or([0.0, 0.5, 0.0, 0.0]);
                    let sub_mats = submesh_materials_storage
                        .get(&node_id)
                        .map(|v| v.as_slice())
                        .unwrap_or(&[]);

                    let tex_bg = if let NodeData::Mesh { materials, .. } = &node.data {
                        materials.first().and_then(|m| {
                            let key = format!("{:?}|{:?}|{:?}", m.albedo_texture_path, m.normal_texture_path, m.roughness_metallic_texture_path);
                            state.gpu_texture_bind_groups.get(&key)
                        })
                    } else {
                        None
                    };

                    meshes_to_render.push(MeshDrawCall {
                        mesh: gpu_mesh,
                        model_matrix: node.world_matrix,
                        color,
                        material: base_mat,
                        show_wireframe: show_wf,
                        submesh_materials: sub_mats,
                        texture_bind_group: tex_bg,
                    });
                }
            }
            NodeData::Light { light_type, color, intensity, range, spot_inner_angle_radians, spot_outer_angle_radians } => {
                let world_pos = node.world_matrix.transform_point3(Vec3::ZERO);
                let forward_dir = -node.world_matrix.transform_vector3(Vec3::Z).normalize();
                let type_id = match light_type {
                    LightType::Directional => 0u32,
                    LightType::Point => 1u32,
                    LightType::Spot => 2u32,
                };
                scene_gpu_lights.push(GpuLight {
                    position: [world_pos.x, world_pos.y, world_pos.z, type_id as f32],
                    direction: [forward_dir.x, forward_dir.y, forward_dir.z, *range],
                    color: [color.x, color.y, color.z, *intensity],
                    spot_params: [spot_inner_angle_radians.cos(), spot_outer_angle_radians.cos(), 1.0, 0.0],
                });
                light_glyphs.push((world_pos, forward_dir, [color.x, color.y, color.z, 1.0], type_id, is_selected));
            }
            NodeData::Camera { fov_y_radians, aspect_ratio, .. } => {
                if Some(node_id) != state.active_camera_node {
                    camera_glyphs.push((node.world_matrix, *fov_y_radians, *aspect_ratio, is_selected));
                }
            }
            NodeData::Joint { length, radius } => {
                joint_glyphs.push((node.world_matrix, *length, *radius, is_selected));
            }
            NodeData::Curve { spline, bevel_radius, .. } => {
                if *bevel_radius > 0.0 {
                    if let Some(gpu_mesh) = state.gpu_mesh_cache.get(&node_id) {
                        let base_rgb = state.object_colors.get(&node_id).copied().unwrap_or([0.3, 0.7, 0.95]);
                        let color = if is_selected {
                            [0.88, 0.68, 0.38, 1.0]
                        } else {
                            [base_rgb[0], base_rgb[1], base_rgb[2], 1.0]
                        };
                        let base_mat = state.object_materials.get(&node_id).copied().unwrap_or([0.0, 0.4, 0.0, 0.0]);
                        meshes_to_render.push(MeshDrawCall {
                            mesh: gpu_mesh,
                            model_matrix: node.world_matrix,
                            color,
                            material: base_mat,
                            show_wireframe: state.show_wireframe,
                            submesh_materials: &[],
                            texture_bind_group: None,
                        });
                    }
                }
                let sampled = spline.sample(16);
                let knots = spline.points.iter().map(|p| (p.position, p.handle_in, p.handle_out)).collect();
                curve_glyphs.push((node.world_matrix, sampled, knots, spline.is_closed, is_selected));
            }
            _ => {}
        }
    }

    // 1. Viewport 3D Geometry Pass
    let cavity = if state.show_cavity { state.cavity_intensity } else { 0.0 };
    state.renderer.render_scene(
        &view,
        &state.depth_view,
        &state.camera,
        &meshes_to_render,
        &scene_gpu_lights,
        state.show_grid,
        state.viewport_shading,
        state.exposure,
        state.tonemapping,
        cavity,
    );

    // 2. Scene Actor Glyphs Pass (Lights, Cameras, Bones, Curves)
    state.renderer.render_actor_glyphs(
        &view,
        &state.depth_view,
        &light_glyphs,
        &camera_glyphs,
        &joint_glyphs,
        &curve_glyphs,
    );

    // 3. Sub-Element Selection Overlay Pass (Edit Mode)
    if state.interaction_mode == InteractionMode::Edit {
        if let Some(sel_id) = state.selected_node {
            if let Some(node) = state.scene.nodes.get(sel_id) {
                if let NodeData::Mesh { base_mesh, .. } = &node.data {
                    let mut all_verts = Vec::new();
                    if state.sub_element_level == SubElementLevel::Vertex {
                        for v in base_mesh.vertices.values() {
                            all_verts.push(node.world_matrix.transform_point3(v.position));
                        }
                    }

                    let sel_v_positions: Vec<Vec3> = state
                        .selected_vertices
                        .iter()
                        .filter_map(|&vk| {
                            base_mesh
                                .vertices
                                .get(vk)
                                .map(|v| node.world_matrix.transform_point3(v.position))
                        })
                        .collect();

                    let sel_edges_endpoints: Vec<(Vec3, Vec3)> = state
                        .selected_edges
                        .iter()
                        .filter_map(|&ek| {
                            let e = base_mesh.edges.get(ek)?;
                            let h1 = &base_mesh.half_edges[e.half_edge];
                            let p0 = base_mesh.vertices.get(h1.origin)?.position;
                            let p1 = base_mesh.vertices.get(h1.target)?.position;
                            Some((
                                node.world_matrix.transform_point3(p0),
                                node.world_matrix.transform_point3(p1),
                            ))
                        })
                        .collect();

                    let sel_face_loops: Vec<Vec<Vec3>> = state
                        .selected_faces
                        .iter()
                        .map(|&fk| {
                            base_mesh
                                .face_vertices(fk)
                                .into_iter()
                                .filter_map(|vk| {
                                    base_mesh
                                        .vertices
                                        .get(vk)
                                        .map(|v| node.world_matrix.transform_point3(v.position))
                                })
                                .collect()
                        })
                        .collect();

                    let creased_edges_endpoints: Vec<(Vec3, Vec3)> = base_mesh
                        .edges
                        .values()
                        .filter(|e| e.crease_weight > 0.0)
                        .filter_map(|e| {
                            let h1 = &base_mesh.half_edges[e.half_edge];
                            let p0 = base_mesh.vertices.get(h1.origin)?.position;
                            let p1 = base_mesh.vertices.get(h1.target)?.position;
                            Some((
                                node.world_matrix.transform_point3(p0),
                                node.world_matrix.transform_point3(p1),
                            ))
                        })
                        .collect();

                    let soft_selection = if state.proportional_editing {
                        gizmo_pos.map(|pos| (pos, state.proportional_radius))
                    } else {
                        None
                    };

                    state.renderer.render_sub_elements(
                        &view,
                        &state.depth_view,
                        &all_verts,
                        &sel_v_positions,
                        &sel_edges_endpoints,
                        &sel_face_loops,
                        &creased_edges_endpoints,
                        soft_selection,
                    );
                }
            }
        }
    } else if state.interaction_mode == InteractionMode::Object {
        if let Some(sel_id) = state.selected_node {
            if let Some(node) = state.scene.nodes.get(sel_id) {
                if let NodeData::Mesh { base_mesh, .. } = &node.data {
                    let mesh_edges: Vec<(Vec3, Vec3)> = base_mesh
                        .edges
                        .values()
                        .filter_map(|e| {
                            let h1 = &base_mesh.half_edges[e.half_edge];
                            let p0 = base_mesh.vertices.get(h1.origin)?.position;
                            let p1 = base_mesh.vertices.get(h1.target)?.position;
                            Some((p0, p1))
                        })
                        .collect();
                    let (b_min, b_max) = base_mesh.bounding_box();
                    state.renderer.render_object_selection_highlight(
                        &view,
                        &state.depth_view,
                        &mesh_edges,
                        node.world_matrix,
                        b_min,
                        b_max,
                        true,
                    );
                }
            }
        }
    }

    // 4. 3D Transform Gizmo Pass (Rendered at object or sub-element origin)
    if state.interaction_mode != InteractionMode::Sculpt {
        if let Some(pos) = gizmo_pos {
            let gizmo_orient = state.get_gizmo_orientation();
            let active_axis = state.active_gizmo_axis.or(state.hovered_gizmo_axis);
            state.renderer.render_gizmo(
                &view,
                &state.depth_view,
                &state.camera,
                pos,
                gizmo_orient,
                state.gizmo.mode,
                active_axis,
            );
        }
    }

    // 5. Egui GUI Overlay Pass
    let raw_input = state.egui_state.take_egui_input(&state.window);
    state.egui_ctx.begin_pass(raw_input);

    crate::ui::draw_ui(state);

    let full_output = state.egui_ctx.end_pass();
    state
        .egui_state
        .handle_platform_output(&state.window, full_output.platform_output);

    let tris = state
        .egui_ctx
        .tessellate(full_output.shapes, state.egui_ctx.pixels_per_point());

    let screen_descriptor = ScreenDescriptor {
        size_in_pixels: [state.surface_config.width, state.surface_config.height],
        pixels_per_point: state.egui_ctx.pixels_per_point(),
    };

    for (id, delta) in &full_output.textures_delta.set {
        state
            .egui_renderer
            .update_texture(&state.device, &state.queue, *id, delta);
    }

    let mut encoder = state.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Egui Render Encoder"),
    });

    state
        .egui_renderer
        .update_buffers(&state.device, &state.queue, &mut encoder, &tris, &screen_descriptor);

    {
        let mut pass = encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Egui Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            })
            .forget_lifetime();

        state
            .egui_renderer
            .render(&mut pass, &tris, &screen_descriptor);
    }

    for id in &full_output.textures_delta.free {
        state.egui_renderer.free_texture(id);
    }

    state.queue.submit(std::iter::once(encoder.finish()));
    output.present();

    if state.animation.is_playing {
        state.window.request_redraw();
    }
}

impl AppState {
    pub fn create_depth_texture(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
    ) -> (wgpu::Texture, wgpu::TextureView) {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth Texture"),
            size: wgpu::Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        (texture, view)
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.surface_config.width = new_size.width;
            self.surface_config.height = new_size.height;
            self.surface.configure(&self.device, &self.surface_config);

            let (d_tex, d_view) = Self::create_depth_texture(&self.device, &self.surface_config);
            self.depth_texture = d_tex;
            self.depth_view = d_view;

            self.camera.aspect = new_size.width as f32 / new_size.height as f32;
        }
    }

    pub fn sync_gpu_meshes(&mut self) {
        self.scene.update_transforms();
        self.scene.sync_skeletons();
        self.scene.sync_curves();
        self.scene.sync_booleans();
        self.scene.sync_shrinkwraps();
        let node_ids: Vec<NodeId> = self.scene.nodes.keys().collect();

        for node_id in node_ids {
            let needs_upload = {
                if let Some(node) = self.scene.nodes.get(node_id) {
                    match &node.data {
                        NodeData::Mesh { cached_render_mesh, .. } | NodeData::Curve { cached_render_mesh, .. } => {
                            cached_render_mesh.is_none() || !self.gpu_mesh_cache.contains_key(&node_id)
                        }
                        _ => false,
                    }
                } else {
                    false
                }
            };

            let weight_group = if self.interaction_mode == InteractionMode::WeightPaint && Some(node_id) == self.selected_node {
                self.active_vertex_group.as_deref()
            } else {
                None
            };

            if needs_upload {
                if let Some(render_mesh) = self.scene.evaluate_mesh_with_weights(node_id, weight_group) {
                    let gpu_mesh = self.renderer.upload_mesh(render_mesh);
                    self.gpu_mesh_cache.insert(node_id, gpu_mesh);
                } else {
                    self.gpu_mesh_cache.remove(&node_id);
                }
            }
        }
    }

    pub fn render(&mut self) {
        render_frame(self);
    }
}

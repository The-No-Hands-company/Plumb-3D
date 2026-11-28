use crate::renderer::{ModelUniform, ViewportRenderer};
use bytemuck;
use core_math::{Mat4, Vec3};
use geo_kernel::GpuVertex;
use wgpu::util::DeviceExt;

impl ViewportRenderer {
    /// Render sub-element selection overlays in Edit mode (vertex markers, edge highlights, face loop outlines, creased edges, soft selection influence)
    pub fn render_sub_elements(
        &self,
        target_view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        vertices: &[Vec3],
        selected_vertices: &[Vec3],
        selected_edges: &[(Vec3, Vec3)],
        selected_face_loops: &[Vec<Vec3>],
        creased_edges: &[(Vec3, Vec3)],
        soft_selection: Option<(Vec3, f32)>,
    ) {
        let mut overlay_vertices = Vec::new();

        // 1. Draw subtle cross markers for editable vertices
        let tick = 0.03_f32;
        let v_color = [0.4, 0.65, 0.95, 0.75];
        for &v in vertices {
            overlay_vertices.push(GpuVertex {
                position: (v - Vec3::X * tick).to_array(),
                normal: [0.0; 3],
                uv: [0.0; 2],
                color: v_color,
            });
            overlay_vertices.push(GpuVertex {
                position: (v + Vec3::X * tick).to_array(),
                normal: [0.0; 3],
                uv: [0.0; 2],
                color: v_color,
            });
            overlay_vertices.push(GpuVertex {
                position: (v - Vec3::Y * tick).to_array(),
                normal: [0.0; 3],
                uv: [0.0; 2],
                color: v_color,
            });
            overlay_vertices.push(GpuVertex {
                position: (v + Vec3::Y * tick).to_array(),
                normal: [0.0; 3],
                uv: [0.0; 2],
                color: v_color,
            });
            overlay_vertices.push(GpuVertex {
                position: (v - Vec3::Z * tick).to_array(),
                normal: [0.0; 3],
                uv: [0.0; 2],
                color: v_color,
            });
            overlay_vertices.push(GpuVertex {
                position: (v + Vec3::Z * tick).to_array(),
                normal: [0.0; 3],
                uv: [0.0; 2],
                color: v_color,
            });
        }

        // 2. Draw prominent diamond / star for all selected vertices
        let s_tick = 0.08_f32;
        let sel_color = [1.0, 0.9, 0.1, 1.0]; // Bright Gold
        for &sv in selected_vertices {
            overlay_vertices.push(GpuVertex {
                position: (sv - Vec3::X * s_tick).to_array(),
                normal: [0.0; 3],
                uv: [0.0; 2],
                color: sel_color,
            });
            overlay_vertices.push(GpuVertex {
                position: (sv + Vec3::X * s_tick).to_array(),
                normal: [0.0; 3],
                uv: [0.0; 2],
                color: sel_color,
            });
            overlay_vertices.push(GpuVertex {
                position: (sv - Vec3::Y * s_tick).to_array(),
                normal: [0.0; 3],
                uv: [0.0; 2],
                color: sel_color,
            });
            overlay_vertices.push(GpuVertex {
                position: (sv + Vec3::Y * s_tick).to_array(),
                normal: [0.0; 3],
                uv: [0.0; 2],
                color: sel_color,
            });
            overlay_vertices.push(GpuVertex {
                position: (sv - Vec3::Z * s_tick).to_array(),
                normal: [0.0; 3],
                uv: [0.0; 2],
                color: sel_color,
            });
            overlay_vertices.push(GpuVertex {
                position: (sv + Vec3::Z * s_tick).to_array(),
                normal: [0.0; 3],
                uv: [0.0; 2],
                color: sel_color,
            });

            let corners = [
                sv + Vec3::X * s_tick,
                sv + Vec3::Y * s_tick,
                sv - Vec3::X * s_tick,
                sv - Vec3::Y * s_tick,
            ];
            for i in 0..4 {
                overlay_vertices.push(GpuVertex {
                    position: corners[i].to_array(),
                    normal: [0.0; 3],
                    uv: [0.0; 2],
                    color: sel_color,
                });
                overlay_vertices.push(GpuVertex {
                    position: corners[(i + 1) % 4].to_array(),
                    normal: [0.0; 3],
                    uv: [0.0; 2],
                    color: sel_color,
                });
            }
        }

        // 3. Draw highlighted selected edges
        let edge_color = [0.05, 0.95, 1.0, 1.0]; // Bright Cyan
        for &(e0, e1) in selected_edges {
            overlay_vertices.push(GpuVertex {
                position: e0.to_array(),
                normal: [0.0; 3],
                uv: [0.0; 2],
                color: edge_color,
            });
            overlay_vertices.push(GpuVertex {
                position: e1.to_array(),
                normal: [0.0; 3],
                uv: [0.0; 2],
                color: edge_color,
            });
        }

        // 3b. Draw creased edges (Blender / Maya OpenSubdiv magenta / hot pink)
        let crease_color = [1.0, 0.22, 0.78, 1.0];
        for &(e0, e1) in creased_edges {
            overlay_vertices.push(GpuVertex {
                position: e0.to_array(),
                normal: [0.0; 3],
                uv: [0.0; 2],
                color: crease_color,
            });
            overlay_vertices.push(GpuVertex {
                position: e1.to_array(),
                normal: [0.0; 3],
                uv: [0.0; 2],
                color: crease_color,
            });
        }

        // 4. Draw highlighted selected face perimeter loops
        let face_color = [1.0, 0.65, 0.1, 1.0]; // Bright Amber / Orange
        for loop_pts in selected_face_loops {
            let n = loop_pts.len();
            if n >= 3 {
                for i in 0..n {
                    let p0 = loop_pts[i];
                    let p1 = loop_pts[(i + 1) % n];
                    overlay_vertices.push(GpuVertex {
                        position: p0.to_array(),
                        normal: [0.0; 3],
                        uv: [0.0; 2],
                        color: face_color,
                    });
                    overlay_vertices.push(GpuVertex {
                        position: p1.to_array(),
                        normal: [0.0; 3],
                        uv: [0.0; 2],
                        color: face_color,
                    });
                }
            }
        }

        // 5. Draw Proportional Editing influence circle (soft selection)
        if let Some((center, radius)) = soft_selection {
            let circle_color = [0.8, 0.5, 1.0, 0.85]; // Violet/Lavender
            let segs = 48;
            for i in 0..segs {
                let th0 = (i as f32 / segs as f32) * std::f32::consts::TAU;
                let th1 = ((i + 1) as f32 / segs as f32) * std::f32::consts::TAU;

                // XZ ground ring
                let p0_xz = center + Vec3::new(th0.cos() * radius, 0.0, th0.sin() * radius);
                let p1_xz = center + Vec3::new(th1.cos() * radius, 0.0, th1.sin() * radius);
                overlay_vertices.push(GpuVertex {
                    position: p0_xz.to_array(),
                    normal: [0.0; 3],
                    uv: [0.0; 2],
                    color: circle_color,
                });
                overlay_vertices.push(GpuVertex {
                    position: p1_xz.to_array(),
                    normal: [0.0; 3],
                    uv: [0.0; 2],
                    color: circle_color,
                });

                // XY ring
                let p0_xy = center + Vec3::new(th0.cos() * radius, th0.sin() * radius, 0.0);
                let p1_xy = center + Vec3::new(th1.cos() * radius, th1.sin() * radius, 0.0);
                overlay_vertices.push(GpuVertex {
                    position: p0_xy.to_array(),
                    normal: [0.0; 3],
                    uv: [0.0; 2],
                    color: circle_color,
                });
                overlay_vertices.push(GpuVertex {
                    position: p1_xy.to_array(),
                    normal: [0.0; 3],
                    uv: [0.0; 2],
                    color: circle_color,
                });
            }
        }

        if overlay_vertices.is_empty() {
            return;
        }

        let v_buf = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("SubElement Overlay Buffer"),
                contents: bytemuck::cast_slice(&overlay_vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let model_data = ModelUniform {
            model: Mat4::IDENTITY.to_cols_array(),
            color: [1.0, 1.0, 1.0, 1.0],
            material: [0.0, 1.0, 0.0, 0.0],
        };
        let m_buf = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("SubElement Model Buffer"),
                contents: bytemuck::bytes_of(&model_data),
                usage: wgpu::BufferUsages::UNIFORM,
            });
        let m_bind = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("SubElement Model Bind Group"),
            layout: &self.model_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: m_buf.as_entire_binding(),
            }],
        });

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("SubElement Overlay Encoder"),
            });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("SubElement Overlay Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            pass.set_pipeline(&self.gizmo_pipeline);
            pass.set_bind_group(0, &self.camera_bind_group, &[]);
            pass.set_bind_group(1, &m_bind, &[]);
            pass.set_vertex_buffer(0, v_buf.slice(..));
            pass.draw(0..overlay_vertices.len() as u32, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
    }

    /// Render object-level selection highlight and bounding box corner brackets in Object mode
    pub fn render_object_selection_highlight(
        &self,
        target_view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        mesh_edges: &[(Vec3, Vec3)],
        world_matrix: Mat4,
        bbox_min: Vec3,
        bbox_max: Vec3,
        show_bbox_brackets: bool,
    ) {
        let mut overlay_vertices = Vec::new();
        let sel_orange = [1.0_f32, 0.62, 0.12, 0.95]; // Standard DCC Active Object Orange

        // 1. Mesh edges in active selection color
        for &(p0, p1) in mesh_edges {
            let wp0 = world_matrix.transform_point3(p0);
            let wp1 = world_matrix.transform_point3(p1);
            overlay_vertices.push(GpuVertex {
                position: wp0.to_array(),
                normal: [0.0; 3],
                uv: [0.0; 2],
                color: sel_orange,
            });
            overlay_vertices.push(GpuVertex {
                position: wp1.to_array(),
                normal: [0.0; 3],
                uv: [0.0; 2],
                color: sel_orange,
            });
        }

        // 2. Corner brackets for 3D oriented bounding box
        if show_bbox_brackets && bbox_min.distance_squared(bbox_max) > 1e-4 {
            let bracket_col = [1.0_f32, 0.88, 0.25, 1.0]; // Golden amber
            let span = bbox_max - bbox_min;
            let arm = (span.length() * 0.12).clamp(0.04, 0.5);

            let xs = [bbox_min.x, bbox_max.x];
            let ys = [bbox_min.y, bbox_max.y];
            let zs = [bbox_min.z, bbox_max.z];

            for &x in &xs {
                for &y in &ys {
                    for &z in &zs {
                        let c = Vec3::new(x, y, z);
                        let dx = if x == bbox_min.x {
                            arm.min(span.x)
                        } else {
                            -arm.min(span.x)
                        };
                        let dy = if y == bbox_min.y {
                            arm.min(span.y)
                        } else {
                            -arm.min(span.y)
                        };
                        let dz = if z == bbox_min.z {
                            arm.min(span.z)
                        } else {
                            -arm.min(span.z)
                        };

                        let wc = world_matrix.transform_point3(c);
                        let w_arm_x = world_matrix.transform_point3(c + Vec3::new(dx, 0.0, 0.0));
                        let w_arm_y = world_matrix.transform_point3(c + Vec3::new(0.0, dy, 0.0));
                        let w_arm_z = world_matrix.transform_point3(c + Vec3::new(0.0, 0.0, dz));

                        overlay_vertices.push(GpuVertex {
                            position: wc.to_array(),
                            normal: [0.0; 3],
                            uv: [0.0; 2],
                            color: bracket_col,
                        });
                        overlay_vertices.push(GpuVertex {
                            position: w_arm_x.to_array(),
                            normal: [0.0; 3],
                            uv: [0.0; 2],
                            color: bracket_col,
                        });

                        overlay_vertices.push(GpuVertex {
                            position: wc.to_array(),
                            normal: [0.0; 3],
                            uv: [0.0; 2],
                            color: bracket_col,
                        });
                        overlay_vertices.push(GpuVertex {
                            position: w_arm_y.to_array(),
                            normal: [0.0; 3],
                            uv: [0.0; 2],
                            color: bracket_col,
                        });

                        overlay_vertices.push(GpuVertex {
                            position: wc.to_array(),
                            normal: [0.0; 3],
                            uv: [0.0; 2],
                            color: bracket_col,
                        });
                        overlay_vertices.push(GpuVertex {
                            position: w_arm_z.to_array(),
                            normal: [0.0; 3],
                            uv: [0.0; 2],
                            color: bracket_col,
                        });
                    }
                }
            }
        }

        if overlay_vertices.is_empty() {
            return;
        }

        let v_buf = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Object Selection Overlay Buffer"),
                contents: bytemuck::cast_slice(&overlay_vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let model_data = ModelUniform {
            model: Mat4::IDENTITY.to_cols_array(),
            color: [1.0, 1.0, 1.0, 1.0],
            material: [0.0, 1.0, 0.0, 0.0],
        };
        let m_buf = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Object Selection Model Buffer"),
                contents: bytemuck::bytes_of(&model_data),
                usage: wgpu::BufferUsages::UNIFORM,
            });
        let m_bind = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Object Selection Model Bind Group"),
            layout: &self.model_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: m_buf.as_entire_binding(),
            }],
        });

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Object Selection Overlay Encoder"),
            });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Object Selection Overlay Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            pass.set_pipeline(&self.gizmo_pipeline);
            pass.set_bind_group(0, &self.camera_bind_group, &[]);
            pass.set_bind_group(1, &m_bind, &[]);
            pass.set_vertex_buffer(0, v_buf.slice(..));
            pass.draw(0..overlay_vertices.len() as u32, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
    }
}

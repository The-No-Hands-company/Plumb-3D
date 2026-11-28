use crate::renderer::{ModelUniform, ViewportRenderer};
use bytemuck;
use core_math::{Mat4, Vec3};
use geo_kernel::GpuVertex;
use wgpu::util::DeviceExt;

impl ViewportRenderer {
    /// Render 3D wireframe actor glyphs for scene lights and cameras (Blender / Maya style)
    pub fn render_actor_glyphs(
        &self,
        target_view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        lights: &[(Vec3, Vec3, [f32; 4], u32, bool)], // (pos, dir, color, light_type: 0 dir, 1 pt, 2 spot, is_selected)
        cameras: &[(Mat4, f32, f32, bool)],           // (world_matrix, fov_y, aspect, is_selected)
        joints: &[(Mat4, f32, f32, bool)],            // (world_matrix, length, radius, is_selected)
        curves: &[(Mat4, Vec<Vec3>, Vec<(Vec3, Vec3, Vec3)>, bool, bool)], // (world_matrix, sampled_path, knots, is_closed, is_selected)
    ) {
        let mut glyph_vertices = Vec::new();
        let sel_gold = [1.0, 0.85, 0.1, 1.0];

        // 1. Scene Lights
        for &(pos, dir, col, light_type, is_selected) in lights {
            let color = if is_selected { sel_gold } else { col };

            match light_type {
                // Directional Light
                0 => {
                    let r = 0.35_f32;
                    let segs = 16;
                    let fwd = if dir.length_squared() > 1e-4 {
                        dir.normalize()
                    } else {
                        Vec3::NEG_Y
                    };
                    let up_hint = if fwd.y.abs() > 0.9 { Vec3::Z } else { Vec3::Y };
                    let right = fwd.cross(up_hint).normalize();
                    let up = right.cross(fwd).normalize();

                    for i in 0..segs {
                        let th0 = (i as f32 / segs as f32) * std::f32::consts::TAU;
                        let th1 = ((i + 1) as f32 / segs as f32) * std::f32::consts::TAU;
                        let p0 = pos + (right * th0.cos() + up * th0.sin()) * r;
                        let p1 = pos + (right * th1.cos() + up * th1.sin()) * r;
                        glyph_vertices.push(GpuVertex {
                            position: p0.to_array(),
                            normal: [0.0; 3],
                            uv: [0.0; 2],
                            color,
                        });
                        glyph_vertices.push(GpuVertex {
                            position: p1.to_array(),
                            normal: [0.0; 3],
                            uv: [0.0; 2],
                            color,
                        });
                    }

                    for k in 0..4 {
                        let angle = (k as f32 / 4.0) * std::f32::consts::TAU;
                        let ray_start = pos + (right * angle.cos() + up * angle.sin()) * r;
                        let ray_end = ray_start + fwd * 0.7;
                        glyph_vertices.push(GpuVertex {
                            position: ray_start.to_array(),
                            normal: [0.0; 3],
                            uv: [0.0; 2],
                            color,
                        });
                        glyph_vertices.push(GpuVertex {
                            position: ray_end.to_array(),
                            normal: [0.0; 3],
                            uv: [0.0; 2],
                            color,
                        });

                        let barb =
                            ray_end - fwd * 0.15 + (right * angle.cos() + up * angle.sin()) * 0.08;
                        glyph_vertices.push(GpuVertex {
                            position: ray_end.to_array(),
                            normal: [0.0; 3],
                            uv: [0.0; 2],
                            color,
                        });
                        glyph_vertices.push(GpuVertex {
                            position: barb.to_array(),
                            normal: [0.0; 3],
                            uv: [0.0; 2],
                            color,
                        });
                    }
                }
                // Point Light
                1 => {
                    let r = 0.22_f32;
                    let segs = 16;
                    for i in 0..segs {
                        let th0 = (i as f32 / segs as f32) * std::f32::consts::TAU;
                        let th1 = ((i + 1) as f32 / segs as f32) * std::f32::consts::TAU;
                        let p0 = pos + Vec3::new(th0.cos() * r, th0.sin() * r, 0.0);
                        let p1 = pos + Vec3::new(th1.cos() * r, th1.sin() * r, 0.0);
                        glyph_vertices.push(GpuVertex {
                            position: p0.to_array(),
                            normal: [0.0; 3],
                            uv: [0.0; 2],
                            color,
                        });
                        glyph_vertices.push(GpuVertex {
                            position: p1.to_array(),
                            normal: [0.0; 3],
                            uv: [0.0; 2],
                            color,
                        });
                    }
                    for i in 0..segs {
                        let th0 = (i as f32 / segs as f32) * std::f32::consts::TAU;
                        let th1 = ((i + 1) as f32 / segs as f32) * std::f32::consts::TAU;
                        let p0 = pos + Vec3::new(th0.cos() * r, 0.0, th0.sin() * r);
                        let p1 = pos + Vec3::new(th1.cos() * r, 0.0, th1.sin() * r);
                        glyph_vertices.push(GpuVertex {
                            position: p0.to_array(),
                            normal: [0.0; 3],
                            uv: [0.0; 2],
                            color,
                        });
                        glyph_vertices.push(GpuVertex {
                            position: p1.to_array(),
                            normal: [0.0; 3],
                            uv: [0.0; 2],
                            color,
                        });
                    }
                    for i in 0..segs {
                        let th0 = (i as f32 / segs as f32) * std::f32::consts::TAU;
                        let th1 = ((i + 1) as f32 / segs as f32) * std::f32::consts::TAU;
                        let p0 = pos + Vec3::new(0.0, th0.cos() * r, th0.sin() * r);
                        let p1 = pos + Vec3::new(0.0, th1.cos() * r, th1.sin() * r);
                        glyph_vertices.push(GpuVertex {
                            position: p0.to_array(),
                            normal: [0.0; 3],
                            uv: [0.0; 2],
                            color,
                        });
                        glyph_vertices.push(GpuVertex {
                            position: p1.to_array(),
                            normal: [0.0; 3],
                            uv: [0.0; 2],
                            color,
                        });
                    }
                    let cross_r = r * 1.5;
                    for axis in [Vec3::X, Vec3::Y, Vec3::Z] {
                        glyph_vertices.push(GpuVertex {
                            position: (pos - axis * cross_r).to_array(),
                            normal: [0.0; 3],
                            uv: [0.0; 2],
                            color,
                        });
                        glyph_vertices.push(GpuVertex {
                            position: (pos + axis * cross_r).to_array(),
                            normal: [0.0; 3],
                            uv: [0.0; 2],
                            color,
                        });
                    }
                }
                // Spot Light
                2 => {
                    let fwd = if dir.length_squared() > 1e-4 {
                        dir.normalize()
                    } else {
                        Vec3::NEG_Y
                    };
                    let up_hint = if fwd.y.abs() > 0.9 { Vec3::Z } else { Vec3::Y };
                    let right = fwd.cross(up_hint).normalize();
                    let up = right.cross(fwd).normalize();

                    let dist = 1.2_f32;
                    let radius = 0.6_f32;
                    let base_center = pos + fwd * dist;

                    let segs = 16;
                    for i in 0..segs {
                        let th0 = (i as f32 / segs as f32) * std::f32::consts::TAU;
                        let th1 = ((i + 1) as f32 / segs as f32) * std::f32::consts::TAU;
                        let p0 = base_center + (right * th0.cos() + up * th0.sin()) * radius;
                        let p1 = base_center + (right * th1.cos() + up * th1.sin()) * radius;
                        glyph_vertices.push(GpuVertex {
                            position: p0.to_array(),
                            normal: [0.0; 3],
                            uv: [0.0; 2],
                            color,
                        });
                        glyph_vertices.push(GpuVertex {
                            position: p1.to_array(),
                            normal: [0.0; 3],
                            uv: [0.0; 2],
                            color,
                        });
                    }

                    for k in 0..4 {
                        let angle = (k as f32 / 4.0) * std::f32::consts::TAU;
                        let rim_p = base_center + (right * angle.cos() + up * angle.sin()) * radius;
                        glyph_vertices.push(GpuVertex {
                            position: pos.to_array(),
                            normal: [0.0; 3],
                            uv: [0.0; 2],
                            color,
                        });
                        glyph_vertices.push(GpuVertex {
                            position: rim_p.to_array(),
                            normal: [0.0; 3],
                            uv: [0.0; 2],
                            color,
                        });
                    }
                    glyph_vertices.push(GpuVertex {
                        position: pos.to_array(),
                        normal: [0.0; 3],
                        uv: [0.0; 2],
                        color,
                    });
                    glyph_vertices.push(GpuVertex {
                        position: base_center.to_array(),
                        normal: [0.0; 3],
                        uv: [0.0; 2],
                        color,
                    });
                }
                _ => {}
            }
        }

        // 2. Scene Cameras
        for &(world_matrix, fov_y, aspect, is_selected) in cameras {
            let color = if is_selected {
                sel_gold
            } else {
                [0.35, 0.75, 1.0, 0.9]
            };
            let pos = world_matrix.transform_point3(Vec3::ZERO);

            let dist = 0.8_f32;
            let half_h = (fov_y * 0.5).tan() * dist;
            let half_w = half_h * aspect;

            let corners_local = [
                Vec3::new(-half_w, -half_h, -dist),
                Vec3::new(half_w, -half_h, -dist),
                Vec3::new(half_w, half_h, -dist),
                Vec3::new(-half_w, half_h, -dist),
            ];

            let corners_world: Vec<Vec3> = corners_local
                .iter()
                .map(|&c| world_matrix.transform_point3(c))
                .collect();

            for &cw in &corners_world {
                glyph_vertices.push(GpuVertex {
                    position: pos.to_array(),
                    normal: [0.0; 3],
                    uv: [0.0; 2],
                    color,
                });
                glyph_vertices.push(GpuVertex {
                    position: cw.to_array(),
                    normal: [0.0; 3],
                    uv: [0.0; 2],
                    color,
                });
            }

            for i in 0..4 {
                let p0 = corners_world[i];
                let p1 = corners_world[(i + 1) % 4];
                glyph_vertices.push(GpuVertex {
                    position: p0.to_array(),
                    normal: [0.0; 3],
                    uv: [0.0; 2],
                    color,
                });
                glyph_vertices.push(GpuVertex {
                    position: p1.to_array(),
                    normal: [0.0; 3],
                    uv: [0.0; 2],
                    color,
                });
            }

            let top_mid = (corners_world[2] + corners_world[3]) * 0.5;
            let top_up = world_matrix.transform_vector3(Vec3::Y * (half_h * 0.4));
            let top_peak = top_mid + top_up;
            let top_left = (corners_world[3] * 0.7) + (corners_world[2] * 0.3);
            let top_right = (corners_world[3] * 0.3) + (corners_world[2] * 0.7);

            glyph_vertices.push(GpuVertex {
                position: top_left.to_array(),
                normal: [0.0; 3],
                uv: [0.0; 2],
                color,
            });
            glyph_vertices.push(GpuVertex {
                position: top_peak.to_array(),
                normal: [0.0; 3],
                uv: [0.0; 2],
                color,
            });
            glyph_vertices.push(GpuVertex {
                position: top_peak.to_array(),
                normal: [0.0; 3],
                uv: [0.0; 2],
                color,
            });
            glyph_vertices.push(GpuVertex {
                position: top_right.to_array(),
                normal: [0.0; 3],
                uv: [0.0; 2],
                color,
            });

            let b_w = 0.15_f32;
            let b_h = 0.12_f32;
            let b_d = 0.25_f32;
            let body_corners = [
                world_matrix.transform_point3(Vec3::new(-b_w, -b_h, 0.0)),
                world_matrix.transform_point3(Vec3::new(b_w, -b_h, 0.0)),
                world_matrix.transform_point3(Vec3::new(b_w, b_h, 0.0)),
                world_matrix.transform_point3(Vec3::new(-b_w, b_h, 0.0)),
                world_matrix.transform_point3(Vec3::new(-b_w, -b_h, b_d)),
                world_matrix.transform_point3(Vec3::new(b_w, -b_h, b_d)),
                world_matrix.transform_point3(Vec3::new(b_w, b_h, b_d)),
                world_matrix.transform_point3(Vec3::new(-b_w, b_h, b_d)),
            ];

            let box_edges = [
                (0, 1),
                (1, 2),
                (2, 3),
                (3, 0),
                (4, 5),
                (5, 6),
                (6, 7),
                (7, 4),
                (0, 4),
                (1, 5),
                (2, 6),
                (3, 7),
            ];

            for (i0, i1) in box_edges {
                glyph_vertices.push(GpuVertex {
                    position: body_corners[i0].to_array(),
                    normal: [0.0; 3],
                    uv: [0.0; 2],
                    color,
                });
                glyph_vertices.push(GpuVertex {
                    position: body_corners[i1].to_array(),
                    normal: [0.0; 3],
                    uv: [0.0; 2],
                    color,
                });
            }
        }

        // 3. Scene Skeleton Joints (Octahedral Bones)
        for &(world_matrix, length, radius, is_selected) in joints {
            let color = if is_selected {
                sel_gold
            } else {
                [0.25, 0.85, 0.75, 1.0]
            };
            let p_base = world_matrix.transform_point3(Vec3::ZERO);
            let p_tip = world_matrix.transform_point3(Vec3::new(0.0, length, 0.0));

            let waist_y = length * 0.22;
            let r = radius.max(0.05);

            let d0 = world_matrix.transform_point3(Vec3::new(r, waist_y, 0.0));
            let d1 = world_matrix.transform_point3(Vec3::new(-r, waist_y, 0.0));
            let d2 = world_matrix.transform_point3(Vec3::new(0.0, waist_y, r));
            let d3 = world_matrix.transform_point3(Vec3::new(0.0, waist_y, -r));

            // Base to diamond
            for &d in &[d0, d1, d2, d3] {
                glyph_vertices.push(GpuVertex {
                    position: p_base.to_array(),
                    normal: [0.0; 3],
                    uv: [0.0; 2],
                    color,
                });
                glyph_vertices.push(GpuVertex {
                    position: d.to_array(),
                    normal: [0.0; 3],
                    uv: [0.0; 2],
                    color,
                });
            }

            // Diamond to tip
            for &d in &[d0, d1, d2, d3] {
                glyph_vertices.push(GpuVertex {
                    position: d.to_array(),
                    normal: [0.0; 3],
                    uv: [0.0; 2],
                    color,
                });
                glyph_vertices.push(GpuVertex {
                    position: p_tip.to_array(),
                    normal: [0.0; 3],
                    uv: [0.0; 2],
                    color,
                });
            }

            // Waist ring
            let ring_edges = [(d0, d2), (d2, d1), (d1, d3), (d3, d0)];
            for (ra, rb) in ring_edges {
                glyph_vertices.push(GpuVertex {
                    position: ra.to_array(),
                    normal: [0.0; 3],
                    uv: [0.0; 2],
                    color,
                });
                glyph_vertices.push(GpuVertex {
                    position: rb.to_array(),
                    normal: [0.0; 3],
                    uv: [0.0; 2],
                    color,
                });
            }

            // Joint base pivot socket ring (in local XZ plane)
            let segs = 12;
            let ball_r = r * 0.6;
            for i in 0..segs {
                let th0 = (i as f32 / segs as f32) * std::f32::consts::TAU;
                let th1 = ((i + 1) as f32 / segs as f32) * std::f32::consts::TAU;
                let b0 = world_matrix.transform_point3(Vec3::new(
                    th0.cos() * ball_r,
                    0.0,
                    th0.sin() * ball_r,
                ));
                let b1 = world_matrix.transform_point3(Vec3::new(
                    th1.cos() * ball_r,
                    0.0,
                    th1.sin() * ball_r,
                ));
                glyph_vertices.push(GpuVertex {
                    position: b0.to_array(),
                    normal: [0.0; 3],
                    uv: [0.0; 2],
                    color,
                });
                glyph_vertices.push(GpuVertex {
                    position: b1.to_array(),
                    normal: [0.0; 3],
                    uv: [0.0; 2],
                    color,
                });
            }
        }

        // 4. Scene Spline Curves
        for (world_matrix, sampled_path, knots, is_closed, is_selected) in curves {
            let curve_color = if *is_selected {
                sel_gold
            } else {
                [0.28, 0.72, 1.0, 1.0]
            };
            let handle_color = if *is_selected {
                [1.0, 0.9, 0.4, 0.8]
            } else {
                [0.95, 0.45, 0.45, 0.8]
            };

            // Render path polyline
            if sampled_path.len() >= 2 {
                for i in 0..(sampled_path.len() - 1) {
                    let p0 = world_matrix.transform_point3(sampled_path[i]);
                    let p1 = world_matrix.transform_point3(sampled_path[i + 1]);
                    glyph_vertices.push(GpuVertex {
                        position: p0.to_array(),
                        normal: [0.0; 3],
                        uv: [0.0; 2],
                        color: curve_color,
                    });
                    glyph_vertices.push(GpuVertex {
                        position: p1.to_array(),
                        normal: [0.0; 3],
                        uv: [0.0; 2],
                        color: curve_color,
                    });
                }
                if *is_closed && sampled_path.len() > 2 {
                    let p0 = world_matrix.transform_point3(*sampled_path.last().unwrap());
                    let p1 = world_matrix.transform_point3(sampled_path[0]);
                    glyph_vertices.push(GpuVertex {
                        position: p0.to_array(),
                        normal: [0.0; 3],
                        uv: [0.0; 2],
                        color: curve_color,
                    });
                    glyph_vertices.push(GpuVertex {
                        position: p1.to_array(),
                        normal: [0.0; 3],
                        uv: [0.0; 2],
                        color: curve_color,
                    });
                }
            }

            // Render knot anchors and handles
            for &(knot_pos, handle_in, handle_out) in knots {
                let p_knot = world_matrix.transform_point3(knot_pos);
                // Knot point star
                let ks = 0.035_f32;
                for axis in [Vec3::X, Vec3::Y, Vec3::Z] {
                    let k0 = p_knot - axis * ks;
                    let k1 = p_knot + axis * ks;
                    glyph_vertices.push(GpuVertex {
                        position: k0.to_array(),
                        normal: [0.0; 3],
                        uv: [0.0; 2],
                        color: curve_color,
                    });
                    glyph_vertices.push(GpuVertex {
                        position: k1.to_array(),
                        normal: [0.0; 3],
                        uv: [0.0; 2],
                        color: curve_color,
                    });
                }

                // Handle in
                if handle_in.length_squared() > 1e-6 {
                    let p_hin = world_matrix.transform_point3(knot_pos + handle_in);
                    glyph_vertices.push(GpuVertex {
                        position: p_knot.to_array(),
                        normal: [0.0; 3],
                        uv: [0.0; 2],
                        color: handle_color,
                    });
                    glyph_vertices.push(GpuVertex {
                        position: p_hin.to_array(),
                        normal: [0.0; 3],
                        uv: [0.0; 2],
                        color: handle_color,
                    });

                    let t0 = p_hin - Vec3::Y * (ks * 0.6);
                    let t1 = p_hin + Vec3::Y * (ks * 0.6);
                    glyph_vertices.push(GpuVertex {
                        position: t0.to_array(),
                        normal: [0.0; 3],
                        uv: [0.0; 2],
                        color: handle_color,
                    });
                    glyph_vertices.push(GpuVertex {
                        position: t1.to_array(),
                        normal: [0.0; 3],
                        uv: [0.0; 2],
                        color: handle_color,
                    });
                }

                // Handle out
                if handle_out.length_squared() > 1e-6 {
                    let p_hout = world_matrix.transform_point3(knot_pos + handle_out);
                    glyph_vertices.push(GpuVertex {
                        position: p_knot.to_array(),
                        normal: [0.0; 3],
                        uv: [0.0; 2],
                        color: handle_color,
                    });
                    glyph_vertices.push(GpuVertex {
                        position: p_hout.to_array(),
                        normal: [0.0; 3],
                        uv: [0.0; 2],
                        color: handle_color,
                    });

                    let t0 = p_hout - Vec3::Y * (ks * 0.6);
                    let t1 = p_hout + Vec3::Y * (ks * 0.6);
                    glyph_vertices.push(GpuVertex {
                        position: t0.to_array(),
                        normal: [0.0; 3],
                        uv: [0.0; 2],
                        color: handle_color,
                    });
                    glyph_vertices.push(GpuVertex {
                        position: t1.to_array(),
                        normal: [0.0; 3],
                        uv: [0.0; 2],
                        color: handle_color,
                    });
                }
            }
        }

        if glyph_vertices.is_empty() {
            return;
        }

        let v_buf = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Actor Glyphs Buffer"),
                contents: bytemuck::cast_slice(&glyph_vertices),
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
                label: Some("Actor Glyphs Model Buffer"),
                contents: bytemuck::bytes_of(&model_data),
                usage: wgpu::BufferUsages::UNIFORM,
            });
        let m_bind = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Actor Glyphs Model Bind Group"),
            layout: &self.model_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: m_buf.as_entire_binding(),
            }],
        });

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Actor Glyphs Encoder"),
            });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Actor Glyphs Pass"),
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
            pass.draw(0..glyph_vertices.len() as u32, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
    }
}

use crate::camera::ViewportCamera;
use crate::gizmo::{GizmoAxis, GizmoMode};
use crate::renderer::{ModelUniform, ViewportRenderer};
use bytemuck;
use core_math::{Mat4, Vec3};
use geo_kernel::GpuVertex;
use wgpu::util::DeviceExt;

impl ViewportRenderer {
    /// Render 3D Coordinate Axis Transform Gizmo with adaptive screen-space sizing, 3D arrow cones, planar square handles, rotation toruses, and scale boxes
    pub fn render_gizmo(
        &self,
        target_view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        camera: &ViewportCamera,
        origin: Vec3,
        orientation: core_math::Quat,
        mode: GizmoMode,
        active_axis: Option<GizmoAxis>,
    ) {
        // Compute adaptive screen-space scale based on camera distance
        let dist = (camera.eye_position() - origin).length().max(0.1);
        let scale = (dist * 0.16).clamp(0.4, 30.0);
        let len = scale;

        // DCC Professional High-Contrast Palette
        let c_active = [1.0_f32, 0.92, 0.23, 1.0]; // Glowing Electric Gold
        let c_x = if active_axis == Some(GizmoAxis::X) {
            c_active
        } else {
            [0.96, 0.26, 0.21, 1.0]
        }; // Crimson Red
        let c_y = if active_axis == Some(GizmoAxis::Y) {
            c_active
        } else {
            [0.24, 0.84, 0.38, 1.0]
        }; // Emerald Green
        let c_z = if active_axis == Some(GizmoAxis::Z) {
            c_active
        } else {
            [0.18, 0.52, 0.98, 1.0]
        }; // Royal Blue
        let c_xy = if active_axis == Some(GizmoAxis::XY) {
            c_active
        } else {
            [0.18, 0.52, 0.98, 0.75]
        };
        let c_xz = if active_axis == Some(GizmoAxis::XZ) {
            c_active
        } else {
            [0.24, 0.84, 0.38, 0.75]
        };
        let c_yz = if active_axis == Some(GizmoAxis::YZ) {
            c_active
        } else {
            [0.96, 0.26, 0.21, 0.75]
        };
        let c_screen = if active_axis == Some(GizmoAxis::Screen) {
            c_active
        } else {
            [0.9, 0.92, 0.95, 0.85]
        };

        let mut gizmo_vertices = Vec::new();

        // 1. Center Screen-Space Drag Handle
        let center_radius = 0.10 * scale;
        let c_segs = 16;
        for i in 0..c_segs {
            let a0 = (i as f32 / c_segs as f32) * std::f32::consts::TAU;
            let a1 = ((i + 1) as f32 / c_segs as f32) * std::f32::consts::TAU;
            let p0 = origin
                + camera.right() * (a0.cos() * center_radius)
                + camera.up() * (a0.sin() * center_radius);
            let p1 = origin
                + camera.right() * (a1.cos() * center_radius)
                + camera.up() * (a1.sin() * center_radius);
            gizmo_vertices.push(GpuVertex {
                position: p0.to_array(),
                normal: [0.0; 3],
                uv: [0.0; 2],
                color: c_screen,
            });
            gizmo_vertices.push(GpuVertex {
                position: p1.to_array(),
                normal: [0.0; 3],
                uv: [0.0; 2],
                color: c_screen,
            });
        }

        match mode {
            GizmoMode::Translate => {
                let dir_x = orientation * Vec3::new(len, 0.0, 0.0);
                let dir_y = orientation * Vec3::new(0.0, len, 0.0);
                let dir_z = orientation * Vec3::new(0.0, 0.0, len);

                // Axis Shafts
                gizmo_vertices.push(GpuVertex {
                    position: origin.to_array(),
                    normal: [0.0; 3],
                    uv: [0.0; 2],
                    color: c_x,
                });
                gizmo_vertices.push(GpuVertex {
                    position: (origin + dir_x * 0.78).to_array(),
                    normal: [0.0; 3],
                    uv: [0.0; 2],
                    color: c_x,
                });

                gizmo_vertices.push(GpuVertex {
                    position: origin.to_array(),
                    normal: [0.0; 3],
                    uv: [0.0; 2],
                    color: c_y,
                });
                gizmo_vertices.push(GpuVertex {
                    position: (origin + dir_y * 0.78).to_array(),
                    normal: [0.0; 3],
                    uv: [0.0; 2],
                    color: c_y,
                });

                gizmo_vertices.push(GpuVertex {
                    position: origin.to_array(),
                    normal: [0.0; 3],
                    uv: [0.0; 2],
                    color: c_z,
                });
                gizmo_vertices.push(GpuVertex {
                    position: (origin + dir_z * 0.78).to_array(),
                    normal: [0.0; 3],
                    uv: [0.0; 2],
                    color: c_z,
                });

                // 3D Arrowhead Cones for X, Y, Z
                append_3d_cone(
                    &mut gizmo_vertices,
                    origin + dir_x * 0.78,
                    origin + dir_x * 1.05,
                    orientation * Vec3::Y,
                    orientation * Vec3::Z,
                    0.07 * scale,
                    c_x,
                );
                append_3d_cone(
                    &mut gizmo_vertices,
                    origin + dir_y * 0.78,
                    origin + dir_y * 1.05,
                    orientation * Vec3::X,
                    orientation * Vec3::Z,
                    0.07 * scale,
                    c_y,
                );
                append_3d_cone(
                    &mut gizmo_vertices,
                    origin + dir_z * 0.78,
                    origin + dir_z * 1.05,
                    orientation * Vec3::X,
                    orientation * Vec3::Y,
                    0.07 * scale,
                    c_z,
                );

                // Planar translation handles (squares in XY, XZ, YZ)
                let p_off = 0.32 * len;
                let p_sz = 0.16 * len;

                // XY plane handle
                let xy0 = origin + orientation * Vec3::new(p_off, p_off, 0.0);
                let xy1 = origin + orientation * Vec3::new(p_off + p_sz, p_off, 0.0);
                let xy2 = origin + orientation * Vec3::new(p_off + p_sz, p_off + p_sz, 0.0);
                let xy3 = origin + orientation * Vec3::new(p_off, p_off + p_sz, 0.0);
                append_square(&mut gizmo_vertices, xy0, xy1, xy2, xy3, c_xy);

                // XZ plane handle
                let xz0 = origin + orientation * Vec3::new(p_off, 0.0, p_off);
                let xz1 = origin + orientation * Vec3::new(p_off + p_sz, 0.0, p_off);
                let xz2 = origin + orientation * Vec3::new(p_off + p_sz, 0.0, p_off + p_sz);
                let xz3 = origin + orientation * Vec3::new(p_off, 0.0, p_off + p_sz);
                append_square(&mut gizmo_vertices, xz0, xz1, xz2, xz3, c_xz);

                // YZ plane handle
                let yz0 = origin + orientation * Vec3::new(0.0, p_off, p_off);
                let yz1 = origin + orientation * Vec3::new(0.0, p_off + p_sz, p_off);
                let yz2 = origin + orientation * Vec3::new(0.0, p_off + p_sz, p_off + p_sz);
                let yz3 = origin + orientation * Vec3::new(0.0, p_off, p_off + p_sz);
                append_square(&mut gizmo_vertices, yz0, yz1, yz2, yz3, c_yz);
            }
            GizmoMode::Rotate => {
                let segs = 48;
                let r = len * 0.95;

                // X-axis ring (YZ plane)
                append_circle_ring(
                    &mut gizmo_vertices,
                    origin,
                    orientation * Vec3::Y,
                    orientation * Vec3::Z,
                    r,
                    segs,
                    c_x,
                );
                // Y-axis ring (XZ plane)
                append_circle_ring(
                    &mut gizmo_vertices,
                    origin,
                    orientation * Vec3::X,
                    orientation * Vec3::Z,
                    r,
                    segs,
                    c_y,
                );
                // Z-axis ring (XY plane)
                append_circle_ring(
                    &mut gizmo_vertices,
                    origin,
                    orientation * Vec3::X,
                    orientation * Vec3::Y,
                    r,
                    segs,
                    c_z,
                );

                // Outer camera-facing trackball ring
                append_circle_ring(
                    &mut gizmo_vertices,
                    origin,
                    camera.right(),
                    camera.up(),
                    r * 1.15,
                    segs,
                    [0.85, 0.88, 0.92, 0.6],
                );
            }
            GizmoMode::Scale => {
                let dir_x = orientation * Vec3::new(len, 0.0, 0.0);
                let dir_y = orientation * Vec3::new(0.0, len, 0.0);
                let dir_z = orientation * Vec3::new(0.0, 0.0, len);

                // Shafts
                gizmo_vertices.push(GpuVertex {
                    position: origin.to_array(),
                    normal: [0.0; 3],
                    uv: [0.0; 2],
                    color: c_x,
                });
                gizmo_vertices.push(GpuVertex {
                    position: (origin + dir_x * 0.85).to_array(),
                    normal: [0.0; 3],
                    uv: [0.0; 2],
                    color: c_x,
                });

                gizmo_vertices.push(GpuVertex {
                    position: origin.to_array(),
                    normal: [0.0; 3],
                    uv: [0.0; 2],
                    color: c_y,
                });
                gizmo_vertices.push(GpuVertex {
                    position: (origin + dir_y * 0.85).to_array(),
                    normal: [0.0; 3],
                    uv: [0.0; 2],
                    color: c_y,
                });

                gizmo_vertices.push(GpuVertex {
                    position: origin.to_array(),
                    normal: [0.0; 3],
                    uv: [0.0; 2],
                    color: c_z,
                });
                gizmo_vertices.push(GpuVertex {
                    position: (origin + dir_z * 0.85).to_array(),
                    normal: [0.0; 3],
                    uv: [0.0; 2],
                    color: c_z,
                });

                // Scale Box Cubes at the tips
                let b_sz = 0.06 * scale;
                append_cube(
                    &mut gizmo_vertices,
                    origin + dir_x * 0.92,
                    orientation,
                    b_sz,
                    c_x,
                );
                append_cube(
                    &mut gizmo_vertices,
                    origin + dir_y * 0.92,
                    orientation,
                    b_sz,
                    c_y,
                );
                append_cube(
                    &mut gizmo_vertices,
                    origin + dir_z * 0.92,
                    orientation,
                    b_sz,
                    c_z,
                );

                // Center uniform scale box
                append_cube(
                    &mut gizmo_vertices,
                    origin,
                    orientation,
                    b_sz * 0.9,
                    c_screen,
                );
            }
        }

        if gizmo_vertices.is_empty() {
            return;
        }

        let v_buf = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Gizmo Vertices"),
                contents: bytemuck::cast_slice(&gizmo_vertices),
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
                label: Some("Gizmo Model Buffer"),
                contents: bytemuck::bytes_of(&model_data),
                usage: wgpu::BufferUsages::UNIFORM,
            });
        let m_bind = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Gizmo Model Bind Group"),
            layout: &self.model_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: m_buf.as_entire_binding(),
            }],
        });

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Gizmo Encoder"),
            });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Gizmo Pass"),
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
            pass.draw(0..gizmo_vertices.len() as u32, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
    }
}

fn append_3d_cone(
    verts: &mut Vec<GpuVertex>,
    base_center: Vec3,
    tip: Vec3,
    radial_u: Vec3,
    radial_v: Vec3,
    radius: f32,
    color: [f32; 4],
) {
    let segs = 8;
    let mut base_pts = Vec::with_capacity(segs);
    for i in 0..segs {
        let th = (i as f32 / segs as f32) * std::f32::consts::TAU;
        let pt = base_center + radial_u * (th.cos() * radius) + radial_v * (th.sin() * radius);
        base_pts.push(pt);
    }

    for i in 0..segs {
        let p0 = base_pts[i];
        let p1 = base_pts[(i + 1) % segs];

        // Side cone lines
        verts.push(GpuVertex {
            position: p0.to_array(),
            normal: [0.0; 3],
            uv: [0.0; 2],
            color,
        });
        verts.push(GpuVertex {
            position: tip.to_array(),
            normal: [0.0; 3],
            uv: [0.0; 2],
            color,
        });

        // Base circle ring lines
        verts.push(GpuVertex {
            position: p0.to_array(),
            normal: [0.0; 3],
            uv: [0.0; 2],
            color,
        });
        verts.push(GpuVertex {
            position: p1.to_array(),
            normal: [0.0; 3],
            uv: [0.0; 2],
            color,
        });
    }
}

fn append_square(
    verts: &mut Vec<GpuVertex>,
    p0: Vec3,
    p1: Vec3,
    p2: Vec3,
    p3: Vec3,
    color: [f32; 4],
) {
    verts.push(GpuVertex {
        position: p0.to_array(),
        normal: [0.0; 3],
        uv: [0.0; 2],
        color,
    });
    verts.push(GpuVertex {
        position: p1.to_array(),
        normal: [0.0; 3],
        uv: [0.0; 2],
        color,
    });

    verts.push(GpuVertex {
        position: p1.to_array(),
        normal: [0.0; 3],
        uv: [0.0; 2],
        color,
    });
    verts.push(GpuVertex {
        position: p2.to_array(),
        normal: [0.0; 3],
        uv: [0.0; 2],
        color,
    });

    verts.push(GpuVertex {
        position: p2.to_array(),
        normal: [0.0; 3],
        uv: [0.0; 2],
        color,
    });
    verts.push(GpuVertex {
        position: p3.to_array(),
        normal: [0.0; 3],
        uv: [0.0; 2],
        color,
    });

    verts.push(GpuVertex {
        position: p3.to_array(),
        normal: [0.0; 3],
        uv: [0.0; 2],
        color,
    });
    verts.push(GpuVertex {
        position: p0.to_array(),
        normal: [0.0; 3],
        uv: [0.0; 2],
        color,
    });
}

fn append_circle_ring(
    verts: &mut Vec<GpuVertex>,
    center: Vec3,
    u_axis: Vec3,
    v_axis: Vec3,
    radius: f32,
    segments: usize,
    color: [f32; 4],
) {
    for i in 0..segments {
        let th0 = (i as f32 / segments as f32) * std::f32::consts::TAU;
        let th1 = ((i + 1) as f32 / segments as f32) * std::f32::consts::TAU;

        let p0 = center + u_axis * (th0.cos() * radius) + v_axis * (th0.sin() * radius);
        let p1 = center + u_axis * (th1.cos() * radius) + v_axis * (th1.sin() * radius);

        verts.push(GpuVertex {
            position: p0.to_array(),
            normal: [0.0; 3],
            uv: [0.0; 2],
            color,
        });
        verts.push(GpuVertex {
            position: p1.to_array(),
            normal: [0.0; 3],
            uv: [0.0; 2],
            color,
        });
    }
}

fn append_cube(
    verts: &mut Vec<GpuVertex>,
    center: Vec3,
    orientation: core_math::Quat,
    half_size: f32,
    color: [f32; 4],
) {
    let h = half_size;
    let corners = [
        center + orientation * Vec3::new(-h, -h, -h),
        center + orientation * Vec3::new(h, -h, -h),
        center + orientation * Vec3::new(h, h, -h),
        center + orientation * Vec3::new(-h, h, -h),
        center + orientation * Vec3::new(-h, -h, h),
        center + orientation * Vec3::new(h, -h, h),
        center + orientation * Vec3::new(h, h, h),
        center + orientation * Vec3::new(-h, h, h),
    ];

    let edges = [
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

    for &(i0, i1) in &edges {
        verts.push(GpuVertex {
            position: corners[i0].to_array(),
            normal: [0.0; 3],
            uv: [0.0; 2],
            color,
        });
        verts.push(GpuVertex {
            position: corners[i1].to_array(),
            normal: [0.0; 3],
            uv: [0.0; 2],
            color,
        });
    }
}

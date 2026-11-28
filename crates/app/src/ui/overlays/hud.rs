use core_math::Vec3;
use graph_engine::NodeData;
use render_viewport::ViewportCamera;

use crate::state::AppState;
use crate::types::*;

/// Interactive 6-Axis 3D Viewport Navigation Orbit Gizmo (Blender 4.x / Maya / Plasticity Style)
fn draw_viewport_orientation_compass(camera: &mut ViewportCamera, ui: &mut egui::Ui) -> bool {
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(76.0, 76.0), egui::Sense::click_and_drag());
    let center = rect.center();
    let painter = ui.painter();

    // 1. Sleek dark frosted glass disc background
    painter.circle_filled(center, 34.0, egui::Color32::from_black_alpha(175));
    painter.circle_stroke(
        center,
        34.0,
        egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(60, 65, 78)),
    );

    let view_mat = camera.view_matrix();
    let mut changed = false;

    // Handle direct click-and-drag orbiting on the navigation sphere
    if response.dragged() {
        let delta = response.drag_delta();
        camera.orbit(delta.x * 2.0, delta.y * 2.0);
        changed = true;
    }

    // 6 Directional Cardinal View Axes (+X, -X, +Y, -Y, +Z, -Z)
    let axes = [
        // +X: Right
        (
            Vec3::X,
            egui::Color32::from_rgb(250, 60, 60),
            "X",
            -std::f32::consts::FRAC_PI_2,
            0.0_f32,
            "Right View (+X)",
        ),
        // -X: Left
        (
            -Vec3::X,
            egui::Color32::from_rgb(140, 40, 40),
            "",
            std::f32::consts::FRAC_PI_2,
            0.0_f32,
            "Left View (-X)",
        ),
        // +Y: Top
        (
            Vec3::Y,
            egui::Color32::from_rgb(50, 225, 75),
            "Y",
            0.0_f32,
            std::f32::consts::FRAC_PI_2 - 0.001_f32,
            "Top View (+Y)",
        ),
        // -Y: Bottom
        (
            -Vec3::Y,
            egui::Color32::from_rgb(30, 120, 45),
            "",
            0.0_f32,
            -std::f32::consts::FRAC_PI_2 + 0.001_f32,
            "Bottom View (-Y)",
        ),
        // +Z: Front
        (
            Vec3::Z,
            egui::Color32::from_rgb(50, 135, 255),
            "Z",
            0.0_f32,
            0.0_f32,
            "Front View (+Z)",
        ),
        // -Z: Back
        (
            -Vec3::Z,
            egui::Color32::from_rgb(28, 70, 145),
            "",
            std::f32::consts::PI,
            0.0_f32,
            "Back View (-Z)",
        ),
    ];

    // Sort by Camera-Space Depth (Z) so front bubbles naturally overlap back bubbles
    let mut sorted_axes: Vec<_> = axes
        .iter()
        .map(|(axis_vec, color, label, target_yaw, target_pitch, tip)| {
            let cam_space = view_mat.transform_vector3(*axis_vec);
            (cam_space, *color, *label, *target_yaw, *target_pitch, *tip)
        })
        .collect();

    sorted_axes.sort_by(|a, b| {
        a.0.z
            .partial_cmp(&b.0.z)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    for (cam_space, color, label, target_yaw, target_pitch, tip) in sorted_axes {
        let len = 23.0_f32;
        let bubble_pos = egui::pos2(center.x + cam_space.x * len, center.y - cam_space.y * len);

        let is_front = cam_space.z >= -0.05;
        let bubble_r = if is_front { 6.5_f32 } else { 4.5_f32 };
        let hit_rect = egui::Rect::from_center_size(bubble_pos, egui::vec2(16.0, 16.0));
        let is_hovered = ui.rect_contains_pointer(hit_rect);

        // Draw connecting axis line from center to bubble
        let line_w = if is_front { 2.2_f32 } else { 1.2_f32 };
        painter.line_segment([center, bubble_pos], egui::Stroke::new(line_w, color));

        // Hover glow
        if is_hovered {
            painter.circle_stroke(
                bubble_pos,
                bubble_r + 2.5,
                egui::Stroke::new(1.8_f32, egui::Color32::from_rgb(255, 235, 80)),
            );
        }

        // Draw Axis Bubble
        painter.circle_filled(bubble_pos, bubble_r, color);
        if !label.is_empty() {
            painter.text(
                bubble_pos,
                egui::Align2::CENTER_CENTER,
                label,
                egui::FontId::monospace(9.0),
                egui::Color32::WHITE,
            );
        }

        if is_hovered {
            response.clone().on_hover_text(tip);
            if ui.input(|i| i.pointer.primary_clicked()) {
                camera.projection = render_viewport::CameraProjection::Orthographic;
                camera.yaw = target_yaw;
                camera.pitch = target_pitch;
                changed = true;
            }
        }
    }

    changed
}

pub fn render(state: &mut AppState) {
    let vp = state.egui_ctx.available_rect();
    let top_y_offset = vp.min.y + 12.0;

    // 1. Render Viewport Statistics Heads-Up Display (HUD)
    if state.show_viewport_hud {
        let left_offset = if state.show_left_toolbar { 76.0 } else { 14.0 };
        let hud_x = vp.min.x + left_offset;
        let hud_y = vp.min.y + 12.0;

        egui::Window::new("Scene Statistics")
            .id(egui::Id::new("viewport_hud_overlay_window"))
            .pivot(egui::Align2::LEFT_TOP)
            .default_pos(egui::pos2(hud_x, hud_y))
            .collapsible(true)
            .resizable(false)
            .title_bar(true)
            .frame(
                egui::Frame::popup(state.egui_ctx.style().as_ref())
                    .fill(egui::Color32::from_rgba_premultiplied(22, 24, 29, 230))
                    .stroke(egui::Stroke::new(
                        1.0_f32,
                        egui::Color32::from_white_alpha(30),
                    ))
                    .rounding(egui::Rounding::same(8.0))
                    .inner_margin(egui::Margin::symmetric(10.0, 7.0)),
            )
            .show(&state.egui_ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Mode:").small().weak());
                    ui.label(
                        egui::RichText::new(format!("{:?}", state.interaction_mode))
                            .small()
                            .color(egui::Color32::LIGHT_GRAY),
                    );
                });
                ui.separator();

                let mut total_verts = 0;
                let mut total_edges = 0;
                let mut total_faces = 0;
                let mut active_mesh_stats: Option<(String, usize, usize, usize, Vec3, Vec3)> = None;

                for (node_id, node) in &state.scene.nodes {
                    if let NodeData::Mesh { base_mesh, .. } = &node.data {
                        let v_cnt = base_mesh.vertices.len();
                        let e_cnt = base_mesh.edges.len();
                        let f_cnt = base_mesh.faces.len();
                        total_verts += v_cnt;
                        total_edges += e_cnt;
                        total_faces += f_cnt;

                        if Some(node_id) == state.selected_node {
                            active_mesh_stats = Some((
                                node.name.clone(),
                                v_cnt,
                                e_cnt,
                                f_cnt,
                                base_mesh.bounding_dimensions(),
                                base_mesh.bounding_center(),
                            ));
                        }
                    }
                }

                if let Some((name, v, e, f, dims, _center)) = active_mesh_stats {
                    ui.label(
                        egui::RichText::new(format!("Active: {}", name))
                            .color(egui::Color32::WHITE)
                            .strong(),
                    );
                    ui.horizontal(|ui| {
                        ui.monospace(format!("Verts: {} | Edges: {} | Faces: {}", v, e, f));
                    });
                    ui.horizontal(|ui| {
                        ui.monospace(format!(
                            "Dims: [{:.2}, {:.2}, {:.2}]",
                            dims.x, dims.y, dims.z
                        ));
                    });
                    if state.interaction_mode == InteractionMode::Edit {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(format!(
                                    "Selected: {} v | {} e | {} f",
                                    state.selected_vertices.len(),
                                    state.selected_edges.len(),
                                    state.selected_faces.len()
                                ))
                                .color(egui::Color32::YELLOW),
                            );
                        });
                    }
                } else {
                    ui.label(
                        egui::RichText::new("Total Scene Mesh Stats").color(egui::Color32::WHITE),
                    );
                    ui.monospace(format!(
                        "Verts: {} | Edges: {} | Faces: {}",
                        total_verts, total_edges, total_faces
                    ));
                }
            });
    }

    // 2. Render 3D Viewport Navigation Orbit Gizmo in top-right overlay area
    let compass_x = vp.max.x - 88.0;
    let camera = &mut state.camera;
    let mut compass_changed = false;
    egui::Area::new(egui::Id::new("viewport_compass_area"))
        .fixed_pos(egui::pos2(compass_x, top_y_offset))
        .show(&state.egui_ctx, |ui| {
            if draw_viewport_orientation_compass(camera, ui) {
                compass_changed = true;
            }
        });
    if compass_changed {
        state.window.request_redraw();
    }
}

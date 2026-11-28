use graph_engine::NodeData;

use crate::state::AppState;
use crate::ui::actions::UiActions;

pub fn draw_uv_editor(state: &mut AppState, actions: &mut UiActions) {
    if !state.show_uv_editor {
        return;
    }
    let mut is_open = state.show_uv_editor;
    let egui_ctx = state.egui_ctx.clone();
    egui::Window::new("2D UV Editor & Texture Coordinates")
        .open(&mut is_open)
        .default_size(egui::vec2(520.0, 560.0))
        .resizable(true)
        .show(&egui_ctx, |ui| {
            draw_uv_editor_inner(state, actions, ui);
        });
    state.show_uv_editor = is_open;
}

pub fn draw_uv_editor_inner(state: &mut AppState, actions: &mut UiActions, ui: &mut egui::Ui) {
    if let Some(sel_id) = state.selected_node {
        if let Some(node) = state.scene.nodes.get_mut(sel_id) {
            if let NodeData::Mesh { base_mesh, .. } = &mut node.data {
                ui.horizontal(|ui| {
                    if ui.button("Unwrap Seams").clicked() {
                        actions.trigger_unwrap_seams = true;
                    }
                    if ui.button("Smart Project").clicked() {
                        actions.trigger_smart_uv_project = true;
                    }
                    if ui.button("Mark Seam").clicked() {
                        actions.trigger_mark_seam = true;
                    }
                    if ui.button("Clear Seam").clicked() {
                        actions.trigger_clear_seam = true;
                    }
                    if ui.button("Clear All").clicked() {
                        actions.trigger_clear_all_seams = true;
                    }
                    ui.separator();
                    ui.checkbox(&mut state.uv_show_distortion, "Heatmap");
                    let cut_btn = if state.uv_seam_cut_mode { "Cut Seams [ON]" } else { "Cut Seams" };
                    if ui.selectable_label(state.uv_seam_cut_mode, cut_btn).on_hover_text("Interactive 2D Seam Cut: Click edges in UV canvas to cut/remove seams").clicked() {
                        state.uv_seam_cut_mode = !state.uv_seam_cut_mode;
                        if state.uv_seam_cut_mode { state.uv_pin_mode = false; }
                    }
                    let pin_btn = if state.uv_pin_mode { "Pin Tool [P] (ON)" } else { "Pin Tool [P]" };
                    if ui.selectable_label(state.uv_pin_mode, pin_btn).on_hover_text("Interactive UV Pin Tool: Click or press 'P' to pin/unpin UV vertices").clicked() {
                        state.uv_pin_mode = !state.uv_pin_mode;
                        if state.uv_pin_mode { state.uv_seam_cut_mode = false; }
                    }
                    ui.checkbox(&mut state.uv_live_relax, "Live Relax").on_hover_text("Iteratively relaxes unpinned UV coordinates in real-time as you drag vertices");
                });

                ui.horizontal(|ui| {
                    if ui.button("Pack Islands").clicked() {
                        actions.trigger_pack_uv_islands = true;
                    }
                    if ui.button("Rotate CW").clicked() {
                        actions.trigger_rotate_uv_cw = true;
                    }
                    if ui.button("Rotate CCW").clicked() {
                        actions.trigger_rotate_uv_ccw = true;
                    }
                    if ui.button("Flip H").clicked() {
                        actions.trigger_flip_uv_h = true;
                    }
                    if ui.button("Flip V").clicked() {
                        actions.trigger_flip_uv_v = true;
                    }
                    if ui.button("+ 1.2x").clicked() {
                        actions.trigger_scale_uv = Some(1.2);
                    }
                    if ui.button("- 0.8x").clicked() {
                        actions.trigger_scale_uv = Some(0.8);
                    }
                    if ui.button("Relax UVs").on_hover_text("Smooths vertex UVs using Laplacian diffusion to minimize distortion while pinning island boundaries").clicked() {
                        actions.trigger_relax_uvs = true;
                    }
                });

                ui.separator();

                let (response, painter) = ui.allocate_painter(
                    egui::vec2(ui.available_width(), 380.0),
                    egui::Sense::click_and_drag(),
                );
                let rect = response.rect;

                painter.rect_filled(rect, 4.0, egui::Color32::from_rgb(20, 22, 26));
                painter.rect_stroke(
                    rect,
                    4.0,
                    egui::Stroke::new(1.0_f32, egui::Color32::from_white_alpha(30)),
                );

                let grid_lines = 8;
                for i in 1..grid_lines {
                    let gx = rect.left() + (i as f32 / grid_lines as f32) * rect.width();
                    let gy = rect.top() + (i as f32 / grid_lines as f32) * rect.height();
                    painter.line_segment(
                        [egui::pos2(gx, rect.top()), egui::pos2(gx, rect.bottom())],
                        egui::Stroke::new(0.5_f32, egui::Color32::from_white_alpha(15)),
                    );
                    painter.line_segment(
                        [egui::pos2(rect.left(), gy), egui::pos2(rect.right(), gy)],
                        egui::Stroke::new(0.5_f32, egui::Color32::from_white_alpha(15)),
                    );
                }

                // Render ABF++ angle and area distortion heatmap overlay
                if state.uv_show_distortion {
                    let heatmaps = base_mesh.compute_uv_distortion_rgba();
                    for (fk, rgba) in heatmaps {
                        let verts = base_mesh.face_vertices(fk);
                        let mut poly_pts = Vec::with_capacity(verts.len());
                        for vk in verts {
                            let uv = base_mesh.vertices[vk].uv;
                            poly_pts.push(egui::pos2(
                                rect.left() + uv.x * rect.width(),
                                rect.top() + (1.0 - uv.y) * rect.height(),
                            ));
                        }
                        if poly_pts.len() >= 3 {
                            let fill_c = egui::Color32::from_rgba_unmultiplied(
                                rgba[0], rgba[1], rgba[2], rgba[3],
                            );
                            painter.add(egui::Shape::convex_polygon(
                                poly_pts,
                                fill_c,
                                egui::Stroke::NONE,
                            ));
                        }
                    }
                }

                for (fk, _face) in &base_mesh.faces {
                    let verts = base_mesh.face_vertices(fk);
                    let mut pts = Vec::new();
                    for vk in verts {
                        let uv = base_mesh.vertices[vk].uv;
                        let screen_pos = egui::pos2(
                            rect.left() + uv.x * rect.width(),
                            rect.top() + (1.0 - uv.y) * rect.height(),
                        );
                        pts.push(screen_pos);
                    }
                    if pts.len() >= 3 {
                        for i in 0..pts.len() {
                            let next_i = (i + 1) % pts.len();
                            painter.line_segment(
                                [pts[i], pts[next_i]],
                                egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(70, 160, 255)),
                            );
                        }
                    }
                }

                // Seam cut detection and rendering
                let mouse_pos = response.hover_pos();
                let mut closest_edge = None;
                let mut min_edge_dist = 8.0_f32;

                for (ek, edge) in &base_mesh.edges {
                    let he = &base_mesh.half_edges[edge.half_edge];
                    let uv0 = base_mesh.vertices[he.origin].uv;
                    let uv1 = base_mesh.vertices[he.target].uv;
                    let p0 = egui::pos2(
                        rect.left() + uv0.x * rect.width(),
                        rect.top() + (1.0 - uv0.y) * rect.height(),
                    );
                    let p1 = egui::pos2(
                        rect.left() + uv1.x * rect.width(),
                        rect.top() + (1.0 - uv1.y) * rect.height(),
                    );

                    if state.uv_seam_cut_mode {
                        if let Some(m) = mouse_pos {
                            let d = distance_to_segment(m, p0, p1);
                            if d < min_edge_dist {
                                min_edge_dist = d;
                                closest_edge = Some((ek, p0, p1));
                            }
                        }
                    }

                    if edge.is_seam {
                        painter.line_segment(
                            [p0, p1],
                            egui::Stroke::new(2.5_f32, egui::Color32::from_rgb(255, 50, 50)),
                        );
                    }
                }

                if state.uv_seam_cut_mode {
                    if let Some((ek, p0, p1)) = closest_edge {
                        painter.line_segment(
                            [p0, p1],
                            egui::Stroke::new(3.0_f32, egui::Color32::YELLOW),
                        );
                        if response.clicked() {
                            base_mesh.edges[ek].is_seam = !base_mesh.edges[ek].is_seam;
                        }
                    }
                }

                // Interactive UV vertex picking, pinning markers, and drag relaxation
                let mut closest_vert = None;
                let mut min_vert_dist = 12.0_f32;

                for (vk, v) in &base_mesh.vertices {
                    let vp = egui::pos2(
                        rect.left() + v.uv.x * rect.width(),
                        rect.top() + (1.0 - v.uv.y) * rect.height(),
                    );
                    if let Some(m) = mouse_pos {
                        let d = (m - vp).length();
                        if d < min_vert_dist {
                            min_vert_dist = d;
                            closest_vert = Some((vk, vp));
                        }
                    }

                    if v.uv_pinned {
                        // Draw red pushpin marker for pinned UV vertices
                        painter.circle_filled(vp, 4.5_f32, egui::Color32::from_rgb(255, 45, 45));
                        painter.circle_stroke(
                            vp,
                            4.5_f32,
                            egui::Stroke::new(1.5_f32, egui::Color32::WHITE),
                        );
                        painter.circle_filled(vp, 1.5_f32, egui::Color32::WHITE);
                    }
                }

                let p_pressed = ui.input(|i| i.key_pressed(egui::Key::P));
                if let Some((cvk, cvp)) = closest_vert {
                    painter.circle_stroke(
                        cvp,
                        6.5_f32,
                        egui::Stroke::new(1.5_f32, egui::Color32::YELLOW),
                    );
                    if p_pressed || (state.uv_pin_mode && response.clicked()) {
                        base_mesh.vertices[cvk].uv_pinned = !base_mesh.vertices[cvk].uv_pinned;
                    }
                }

                // Dragging UV vertices with optional real-time live relaxation
                if response.dragged() {
                    if let Some(m) = mouse_pos {
                        if let Some((cvk, _)) = closest_vert {
                            let new_u = ((m.x - rect.left()) / rect.width()).clamp(0.0, 1.0);
                            let new_v = (1.0 - (m.y - rect.top()) / rect.height()).clamp(0.0, 1.0);
                            base_mesh.vertices[cvk].uv = core_math::vec2(new_u, new_v);
                            if state.uv_live_relax {
                                base_mesh.relax_uvs(2, 0.45);
                            }
                        }
                    }
                }

                ui.separator();
                let summary = base_mesh.summarize_uv_distortion();
                ui.horizontal(|ui| {
                    ui.label(format!(
                        "Mesh: {} | V: {} | F: {}",
                        node.name,
                        base_mesh.vertices.len(),
                        base_mesh.faces.len()
                    ));
                    ui.separator();
                    ui.label(
                        egui::RichText::new(format!(
                            "Angle Err: {:.1} deg | Area Stretch: {:.2}x",
                            summary.mean_angle_error_deg, summary.mean_area_stretch
                        ))
                        .small()
                        .color(egui::Color32::from_rgb(180, 200, 230)),
                    );
                });
            }
        }
    } else {
        ui.label("Select a mesh object in the 3D scene to view and edit its UV coordinates.");
    }
}

fn distance_to_segment(p: egui::Pos2, a: egui::Pos2, b: egui::Pos2) -> f32 {
    let ab = b - a;
    let len_sq = ab.length_sq();
    if len_sq < 1e-6 {
        return (p - a).length();
    }
    let t = ((p - a).dot(ab) / len_sq).clamp(0.0, 1.0);
    let proj = a + ab * t;
    (p - proj).length()
}

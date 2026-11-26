use core_math::Vec3;
use graph_engine::NodeData;
use render_viewport::GizmoAxis;

use crate::state::AppState;
use crate::types::*;

pub fn render(state: &mut AppState) {
    let width = state.surface_config.width as f32;
    let height = state.surface_config.height as f32;

    // 1. Knife Tool Overlay (Blender 'K' / Maya Multi-Cut)
    if state.knife_active {
        let painter = state.egui_ctx.layer_painter(egui::LayerId::new(
            egui::Order::Foreground,
            egui::Id::new("knife_overlay"),
        ));

        for i in 0..state.knife_points.len() {
            let (_, p_curr, _) = state.knife_points[i];
            if let Some(s_curr) = state.camera.world_to_screen(p_curr, width, height) {
                let pt = egui::pos2(s_curr.x, s_curr.y);
                painter.circle_filled(pt, 5.0, egui::Color32::from_rgb(0, 255, 200));
                painter.circle_stroke(pt, 5.0, egui::Stroke::new(1.0_f32, egui::Color32::BLACK));

                if i > 0 {
                    let (_, p_prev, _) = state.knife_points[i - 1];
                    if let Some(s_prev) = state.camera.world_to_screen(p_prev, width, height) {
                        painter.line_segment(
                            [egui::pos2(s_prev.x, s_prev.y), pt],
                            egui::Stroke::new(2.5_f32, egui::Color32::from_rgb(0, 255, 200)),
                        );
                    }
                }
            }
        }

        if let Some((_, h_pos, _)) = state.knife_hovered_hit {
            if let Some(s_hov) = state.camera.world_to_screen(h_pos, width, height) {
                let h_pt = egui::pos2(s_hov.x, s_hov.y);
                painter.circle_filled(h_pt, 6.0, egui::Color32::from_rgb(255, 100, 100));
                painter.circle_stroke(h_pt, 6.0, egui::Stroke::new(1.5_f32, egui::Color32::WHITE));

                if let Some((_, last_p, _)) = state.knife_points.last() {
                    if let Some(s_last) = state.camera.world_to_screen(*last_p, width, height) {
                        painter.line_segment(
                            [egui::pos2(s_last.x, s_last.y), h_pt],
                            egui::Stroke::new(2.0_f32, egui::Color32::from_rgb(255, 180, 50)),
                        );
                    }
                }
            }
        }

        let banner_rect = egui::Rect::from_center_size(
            egui::pos2(width * 0.5, 45.0),
            egui::vec2(540.0, 28.0),
        );
        painter.rect_filled(banner_rect, 5.0, egui::Color32::from_black_alpha(220));
        painter.rect_stroke(banner_rect, 5.0, egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(0, 255, 200)));
        painter.text(
            banner_rect.center(),
            egui::Align2::CENTER_CENTER,
            format!("KNIFE MULTI-CUT: {} points | Click: Add Point | Enter: Confirm | Esc: Cancel", state.knife_points.len()),
            egui::FontId::proportional(12.5),
            egui::Color32::from_rgb(240, 255, 255),
        );
    }

    // 2. Loop Cut Preview Overlay (Blender Ctrl+R / Maya Insert Edge Loop)
    if state.loop_cut_active {
        let painter = state.egui_ctx.layer_painter(egui::LayerId::new(
            egui::Order::Foreground,
            egui::Id::new("loop_cut_overlay"),
        ));

        if let Some(sel_id) = state.selected_node {
            if let Some(node) = state.scene.nodes.get(sel_id) {
                if let NodeData::Mesh { base_mesh, .. } = &node.data {
                    if let Some(edge) = state.loop_cut_hovered_edge {
                        let cuts = state.loop_cut_num_cuts;
                        let factor = state.loop_cut_factor;
                        let stroke_color = if state.loop_cut_sliding {
                            egui::Color32::from_rgb(255, 120, 0)
                        } else {
                            egui::Color32::from_rgb(255, 230, 0)
                        };

                        let lines = base_mesh.get_loop_cut_multi_preview(edge, cuts, factor);
                        for (p0, p1) in lines {
                            let w0 = node.world_matrix.transform_point3(p0);
                            let w1 = node.world_matrix.transform_point3(p1);
                            if let (Some(s0), Some(s1)) = (
                                state.camera.world_to_screen(w0, width, height),
                                state.camera.world_to_screen(w1, width, height),
                            ) {
                                painter.line_segment(
                                    [egui::pos2(s0.x, s0.y), egui::pos2(s1.x, s1.y)],
                                    egui::Stroke::new(3.0_f32, stroke_color),
                                );
                            }
                        }
                    }
                }
            }
        }

        let banner_rect = egui::Rect::from_center_size(
            egui::pos2(width * 0.5, 45.0),
            egui::vec2(580.0, 28.0),
        );
        painter.rect_filled(banner_rect, 5.0, egui::Color32::from_black_alpha(220));
        let border_color = if state.loop_cut_sliding {
            egui::Color32::from_rgb(255, 150, 0)
        } else {
            egui::Color32::from_rgb(255, 230, 0)
        };
        painter.rect_stroke(banner_rect, 5.0, egui::Stroke::new(1.0_f32, border_color));
        let banner_text = if !state.loop_cut_sliding {
            format!("LOOP CUT: Hover edge | Scroll: Cuts ({}) | Left-Click: Slide | Esc: Cancel", state.loop_cut_num_cuts)
        } else {
            format!("SLIDE LOOP: Factor: {:.1}% | Left-Click / Enter: Confirm | Esc: Cancel", state.loop_cut_factor * 100.0)
        };
        painter.text(
            banner_rect.center(),
            egui::Align2::CENTER_CENTER,
            banner_text,
            egui::FontId::proportional(12.5),
            egui::Color32::from_rgb(255, 255, 240),
        );
    }

    // 3. Extrude Modal Preview Overlay (Blender 'E' / Maya Extrude)
    if state.extrude_modal_active {
        let painter = state.egui_ctx.layer_painter(egui::LayerId::new(
            egui::Order::Foreground,
            egui::Id::new("extrude_modal_overlay"),
        ));

        if let Some(sel_id) = state.selected_node {
            if let Some(node) = state.scene.nodes.get(sel_id) {
                if let NodeData::Mesh { base_mesh, .. } = &node.data {
                    for &fk in &state.extrude_modal_faces {
                        if let Some(face) = base_mesh.faces.get(fk) {
                            let c = node.world_matrix.transform_point3(base_mesh.face_centroid(fk));
                            let dir = match state.extrude_modal_axis_constraint {
                                Some(GizmoAxis::X) => Vec3::X,
                                Some(GizmoAxis::Y) => Vec3::Y,
                                Some(GizmoAxis::Z) => Vec3::Z,
                                Some(GizmoAxis::XY) => (Vec3::X + Vec3::Y).normalize(),
                                Some(GizmoAxis::XZ) => (Vec3::X + Vec3::Z).normalize(),
                                Some(GizmoAxis::YZ) => (Vec3::Y + Vec3::Z).normalize(),
                                Some(GizmoAxis::Screen) => state.camera.right(),
                                None => node.world_matrix.transform_vector3(face.normal).normalize_or_zero(),
                            };
                            let target_p = c + dir * state.extrude_modal_distance;
                            if let (Some(s_c), Some(s_t)) = (
                                state.camera.world_to_screen(c, width, height),
                                state.camera.world_to_screen(target_p, width, height),
                            ) {
                                painter.line_segment(
                                    [egui::pos2(s_c.x, s_c.y), egui::pos2(s_t.x, s_t.y)],
                                    egui::Stroke::new(2.5_f32, egui::Color32::from_rgb(255, 215, 0)),
                                );
                                painter.circle_filled(
                                    egui::pos2(s_t.x, s_t.y),
                                    4.5_f32,
                                    egui::Color32::from_rgb(255, 255, 120),
                                );
                            }
                        }
                    }
                }
            }
        }

        let mut commit_extrude = false;
        let mut cancel_extrude = false;
        let ctx = state.egui_ctx.clone();
        egui::Area::new(egui::Id::new("interactive_extrude_hud"))
            .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 48.0))
            .order(egui::Order::Foreground)
            .show(&ctx, |ui| {
                egui::Frame::window(ui.style())
                    .fill(egui::Color32::from_rgba_premultiplied(20, 24, 30, 240))
                    .stroke(egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(255, 215, 0)))
                    .rounding(egui::Rounding::same(6.0))
                    .inner_margin(egui::Margin::symmetric(14.0, 7.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("EXTRUDE").strong().color(egui::Color32::from_rgb(255, 215, 0)));
                            ui.separator();
                            ui.label("Distance:");
                            if ui.add(egui::DragValue::new(&mut state.extrude_modal_distance).speed(0.02).suffix(" m")).changed() {
                                state.modal_numeric_buffer.clear();
                            }
                            ui.separator();
                            ui.label("Axis:");
                            if ui.selectable_label(state.extrude_modal_axis_constraint.is_none(), "Normal (C)").clicked() {
                                state.extrude_modal_axis_constraint = None;
                            }
                            if ui.selectable_label(state.extrude_modal_axis_constraint == Some(GizmoAxis::X), "X").clicked() {
                                state.extrude_modal_axis_constraint = Some(GizmoAxis::X);
                            }
                            if ui.selectable_label(state.extrude_modal_axis_constraint == Some(GizmoAxis::Y), "Y").clicked() {
                                state.extrude_modal_axis_constraint = Some(GizmoAxis::Y);
                            }
                            if ui.selectable_label(state.extrude_modal_axis_constraint == Some(GizmoAxis::Z), "Z").clicked() {
                                state.extrude_modal_axis_constraint = Some(GizmoAxis::Z);
                            }
                            if !state.modal_numeric_buffer.is_empty() {
                                ui.label(egui::RichText::new(format!("Input: [{}]", state.modal_numeric_buffer)).monospace().strong().color(egui::Color32::YELLOW));
                            }
                            ui.separator();
                            if ui.button("Confirm (Enter)").clicked() {
                                commit_extrude = true;
                            }
                            if ui.button("Cancel (Esc)").clicked() {
                                cancel_extrude = true;
                            }
                        });
                    });
            });

        if commit_extrude {
            crate::modals::extrude::commit(state);
            return;
        }
        if cancel_extrude {
            crate::modals::extrude::cancel(state);
            return;
        }
    }

    // 4. Bevel Modal Preview Overlay (Blender Ctrl+B / Maya Bevel)
    if state.bevel_modal_active {
        let painter = state.egui_ctx.layer_painter(egui::LayerId::new(
            egui::Order::Foreground,
            egui::Id::new("bevel_modal_overlay"),
        ));

        if let Some(sel_id) = state.selected_node {
            if let Some(node) = state.scene.nodes.get(sel_id) {
                if let NodeData::Mesh { base_mesh, .. } = &node.data {
                    for &ek in &state.bevel_modal_edges {
                        if let Some(edge) = base_mesh.edges.get(ek) {
                            let h = &base_mesh.half_edges[edge.half_edge];
                            let p0 = node.world_matrix.transform_point3(base_mesh.vertices[h.origin].position);
                            let p1 = node.world_matrix.transform_point3(base_mesh.vertices[h.target].position);
                            if let (Some(s0), Some(s1)) = (
                                state.camera.world_to_screen(p0, width, height),
                                state.camera.world_to_screen(p1, width, height),
                            ) {
                                painter.line_segment(
                                    [egui::pos2(s0.x, s0.y), egui::pos2(s1.x, s1.y)],
                                    egui::Stroke::new(3.5_f32, egui::Color32::from_rgb(0, 230, 255)),
                                );
                            }
                        }
                    }
                }
            }
        }

        let mut commit_bevel = false;
        let mut cancel_bevel = false;
        let ctx = state.egui_ctx.clone();
        egui::Area::new(egui::Id::new("interactive_bevel_hud"))
            .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 48.0))
            .order(egui::Order::Foreground)
            .show(&ctx, |ui| {
                egui::Frame::window(ui.style())
                    .fill(egui::Color32::from_rgba_premultiplied(20, 24, 30, 240))
                    .stroke(egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(0, 230, 255)))
                    .rounding(egui::Rounding::same(6.0))
                    .inner_margin(egui::Margin::symmetric(14.0, 7.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("BEVEL").strong().color(egui::Color32::from_rgb(0, 230, 255)));
                            ui.separator();
                            ui.label("Width:");
                            if ui.add(egui::DragValue::new(&mut state.bevel_modal_width).speed(0.005).suffix(" m").range(0.001..=5.0)).changed() {
                                state.modal_numeric_buffer.clear();
                            }
                            ui.label("Segments:");
                            ui.add(egui::DragValue::new(&mut state.bevel_modal_segments).range(1..=16));
                            if !state.modal_numeric_buffer.is_empty() {
                                ui.label(egui::RichText::new(format!("Input: [{}]", state.modal_numeric_buffer)).monospace().strong().color(egui::Color32::YELLOW));
                            }
                            ui.separator();
                            if ui.button("Confirm (Enter)").clicked() {
                                commit_bevel = true;
                            }
                            if ui.button("Cancel (Esc)").clicked() {
                                cancel_bevel = true;
                            }
                        });
                    });
            });

        if commit_bevel {
            crate::modals::bevel::commit(state);
            return;
        }
        if cancel_bevel {
            crate::modals::bevel::cancel(state);
            return;
        }
    }

    // 5. Weight Gradient Overlay
    if state.interaction_mode == InteractionMode::WeightPaint && state.weight_paint_tool == WeightPaintTool::LinearGradient {
        if let (Some(start), Some(end)) = (state.weight_gradient_start, state.weight_gradient_end) {
            let painter = state.egui_ctx.layer_painter(egui::LayerId::new(
                egui::Order::Foreground,
                egui::Id::new("weight_gradient_overlay"),
            ));

            if let (Some(s0), Some(s1)) = (
                state.camera.world_to_screen(start, width, height),
                state.camera.world_to_screen(end, width, height),
            ) {
                let p0 = egui::pos2(s0.x, s0.y);
                let p1 = egui::pos2(s1.x, s1.y);

                let num_segs = 16;
                for i in 0..num_segs {
                    let t0 = i as f32 / num_segs as f32;
                    let t1 = (i + 1) as f32 / num_segs as f32;
                    let seg_p0 = p0 + (p1 - p0) * t0;
                    let seg_p1 = p0 + (p1 - p0) * t1;

                    let w = state.weight_gradient_start_w + (state.weight_gradient_end_w - state.weight_gradient_start_w) * ((t0 + t1) * 0.5);
                    let col = if w <= 0.5 {
                        let f = w * 2.0;
                        egui::Color32::from_rgb((f * 30.0) as u8, (f * 220.0) as u8, ((1.0 - f) * 255.0) as u8)
                    } else {
                        let f = (w - 0.5) * 2.0;
                        egui::Color32::from_rgb(((0.5 + 0.5 * f) * 255.0) as u8, ((1.0 - f) * 220.0) as u8, 20)
                    };

                    painter.line_segment([seg_p0, seg_p1], egui::Stroke::new(3.5_f32, col));
                }

                painter.circle_filled(p0, 6.0_f32, egui::Color32::from_rgb(255, 50, 50));
                painter.circle_stroke(p0, 6.0_f32, egui::Stroke::new(1.5_f32, egui::Color32::WHITE));
                painter.circle_filled(p1, 6.0_f32, egui::Color32::from_rgb(50, 100, 255));
                painter.circle_stroke(p1, 6.0_f32, egui::Stroke::new(1.5_f32, egui::Color32::WHITE));

                painter.text(
                    p0 + egui::vec2(0.0, -12.0),
                    egui::Align2::CENTER_BOTTOM,
                    format!("{:.2}", state.weight_gradient_start_w),
                    egui::FontId::proportional(11.0),
                    egui::Color32::WHITE,
                );
                painter.text(
                    p1 + egui::vec2(0.0, -12.0),
                    egui::Align2::CENTER_BOTTOM,
                    format!("{:.2}", state.weight_gradient_end_w),
                    egui::FontId::proportional(11.0),
                    egui::Color32::WHITE,
                );
            }

            let banner_rect = egui::Rect::from_center_size(
                egui::pos2(width * 0.5, 45.0),
                egui::vec2(600.0, 28.0),
            );
            painter.rect_filled(banner_rect, 5.0, egui::Color32::from_black_alpha(220));
            painter.rect_stroke(banner_rect, 5.0, egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(255, 180, 50)));
            let banner_text = format!(
                "LINEAR WEIGHT GRADIENT: Drag Start -> End | W: {:.2} -> {:.2} | Release to Apply",
                state.weight_gradient_start_w, state.weight_gradient_end_w
            );
            painter.text(
                banner_rect.center(),
                egui::Align2::CENTER_CENTER,
                banner_text,
                egui::FontId::proportional(12.5),
                egui::Color32::from_rgb(255, 240, 200),
            );
        }
    }

    // 6. Inset Modal Preview Overlay (Blender 'I' / Maya Inset)
    if state.inset_modal_active {
        let painter = state.egui_ctx.layer_painter(egui::LayerId::new(
            egui::Order::Foreground,
            egui::Id::new("inset_modal_overlay"),
        ));

        if let Some(sel_id) = state.selected_node {
            if let Some(node) = state.scene.nodes.get(sel_id) {
                if let NodeData::Mesh { base_mesh, .. } = &node.data {
                    for &fk in &state.inset_modal_faces {
                        let fverts = base_mesh.face_vertices(fk);
                        let c = base_mesh.face_centroid(fk);
                        let pts: Vec<egui::Pos2> = fverts.iter().filter_map(|&vk| {
                            let orig = base_mesh.vertices[vk].position;
                            let inset_pos = c + (orig - c) * state.inset_modal_factor;
                            let w = node.world_matrix.transform_point3(inset_pos);
                            state.camera.world_to_screen(w, width, height).map(|s| egui::pos2(s.x, s.y))
                        }).collect();

                        if pts.len() >= 3 {
                            for i in 0..pts.len() {
                                painter.line_segment([pts[i], pts[(i + 1) % pts.len()]], egui::Stroke::new(2.5_f32, egui::Color32::from_rgb(255, 180, 0)));
                            }

                            if state.inset_modal_depth.abs() > 0.001 {
                                let norm = node.world_matrix.transform_vector3(base_mesh.faces[fk].normal).normalize_or_zero();
                                let recessed_pts: Vec<egui::Pos2> = fverts.iter().filter_map(|&vk| {
                                    let orig = base_mesh.vertices[vk].position;
                                    let inset_pos = c + (orig - c) * state.inset_modal_factor;
                                    let w = node.world_matrix.transform_point3(inset_pos) - norm * state.inset_modal_depth;
                                    state.camera.world_to_screen(w, width, height).map(|s| egui::pos2(s.x, s.y))
                                }).collect();

                                if recessed_pts.len() == pts.len() {
                                    let bore_color = egui::Color32::from_rgb(255, 120, 20);
                                    for i in 0..pts.len() {
                                        painter.line_segment([recessed_pts[i], recessed_pts[(i + 1) % pts.len()]], egui::Stroke::new(1.5_f32, bore_color));
                                        painter.line_segment([pts[i], recessed_pts[i]], egui::Stroke::new(1.0_f32, bore_color));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        let mut commit_inset = false;
        let mut cancel_inset = false;
        let ctx = state.egui_ctx.clone();
        egui::Area::new(egui::Id::new("interactive_inset_hud"))
            .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 48.0))
            .order(egui::Order::Foreground)
            .show(&ctx, |ui| {
                egui::Frame::window(ui.style())
                    .fill(egui::Color32::from_rgba_premultiplied(20, 24, 30, 240))
                    .stroke(egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(255, 180, 0)))
                    .rounding(egui::Rounding::same(6.0))
                    .inner_margin(egui::Margin::symmetric(14.0, 7.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("INSET").strong().color(egui::Color32::from_rgb(255, 180, 0)));
                            ui.separator();
                            ui.label("Thickness:");
                            let r = state.inset_modal_initial_radius.max(0.001);
                            if ui.add(egui::DragValue::new(&mut state.inset_modal_thickness).speed(0.01).suffix(" m").range(0.0..=r * 0.99)).changed() {
                                state.inset_modal_factor = (1.0 - state.inset_modal_thickness / r).clamp(0.01, 0.99);
                                state.modal_numeric_buffer.clear();
                            }
                            ui.label("Factor:");
                            let mut pct = state.inset_modal_factor * 100.0;
                            if ui.add(egui::DragValue::new(&mut pct).speed(0.5).suffix(" %").range(1.0..=99.0)).changed() {
                                state.inset_modal_factor = (pct / 100.0).clamp(0.01, 0.99);
                                state.inset_modal_thickness = (1.0 - state.inset_modal_factor) * r;
                                state.modal_numeric_buffer.clear();
                            }
                            if ui.checkbox(&mut state.inset_modal_individual, "Individual (I)").changed() {
                                state.window.request_redraw();
                            }
                            ui.label("Depth (Ctrl):");
                            if ui.add(egui::DragValue::new(&mut state.inset_modal_depth).speed(0.01).suffix(" m")).changed() {
                                state.window.request_redraw();
                            }
                            if !state.modal_numeric_buffer.is_empty() {
                                ui.label(egui::RichText::new(format!("Input: [{}]", state.modal_numeric_buffer)).monospace().strong().color(egui::Color32::YELLOW));
                            }
                            ui.separator();
                            if ui.button("Confirm (Enter)").clicked() {
                                commit_inset = true;
                            }
                            if ui.button("Cancel (Esc)").clicked() {
                                cancel_inset = true;
                            }
                        });
                    });
            });

        if commit_inset {
            crate::modals::inset::commit(state);
            return;
        }
        if cancel_inset {
            crate::modals::inset::cancel(state);
            return;
        }
    }

    // 7. Interactive Edge Crease Modal Banner (Blender Shift+E / Maya Crease)
    if state.edge_crease_modal_active {
        let mut commit_edge_crease = false;
        let mut cancel_edge_crease = false;
        let ctx = state.egui_ctx.clone();
        egui::Area::new(egui::Id::new("interactive_edge_crease_hud"))
            .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 48.0))
            .order(egui::Order::Foreground)
            .show(&ctx, |ui| {
                egui::Frame::window(ui.style())
                    .fill(egui::Color32::from_rgba_premultiplied(20, 24, 30, 240))
                    .stroke(egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(217, 70, 239)))
                    .rounding(egui::Rounding::same(6.0))
                    .inner_margin(egui::Margin::symmetric(14.0, 7.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("EDGE CREASE").strong().color(egui::Color32::from_rgb(217, 70, 239)));
                            ui.separator();
                            ui.label("Weight:");
                            let mut pct = state.edge_crease_modal_weight * 100.0;
                            if ui.add(egui::DragValue::new(&mut pct).speed(1.0).suffix(" %").range(0.0..=100.0)).changed() {
                                state.edge_crease_modal_weight = (pct / 100.0).clamp(0.0, 1.0);
                            }
                            ui.separator();
                            if ui.button("Confirm (Enter)").clicked() { commit_edge_crease = true; }
                            if ui.button("Cancel (Esc)").clicked() { cancel_edge_crease = true; }
                        });
                    });
            });

        if commit_edge_crease { crate::modals::edge_crease::commit(state); return; }
        if cancel_edge_crease { crate::modals::edge_crease::cancel(state); return; }
    }

    // 8. Interactive Vertex Crease Modal Banner (Blender Shift+V / OpenSubdiv Semi-Sharp Corners)
    if state.vertex_crease_modal_active {
        let mut commit_vertex_crease = false;
        let mut cancel_vertex_crease = false;
        let ctx = state.egui_ctx.clone();
        egui::Area::new(egui::Id::new("interactive_vertex_crease_hud"))
            .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 48.0))
            .order(egui::Order::Foreground)
            .show(&ctx, |ui| {
                egui::Frame::window(ui.style())
                    .fill(egui::Color32::from_rgba_premultiplied(20, 24, 30, 240))
                    .stroke(egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(56, 189, 248)))
                    .rounding(egui::Rounding::same(6.0))
                    .inner_margin(egui::Margin::symmetric(14.0, 7.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("VERTEX CREASE").strong().color(egui::Color32::from_rgb(56, 189, 248)));
                            ui.separator();
                            ui.label("Weight:");
                            let mut pct = state.vertex_crease_modal_weight * 100.0;
                            if ui.add(egui::DragValue::new(&mut pct).speed(1.0).suffix(" %").range(0.0..=100.0)).changed() {
                                state.vertex_crease_modal_weight = (pct / 100.0).clamp(0.0, 1.0);
                            }
                            ui.separator();
                            if ui.button("Confirm (Enter)").clicked() { commit_vertex_crease = true; }
                            if ui.button("Cancel (Esc)").clicked() { cancel_vertex_crease = true; }
                        });
                    });
            });

        if commit_vertex_crease { crate::modals::vertex_crease::commit(state); }
        else if cancel_vertex_crease { crate::modals::vertex_crease::cancel(state); }
    }

    // 9. Hardware Kitbash / Insert Mesh HUD
    if let Some(kb) = &state.kitbash_modal {
        let ctx = state.egui_ctx.clone();
        let hw_label = kb.hardware_type.label();
        let scale = kb.scale;
        let is_sub = kb.is_subtractive;
        let mut exit_kb = false;
        let mut toggle_sub = false;

        egui::Area::new(egui::Id::new("kitbash_hardware_hud"))
            .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 48.0))
            .order(egui::Order::Foreground)
            .show(&ctx, |ui| {
                egui::Frame::window(ui.style())
                    .fill(egui::Color32::from_rgba_premultiplied(20, 24, 30, 245))
                    .stroke(egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(255, 180, 50)))
                    .rounding(egui::Rounding::same(6.0))
                    .inner_margin(egui::Margin::symmetric(14.0, 7.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("HARDWARE INSERT").strong().color(egui::Color32::from_rgb(255, 180, 50)));
                            ui.separator();
                            ui.label(egui::RichText::new(hw_label).strong());
                            ui.label(format!("Scale: {:.2}x", scale));
                            let mode_txt = if is_sub { "Socket Cut (B)" } else { "Union (B)" };
                            if ui.button(mode_txt).clicked() { toggle_sub = true; }
                            ui.separator();
                            ui.label(egui::RichText::new("LMB: Drop | Wheel: Scale | Tab: Cycle").weak().small());
                            if ui.button("Exit (Esc)").clicked() { exit_kb = true; }
                        });
                    });
            });

        if toggle_sub {
            if let Some(kb) = &mut state.kitbash_modal { kb.is_subtractive = !kb.is_subtractive; }
        }
        if exit_kb {
            state.kitbash_modal = None;
            state.set_status_message("Exited Kitbash Tool");
        }
    }
}

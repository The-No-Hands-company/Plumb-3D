use core_math::Vec3;
use graph_engine::{Constraint, InterpolationMode, NodeData};
use render_viewport::{CameraProjection, GizmoMode};

use crate::state::AppState;
use crate::types::*;
use crate::ui::actions::UiActions;
use crate::ui::workspaces::WorkspaceTemplate;

pub fn draw_status_bar(state: &mut AppState) {
        egui::TopBottomPanel::bottom("bottom_status_bar").show(&state.egui_ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(format!("FPS: {:.0}", state.fps));
                ui.separator();

                let mode_label = match state.interaction_mode {
                    InteractionMode::Object => egui::RichText::new("OBJECT MODE").strong().color(egui::Color32::from_rgb(100, 200, 255)),
                    InteractionMode::Edit => egui::RichText::new(format!("EDIT MODE: {:?}", state.sub_element_level)).strong().color(egui::Color32::from_rgb(255, 150, 70)),
                    InteractionMode::Sculpt => egui::RichText::new(format!("SCULPT MODE: {:?}", state.sculpt_settings.brush)).strong().color(egui::Color32::from_rgb(255, 90, 90)),
                    InteractionMode::WeightPaint => {
                        let group = state.active_vertex_group.as_deref().unwrap_or("None");
                        let tool_str = match state.weight_paint_tool {
                            WeightPaintTool::Brush => format!("Brush W: {:.2}", state.weight_brush_target),
                            WeightPaintTool::LinearGradient => format!("Gradient: {:.2} -> {:.2}", state.weight_gradient_start_w, state.weight_gradient_end_w),
                            WeightPaintTool::NormalizeBrush => "Normalize Bone Weights".to_string(),
                        };
                        egui::RichText::new(format!("WEIGHT PAINT: {} ({})", group, tool_str))
                            .strong()
                            .color(egui::Color32::from_rgb(255, 120, 40))
                    }
                    InteractionMode::VertexPaint => {
                        egui::RichText::new(format!("VERTEX PAINT: Color [{:.2}, {:.2}, {:.2}]", state.vertex_paint_color[0], state.vertex_paint_color[1], state.vertex_paint_color[2]))
                            .strong()
                            .color(egui::Color32::from_rgb(255, 120, 180))
                    }
                    InteractionMode::TexturePaint => {
                        egui::RichText::new(format!("TEXTURE PAINT: {:?} ({:?})", state.texture_paint_brush.channel, state.texture_paint_brush.blend_mode))
                            .strong()
                            .color(egui::Color32::from_rgb(255, 180, 50))
                    }
                    InteractionMode::Pose => egui::RichText::new("POSE MODE (RIGGING)").strong().color(egui::Color32::from_rgb(80, 240, 200)),
                    InteractionMode::CurveEdit => egui::RichText::new("CURVE EDIT MODE").strong().color(egui::Color32::from_rgb(100, 220, 255)),
                };
                ui.label(mode_label);
                ui.separator();

                let gizmo_label = match state.gizmo.mode {
                    GizmoMode::Translate => "Translate (W)",
                    GizmoMode::Rotate => "Rotate (E)",
                    GizmoMode::Scale => "Scale (R)",
                };
                ui.label(format!("Gizmo: {}", gizmo_label));
                ui.separator();

                let orient_label = match state.transform_orientation {
                    TransformOrientation::Global => "Global",
                    TransformOrientation::Local => "Local",
                    TransformOrientation::Normal => "Normal",
                    TransformOrientation::View => "View",
                };
                if ui.button(format!("Space (,): {}", orient_label)).clicked() {
                    state.transform_orientation = match state.transform_orientation {
                        TransformOrientation::Global => TransformOrientation::Local,
                        TransformOrientation::Local => TransformOrientation::Normal,
                        TransformOrientation::Normal => TransformOrientation::View,
                        TransformOrientation::View => TransformOrientation::Global,
                    };
                }
                ui.separator();

                let cam_label = match state.camera.projection {
                    CameraProjection::Perspective => "Persp",
                    CameraProjection::Orthographic => "Ortho",
                };
                ui.label(format!("Cam: {}", cam_label));
                ui.separator();

                if let Some(sel_id) = state.selected_node {
                    if let Some(node) = state.scene.nodes.get(sel_id) {
                        if let NodeData::Mesh { base_mesh, .. } = &node.data {
                            ui.label(format!("V: {}", base_mesh.vertex_count()));
                            ui.separator();
                            ui.label(format!("E: {}", base_mesh.edge_count()));
                            ui.separator();
                            ui.label(format!("F: {}", base_mesh.face_count()));
                            ui.separator();
                            ui.label(format!("Euler Chi: {}", base_mesh.euler_characteristic()));
                            ui.separator();
                            match state.sub_element_level {
                                SubElementLevel::Vertex => {
                                    ui.label(format!("Selected Vertices: {}", state.selected_vertices.len()));
                                }
                                SubElementLevel::Edge => {
                                    ui.label(format!("Selected Edges: {}", state.selected_edges.len()));
                                }
                                SubElementLevel::Face => {
                                    ui.label(format!("Selected Faces: {}", state.selected_faces.len()));
                                }
                            }
                        }
                    }
                } else {
                    ui.label("No selection (Click in viewport to select)");
                }

                if let Some(ref display) = state.autosave.last_autosave_display {
                    ui.separator();
                    ui.label(egui::RichText::new(format!("Auto: {}", display)).size(11.0).color(egui::Color32::from_rgb(140, 160, 190)));
                }

                if let Some((ref msg, time)) = state.status_toast {
                    if time.elapsed().as_secs() < 5 {
                        ui.separator();
                        ui.label(egui::RichText::new(msg).strong().color(egui::Color32::from_rgb(226, 179, 87)));
                    }
                }
            });
        });

        // Animation Timeline Panel (Maya / Blender Style)

}

pub fn draw_timeline(state: &mut AppState, actions: &mut UiActions) {
    if state.timeline_expanded && state.current_workspace == WorkspaceTemplate::Animation {
        let timeline_height = match state.timeline_mode {
            TimelineViewMode::DopeSheet => 68.0,
            TimelineViewMode::FCurve => 210.0,
        };
        let egui_ctx = state.egui_ctx.clone();
        egui::TopBottomPanel::bottom("timeline_panel")
            .resizable(false)
            .default_height(timeline_height)
            .show(&egui_ctx, |ui| {
                draw_timeline_inner(state, actions, ui);
            });
    }
}

pub fn draw_timeline_inner(state: &mut AppState, _actions: &mut UiActions, ui: &mut egui::Ui) {
                    ui.horizontal(|ui| {
                        ui.selectable_value(&mut state.timeline_mode, TimelineViewMode::DopeSheet, "Dope Sheet");
                        ui.selectable_value(&mut state.timeline_mode, TimelineViewMode::FCurve, "F-Curve Editor");
                        ui.separator();

                        let play_text = if state.animation.is_playing {
                            egui::RichText::new("Pause (Space)").strong().color(egui::Color32::from_rgb(255, 180, 50))
                        } else {
                            egui::RichText::new("Play (Space)").strong().color(egui::Color32::from_rgb(100, 220, 100))
                        };
                        if ui.button(play_text).clicked() {
                            state.animation.is_playing = !state.animation.is_playing;
                        }
                        if ui.button("|< Start").clicked() {
                            state.animation.scrub_to_frame(state.animation.start_frame, &mut state.scene);
                        }
                        if ui.button(">| End").clicked() {
                            state.animation.scrub_to_frame(state.animation.end_frame, &mut state.scene);
                        }
                        ui.checkbox(&mut state.animation.loop_playback, "Loop");

                        ui.separator();
                        let key_btn = ui.button(egui::RichText::new("Key (I)").strong().color(egui::Color32::from_rgb(255, 215, 0)));
                        if key_btn.clicked() {
                            if let Some(sel_id) = state.selected_node {
                                if let Some(node) = state.scene.nodes.get(sel_id) {
                                    let frame = state.animation.current_frame.round();
                                    state.animation.insert_keyframe(sel_id, frame, node.local_transform, InterpolationMode::Linear);
                                }
                            }
                        }

                        ui.separator();
                        ui.label(egui::RichText::new(format!("Frame: {:03.0}", state.animation.current_frame)).strong());
                        ui.add(egui::DragValue::new(&mut state.animation.fps).prefix("FPS: ").range(1.0..=120.0));
                        ui.add(egui::DragValue::new(&mut state.animation.start_frame).prefix("Start: "));
                        ui.add(egui::DragValue::new(&mut state.animation.end_frame).prefix("End: "));
                    });

                    match state.timeline_mode {
                        TimelineViewMode::DopeSheet => {
                            // Scrubber slider with keyframe diamond indicators
                            let total_frames = (state.animation.end_frame - state.animation.start_frame).max(1.0);
                            let mut current_frame = state.animation.current_frame;
                            let slider_resp = ui.add(
                                egui::Slider::new(&mut current_frame, state.animation.start_frame..=state.animation.end_frame)
                                    .show_value(false)
                                    .step_by(1.0),
                            );
                            if slider_resp.changed() {
                                state.animation.scrub_to_frame(current_frame, &mut state.scene);
                                let total = (state.animation.end_frame - state.animation.start_frame).max(1.0);
                                let u = ((current_frame - state.animation.start_frame) / total).rem_euclid(1.0);
                                for c in &mut state.constraints.constraints {
                                    if let Constraint::Path(p) = c {
                                        p.u = u;
                                    }
                                }
                                state.constraints.evaluate(&mut state.scene);
                            }

                            if let Some(sel_id) = state.selected_node {
                                if let Some(track) = state.animation.tracks.get(&sel_id) {
                                    let frames = track.keyframe_frames();
                                    let rect = slider_resp.rect;
                                    let painter = ui.painter();
                                    for f in frames {
                                        let norm = (f - state.animation.start_frame) / total_frames;
                                        if (0.0..=1.0).contains(&norm) {
                                            let x = rect.left() + norm * rect.width();
                                            let y = rect.center().y;
                                            let size = 4.5_f32;
                                            let diamond = vec![
                                                egui::pos2(x, y - size),
                                                egui::pos2(x + size, y),
                                                egui::pos2(x, y + size),
                                                egui::pos2(x - size, y),
                                            ];
                                            painter.add(egui::epaint::PathShape::convex_polygon(
                                                diamond,
                                                egui::Color32::from_rgb(255, 215, 0),
                                                egui::Stroke::new(1.0_f32, egui::Color32::BLACK),
                                            ));
                                        }
                                    }
                                }
                            }
                        }
                        TimelineViewMode::FCurve => {
                            // Channel filters row:
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("Channels:").strong());
                                ui.checkbox(&mut state.fcurve_show_tx, egui::RichText::new("Loc X").color(egui::Color32::from_rgb(255, 80, 80)));
                                ui.checkbox(&mut state.fcurve_show_ty, egui::RichText::new("Loc Y").color(egui::Color32::from_rgb(80, 255, 80)));
                                ui.checkbox(&mut state.fcurve_show_tz, egui::RichText::new("Loc Z").color(egui::Color32::from_rgb(80, 160, 255)));
                                ui.checkbox(&mut state.fcurve_show_sx, egui::RichText::new("Scale X").color(egui::Color32::from_rgb(255, 180, 60)));
                                ui.checkbox(&mut state.fcurve_show_sy, egui::RichText::new("Scale Y").color(egui::Color32::from_rgb(180, 255, 60)));
                                ui.checkbox(&mut state.fcurve_show_sz, egui::RichText::new("Scale Z").color(egui::Color32::from_rgb(60, 240, 220)));
                                ui.checkbox(&mut state.fcurve_show_rot, egui::RichText::new("Rot").color(egui::Color32::from_rgb(220, 120, 255)));
                                ui.separator();
                                ui.label("Range:");
                                ui.add(egui::DragValue::new(&mut state.fcurve_val_min).speed(0.2).prefix("Min: "));
                                ui.add(egui::DragValue::new(&mut state.fcurve_val_max).speed(0.2).prefix("Max: "));
                                if ui.small_button("Auto-Fit").on_hover_text("Auto-fit vertical value range to keyframes").clicked() {
                                    if let Some(sel_id) = state.selected_node {
                                        if let Some(track) = state.animation.tracks.get(&sel_id) {
                                            let mut min_v = f32::INFINITY;
                                            let mut max_v = f32::NEG_INFINITY;
                                            for k in &track.translation.keyframes {
                                                min_v = min_v.min(k.value.x).min(k.value.y).min(k.value.z);
                                                max_v = max_v.max(k.value.x).max(k.value.y).max(k.value.z);
                                            }
                                            for k in &track.scale.keyframes {
                                                min_v = min_v.min(k.value.x).min(k.value.y).min(k.value.z);
                                                max_v = max_v.max(k.value.x).max(k.value.y).max(k.value.z);
                                            }
                                            if min_v.is_finite() && max_v.is_finite() {
                                                let pad = ((max_v - min_v) * 0.2).max(1.0);
                                                state.fcurve_val_min = min_v - pad;
                                                state.fcurve_val_max = max_v + pad;
                                            }
                                        }
                                    }
                                }
                            });

                            let (resp, painter) = ui.allocate_painter(
                                egui::Vec2::new(ui.available_width(), 130.0),
                                egui::Sense::click_and_drag(),
                            );
                            let rect = resp.rect;
                            painter.rect_filled(rect, 4.0, egui::Color32::from_rgb(22, 24, 28));
                            painter.rect_stroke(rect, 4.0, egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(55, 60, 70)));

                            let anim_start_f = state.animation.start_frame;
                            let anim_end_f = state.animation.end_frame;
                            let anim_cur_f = state.animation.current_frame;
                            let val_min = state.fcurve_val_min;
                            let val_max = state.fcurve_val_max;
                            let total_f = (anim_end_f - anim_start_f).max(1.0_f32);
                            let val_span = (val_max - val_min).max(0.1_f32);

                            let frame_to_x = move |f: f32| -> f32 {
                                rect.left() + ((f - anim_start_f) / total_f).clamp(0.0_f32, 1.0_f32) * rect.width()
                            };
                            let val_to_y = move |v: f32| -> f32 {
                                rect.bottom() - ((v - val_min) / val_span).clamp(0.0_f32, 1.0_f32) * rect.height()
                            };

                            // Draw horizontal zero line
                            if (val_min..=val_max).contains(&0.0) {
                                let y0 = val_to_y(0.0);
                                painter.line_segment(
                                    [egui::pos2(rect.left(), y0), egui::pos2(rect.right(), y0)],
                                    egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(90, 95, 110)),
                                );
                            }

                            // Draw vertical frame divisions (every 10 frames)
                            let mut step = 10.0;
                            if total_f > 150.0 { step = 25.0; }
                            let first_grid = (anim_start_f / step).ceil() * step;
                            let mut gf = first_grid;
                            while gf <= anim_end_f {
                                let gx = frame_to_x(gf);
                                painter.line_segment(
                                    [egui::pos2(gx, rect.top()), egui::pos2(gx, rect.bottom())],
                                    egui::Stroke::new(0.5_f32, egui::Color32::from_rgb(45, 50, 60)),
                                );
                                painter.text(
                                    egui::pos2(gx + 2.0, rect.bottom() - 12.0),
                                    egui::Align2::LEFT_BOTTOM,
                                    format!("{:.0}", gf),
                                    egui::FontId::monospace(9.0),
                                    egui::Color32::from_rgb(110, 115, 130),
                                );
                                gf += step;
                            }

                            // Handle click / drag scrub on graph
                            let mut scrubbed_f = None;
                            if resp.dragged() || resp.clicked() {
                                if let Some(mpos) = resp.interact_pointer_pos() {
                                    let norm = ((mpos.x - rect.left()) / rect.width()).clamp(0.0_f32, 1.0_f32);
                                    scrubbed_f = Some((anim_start_f + norm * total_f).round());
                                }
                            }

                            // Plot curves for selected node
                            if let Some(sel_id) = state.selected_node {
                                if let Some(track) = state.animation.tracks.get(&sel_id) {
                                    let samples = 80;
                                    let plot_channel = |keyframes: &[graph_engine::Keyframe<Vec3>], get_val: fn(Vec3) -> f32, color: egui::Color32| {
                                        if keyframes.is_empty() { return; }
                                        let anim_track = graph_engine::AnimationTrack { keyframes: keyframes.to_vec() };
                                        let mut prev_pt: Option<egui::Pos2> = None;
                                        for s in 0..=samples {
                                            let f = anim_start_f + (s as f32 / samples as f32) * total_f;
                                            if let Some(val3) = anim_track.evaluate(f) {
                                                let y_val = get_val(val3);
                                                let pt = egui::pos2(frame_to_x(f), val_to_y(y_val));
                                                if let Some(p0) = prev_pt {
                                                    painter.line_segment([p0, pt], egui::Stroke::new(1.8_f32, color));
                                                }
                                                prev_pt = Some(pt);
                                            }
                                        }
                                        // Draw keyframe diamonds
                                        for k in keyframes {
                                            let k_val = get_val(k.value);
                                            let pt = egui::pos2(frame_to_x(k.frame), val_to_y(k_val));
                                            painter.circle_filled(pt, 3.5, color);
                                            painter.circle_stroke(pt, 3.5, egui::Stroke::new(1.0_f32, egui::Color32::BLACK));
                                        }
                                    };

                                    if state.fcurve_show_tx {
                                        plot_channel(&track.translation.keyframes, |v| v.x, egui::Color32::from_rgb(255, 80, 80));
                                    }
                                    if state.fcurve_show_ty {
                                        plot_channel(&track.translation.keyframes, |v| v.y, egui::Color32::from_rgb(80, 255, 80));
                                    }
                                    if state.fcurve_show_tz {
                                        plot_channel(&track.translation.keyframes, |v| v.z, egui::Color32::from_rgb(80, 160, 255));
                                    }
                                    if state.fcurve_show_sx {
                                        plot_channel(&track.scale.keyframes, |v| v.x, egui::Color32::from_rgb(255, 180, 60));
                                    }
                                    if state.fcurve_show_sy {
                                        plot_channel(&track.scale.keyframes, |v| v.y, egui::Color32::from_rgb(180, 255, 60));
                                    }
                                    if state.fcurve_show_sz {
                                        plot_channel(&track.scale.keyframes, |v| v.z, egui::Color32::from_rgb(60, 240, 220));
                                    }
                                    if state.fcurve_show_rot {
                                        let keyframes = &track.rotation.keyframes;
                                        if !keyframes.is_empty() {
                                            let anim_track = graph_engine::AnimationTrack { keyframes: keyframes.to_vec() };
                                            let mut prev_pt: Option<egui::Pos2> = None;
                                            let rot_color = egui::Color32::from_rgb(220, 120, 255);
                                            for s in 0..=samples {
                                                let f = anim_start_f + (s as f32 / samples as f32) * total_f;
                                                if let Some(rot) = anim_track.evaluate(f) {
                                                    let angle_deg = rot.to_axis_angle().1.to_degrees();
                                                    let pt = egui::pos2(frame_to_x(f), val_to_y(angle_deg * 0.1));
                                                    if let Some(p0) = prev_pt {
                                                        painter.line_segment([p0, pt], egui::Stroke::new(1.8_f32, rot_color));
                                                    }
                                                    prev_pt = Some(pt);
                                                }
                                            }
                                            for k in keyframes {
                                                let angle_deg = k.value.to_axis_angle().1.to_degrees();
                                                let pt = egui::pos2(frame_to_x(k.frame), val_to_y(angle_deg * 0.1));
                                                painter.circle_filled(pt, 3.5, rot_color);
                                                painter.circle_stroke(pt, 3.5, egui::Stroke::new(1.0_f32, egui::Color32::BLACK));
                                            }
                                        }
                                    }
                                }
                            }

                            // Current playhead vertical line
                            let cur_x = frame_to_x(anim_cur_f);
                            painter.line_segment(
                                [egui::pos2(cur_x, rect.top()), egui::pos2(cur_x, rect.bottom())],
                                egui::Stroke::new(2.0_f32, egui::Color32::from_rgb(255, 175, 0)),
                            );

                            if let Some(f) = scrubbed_f {
                                state.animation.scrub_to_frame(f, &mut state.scene);
                            }
                        }
                    }
}

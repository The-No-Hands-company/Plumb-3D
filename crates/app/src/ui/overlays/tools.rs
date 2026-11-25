use core_math::Vec3;
use geo_kernel::Mesh;

use crate::state::AppState;
use crate::types::*;
use crate::ui::workspaces::WorkspaceTemplate;

pub fn render(state: &mut AppState) {
    // 1. Render Marquee / Box Selection Overlay
    if let (Some(start), Some(curr)) = (state.box_select_start, state.box_select_current) {
        let min_x = start.0.min(curr.0);
        let max_x = start.0.max(curr.0);
        let min_y = start.1.min(curr.1);
        let max_y = start.1.max(curr.1);
        let painter = state.egui_ctx.layer_painter(egui::LayerId::new(
            egui::Order::Foreground,
            egui::Id::new("marquee_overlay"),
        ));
        painter.rect(
            egui::Rect::from_min_max(egui::pos2(min_x, min_y), egui::pos2(max_x, max_y)),
            0.0,
            egui::Color32::from_rgba_premultiplied(40, 130, 255, 30),
            egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(80, 170, 255)),
        );
    }

    // 2. Render Sculpt / Weight Paint Brush Circular Overlay
    if state.interaction_mode == InteractionMode::Sculpt
        || state.interaction_mode == InteractionMode::WeightPaint
        || state.interaction_mode == InteractionMode::VertexPaint
    {
        if let Some((hit_pos, _)) = state.last_sculpt_hit {
            let width = state.surface_config.width as f32;
            let height = state.surface_config.height as f32;
            let radius = match state.interaction_mode {
                InteractionMode::Sculpt => state.sculpt_settings.radius,
                InteractionMode::WeightPaint => state.weight_brush_radius,
                InteractionMode::VertexPaint => state.vertex_paint_radius,
                _ => 0.5,
            };
            let stroke_color = match state.interaction_mode {
                InteractionMode::Sculpt => egui::Color32::from_rgb(255, 120, 60),
                InteractionMode::WeightPaint => {
                    let c = Mesh::weight_to_heatmap_rgb(state.weight_brush_target);
                    egui::Color32::from_rgba_premultiplied(
                        (c[0] * 255.0) as u8,
                        (c[1] * 255.0) as u8,
                        (c[2] * 255.0) as u8,
                        255,
                    )
                }
                InteractionMode::VertexPaint => {
                    egui::Color32::from_rgba_premultiplied(
                        (state.vertex_paint_color[0] * 255.0) as u8,
                        (state.vertex_paint_color[1] * 255.0) as u8,
                        (state.vertex_paint_color[2] * 255.0) as u8,
                        (state.vertex_paint_color[3] * 255.0) as u8,
                    )
                }
                _ => egui::Color32::WHITE,
            };

            if let Some(center_screen) = state.camera.world_to_screen(hit_pos, width, height) {
                let forward = (state.camera.target - state.camera.eye_position()).normalize();
                let right = forward.cross(Vec3::Y).normalize();
                let rim_world = hit_pos + right * radius;
                if let Some(rim_screen) = state.camera.world_to_screen(rim_world, width, height) {
                    let radius_screen = (rim_screen - center_screen).length();
                    let painter = state.egui_ctx.layer_painter(egui::LayerId::new(
                        egui::Order::Foreground,
                        egui::Id::new("brush_overlay"),
                    ));
                    painter.circle_stroke(
                        egui::pos2(center_screen.x, center_screen.y),
                        radius_screen,
                        egui::Stroke::new(1.5_f32, stroke_color),
                    );

                    if state.interaction_mode == InteractionMode::Sculpt {
                        let mut sym_points = Vec::new();
                        if state.sculpt_symmetry_x { sym_points.push(Vec3::new(-hit_pos.x, hit_pos.y, hit_pos.z)); }
                        if state.sculpt_symmetry_y { sym_points.push(Vec3::new(hit_pos.x, -hit_pos.y, hit_pos.z)); }
                        if state.sculpt_symmetry_z { sym_points.push(Vec3::new(hit_pos.x, hit_pos.y, -hit_pos.z)); }

                        let sym_stroke = egui::Stroke::new(1.2_f32, egui::Color32::from_rgb(255, 180, 80));
                        for p in sym_points {
                            if let Some(s_center) = state.camera.world_to_screen(p, width, height) {
                                painter.circle_stroke(egui::pos2(s_center.x, s_center.y), radius_screen, sym_stroke);
                                painter.circle_filled(egui::pos2(s_center.x, s_center.y), 2.0_f32, egui::Color32::from_rgb(255, 180, 80));
                            }
                        }
                    }
                }
            }
        }
    }

    // 3. Render 3D Measurement Overlay (Maya Distance / Blender Measure)
    if state.measure_mode || !state.measure_points.is_empty() {
        let width = state.surface_config.width as f32;
        let height = state.surface_config.height as f32;
        let painter = state.egui_ctx.layer_painter(egui::LayerId::new(
            egui::Order::Foreground,
            egui::Id::new("measurement_overlay"),
        ));

        for i in 0..state.measure_points.len() {
            let p_curr = state.measure_points[i];
            if let Some(s_curr) = state.camera.world_to_screen(p_curr, width, height) {
                painter.circle_filled(egui::pos2(s_curr.x, s_curr.y), 5.0, egui::Color32::from_rgb(255, 215, 0));
                painter.circle_stroke(egui::pos2(s_curr.x, s_curr.y), 5.0, egui::Stroke::new(1.0_f32, egui::Color32::BLACK));

                if i > 0 {
                    let p_prev = state.measure_points[i - 1];
                    if let Some(s_prev) = state.camera.world_to_screen(p_prev, width, height) {
                        let p0 = egui::pos2(s_prev.x, s_prev.y);
                        let p1 = egui::pos2(s_curr.x, s_curr.y);
                        painter.line_segment([p0, p1], egui::Stroke::new(2.5_f32, egui::Color32::from_rgb(255, 215, 0)));

                        let dist = (p_curr - p_prev).length();
                        let mid = egui::pos2((p0.x + p1.x) * 0.5, (p0.y + p1.y) * 0.5);
                        let label = format!("{:.3} m", dist);
                        painter.rect_filled(
                            egui::Rect::from_center_size(mid, egui::vec2(label.len() as f32 * 7.5 + 10.0, 18.0)),
                            3.0,
                            egui::Color32::from_black_alpha(190),
                        );
                        painter.text(
                            mid,
                            egui::Align2::CENTER_CENTER,
                            label,
                            egui::FontId::proportional(12.0),
                            egui::Color32::from_rgb(255, 255, 120),
                        );
                    }
                }

                if i >= 2 {
                    let p0 = state.measure_points[i - 2];
                    let p1 = state.measure_points[i - 1];
                    let p2 = state.measure_points[i];
                    let v10 = (p0 - p1).normalize_or_zero();
                    let v12 = (p2 - p1).normalize_or_zero();
                    let dot = v10.dot(v12).clamp(-1.0, 1.0);
                    let angle_deg = dot.acos() * 180.0 / std::f32::consts::PI;

                    if let Some(s1) = state.camera.world_to_screen(p1, width, height) {
                        let angle_pos = egui::pos2(s1.x, s1.y - 18.0);
                        let angle_label = format!("{:.1}°", angle_deg);
                        painter.rect_filled(
                            egui::Rect::from_center_size(angle_pos, egui::vec2(angle_label.len() as f32 * 7.5 + 8.0, 16.0)),
                            3.0,
                            egui::Color32::from_rgba_premultiplied(20, 60, 120, 210),
                        );
                        painter.text(
                            angle_pos,
                            egui::Align2::CENTER_CENTER,
                            angle_label,
                            egui::FontId::proportional(11.0),
                            egui::Color32::from_rgb(120, 210, 255),
                        );
                    }
                }
            }
        }

        // Measurement Tool floating panel
        egui::Window::new("📏 3D Measurement Tool (Ruler)")
            .default_pos(egui::pos2(width - 320.0, height - 260.0))
            .show(&state.egui_ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.checkbox(&mut state.measure_mode, "Measurement Active");
                    if ui.button("Clear Points").clicked() {
                        state.measure_points.clear();
                    }
                });
                ui.label("Click in viewport to place measurement anchors.");
                ui.separator();

                if state.measure_points.is_empty() {
                    ui.label(egui::RichText::new("No measurement points placed.").italics());
                } else {
                    ui.label(format!("Anchors placed: {}", state.measure_points.len()));
                    for i in 1..state.measure_points.len() {
                        let p0 = state.measure_points[i - 1];
                        let p1 = state.measure_points[i];
                        let d = (p1 - p0).length();
                        let dx = (p1.x - p0.x).abs();
                        let dy = (p1.y - p0.y).abs();
                        let dz = (p1.z - p0.z).abs();
                        ui.label(format!("P{} -> P{}: Dist: {:.3}m (ΔX: {:.2}, ΔY: {:.2}, ΔZ: {:.2})", i - 1, i, d, dx, dy, dz));
                    }
                }
            });
    }

    // 4. Render Precision CAD Snapping Glyph & HUD Pill Overlay
    if let Some((hit_pos, label)) = state.last_snap_hit {
        let width = state.surface_config.width as f32;
        let height = state.surface_config.height as f32;
        let painter = state.egui_ctx.layer_painter(egui::LayerId::new(
            egui::Order::Foreground,
            egui::Id::new("snap_overlay"),
        ));
        if let Some(s_pos) = state.camera.world_to_screen(hit_pos, width, height) {
            let center = egui::pos2(s_pos.x, s_pos.y);
            let (target_color, bg_glow) = match label {
                "Vertex" => (egui::Color32::from_rgb(0, 240, 255), egui::Color32::from_rgba_premultiplied(0, 180, 255, 40)),
                "Edge Midpoint" => (egui::Color32::from_rgb(50, 255, 120), egui::Color32::from_rgba_premultiplied(50, 255, 120, 40)),
                "Face Center" => (egui::Color32::from_rgb(255, 200, 50), egui::Color32::from_rgba_premultiplied(255, 200, 50, 40)),
                "Angle" => (egui::Color32::from_rgb(255, 100, 220), egui::Color32::from_rgba_premultiplied(255, 100, 220, 40)),
                _ => (egui::Color32::from_rgb(100, 160, 255), egui::Color32::from_rgba_premultiplied(100, 160, 255, 40)),
            };

            // Magnetic outer glow & ring
            painter.circle_filled(center, 12.0_f32, bg_glow);
            painter.circle_stroke(center, 10.0_f32, egui::Stroke::new(1.5_f32, target_color));
            painter.circle_filled(center, 2.5_f32, target_color);

            // Precision crosshairs
            painter.line_segment([egui::pos2(center.x - 16.0, center.y), egui::pos2(center.x - 4.0, center.y)], egui::Stroke::new(1.5_f32, target_color));
            painter.line_segment([egui::pos2(center.x + 4.0, center.y), egui::pos2(center.x + 16.0, center.y)], egui::Stroke::new(1.5_f32, target_color));
            painter.line_segment([egui::pos2(center.x, center.y - 16.0), egui::pos2(center.x, center.y - 4.0)], egui::Stroke::new(1.5_f32, target_color));
            painter.line_segment([egui::pos2(center.x, center.y + 4.0), egui::pos2(center.x, center.y + 16.0)], egui::Stroke::new(1.5_f32, target_color));

            // Snap guide line to current gizmo pivot
            if let Some(pivot) = state.get_gizmo_world_position() {
                if let Some(pivot_s) = state.camera.world_to_screen(pivot, width, height) {
                    painter.line_segment(
                        [egui::pos2(pivot_s.x, pivot_s.y), center],
                        egui::Stroke::new(1.0_f32, egui::Color32::from_white_alpha(100)),
                    );
                }
            }

            let pill_text = if label == "Angle" {
                let snap_step = if state.ctrl_pressed && state.shift_pressed {
                    90.0
                } else if state.shift_pressed {
                    45.0
                } else if state.ctrl_pressed {
                    15.0
                } else {
                    state.angle_snap_degrees
                };
                format!("📐 Angle Snapped ({:.0}° step)", snap_step)
            } else {
                format!("⌖ Snapped to {} ({:.2}, {:.2}, {:.2})", label, hit_pos.x, hit_pos.y, hit_pos.z)
            };
            let pill_pos = egui::pos2(center.x, center.y - 24.0);
            let pill_rect = egui::Rect::from_center_size(pill_pos, egui::vec2(pill_text.len() as f32 * 6.8 + 14.0, 20.0));
            painter.rect_filled(pill_rect, 4.0, egui::Color32::from_black_alpha(225));
            painter.rect_stroke(pill_rect, 4.0, egui::Stroke::new(1.0_f32, target_color));
            painter.text(
                pill_pos,
                egui::Align2::CENTER_CENTER,
                pill_text,
                egui::FontId::proportional(11.5),
                target_color,
            );
        }
    }

    // 5. Render Interactive Curve Pen Tool Overlay
    if state.curve_pen_active {
        let width = state.surface_config.width as f32;
        let height = state.surface_config.height as f32;
        let painter = state.egui_ctx.layer_painter(egui::LayerId::new(
            egui::Order::Foreground,
            egui::Id::new("curve_pen_overlay"),
        ));

        let pen_color = egui::Color32::from_rgb(0, 220, 255);
        let handle_color = egui::Color32::from_rgb(255, 200, 60);

        let mut screen_pts: Vec<egui::Pos2> = Vec::new();
        for pt in &state.curve_pen_points {
            if let Some(s) = state.camera.world_to_screen(pt.position, width, height) {
                let p = egui::pos2(s.x, s.y);
                screen_pts.push(p);

                // Anchor point dot
                painter.circle_filled(p, 4.5, pen_color);
                painter.circle_stroke(p, 6.0, egui::Stroke::new(1.0_f32, egui::Color32::WHITE));

                // Tangent handles if smooth
                if pt.handle_out.length_squared() > 1e-4 {
                    if let Some(h_out) = state.camera.world_to_screen(pt.position + pt.handle_out, width, height) {
                        let hp = egui::pos2(h_out.x, h_out.y);
                        painter.line_segment([p, hp], egui::Stroke::new(1.0_f32, handle_color));
                        painter.circle_filled(hp, 3.0, handle_color);
                    }
                    if let Some(h_in) = state.camera.world_to_screen(pt.position + pt.handle_in, width, height) {
                        let hp = egui::pos2(h_in.x, h_in.y);
                        painter.line_segment([p, hp], egui::Stroke::new(1.0_f32, handle_color));
                        painter.circle_filled(hp, 3.0, handle_color);
                    }
                }
            }
        }

        // Connect placed anchor points with curve line segments
        for i in 1..screen_pts.len() {
            painter.line_segment([screen_pts[i - 1], screen_pts[i]], egui::Stroke::new(2.0_f32, pen_color));
        }

        // Rubber-band line from last point to mouse cursor
        if let Some((mx, my)) = state.last_mouse_pos {
            let mouse_pos = egui::pos2(mx as f32, my as f32);
            if let Some(&last_p) = screen_pts.last() {
                painter.line_segment([last_p, mouse_pos], egui::Stroke::new(1.5_f32, egui::Color32::from_rgba_premultiplied(0, 220, 255, 160)));
            }

            // If mouse is near start point, draw closing ring
            if screen_pts.len() >= 2 {
                let start_p = screen_pts[0];
                let dist = (mouse_pos - start_p).length();
                if dist < 24.0 {
                    painter.circle_stroke(start_p, 12.0, egui::Stroke::new(2.0_f32, egui::Color32::YELLOW));
                    painter.text(
                        egui::pos2(start_p.x, start_p.y - 18.0),
                        egui::Align2::CENTER_CENTER,
                        "Close Loop",
                        egui::FontId::proportional(11.0),
                        egui::Color32::YELLOW,
                    );
                }
            }
        }

        // Curve Pen HUD banner
        let hud_text = "✒ Curve Pen: Click to place anchor | Drag for smooth tangent | C/Click Start to close | Enter to commit | Esc to cancel";
        let banner_rect = egui::Rect::from_center_size(
            egui::pos2(width * 0.5, 38.0),
            egui::vec2(hud_text.len() as f32 * 6.5 + 24.0, 26.0),
        );
        painter.rect_filled(banner_rect, 6.0, egui::Color32::from_black_alpha(220));
        painter.rect_stroke(banner_rect, 6.0, egui::Stroke::new(1.0_f32, pen_color));
        painter.text(
            banner_rect.center(),
            egui::Align2::CENTER_CENTER,
            hud_text,
            egui::FontId::proportional(12.0),
            egui::Color32::WHITE,
        );
    }

    // 6. Render Active Workplane Visual Indicator
    if state.workplane.is_active {
        let width = state.surface_config.width as f32;
        let height = state.surface_config.height as f32;
        let painter = state.egui_ctx.layer_painter(egui::LayerId::new(
            egui::Order::Foreground,
            egui::Id::new("workplane_overlay"),
        ));

        let o = state.workplane.origin;
        let u = o + state.workplane.tangent_u * 0.8;
        let v = o + state.workplane.tangent_v * 0.8;
        let n = o + state.workplane.normal * 0.8;

        if let (Some(so), Some(su), Some(sv), Some(sn)) = (
            state.camera.world_to_screen(o, width, height),
            state.camera.world_to_screen(u, width, height),
            state.camera.world_to_screen(v, width, height),
            state.camera.world_to_screen(n, width, height),
        ) {
            let po = egui::pos2(so.x, so.y);
            painter.line_segment([po, egui::pos2(su.x, su.y)], egui::Stroke::new(2.0_f32, egui::Color32::RED));
            painter.line_segment([po, egui::pos2(sv.x, sv.y)], egui::Stroke::new(2.0_f32, egui::Color32::GREEN));
            painter.line_segment([po, egui::pos2(sn.x, sn.y)], egui::Stroke::new(2.0_f32, egui::Color32::BLUE));

            painter.circle_filled(po, 3.5, egui::Color32::WHITE);
            let pill_pos = egui::pos2(po.x + 30.0, po.y);
            painter.text(pill_pos, egui::Align2::LEFT_CENTER, "📐 CAD Workplane", egui::FontId::proportional(11.0), egui::Color32::from_rgb(0, 220, 255));
        }
    }

    // 7. Render Interactive Cutter Modal Overlay & Caliper Callouts
    if let Some(cutter) = &state.cutter_modal {
        let width = state.surface_config.width as f32;
        let height = state.surface_config.height as f32;
        let painter = state.egui_ctx.layer_painter(egui::LayerId::new(
            egui::Order::Foreground,
            egui::Id::new("cutter_overlay"),
        ));

        let cutter_color = if cutter.is_ghost {
            egui::Color32::from_rgb(0, 230, 255) // Cyan for Live Ghost Cutter
        } else {
            egui::Color32::from_rgb(255, 140, 20) // Bright BoxCutter Orange
        };

        let min_u = cutter.start_uv.x.min(cutter.current_uv.x);
        let max_u = cutter.start_uv.x.max(cutter.current_uv.x);
        let min_v = cutter.start_uv.y.min(cutter.current_uv.y);
        let max_v = cutter.start_uv.y.max(cutter.current_uv.y);

        let p00 = cutter.workplane.uv_to_world(core_math::Vec2::new(min_u, min_v));
        let p10 = cutter.workplane.uv_to_world(core_math::Vec2::new(max_u, min_v));
        let p11 = cutter.workplane.uv_to_world(core_math::Vec2::new(max_u, max_v));
        let p01 = cutter.workplane.uv_to_world(core_math::Vec2::new(min_u, max_v));

        if let (Some(s00), Some(s10), Some(s11), Some(s01)) = (
            state.camera.world_to_screen(p00, width, height),
            state.camera.world_to_screen(p10, width, height),
            state.camera.world_to_screen(p11, width, height),
            state.camera.world_to_screen(p01, width, height),
        ) {
            let pts = [
                egui::pos2(s00.x, s00.y),
                egui::pos2(s10.x, s10.y),
                egui::pos2(s11.x, s11.y),
                egui::pos2(s01.x, s01.y),
            ];

            for i in 0..4 {
                painter.line_segment([pts[i], pts[(i + 1) % 4]], egui::Stroke::new(2.5_f32, cutter_color));
            }

            // Draw radial pattern preview if N > 1
            if cutter.radial_count > 1 {
                let n = cutter.radial_count;
                let v0 = cutter.current_uv - cutter.start_uv;
                for k in 1..n {
                    let theta = (k as f32 / n as f32) * std::f32::consts::TAU;
                    let cos_t = theta.cos();
                    let sin_t = theta.sin();
                    let rot_v = core_math::Vec2::new(v0.x * cos_t - v0.y * sin_t, v0.x * sin_t + v0.y * cos_t);
                    let inst_center = cutter.start_uv + rot_v;
                    let inst_p = cutter.workplane.uv_to_world(inst_center);
                    if let Some(sc) = state.camera.world_to_screen(inst_p, width, height) {
                        painter.circle_stroke(
                            egui::pos2(sc.x, sc.y),
                            8.0,
                            egui::Stroke::new(1.5_f32, cutter_color),
                        );
                    }
                }
            }

            if cutter.stage == crate::modals::cutter::CutterStage::ExtrudeDepth {
                let n = cutter.workplane.normal;
                let d = cutter.depth;
                let b00 = p00 - n * d;
                let b10 = p10 - n * d;
                let b11 = p11 - n * d;
                let b01 = p01 - n * d;

                if let (Some(sb00), Some(sb10), Some(sb11), Some(sb01)) = (
                    state.camera.world_to_screen(b00, width, height),
                    state.camera.world_to_screen(b10, width, height),
                    state.camera.world_to_screen(b11, width, height),
                    state.camera.world_to_screen(b01, width, height),
                ) {
                    let bpts = [
                        egui::pos2(sb00.x, sb00.y),
                        egui::pos2(sb10.x, sb10.y),
                        egui::pos2(sb11.x, sb11.y),
                        egui::pos2(sb01.x, sb01.y),
                    ];
                    for i in 0..4 {
                        painter.line_segment([bpts[i], bpts[(i + 1) % 4]], egui::Stroke::new(1.5_f32, cutter_color));
                        painter.line_segment([pts[i], bpts[i]], egui::Stroke::new(1.5_f32, cutter_color));
                    }
                }
            }

            let dim_w = ((max_u - min_u).abs() * 1000.0).max(1.0);
            let dim_h = ((max_v - min_v).abs() * 1000.0).max(1.0);
            let dim_d = (cutter.depth * 1000.0).max(1.0);
            let shape_label = match cutter.shape {
                crate::modals::cutter::CutterShape::Box => "Box",
                crate::modals::cutter::CutterShape::Circle => "Circle",
            };
            let array_label = if cutter.radial_count > 1 {
                format!(" | ⚙ N={}", cutter.radial_count)
            } else {
                String::new()
            };
            let ghost_label = if cutter.is_ghost { " | 👻 Ghost" } else { "" };
            let caliper_text = format!("📐 {} W: {:.1} mm H: {:.1} mm D: {:.1} mm{}{}", shape_label, dim_w, dim_h, dim_d, array_label, ghost_label);

            let center_screen = egui::pos2((pts[0].x + pts[2].x) * 0.5, (pts[0].y + pts[2].y) * 0.5 - 20.0);
            let pill_rect = egui::Rect::from_center_size(center_screen, egui::vec2(caliper_text.len() as f32 * 6.5 + 16.0, 22.0));
            painter.rect_filled(pill_rect, 4.0, egui::Color32::from_black_alpha(230));
            painter.rect_stroke(pill_rect, 4.0, egui::Stroke::new(1.0_f32, cutter_color));
            painter.text(center_screen, egui::Align2::CENTER_CENTER, caliper_text, egui::FontId::proportional(11.5), egui::Color32::WHITE);
        }
    }

    // 8. Left Vertical Viewport Tool Strip (Plasticity / Blender T-Shelf Style)
    if state.show_left_toolbar {
        let mut toggle_cutter = false;
        let mut toggle_push_pull = false;
        let left_offset = if state.show_outliner { 235.0 } else { 15.0 };
        egui::Area::new(egui::Id::new("left_viewport_tool_strip"))
            .fixed_pos(egui::pos2(left_offset, 75.0))
            .order(egui::Order::Middle)
            .show(&state.egui_ctx, |ui| {
                egui::Frame::popup(ui.style())
                    .fill(egui::Color32::from_rgba_premultiplied(22, 24, 29, 230))
                    .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_white_alpha(30)))
                    .rounding(egui::Rounding::same(8.0))
                    .inner_margin(egui::Margin::symmetric(5.0, 7.0))
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            if ui.selectable_label(state.is_box_selecting, "⬚").on_hover_text("Marquee Box Select (B)").clicked() {
                                state.is_box_selecting = !state.is_box_selecting;
                            }
                            if ui.selectable_label(state.gizmo.mode == render_viewport::GizmoMode::Translate, "✥").on_hover_text("Move Tool (W)").clicked() {
                                state.gizmo.mode = render_viewport::GizmoMode::Translate;
                            }
                            if ui.selectable_label(state.gizmo.mode == render_viewport::GizmoMode::Rotate, "⟲").on_hover_text("Rotate Tool (E)").clicked() {
                                state.gizmo.mode = render_viewport::GizmoMode::Rotate;
                            }
                            if ui.selectable_label(state.gizmo.mode == render_viewport::GizmoMode::Scale, "⤢").on_hover_text("Scale Tool (R)").clicked() {
                                state.gizmo.mode = render_viewport::GizmoMode::Scale;
                            }
                            ui.separator();
                            if ui.selectable_label(state.measure_mode, "📏").on_hover_text("Tape Measure Tool (M)").clicked() {
                                state.measure_mode = !state.measure_mode;
                            }
                            if ui.button("📐").on_hover_text("Align Objects (Alt+A)").clicked() {
                                state.show_align_dialog = !state.show_align_dialog;
                            }
                            if ui.selectable_label(state.cutter_modal.is_some(), "✂").on_hover_text("Interactive Direct Cutter (C)").clicked() {
                                toggle_cutter = true;
                            }
                            if ui.selectable_label(state.push_pull_tool, "⤒").on_hover_text("Direct Push-Pull (Shift+E / Click-Drag Face)").clicked() {
                                toggle_push_pull = true;
                            }
                            if state.current_workspace == WorkspaceTemplate::Sculpting || state.interaction_mode == InteractionMode::Sculpt {
                                if ui.selectable_label(state.interaction_mode == InteractionMode::Sculpt, "🖌").on_hover_text("Sculpt Brush").clicked() {
                                    state.interaction_mode = InteractionMode::Sculpt;
                                }
                            }
                            if ui.selectable_label(state.curve_pen_active, "✒").on_hover_text("Bézier Curve Pen Tool (P)").clicked() {
                                state.curve_pen_active = !state.curve_pen_active;
                                if state.curve_pen_active {
                                    state.curve_pen_points.clear();
                                    state.curve_pen_is_closed = false;
                                }
                            }
                            ui.separator();
                            if ui.selectable_label(state.show_node_editor, "🌿").on_hover_text("Geometry Nodes Canvas").clicked() {
                                state.show_node_editor = !state.show_node_editor;
                            }
                            if ui.selectable_label(state.show_uv_editor, "🗺").on_hover_text("2D UV Texture Editor").clicked() {
                                state.show_uv_editor = !state.show_uv_editor;
                            }
                            if ui.selectable_label(state.show_script_console, "📜").on_hover_text("Scripting Console (Rhai)").clicked() {
                                state.show_script_console = !state.show_script_console;
                            }
                        });
                    });
            });

        if toggle_cutter {
            if state.cutter_modal.is_some() {
                crate::modals::cutter::cancel(state);
            } else {
                crate::modals::cutter::start(state);
            }
        }
        if toggle_push_pull {
            state.push_pull_tool = !state.push_pull_tool;
            if state.push_pull_tool {
                state.set_status_message("Direct Push-Pull Active: Click and drag any face to extrude along normal");
            }
        }
    }
}

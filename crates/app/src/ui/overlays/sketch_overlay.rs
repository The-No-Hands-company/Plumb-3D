//! Viewport Overlay and Interactive Tooling for 2D CAD Constraint Sketcher.
//!
//! Provides top floating toolbar, workplane grid, entity rendering,
//! clickable live dimension callout badges, and mouse drafting interaction.

use core_math::{vec2, Vec2};
use geo_kernel::{
    extract_dimension_callouts, solve_sketch, ConstraintId, ConstraintType, Sketch, SketchEntity,
    SolverConfig, SolverStatus,
};
use graph_engine::{NodeData, NodeId};

use crate::commands::modeling::sketch::{
    AddSketchConstraintCommand, ExtrudeSketchCommand, SetDimensionValueCommand,
};
use crate::state::AppState;
use crate::types::SketchTool;

/// Main entry point for sketcher viewport rendering.
pub fn render(state: &mut AppState) {
    let sketch_node_id = find_active_sketch_node(state);

    // If sketch is not active and no sketch node is selected, skip rendering
    if !state.sketch_active && sketch_node_id.is_none() {
        return;
    }

    let node_id = match sketch_node_id {
        Some(id) => id,
        None => return,
    };

    render_sketch_toolbar(state, node_id);
    render_sketch_geometry_and_dimensions(state, node_id);
}

/// Finds the selected or first active sketch node in the scene.
pub fn find_active_sketch_node(state: &AppState) -> Option<NodeId> {
    if let Some(sel_id) = state.selected_node {
        if let Some(node) = state.scene.nodes.get(sel_id) {
            if matches!(node.data, NodeData::Sketch { .. }) {
                return Some(sel_id);
            }
        }
    }
    state.scene.nodes.iter().find_map(|(id, node)| {
        if matches!(node.data, NodeData::Sketch { .. }) {
            Some(id)
        } else {
            None
        }
    })
}

/// Renders the floating CAD Sketcher toolbar at the top of the viewport.
fn render_sketch_toolbar(state: &mut AppState, node_id: NodeId) {
    let ctx = state.egui_ctx.clone();

    egui::Window::new("cad_sketcher_toolbar")
        .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 48.0))
        .title_bar(false)
        .resizable(false)
        .collapsible(false)
        .frame(
            egui::Frame::window(&ctx.style())
                .fill(egui::Color32::from_rgba_premultiplied(18, 22, 30, 230))
                .stroke(egui::Stroke::new(
                    1.0_f32,
                    egui::Color32::from_rgb(226, 179, 87),
                ))
                .rounding(egui::Rounding::same(8.0))
                .inner_margin(egui::Margin::symmetric(12.0, 6.0)),
        )
        .show(&ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("📐 CAD SKETCHER")
                        .strong()
                        .color(egui::Color32::from_rgb(226, 179, 87)),
                );
                ui.separator();

                // Tool Selectors
                let tools = [
                    (SketchTool::Select, "✋ Select"),
                    (SketchTool::Line, "╱ Line"),
                    (SketchTool::Rectangle, "▭ Rect"),
                    (SketchTool::Circle, "○ Circle"),
                    (SketchTool::Dimension, "📏 Dim"),
                ];

                for (tool, label) in tools {
                    let is_active = state.sketch_tool == tool;
                    if ui.selectable_label(is_active, label).clicked() {
                        state.sketch_tool = tool;
                        state.sketch_in_progress_point = None;
                    }
                }

                ui.separator();

                // Quick Constraints Menu
                ui.menu_button("Constraints ▾", |ui| {
                    if ui.button("— Horizontal").clicked() {
                        add_constraint_to_active(
                            state,
                            node_id,
                            ConstraintType::Horizontal {
                                pt_a: geo_kernel::SketchPointId(0),
                                pt_b: geo_kernel::SketchPointId(1),
                            },
                        );
                        ui.close_menu();
                    }
                    if ui.button("| Vertical").clicked() {
                        add_constraint_to_active(
                            state,
                            node_id,
                            ConstraintType::Vertical {
                                pt_a: geo_kernel::SketchPointId(0),
                                pt_b: geo_kernel::SketchPointId(1),
                            },
                        );
                        ui.close_menu();
                    }
                    if ui.button("⛓ Coincident").clicked() {
                        add_constraint_to_active(
                            state,
                            node_id,
                            ConstraintType::Coincident {
                                pt_a: geo_kernel::SketchPointId(0),
                                pt_b: geo_kernel::SketchPointId(1),
                            },
                        );
                        ui.close_menu();
                    }
                });

                // Solve Button
                if ui.button("⚡ Solve").clicked() {
                    if let Some(node) = state.scene.nodes.get_mut(node_id) {
                        if let NodeData::Sketch {
                            sketch,
                            cached_render_mesh,
                        } = &mut node.data
                        {
                            let res = solve_sketch(sketch, SolverConfig::default());
                            *cached_render_mesh = None;
                            state.scene.mark_dirty(node_id);
                            match res {
                                SolverStatus::Solved {
                                    iterations,
                                    max_residual,
                                } => {
                                    state.set_status_message(format!(
                                        "✓ Sketch Solved in {} iter (residual {:.4} mm)",
                                        iterations, max_residual
                                    ));
                                }
                                SolverStatus::DidNotConverge { max_residual, .. } => {
                                    state.set_status_message(format!(
                                        "⚠ Inconsistent: residual {:.3} mm",
                                        max_residual
                                    ));
                                }
                                SolverStatus::Unconstrained => {
                                    state.set_status_message("Sketch is unconstrained");
                                }
                            }
                        }
                    }
                    state.window.request_redraw();
                }

                ui.separator();

                // Extrude Action
                ui.label("Depth:");
                ui.add(
                    egui::DragValue::new(&mut state.sketch_extrude_depth)
                        .speed(1.0)
                        .range(1.0..=500.0)
                        .suffix(" mm"),
                );

                if ui
                    .button(
                        egui::RichText::new("🧊 Extrude to Solid")
                            .color(egui::Color32::from_rgb(100, 220, 140))
                            .strong(),
                    )
                    .clicked()
                {
                    let cmd = Box::new(ExtrudeSketchCommand::new(
                        node_id,
                        state.sketch_extrude_depth,
                    ));
                    let _ = state.history.execute(cmd, &mut state.scene);
                    state.sketch_active = false;
                    state.set_status_message("Extruded Sketch to 2-Manifold BMesh Solid");
                    state.window.request_redraw();
                }

                if ui.button("✕ Exit").clicked() {
                    state.sketch_active = false;
                    state.sketch_in_progress_point = None;
                    state.window.request_redraw();
                }
            });
        });
}

fn add_constraint_to_active(state: &mut AppState, node_id: NodeId, c_type: ConstraintType) {
    let cmd = Box::new(AddSketchConstraintCommand::new(node_id, c_type));
    let _ = state.history.execute(cmd, &mut state.scene);
    state.window.request_redraw();
}

/// Renders projected sketch lines, points, and clickable live dimension badges.
fn render_sketch_geometry_and_dimensions(state: &mut AppState, node_id: NodeId) {
    let width = state.surface_config.width as f32;
    let height = state.surface_config.height as f32;
    let painter = state.egui_ctx.layer_painter(egui::LayerId::new(
        egui::Order::Foreground,
        egui::Id::new("cad_sketch_viewport_overlay"),
    ));

    let (sketch_clone, world_mat) = {
        let node = match state.scene.nodes.get(node_id) {
            Some(n) => n,
            None => return,
        };
        match &node.data {
            NodeData::Sketch { sketch, .. } => (sketch.clone(), node.world_matrix),
            _ => return,
        }
    };

    let wp = &sketch_clone.workplane;

    // 1. Draw Workplane Grid Center Lines
    let origin_world = world_mat.transform_point3(wp.origin);
    let u_end_world = world_mat.transform_point3(wp.to_3d(vec2(100.0, 0.0)));
    let v_end_world = world_mat.transform_point3(wp.to_3d(vec2(0.0, 100.0)));

    if let (Some(s_origin), Some(s_u), Some(s_v)) = (
        state.camera.world_to_screen(origin_world, width, height),
        state.camera.world_to_screen(u_end_world, width, height),
        state.camera.world_to_screen(v_end_world, width, height),
    ) {
        // Red U-Axis
        painter.line_segment(
            [egui::pos2(s_origin.x, s_origin.y), egui::pos2(s_u.x, s_u.y)],
            egui::Stroke::new(1.8_f32, egui::Color32::from_rgb(240, 80, 80)),
        );
        // Green V-Axis
        painter.line_segment(
            [egui::pos2(s_origin.x, s_origin.y), egui::pos2(s_v.x, s_v.y)],
            egui::Stroke::new(1.8_f32, egui::Color32::from_rgb(80, 220, 100)),
        );
    }

    // 2. Draw Sketch Lines & Curves
    for entity in &sketch_clone.entities {
        match entity {
            SketchEntity::Line(l) => {
                if let (Some(p1), Some(p2)) = (
                    sketch_clone.get_point(l.start),
                    sketch_clone.get_point(l.end),
                ) {
                    let w1 = world_mat.transform_point3(wp.to_3d(p1.pos));
                    let w2 = world_mat.transform_point3(wp.to_3d(p2.pos));
                    if let (Some(s1), Some(s2)) = (
                        state.camera.world_to_screen(w1, width, height),
                        state.camera.world_to_screen(w2, width, height),
                    ) {
                        let color = if l.is_construction {
                            egui::Color32::from_rgb(120, 130, 150)
                        } else {
                            egui::Color32::from_rgb(70, 180, 255)
                        };
                        painter.line_segment(
                            [egui::pos2(s1.x, s1.y), egui::pos2(s2.x, s2.y)],
                            egui::Stroke::new(2.4_f32, color),
                        );
                    }
                }
            }
            SketchEntity::Circle(c) => {
                if let Some(cp) = sketch_clone.get_point(c.center) {
                    let mut prev_screen: Option<Vec2> = None;
                    let segs = 32;
                    for i in 0..=segs {
                        let theta = (i as f32 / segs as f32) * std::f32::consts::TAU;
                        let uv = cp.pos + vec2(theta.cos(), theta.sin()) * c.radius;
                        let w = world_mat.transform_point3(wp.to_3d(uv));
                        if let Some(s) = state.camera.world_to_screen(w, width, height) {
                            if let Some(ps) = prev_screen {
                                painter.line_segment(
                                    [egui::pos2(ps.x, ps.y), egui::pos2(s.x, s.y)],
                                    egui::Stroke::new(
                                        2.2_f32,
                                        egui::Color32::from_rgb(70, 180, 255),
                                    ),
                                );
                            }
                            prev_screen = Some(s);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    // 3. Draw Points
    for pt in &sketch_clone.points {
        let w = world_mat.transform_point3(wp.to_3d(pt.pos));
        if let Some(s) = state.camera.world_to_screen(w, width, height) {
            let color = if pt.is_fixed {
                egui::Color32::from_rgb(226, 179, 87) // Gold
            } else {
                egui::Color32::from_rgb(80, 200, 255) // Cyan
            };
            painter.circle_filled(egui::pos2(s.x, s.y), 4.5, color);
            painter.circle_stroke(
                egui::pos2(s.x, s.y),
                4.5,
                egui::Stroke::new(1.0_f32, egui::Color32::BLACK),
            );
        }
    }

    // 4. Draw In-Progress Rubberband Line
    if let Some(p_start_uv) = state.sketch_in_progress_point {
        let w1 = world_mat.transform_point3(wp.to_3d(p_start_uv));
        let mouse_uv = get_mouse_workplane_uv(state, wp);
        let w2 = world_mat.transform_point3(wp.to_3d(mouse_uv));
        if let (Some(s1), Some(s2)) = (
            state.camera.world_to_screen(w1, width, height),
            state.camera.world_to_screen(w2, width, height),
        ) {
            painter.line_segment(
                [egui::pos2(s1.x, s1.y), egui::pos2(s2.x, s2.y)],
                egui::Stroke::new(1.8_f32, egui::Color32::from_rgb(255, 215, 0)),
            );
        }
    }

    // 5. Draw Live Dimension Callout Badges with Click-to-Edit Popovers
    render_dimension_callouts(state, node_id, &sketch_clone, world_mat, width, height);
}

/// Renders witness lines, dimension lines, and interactive click-to-edit badges for live dimensions.
fn render_dimension_callouts(
    state: &mut AppState,
    node_id: NodeId,
    sketch: &Sketch,
    world_mat: core_math::Mat4,
    width: f32,
    height: f32,
) {
    let callouts = extract_dimension_callouts(sketch, 16.0);
    let ctx = state.egui_ctx.clone();
    let painter = ctx.layer_painter(egui::LayerId::new(
        egui::Order::Foreground,
        egui::Id::new("cad_dimension_badges"),
    ));

    for callout in callouts {
        let wp = &sketch.workplane;
        let w_p1 = world_mat.transform_point3(wp.to_3d(callout.p1));
        let w_p2 = world_mat.transform_point3(wp.to_3d(callout.p2));
        let w_d1 = world_mat.transform_point3(wp.to_3d(callout.dim_start));
        let w_d2 = world_mat.transform_point3(wp.to_3d(callout.dim_end));
        let w_txt = world_mat.transform_point3(wp.to_3d(callout.text_pos));

        if let (Some(s_p1), Some(s_p2), Some(s_d1), Some(s_d2), Some(s_txt)) = (
            state.camera.world_to_screen(w_p1, width, height),
            state.camera.world_to_screen(w_p2, width, height),
            state.camera.world_to_screen(w_d1, width, height),
            state.camera.world_to_screen(w_d2, width, height),
            state.camera.world_to_screen(w_txt, width, height),
        ) {
            // Witness extension lines
            let witness_stroke = egui::Stroke::new(
                1.0_f32,
                egui::Color32::from_rgba_premultiplied(160, 175, 200, 140),
            );
            painter.line_segment(
                [egui::pos2(s_p1.x, s_p1.y), egui::pos2(s_d1.x, s_d1.y)],
                witness_stroke,
            );
            painter.line_segment(
                [egui::pos2(s_p2.x, s_p2.y), egui::pos2(s_d2.x, s_d2.y)],
                witness_stroke,
            );

            // Dimension line
            let dim_stroke = egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(226, 179, 87));
            painter.line_segment(
                [egui::pos2(s_d1.x, s_d1.y), egui::pos2(s_d2.x, s_d2.y)],
                dim_stroke,
            );

            // Clickable Badge Pill
            let badge_pos = egui::pos2(s_txt.x, s_txt.y);
            let badge_rect = egui::Rect::from_center_size(badge_pos, egui::vec2(72.0, 22.0));

            let is_editing = if let Some((cid, _)) = &state.editing_dimension {
                callout.constraint_id == Some(cid.0)
            } else {
                false
            };

            if is_editing {
                // Interactive text input popover
                let mut commit_val = None;
                if let Some((_, ref mut text_buf)) = state.editing_dimension {
                    egui::Area::new(egui::Id::new(format!(
                        "dim_edit_{:?}",
                        callout.constraint_id
                    )))
                    .fixed_pos(badge_rect.min)
                    .show(&ctx, |ui| {
                        let response = ui.add(
                            egui::TextEdit::singleline(text_buf)
                                .desired_width(70.0)
                                .font(egui::TextStyle::Monospace),
                        );
                        if response.lost_focus() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            if let Ok(val) = text_buf.parse::<f32>() {
                                commit_val = Some(val);
                            }
                        }
                    });
                }
                if let Some(new_val) = commit_val {
                    if let Some(cid_num) = callout.constraint_id {
                        let cmd = Box::new(SetDimensionValueCommand::new(
                            node_id,
                            ConstraintId(cid_num),
                            new_val,
                        ));
                        let _ = state.history.execute(cmd, &mut state.scene);
                    }
                    state.editing_dimension = None;
                    state.window.request_redraw();
                }
            } else {
                // Render badge background and label
                painter.rect_filled(
                    badge_rect,
                    4.0,
                    egui::Color32::from_rgba_premultiplied(20, 24, 32, 220),
                );
                painter.rect_stroke(
                    badge_rect,
                    4.0,
                    egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(226, 179, 87)),
                );
                painter.text(
                    badge_pos,
                    egui::Align2::CENTER_CENTER,
                    &callout.label,
                    egui::FontId::monospace(11.0),
                    egui::Color32::WHITE,
                );

                // Check click on badge to begin editing
                if let Some(cid_num) = callout.constraint_id {
                    let mouse_pos = ctx.input(|i| i.pointer.hover_pos());
                    let clicked = ctx.input(|i| i.pointer.primary_clicked());
                    if clicked {
                        if let Some(m) = mouse_pos {
                            if badge_rect.contains(m) {
                                state.editing_dimension =
                                    Some((ConstraintId(cid_num), format!("{:.2}", callout.value)));
                                state.window.request_redraw();
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Helper to project mouse cursor onto the 2D workplane coordinates $(u, v)$.
fn get_mouse_workplane_uv(state: &AppState, wp: &core_math::Workplane) -> Vec2 {
    let mouse_pos = state
        .egui_ctx
        .input(|i| i.pointer.hover_pos())
        .unwrap_or(egui::pos2(0.0, 0.0));
    let ray = state.get_screen_ray(mouse_pos.x as f64, mouse_pos.y as f64);
    wp.intersect_ray_uv(&ray).unwrap_or(Vec2::ZERO)
}

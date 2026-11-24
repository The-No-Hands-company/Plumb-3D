use crate::commands::{BevelEdgeCommand, ExtrudeCommand, InsetCommand, LoopCutCommand};
use crate::state::AppState;
use crate::types::LastOperatorInfo;

pub fn render(state: &mut AppState) {
    if state.last_operator.is_none() {
        return;
    }

    let ctx = state.egui_ctx.clone();
    let height = state.surface_config.height as f32;
    let panel_x = if state.show_outliner { 250.0 } else { 20.0 };
    let panel_y = height - 70.0;

    let op = state.last_operator.as_ref().unwrap().clone();
    let op_title = match &op {
        LastOperatorInfo::Inset { .. } => "Inset Faces",
        LastOperatorInfo::Extrude { .. } => "Extrude Faces",
        LastOperatorInfo::Bevel { .. } => "Bevel Edges",
        LastOperatorInfo::LoopCut { .. } => "Loop Cut",
    };

    if !state.show_last_operator_panel {
        egui::Area::new(egui::Id::new("collapsed_operator_panel"))
            .fixed_pos(egui::pos2(panel_x, panel_y))
            .order(egui::Order::Middle)
            .show(&ctx, |ui| {
                if ui.button(format!("⚙ Adjust {} (F9)", op_title)).clicked() {
                    state.show_last_operator_panel = true;
                }
            });
        return;
    }

    let mut updated_op: Option<LastOperatorInfo> = None;
    let mut close_panel = false;

    egui::Area::new(egui::Id::new("adjust_last_operator_panel"))
        .fixed_pos(egui::pos2(panel_x, panel_y - 120.0))
        .order(egui::Order::Middle)
        .show(&ctx, |ui| {
            egui::Frame::window(ui.style())
                .fill(egui::Color32::from_rgba_premultiplied(22, 26, 34, 245))
                .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(80, 95, 120)))
                .rounding(egui::Rounding::same(6.0))
                .inner_margin(egui::Margin::symmetric(10.0, 8.0))
                .show(ui, |ui| {
                    ui.set_width(220.0);
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(format!("▼ {} (F9)", op_title)).strong().color(egui::Color32::from_rgb(100, 200, 255)));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.small_button("✕").clicked() {
                                close_panel = true;
                            }
                        });
                    });
                    ui.separator();

                    match op.clone() {
                        LastOperatorInfo::Inset { faces, mut thickness, mut factor, mut individual, initial_radius } => {
                            let mut changed = false;
                            let r = initial_radius.max(0.001);
                            ui.horizontal(|ui| {
                                ui.label("Thickness:");
                                if ui.add(egui::DragValue::new(&mut thickness).speed(0.01).suffix(" m").range(0.0..=r * 0.99)).changed() {
                                    factor = (1.0 - thickness / r).clamp(0.01, 0.99);
                                    changed = true;
                                }
                            });
                            ui.horizontal(|ui| {
                                ui.label("Factor:");
                                let mut pct = factor * 100.0;
                                if ui.add(egui::DragValue::new(&mut pct).speed(0.5).suffix(" %").range(1.0..=99.0)).changed() {
                                    factor = (pct / 100.0).clamp(0.01, 0.99);
                                    thickness = (1.0 - factor) * r;
                                    changed = true;
                                }
                            });
                            if ui.checkbox(&mut individual, "Individual Faces").changed() {
                                changed = true;
                            }
                            if changed {
                                updated_op = Some(LastOperatorInfo::Inset {
                                    faces,
                                    thickness,
                                    factor,
                                    individual,
                                    initial_radius,
                                });
                            }
                        }
                        LastOperatorInfo::Extrude { faces, mut distance, axis } => {
                            let mut changed = false;
                            ui.horizontal(|ui| {
                                ui.label("Distance:");
                                if ui.add(egui::DragValue::new(&mut distance).speed(0.02).suffix(" m")).changed() {
                                    changed = true;
                                }
                            });
                            if changed {
                                updated_op = Some(LastOperatorInfo::Extrude { faces, distance, axis });
                            }
                        }
                        LastOperatorInfo::Bevel { edges, mut width, mut segments } => {
                            let mut changed = false;
                            ui.horizontal(|ui| {
                                ui.label("Width:");
                                if ui.add(egui::DragValue::new(&mut width).speed(0.005).suffix(" m").range(0.001..=5.0)).changed() {
                                    changed = true;
                                }
                            });
                            ui.horizontal(|ui| {
                                ui.label("Segments:");
                                if ui.add(egui::DragValue::new(&mut segments).range(1..=16)).changed() {
                                    changed = true;
                                }
                            });
                            if changed {
                                updated_op = Some(LastOperatorInfo::Bevel { edges, width, segments });
                            }
                        }
                        LastOperatorInfo::LoopCut { edge, mut num_cuts, mut factor } => {
                            let mut changed = false;
                            ui.horizontal(|ui| {
                                ui.label("Cuts:");
                                if ui.add(egui::DragValue::new(&mut num_cuts).range(1..=16)).changed() {
                                    changed = true;
                                }
                            });
                            ui.horizontal(|ui| {
                                ui.label("Factor:");
                                if ui.add(egui::Slider::new(&mut factor, 0.05..=0.95)).changed() {
                                    changed = true;
                                }
                            });
                            if changed {
                                updated_op = Some(LastOperatorInfo::LoopCut { edge, num_cuts, factor });
                            }
                        }
                    }
                });
        });

    if close_panel {
        state.show_last_operator_panel = false;
    }

    if let Some(new_op) = updated_op {
        if let Some(sel_id) = state.selected_node {
            match &new_op {
                LastOperatorInfo::Inset { faces, factor, individual, .. } => {
                    if state.history.undo(&mut state.scene).is_ok() {
                        let cmd = Box::new(InsetCommand {
                            node_id: sel_id,
                            faces: faces.clone(),
                            factor: *factor,
                            individual: *individual,
                            old_mesh: None,
                            created_faces: Vec::new(),
                        });
                        let _ = state.history.execute(cmd, &mut state.scene);
                        state.last_operator = Some(new_op);
                    }
                }
                LastOperatorInfo::Extrude { faces, distance, .. } => {
                    if state.history.undo(&mut state.scene).is_ok() {
                        let cmd = Box::new(ExtrudeCommand {
                            node_id: sel_id,
                            faces: faces.clone(),
                            distance: *distance,
                            old_mesh: None,
                        });
                        let _ = state.history.execute(cmd, &mut state.scene);
                        state.last_operator = Some(new_op);
                    }
                }
                LastOperatorInfo::Bevel { edges, width, segments } => {
                    if state.history.undo(&mut state.scene).is_ok() {
                        let cmd = Box::new(BevelEdgeCommand {
                            node_id: sel_id,
                            edges: edges.clone(),
                            width: *width,
                            segments: *segments,
                            old_mesh: None,
                        });
                        let _ = state.history.execute(cmd, &mut state.scene);
                        state.last_operator = Some(new_op);
                    }
                }
                LastOperatorInfo::LoopCut { edge, num_cuts, factor } => {
                    // Undo old cuts
                    let old_cuts = match &op {
                        LastOperatorInfo::LoopCut { num_cuts, .. } => *num_cuts,
                        _ => 1,
                    };
                    for _ in 0..old_cuts {
                        let _ = state.history.undo(&mut state.scene);
                    }
                    for k in 0..*num_cuts {
                        let t = if *num_cuts == 1 {
                            *factor
                        } else {
                            let base = (k + 1) as f32 / (*num_cuts + 1) as f32;
                            (base + (*factor - 0.5) * 0.4).clamp(0.05, 0.95)
                        };
                        let cmd = Box::new(LoopCutCommand {
                            node_id: sel_id,
                            edge_key: *edge,
                            ratio: t,
                            old_mesh: None,
                        });
                        let _ = state.history.execute(cmd, &mut state.scene);
                    }
                    state.last_operator = Some(new_op);
                }
            }
            state.window.request_redraw();
        }
    }
}

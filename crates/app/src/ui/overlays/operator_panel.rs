use crate::commands::{
    BevelEdgeCommand, BridgeEdgeLoopsCommand, ExtrudeCommand, InsetCommand, LoopCutCommand,
};
use crate::state::AppState;
use crate::types::{LastOperatorInfo, PrimitiveParams};

pub fn render(state: &mut AppState) {
    if state.last_operator.is_none() {
        return;
    }

    let ctx = state.egui_ctx.clone();
    let vp = state.egui_ctx.available_rect();
    let left_offset = if state.show_left_toolbar { 76.0 } else { 14.0 };
    let panel_x = vp.min.x + left_offset;
    let panel_y = vp.max.y - 36.0;

    let op = state.last_operator.as_ref().unwrap().clone();
    let op_title = match &op {
        LastOperatorInfo::Inset { .. } => "Inset Faces",
        LastOperatorInfo::Extrude { .. } => "Extrude Faces",
        LastOperatorInfo::Bevel { .. } => "Bevel Edges",
        LastOperatorInfo::LoopCut { .. } => "Loop Cut",
        LastOperatorInfo::Bridge { .. } => "Bridge Edge Loops",
        LastOperatorInfo::AddPrimitive { params, .. } => params.name(),
    };

    if !state.show_last_operator_panel {
        egui::Area::new(egui::Id::new("collapsed_operator_panel"))
            .fixed_pos(egui::pos2(panel_x, panel_y))
            .order(egui::Order::Middle)
            .show(&ctx, |ui| {
                if ui.button(format!("Adjust {} (F9)", op_title)).clicked() {
                    state.show_last_operator_panel = true;
                }
            });
        return;
    }

    let mut updated_op: Option<LastOperatorInfo> = None;
    let mut close_panel = false;

    egui::Area::new(egui::Id::new("adjust_last_operator_panel"))
        .fixed_pos(egui::pos2(panel_x, vp.max.y - 200.0))
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
                        ui.label(egui::RichText::new(format!("v {} (F9)", op_title)).strong().color(egui::Color32::from_rgb(100, 200, 255)));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.small_button("X").clicked() {
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
                        LastOperatorInfo::Bevel { edges, mut width, mut segments, mut profile_type, mut tension } => {
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
                            ui.horizontal(|ui| {
                                ui.label("Profile:");
                                let p_name = match profile_type { 1 => "Stepped", 2 => "Concave", _ => "Round" };
                                egui::ComboBox::from_id_salt("bevel_p_combo")
                                    .selected_text(p_name)
                                    .show_ui(ui, |ui| {
                                        changed |= ui.selectable_value(&mut profile_type, 0, "Round").changed();
                                        changed |= ui.selectable_value(&mut profile_type, 1, "Stepped").changed();
                                        changed |= ui.selectable_value(&mut profile_type, 2, "Concave").changed();
                                    });
                            });
                            if profile_type == 0 {
                                ui.horizontal(|ui| {
                                    ui.label("Tension:");
                                    if ui.add(egui::Slider::new(&mut tension, 0.0..=1.0)).changed() {
                                        changed = true;
                                    }
                                });
                            }
                            if changed {
                                updated_op = Some(LastOperatorInfo::Bevel { edges, width, segments, profile_type, tension });
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
                        LastOperatorInfo::Bridge { edges, mut twist, mut cuts } => {
                            let mut changed = false;
                            ui.horizontal(|ui| {
                                ui.label("Twist:");
                                if ui.add(egui::DragValue::new(&mut twist).range(-16..=16)).changed() {
                                    changed = true;
                                }
                            });
                            ui.horizontal(|ui| {
                                ui.label("Cuts:");
                                if ui.add(egui::DragValue::new(&mut cuts).range(0..=16)).changed() {
                                    changed = true;
                                }
                            });
                            if changed {
                                updated_op = Some(LastOperatorInfo::Bridge { edges, twist, cuts });
                            }
                        }
                        LastOperatorInfo::AddPrimitive { node_id, params } => {
                            let mut p = params.clone();
                            let mut changed = false;
                            match &mut p {
                                PrimitiveParams::Box { width, height, depth, seg_x, seg_y, seg_z } => {
                                    ui.horizontal(|ui| {
                                        ui.label("Size:");
                                        changed |= ui.add(egui::DragValue::new(width).speed(0.01).range(0.01..=100.0).prefix("X:")).changed();
                                        changed |= ui.add(egui::DragValue::new(height).speed(0.01).range(0.01..=100.0).prefix("Y:")).changed();
                                        changed |= ui.add(egui::DragValue::new(depth).speed(0.01).range(0.01..=100.0).prefix("Z:")).changed();
                                    });
                                    ui.horizontal(|ui| {
                                        ui.label("Segs:");
                                        changed |= ui.add(egui::DragValue::new(seg_x).range(1..=32).prefix("X:")).changed();
                                        changed |= ui.add(egui::DragValue::new(seg_y).range(1..=32).prefix("Y:")).changed();
                                        changed |= ui.add(egui::DragValue::new(seg_z).range(1..=32).prefix("Z:")).changed();
                                    });
                                }
                                PrimitiveParams::Plane { width, length, sub_x, sub_z } => {
                                    ui.horizontal(|ui| {
                                        ui.label("Dimensions:");
                                        changed |= ui.add(egui::DragValue::new(width).speed(0.01).range(0.01..=100.0).prefix("W:")).changed();
                                        changed |= ui.add(egui::DragValue::new(length).speed(0.01).range(0.01..=100.0).prefix("L:")).changed();
                                    });
                                    ui.horizontal(|ui| {
                                        ui.label("Subdivisions:");
                                        changed |= ui.add(egui::DragValue::new(sub_x).range(1..=64).prefix("X:")).changed();
                                        changed |= ui.add(egui::DragValue::new(sub_z).range(1..=64).prefix("Z:")).changed();
                                    });
                                }
                                PrimitiveParams::UvSphere { radius, segments, rings } => {
                                    ui.horizontal(|ui| {
                                        ui.label("Radius:");
                                        changed |= ui.add(egui::DragValue::new(radius).speed(0.01).range(0.01..=50.0)).changed();
                                    });
                                    ui.horizontal(|ui| {
                                        ui.label("Segments:");
                                        changed |= ui.add(egui::DragValue::new(segments).range(3..=64)).changed();
                                        ui.label("Rings:");
                                        changed |= ui.add(egui::DragValue::new(rings).range(2..=64)).changed();
                                    });
                                }
                                PrimitiveParams::Cylinder { radius, height, segments, cap_fill } => {
                                    ui.horizontal(|ui| {
                                        ui.label("Radius:");
                                        changed |= ui.add(egui::DragValue::new(radius).speed(0.01).range(0.01..=50.0)).changed();
                                        ui.label("Height:");
                                        changed |= ui.add(egui::DragValue::new(height).speed(0.01).range(0.01..=50.0)).changed();
                                    });
                                    ui.horizontal(|ui| {
                                        ui.label("Segments:");
                                        changed |= ui.add(egui::DragValue::new(segments).range(3..=64)).changed();
                                    });
                                    ui.horizontal(|ui| {
                                        ui.label("Cap Fill:");
                                        changed |= ui.selectable_value(cap_fill, geo_kernel::CapFillType::Ngon, "N-Gon").changed();
                                        changed |= ui.selectable_value(cap_fill, geo_kernel::CapFillType::TriangleFan, "Fan").changed();
                                        changed |= ui.selectable_value(cap_fill, geo_kernel::CapFillType::None, "None").changed();
                                    });
                                }
                                PrimitiveParams::Cone { radius_bottom, radius_top, height, segments, cap_fill } => {
                                    ui.horizontal(|ui| {
                                        ui.label("R Base:");
                                        changed |= ui.add(egui::DragValue::new(radius_bottom).speed(0.01).range(0.01..=50.0)).changed();
                                        ui.label("R Top:");
                                        changed |= ui.add(egui::DragValue::new(radius_top).speed(0.01).range(0.0..=50.0)).changed();
                                    });
                                    ui.horizontal(|ui| {
                                        ui.label("Height:");
                                        changed |= ui.add(egui::DragValue::new(height).speed(0.01).range(0.01..=50.0)).changed();
                                        ui.label("Segments:");
                                        changed |= ui.add(egui::DragValue::new(segments).range(3..=64)).changed();
                                    });
                                    ui.horizontal(|ui| {
                                        ui.label("Cap Fill:");
                                        changed |= ui.selectable_value(cap_fill, geo_kernel::CapFillType::Ngon, "N-Gon").changed();
                                        changed |= ui.selectable_value(cap_fill, geo_kernel::CapFillType::TriangleFan, "Fan").changed();
                                    });
                                }
                                PrimitiveParams::Torus { major_radius, minor_radius, major_segments, minor_segments } => {
                                    ui.horizontal(|ui| {
                                        ui.label("R Major:");
                                        changed |= ui.add(egui::DragValue::new(major_radius).speed(0.01).range(0.05..=50.0)).changed();
                                        ui.label("R Minor:");
                                        changed |= ui.add(egui::DragValue::new(minor_radius).speed(0.01).range(0.01..=20.0)).changed();
                                    });
                                    ui.horizontal(|ui| {
                                        ui.label("Segments:");
                                        changed |= ui.add(egui::DragValue::new(major_segments).range(3..=64).prefix("Maj:")).changed();
                                        changed |= ui.add(egui::DragValue::new(minor_segments).range(3..=64).prefix("Min:")).changed();
                                    });
                                }
                                PrimitiveParams::Pyramid { base_width, base_depth, height } => {
                                    ui.horizontal(|ui| {
                                        ui.label("Base W/D:");
                                        changed |= ui.add(egui::DragValue::new(base_width).speed(0.01).range(0.01..=50.0).prefix("W:")).changed();
                                        changed |= ui.add(egui::DragValue::new(base_depth).speed(0.01).range(0.01..=50.0).prefix("D:")).changed();
                                    });
                                    ui.horizontal(|ui| {
                                        ui.label("Height:");
                                        changed |= ui.add(egui::DragValue::new(height).speed(0.01).range(0.01..=50.0)).changed();
                                    });
                                }
                                PrimitiveParams::Capsule { radius, cylinder_height, segments, hemisphere_rings } => {
                                    ui.horizontal(|ui| {
                                        ui.label("Radius:");
                                        changed |= ui.add(egui::DragValue::new(radius).speed(0.01).range(0.01..=50.0)).changed();
                                        ui.label("Height:");
                                        changed |= ui.add(egui::DragValue::new(cylinder_height).speed(0.01).range(0.01..=50.0)).changed();
                                    });
                                    ui.horizontal(|ui| {
                                        ui.label("Segments:");
                                        changed |= ui.add(egui::DragValue::new(segments).range(3..=64)).changed();
                                        ui.label("Rings:");
                                        changed |= ui.add(egui::DragValue::new(hemisphere_rings).range(2..=32)).changed();
                                    });
                                }
                                PrimitiveParams::IcoSphere { radius, subdivisions } => {
                                    ui.horizontal(|ui| {
                                        ui.label("Radius:");
                                        changed |= ui.add(egui::DragValue::new(radius).speed(0.01).range(0.01..=50.0)).changed();
                                        ui.label("Subdivisions:");
                                        changed |= ui.add(egui::DragValue::new(subdivisions).range(1..=4)).changed();
                                    });
                                }
                                PrimitiveParams::PlumbBob { segments } => {
                                    ui.horizontal(|ui| {
                                        ui.label("Radial Segments:");
                                        changed |= ui.add(egui::DragValue::new(segments).range(8..=48)).changed();
                                    });
                                }
                                PrimitiveParams::GeodesicDome { radius, frequency } => {
                                    ui.horizontal(|ui| {
                                        ui.label("Radius:");
                                        changed |= ui.add(egui::DragValue::new(radius).speed(0.01).range(0.1..=50.0)).changed();
                                        ui.label("Frequency (V):");
                                        changed |= ui.add(egui::DragValue::new(frequency).range(1..=6)).changed();
                                    });
                                }
                                PrimitiveParams::PipeThread { major_diameter, pitch, thread_length, segments_per_turn } => {
                                    ui.horizontal(|ui| {
                                        ui.label("Major Dia:");
                                        changed |= ui.add(egui::DragValue::new(major_diameter).speed(0.01).range(0.1..=20.0)).changed();
                                        ui.label("Pitch:");
                                        changed |= ui.add(egui::DragValue::new(pitch).speed(0.005).range(0.05..=5.0)).changed();
                                    });
                                    ui.horizontal(|ui| {
                                        ui.label("Length:");
                                        changed |= ui.add(egui::DragValue::new(thread_length).speed(0.01).range(0.1..=50.0)).changed();
                                        ui.label("Segs/Turn:");
                                        changed |= ui.add(egui::DragValue::new(segments_per_turn).range(8..=64)).changed();
                                    });
                                }
                            }
                            if changed {
                                updated_op = Some(LastOperatorInfo::AddPrimitive { node_id, params: p });
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
                        let cmd = Box::new(ExtrudeCommand::new(
                            sel_id,
                            faces.clone(),
                            *distance,
                        ));
                        let _ = state.history.execute(cmd, &mut state.scene);
                        state.last_operator = Some(new_op);
                    }
                }
                LastOperatorInfo::Bevel { edges, width, segments, profile_type, tension } => {
                    if state.history.undo(&mut state.scene).is_ok() {
                        let profile = match profile_type {
                            1 => geo_kernel::BevelProfile::Stepped,
                            2 => geo_kernel::BevelProfile::Concave,
                            _ => geo_kernel::BevelProfile::Round(*tension),
                        };
                        let cmd = Box::new(BevelEdgeCommand {
                            node_id: sel_id,
                            edges: edges.clone(),
                            width: *width,
                            segments: *segments,
                            profile,
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
                    let cmd = Box::new(LoopCutCommand {
                        node_id: sel_id,
                        edge_key: *edge,
                        ratio: *factor,
                        num_cuts: *num_cuts,
                        old_mesh: None,
                    });
                    let _ = state.history.execute(cmd, &mut state.scene);
                    state.last_operator = Some(new_op);
                }
                LastOperatorInfo::Bridge { edges, twist, cuts } => {
                    if state.history.undo(&mut state.scene).is_ok() {
                        let cmd = Box::new(BridgeEdgeLoopsCommand::from_edges(
                            sel_id,
                            edges.clone(),
                            *twist,
                            *cuts,
                        ));
                        let _ = state.history.execute(cmd, &mut state.scene);
                        state.last_operator = Some(new_op);
                    }
                }
                LastOperatorInfo::AddPrimitive { node_id, params } => {
                    if let Some(node) = state.scene.nodes.get_mut(*node_id) {
                        if let graph_engine::NodeData::Mesh { base_mesh, .. } = &mut node.data {
                            *base_mesh = params.generate_mesh();
                            state.scene.mark_dirty(*node_id);
                        }
                    }
                    state.sync_gpu_meshes();
                    state.last_operator = Some(new_op);
                }
            }
            state.window.request_redraw();
        }
    }
}

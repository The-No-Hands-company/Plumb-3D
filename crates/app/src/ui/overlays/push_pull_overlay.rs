use graph_engine::NodeData;

use crate::state::AppState;

/// Interactive Viewport HUD and normal guide for the modal Bi-Directional Push-Pull Carving tool.
pub fn render(state: &mut AppState) {
    if !state.push_pull_modal_active {
        return;
    }

    let width = state.surface_config.width as f32;
    let height = state.surface_config.height as f32;

    let painter = state.egui_ctx.layer_painter(egui::LayerId::new(
        egui::Order::Foreground,
        egui::Id::new("push_pull_modal_overlay"),
    ));

    // 1. Draw 3D normal vector guide and displacement point
    if let (Some(sel_id), Some(face_key)) = (state.selected_node, state.push_pull_modal_face) {
        if let Some(node) = state.scene.nodes.get(sel_id) {
            if let NodeData::Mesh { base_mesh, .. } = &node.data {
                if let Some(face) = base_mesh.faces.get(face_key) {
                    let c = node
                        .world_matrix
                        .transform_point3(base_mesh.face_centroid(face_key));
                    let n = node
                        .world_matrix
                        .transform_vector3(face.normal)
                        .normalize_or_zero();
                    let target_p = c + n * state.push_pull_modal_distance;

                    if let (Some(s_c), Some(s_t)) = (
                        state.camera.world_to_screen(c, width, height),
                        state.camera.world_to_screen(target_p, width, height),
                    ) {
                        let is_boss = state.push_pull_modal_distance >= 0.0;
                        let line_col = if is_boss {
                            egui::Color32::from_rgb(100, 220, 120) // Green for Boss
                        } else {
                            egui::Color32::from_rgb(255, 140, 50) // Orange for Cavity
                        };

                        painter.line_segment(
                            [egui::pos2(s_c.x, s_c.y), egui::pos2(s_t.x, s_t.y)],
                            egui::Stroke::new(2.5_f32, line_col),
                        );
                        painter.circle_filled(egui::pos2(s_t.x, s_t.y), 5.0_f32, line_col);
                    }
                }
            }
        }
    }

    // 2. Interactive Top-Center HUD Banner
    let mut commit_action = false;
    let mut cancel_action = false;
    let is_boss = state.push_pull_modal_distance >= 0.0;

    let banner_col = if is_boss {
        egui::Color32::from_rgb(80, 200, 120)
    } else {
        egui::Color32::from_rgb(255, 140, 50)
    };

    let title = if is_boss {
        "⇡ PUSH-PULL: BOSS UNION"
    } else {
        "⇣ PUSH-PULL: CAVITY CUT"
    };

    let ctx = state.egui_ctx.clone();
    egui::Area::new(egui::Id::new("push_pull_hud_area"))
        .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 50.0))
        .order(egui::Order::Foreground)
        .show(&ctx, |ui| {
            egui::Frame::window(ui.style())
                .fill(egui::Color32::from_rgba_premultiplied(22, 26, 32, 240))
                .stroke(egui::Stroke::new(1.5_f32, banner_col))
                .rounding(egui::Rounding::same(8.0))
                .inner_margin(egui::Margin::symmetric(14.0, 8.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(title).strong().color(banner_col));
                        ui.separator();
                        ui.label("Depth:");
                        ui.add(
                            egui::DragValue::new(&mut state.push_pull_modal_distance)
                                .speed(0.01)
                                .suffix(" m"),
                        );

                        if !state.modal_numeric_buffer.is_empty() {
                            ui.label(
                                egui::RichText::new(format!("[{}]", state.modal_numeric_buffer))
                                    .color(egui::Color32::YELLOW)
                                    .monospace(),
                            );
                        }

                        ui.separator();
                        if ui
                            .button(
                                egui::RichText::new("Confirm (LMB)")
                                    .color(egui::Color32::from_rgb(120, 220, 140)),
                            )
                            .clicked()
                        {
                            commit_action = true;
                        }
                        if ui.button("Cancel (RMB/Esc)").clicked() {
                            cancel_action = true;
                        }
                    });
                });
        });

    if commit_action {
        crate::modals::push_pull::commit(state);
    } else if cancel_action {
        crate::modals::push_pull::cancel(state);
    }
}

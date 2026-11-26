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
        if let Some(node) = state.scene.nodes.get(sel_id) {
            if let NodeData::Mesh { base_mesh, .. } = &node.data {
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

                let (response, painter) = ui.allocate_painter(egui::vec2(ui.available_width(), 380.0), egui::Sense::drag());
                let rect = response.rect;

                painter.rect_filled(rect, 4.0, egui::Color32::from_rgb(20, 22, 26));
                painter.rect_stroke(rect, 4.0, egui::Stroke::new(1.0_f32, egui::Color32::from_white_alpha(30)));

                let grid_lines = 8;
                for i in 1..grid_lines {
                    let gx = rect.left() + (i as f32 / grid_lines as f32) * rect.width();
                    let gy = rect.top() + (i as f32 / grid_lines as f32) * rect.height();
                    painter.line_segment([egui::pos2(gx, rect.top()), egui::pos2(gx, rect.bottom())], egui::Stroke::new(0.5_f32, egui::Color32::from_white_alpha(15)));
                    painter.line_segment([egui::pos2(rect.left(), gy), egui::pos2(rect.right(), gy)], egui::Stroke::new(0.5_f32, egui::Color32::from_white_alpha(15)));
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
                            painter.line_segment([pts[i], pts[next_i]], egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(70, 160, 255)));
                        }
                    }
                }

                for (_ek, edge) in &base_mesh.edges {
                    if edge.is_seam {
                        let he = &base_mesh.half_edges[edge.half_edge];
                        let uv0 = base_mesh.vertices[he.origin].uv;
                        let uv1 = base_mesh.vertices[he.target].uv;
                        let p0 = egui::pos2(rect.left() + uv0.x * rect.width(), rect.top() + (1.0 - uv0.y) * rect.height());
                        let p1 = egui::pos2(rect.left() + uv1.x * rect.width(), rect.top() + (1.0 - uv1.y) * rect.height());
                        painter.line_segment([p0, p1], egui::Stroke::new(2.5_f32, egui::Color32::from_rgb(255, 50, 50)));
                    }
                }

                ui.separator();
                ui.horizontal(|ui| {
                    ui.label(format!("Object: {} | Verts: {} | Faces: {}", node.name, base_mesh.vertices.len(), base_mesh.faces.len()));
                });
            }
        }
    } else {
        ui.label("Select a mesh object in the 3D scene to view and edit its UV coordinates.");
    }
}

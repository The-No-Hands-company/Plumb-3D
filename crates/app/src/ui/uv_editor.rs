use graph_engine::NodeData;

use crate::state::AppState;
use crate::ui::actions::UiActions;

pub fn draw_uv_editor(state: &mut AppState, actions: &mut UiActions) {
        // Render 2D UV Editor Window (Blender / Maya UV Editor)
        if state.show_uv_editor {
            egui::Window::new("🌐 2D UV Editor & Texture Coordinates")
                .open(&mut state.show_uv_editor)
                .default_size(egui::vec2(520.0, 560.0))
                .resizable(true)
                .show(&state.egui_ctx, |ui| {
                    if let Some(sel_id) = state.selected_node {
                        if let Some(node) = state.scene.nodes.get(sel_id) {
                            if let NodeData::Mesh { base_mesh, .. } = &node.data {
                                ui.horizontal(|ui| {
                                    if ui.button("🌐 Unwrap Seams").clicked() {
                                        actions.trigger_unwrap_seams = true;
                                    }
                                    if ui.button("📐 Smart Project").clicked() {
                                        actions.trigger_smart_uv_project = true;
                                    }
                                    if ui.button("✂ Mark Seam").clicked() {
                                        actions.trigger_mark_seam = true;
                                    }
                                    if ui.button("✖ Clear Seam").clicked() {
                                        actions.trigger_clear_seam = true;
                                    }
                                    if ui.button("🗑 Clear All").clicked() {
                                        actions.trigger_clear_all_seams = true;
                                    }
                                });

                                ui.horizontal(|ui| {
                                    if ui.button("📦 Pack Islands").clicked() {
                                        actions.trigger_pack_uv_islands = true;
                                    }
                                    if ui.button("↷ Rotate CW").clicked() {
                                        actions.trigger_rotate_uv_cw = true;
                                    }
                                    if ui.button("↶ Rotate CCW").clicked() {
                                        actions.trigger_rotate_uv_ccw = true;
                                    }
                                    if ui.button("↔ Flip H").clicked() {
                                        actions.trigger_flip_uv_h = true;
                                    }
                                    if ui.button("↕ Flip V").clicked() {
                                        actions.trigger_flip_uv_v = true;
                                    }
                                    if ui.button("➕ 1.2x").clicked() {
                                        actions.trigger_scale_uv = Some(1.2);
                                    }
                                    if ui.button("➖ 0.8x").clicked() {
                                        actions.trigger_scale_uv = Some(0.8);
                                    }
                                    if ui.button("🍃 Relax UVs").on_hover_text("Smooths vertex UVs using Laplacian diffusion to minimize distortion while pinning island boundaries").clicked() {
                                        actions.trigger_relax_uvs = true;
                                    }
                                });

                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new("Projections:").weak());
                                    if ui.button("📐 Planar Y").clicked() { actions.trigger_planar_uv = true; }
                                    if ui.button("📦 Box Map").clicked() { actions.trigger_box_uv = true; }
                                    if ui.button("🥫 Cylinder").clicked() { actions.trigger_cylindrical_uv = true; }
                                    if ui.button("🌐 Sphere").clicked() { actions.trigger_spherical_uv = true; }
                                });

                                ui.horizontal(|ui| {
                                    ui.add(egui::Slider::new(&mut state.smart_uv_angle_threshold, 15.0..=89.0).text("Angle Thresh (°)"));
                                    ui.add(egui::Slider::new(&mut state.smart_uv_margin, 0.005..=0.1).text("Margin"));
                                });

                                ui.separator();

                                let (response, painter) = ui.allocate_painter(
                                    egui::vec2(ui.available_width(), (ui.available_height() - 40.0).max(320.0)),
                                    egui::Sense::hover(),
                                );
                                let rect = response.rect;

                                painter.rect_filled(rect, 4.0, egui::Color32::from_rgb(22, 24, 28));
                                painter.rect_stroke(rect, 4.0, egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(55, 60, 70)));

                                for i in 1..10 {
                                    let t = i as f32 / 10.0;
                                    let x = rect.left() + rect.width() * t;
                                    let y = rect.top() + rect.height() * t;
                                    painter.line_segment([egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())], egui::Stroke::new(0.5_f32, egui::Color32::from_rgb(38, 42, 50)));
                                    painter.line_segment([egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)], egui::Stroke::new(0.5_f32, egui::Color32::from_rgb(38, 42, 50)));
                                }

                                for fk in base_mesh.faces.keys() {
                                    let v_keys = base_mesh.face_vertices(fk);
                                    let pts: Vec<egui::Pos2> = v_keys.iter().map(|&vk| {
                                        let uv = base_mesh.vertices[vk].uv;
                                        egui::pos2(
                                            rect.left() + uv.x.clamp(0.0, 1.0) * rect.width(),
                                            rect.top() + (1.0 - uv.y.clamp(0.0, 1.0)) * rect.height(),
                                        )
                                    }).collect();

                                    if pts.len() >= 3 {
                                        let is_sel = state.selected_faces.contains(&fk);
                                        let poly_fill = if is_sel {
                                            egui::Color32::from_rgba_premultiplied(255, 180, 50, 60)
                                        } else {
                                            egui::Color32::from_rgba_premultiplied(100, 150, 220, 25)
                                        };
                                        let poly_stroke = if is_sel {
                                            egui::Stroke::new(1.8_f32, egui::Color32::from_rgb(255, 200, 50))
                                        } else {
                                            egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(120, 160, 220))
                                        };
                                        painter.add(egui::Shape::convex_polygon(pts.clone(), poly_fill, poly_stroke));
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
                });
        }


}

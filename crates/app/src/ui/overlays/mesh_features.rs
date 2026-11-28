use graph_engine::NodeData;

use crate::state::AppState;
use crate::types::*;

pub fn render(state: &mut AppState) {
    // 1. Render UV Seams Overlay in Edit Mode (Bright Crimson Red lines)
    if state.interaction_mode == InteractionMode::Edit {
        if let Some(sel_id) = state.selected_node {
            if let Some(node) = state.scene.nodes.get(sel_id) {
                if let NodeData::Mesh { base_mesh, .. } = &node.data {
                    let width = state.surface_config.width as f32;
                    let height = state.surface_config.height as f32;
                    let painter = state.egui_ctx.layer_painter(egui::LayerId::new(
                        egui::Order::Foreground,
                        egui::Id::new("uv_seams_overlay"),
                    ));

                    for (_ek, edge) in &base_mesh.edges {
                        if edge.is_seam {
                            let he = &base_mesh.half_edges[edge.half_edge];
                            let p0 = node
                                .world_matrix
                                .transform_point3(base_mesh.vertices[he.origin].position);
                            let p1 = node
                                .world_matrix
                                .transform_point3(base_mesh.vertices[he.target].position);
                            if let (Some(s0), Some(s1)) = (
                                state.camera.world_to_screen(p0, width, height),
                                state.camera.world_to_screen(p1, width, height),
                            ) {
                                painter.line_segment(
                                    [egui::pos2(s0.x, s0.y), egui::pos2(s1.x, s1.y)],
                                    egui::Stroke::new(
                                        2.8_f32,
                                        egui::Color32::from_rgb(255, 50, 50),
                                    ),
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    // 2. Render Creased and Sharp Edges & Vertices in Edit Mode (Blender Shift+E / Shift+V Crease / Maya Hard Edges)
    if state.interaction_mode == InteractionMode::Edit {
        if let Some(sel_id) = state.selected_node {
            if let Some(node) = state.scene.nodes.get(sel_id) {
                if let NodeData::Mesh { base_mesh, .. } = &node.data {
                    let width = state.surface_config.width as f32;
                    let height = state.surface_config.height as f32;
                    let painter = state.egui_ctx.layer_painter(egui::LayerId::new(
                        egui::Order::Foreground,
                        egui::Id::new("crease_sharp_overlay"),
                    ));

                    for (_ek, edge) in &base_mesh.edges {
                        if edge.crease_weight > 0.0 || edge.is_sharp {
                            let he = &base_mesh.half_edges[edge.half_edge];
                            let p0 = node
                                .world_matrix
                                .transform_point3(base_mesh.vertices[he.origin].position);
                            let p1 = node
                                .world_matrix
                                .transform_point3(base_mesh.vertices[he.target].position);
                            if let (Some(s0), Some(s1)) = (
                                state.camera.world_to_screen(p0, width, height),
                                state.camera.world_to_screen(p1, width, height),
                            ) {
                                let (color, stroke_w) = if edge.crease_weight > 0.0 {
                                    (
                                        egui::Color32::from_rgb(217, 70, 239),
                                        1.8 + edge.crease_weight * 2.5,
                                    )
                                } else {
                                    (egui::Color32::from_rgb(6, 182, 212), 2.2)
                                };
                                painter.line_segment(
                                    [egui::pos2(s0.x, s0.y), egui::pos2(s1.x, s1.y)],
                                    egui::Stroke::new(stroke_w, color),
                                );
                            }
                        }
                    }

                    // Render Vertex Crease Halos
                    for (_vk, vertex) in &base_mesh.vertices {
                        if vertex.crease_weight > 0.0 {
                            let p = node.world_matrix.transform_point3(vertex.position);
                            if let Some(s) = state.camera.world_to_screen(p, width, height) {
                                let pos = egui::pos2(s.x, s.y);
                                let halo_r = 4.0 + vertex.crease_weight * 3.5;
                                painter.circle_stroke(
                                    pos,
                                    halo_r,
                                    egui::Stroke::new(
                                        1.8_f32,
                                        egui::Color32::from_rgb(56, 189, 248),
                                    ),
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    // 3. Render Subdivision Control Cage Overlay in Edit Mode (3ds Max Subd Cage / Blender Optimal Display Cage)
    if state.show_subd_cage && state.interaction_mode == InteractionMode::Edit {
        if let Some(sel_id) = state.selected_node {
            if let Some(node) = state.scene.nodes.get(sel_id) {
                if let NodeData::Mesh {
                    base_mesh,
                    modifier_stack,
                    ..
                } = &node.data
                {
                    let has_subdiv = modifier_stack.modifiers().iter().any(|m| {
                        m.is_enabled() && (m.name() == "Subdivide" || m.name() == "Catmull-Clark")
                    });

                    if has_subdiv {
                        let width = state.surface_config.width as f32;
                        let height = state.surface_config.height as f32;
                        let painter = state.egui_ctx.layer_painter(egui::LayerId::new(
                            egui::Order::Foreground,
                            egui::Id::new("subd_control_cage_overlay"),
                        ));

                        // 1. Draw control cage edges
                        for (ek, edge) in &base_mesh.edges {
                            let he = &base_mesh.half_edges[edge.half_edge];
                            let p0 = node
                                .world_matrix
                                .transform_point3(base_mesh.vertices[he.origin].position);
                            let p1 = node
                                .world_matrix
                                .transform_point3(base_mesh.vertices[he.target].position);
                            if let (Some(s0), Some(s1)) = (
                                state.camera.world_to_screen(p0, width, height),
                                state.camera.world_to_screen(p1, width, height),
                            ) {
                                let is_selected = state.selected_edges.contains(&ek);
                                let (color, stroke_w) = if is_selected {
                                    (egui::Color32::from_rgb(255, 230, 80), 2.5_f32)
                                } else {
                                    (egui::Color32::from_rgb(255, 150, 40), 1.4_f32)
                                };
                                painter.line_segment(
                                    [egui::pos2(s0.x, s0.y), egui::pos2(s1.x, s1.y)],
                                    egui::Stroke::new(stroke_w, color),
                                );
                            }
                        }

                        // 2. Draw control cage vertices
                        for (vk, v) in &base_mesh.vertices {
                            let p = node.world_matrix.transform_point3(v.position);
                            if let Some(s) = state.camera.world_to_screen(p, width, height) {
                                let pos = egui::pos2(s.x, s.y);
                                let is_selected = state.selected_vertices.contains(&vk);
                                if is_selected {
                                    painter.circle_filled(
                                        pos,
                                        4.5_f32,
                                        egui::Color32::from_rgb(255, 230, 80),
                                    );
                                    painter.circle_stroke(
                                        pos,
                                        4.5_f32,
                                        egui::Stroke::new(1.5_f32, egui::Color32::WHITE),
                                    );
                                } else {
                                    painter.circle_filled(
                                        pos,
                                        3.0_f32,
                                        egui::Color32::from_rgb(255, 150, 40),
                                    );
                                    painter.circle_stroke(
                                        pos,
                                        3.0_f32,
                                        egui::Stroke::new(1.0_f32, egui::Color32::BLACK),
                                    );
                                }
                            }
                        }

                        // 3. Status indicator pill in top overlay
                        let cage_pill_rect = egui::Rect::from_min_size(
                            egui::pos2(width - 290.0, 45.0),
                            egui::vec2(275.0, 24.0),
                        );
                        painter.rect_filled(
                            cage_pill_rect,
                            4.0,
                            egui::Color32::from_black_alpha(200),
                        );
                        painter.rect_stroke(
                            cage_pill_rect,
                            4.0,
                            egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(255, 150, 40)),
                        );
                        painter.text(
                            cage_pill_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            format!(
                                "SUBD CONTROL CAGE | Optimal: {}",
                                if state.optimal_display { "ON" } else { "OFF" }
                            ),
                            egui::FontId::proportional(11.0),
                            egui::Color32::from_rgb(255, 220, 160),
                        );
                    }
                }
            }
        }
    }

    // 4. Render Face Normals Overlay (Blender Edit Mode: Face Normals Whiskers)
    if state.show_face_normals && state.interaction_mode == InteractionMode::Edit {
        if let Some(sel_id) = state.selected_node {
            if let Some(node) = state.scene.nodes.get(sel_id) {
                if let NodeData::Mesh { base_mesh, .. } = &node.data {
                    let width = state.surface_config.width as f32;
                    let height = state.surface_config.height as f32;
                    let painter = state.egui_ctx.layer_painter(egui::LayerId::new(
                        egui::Order::Foreground,
                        egui::Id::new("face_normals_overlay"),
                    ));

                    let normal_mat = node.world_matrix.inverse().transpose();
                    let len = state.normal_display_length;
                    for (fk, face) in &base_mesh.faces {
                        let local_c = base_mesh.face_centroid(fk);
                        let world_c = node.world_matrix.transform_point3(local_c);
                        let world_n = normal_mat
                            .transform_vector3(face.normal)
                            .normalize_or_zero();
                        let world_tip = world_c + world_n * len;

                        if let (Some(s0), Some(s1)) = (
                            state.camera.world_to_screen(world_c, width, height),
                            state.camera.world_to_screen(world_tip, width, height),
                        ) {
                            painter.line_segment(
                                [egui::pos2(s0.x, s0.y), egui::pos2(s1.x, s1.y)],
                                egui::Stroke::new(1.6_f32, egui::Color32::from_rgb(6, 182, 212)),
                            );
                            painter.circle_filled(
                                egui::pos2(s1.x, s1.y),
                                2.2_f32,
                                egui::Color32::from_rgb(34, 211, 238),
                            );
                        }
                    }
                }
            }
        }
    }

    // 5. Render Vertex Normals Overlay (Blender Edit Mode: Vertex Normals Whiskers)
    if state.show_vertex_normals && state.interaction_mode == InteractionMode::Edit {
        if let Some(sel_id) = state.selected_node {
            if let Some(node) = state.scene.nodes.get(sel_id) {
                if let NodeData::Mesh { base_mesh, .. } = &node.data {
                    let width = state.surface_config.width as f32;
                    let height = state.surface_config.height as f32;
                    let painter = state.egui_ctx.layer_painter(egui::LayerId::new(
                        egui::Order::Foreground,
                        egui::Id::new("vertex_normals_overlay"),
                    ));

                    let normal_mat = node.world_matrix.inverse().transpose();
                    let len = state.normal_display_length;
                    for (_vk, vertex) in &base_mesh.vertices {
                        let world_p = node.world_matrix.transform_point3(vertex.position);
                        let world_n = normal_mat
                            .transform_vector3(vertex.normal)
                            .normalize_or_zero();
                        let world_tip = world_p + world_n * len;

                        if let (Some(s0), Some(s1)) = (
                            state.camera.world_to_screen(world_p, width, height),
                            state.camera.world_to_screen(world_tip, width, height),
                        ) {
                            painter.line_segment(
                                [egui::pos2(s0.x, s0.y), egui::pos2(s1.x, s1.y)],
                                egui::Stroke::new(1.4_f32, egui::Color32::from_rgb(96, 165, 250)),
                            );
                        }
                    }
                }
            }
        }
    }

    // 6. Render Architectural CAD Dimension Line Overlay between Selected Vertices
    if state.interaction_mode == InteractionMode::Edit && state.selected_vertices.len() == 2 {
        if let Some(sel_id) = state.selected_node {
            if let Some(node) = state.scene.nodes.get(sel_id) {
                if let NodeData::Mesh { base_mesh, .. } = &node.data {
                    let mut iter = state.selected_vertices.iter();
                    let v0 = *iter.next().unwrap();
                    let v1 = *iter.next().unwrap();
                    if let (Some(vert0), Some(vert1)) =
                        (base_mesh.vertices.get(v0), base_mesh.vertices.get(v1))
                    {
                        let w0 = node.world_matrix.transform_point3(vert0.position);
                        let w1 = node.world_matrix.transform_point3(vert1.position);
                        let width = state.surface_config.width as f32;
                        let height = state.surface_config.height as f32;

                        if let (Some(s0), Some(s1)) = (
                            state.camera.world_to_screen(w0, width, height),
                            state.camera.world_to_screen(w1, width, height),
                        ) {
                            let painter = state.egui_ctx.layer_painter(egui::LayerId::new(
                                egui::Order::Foreground,
                                egui::Id::new("cad_dimension_overlay"),
                            ));

                            let p_a = egui::pos2(s0.x, s0.y);
                            let p_b = egui::pos2(s1.x, s1.y);
                            let stroke =
                                egui::Stroke::new(2.0_f32, egui::Color32::from_rgb(255, 215, 100));

                            // Main dimension line
                            painter.line_segment([p_a, p_b], stroke);

                            // End tick marks
                            let dir = (p_b - p_a).normalized();
                            let perp = egui::vec2(-dir.y, dir.x) * 6.0_f32;
                            painter.line_segment([p_a - perp, p_a + perp], stroke);
                            painter.line_segment([p_b - perp, p_b + perp], stroke);

                            // Distance text badge at midpoint
                            let mid = p_a + (p_b - p_a) * 0.5_f32;
                            let dist_3d = (w1 - w0).length();
                            let text = format!("{:.3}m", dist_3d);

                            let text_color = egui::Color32::from_rgb(255, 240, 180);
                            let bg_color = egui::Color32::from_rgba_premultiplied(15, 18, 25, 230);
                            let font_id = egui::FontId::monospace(11.0);

                            let text_galley = painter.layout_no_wrap(text, font_id, text_color);
                            let pad = egui::vec2(6.0_f32, 3.0_f32);
                            let badge_rect =
                                egui::Rect::from_center_size(mid, text_galley.size() + pad);

                            painter.rect_filled(badge_rect, 4.0_f32, bg_color);
                            painter.rect_stroke(
                                badge_rect,
                                4.0_f32,
                                egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(255, 215, 100)),
                            );
                            painter.galley(badge_rect.min + pad * 0.5_f32, text_galley, text_color);
                        }
                    }
                }
            }
        }
    }
}

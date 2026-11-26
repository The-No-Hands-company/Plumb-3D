use crate::state::AppState;
use crate::types::*;
use crate::ui::actions::UiActions;
use geo_kernel::*;
use graph_engine::{InterpolationMode, NodeData, NodeId};

pub fn draw_mesh_tools_panel(
    ui: &mut egui::Ui,
    state: &mut AppState,
    actions: &mut UiActions,
    sel_id: NodeId,
) {
    if let Some(node) = state.scene.nodes.get_mut(sel_id) {
                        if state.interaction_mode == InteractionMode::Edit {
                            ui.heading("Sub-Element Selection (1 / 2 / 3)");
                            ui.horizontal(|ui| {
                                if ui.selectable_label(state.sub_element_level == SubElementLevel::Vertex, "1. Vertex").clicked() {
                                    state.sub_element_level = SubElementLevel::Vertex;
                                }
                                if ui.selectable_label(state.sub_element_level == SubElementLevel::Edge, "2. Edge").clicked() {
                                    state.sub_element_level = SubElementLevel::Edge;
                                }
                                if ui.selectable_label(state.sub_element_level == SubElementLevel::Face, "3. Face").clicked() {
                                    state.sub_element_level = SubElementLevel::Face;
                                }
                                if ui.selectable_label(state.is_box_selecting, "Box Select (B)").clicked() {
                                    state.is_box_selecting = !state.is_box_selecting;
                                }
                            });

                            ui.horizontal(|ui| {
                                match state.sub_element_level {
                                    SubElementLevel::Vertex => {
                                        ui.label(format!("Selected: {} vertices", state.selected_vertices.len()));
                                    }
                                    SubElementLevel::Edge => {
                                        ui.label(format!("Selected: {} edges", state.selected_edges.len()));
                                    }
                                    SubElementLevel::Face => {
                                        ui.label(format!("Selected: {} faces", state.selected_faces.len()));
                                    }
                                }
                            });
                            ui.horizontal_wrapped(|ui| {
                                if ui.button("Select All (A)").clicked() {
                                    actions.trigger_select_all = true;
                                }
                                if ui.button("Deselect (Alt+A)").clicked() {
                                    actions.trigger_deselect_all = true;
                                }
                                if ui.button("Invert (Ctrl+I)").clicked() {
                                    actions.trigger_invert_selection = true;
                                }
                            });
                            ui.horizontal_wrapped(|ui| {
                                if ui.button("Grow (+)").clicked() {
                                    actions.trigger_grow_selection = true;
                                }
                                if ui.button("Shrink (-)").clicked() {
                                    actions.trigger_shrink_selection = true;
                                }
                                if state.sub_element_level == SubElementLevel::Edge {
                                    if ui.button("Loop (Alt+L)").clicked() {
                                        actions.trigger_edge_loop = true;
                                    }
                                    if ui.button("Ring (Alt+R)").clicked() {
                                        actions.trigger_edge_ring = true;
                                    }
                                    if ui.button("Crease (Shift+E)").clicked() {
                                        if let NodeData::Mesh { base_mesh, .. } = &node.data {
                                            let edges: Vec<EdgeKey> = state.selected_edges.iter().copied().collect();
                                            if !edges.is_empty() {
                                                state.edge_crease_modal_edges = edges.clone();
                                                state.edge_crease_modal_original_weights = edges
                                                    .iter()
                                                    .map(|&ek| (ek, base_mesh.get_edge_crease(ek)))
                                                    .collect();
                                                state.edge_crease_modal_weight = state
                                                    .edge_crease_modal_original_weights
                                                    .first()
                                                    .map(|&(_, w)| w)
                                                    .unwrap_or(0.0);
                                                state.edge_crease_modal_start_mouse = state.last_mouse_pos.unwrap_or((0.0, 0.0));
                                                state.edge_crease_modal_active = true;
                                            }
                                        }
                                    }
                                    if ui.button("Clear Crease").clicked() {
                                        actions.trigger_clear_crease = true;
                                    }
                                }
                            });

                            ui.separator();
                            ui.heading("Soft Selection / Proportional (O)");
                            ui.checkbox(&mut state.proportional_editing, "Enabled (O)");
                            if state.proportional_editing {
                                ui.add(egui::Slider::new(&mut state.proportional_radius, 0.2..=10.0).text("Radius"));
                            }
                            ui.separator();

                            ui.heading("Direct Operators (BMesh)");
                            ui.add(egui::Slider::new(&mut state.extrude_distance, 0.1..=3.0).text("Extrude Distance"));
                            if ui.button("Extrude Face (Ctrl+E)").clicked() {
                                actions.trigger_extrude = true;
                            }

                            ui.add_space(5.0);
                            ui.add(egui::Slider::new(&mut state.inset_factor, 0.05..=0.95).text("Inset Factor"));
                            ui.checkbox(&mut state.inset_individual, "Individual Faces (I)");
                            ui.horizontal(|ui| {
                                if ui.button("Inset Faces (I)").clicked() {
                                    actions.trigger_inset = true;
                                }
                                if ui.button("Flip Normals (Alt+N)").clicked() {
                                    actions.trigger_flip_normals = true;
                                }
                            });

                            ui.add_space(5.0);
                            ui.add(egui::Slider::new(&mut state.bevel_offset, 0.05..=1.5).text("Bevel Offset"));
                            ui.add(egui::Slider::new(&mut state.bevel_inset, 0.05..=1.0).text("Bevel Inset"));
                            if ui.button("Bevel Face (B)").clicked() {
                                actions.trigger_bevel = true;
                            }

                            ui.add_space(5.0);
                            ui.collapsing("Bisect / Slice Plane (3ds Max / Blender)", |ui| {
                                ui.horizontal(|ui| {
                                    ui.label("Plane:");
                                    ui.radio_value(&mut state.bisect_plane_axis, 0, "YZ");
                                    ui.radio_value(&mut state.bisect_plane_axis, 1, "XZ");
                                    ui.radio_value(&mut state.bisect_plane_axis, 2, "XY");
                                });
                                ui.horizontal(|ui| {
                                    ui.checkbox(&mut state.bisect_clear_positive, "Clear (+)");
                                    ui.checkbox(&mut state.bisect_clear_negative, "Clear (-)");
                                    ui.checkbox(&mut state.bisect_fill_cut, "Fill Cap");
                                });
                                if ui.button("Execute Bisect Cut").clicked() {
                                    actions.trigger_bisect = true;
                                }
                            });

                            ui.add_space(5.0);
                            ui.add(egui::Slider::new(&mut state.loop_cut_ratio, 0.1..=0.9).text("Cut Ratio"));
                            ui.horizontal(|ui| {
                                if ui.button("Split Quad").clicked() {
                                    actions.trigger_split_quad = true;
                                }
                                if ui.button("Loop Cut Ring (Ctrl+R)").clicked() {
                                    actions.trigger_loop_cut = true;
                                }
                                if ui.button("Poke Face (Alt+P)").clicked() {
                                    actions.trigger_poke_face = true;
                                }
                                if ui.button("Separate (P)").clicked() {
                                    actions.trigger_separate_faces = true;
                                }
                            });

                            ui.add_space(5.0);
                            ui.horizontal(|ui| {
                                if ui.button("Connect Vertices (J)").clicked() {
                                    actions.trigger_connect_vertices = true;
                                }
                                if ui.button("Bevel Edge").clicked() {
                                    actions.trigger_bevel_edge = true;
                                }
                                if ui.button("Subdivide Edge").clicked() {
                                    actions.trigger_split_edge = true;
                                }
                                if ui.button("Dissolve Edge (X)").clicked() {
                                    actions.trigger_dissolve_edge = true;
                                }
                            });

                            ui.add_space(5.0);
                            ui.collapsing("Triangulate Faces (Ctrl+T)", |ui| {
                                ui.horizontal(|ui| {
                                    ui.label("Quad:");
                                    egui::ComboBox::from_id_salt("edit_tri_quad")
                                        .selected_text(format!("{:?}", state.triangulate_quad_method))
                                        .show_ui(ui, |ui| {
                                            ui.selectable_value(&mut state.triangulate_quad_method, QuadMethod::Beauty, "Beauty");
                                            ui.selectable_value(&mut state.triangulate_quad_method, QuadMethod::ShortestDiagonal, "Shortest Diagonal");
                                            ui.selectable_value(&mut state.triangulate_quad_method, QuadMethod::Fixed, "Fixed");
                                            ui.selectable_value(&mut state.triangulate_quad_method, QuadMethod::Alternate, "Alternate");
                                        });
                                });
                                ui.horizontal(|ui| {
                                    ui.label("N-gon:");
                                    egui::ComboBox::from_id_salt("edit_tri_ngon")
                                        .selected_text(format!("{:?}", state.triangulate_ngon_method))
                                        .show_ui(ui, |ui| {
                                            ui.selectable_value(&mut state.triangulate_ngon_method, NgonMethod::EarClipping, "Ear Clipping");
                                            ui.selectable_value(&mut state.triangulate_ngon_method, NgonMethod::Fan, "Fan");
                                        });
                                });
                                if ui.button("Execute Triangulate (Ctrl+T)").clicked() {
                                    actions.trigger_triangulate = true;
                                }
                            });

                            ui.add_space(5.0);
                            ui.horizontal(|ui| {
                                ui.add(egui::Slider::new(&mut state.vertex_bevel_offset, 0.05..=1.0).text("V-Offset"));
                                if ui.button("Bevel Vertex (Ctrl+Shift+B)").clicked() {
                                    actions.trigger_bevel_vertex = true;
                                }
                            });

                            ui.add_space(5.0);
                            ui.horizontal(|ui| {
                                if ui.button("Merge at Center (M)").clicked() {
                                    actions.trigger_merge_vertices = true;
                                }
                                ui.checkbox(&mut state.x_mirror_editing, "X-Mirror Symmetry");
                            });

                            ui.add_space(5.0);
                            ui.horizontal(|ui| {
                                ui.add(egui::Slider::new(&mut state.edge_slide_factor, -1.0..=1.0).text("Slide Factor"));
                                if ui.button("Slide (G, G)").clicked() {
                                    actions.trigger_slide_edges = true;
                                }
                            });

                            ui.add_space(5.0);
                            ui.horizontal(|ui| {
                                if ui.button("Mark Sharp (Ctrl+E)").clicked() {
                                    actions.trigger_mark_sharp = true;
                                }
                                if ui.button("Clear Sharp").clicked() {
                                    actions.trigger_clear_sharp = true;
                                }
                            });

                            ui.add_space(5.0);
                            ui.collapsing("Symmetrize Mesh (Blender / Maya)", |ui| {
                                ui.horizontal(|ui| {
                                    ui.label("Axis:");
                                    ui.selectable_value(&mut state.symmetrize_axis, MirrorAxis::X, "X");
                                    ui.selectable_value(&mut state.symmetrize_axis, MirrorAxis::Y, "Y");
                                    ui.selectable_value(&mut state.symmetrize_axis, MirrorAxis::Z, "Z");
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Direction:");
                                    ui.selectable_value(&mut state.symmetrize_direction, SymmetrizeDirection::PositiveToNegative, "+ to -");
                                    ui.selectable_value(&mut state.symmetrize_direction, SymmetrizeDirection::NegativeToPositive, "- to +");
                                });
                                if ui.button("Execute Symmetrize").clicked() {
                                    actions.trigger_symmetrize = true;
                                }
                            });

                            ui.add_space(5.0);
                            ui.collapsing("Spin Revolve (Lathe)", |ui| {
                                ui.add(egui::Slider::new(&mut state.spin_angle_degrees, 15.0..=360.0).text("Angle (deg)"));
                                ui.add(egui::Slider::new(&mut state.spin_steps, 3..=64).text("Steps"));
                                if ui.button("Execute Spin").clicked() {
                                    actions.trigger_spin = true;
                                }
                            });

                            ui.add_space(5.0);
                            ui.collapsing("Mesh Cleanup", |ui| {
                                ui.add(egui::Slider::new(&mut state.clean_min_edge_len, 0.0001..=0.1).text("Min Edge Len"));
                                ui.add(egui::Slider::new(&mut state.clean_min_area, 1e-6..=0.01).text("Min Face Area"));
                                if ui.button("Clean Degenerate Geometry").clicked() {
                                    actions.trigger_clean_degenerate = true;
                                }
                            });

                            ui.add_space(5.0);
                            ui.horizontal(|ui| {
                                if ui.button("Separate Loose Parts").clicked() {
                                    actions.trigger_separate_loose_parts = true;
                                }
                                if ui.button("Separate by Material").clicked() {
                                    actions.trigger_separate_by_material = true;
                                }
                            });

                            ui.add_space(5.0);
                            ui.collapsing("Hole Filling & Grid Patch", |ui| {
                                ui.horizontal(|ui| {
                                    ui.checkbox(&mut state.cap_holes_triangulate, "Triangulate Caps");
                                    if ui.button("Cap Holes").clicked() {
                                        actions.trigger_cap_holes = true;
                                    }
                                });
                                ui.horizontal(|ui| {
                                    ui.add(egui::Slider::new(&mut state.grid_fill_span_count, 1..=16).text("Span"));
                                    if ui.button("Grid Fill").clicked() {
                                        actions.trigger_grid_fill = true;
                                    }
                                });
                            });

                            if state.selected_vertices.len() >= 2 {
                                ui.add_space(5.0);
                                if ui.button("Pick Shortest Path").clicked() {
                                    actions.trigger_shortest_path = true;
                                }
                            }

                            if let NodeData::Mesh { base_mesh, .. } = &mut node.data {
                                if state.sub_element_level == SubElementLevel::Edge && !state.selected_edges.is_empty() {
                                    ui.group(|ui| {
                                        ui.label("Edge Creasing (OpenSubdiv):");
                                        let mut crease_val = 0.0_f32;
                                        if let Some(&first_ek) = state.selected_edges.iter().next() {
                                            if let Some(edge) = base_mesh.edges.get(first_ek) {
                                                crease_val = edge.crease_weight;
                                            }
                                        }
                                        if ui.add(egui::Slider::new(&mut crease_val, 0.0..=1.0).text("Crease Weight")).changed() {
                                            for &ek in &state.selected_edges {
                                                base_mesh.set_edge_crease(ek, crease_val);
                                            }
                                            node.is_dirty = true;
                                        }
                                        ui.horizontal(|ui| {
                                            if ui.button("Max Crease (1.0)").clicked() {
                                                for &ek in &state.selected_edges {
                                                    base_mesh.set_edge_crease(ek, 1.0);
                                                }
                                                node.is_dirty = true;
                                            }
                                            if ui.button("Clear Crease (0.0)").clicked() {
                                                for &ek in &state.selected_edges {
                                                    base_mesh.set_edge_crease(ek, 0.0);
                                                }
                                                node.is_dirty = true;
                                            }
                                            if ui.button("Bridge Loops").clicked() {
                                                actions.trigger_bridge_edge_loops = true;
                                            }
                                        });
                                    });
                                }

                                ui.separator();

                                ui.collapsing("2D UV Texture Coordinate Editor", |ui| {
                                    ui.horizontal(|ui| {
                                        if ui.button("Planar (Y)").clicked() {
                                            actions.trigger_planar_uv = true;
                                        }
                                        if ui.button("Box (Tri-planar)").clicked() {
                                            actions.trigger_box_uv = true;
                                        }
                                        if ui.button("Cylindrical").clicked() {
                                            actions.trigger_cylindrical_uv = true;
                                        }
                                        if ui.button("Spherical").clicked() {
                                            actions.trigger_spherical_uv = true;
                                        }
                                    });

                                    ui.separator();
                                    ui.label("Interactive 2D UV Space [0, 1]:");
                                    let (response, painter) = ui.allocate_painter(egui::Vec2::new(240.0, 240.0), egui::Sense::hover());
                                    let rect = response.rect;
                                    painter.rect_filled(rect, 4.0, egui::Color32::from_rgb(25, 27, 32));
                                    painter.rect_stroke(rect, 4.0, egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(70, 75, 85)));

                                    for step in 1..4 {
                                        let frac = step as f32 * 0.25;
                                        let x = rect.left() + frac * rect.width();
                                        let y = rect.top() + frac * rect.height();
                                        painter.line_segment([egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())], egui::Stroke::new(0.5_f32, egui::Color32::from_rgb(45, 48, 55)));
                                        painter.line_segment([egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)], egui::Stroke::new(0.5_f32, egui::Color32::from_rgb(45, 48, 55)));
                                    }

                                    let uv_stroke = egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(100, 200, 255));
                                    for fk in base_mesh.faces.keys() {
                                        let vks = base_mesh.face_vertices(fk);
                                        let pts: Vec<egui::Pos2> = vks.iter().filter_map(|&vk| {
                                            base_mesh.vertices.get(vk).map(|v| {
                                                let u = v.uv.x.clamp(0.0, 1.0);
                                                let v_coord = 1.0 - v.uv.y.clamp(0.0, 1.0);
                                                egui::pos2(rect.left() + u * rect.width(), rect.top() + v_coord * rect.height())
                                            })
                                        }).collect();

                                        if pts.len() >= 3 {
                                            for i in 0..pts.len() {
                                                painter.line_segment([pts[i], pts[(i + 1) % pts.len()]], uv_stroke);
                                            }
                                        }
                                    }

                                    ui.horizontal(|ui| {
                                        if ui.button("Normalize [0, 1]").clicked() {
                                            let mut min_u = f32::INFINITY;
                                            let mut max_u = f32::NEG_INFINITY;
                                            let mut min_v = f32::INFINITY;
                                            let mut max_v = f32::NEG_INFINITY;
                                            for v in base_mesh.vertices.values() {
                                                min_u = min_u.min(v.uv.x);
                                                max_u = max_u.max(v.uv.x);
                                                min_v = min_v.min(v.uv.y);
                                                max_v = max_v.max(v.uv.y);
                                            }
                                            let range_u = (max_u - min_u).max(1e-4);
                                            let range_v = (max_v - min_v).max(1e-4);
                                            for v in base_mesh.vertices.values_mut() {
                                                v.uv.x = (v.uv.x - min_u) / range_u;
                                                v.uv.y = (v.uv.y - min_v) / range_v;
                                            }
                                            node.is_dirty = true;
                                        }
                                        if ui.button("Flip V").clicked() {
                                            for v in base_mesh.vertices.values_mut() {
                                                v.uv.y = 1.0 - v.uv.y;
                                            }
                                            node.is_dirty = true;
                                        }
                                    });
                                });
                            }

                            ui.separator();
                        }

                        if state.interaction_mode == InteractionMode::Pose {
                            ui.heading("Pose Mode (Rigging)");
                            if matches!(node.data, NodeData::Joint { .. }) {
                                ui.label(egui::RichText::new(format!("Active Pose Bone: {}", node.name)).strong().color(egui::Color32::from_rgb(80, 240, 200)));
                                ui.horizontal(|ui| {
                                    if ui.button("Clear Rotation (Alt+R)").clicked() {
                                        node.local_transform.rotation = core_math::Quat::IDENTITY;
                                        node.is_dirty = true;
                                    }
                                    if ui.button("Clear Translation (Alt+G)").clicked() {
                                        node.local_transform.translation = core_math::Vec3::ZERO;
                                        node.is_dirty = true;
                                    }
                                });
                                if ui.button("Keyframe Pose (I)").clicked() {
                                    let frame = state.animation.current_frame.round();
                                    state.animation.insert_keyframe(sel_id, frame, node.local_transform, InterpolationMode::Linear);
                                }
                            } else {
                                ui.label(egui::RichText::new("Select a bone to pose and keyframe").weak());
                            }
                            ui.separator();
                        }

                        if state.interaction_mode == InteractionMode::Sculpt {
                            super::sculpt::draw_sculpt_tools(
                                ui,
                                &mut state.sculpt_settings,
                                &mut state.dyntopo,
                                &mut state.sculpt_symmetry_x,
                                &mut state.sculpt_symmetry_y,
                                &mut state.sculpt_symmetry_z,
                                actions,
                            );
                        }

                        if state.interaction_mode == InteractionMode::WeightPaint {
                            ui.heading("Weight Paint Tools");

                            if let NodeData::Mesh { base_mesh, .. } = &node.data {
                                let groups: Vec<String> = base_mesh.vertex_groups.iter().map(|vg| vg.name.clone()).collect();
                                if !groups.is_empty() {
                                    if state.active_vertex_group.is_none() {
                                        state.active_vertex_group = Some(groups[0].clone());
                                    }
                                    let active_text = state.active_vertex_group.as_deref().unwrap_or("None");
                                    let mut group_changed = false;
                                    egui::ComboBox::from_label("Active Group")
                                        .selected_text(active_text)
                                        .show_ui(ui, |ui| {
                                            for g in &groups {
                                                if ui.selectable_value(&mut state.active_vertex_group, Some(g.clone()), g).clicked() {
                                                    group_changed = true;
                                                }
                                            }
                                        });
                                    if group_changed {
                                        node.is_dirty = true;
                                    }
                                }
                            }

                            ui.horizontal(|ui| {
                                ui.selectable_value(&mut state.weight_paint_tool, WeightPaintTool::Brush, "Brush");
                                ui.selectable_value(&mut state.weight_paint_tool, WeightPaintTool::LinearGradient, "Linear Gradient");
                            });

                            if state.weight_paint_tool == WeightPaintTool::Brush {
                                ui.add(egui::Slider::new(&mut state.weight_brush_target, 0.0..=1.0).text("Weight"));
                                ui.horizontal(|ui| {
                                    if ui.button("0.0 (Clear)").clicked() { state.weight_brush_target = 0.0; }
                                    if ui.button("0.25").clicked() { state.weight_brush_target = 0.25; }
                                    if ui.button("0.5").clicked() { state.weight_brush_target = 0.5; }
                                    if ui.button("0.75").clicked() { state.weight_brush_target = 0.75; }
                                    if ui.button("1.0 (Full)").clicked() { state.weight_brush_target = 1.0; }
                                });

                                ui.add(egui::Slider::new(&mut state.weight_brush_radius, 0.05..=3.0).text("Radius"));
                                ui.add(egui::Slider::new(&mut state.weight_brush_strength, 0.01..=1.0).text("Strength"));

                                egui::ComboBox::from_id_salt("wp_falloff_combo")
                                    .selected_text(format!("{:?}", state.weight_brush_falloff))
                                    .show_ui(ui, |ui| {
                                        ui.selectable_value(&mut state.weight_brush_falloff, BrushFalloff::Smooth, "Smooth");
                                        ui.selectable_value(&mut state.weight_brush_falloff, BrushFalloff::Spherical, "Spherical");
                                        ui.selectable_value(&mut state.weight_brush_falloff, BrushFalloff::Linear, "Linear");
                                        ui.selectable_value(&mut state.weight_brush_falloff, BrushFalloff::Sharp, "Sharp");
                                    });
                            } else {
                                ui.label("Drag across viewport mesh to apply 3D linear gradient:");
                                ui.add(egui::Slider::new(&mut state.weight_gradient_start_w, 0.0..=1.0).text("Start Weight"));
                                ui.add(egui::Slider::new(&mut state.weight_gradient_end_w, 0.0..=1.0).text("End Weight"));
                                ui.horizontal(|ui| {
                                    if ui.button("1.0 -> 0.0 (Fade Out)").clicked() {
                                        state.weight_gradient_start_w = 1.0;
                                        state.weight_gradient_end_w = 0.0;
                                    }
                                    if ui.button("0.0 -> 1.0 (Fade In)").clicked() {
                                        state.weight_gradient_start_w = 0.0;
                                        state.weight_gradient_end_w = 1.0;
                                    }
                                });
                            }

                            ui.separator();
                        }

                        if state.interaction_mode == InteractionMode::VertexPaint {
                            super::paint::draw_vertex_paint_tools(ui, state, actions);
                        }

                        if state.interaction_mode == InteractionMode::TexturePaint {
                            super::paint::draw_texture_paint_tools(ui, state);
                        }
    }
}

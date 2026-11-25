use crate::state::AppState;
use crate::types::*;
use crate::ui::actions::UiActions;
use core_math::{vec3, Vec3};
use geo_kernel::{BezierPoint, ProfileShape};
use graph_engine::{LightType, NodeData, NodeId};

pub fn draw_scene_nodes_panel(
    ui: &mut egui::Ui,
    state: &mut AppState,
    actions: &mut UiActions,
    sel_id: NodeId,
    available_mesh_ids: &[(NodeId, String)],
) {
    let camera_aspect = state.camera.aspect;
    let mut active_cam = state.active_camera_node;
    let mut interaction_mode = state.interaction_mode;
    let mut curve_knot = state.selected_curve_knot;
    let mut curve_handle = state.selected_curve_handle;

    if let Some(node) = state.scene.nodes.get_mut(sel_id) {
        match &mut node.data {
                            NodeData::Light { light_type, color, intensity, range, spot_inner_angle_radians, spot_outer_angle_radians } => {
                                ui.collapsing("Light Properties", |ui| {
                                    ui.horizontal(|ui| {
                                        ui.label("Type:");
                                        egui::ComboBox::from_id_salt("inspector_light_type")
                                            .selected_text(match light_type {
                                                LightType::Directional => "Directional (Sun)",
                                                LightType::Point => "Point (Omni)",
                                                LightType::Spot => "Spot (Cone)",
                                            })
                                            .show_ui(ui, |ui| {
                                                ui.selectable_value(light_type, LightType::Directional, "Directional");
                                                ui.selectable_value(light_type, LightType::Point, "Point");
                                                ui.selectable_value(light_type, LightType::Spot, "Spot");
                                            });
                                    });

                                    let mut rgb = [color.x, color.y, color.z];
                                    ui.horizontal(|ui| {
                                        ui.label("Color:");
                                        if ui.color_edit_button_rgb(&mut rgb).changed() {
                                            *color = vec3(rgb[0], rgb[1], rgb[2]);
                                        }
                                    });

                                    ui.horizontal(|ui| {
                                        ui.label("Intensity:");
                                        ui.add(egui::Slider::new(intensity, 0.0..=50.0).step_by(0.1));
                                    });

                                    if *light_type != LightType::Directional {
                                        ui.horizontal(|ui| {
                                            ui.label("Range:");
                                            ui.add(egui::Slider::new(range, 0.5..=100.0).step_by(0.5));
                                        });
                                    }

                                    if *light_type == LightType::Spot {
                                        let mut inner_deg = spot_inner_angle_radians.to_degrees();
                                        let mut outer_deg = spot_outer_angle_radians.to_degrees();
                                        ui.horizontal(|ui| {
                                            ui.label("Inner Cone:");
                                            if ui.add(egui::Slider::new(&mut inner_deg, 1.0..=85.0).suffix("°")).changed() {
                                                if inner_deg > outer_deg { outer_deg = inner_deg; }
                                                *spot_inner_angle_radians = inner_deg.to_radians();
                                                *spot_outer_angle_radians = outer_deg.to_radians();
                                            }
                                        });
                                        ui.horizontal(|ui| {
                                            ui.label("Outer Cone:");
                                            if ui.add(egui::Slider::new(&mut outer_deg, 1.0..=89.0).suffix("°")).changed() {
                                                if outer_deg < inner_deg { inner_deg = outer_deg; }
                                                *spot_inner_angle_radians = inner_deg.to_radians();
                                                *spot_outer_angle_radians = outer_deg.to_radians();
                                            }
                                        });
                                    }
                                });
                                ui.separator();
                            }
                            NodeData::Camera { fov_y_radians, z_near, z_far, aspect_ratio, orthographic, ortho_size } => {
                                ui.collapsing("Camera Actor", |ui| {
                                    let is_active_view = active_cam == Some(sel_id);
                                    let btn_text = if is_active_view { "Exit Camera View (Free Orbit)" } else { "Look Through Camera" };
                                    if ui.button(btn_text).clicked() {
                                        if is_active_view {
                                            active_cam = None;
                                        } else {
                                            active_cam = Some(sel_id);
                                        }
                                    }

                                    ui.separator();
                                    ui.checkbox(orthographic, "Orthographic Projection");

                                    if *orthographic {
                                        ui.horizontal(|ui| {
                                            ui.label("Ortho Size:");
                                            ui.add(egui::Slider::new(ortho_size, 0.5..=50.0).step_by(0.2));
                                        });
                                    } else {
                                        let mut fov_deg = fov_y_radians.to_degrees();
                                        ui.horizontal(|ui| {
                                            ui.label("FOV:");
                                            if ui.add(egui::Slider::new(&mut fov_deg, 10.0..=120.0).suffix("°")).changed() {
                                                *fov_y_radians = fov_deg.to_radians();
                                            }
                                        });
                                    }

                                    ui.horizontal(|ui| {
                                        ui.label("Near Clip:");
                                        ui.add(egui::DragValue::new(z_near).speed(0.05).range(0.01..=10.0));
                                    });
                                    ui.horizontal(|ui| {
                                        ui.label("Far Clip:");
                                        ui.add(egui::DragValue::new(z_far).speed(10.0).range(10.0..=50000.0));
                                    });
                                    *aspect_ratio = camera_aspect;
                                });
                                ui.separator();
                            }
                            NodeData::Joint { length, radius } => {
                                ui.collapsing("Skeleton Joint / Bone", |ui| {
                                    ui.horizontal(|ui| {
                                        ui.label("Bone Length:");
                                        ui.add(egui::DragValue::new(length).speed(0.05).range(0.05..=50.0));
                                    });
                                    ui.horizontal(|ui| {
                                        ui.label("Bone Radius:");
                                        ui.add(egui::DragValue::new(radius).speed(0.01).range(0.01..=5.0));
                                    });

                                    ui.separator();
                                    if ui.button("➕ Extrude Child Bone").clicked() {
                                        actions.add_child_joint = Some((sel_id, *length));
                                    }

                                    if !available_mesh_ids.is_empty() {
                                        ui.separator();
                                        ui.label("Armature Auto-Skinning:");
                                        for (mesh_id, mesh_name) in available_mesh_ids {
                                            if ui.button(format!("Bind '{}' to Armature", mesh_name)).clicked() {
                                                actions.bind_armature_request = Some((*mesh_id, sel_id));
                                            }
                                        }
                                    }
                                });
                                ui.separator();
                            }
                            NodeData::Curve { spline, bevel_radius, bevel_resolution, cached_render_mesh, profile_shape } => {
                                ui.collapsing("Bézier Curve / Spline", |ui| {
                                    let is_curve_edit = interaction_mode == InteractionMode::CurveEdit;
                                    let btn_text = if is_curve_edit {
                                        egui::RichText::new("Exit Curve Edit (Tab)").color(egui::Color32::from_rgb(100, 220, 255)).strong()
                                    } else {
                                        egui::RichText::new("✏️ Curve Edit Mode (Tab)").strong()
                                    };
                                    ui.horizontal(|ui| {
                                        if ui.button(btn_text).clicked() {
                                            interaction_mode = if is_curve_edit {
                                                InteractionMode::Object
                                            } else {
                                                InteractionMode::CurveEdit
                                            };
                                            node.is_dirty = true;
                                        }
                                        if ui.button("Convert to Mesh").clicked() {
                                            actions.trigger_convert_curve = true;
                                        }
                                    });
                                    ui.separator();

                                    ui.horizontal(|ui| {
                                        ui.label("Profile Shape:");
                                        egui::ComboBox::from_id_salt("curve_profile_shape")
                                            .selected_text(match profile_shape {
                                                ProfileShape::Circle => "Circle (Pipe)",
                                                ProfileShape::Square => "Square (Duct)",
                                                ProfileShape::Rectangle => "Rectangle (Molding)",
                                                ProfileShape::Star => "Star",
                                                ProfileShape::Triangle => "Triangle",
                                                ProfileShape::Hexagon => "Hexagon",
                                            })
                                            .show_ui(ui, |ui| {
                                                let mut changed = false;
                                                changed |= ui.selectable_value(profile_shape, ProfileShape::Circle, "Circle (Pipe)").changed();
                                                changed |= ui.selectable_value(profile_shape, ProfileShape::Square, "Square (Duct)").changed();
                                                changed |= ui.selectable_value(profile_shape, ProfileShape::Rectangle, "Rectangle (Molding)").changed();
                                                changed |= ui.selectable_value(profile_shape, ProfileShape::Star, "Star").changed();
                                                changed |= ui.selectable_value(profile_shape, ProfileShape::Triangle, "Triangle").changed();
                                                changed |= ui.selectable_value(profile_shape, ProfileShape::Hexagon, "Hexagon").changed();
                                                if changed {
                                                    *cached_render_mesh = None;
                                                }
                                            });
                                    });

                                    ui.horizontal(|ui| {
                                        ui.label("Bevel Radius:");
                                        if ui.add(egui::DragValue::new(bevel_radius).speed(0.01).range(0.0..=10.0)).changed() {
                                            *cached_render_mesh = None;
                                        }
                                    });
                                    ui.horizontal(|ui| {
                                        ui.label("Bevel Resolution:");
                                        if ui.add(egui::DragValue::new(bevel_resolution).speed(0.2).range(3..=64)).changed() {
                                            *cached_render_mesh = None;
                                        }
                                    });
                                    if ui.checkbox(&mut spline.is_closed, "Cyclic (Closed Curve)").changed() {
                                        *cached_render_mesh = None;
                                    }

                                    let total_len = spline.arc_length(16);
                                    ui.label(format!("Total Arc Length: {:.3} units", total_len));

                                    ui.separator();
                                    ui.horizontal(|ui| {
                                        let mut fillet_r = ui.data_mut(|d| *d.get_temp_mut_or_insert_with(egui::Id::new("fillet_radius_val"), || 0.2_f32));
                                        ui.label("Fillet R:");
                                        ui.add(egui::DragValue::new(&mut fillet_r).speed(0.01).range(0.01..=10.0));
                                        ui.data_mut(|d| d.insert_temp(egui::Id::new("fillet_radius_val"), fillet_r));
                                        if ui.button("✨ Fillet Corners").on_hover_text("Rounds sharp corners of the Bézier curve with precision CAD circular G1-continuous fillets").clicked() {
                                            actions.trigger_fillet_curve = Some(fillet_r);
                                        }
                                    });
                                    ui.horizontal(|ui| {
                                        let mut lathe_segs = ui.data_mut(|d| *d.get_temp_mut_or_insert_with(egui::Id::new("lathe_segs_val"), || 16_u32));
                                        let mut lathe_ang = ui.data_mut(|d| *d.get_temp_mut_or_insert_with(egui::Id::new("lathe_ang_val"), || 360.0_f32));
                                        ui.label("Lathe:");
                                        ui.add(egui::DragValue::new(&mut lathe_segs).speed(0.2).range(3..=128).suffix(" segs"));
                                        ui.add(egui::DragValue::new(&mut lathe_ang).speed(1.0).range(1.0..=360.0).suffix("°"));
                                        ui.data_mut(|d| {
                                            d.insert_temp(egui::Id::new("lathe_segs_val"), lathe_segs);
                                            d.insert_temp(egui::Id::new("lathe_ang_val"), lathe_ang);
                                        });
                                        if ui.button("🏺 Revolve to Mesh").on_hover_text("Revolves this curve profile around the Y-axis into a 3D solid rotational mesh").clicked() {
                                            actions.trigger_lathe_curve = Some((lathe_segs, lathe_ang));
                                        }
                                    });

                                    ui.separator();
                                    let points_count = spline.points.len();
                                    ui.label(format!("Knots ({} points):", points_count));

                                    let mut knot_to_remove = None;
                                    let mut knot_to_subdivide = None;

                                    if let Some(sel_k) = curve_knot {
                                        if let Some(pt) = spline.points.get_mut(sel_k) {
                                            ui.group(|ui| {
                                                ui.label(egui::RichText::new(format!("Selected Knot #{}", sel_k)).strong().color(egui::Color32::from_rgb(255, 215, 0)));
                                                ui.horizontal(|ui| {
                                                    ui.label("Position:");
                                                    if ui.add(egui::DragValue::new(&mut pt.position.x).speed(0.05).prefix("X: ")).changed()
                                                        || ui.add(egui::DragValue::new(&mut pt.position.y).speed(0.05).prefix("Y: ")).changed()
                                                        || ui.add(egui::DragValue::new(&mut pt.position.z).speed(0.05).prefix("Z: ")).changed()
                                                    {
                                                        *cached_render_mesh = None;
                                                    }
                                                });
                                                ui.horizontal(|ui| {
                                                    ui.label("Handle In:");
                                                    if ui.add(egui::DragValue::new(&mut pt.handle_in.x).speed(0.05).prefix("X: ")).changed()
                                                        || ui.add(egui::DragValue::new(&mut pt.handle_in.y).speed(0.05).prefix("Y: ")).changed()
                                                        || ui.add(egui::DragValue::new(&mut pt.handle_in.z).speed(0.05).prefix("Z: ")).changed()
                                                    {
                                                        *cached_render_mesh = None;
                                                    }
                                                });
                                                ui.horizontal(|ui| {
                                                    ui.label("Handle Out:");
                                                    if ui.add(egui::DragValue::new(&mut pt.handle_out.x).speed(0.05).prefix("X: ")).changed()
                                                        || ui.add(egui::DragValue::new(&mut pt.handle_out.y).speed(0.05).prefix("Y: ")).changed()
                                                        || ui.add(egui::DragValue::new(&mut pt.handle_out.z).speed(0.05).prefix("Z: ")).changed()
                                                    {
                                                        *cached_render_mesh = None;
                                                    }
                                                });

                                                ui.horizontal(|ui| {
                                                    if ui.button("Set Sharp").on_hover_text("Reset tangent handles to zero").clicked() {
                                                        pt.handle_in = Vec3::ZERO;
                                                        pt.handle_out = Vec3::ZERO;
                                                        *cached_render_mesh = None;
                                                    }
                                                    if ui.button("Set Smooth").on_hover_text("Align handles tangentially").clicked() {
                                                        let tan = if pt.handle_out.length_squared() > 1e-4 {
                                                            pt.handle_out
                                                        } else {
                                                            Vec3::new(0.5, 0.0, 0.0)
                                                        };
                                                        pt.handle_out = tan;
                                                        pt.handle_in = -tan;
                                                        *cached_render_mesh = None;
                                                    }
                                                    if ui.button("Subdivide").on_hover_text("Insert knot between this and next").clicked() {
                                                        knot_to_subdivide = Some(sel_k);
                                                    }
                                                    if points_count > 2
                                                        && ui.button("Delete (X)").clicked() {
                                                            knot_to_remove = Some(sel_k);
                                                        }
                                                });
                                            });
                                        }
                                    }

                                    if let Some(rem_k) = knot_to_remove {
                                        spline.points.remove(rem_k);
                                        curve_knot = Some(rem_k.min(spline.points.len() - 1));
                                        *cached_render_mesh = None;
                                    }

                                    if let Some(sub_k) = knot_to_subdivide {
                                        if let Ok(ins_idx) = spline.subdivide_segment(sub_k, 0.5) {
                                            curve_knot = Some(ins_idx);
                                            *cached_render_mesh = None;
                                        }
                                    }

                                    ui.horizontal(|ui| {
                                        for ki in 0..spline.points.len() {
                                            let is_sel = curve_knot == Some(ki);
                                            if ui.selectable_label(is_sel, format!("#{}", ki)).clicked() {
                                                curve_knot = Some(ki);
                                                curve_handle = Some(CurveHandleSelection::Knot);
                                            }
                                        }
                                    });

                                    ui.horizontal(|ui| {
                                        if ui.button("➕ Extrude Knot (E)").clicked() {
                                            let cur_idx = curve_knot.unwrap_or_else(|| spline.points.len().saturating_sub(1));
                                            let cur_pt = spline.points[cur_idx];
                                            let dir = if cur_pt.handle_out.length_squared() > 1e-4 {
                                                cur_pt.handle_out.normalize()
                                            } else {
                                                Vec3::X
                                            };
                                            let new_pos = cur_pt.position + dir * 1.5;
                                            let new_pt = BezierPoint::smooth(new_pos, dir * 0.5);
                                            let ins_idx = (cur_idx + 1).min(spline.points.len());
                                            spline.points.insert(ins_idx, new_pt);
                                            curve_knot = Some(ins_idx);
                                            curve_handle = Some(CurveHandleSelection::Knot);
                                            *cached_render_mesh = None;
                                        }
                                        if ui.button("🔄 Reverse Direction").clicked() {
                                            spline.reverse();
                                            if let Some(k) = curve_knot {
                                                curve_knot = Some(spline.points.len().saturating_sub(1) - k);
                                            }
                                            *cached_render_mesh = None;
                                        }
                                    });
                                });
                                ui.separator();
                            }
                            _ => {}
        }
    }
    state.active_camera_node = active_cam;
    state.interaction_mode = interaction_mode;
    state.selected_curve_knot = curve_knot;
    state.selected_curve_handle = curve_handle;
}

use crate::state::AppState;
use crate::ui::actions::UiActions;
use geo_kernel::*;
use graph_engine::{InterpolationMode, NodeData, NodeId};

pub fn draw_modifiers_panel(
    ui: &mut egui::Ui,
    state: &mut AppState,
    actions: &mut UiActions,
    sel_id: NodeId,
    available_mesh_ids: &[(NodeId, String)],
    available_curve_names: &[String],
) {
    let can_capture = state.selected_node.is_some() && state.previous_selected_node.is_some();
    let cur_frame = state.animation.current_frame;
    let (scene, animation) = (&mut state.scene, &mut state.animation);

    if let Some(node) = scene.nodes.get_mut(sel_id) {
        ui.heading("3ds Max Modifier Stack");
        let node_name = &node.name;

                        if let NodeData::Mesh { modifier_stack, .. } = &mut node.data {
                            ui.horizontal(|ui| {
                                ui.menu_button("➕ Add Modifier", |ui| {
                                    ui.set_min_width(180.0);
                                    ui.label(egui::RichText::new("GENERATE").small().color(egui::Color32::from_rgb(100, 180, 255)));
                                    if ui.button("🌿 Geometry Nodes").clicked() {
                                        actions.add_geometry_nodes_to_selected = true;
                                        ui.close_menu();
                                    }
                                    if ui.button("Subdivide (Catmull-Clark)").clicked() {
                                        actions.add_catmull_clark_to_selected = true;
                                        ui.close_menu();
                                    }
                                    if ui.button("Linear Subdivide").clicked() {
                                        actions.add_subdivide_to_selected = true;
                                        ui.close_menu();
                                    }
                                    if ui.button("Solidify (Thicken)").clicked() {
                                        actions.add_solidify_to_selected = true;
                                        ui.close_menu();
                                    }
                                    if ui.button("Array (Duplicate)").clicked() {
                                        actions.add_array_to_selected = true;
                                        ui.close_menu();
                                    }
                                    if ui.button("Bevel (Chamfer)").clicked() {
                                        actions.add_bevel_to_selected = true;
                                        ui.close_menu();
                                    }
                                    if ui.button("Boolean Cut (Difference)").clicked() {
                                        actions.add_boolean_diff = true;
                                        ui.close_menu();
                                    }
                                    if ui.button("Decimate (QEM Simplify)").clicked() {
                                        actions.add_decimate_to_selected = true;
                                        ui.close_menu();
                                    }
                                    if ui.button("Screw (Lathe / Revolve)").clicked() {
                                        actions.add_screw_to_selected = true;
                                        ui.close_menu();
                                    }
                                    if ui.button("Triangulate").clicked() {
                                        actions.add_triangulate_to_selected = true;
                                        ui.close_menu();
                                    }
                                    if ui.button("Wireframe").clicked() {
                                        actions.add_wireframe_to_selected = true;
                                        ui.close_menu();
                                    }
                                    if ui.button("Skin Envelope (ZSpheres)").clicked() {
                                        actions.add_skin_envelope_to_selected = true;
                                        ui.close_menu();
                                    }
                                    if ui.button("Voxel Remesh (DynaMesh)").clicked() {
                                        actions.add_remesh_to_selected = true;
                                        ui.close_menu();
                                    }

                                    ui.separator();
                                    ui.label(egui::RichText::new("DEFORM").small().color(egui::Color32::from_rgb(255, 180, 100)));
                                    if ui.button("Displace (Texture / Normal)").clicked() {
                                        actions.add_displace_to_selected = true;
                                        ui.close_menu();
                                    }
                                    if ui.button("Simple Deform (Twist/Bend)").clicked() {
                                        actions.add_simple_deform_to_selected = true;
                                        ui.close_menu();
                                    }
                                    if ui.button("Taper").clicked() {
                                        actions.add_taper_to_selected = true;
                                        ui.close_menu();
                                    }
                                    if ui.button("Smooth (Relax)").clicked() {
                                        actions.add_smooth_to_selected = true;
                                        ui.close_menu();
                                    }
                                    if ui.button("Shrinkwrap (Wrap onto mesh)").clicked() {
                                        actions.add_shrinkwrap_to_selected = true;
                                        ui.close_menu();
                                    }
                                    if ui.button("Curve Deform (Path follow)").clicked() {
                                        actions.add_curve_mod_to_selected = true;
                                        ui.close_menu();
                                    }
                                    if ui.button("Lattice (FFD Cage)").clicked() {
                                        actions.add_lattice_to_selected = true;
                                        ui.close_menu();
                                    }
                                    ui.separator();
                                    ui.label(egui::RichText::new("SIMULATION & DYNAMICS").small().color(egui::Color32::from_rgb(255, 140, 180)));
                                    if ui.button("🧵 Cloth Simulation").clicked() {
                                        actions.add_cloth_to_selected = true;
                                        ui.close_menu();
                                    }
                                    if ui.button("✨ Particle System").clicked() {
                                        actions.add_particle_system_to_selected = true;
                                        ui.close_menu();
                                    }

                                    ui.separator();
                                    ui.label(egui::RichText::new("MODIFY & NORMALS").small().color(egui::Color32::from_rgb(180, 255, 100)));
                                    if ui.button("Mirror (Symmetry)").clicked() {
                                        actions.add_mirror_to_selected = Some(MirrorAxis::X);
                                        ui.close_menu();
                                    }
                                    if ui.button("Vertex Weld (Merge by dist)").clicked() {
                                        actions.add_weld_to_selected = true;
                                        ui.close_menu();
                                    }
                                    if ui.button("Weighted Normal").clicked() {
                                        actions.add_weighted_normal_to_selected = true;
                                        ui.close_menu();
                                    }
                                    if ui.button("Morpher (Blendshapes)").clicked() {
                                        actions.add_morpher_to_selected = true;
                                        ui.close_menu();
                                    }
                                });
                            });

                            ui.separator();
                            for (i, modifier) in modifier_stack.modifiers_mut().iter_mut().enumerate() {
                                ui.group(|ui| {
                                    ui.horizontal(|ui| {
                                        let mut enabled = modifier.is_enabled();
                                        if ui.checkbox(&mut enabled, "").changed() {
                                            modifier.set_enabled(enabled);
                                        }
                                        ui.label(egui::RichText::new(format!("{}. {}", i + 1, modifier.name())).strong());
                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            if ui.small_button("✖").on_hover_text("Delete modifier").clicked() {
                                                actions.remove_modifier_idx = Some(i);
                                            }
                                            if ui.small_button("Apply").on_hover_text("Bake modifier into base mesh").clicked() {
                                                actions.apply_modifier_idx = Some((i, modifier.name().to_string()));
                                            }
                                        });
                                    });

                                    if let Some(curve_mod) = modifier.as_curve_mut() {
                                        ui.horizontal(|ui| {
                                            ui.label("Target Curve:");
                                            let cur_target = curve_mod.target_curve.as_deref().unwrap_or("None");
                                            egui::ComboBox::from_id_salt(format!("curve_mod_target_{}", i))
                                                .selected_text(cur_target)
                                                .show_ui(ui, |ui| {
                                                    for cname in available_curve_names {
                                                        if ui.selectable_label(curve_mod.target_curve.as_deref() == Some(cname.as_str()), cname).clicked() {
                                                            curve_mod.target_curve = Some(cname.clone());
                                                        }
                                                    }
                                                });
                                        });
                                    }

                                    if let Some(skin_mod) = modifier.as_skin_mut() {
                                        ui.label(format!("Bound Bones: {}", skin_mod.bones.len()));
                                    }

                                    if let Some(bool_mod) = modifier.as_boolean_mut() {
                                        ui.horizontal(|ui| {
                                            ui.label("Cutter Target:");
                                            let cur_target = bool_mod.target_node_name.as_deref().unwrap_or("None");
                                            egui::ComboBox::from_id_salt(format!("bool_mod_target_{}", i))
                                                .selected_text(cur_target)
                                                .show_ui(ui, |ui| {
                                                    for (_, mname) in available_mesh_ids {
                                                        if mname != node_name {
                                                            if ui.selectable_label(bool_mod.target_node_name.as_deref() == Some(mname.as_str()), mname).clicked() {
                                                                bool_mod.target_node_name = Some(mname.clone());
                                                            }
                                                        }
                                                    }
                                                });
                                        });
                                    }

                                    if let Some(solidify_mod) = modifier.as_solidify_mut() {
                                        ui.horizontal(|ui| {
                                            ui.label("Thickness:");
                                            ui.add(egui::DragValue::new(&mut solidify_mod.thickness).speed(0.01));
                                            ui.label("Offset:");
                                            ui.add(egui::Slider::new(&mut solidify_mod.offset, -1.0..=1.0));
                                            ui.checkbox(&mut solidify_mod.rim_only, "Rim Only");
                                        });
                                    }

                                    if let Some(array_mod) = modifier.as_array_mut() {
                                        ui.horizontal(|ui| {
                                            ui.label("Mode:");
                                            ui.selectable_value(&mut array_mod.mode, geo_kernel::ArrayMode::Linear, "Linear");
                                            ui.selectable_value(&mut array_mod.mode, geo_kernel::ArrayMode::Radial, "Radial / Spiral");
                                        });
                                        ui.horizontal(|ui| {
                                            ui.label("Count:");
                                            ui.add(egui::DragValue::new(&mut array_mod.count).speed(0.2).range(1..=64));
                                            ui.checkbox(&mut array_mod.merge, "Merge Vertices");
                                            if array_mod.merge {
                                                ui.add(egui::DragValue::new(&mut array_mod.merge_distance).speed(0.001).prefix("Dist: "));
                                            }
                                        });
                                        match array_mod.mode {
                                            geo_kernel::ArrayMode::Linear => {
                                                ui.horizontal(|ui| {
                                                    ui.label("Rel Offset:");
                                                    ui.add(egui::DragValue::new(&mut array_mod.relative_offset.x).speed(0.05).prefix("X: "));
                                                    ui.add(egui::DragValue::new(&mut array_mod.relative_offset.y).speed(0.05).prefix("Y: "));
                                                    ui.add(egui::DragValue::new(&mut array_mod.relative_offset.z).speed(0.05).prefix("Z: "));
                                                });
                                                ui.horizontal(|ui| {
                                                    ui.label("Const Offset:");
                                                    ui.add(egui::DragValue::new(&mut array_mod.constant_offset.x).speed(0.05).prefix("X: "));
                                                    ui.add(egui::DragValue::new(&mut array_mod.constant_offset.y).speed(0.05).prefix("Y: "));
                                                    ui.add(egui::DragValue::new(&mut array_mod.constant_offset.z).speed(0.05).prefix("Z: "));
                                                });
                                                ui.horizontal(|ui| {
                                                    ui.label("Scale Step:");
                                                    ui.add(egui::DragValue::new(&mut array_mod.scale_step.x).speed(0.02).prefix("X: "));
                                                    ui.add(egui::DragValue::new(&mut array_mod.scale_step.y).speed(0.02).prefix("Y: "));
                                                    ui.add(egui::DragValue::new(&mut array_mod.scale_step.z).speed(0.02).prefix("Z: "));
                                                });
                                            }
                                            geo_kernel::ArrayMode::Radial => {
                                                ui.horizontal(|ui| {
                                                    ui.label("Total Angle (°):");
                                                    ui.add(egui::DragValue::new(&mut array_mod.total_angle_degrees).speed(1.0).range(1.0..=720.0));
                                                    ui.label("Axis:");
                                                    ui.selectable_value(&mut array_mod.radial_axis, geo_kernel::RadialAxis::Y, "Y");
                                                    ui.selectable_value(&mut array_mod.radial_axis, geo_kernel::RadialAxis::X, "X");
                                                    ui.selectable_value(&mut array_mod.radial_axis, geo_kernel::RadialAxis::Z, "Z");
                                                });
                                                ui.horizontal(|ui| {
                                                    ui.label("Spiral Rise (step):");
                                                    ui.add(egui::DragValue::new(&mut array_mod.spiral_height_offset).speed(0.05));
                                                });
                                            }
                                        }
                                    }

                                    if let Some(smooth_mod) = modifier.as_smooth_mut() {
                                        ui.horizontal(|ui| {
                                            ui.label("Factor:");
                                            ui.add(egui::Slider::new(&mut smooth_mod.factor, 0.0..=1.0));
                                            ui.label("Iterations:");
                                            ui.add(egui::DragValue::new(&mut smooth_mod.iterations).range(1..=50));
                                        });
                                    }

                                    if let Some(shrink_mod) = modifier.as_shrinkwrap_mut() {
                                        ui.horizontal(|ui| {
                                            ui.label("Target Mesh:");
                                            let cur_target = shrink_mod.target_node_name.as_deref().unwrap_or("None");
                                            egui::ComboBox::from_id_salt(format!("shrink_mod_target_{}", i))
                                                .selected_text(cur_target)
                                                .show_ui(ui, |ui| {
                                                    for (_, mname) in available_mesh_ids {
                                                        if mname != node_name {
                                                            if ui.selectable_label(shrink_mod.target_node_name.as_deref() == Some(mname.as_str()), mname).clicked() {
                                                                shrink_mod.target_node_name = Some(mname.clone());
                                                            }
                                                        }
                                                    }
                                                });
                                        });
                                        ui.horizontal(|ui| {
                                            ui.label("Method:");
                                            egui::ComboBox::from_id_salt(format!("shrink_mod_method_{}", i))
                                                .selected_text(match shrink_mod.method {
                                                    ShrinkwrapMethod::NearestSurfacePoint => "Nearest Surface",
                                                    ShrinkwrapMethod::ProjectNormal => "Project Normal",
                                                })
                                                .show_ui(ui, |ui| {
                                                    ui.selectable_value(&mut shrink_mod.method, ShrinkwrapMethod::NearestSurfacePoint, "Nearest Surface");
                                                    ui.selectable_value(&mut shrink_mod.method, ShrinkwrapMethod::ProjectNormal, "Project Normal");
                                                });
                                            ui.label("Offset:");
                                            ui.add(egui::DragValue::new(&mut shrink_mod.offset).speed(0.01));
                                        });
                                    }

                                    if let Some(deform_mod) = modifier.as_simple_deform_mut() {
                                        ui.horizontal(|ui| {
                                            ui.label("Method:");
                                            egui::ComboBox::from_id_salt(format!("deform_mod_method_{}", i))
                                                .selected_text(match deform_mod.method {
                                                    SimpleDeformMethod::Twist => "Twist",
                                                    SimpleDeformMethod::Bend => "Bend",
                                                    SimpleDeformMethod::Taper => "Taper",
                                                    SimpleDeformMethod::Stretch => "Stretch",
                                                })
                                                .show_ui(ui, |ui| {
                                                    ui.selectable_value(&mut deform_mod.method, SimpleDeformMethod::Twist, "Twist");
                                                    ui.selectable_value(&mut deform_mod.method, SimpleDeformMethod::Bend, "Bend");
                                                    ui.selectable_value(&mut deform_mod.method, SimpleDeformMethod::Taper, "Taper");
                                                    ui.selectable_value(&mut deform_mod.method, SimpleDeformMethod::Stretch, "Stretch");
                                                });
                                            ui.label("Axis:");
                                            egui::ComboBox::from_id_salt(format!("deform_mod_axis_{}", i))
                                                .selected_text(match deform_mod.axis {
                                                    DeformAxis::X => "X",
                                                    DeformAxis::Y => "Y",
                                                    DeformAxis::Z => "Z",
                                                })
                                                .show_ui(ui, |ui| {
                                                    ui.selectable_value(&mut deform_mod.axis, DeformAxis::X, "X");
                                                    ui.selectable_value(&mut deform_mod.axis, DeformAxis::Y, "Y");
                                                    ui.selectable_value(&mut deform_mod.axis, DeformAxis::Z, "Z");
                                                });
                                        });
                                        ui.horizontal(|ui| {
                                            ui.label("Factor:");
                                            let speed = if matches!(deform_mod.method, SimpleDeformMethod::Twist | SimpleDeformMethod::Bend) { 0.05 } else { 0.02 };
                                            ui.add(egui::DragValue::new(&mut deform_mod.factor).speed(speed));
                                            ui.label("Limits:");
                                            ui.add(egui::Slider::new(&mut deform_mod.limits.0, 0.0..=1.0).text("Min"));
                                            ui.add(egui::Slider::new(&mut deform_mod.limits.1, 0.0..=1.0).text("Max"));
                                        });
                                    }

                                    if let Some(screw_mod) = modifier.as_screw_mut() {
                                        ui.horizontal(|ui| {
                                            ui.label("Axis:");
                                            egui::ComboBox::from_id_salt(format!("screw_mod_axis_{}", i))
                                                .selected_text(match screw_mod.axis {
                                                    DeformAxis::X => "X",
                                                    DeformAxis::Y => "Y",
                                                    DeformAxis::Z => "Z",
                                                })
                                                .show_ui(ui, |ui| {
                                                    ui.selectable_value(&mut screw_mod.axis, DeformAxis::X, "X");
                                                    ui.selectable_value(&mut screw_mod.axis, DeformAxis::Y, "Y");
                                                    ui.selectable_value(&mut screw_mod.axis, DeformAxis::Z, "Z");
                                                });
                                            ui.label("Pitch:");
                                            ui.add(egui::DragValue::new(&mut screw_mod.screw_offset).speed(0.1));
                                        });
                                        ui.horizontal(|ui| {
                                            ui.label("Angle:");
                                            ui.add(egui::DragValue::new(&mut screw_mod.angle).speed(0.1));
                                            ui.label("Steps:");
                                            ui.add(egui::DragValue::new(&mut screw_mod.steps).range(3..=64));
                                            ui.label("Iterations:");
                                            ui.add(egui::DragValue::new(&mut screw_mod.iterations).range(1..=16));
                                        });
                                    }

                                    if let Some(tri_mod) = modifier.as_triangulate_mut() {
                                        ui.horizontal(|ui| {
                                            ui.label("Quad:");
                                            egui::ComboBox::from_id_salt(format!("tri_mod_quad_{}", i))
                                                .selected_text(format!("{:?}", tri_mod.quad_method))
                                                .show_ui(ui, |ui| {
                                                    ui.selectable_value(&mut tri_mod.quad_method, QuadMethod::Beauty, "Beauty");
                                                    ui.selectable_value(&mut tri_mod.quad_method, QuadMethod::ShortestDiagonal, "Shortest Diagonal");
                                                    ui.selectable_value(&mut tri_mod.quad_method, QuadMethod::Fixed, "Fixed");
                                                    ui.selectable_value(&mut tri_mod.quad_method, QuadMethod::Alternate, "Alternate");
                                                });
                                        });
                                        ui.horizontal(|ui| {
                                            ui.label("N-gon:");
                                            egui::ComboBox::from_id_salt(format!("tri_mod_ngon_{}", i))
                                                .selected_text(format!("{:?}", tri_mod.ngon_method))
                                                .show_ui(ui, |ui| {
                                                    ui.selectable_value(&mut tri_mod.ngon_method, NgonMethod::EarClipping, "Ear Clipping");
                                                    ui.selectable_value(&mut tri_mod.ngon_method, NgonMethod::Fan, "Fan");
                                                });
                                            ui.label("Min Verts:");
                                            ui.add(egui::DragValue::new(&mut tri_mod.min_vertices).range(4..=8));
                                        });
                                    }

                                    if let Some(wire_mod) = modifier.as_wireframe_mut() {
                                        ui.horizontal(|ui| {
                                            ui.label("Thickness:");
                                            ui.add(egui::Slider::new(&mut wire_mod.thickness, 0.01..=0.5));
                                        });
                                    }

                                    if let Some(skin_env) = modifier.as_skin_envelope_mut() {
                                        ui.horizontal(|ui| {
                                            ui.label("Radius (m):");
                                            ui.add(egui::DragValue::new(&mut skin_env.default_radius).speed(0.01).range(0.01..=5.0));
                                            ui.label("Smooth:");
                                            let mut steps = skin_env.smooth_iterations as u32;
                                            if ui.add(egui::DragValue::new(&mut steps).range(0..=8)).changed() {
                                                skin_env.smooth_iterations = steps as usize;
                                            }
                                        });
                                    }

                                    if let Some(remesh_mod) = modifier.as_remesh_mut() {
                                        ui.horizontal(|ui| {
                                            ui.label("Voxel Size (m):");
                                            ui.add(egui::DragValue::new(&mut remesh_mod.voxel_size).speed(0.005).range(0.02..=1.0));
                                            ui.checkbox(&mut remesh_mod.smooth_shading, "Smooth");
                                        });
                                    }

                                    if let Some(dec_mod) = modifier.as_decimate_mut() {
                                        ui.horizontal(|ui| {
                                            ui.label("Ratio (QEM):");
                                            ui.add(egui::Slider::new(&mut dec_mod.ratio, 0.01..=1.0).text("Target Face %"));
                                        });
                                    }

                                    if let Some(cloth_mod) = modifier.as_cloth_mut() {
                                        ui.horizontal(|ui| {
                                            ui.label("Stiffness:");
                                            ui.add(egui::Slider::new(&mut cloth_mod.stiffness, 0.1..=1.0));
                                        });
                                        ui.horizontal(|ui| {
                                            ui.label("Damping:");
                                            ui.add(egui::Slider::new(&mut cloth_mod.damping, 0.0..=0.1));
                                            ui.checkbox(&mut cloth_mod.ground_collision, "Floor (Y=0)");
                                        });
                                        ui.horizontal(|ui| {
                                            if ui.button("▶ Step (1/60s)").clicked() {
                                                cloth_mod.step(1.0 / 60.0);
                                            }
                                            if ui.button("🔄 Reset").clicked() {
                                                cloth_mod.reset();
                                            }
                                        });
                                    }

                                    if let Some(particle_mod) = modifier.as_particle_system_mut() {
                                        ui.label(format!("Active Particles: {}", particle_mod.particles.len()));
                                        ui.horizontal(|ui| {
                                            ui.label("Rate (p/s):");
                                            ui.add(egui::Slider::new(&mut particle_mod.emission_rate, 1.0..=500.0));
                                        });
                                        ui.horizontal(|ui| {
                                            ui.label("Lifetime (s):");
                                            ui.add(egui::Slider::new(&mut particle_mod.lifetime, 0.1..=10.0));
                                            ui.label("Size:");
                                            ui.add(egui::Slider::new(&mut particle_mod.particle_size, 0.01..=0.3));
                                        });
                                        ui.horizontal(|ui| {
                                            ui.label("Speed:");
                                            ui.add(egui::Slider::new(&mut particle_mod.initial_speed, 0.0..=15.0));
                                            ui.label("Spread:");
                                            ui.add(egui::Slider::new(&mut particle_mod.spread, 0.0..=2.0));
                                        });
                                        ui.horizontal(|ui| {
                                            ui.label("Shape:");
                                            egui::ComboBox::from_id_salt(format!("part_shape_{}", i))
                                                .selected_text(match particle_mod.render_shape {
                                                    ParticleRenderShape::Tetrahedron => "Tetrahedron",
                                                    ParticleRenderShape::Diamond => "Diamond",
                                                    ParticleRenderShape::Quad => "Quad",
                                                })
                                                .show_ui(ui, |ui| {
                                                    ui.selectable_value(&mut particle_mod.render_shape, ParticleRenderShape::Tetrahedron, "Tetrahedron");
                                                    ui.selectable_value(&mut particle_mod.render_shape, ParticleRenderShape::Diamond, "Diamond");
                                                    ui.selectable_value(&mut particle_mod.render_shape, ParticleRenderShape::Quad, "Quad");
                                                });
                                            ui.checkbox(&mut particle_mod.emit_only, "Emit Only");
                                        });
                                        ui.horizontal(|ui| {
                                            ui.checkbox(&mut particle_mod.floor_collision, "Floor Collision");
                                            if particle_mod.floor_collision {
                                                ui.label("Y:");
                                                ui.add(egui::DragValue::new(&mut particle_mod.floor_y).speed(0.1));
                                                ui.label("Bounce:");
                                                ui.add(egui::Slider::new(&mut particle_mod.bounce_restitution, 0.0..=1.0));
                                            }
                                        });
                                        ui.horizontal(|ui| {
                                            if ui.button("🔄 Reset Particles").clicked() {
                                                particle_mod.reset();
                                            }
                                        });
                                    }

                                    if let Some(wn_mod) = modifier.as_weighted_normal_mut() {
                                        ui.horizontal(|ui| {
                                            ui.label("Weighting:");
                                            egui::ComboBox::from_id_salt(format!("wn_mode_{}", i))
                                                .selected_text(match wn_mod.mode {
                                                    WeightedNormalMode::FaceArea => "Face Area",
                                                    WeightedNormalMode::CornerAngle => "Corner Angle",
                                                    WeightedNormalMode::AreaAndAngle => "Area & Angle",
                                                })
                                                .show_ui(ui, |ui| {
                                                    ui.selectable_value(&mut wn_mod.mode, WeightedNormalMode::FaceArea, "Face Area");
                                                    ui.selectable_value(&mut wn_mod.mode, WeightedNormalMode::CornerAngle, "Corner Angle");
                                                    ui.selectable_value(&mut wn_mod.mode, WeightedNormalMode::AreaAndAngle, "Area & Angle");
                                                });
                                        });
                                        ui.horizontal(|ui| {
                                            ui.label("Weight:");
                                            ui.add(egui::Slider::new(&mut wn_mod.weight, 0.1..=10.0));
                                            ui.checkbox(&mut wn_mod.keep_sharp, "Keep Sharp");
                                        });
                                    }

                                    if let Some(morph_mod) = modifier.as_morpher_mut() {
                                        let mut target_to_remove = None;
                                        ui.label(format!("Morph Targets ({} channels):", morph_mod.targets.len()));

                                        for (t_idx, target) in morph_mod.targets.iter_mut().enumerate() {
                                            ui.horizontal(|ui| {
                                                let channel_name = format!("morph:{}", target.name);
                                                let has_key = animation.has_scalar_keyframe(sel_id, &channel_name, cur_frame);
                                                let key_btn = if has_key {
                                                    egui::RichText::new("◆").color(egui::Color32::from_rgb(255, 215, 0))
                                                } else {
                                                    egui::RichText::new("◇").color(egui::Color32::GRAY)
                                                };
                                                if ui.button(key_btn).on_hover_text("Insert / Remove Keyframe at Current Frame").clicked() {
                                                    if has_key {
                                                        animation.remove_scalar_keyframe(sel_id, &channel_name, cur_frame);
                                                    } else {
                                                        animation.insert_scalar_keyframe(sel_id, &channel_name, cur_frame, target.weight, InterpolationMode::Linear);
                                                    }
                                                }
                                                ui.label(format!("{}:", target.name));
                                                if ui.add(egui::Slider::new(&mut target.weight, 0.0..=1.0).text("Weight")).changed() {
                                                    if has_key {
                                                        animation.insert_scalar_keyframe(sel_id, &channel_name, cur_frame, target.weight, InterpolationMode::Linear);
                                                    }
                                                }
                                                if ui.small_button("✖").clicked() {
                                                    target_to_remove = Some(t_idx);
                                                }
                                            });
                                        }

                                        if let Some(t_del) = target_to_remove {
                                            morph_mod.targets.remove(t_del);
                                        }

                                        ui.horizontal(|ui| {
                                            if ui.button("+ Add Empty Channel").clicked() {
                                                let count = morph_mod.targets.len() + 1;
                                                morph_mod.add_target(MorphTarget::new(format!("Channel_{}", count)));
                                            }
                                            
                                            if ui.add_enabled(can_capture, egui::Button::new("Capture Target from Selection")).clicked() {
                                                actions.trigger_capture_morph_target = true;
                                            }
                                        });
                                    }
                                });
                            }
                            if !modifier_stack.modifiers().is_empty() {
                                if ui.button("Collapse All Modifiers").on_hover_text("Bake entire stack into base mesh").clicked() {
                                    actions.collapse_modifiers = true;
                                }
                            }
                        }
    }
}

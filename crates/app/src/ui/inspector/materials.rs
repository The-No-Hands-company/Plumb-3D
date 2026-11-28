use super::mesh_info::draw_mesh_info;
use crate::state::AppState;
use crate::types::*;
use crate::ui::actions::UiActions;
use graph_engine::{Material, NodeData, NodeId};

pub fn draw_materials_panel(
    ui: &mut egui::Ui,
    state: &mut AppState,
    actions: &mut UiActions,
    sel_id: NodeId,
) {
    let interaction_mode = state.interaction_mode;
    let sub_element_level = state.sub_element_level;
    let mut auto_smooth_angle_degrees = state.auto_smooth_angle_degrees;
    let mut active_material_slot = state.active_material_slot;
    let sel_verts = state.selected_vertices.clone();
    let sel_edges = state.selected_edges.clone();
    let sel_faces = state.selected_faces.clone();

    if let Some(node) = state.scene.nodes.get_mut(sel_id) {
        if let NodeData::Mesh {
            base_mesh,
            materials,
            ..
        } = &mut node.data
        {
            draw_mesh_info(
                ui,
                interaction_mode,
                &sel_verts,
                &sel_edges,
                &sel_faces,
                base_mesh,
            );
            ui.collapsing("Material Slots & PBR Shading", |ui| {
                if materials.is_empty() {
                    materials.push(Material::default());
                }
                if active_material_slot >= materials.len() {
                    active_material_slot = materials.len().saturating_sub(1);
                }

                ui.horizontal(|ui| {
                    ui.label(format!("Slots ({}):", materials.len()));
                    if ui
                        .button("+ Add Slot")
                        .on_hover_text("Add a new material slot to this mesh")
                        .clicked()
                    {
                        actions.trigger_add_material_slot = true;
                    }
                    if materials.len() > 1
                        && ui
                            .button("- Remove")
                            .on_hover_text("Remove active material slot")
                            .clicked()
                    {
                        actions.trigger_remove_material_slot = true;
                    }
                });

                // Display slots list with color swatches
                egui::ScrollArea::vertical()
                    .max_height(100.0)
                    .show(ui, |ui| {
                        for (slot_idx, slot_mat) in materials.iter().enumerate() {
                            let is_selected = slot_idx == active_material_slot;
                            let c = slot_mat.base_color;
                            let col32 = egui::Color32::from_rgb(
                                (c[0].clamp(0.0, 1.0) * 255.0) as u8,
                                (c[1].clamp(0.0, 1.0) * 255.0) as u8,
                                (c[2].clamp(0.0, 1.0) * 255.0) as u8,
                            );
                            let label = egui::RichText::new(format!(
                                "[*] Slot {}: {}",
                                slot_idx, slot_mat.name
                            ))
                            .color(col32);
                            if ui.selectable_label(is_selected, label).clicked() {
                                active_material_slot = slot_idx;
                            }
                        }
                    });

                if interaction_mode == InteractionMode::Edit
                    && sub_element_level == SubElementLevel::Face
                {
                    ui.horizontal_wrapped(|ui| {
                        if ui
                            .button("Assign to Selected")
                            .on_hover_text("Assign active material slot to selected faces")
                            .clicked()
                        {
                            actions.trigger_assign_material_slot =
                                Some(active_material_slot as u32);
                        }
                        if ui
                            .button("Select Faces")
                            .on_hover_text("Select all faces using this material slot")
                            .clicked()
                        {
                            actions.trigger_select_material_faces =
                                Some(active_material_slot as u32);
                        }
                        if ui
                            .button("Deselect")
                            .on_hover_text("Deselect faces using this material slot")
                            .clicked()
                        {
                            actions.trigger_deselect_material_faces =
                                Some(active_material_slot as u32);
                        }
                    });
                }
                ui.separator();

                // Parameters for Active Slot
                if let Some(active_mat) = materials.get_mut(active_material_slot) {
                    ui.horizontal(|ui| {
                        ui.label("Name:");
                        ui.text_edit_singleline(&mut active_mat.name);
                    });

                    let mut rgb = [
                        active_mat.base_color[0],
                        active_mat.base_color[1],
                        active_mat.base_color[2],
                    ];
                    ui.horizontal(|ui| {
                        ui.label("Albedo:");
                        if ui.color_edit_button_rgb(&mut rgb).changed() {
                            active_mat.base_color[0] = rgb[0];
                            active_mat.base_color[1] = rgb[1];
                            active_mat.base_color[2] = rgb[2];
                            node.is_dirty = true;
                        }
                    });

                    let mut mat_changed = false;
                    ui.horizontal(|ui| {
                        ui.label("Metallic:");
                        mat_changed |= ui
                            .add(egui::Slider::new(&mut active_mat.metallic, 0.0..=1.0))
                            .changed();
                    });
                    ui.horizontal(|ui| {
                        ui.label("Roughness:");
                        mat_changed |= ui
                            .add(egui::Slider::new(&mut active_mat.roughness, 0.04..=1.0))
                            .changed();
                    });
                    ui.horizontal(|ui| {
                        ui.label("Emissive:");
                        mat_changed |= ui
                            .add(egui::Slider::new(&mut active_mat.emissive, 0.0..=5.0))
                            .changed();
                    });

                    // Procedural Texture Pattern Selection
                    let mut pattern = active_mat.texture_pattern;
                    ui.horizontal(|ui| {
                        ui.label("Pattern:");
                        egui::ComboBox::from_id_salt("inspector_mat_pattern")
                            .selected_text(match pattern {
                                0 => "Solid Color",
                                1 => "UV Checkerboard",
                                2 => "UV Coordinate Grid",
                                3 => "World Triplanar Box Grid",
                                4 => "Concentric Rings",
                                5 => "Procedural Marble",
                                _ => "Custom",
                            })
                            .show_ui(ui, |ui| {
                                if ui
                                    .selectable_value(&mut pattern, 0, "Solid Color")
                                    .changed()
                                {
                                    active_mat.texture_pattern = pattern;
                                    mat_changed = true;
                                }
                                if ui
                                    .selectable_value(&mut pattern, 1, "UV Checkerboard")
                                    .changed()
                                {
                                    active_mat.texture_pattern = pattern;
                                    mat_changed = true;
                                }
                                if ui
                                    .selectable_value(&mut pattern, 2, "UV Coordinate Grid")
                                    .changed()
                                {
                                    active_mat.texture_pattern = pattern;
                                    mat_changed = true;
                                }
                                if ui
                                    .selectable_value(&mut pattern, 3, "World Triplanar Box Grid")
                                    .changed()
                                {
                                    active_mat.texture_pattern = pattern;
                                    mat_changed = true;
                                }
                                if ui
                                    .selectable_value(&mut pattern, 4, "Concentric Rings")
                                    .changed()
                                {
                                    active_mat.texture_pattern = pattern;
                                    mat_changed = true;
                                }
                                if ui
                                    .selectable_value(&mut pattern, 5, "Procedural Marble")
                                    .changed()
                                {
                                    active_mat.texture_pattern = pattern;
                                    mat_changed = true;
                                }
                            });
                    });

                    // Image Texture Maps (PBR Diffuse, Normal, Roughness/Metallic)
                    ui.collapsing("Texture Maps (PBR)", |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Albedo:");
                            let mut path_str =
                                active_mat.albedo_texture_path.clone().unwrap_or_default();
                            if ui.text_edit_singleline(&mut path_str).changed() {
                                active_mat.albedo_texture_path = if path_str.trim().is_empty() {
                                    None
                                } else {
                                    Some(path_str.trim().to_string())
                                };
                                node.is_dirty = true;
                            }
                            if ui
                                .button("📁")
                                .on_hover_text("Browse for image texture (PNG, JPG, HDR, WebP)")
                                .clicked()
                            {
                                if let Some(path) = rfd::FileDialog::new()
                                    .add_filter(
                                        "Image Files",
                                        &["png", "jpg", "jpeg", "hdr", "webp", "tga", "bmp"],
                                    )
                                    .pick_file()
                                {
                                    active_mat.albedo_texture_path =
                                        Some(path.to_string_lossy().to_string());
                                    node.is_dirty = true;
                                }
                            }
                        });
                        ui.horizontal(|ui| {
                            ui.label("Normal Map:");
                            let mut path_str =
                                active_mat.normal_texture_path.clone().unwrap_or_default();
                            if ui.text_edit_singleline(&mut path_str).changed() {
                                active_mat.normal_texture_path = if path_str.trim().is_empty() {
                                    None
                                } else {
                                    Some(path_str.trim().to_string())
                                };
                                node.is_dirty = true;
                            }
                            if ui
                                .button("📁")
                                .on_hover_text("Browse for tangent normal map (PNG, JPG)")
                                .clicked()
                            {
                                if let Some(path) = rfd::FileDialog::new()
                                    .add_filter(
                                        "Image Files",
                                        &["png", "jpg", "jpeg", "webp", "tga", "bmp"],
                                    )
                                    .pick_file()
                                {
                                    active_mat.normal_texture_path =
                                        Some(path.to_string_lossy().to_string());
                                    node.is_dirty = true;
                                }
                            }
                        });
                        ui.horizontal(|ui| {
                            ui.label("Rough/Metallic:");
                            let mut path_str = active_mat
                                .roughness_metallic_texture_path
                                .clone()
                                .unwrap_or_default();
                            if ui.text_edit_singleline(&mut path_str).changed() {
                                active_mat.roughness_metallic_texture_path =
                                    if path_str.trim().is_empty() {
                                        None
                                    } else {
                                        Some(path_str.trim().to_string())
                                    };
                                node.is_dirty = true;
                            }
                            if ui
                                .button("📁")
                                .on_hover_text("Browse for roughness/metallic map (PNG, JPG)")
                                .clicked()
                            {
                                if let Some(path) = rfd::FileDialog::new()
                                    .add_filter(
                                        "Image Files",
                                        &["png", "jpg", "jpeg", "webp", "tga", "bmp"],
                                    )
                                    .pick_file()
                                {
                                    active_mat.roughness_metallic_texture_path =
                                        Some(path.to_string_lossy().to_string());
                                    node.is_dirty = true;
                                }
                            }
                        });
                    });

                    if mat_changed {
                        node.is_dirty = true;
                    }

                    ui.label("Presets:");
                    ui.horizontal_wrapped(|ui| {
                        if ui.small_button("Gold").clicked() {
                            *active_mat = Material::gold(&active_mat.name);
                            node.is_dirty = true;
                        }
                        if ui.small_button("Copper").clicked() {
                            *active_mat = Material::copper(&active_mat.name);
                            node.is_dirty = true;
                        }
                        if ui.small_button("Chrome").clicked() {
                            *active_mat = Material::chrome(&active_mat.name);
                            node.is_dirty = true;
                        }
                        if ui.small_button("Plastic").clicked() {
                            *active_mat = Material::plastic(&active_mat.name);
                            node.is_dirty = true;
                        }
                        if ui.small_button("Rubber").clicked() {
                            *active_mat = Material::rubber(&active_mat.name);
                            node.is_dirty = true;
                        }
                        if ui.small_button("Clay").clicked() {
                            *active_mat = Material::clay(&active_mat.name);
                            node.is_dirty = true;
                        }
                        if ui.small_button("Glass").clicked() {
                            *active_mat = Material::glass(&active_mat.name);
                            node.is_dirty = true;
                        }
                        if ui.small_button("Ruby").clicked() {
                            *active_mat = Material::ruby(&active_mat.name);
                            node.is_dirty = true;
                        }
                        if ui.small_button("Emerald").clicked() {
                            *active_mat = Material::emerald(&active_mat.name);
                            node.is_dirty = true;
                        }
                    });
                }
            });
            ui.separator();

            // Normals & Auto-Smooth
            ui.collapsing("Normals & Shading", |ui| {
                ui.horizontal_wrapped(|ui| {
                    if ui
                        .button("Auto-Smooth (30 deg)")
                        .on_hover_text("Mark edges >30 deg sharp and split normals")
                        .clicked()
                    {
                        base_mesh.auto_smooth_normals(std::f32::consts::PI / 6.0);
                        node.is_dirty = true;
                    }
                    if ui
                        .button("Auto-Smooth (45 deg)")
                        .on_hover_text("Mark edges >45 deg sharp")
                        .clicked()
                    {
                        base_mesh.auto_smooth_normals(std::f32::consts::FRAC_PI_4);
                        node.is_dirty = true;
                    }
                    if ui.button("Clear Sharp").clicked() {
                        for e in base_mesh.edges.values_mut() {
                            e.is_sharp = false;
                        }
                        base_mesh.update_normals();
                        node.is_dirty = true;
                    }
                });
                ui.horizontal(|ui| {
                    ui.add(
                        egui::Slider::new(&mut auto_smooth_angle_degrees, 0.0..=180.0)
                            .text("Angle (deg)"),
                    );
                    if ui.button("Auto-Smooth").clicked() {
                        actions.trigger_auto_smooth = true;
                    }
                });
            });
            ui.separator();
        }
    }
    state.auto_smooth_angle_degrees = auto_smooth_angle_degrees;
    state.active_material_slot = active_material_slot;
}

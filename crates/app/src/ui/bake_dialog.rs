//! Interactive PBR Texture Map Baker Dialog (Blender / Substance 3D Baker style).
//! Bakes Tangent Normal, Ambient Occlusion, Curvature, and Height maps from high-poly sculpts to low-poly UVs.

use crate::state::AppState;
use graph_engine::{NodeData, NodeId};
use render_pathtracer::{bake_texture, BakeMapType, BakeSettings};

/// Holds state for the Texture Baking modal window.
pub struct BakeDialogState {
    pub show_dialog: bool,
    pub map_type: BakeMapType,
    pub resolution: u32,
    pub cage_distance: f32,
    pub ao_samples: u32,
    pub high_poly_node: Option<NodeId>,
    pub preview_tex: Option<egui::TextureHandle>,
    pub status_message: String,
    pub last_baked_image: Option<image::RgbaImage>,
}

impl std::fmt::Debug for BakeDialogState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BakeDialogState")
            .field("show_dialog", &self.show_dialog)
            .field("map_type", &self.map_type)
            .field("resolution", &self.resolution)
            .field("cage_distance", &self.cage_distance)
            .field("ao_samples", &self.ao_samples)
            .field("high_poly_node", &self.high_poly_node)
            .field("status_message", &self.status_message)
            .finish()
    }
}

impl Default for BakeDialogState {
    fn default() -> Self {
        Self {
            show_dialog: false,
            map_type: BakeMapType::TangentNormal,
            resolution: 512,
            cage_distance: 0.15,
            ao_samples: 16,
            high_poly_node: None,
            preview_tex: None,
            status_message: "Select Low-Poly target and High-Poly source to bake.".into(),
            last_baked_image: None,
        }
    }
}

/// Renders the PBR Texture Baker dialog window in egui.
pub fn draw_bake_dialog(state: &mut AppState) {
    if !state.bake_state.show_dialog {
        return;
    }

    let mut open = state.bake_state.show_dialog;
    let mut trigger_bake = false;
    let mut trigger_export = false;

    egui::Window::new("🍳 PBR Texture Map Baker")
        .open(&mut open)
        .default_size(egui::vec2(480.0, 560.0))
        .resizable(true)
        .show(&state.egui_ctx.clone(), |ui| {
            ui.heading("Texture Baking Engine");
            ui.label(egui::RichText::new("Bake surface details, normals, and occlusion from sculpts onto game-ready meshes.").small().weak());
            ui.separator();

            // Low-poly target info
            let low_poly_info = if let Some(sel_id) = state.selected_node {
                if let Some(node) = state.scene.nodes.get(sel_id) {
                    format!("Target (Low-Poly): {}", node.name)
                } else {
                    "Target (Low-Poly): None (Select a mesh)".into()
                }
            } else {
                "Target (Low-Poly): None (Select a mesh)".into()
            };
            ui.label(egui::RichText::new(low_poly_info).strong());

            // High-poly source selector
            let mesh_nodes: Vec<(NodeId, String)> = state
                .scene
                .nodes
                .iter()
                .filter_map(|(id, n)| {
                    if matches!(n.data, NodeData::Mesh { .. }) {
                        Some((id, n.name.clone()))
                    } else {
                        None
                    }
                })
                .collect();

            ui.horizontal(|ui| {
                ui.label("Source (High-Poly):");
                let current_high_name = state
                    .bake_state
                    .high_poly_node
                    .and_then(|id| state.scene.nodes.get(id).map(|n| n.name.clone()))
                    .unwrap_or_else(|| "Self (Target Mesh)".into());

                egui::ComboBox::from_id_salt("bake_high_poly_source")
                    .selected_text(current_high_name)
                    .show_ui(ui, |ui| {
                        if ui.selectable_label(state.bake_state.high_poly_node.is_none(), "Self (Target Mesh)").clicked() {
                            state.bake_state.high_poly_node = None;
                        }
                        for (nid, name) in &mesh_nodes {
                            let is_sel = state.bake_state.high_poly_node == Some(*nid);
                            if ui.selectable_label(is_sel, name).clicked() {
                                state.bake_state.high_poly_node = Some(*nid);
                            }
                        }
                    });
            });

            ui.separator();

            // Map type selection
            ui.horizontal(|ui| {
                ui.label("Map Type:");
                egui::ComboBox::from_id_salt("bake_map_type")
                    .selected_text(match state.bake_state.map_type {
                        BakeMapType::TangentNormal => "Tangent Space Normal",
                        BakeMapType::AmbientOcclusion => "Ambient Occlusion (AO)",
                        BakeMapType::Curvature => "Curvature / Cavity",
                        BakeMapType::Height => "Height / Displacement",
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut state.bake_state.map_type, BakeMapType::TangentNormal, "Tangent Space Normal");
                        ui.selectable_value(&mut state.bake_state.map_type, BakeMapType::AmbientOcclusion, "Ambient Occlusion (AO)");
                        ui.selectable_value(&mut state.bake_state.map_type, BakeMapType::Curvature, "Curvature / Cavity");
                        ui.selectable_value(&mut state.bake_state.map_type, BakeMapType::Height, "Height / Displacement");
                    });
            });

            // Resolution selection
            ui.horizontal(|ui| {
                ui.label("Resolution:");
                egui::ComboBox::from_id_salt("bake_resolution")
                    .selected_text(format!("{} x {}", state.bake_state.resolution, state.bake_state.resolution))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut state.bake_state.resolution, 256, "256 x 256");
                        ui.selectable_value(&mut state.bake_state.resolution, 512, "512 x 512");
                        ui.selectable_value(&mut state.bake_state.resolution, 1024, "1024 x 1024");
                        ui.selectable_value(&mut state.bake_state.resolution, 2048, "2048 x 2048");
                    });
            });

            // Parameters
            ui.horizontal(|ui| {
                ui.label("Cage Distance (m):");
                ui.add(egui::DragValue::new(&mut state.bake_state.cage_distance).speed(0.01).range(0.01..=2.0));
            });

            if state.bake_state.map_type == BakeMapType::AmbientOcclusion {
                ui.horizontal(|ui| {
                    ui.label("AO Samples:");
                    ui.add(egui::DragValue::new(&mut state.bake_state.ao_samples).range(4..=64));
                });
            }

            ui.separator();

            let can_bake = state.selected_node.is_some();
            ui.horizontal(|ui| {
                if ui.add_enabled(can_bake, egui::Button::new("🍳 Bake Texture")).clicked() {
                    trigger_bake = true;
                }
                if state.bake_state.last_baked_image.is_some() {
                    if ui.button("💾 Export PNG...").clicked() {
                        trigger_export = true;
                    }
                }
            });

            ui.label(egui::RichText::new(&state.bake_state.status_message).italics().small());

            // Preview viewport inside dialog
            if let Some(tex) = &state.bake_state.preview_tex {
                ui.separator();
                ui.label("Baked Texture Preview:");
                let avail = ui.available_size_before_wrap();
                let side = avail.x.min(avail.y).max(180.0).min(320.0);
                ui.image((tex.id(), egui::vec2(side, side)));
            }
        });

    state.bake_state.show_dialog = open;

    if trigger_bake {
        let sel_id = match state.selected_node {
            Some(id) => id,
            None => return,
        };

        let low_mesh = match state.scene.nodes.get(sel_id) {
            Some(n) => match &n.data {
                NodeData::Mesh { base_mesh, .. } => base_mesh.clone(),
                _ => return,
            },
            None => return,
        };

        let high_id = state.bake_state.high_poly_node.unwrap_or(sel_id);
        let high_mesh = match state.scene.nodes.get(high_id) {
            Some(n) => match &n.data {
                NodeData::Mesh { base_mesh, .. } => base_mesh.clone(),
                _ => return,
            },
            None => return,
        };

        let settings = BakeSettings {
            map_type: state.bake_state.map_type,
            width: state.bake_state.resolution,
            height: state.bake_state.resolution,
            cage_distance: state.bake_state.cage_distance,
            ao_samples: state.bake_state.ao_samples,
        };

        let start = std::time::Instant::now();
        let baked_img = bake_texture(&low_mesh, &high_mesh, &settings);
        let elapsed = start.elapsed().as_millis();

        // Convert to egui texture handle
        let w = baked_img.width() as usize;
        let h = baked_img.height() as usize;
        let color_img = egui::ColorImage::from_rgba_unmultiplied([w, h], baked_img.as_raw());
        let tex = state.egui_ctx.load_texture("baked_preview_tex", color_img, Default::default());

        state.bake_state.preview_tex = Some(tex);
        state.bake_state.last_baked_image = Some(baked_img);
        state.bake_state.status_message = format!("Bake completed in {} ms ({}x{}).", elapsed, w, h);
    }

    if trigger_export {
        if let Some(img) = &state.bake_state.last_baked_image {
            let filename = match state.bake_state.map_type {
                BakeMapType::TangentNormal => "baked_normal.png",
                BakeMapType::AmbientOcclusion => "baked_ao.png",
                BakeMapType::Curvature => "baked_curvature.png",
                BakeMapType::Height => "baked_height.png",
            };
            match img.save(filename) {
                Ok(_) => {
                    state.bake_state.status_message = format!("Exported successfully to '{}'.", filename);
                }
                Err(e) => {
                    state.bake_state.status_message = format!("Export failed: {}", e);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bake_dialog_state_defaults() {
        let state = BakeDialogState::default();
        assert!(!state.show_dialog);
        assert_eq!(state.map_type, BakeMapType::TangentNormal);
        assert_eq!(state.resolution, 512);
        assert!((state.cage_distance - 0.15).abs() < 1e-4);
        assert_eq!(state.ao_samples, 16);
        assert!(state.high_poly_node.is_none());
        assert!(state.last_baked_image.is_none());
    }
}

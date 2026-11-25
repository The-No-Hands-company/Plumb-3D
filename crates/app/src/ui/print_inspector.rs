//! 3D Print & CAD Quality Inspector & Direct CAM Slicer Studio.
//! Inspects meshes for watertightness, physical volume, filament weight, dimensions,
//! overhang support requirements, and provides direct CAM G-code toolpath slicing.

use crate::state::AppState;
use geo_kernel::{InfillPattern, MeshAnalysisReport, SlicerSettings};
use graph_engine::NodeData;

/// State for the 3D Print Quality Inspector & CAM Slicer window.
#[derive(Debug, Clone)]
pub struct PrintInspectorState {
    pub show_dialog: bool,
    pub overhang_angle_deg: f32,
    pub highlight_overhangs: bool,
    pub report: Option<MeshAnalysisReport>,
    pub layer_height_mm: f32,
    pub wall_perimeters: usize,
    pub infill_density: f32,
    pub infill_pattern: InfillPattern,
    pub print_speed_mms: f32,
    pub nozzle_temp_c: u32,
    pub bed_temp_c: u32,
}

impl Default for PrintInspectorState {
    fn default() -> Self {
        Self {
            show_dialog: false,
            overhang_angle_deg: 45.0,
            highlight_overhangs: true,
            report: None,
            layer_height_mm: 0.20,
            wall_perimeters: 2,
            infill_density: 0.15,
            infill_pattern: InfillPattern::Gyroid,
            print_speed_mms: 60.0,
            nozzle_temp_c: 215,
            bed_temp_c: 60,
        }
    }
}

/// Renders the 3D Print Quality Inspector and CAM Slicer dialog in egui.
pub fn draw_print_inspector(state: &mut AppState) {
    if !state.print_inspector.show_dialog {
        return;
    }

    let mut open = state.print_inspector.show_dialog;
    let mut trigger_autofix = false;
    let mut trigger_stl_export = false;
    let mut trigger_gcode_export = false;

    // Refresh analysis if selected mesh exists
    if let Some(sel_id) = state.selected_node {
        if let Some(node) = state.scene.nodes.get(sel_id) {
            if let NodeData::Mesh { base_mesh, .. } = &node.data {
                state.print_inspector.report = Some(
                    base_mesh.analyze_for_3d_printing(state.print_inspector.overhang_angle_deg),
                );
            } else {
                state.print_inspector.report = None;
            }
        }
    } else {
        state.print_inspector.report = None;
    }

    egui::Window::new("🖨 3D Print Quality Inspector & CAM Slicer Studio")
        .open(&mut open)
        .default_size(egui::vec2(500.0, 600.0))
        .resizable(true)
        .show(&state.egui_ctx.clone(), |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading("Slicer Pre-Flight, Solid Integrity & G-Code Studio");
                ui.label(egui::RichText::new("Analyze manifold solids, overhang angles, configure infill, and slice G-code directly.").small().weak());
                ui.separator();

                let active_name = state
                    .selected_node
                    .and_then(|id| state.scene.nodes.get(id).map(|n| n.name.clone()))
                    .unwrap_or_else(|| "No Mesh Selected".into());

                ui.horizontal(|ui| {
                    ui.label("Inspecting Solid:");
                    ui.label(egui::RichText::new(active_name).strong());
                });

                if let Some(report) = &state.print_inspector.report {
                    ui.separator();

                    // 1. Watertight Solid Status Banner
                    if report.is_watertight {
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("✔ 100% Watertight Solid (2-Manifold)").color(egui::Color32::from_rgb(80, 220, 100)).strong());
                            });
                            ui.label(egui::RichText::new("Zero boundary holes or degenerate faces. Certified ready for slicing.").small().weak());
                        });
                    } else {
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("✖ Non-Watertight / Boundary Holes").color(egui::Color32::from_rgb(255, 80, 80)).strong());
                            });
                            if report.open_boundary_edges > 0 {
                                ui.label(format!("• Open boundary half-edges: {}", report.open_boundary_edges));
                            }
                            if report.degenerate_faces > 0 {
                                ui.label(format!("• Degenerate zero-area faces: {}", report.degenerate_faces));
                            }
                        });
                    }

                    ui.separator();

                    // 2. Physical Dimensions & Volume
                    ui.label(egui::RichText::new("Physical Dimensions & Slicing Metrics").strong());
                    ui.horizontal(|ui| {
                        ui.label("Bounding Box (mm):");
                        ui.label(format!("{:.1} x {:.1} x {:.1} mm", report.dimensions_mm.x, report.dimensions_mm.y, report.dimensions_mm.z));
                    });

                    ui.horizontal(|ui| {
                        ui.label("Enclosed Volume:");
                        ui.label(format!("{:.2} cm³ ({:.6} m³)", report.volume_cm3, report.volume_m3));
                    });

                    ui.horizontal(|ui| {
                        ui.label("Filament Estimate (PLA):");
                        ui.label(format!("{:.1} grams (~{:.2} meters of 1.75mm)", report.estimated_weight_pla_grams, report.estimated_weight_pla_grams / 3.0));
                    });

                    ui.separator();

                    // 3. Overhang Angle & Support Structure Analysis
                    ui.label(egui::RichText::new("FDM Support & Overhang Analysis").strong());
                    ui.horizontal(|ui| {
                        ui.label("Overhang Threshold:");
                        ui.add(egui::Slider::new(&mut state.print_inspector.overhang_angle_deg, 20.0..=75.0).text("° from bed"));
                    });

                    let pct = if report.face_count > 0 {
                        (report.overhang_faces as f32 / report.face_count as f32) * 100.0
                    } else {
                        0.0
                    };

                    ui.horizontal(|ui| {
                        ui.label("Overhang Faces:");
                        let color = if report.overhang_faces > 0 {
                            egui::Color32::from_rgb(255, 180, 60)
                        } else {
                            egui::Color32::from_rgb(100, 220, 100)
                        };
                        ui.label(egui::RichText::new(format!("{} faces ({:.1}%)", report.overhang_faces, pct)).color(color));
                    });

                    ui.checkbox(&mut state.print_inspector.highlight_overhangs, "Highlight Overhang Faces in 3D Viewport");

                    ui.separator();

                    // 4. Direct CAM 3D Print Slicer Settings
                    ui.group(|ui| {
                        ui.label(egui::RichText::new("⚙ Direct CAM Slicer Parameters").strong());

                        ui.horizontal(|ui| {
                            ui.label("Layer Height:");
                            ui.add(egui::Slider::new(&mut state.print_inspector.layer_height_mm, 0.08..=0.36).step_by(0.04).text("mm"));
                        });

                        ui.horizontal(|ui| {
                            ui.label("Wall Loops (Perimeters):");
                            ui.add(egui::Slider::new(&mut state.print_inspector.wall_perimeters, 1..=6).text("loops"));
                        });

                        ui.horizontal(|ui| {
                            ui.label("Infill Pattern:");
                            egui::ComboBox::from_id_salt("slicer_infill_pattern_combo")
                                .selected_text(match state.print_inspector.infill_pattern {
                                    InfillPattern::Gyroid => "Gyroid (Isotropic)",
                                    InfillPattern::Rectilinear => "Rectilinear",
                                    InfillPattern::Grid => "Grid (Rigid)",
                                    InfillPattern::Honeycomb => "Honeycomb",
                                })
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut state.print_inspector.infill_pattern, InfillPattern::Gyroid, "Gyroid (Isotropic)");
                                    ui.selectable_value(&mut state.print_inspector.infill_pattern, InfillPattern::Rectilinear, "Rectilinear");
                                    ui.selectable_value(&mut state.print_inspector.infill_pattern, InfillPattern::Grid, "Grid (Rigid)");
                                    ui.selectable_value(&mut state.print_inspector.infill_pattern, InfillPattern::Honeycomb, "Honeycomb");
                                });
                        });

                        ui.horizontal(|ui| {
                            ui.label("Infill Density:");
                            let mut pct = state.print_inspector.infill_density * 100.0;
                            if ui.add(egui::Slider::new(&mut pct, 0.0..=100.0).text("%")).changed() {
                                state.print_inspector.infill_density = pct / 100.0;
                            }
                        });

                        ui.horizontal(|ui| {
                            ui.label("Print Speed:");
                            ui.add(egui::Slider::new(&mut state.print_inspector.print_speed_mms, 20.0..=300.0).text("mm/s"));
                        });

                        ui.horizontal(|ui| {
                            ui.label("Nozzle Temp:");
                            ui.add(egui::Slider::new(&mut state.print_inspector.nozzle_temp_c, 180..=280).text("°C"));
                            ui.label("Bed Temp:");
                            ui.add(egui::Slider::new(&mut state.print_inspector.bed_temp_c, 40..=100).text("°C"));
                        });

                        ui.add_space(4.0);
                        if ui.button("🖨 Slice Mesh & Export G-Code (.gcode)").on_hover_text("Directly slices geometry and outputs CNC G-code toolpath file").clicked() {
                            trigger_gcode_export = true;
                        }
                    });

                    ui.separator();

                    // 5. General Actions
                    ui.horizontal(|ui| {
                        if !report.is_watertight
                            && ui.button("🛠 Auto-Fix Solid for 3D Printing").on_hover_text("Cap holes, weld degenerates, and align outward normals").clicked() {
                                trigger_autofix = true;
                            }
                        if ui.button("💾 Export Binary STL (.stl)").clicked() {
                            trigger_stl_export = true;
                        }
                    });
                } else {
                    ui.label(egui::RichText::new("Select a mesh object in the 3D viewport or Outliner to begin print inspection & slicing.").italics());
                }
            });
        });

    state.print_inspector.show_dialog = open;

    // Handle Auto-Fix action
    if trigger_autofix {
        if let Some(sel_id) = state.selected_node {
            if let Some(node) = state.scene.nodes.get_mut(sel_id) {
                if let NodeData::Mesh { base_mesh, .. } = &mut node.data {
                    let _ = base_mesh.auto_fix_for_3d_printing();
                    state.scene.mark_dirty(sel_id);
                }
            }
        }
    }

    // Handle Quick STL Export
    if trigger_stl_export {
        if let Some(sel_id) = state.selected_node {
            if let Some(node) = state.scene.nodes.get(sel_id) {
                if let NodeData::Mesh { base_mesh, .. } = &node.data {
                    let stl_data = base_mesh.to_stl_binary();
                    let filename = format!("{}_3dprint.stl", node.name);
                    let _ = std::fs::write(&filename, stl_data);
                    state.set_status_message(format!("💾 Saved binary STL to: {}", filename));
                }
            }
        }
    }

    // Handle Direct CAM G-Code Slicing and Export
    if trigger_gcode_export {
        if let Some(sel_id) = state.selected_node {
            if let Some(node) = state.scene.nodes.get(sel_id) {
                if let NodeData::Mesh { base_mesh, .. } = &node.data {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("3D Printer G-code (*.gcode)", &["gcode"])
                        .set_file_name(format!("{}.gcode", node.name))
                        .save_file()
                    {
                        let mut settings = SlicerSettings::default();
                        settings.layer_height = state.print_inspector.layer_height_mm;
                        settings.wall_loops = state.print_inspector.wall_perimeters;
                        settings.infill_density = state.print_inspector.infill_density;
                        settings.infill_pattern = state.print_inspector.infill_pattern;
                        settings.print_speed_mm_s = state.print_inspector.print_speed_mms;
                        settings.nozzle_temp = state.print_inspector.nozzle_temp_c;
                        settings.bed_temp = state.print_inspector.bed_temp_c;

                        if base_mesh.export_gcode_file(&path, &settings).is_ok() {
                            state.set_status_message(format!("🖨 Exported G-code toolpath to: {}", path.display()));
                        }
                    }
                }
            }
        }
    }

    // Apply Overhang Selection Highlight if enabled
    if state.print_inspector.highlight_overhangs {
        if let Some(report) = &state.print_inspector.report {
            if !report.overhang_face_keys.is_empty() {
                state.selected_faces = report.overhang_face_keys.iter().copied().collect();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_print_inspector_state_defaults() {
        let state = PrintInspectorState::default();
        assert!(!state.show_dialog);
        assert!((state.overhang_angle_deg - 45.0).abs() < 1e-4);
        assert!(state.highlight_overhangs);
        assert!(state.report.is_none());
        assert_eq!(state.wall_perimeters, 2);
        assert_eq!(state.infill_pattern, InfillPattern::Gyroid);
    }
}

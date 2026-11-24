//! 3D Print & CAD Quality Inspector Dialog.
//! Inspects meshes for watertightness, physical volume, filament weight, dimensions,
//! overhang support requirements, and provides one-click mesh auto-healing.

use crate::state::AppState;
use geo_kernel::MeshAnalysisReport;
use graph_engine::NodeData;

/// State for the 3D Print Quality Inspector tool window.
#[derive(Debug, Clone)]
pub struct PrintInspectorState {
    pub show_dialog: bool,
    pub overhang_angle_deg: f32,
    pub highlight_overhangs: bool,
    pub report: Option<MeshAnalysisReport>,
}

impl Default for PrintInspectorState {
    fn default() -> Self {
        Self {
            show_dialog: false,
            overhang_angle_deg: 45.0,
            highlight_overhangs: true,
            report: None,
        }
    }
}

/// Renders the 3D Print Quality Inspector dialog in egui.
pub fn draw_print_inspector(state: &mut AppState) {
    if !state.print_inspector.show_dialog {
        return;
    }

    let mut open = state.print_inspector.show_dialog;
    let mut trigger_autofix = false;
    let mut trigger_stl_export = false;

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

    egui::Window::new("🖨 3D Print & CAD Quality Inspector")
        .open(&mut open)
        .default_size(egui::vec2(460.0, 520.0))
        .resizable(true)
        .show(&state.egui_ctx.clone(), |ui| {
            ui.heading("Slicer Pre-Flight & Mesh Integrity");
            ui.label(egui::RichText::new("Analyze manifold solids, overhang angles, physical volume, and filament weight for 3D printing.").small().weak());
            ui.separator();

            let active_name = state
                .selected_node
                .and_then(|id| state.scene.nodes.get(id).map(|n| n.name.clone()))
                .unwrap_or_else(|| "No Mesh Selected".into());

            ui.horizontal(|ui| {
                ui.label("Inspecting Mesh:");
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

                ui.horizontal(|ui| {
                    ui.label("Surface Area:");
                    ui.label(format!("{:.1} cm² ({:.4} m²)", report.surface_area_cm2, report.surface_area_m2));
                });

                ui.separator();

                // 3. Overhang Angle & Support Structure Analysis
                ui.label(egui::RichText::new("FDM Support & Overhang Analysis").strong());
                ui.horizontal(|ui| {
                    ui.label("Overhang Threshold:");
                    ui.add(egui::Slider::new(&mut state.print_inspector.overhang_angle_deg, 20.0..=75.0).text("° from build plate"));
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

                ui.checkbox(&mut state.print_inspector.highlight_overhangs, "Highlight Overhang Faces in Selection");

                ui.separator();

                // 4. Action Buttons
                ui.horizontal(|ui| {
                    if !report.is_watertight {
                        if ui.button("🛠 Auto-Fix for 3D Printing").on_hover_text("Cap holes, weld degenerates, and align outward normals").clicked() {
                            trigger_autofix = true;
                        }
                    }
                    if ui.button("💾 Export Binary STL (.stl)").clicked() {
                        trigger_stl_export = true;
                    }
                });
            } else {
                ui.label(egui::RichText::new("Select a mesh object in the 3D viewport or Outliner to begin print inspection.").italics());
            }
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
    }
}

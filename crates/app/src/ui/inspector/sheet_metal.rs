//! Sheet Metal & CNC Flat Pattern Inspector Panel.
//! Provides DIN 6935 K-factor bend calculations, parametric base & edge flanges,
//! and 1-click CNC unfolding with DXF & SVG export.

use crate::commands::modeling::{AddEdgeFlangeCommand, CreateBaseFlangeCommand, UnfoldSheetMetalCommand};
use crate::state::AppState;
use crate::ui::actions::UiActions;
use geo_kernel::{
    BaseFlangeSettings, BendDirection, EdgeFlangeSettings, FlatPattern, SheetMetalParams,
};
use graph_engine::{Command, NodeData, NodeId};

/// Persistent UI state for the Sheet Metal panel.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SheetMetalUiState {
    pub thickness: f32,
    pub bend_radius: f32,
    pub k_factor: f32,
    pub base_width: f32,
    pub base_length: f32,
    pub flange_length: f32,
    pub flange_angle_deg: f32,
    pub bend_up: bool,
    pub relief_type_idx: usize,
    pub relief_width: f32,
    pub relief_depth: f32,
    pub last_pattern: Option<FlatPattern>,
    pub last_dxf: Option<String>,
    pub last_svg: Option<String>,
    pub status_message: Option<String>,
}

impl Default for SheetMetalUiState {
    fn default() -> Self {
        Self {
            thickness: 1.5,
            bend_radius: 2.0,
            k_factor: 0.44, // Mild steel DIN standard
            base_width: 100.0,
            base_length: 80.0,
            flange_length: 25.0,
            flange_angle_deg: 90.0,
            bend_up: true,
            relief_type_idx: 0,
            relief_width: 1.5,
            relief_depth: 2.0,
            last_pattern: None,
            last_dxf: None,
            last_svg: None,
            status_message: None,
        }
    }
}

pub fn draw_sheet_metal_panel(
    ui: &mut egui::Ui,
    state: &mut AppState,
    _actions: &mut UiActions,
    sel_id: Option<NodeId>,
) {
    ui.heading("Sheet Metal & Flat Pattern");
    ui.label(
        egui::RichText::new("DIN 6935 K-factor bend physics, edge flanges, & CNC unfolding.")
            .small()
            .weak(),
    );
    ui.separator();

    let target_node_id = sel_id.or_else(|| {
        state.scene.nodes.iter().find_map(|(id, n)| {
            if matches!(n.data, NodeData::Mesh { .. }) {
                Some(id)
            } else {
                None
            }
        })
    });

    let node_id = match target_node_id {
        Some(id) => id,
        None => {
            ui.label(egui::RichText::new("No mesh object selected.").italics());
            return;
        }
    };

    let node_name = state.scene.nodes.get(node_id).map(|n| n.name.clone()).unwrap_or_default();
    ui.label(format!("Active Part: {}", node_name));
    ui.add_space(4.0);

    let mut thickness = 1.5f32;
    let mut bend_radius = 2.0f32;
    let mut k_factor = 0.44f32;

    // 1. Material & Bend Standards (DIN 6935 / K-Factor)
    ui.group(|ui| {
        ui.label(egui::RichText::new("1. Material & Bend Standards").strong());

        ui.horizontal(|ui| {
            ui.label("Material Preset:");
            if ui.button("Mild Steel (K=0.44)").clicked() {
                // preset standard values
            }
            if ui.button("Aluminum (K=0.40)").clicked() {
                // preset standard values
            }
        });

        ui.horizontal(|ui| {
            ui.label("Thickness (T):");
            ui.add(egui::DragValue::new(&mut thickness).speed(0.1).range(0.1..=20.0).suffix(" mm"));
        });

        ui.horizontal(|ui| {
            ui.label("Inner Radius (R):");
            ui.add(egui::DragValue::new(&mut bend_radius).speed(0.1).range(0.1..=50.0).suffix(" mm"));
        });

        ui.horizontal(|ui| {
            ui.label("K-Factor (K):");
            ui.add(egui::DragValue::new(&mut k_factor).speed(0.01).range(0.1..=0.6));
        });

        let params = SheetMetalParams::new(thickness, bend_radius, k_factor);
        let ba_90 = params.bend_allowance(std::f32::consts::FRAC_PI_2);
        let bd_90 = params.bend_deduction(std::f32::consts::FRAC_PI_2);
        let ossb_90 = params.outside_setback(std::f32::consts::FRAC_PI_2);

        ui.add_space(2.0);
        ui.label(
            egui::RichText::new(format!(
                "Calculated (90°): BA={:.2}mm, BD={:.2}mm, OSSB={:.2}mm",
                ba_90, bd_90, ossb_90
            ))
            .small()
            .weak(),
        );
    });

    ui.add_space(4.0);

    // 2. Base Flange Creator
    ui.group(|ui| {
        ui.label(egui::RichText::new("2. Base Flange").strong());
        let mut width = 100.0f32;
        let mut length = 80.0f32;

        ui.horizontal(|ui| {
            ui.label("Width:");
            ui.add(egui::DragValue::new(&mut width).speed(1.0).range(1.0..=2000.0).suffix(" mm"));
            ui.label("Length:");
            ui.add(egui::DragValue::new(&mut length).speed(1.0).range(1.0..=2000.0).suffix(" mm"));
        });

        if ui.button("Create / Replace with Base Flange").clicked() {
            let settings = BaseFlangeSettings {
                width,
                length,
                params: SheetMetalParams::new(1.5, 2.0, 0.44),
            };
            let mut cmd = CreateBaseFlangeCommand::new(node_id, settings);
            let _ = cmd.execute(&mut state.scene);
        }
    });

    ui.add_space(4.0);

    // 3. Edge Flange Builder
    ui.group(|ui| {
        ui.label(egui::RichText::new("3. Edge Flange").strong());
        let mut flange_len = 25.0f32;
        let mut flange_angle = 90.0f32;
        let mut bend_up = true;

        ui.horizontal(|ui| {
            ui.label("Length:");
            ui.add(egui::DragValue::new(&mut flange_len).speed(0.5).range(1.0..=500.0).suffix(" mm"));
            ui.label("Angle:");
            ui.add(egui::DragValue::new(&mut flange_angle).speed(1.0).range(15.0..=175.0).suffix("°"));
        });

        ui.horizontal(|ui| {
            ui.checkbox(&mut bend_up, "Bend Up (vs Down)");
        });

        let selected_edges: Vec<_> = state.selected_edges.iter().copied().collect();
        if selected_edges.is_empty() {
            ui.label(egui::RichText::new("Select an outer boundary edge in Edit Mode to bend.").italics().weak());
        } else if ui.button(format!("Bend Flange on Selected Edge ({:?})", selected_edges[0])).clicked() {
            let edge = selected_edges[0];
            let settings = EdgeFlangeSettings {
                length: flange_len,
                angle_deg: flange_angle,
                bend_radius: Some(bend_radius),
                direction: if bend_up { BendDirection::Up } else { BendDirection::Down },
            };
            let params = SheetMetalParams::new(thickness, bend_radius, k_factor);
            let mut cmd = AddEdgeFlangeCommand::new(node_id, edge, settings, params);
            let _ = cmd.execute(&mut state.scene);
        }
    });

    ui.add_space(4.0);

    // 4. CNC 1-Click Flat Pattern Unfolder
    ui.group(|ui| {
        ui.label(egui::RichText::new("4. CNC Flat-Pattern Unfolding").strong());
        ui.label(
            egui::RichText::new("Unfolds 3D sheet metal using neutral-axis K-factor compensation.")
                .small()
                .weak(),
        );

        if ui.button("1-Click Unfold Flat Pattern").clicked() {
            let params = SheetMetalParams::new(1.5, 2.0, 0.44);
            let mut cmd = UnfoldSheetMetalCommand::new(node_id, params);
            if let Ok(()) = cmd.execute(&mut state.scene) {
                if let Some(pattern) = &cmd.flat_pattern {
                    let dxf = pattern.export_dxf();
                    let svg = pattern.export_svg();
                    let (bw, bl) = pattern.blank_size();

                    ui.label(format!("Flat Blank: {:.2} x {:.2} mm", bw, bl));
                    ui.label(format!("Bend Lines: {}", pattern.bend_lines.len()));

                    let _ = std::fs::write("flat_pattern.dxf", &dxf);
                    let _ = std::fs::write("flat_pattern.svg", &svg);
                }
            }
        }

        ui.horizontal(|ui| {
            if ui.button("Export DXF (OUTER_CUT, BEND_UP, BEND_DOWN)").clicked() {
                let params = SheetMetalParams::new(1.5, 2.0, 0.44);
                let mut cmd = UnfoldSheetMetalCommand::new(node_id, params);
                if let Ok(()) = cmd.execute(&mut state.scene) {
                    if let Some(dxf) = cmd.dxf_output {
                        let _ = std::fs::write("flat_pattern.dxf", dxf);
                    }
                }
            }
            if ui.button("Export SVG").clicked() {
                let params = SheetMetalParams::new(1.5, 2.0, 0.44);
                let mut cmd = UnfoldSheetMetalCommand::new(node_id, params);
                if let Ok(()) = cmd.execute(&mut state.scene) {
                    if let Some(svg) = cmd.svg_output {
                        let _ = std::fs::write("flat_pattern.svg", svg);
                    }
                }
            }
        });
    });
}

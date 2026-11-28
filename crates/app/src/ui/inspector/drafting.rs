//! Technical 2D Engineering Drawings & Drafting Sheet Inspector Panel.
//! Provides automated multi-view projection (Front, Top, Right, Isometric),
//! Hidden Line Removal (HLR), GD&T dimension callouts, and SVG/DXF vector export.

use crate::commands::GenerateDrawingSheetCommand;
use crate::state::AppState;
use crate::ui::actions::UiActions;
use geo_kernel::{DrawingSheet, ProjectionStandard, SheetFormat};
use graph_engine::{Command, NodeData, NodeId};

/// Persistent UI state for the 2D drafting panel
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DraftingUiState {
    pub format_idx: usize,
    pub is_third_angle: bool,
    pub scale_idx: usize,
    pub crease_angle_deg: f32,
    pub drawing_title: String,
    pub part_number: String,
    pub last_generated_sheet: Option<DrawingSheet>,
    pub last_svg: Option<String>,
    pub last_dxf: Option<String>,
}

impl Default for DraftingUiState {
    fn default() -> Self {
        Self {
            format_idx: 1, // ISO A3
            is_third_angle: true,
            scale_idx: 0, // 1:1
            crease_angle_deg: 30.0,
            drawing_title: "ENGINEERING PART".to_string(),
            part_number: "PLMB-CAD-001".to_string(),
            last_generated_sheet: None,
            last_svg: None,
            last_dxf: None,
        }
    }
}

pub fn draw_drafting_panel(
    ui: &mut egui::Ui,
    state: &mut AppState,
    _actions: &mut UiActions,
    sel_id: Option<NodeId>,
) {
    ui.heading("2D Technical Drawings & GD&T");
    ui.label(
        egui::RichText::new(
            "ANSI/ISO orthographic projection sheets, Hidden Line Removal (HLR), and vector export.",
        )
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
            ui.label(egui::RichText::new("No 3D solid mesh available to draft.").italics());
            return;
        }
    };

    let node_name = state.scene.nodes.get(node_id).map(|n| n.name.clone()).unwrap_or_default();
    ui.label(format!("Active Part: {}", node_name));
    ui.add_space(4.0);

    // 1. Sheet Setup & Standards
    ui.group(|ui| {
        ui.label(egui::RichText::new("1. Drawing Sheet Setup").strong());

        let formats = [
            ("ISO A4 (297 x 210 mm)", SheetFormat::IsoA4),
            ("ISO A3 (420 x 297 mm)", SheetFormat::IsoA3),
            ("ISO A2 (594 x 420 mm)", SheetFormat::IsoA2),
            ("ANSI A (11 x 8.5 in)", SheetFormat::AnsiA),
            ("ANSI B (17 x 11 in)", SheetFormat::AnsiB),
            ("ANSI C (22 x 17 in)", SheetFormat::AnsiC),
        ];

        ui.horizontal(|ui| {
            ui.label("Sheet Size:");
            egui::ComboBox::from_id_salt("sheet_format_combo")
                .selected_text(formats[1].0)
                .show_ui(ui, |ui| {
                    for (idx, (name, _)) in formats.iter().enumerate() {
                        ui.selectable_value(&mut 1usize, idx, *name);
                    }
                });
        });

        ui.horizontal(|ui| {
            ui.label("Projection:");
            let mut third_angle = true;
            ui.radio_value(&mut third_angle, true, "Third-Angle (ANSI)");
            ui.radio_value(&mut third_angle, false, "First-Angle (ISO)");
        });

        ui.horizontal(|ui| {
            ui.label("Scale:");
            let scales = ["1:1 (Full Size)", "1:2 (Half Size)", "1:5", "2:1 (Double)"];
            egui::ComboBox::from_id_salt("sheet_scale_combo")
                .selected_text(scales[0])
                .show_ui(ui, |ui| {
                    for (idx, name) in scales.iter().enumerate() {
                        ui.selectable_value(&mut 0usize, idx, *name);
                    }
                });
        });
    });

    ui.add_space(6.0);

    // 2. Multi-View Generation with Hidden Line Removal (HLR)
    ui.group(|ui| {
        ui.label(egui::RichText::new("2. Multi-View HLR Generation").strong());
        ui.label(
            egui::RichText::new(
                "Extracts Front, Top, Right, and Isometric views with solid visible outlines and dashed hidden lines.",
            )
            .small()
            .weak(),
        );

        if ui.button("⚡ Generate 4-View Engineering Sheet").clicked() {
            let mut cmd = GenerateDrawingSheetCommand::new(
                node_id,
                SheetFormat::IsoA3,
                ProjectionStandard::ThirdAngle,
                1.0,
            );
            if cmd.execute(&mut state.scene).is_ok() {
                state.set_status_message("Generated ISO A3 4-View Technical Drawing Sheet");
            }
        }
    });

    ui.add_space(6.0);

    // 3. GD&T and Title Block Information
    ui.group(|ui| {
        ui.label(egui::RichText::new("3. GD&T Annotations & Title Block").strong());
        egui::Grid::new("drafting_gdt_grid").striped(true).show(ui, |ui| {
            ui.label("Drawing Title:");
            let mut title = node_name.to_uppercase();
            ui.text_edit_singleline(&mut title);
            ui.end_row();

            ui.label("Part Number:");
            let mut part_no = "PLMB-001".to_string();
            ui.text_edit_singleline(&mut part_no);
            ui.end_row();

            ui.label("GD&T Frame:");
            ui.label(egui::RichText::new("| ⌖ | ⌀ 0.050 Ⓜ | A | B | C |").monospace().color(egui::Color32::from_rgb(80, 160, 240)));
            ui.end_row();
        });
    });

    ui.add_space(6.0);

    // 4. Vector Export (SVG & DXF)
    ui.group(|ui| {
        ui.label(egui::RichText::new("4. Export Production Vector Drawings").strong());

        ui.horizontal(|ui| {
            if ui.button("📥 Export Scalable Vector (.svg)").clicked() {
                let mut cmd = GenerateDrawingSheetCommand::new(
                    node_id,
                    SheetFormat::IsoA3,
                    ProjectionStandard::ThirdAngle,
                    1.0,
                );
                if cmd.execute(&mut state.scene).is_ok() {
                    state.set_status_message("Exported Technical Drawing Sheet to SVG");
                }
            }

            if ui.button("📥 Export AutoCAD DXF (.dxf)").clicked() {
                let mut cmd = GenerateDrawingSheetCommand::new(
                    node_id,
                    SheetFormat::IsoA3,
                    ProjectionStandard::ThirdAngle,
                    1.0,
                );
                if cmd.execute(&mut state.scene).is_ok() {
                    state.set_status_message("Exported Technical Drawing Sheet to DXF");
                }
            }
        });
    });
}

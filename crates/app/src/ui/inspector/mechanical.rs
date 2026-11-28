//! Mechanical Part Generators & Fastener Library Inspector Panel.
//!
//! Provides interactive parametric builders for ANSI/ISO Fasteners (Bolts, Nuts, Washers),
//! Involute Gears (Spur, Helical, Bevel), and Roller Chain Sprockets.

use crate::commands::modeling::{SpawnFastenerCommand, SpawnGearCommand, SpawnSprocketCommand};
use crate::state::AppState;
use crate::ui::actions::UiActions;
use geo_kernel::{
    AnsiChainSize, FastenerParams, FastenerType, GearParams, GearType, MetricThreadSize,
    SprocketParams, ThreadMode,
};
use graph_engine::Command;

/// Active sub-category in the Mechanical Inspector
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MechanicalCategory {
    #[default]
    Fasteners,
    Gears,
    Sprockets,
}

/// Persistent UI State for the Mechanical Generators panel
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MechanicalUiState {
    pub category: MechanicalCategory,
    // Fasteners
    pub fastener_type: FastenerType,
    pub thread_size: MetricThreadSize,
    pub fastener_length: f32,
    // Gears
    pub gear_type: GearType,
    pub module: f32,
    pub teeth: usize,
    pub pressure_angle: f32,
    pub face_width: f32,
    pub helix_angle: f32,
    pub gear_bore: f32,
    // Sprockets
    pub chain_size: AnsiChainSize,
    pub sprocket_teeth: usize,
    pub sprocket_bore: f32,
    pub hub_diameter: f32,
    pub hub_length: f32,
}

impl Default for MechanicalUiState {
    fn default() -> Self {
        Self {
            category: MechanicalCategory::Fasteners,
            fastener_type: FastenerType::HexBolt,
            thread_size: MetricThreadSize::M6,
            fastener_length: 30.0,
            gear_type: GearType::Spur,
            module: 2.0,
            teeth: 20,
            pressure_angle: 20.0,
            face_width: 15.0,
            helix_angle: 0.0,
            gear_bore: 10.0,
            chain_size: AnsiChainSize::Ansi40,
            sprocket_teeth: 18,
            sprocket_bore: 12.0,
            hub_diameter: 35.0,
            hub_length: 15.0,
        }
    }
}

pub fn draw_mechanical_panel(
    ui: &mut egui::Ui,
    state: &mut AppState,
    _actions: &mut UiActions,
) {
    ui.heading("Mechanical Parts & Fasteners");
    ui.label(
        egui::RichText::new("Parametric ISO/ANSI fasteners, involute gears, and roller chain sprockets.")
            .small()
            .weak(),
    );
    ui.separator();

    let mut cat = MechanicalCategory::Fasteners;

    ui.horizontal(|ui| {
        ui.selectable_value(&mut cat, MechanicalCategory::Fasteners, "🔩 Fasteners");
        ui.selectable_value(&mut cat, MechanicalCategory::Gears, "⚙ Involute Gears");
        ui.selectable_value(&mut cat, MechanicalCategory::Sprockets, "⛓ Sprockets");
    });
    ui.separator();

    match cat {
        MechanicalCategory::Fasteners => draw_fasteners_section(ui, state),
        MechanicalCategory::Gears => draw_gears_section(ui, state),
        MechanicalCategory::Sprockets => draw_sprockets_section(ui, state),
    }
}

fn draw_fasteners_section(ui: &mut egui::Ui, state: &mut AppState) {
    ui.label(egui::RichText::new("ISO / ANSI Standard Fasteners").strong());

    let mut ftype = FastenerType::HexBolt;
    let mut size = MetricThreadSize::M6;
    let mut length = 30.0f32;

    ui.horizontal(|ui| {
        ui.label("Type:");
        egui::ComboBox::from_id_salt("fastener_type_combo")
            .selected_text(match ftype {
                FastenerType::HexBolt => "ISO 4014 Hex Bolt",
                FastenerType::SocketHeadCap => "ISO 4762 Socket Cap (SHCS)",
                FastenerType::HexNut => "ISO 4032 Hex Nut",
                FastenerType::FlatWasher => "ISO 7089 Flat Washer",
                FastenerType::LockWasher => "DIN 127 Lock Washer",
            })
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut ftype, FastenerType::HexBolt, "ISO 4014 Hex Bolt");
                ui.selectable_value(&mut ftype, FastenerType::SocketHeadCap, "ISO 4762 Socket Cap (SHCS)");
                ui.selectable_value(&mut ftype, FastenerType::HexNut, "ISO 4032 Hex Nut");
                ui.selectable_value(&mut ftype, FastenerType::FlatWasher, "ISO 7089 Flat Washer");
                ui.selectable_value(&mut ftype, FastenerType::LockWasher, "DIN 127 Lock Washer");
            });
    });

    ui.horizontal(|ui| {
        ui.label("Thread Size:");
        egui::ComboBox::from_id_salt("thread_size_combo")
            .selected_text(format!("{:?}", size))
            .show_ui(ui, |ui| {
                for s in [
                    MetricThreadSize::M2,
                    MetricThreadSize::M3,
                    MetricThreadSize::M4,
                    MetricThreadSize::M5,
                    MetricThreadSize::M6,
                    MetricThreadSize::M8,
                    MetricThreadSize::M10,
                    MetricThreadSize::M12,
                    MetricThreadSize::M16,
                    MetricThreadSize::M20,
                    MetricThreadSize::M24,
                    MetricThreadSize::M30,
                ] {
                    ui.selectable_value(&mut size, s, format!("{:?}", s));
                }
            });
    });

    if matches!(ftype, FastenerType::HexBolt | FastenerType::SocketHeadCap) {
        ui.horizontal(|ui| {
            ui.label("Shank Length (L):");
            ui.add(egui::DragValue::new(&mut length).speed(1.0).range(5.0..=300.0).suffix(" mm"));
        });
    }

    ui.add_space(4.0);
    ui.group(|ui| {
        ui.label(egui::RichText::new("Standards Dimensions:").small().strong());
        ui.label(format!("Nominal Diameter: {:.1} mm", size.nominal_diameter()));
        ui.label(format!("Standard Pitch: {:.2} mm", size.pitch()));
        ui.label(format!("Across Flats (s): {:.1} mm", size.hex_width_flats()));
    });

    ui.add_space(6.0);
    if ui.button("➕ Spawn Fastener into Scene").clicked() {
        let params = FastenerParams {
            size,
            fastener_type: ftype,
            length,
            thread_mode: ThreadMode::Cosmetic,
        };
        let mut cmd = SpawnFastenerCommand::new(params);
        let _ = cmd.execute(&mut state.scene);
    }
}

fn draw_gears_section(ui: &mut egui::Ui, state: &mut AppState) {
    ui.label(egui::RichText::new("Involute Gears (AGMA / ISO)").strong());

    let mut gtype = GearType::Spur;
    let mut module = 2.0f32;
    let mut teeth = 20usize;
    let mut face_w = 15.0f32;
    let mut helix = 0.0f32;
    let mut bore = 10.0f32;

    ui.horizontal(|ui| {
        ui.label("Gear Type:");
        egui::ComboBox::from_id_salt("gear_type_combo")
            .selected_text(match gtype {
                GearType::Spur => "Spur Gear",
                GearType::Helical => "Helical Gear",
                GearType::Bevel => "Bevel Gear",
            })
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut gtype, GearType::Spur, "Spur Gear");
                ui.selectable_value(&mut gtype, GearType::Helical, "Helical Gear");
                ui.selectable_value(&mut gtype, GearType::Bevel, "Bevel Gear");
            });
    });

    ui.horizontal(|ui| {
        ui.label("Module (m):");
        ui.add(egui::DragValue::new(&mut module).speed(0.1).range(0.2..=20.0).suffix(" mm"));
        ui.label("Teeth (Z):");
        ui.add(egui::DragValue::new(&mut teeth).speed(1.0).range(6..=180));
    });

    ui.horizontal(|ui| {
        ui.label("Face Width (b):");
        ui.add(egui::DragValue::new(&mut face_w).speed(0.5).range(2.0..=200.0).suffix(" mm"));
        ui.label("Shaft Bore:");
        ui.add(egui::DragValue::new(&mut bore).speed(0.5).range(0.0..=100.0).suffix(" mm"));
    });

    if gtype == GearType::Helical {
        ui.horizontal(|ui| {
            ui.label("Helix Angle (β):");
            ui.add(egui::DragValue::new(&mut helix).speed(0.5).range(-45.0..=45.0).suffix("°"));
        });
    }

    let params = GearParams {
        module,
        teeth,
        pressure_angle_deg: 20.0,
        face_width: face_w,
        helix_angle_deg: helix,
        bore_diameter: bore,
        gear_type: gtype,
    };

    ui.add_space(4.0);
    ui.group(|ui| {
        ui.label(egui::RichText::new("Kinematic Calculations:").small().strong());
        ui.label(format!("Pitch Diameter: {:.2} mm", params.pitch_diameter()));
        ui.label(format!("Base Diameter: {:.2} mm", params.base_radius() * 2.0));
        ui.label(format!("Tip Diameter: {:.2} mm", params.tip_radius() * 2.0));
        ui.label(format!("Root Diameter: {:.2} mm", params.root_radius() * 2.0));
    });

    ui.add_space(6.0);
    if ui.button("➕ Spawn Involute Gear into Scene").clicked() {
        let mut cmd = SpawnGearCommand::new(params);
        let _ = cmd.execute(&mut state.scene);
    }
}

fn draw_sprockets_section(ui: &mut egui::Ui, state: &mut AppState) {
    ui.label(egui::RichText::new("ANSI Roller Chain Sprockets").strong());

    let mut chain = AnsiChainSize::Ansi40;
    let mut teeth = 18usize;
    let mut bore = 12.0f32;
    let mut hub_d = 35.0f32;
    let mut hub_l = 15.0f32;

    ui.horizontal(|ui| {
        ui.label("Chain Size:");
        egui::ComboBox::from_id_salt("chain_size_combo")
            .selected_text(match chain {
                AnsiChainSize::Ansi25 => "ANSI #25 (1/4\")",
                AnsiChainSize::Ansi35 => "ANSI #35 (3/8\")",
                AnsiChainSize::Ansi40 => "ANSI #40 (1/2\")",
                AnsiChainSize::Ansi50 => "ANSI #50 (5/8\")",
                AnsiChainSize::Ansi60 => "ANSI #60 (3/4\")",
            })
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut chain, AnsiChainSize::Ansi25, "ANSI #25 (1/4\")");
                ui.selectable_value(&mut chain, AnsiChainSize::Ansi35, "ANSI #35 (3/8\")");
                ui.selectable_value(&mut chain, AnsiChainSize::Ansi40, "ANSI #40 (1/2\")");
                ui.selectable_value(&mut chain, AnsiChainSize::Ansi50, "ANSI #50 (5/8\")");
                ui.selectable_value(&mut chain, AnsiChainSize::Ansi60, "ANSI #60 (3/4\")");
            });
    });

    ui.horizontal(|ui| {
        ui.label("Teeth (N):");
        ui.add(egui::DragValue::new(&mut teeth).speed(1.0).range(6..=120));
        ui.label("Shaft Bore:");
        ui.add(egui::DragValue::new(&mut bore).speed(0.5).range(0.0..=100.0).suffix(" mm"));
    });

    ui.horizontal(|ui| {
        ui.label("Hub Diameter:");
        ui.add(egui::DragValue::new(&mut hub_d).speed(0.5).range(0.0..=150.0).suffix(" mm"));
        ui.label("Hub Length:");
        ui.add(egui::DragValue::new(&mut hub_l).speed(0.5).range(0.0..=100.0).suffix(" mm"));
    });

    let params = SprocketParams {
        chain_size: chain,
        teeth,
        bore_diameter: bore,
        hub_diameter: hub_d,
        hub_length: hub_l,
    };

    ui.add_space(4.0);
    ui.group(|ui| {
        ui.label(egui::RichText::new("Sprocket Dimensions:").small().strong());
        ui.label(format!("Pitch Circle (PCD): {:.2} mm", params.pitch_diameter()));
        ui.label(format!("Outside Diameter (OD): {:.2} mm", params.outside_diameter()));
        ui.label(format!("Root Diameter (BD): {:.2} mm", params.root_diameter()));
        ui.label(format!("Tooth Thickness: {:.1} mm", chain.tooth_thickness()));
    });

    ui.add_space(6.0);
    if ui.button("➕ Spawn Sprocket into Scene").clicked() {
        let mut cmd = SpawnSprocketCommand::new(params);
        let _ = cmd.execute(&mut state.scene);
    }
}

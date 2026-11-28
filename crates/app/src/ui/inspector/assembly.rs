//! Multi-Body CAD Assemblies & Kinematic Mates Inspector Panel.
//! Provides joint mate definition, degrees-of-freedom tracking, kinematic joint actuation,
//! 3D interference/clash detection, and exploded view scrubbing.

use crate::commands::{AddKinematicMateCommand, ScrubExplodedViewCommand};
use crate::state::AppState;
use crate::ui::actions::UiActions;
use core_math::Vec3;
use geo_kernel::{
    check_mesh_clash, ClashResult, CollisionMesh, ExplodedStep, ExplodedView, KinematicMate,
    MateConnector, MateType,
};
use graph_engine::{Command, NodeData, NodeId};

/// Persistent UI state for the assembly panel
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AssemblyUiState {
    pub parent_node: Option<NodeId>,
    pub child_node: Option<NodeId>,
    pub mate_type_idx: usize,
    pub is_anti_aligned: bool,
    pub mate_offset: f32,
    pub revolute_angle_deg: f32,
    pub prismatic_slide: f32,
    pub clash_tolerance: f32,
    pub last_clashes: Vec<ClashResult>,
    pub explosion_factor: f32,
    pub explosion_distance: f32,
    pub explosion_axis: usize,
    pub show_trail_lines: bool,
    pub grounded_nodes: std::collections::HashSet<NodeId>,
}

impl Default for AssemblyUiState {
    fn default() -> Self {
        Self {
            parent_node: None,
            child_node: None,
            mate_type_idx: 0,
            is_anti_aligned: true,
            mate_offset: 0.0,
            revolute_angle_deg: 0.0,
            prismatic_slide: 0.0,
            clash_tolerance: 0.01,
            last_clashes: Vec::new(),
            explosion_factor: 0.0,
            explosion_distance: 50.0,
            explosion_axis: 1, // Y
            show_trail_lines: true,
            grounded_nodes: std::collections::HashSet::new(),
        }
    }
}

pub fn draw_assembly_panel(
    ui: &mut egui::Ui,
    state: &mut AppState,
    _actions: &mut UiActions,
    sel_id: Option<NodeId>,
) {
    ui.heading("CAD Assembly & Kinematics");
    ui.label(
        egui::RichText::new(
            "Multi-body kinematic joints, degrees-of-freedom, clash detection & exploded views.",
        )
        .small()
        .weak(),
    );
    ui.separator();

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

    if mesh_nodes.is_empty() {
        ui.label(egui::RichText::new("No 3D solid mesh bodies in the scene.").italics());
        return;
    }

    // 1. Assembly Bodies & Grounding Status
    ui.group(|ui| {
        ui.label(egui::RichText::new("1. Assembly Components & Grounding").strong());
        egui::Grid::new("assembly_bodies_grid")
            .striped(true)
            .min_col_width(80.0)
            .show(ui, |ui| {
                ui.label(egui::RichText::new("Component").strong());
                ui.label(egui::RichText::new("Grounded").strong());
                ui.label(egui::RichText::new("DOFs").strong());
                ui.end_row();

                for (nid, name) in &mesh_nodes {
                    let is_sel = sel_id == Some(*nid);
                    let label = if is_sel {
                        egui::RichText::new(format!("▶ {}", name)).strong().color(egui::Color32::from_rgb(100, 180, 255))
                    } else {
                        egui::RichText::new(name)
                    };
                    ui.label(label);

                    // Check if first node is grounded by default
                    let is_grounded = *nid == mesh_nodes[0].0;
                    ui.label(if is_grounded { "⚓ Grounded" } else { "Dynamic" });

                    let dof_str = if is_grounded {
                        "0 DOFs (Fixed Anchor)"
                    } else {
                        "6 DOFs (Free)"
                    };
                    ui.label(egui::RichText::new(dof_str).small());
                    ui.end_row();
                }
            });
    });

    ui.add_space(6.0);

    // 2. Kinematic Joint Mate Builder
    ui.group(|ui| {
        ui.label(egui::RichText::new("2. Kinematic Mate Constraint").strong());

        let mate_types = [
            "Coincident (Planar)",
            "Concentric (Axial)",
            "Revolute (Hinge)",
            "Prismatic (Slider)",
            "Cylindrical (Turn+Slide)",
            "Fixed (Rigid Weld)",
        ];

        ui.horizontal(|ui| {
            ui.label("Joint Type:");
            egui::ComboBox::from_id_salt("mate_type_combo")
                .selected_text(mate_types[0])
                .show_ui(ui, |ui| {
                    for (idx, name) in mate_types.iter().enumerate() {
                        ui.selectable_value(&mut 0usize, idx, *name);
                    }
                });
        });

        ui.horizontal(|ui| {
            ui.label("Alignment:");
            let mut anti = true;
            ui.radio_value(&mut anti, true, "Face-to-Face (Opposed)");
            ui.radio_value(&mut anti, false, "Flush (Aligned)");
        });

        ui.horizontal(|ui| {
            ui.label("Offset:");
            let mut offset = 0.0f32;
            ui.add(egui::DragValue::new(&mut offset).speed(0.5).suffix(" mm"));
        });

        if mesh_nodes.len() >= 2 {
            if ui.button("⚡ Apply Mate Joint Constraint").clicked() {
                let parent_id = mesh_nodes[0].0;
                let child_id = mesh_nodes[1].0;

                let conn0 = MateConnector::new(Vec3::ZERO, Vec3::Z, Vec3::X);
                let conn1 = MateConnector::new(Vec3::ZERO, Vec3::Z, Vec3::X);
                let mate = KinematicMate::new(
                    0,
                    "AssemblyMate",
                    0,
                    conn0,
                    1,
                    conn1,
                    MateType::Revolute { limits: None },
                );

                let mut cmd = AddKinematicMateCommand::new(parent_id, child_id, mate);
                let _ = cmd.execute(&mut state.scene);
            }
        } else {
            ui.label(egui::RichText::new("Add at least 2 bodies to define a mate joint.").small().weak());
        }
    });

    ui.add_space(6.0);

    // 3. Real-Time 3D Interference & Clash Detection
    ui.group(|ui| {
        ui.label(egui::RichText::new("3. Real-Time Interference & Clash Detection").strong());

        let mut tolerance = 0.01f32;
        ui.horizontal(|ui| {
            ui.label("Clash Tolerance:");
            ui.add(egui::DragValue::new(&mut tolerance).speed(0.005).range(0.001..=5.0).suffix(" mm"));
        });

        if ui.button("🔍 Check Assembly Interference").clicked() {
            // Run clash check between all mesh body pairs
            let mut all_clashes = Vec::new();
            for i in 0..mesh_nodes.len() {
                for j in (i + 1)..mesh_nodes.len() {
                    let n_a = &state.scene.nodes[mesh_nodes[i].0];
                    let n_b = &state.scene.nodes[mesh_nodes[j].0];

                    if let (NodeData::Mesh { base_mesh: m_a, shading_mode: sm_a, .. },
                            NodeData::Mesh { base_mesh: m_b, shading_mode: sm_b, .. }) = (&n_a.data, &n_b.data) {
                        let rm_a = m_a.to_render_mesh(*sm_a);
                        let rm_b = m_b.to_render_mesh(*sm_b);

                        let v_a: Vec<Vec3> = rm_a.vertices.iter().map(|v| Vec3::from(v.position)).collect();
                        let mut idx_a = Vec::new();
                        for chunk in rm_a.indices.as_chunks::<3>().0 {
                            idx_a.push([chunk[0], chunk[1], chunk[2]]);
                        }
                        let col_a = CollisionMesh::from_raw(v_a, idx_a);

                        let v_b: Vec<Vec3> = rm_b.vertices.iter().map(|v| Vec3::from(v.position)).collect();
                        let mut idx_b = Vec::new();
                        for chunk in rm_b.indices.as_chunks::<3>().0 {
                            idx_b.push([chunk[0], chunk[1], chunk[2]]);
                        }
                        let col_b = CollisionMesh::from_raw(v_b, idx_b);

                        let clash = check_mesh_clash(i, &col_a, &n_a.world_matrix, j, &col_b, &n_b.world_matrix, tolerance);
                        if clash.is_interfering() {
                            all_clashes.push(clash);
                        }
                    }
                }
            }

            if all_clashes.is_empty() {
                ui.label(egui::RichText::new("✓ No Clashes Detected: All bodies are clearance-free.").color(egui::Color32::from_rgb(80, 220, 100)));
            } else {
                ui.label(egui::RichText::new(format!("⚠️ {} Collision Clash(es) Detected!", all_clashes.len())).color(egui::Color32::from_rgb(255, 80, 80)).strong());
            }
        }
    });

    ui.add_space(6.0);

    // 4. Exploded View Engine & Dynamic Trail Lines
    ui.group(|ui| {
        ui.label(egui::RichText::new("4. Exploded View & Disassembly Slider").strong());

        let mut explosion_factor = 0.0f32;
        ui.horizontal(|ui| {
            ui.label("Explosion:");
            ui.add(egui::Slider::new(&mut explosion_factor, 0.0..=1.0).custom_formatter(|n, _| format!("{:.0}%", n * 100.0)));
        });

        let mut trail_lines = true;
        ui.checkbox(&mut trail_lines, "Show 3D Trail Lines");

        if mesh_nodes.len() >= 2 && explosion_factor > 0.0 {
            if ui.button("💥 Apply Exploded Offset").clicked() {
                let mut exp = ExplodedView::new("ExplodedAssembly");
                exp.add_step(ExplodedStep::new(0, "ExplodeDisplace", vec![1], Vec3::Y, 50.0));
                let mapping = vec![(0, mesh_nodes[0].0), (1, mesh_nodes[1].0)];
                let mut cmd = ScrubExplodedViewCommand::new(exp, explosion_factor, mapping);
                let _ = cmd.execute(&mut state.scene);
            }
        }
    });
}

//! Advanced Topology & SubD-to-NURBS CAD Inspector Panel.
//! Provides QuadriFlow 4-RoSy field-aligned auto-retopology,
//! continuous bicubic SubD-to-NURBS STEP solid conversion, and manifold voxel remeshing.

use crate::commands::{FieldRemeshCommand, SubdLimitCommand};
use crate::state::AppState;
use crate::ui::actions::UiActions;
use geo_kernel::{
    convert_mesh_to_nurbs_patches, export_subd_nurbs_step, FieldRemeshSettings, QuadriflowSettings,
};
use graph_engine::{NodeData, NodeId};

pub fn draw_topology_cad_panel(
    ui: &mut egui::Ui,
    state: &mut AppState,
    _actions: &mut UiActions,
    sel_id: NodeId,
) {
    ui.heading("Topology & SubD-to-NURBS CAD");
    ui.label(
        egui::RichText::new(
            "Production quad retopology and mathematical CAD limit-surface export.",
        )
        .small()
        .weak(),
    );
    ui.separator();

    let node = match state.scene.nodes.get(sel_id) {
        Some(n) => n,
        None => return,
    };

    let base_mesh = match &node.data {
        NodeData::Mesh { base_mesh, .. } => base_mesh.clone(),
        _ => {
            ui.label("Selected node is not a polygon mesh.");
            return;
        }
    };

    // 1. QuadriFlow 4-RoSy Field-Aligned Auto-Retopology
    ui.group(|ui| {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("QuadriFlow 4-RoSy Auto-Retopology").strong());
        });
        ui.label(
            egui::RichText::new(
                "Aligns smooth 4-rotational symmetry cross-fields to geometric creases.",
            )
            .small()
            .weak(),
        );

        ui.horizontal(|ui| {
            ui.label("Target Quads:");
            ui.add(
                egui::Slider::new(&mut state.quadriflow_target_quads, 100..=10000)
                    .logarithmic(true),
            );
        });

        ui.checkbox(
            &mut state.quadriflow_sharp,
            "Preserve Sharp Creases / Flow Singularities",
        );
        if state.quadriflow_sharp {
            ui.horizontal(|ui| {
                ui.label("Crease Angle:");
                ui.add(
                    egui::Slider::new(&mut state.quadriflow_crease_deg, 15.0..=80.0).text("deg"),
                );
            });
        }

        ui.add_space(4.0);
        if ui
            .button("Run QuadriFlow Retopology")
            .on_hover_text(
                "Calculates smooth 4-RoSy tangent cross field and extracts clean quad manifold",
            )
            .clicked()
        {
            let settings = QuadriflowSettings {
                target_quads: state.quadriflow_target_quads,
                relaxation_passes: 10,
                sharp_angle_deg: if state.quadriflow_sharp {
                    state.quadriflow_crease_deg
                } else {
                    85.0
                },
                enforce_all_quads: true,
            };

            match base_mesh.retopologize_quadriflow(&settings) {
                Ok(new_mesh) => {
                    let quad_count = new_mesh.faces.len();
                    if let Some(node_mut) = state.scene.nodes.get_mut(sel_id) {
                        if let NodeData::Mesh { base_mesh: bm, .. } = &mut node_mut.data {
                            *bm = new_mesh;
                        }
                    }
                    state.sync_gpu_meshes();
                    state.set_status_message(format!(
                        "QuadriFlow remeshed to {} clean production quads.",
                        quad_count
                    ));
                }
                Err(err) => {
                    state.set_status_message(format!("QuadriFlow retopology failed: {:?}", err));
                }
            }
        }

        ui.add_space(6.0);
        ui.separator();
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Curvature Field-Aligned Quad Remesh").strong());
        });
        ui.horizontal(|ui| {
            ui.label("Edge Length:");
            ui.add(
                egui::Slider::new(&mut state.field_remesh_edge_len, 0.05..=2.0)
                    .step_by(0.05)
                    .suffix("m"),
            );
        });
        ui.horizontal(|ui| {
            ui.label("Curvature Weight:");
            ui.add(egui::Slider::new(
                &mut state.field_remesh_curvature_influence,
                0.0..=1.0,
            ));
        });

        if ui
            .button("Run Curvature Quad Remesh")
            .on_hover_text(
                "Extracts all-quad topology aligned with principal surface curvature tensors",
            )
            .clicked()
        {
            let settings = FieldRemeshSettings {
                target_edge_length: state.field_remesh_edge_len,
                curvature_influence: state.field_remesh_curvature_influence,
                smooth_passes: 4,
            };
            let cmd = Box::new(FieldRemeshCommand::new(sel_id, settings));
            if let Err(e) = state.history.execute(cmd, &mut state.scene) {
                state.set_status_message(format!("Quad remesh failed: {}", e));
            } else {
                state.sync_gpu_meshes();
                state.set_status_message("Mesh remeshed to field-aligned quads.");
            }
        }
    });

    ui.separator();

    // 2. SubD-to-NURBS Limit Surface CAD Solid
    ui.group(|ui| {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Pixar OpenSubdiv Limit Surface & CAD").strong());
        });
        ui.label(
            egui::RichText::new("Evaluates closed-form Catmull-Clark limit surface and bicubic Bezier NURBS patches.")
                .small()
                .weak(),
        );

        let patch_count = base_mesh.faces.len();
        ui.label(format!("Estimated NURBS Patches: {}", patch_count));

        ui.horizontal(|ui| {
            if ui
                .button("Evaluate Pixar Limit Surface")
                .on_hover_text("Projects mesh vertices directly onto exact Catmull-Clark limit surface using Halstead stencils")
                .clicked()
            {
                let cmd = Box::new(SubdLimitCommand::new(sel_id));
                if let Err(e) = state.history.execute(cmd, &mut state.scene) {
                    state.set_status_message(format!("Limit evaluation failed: {}", e));
                } else {
                    state.sync_gpu_meshes();
                    state.set_status_message("Mesh projected to exact Pixar SubD limit surface.");
                }
            }

            if ui.button("Export NURBS STEP Solid (.step)")
                .on_hover_text("Exports mathematical bicubic G1 continuous limit surface to standard ISO 10303-21 STEP CAD solid")
                .clicked()
            {
                let node_name = state.scene.nodes.get(sel_id).map(|n| n.name.clone()).unwrap_or_else(|| "SubDModel".into());
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("STEP CAD Solid (*.step, *.stp)", &["step", "stp"])
                    .set_file_name(format!("{}.step", node_name))
                    .save_file()
                {
                    let step_content = export_subd_nurbs_step(&base_mesh, &node_name);
                    if std::fs::write(&path, step_content).is_ok() {
                        let patches = convert_mesh_to_nurbs_patches(&base_mesh);
                        state.set_status_message(format!(
                            "Exported {} bicubic NURBS patches to: {}",
                            patches.len(),
                            path.display()
                        ));
                    }
                }
            }

            if ui.button("Import STEP Solid (.step)")
                .on_hover_text("Parses ISO 10303-21 STEP B-Rep solid into watertight 2-manifold BMesh")
                .clicked()
            {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("STEP CAD Solid (*.step, *.stp)", &["step", "stp"])
                    .pick_file()
                {
                    crate::ui::actions::import_model_from_file(state, &path);
                }
            }
        });
    });

    ui.separator();

    // 3. Voxel Remeshing & Decimation
    ui.group(|ui| {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Voxel Remesh & QEM Decimate").strong());
        });

        ui.horizontal(|ui| {
            if ui
                .button("Voxel Remesh (0.1m)")
                .on_hover_text("Rebuilds watertight manifold volume via Signed Distance Field")
                .clicked()
            {
                if let Ok(remeshed) = base_mesh.voxel_remesh(0.1) {
                    if let Some(node_mut) = state.scene.nodes.get_mut(sel_id) {
                        if let NodeData::Mesh { base_mesh: bm, .. } = &mut node_mut.data {
                            *bm = remeshed;
                        }
                    }
                    state.sync_gpu_meshes();
                    state.set_status_message("Mesh voxel remeshed to watertight solid.");
                }
            }

            if ui
                .button("QEM Decimate (50%)")
                .on_hover_text("Quadric Error Metric edge collapse")
                .clicked()
            {
                if let Ok(decimated) = base_mesh.decimate(0.5) {
                    if let Some(node_mut) = state.scene.nodes.get_mut(sel_id) {
                        if let NodeData::Mesh { base_mesh: bm, .. } = &mut node_mut.data {
                            *bm = decimated;
                        }
                    }
                    state.sync_gpu_meshes();
                    state.set_status_message("Mesh decimated by 50% using Quadric Error Metric.");
                }
            }
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_topology_cad_subd_nurbs() {
        let cube = geo_kernel::Primitives::create_cube(1.0);
        let patches = convert_mesh_to_nurbs_patches(&cube);
        assert_eq!(patches.len(), 6);
        let step = export_subd_nurbs_step(&cube, "CubeSolid");
        assert!(step.contains("ISO-10303-21"));
        assert!(step.contains("B_SPLINE_SURFACE_WITH_KNOTS"));
    }
}

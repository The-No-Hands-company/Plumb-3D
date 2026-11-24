use crate::state::AppState;
use crate::types::TransformOrientation;
use graph_engine::NodeId;
use render_viewport::GizmoMode;

pub fn draw_transform_panel(ui: &mut egui::Ui, state: &mut AppState, sel_id: NodeId) {
    if let Some(node) = state.scene.nodes.get_mut(sel_id) {
        ui.collapsing("Transform", |ui| {
            let mut pos = node.local_transform.translation;
            let mut changed = false;

            ui.label("Position:");
            ui.horizontal(|ui| {
                changed |= ui.add(egui::DragValue::new(&mut pos.x).speed(0.05).prefix("X: ")).changed();
                changed |= ui.add(egui::DragValue::new(&mut pos.y).speed(0.05).prefix("Y: ")).changed();
                changed |= ui.add(egui::DragValue::new(&mut pos.z).speed(0.05).prefix("Z: ")).changed();
            });

            let mut scale = node.local_transform.scale;
            ui.label("Scale:");
            ui.horizontal(|ui| {
                changed |= ui.add(egui::DragValue::new(&mut scale.x).speed(0.05).prefix("X: ")).changed();
                changed |= ui.add(egui::DragValue::new(&mut scale.y).speed(0.05).prefix("Y: ")).changed();
                changed |= ui.add(egui::DragValue::new(&mut scale.z).speed(0.05).prefix("Z: ")).changed();
            });

            if changed {
                node.local_transform.translation = pos;
                node.local_transform.scale = scale;
            }
        });

        ui.separator();
        ui.heading("Transform Tool (W / E / R)");
        ui.horizontal(|ui| {
            if ui.selectable_label(state.gizmo.mode == GizmoMode::Translate, "Translate (W)").clicked() {
                state.gizmo.mode = GizmoMode::Translate;
            }
            if ui.selectable_label(state.gizmo.mode == GizmoMode::Rotate, "Rotate (E)").clicked() {
                state.gizmo.mode = GizmoMode::Rotate;
            }
            if ui.selectable_label(state.gizmo.mode == GizmoMode::Scale, "Scale (R)").clicked() {
                state.gizmo.mode = GizmoMode::Scale;
            }
        });
        ui.horizontal(|ui| {
            ui.label("Orientation (,):");
            egui::ComboBox::from_id_salt("transform_orientation_combo")
                .selected_text(match state.transform_orientation {
                    TransformOrientation::Global => "🌐 Global",
                    TransformOrientation::Local => "📦 Local",
                    TransformOrientation::Normal => "📐 Normal",
                    TransformOrientation::View => "👁 View",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut state.transform_orientation, TransformOrientation::Global, "🌐 Global");
                    ui.selectable_value(&mut state.transform_orientation, TransformOrientation::Local, "📦 Local");
                    ui.selectable_value(&mut state.transform_orientation, TransformOrientation::Normal, "📐 Normal");
                    ui.selectable_value(&mut state.transform_orientation, TransformOrientation::View, "👁 View");
                });
        });
        ui.horizontal(|ui| {
            ui.checkbox(&mut state.snap_to_grid, "Grid Snap");
            if state.snap_to_grid {
                ui.add(egui::DragValue::new(&mut state.grid_snap_size).speed(0.05).range(0.01..=10.0));
            }
            ui.checkbox(&mut state.snap_angle, "Angle Snap");
            if state.snap_angle {
                ui.add(egui::DragValue::new(&mut state.angle_snap_degrees).speed(1.0).range(1.0..=180.0).suffix("°"));
            }
        });
        ui.separator();
    }
}

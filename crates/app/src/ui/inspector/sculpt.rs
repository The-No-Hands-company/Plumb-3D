use crate::ui::actions::UiActions;
use geo_kernel::{BrushFalloff, DyntopoMode, SculptBrush, SculptSettings, DyntopoSettings};

pub fn draw_sculpt_tools(
    ui: &mut egui::Ui,
    sculpt_settings: &mut SculptSettings,
    dyntopo: &mut DyntopoSettings,
    sym_x: &mut bool,
    sym_y: &mut bool,
    sym_z: &mut bool,
    actions: &mut UiActions,
) {
    ui.heading("Sculpt Brushes (Blender / ZBrush)");
    ui.horizontal_wrapped(|ui| {
        let b = sculpt_settings.brush;
        if ui.selectable_label(b == SculptBrush::Draw, "🖌 Draw").clicked() { sculpt_settings.brush = SculptBrush::Draw; }
        if ui.selectable_label(b == SculptBrush::Clay, "🧱 Clay").clicked() { sculpt_settings.brush = SculptBrush::Clay; }
        if ui.selectable_label(b == SculptBrush::Smooth, "💧 Smooth").clicked() { sculpt_settings.brush = SculptBrush::Smooth; }
        if ui.selectable_label(b == SculptBrush::Inflate, "🎈 Inflate").clicked() { sculpt_settings.brush = SculptBrush::Inflate; }
        if ui.selectable_label(b == SculptBrush::Flatten, "📐 Flatten").clicked() { sculpt_settings.brush = SculptBrush::Flatten; }
        if ui.selectable_label(b == SculptBrush::Grab, "🖐 Grab").clicked() { sculpt_settings.brush = SculptBrush::Grab; }
        if ui.selectable_label(b == SculptBrush::Pinch, "🤏 Pinch").clicked() { sculpt_settings.brush = SculptBrush::Pinch; }
        if ui.selectable_label(b == SculptBrush::Crease, "🔪 Crease").clicked() { sculpt_settings.brush = SculptBrush::Crease; }
        if ui.selectable_label(b == SculptBrush::SnakeHook, "🐍 Snake").clicked() { sculpt_settings.brush = SculptBrush::SnakeHook; }
        if ui.selectable_label(b == SculptBrush::Mask, "🎭 Mask (M)").clicked() { sculpt_settings.brush = SculptBrush::Mask; }
    });

    ui.horizontal(|ui| {
        if ui.button("Invert Mask (Ctrl+I)").clicked() { actions.trigger_invert_sculpt_mask = true; }
        if ui.button("Clear Mask (Alt+M)").clicked() { actions.trigger_clear_sculpt_mask = true; }
        if ui.button("Smooth Mask").clicked() { actions.trigger_smooth_sculpt_mask = true; }
    });

    ui.add(egui::Slider::new(&mut sculpt_settings.radius, 0.05..=3.0).text("Radius"));
    ui.add(egui::Slider::new(&mut sculpt_settings.strength, 0.01..=1.5).text("Strength"));
    ui.checkbox(&mut sculpt_settings.invert, "Invert Stroke (Ctrl)");

    egui::ComboBox::from_label("Falloff")
        .selected_text(format!("{:?}", sculpt_settings.falloff))
        .show_ui(ui, |ui| {
            ui.selectable_value(&mut sculpt_settings.falloff, BrushFalloff::Smooth, "Smooth");
            ui.selectable_value(&mut sculpt_settings.falloff, BrushFalloff::Spherical, "Spherical");
            ui.selectable_value(&mut sculpt_settings.falloff, BrushFalloff::Linear, "Linear");
            ui.selectable_value(&mut sculpt_settings.falloff, BrushFalloff::Sharp, "Sharp");
        });

    ui.horizontal(|ui| {
        ui.label("Symmetry:");
        ui.toggle_value(sym_x, "X");
        ui.toggle_value(sym_y, "Y");
        ui.toggle_value(sym_z, "Z");
    });

    ui.separator();

    // Dynamic Topology (DynTopo) Section
    ui.collapsing("🌀 Dynamic Topology (DynTopo)", |ui| {
        ui.checkbox(&mut dyntopo.enabled, "Enable Live DynTopo");
        if dyntopo.enabled {
            ui.horizontal(|ui| {
                ui.label("Detail Size:");
                ui.add(egui::DragValue::new(&mut dyntopo.detail_size).speed(0.01).range(0.01..=2.0));
            });
            ui.horizontal(|ui| {
                ui.label("Refine Mode:");
                ui.selectable_value(&mut dyntopo.mode, DyntopoMode::SubdivideCollapse, "Subdivide & Collapse");
                ui.selectable_value(&mut dyntopo.mode, DyntopoMode::SubdivideOnly, "Subdivide Only");
                ui.selectable_value(&mut dyntopo.mode, DyntopoMode::CollapseOnly, "Collapse Only");
            });
            ui.horizontal(|ui| {
                ui.label("Collapse Ratio:");
                ui.add(egui::Slider::new(&mut dyntopo.collapse_ratio, 0.1..=0.8));
            });
        }
    });

    ui.separator();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sculpt_inspector_defaults() {
        let sculpt_settings = SculptSettings::default();
        let dyntopo = DyntopoSettings::default();
        assert_eq!(sculpt_settings.brush, SculptBrush::Draw);
        assert_eq!(dyntopo.mode, DyntopoMode::SubdivideCollapse);
        assert!(dyntopo.detail_size > 0.0);
    }
}

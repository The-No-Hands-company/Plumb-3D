use crate::state::AppState;
use crate::types::*;

/// Draws the dynamic, context-sensitive Viewport Action Hints badge (HUD overlay).
pub fn draw_viewport_hints(state: &mut AppState) {
    let ctx = state.egui_ctx.clone();

    // Determine the active operational context and appropriate hint text
    let (icon, main_hint, sub_hint) = if state.inset_modal_active {
        (
            "INSET",
            "Drag: Inset Factor  |  Type: Exact Thickness  |  I: Individual Faces",
            "Enter / LMB: Confirm  |  Esc / RMB: Cancel",
        )
    } else if state.extrude_modal_active {
        (
            "EXTRUDE",
            "Drag: Extrude Distance  |  Type: Value (m)  |  X/Y/Z: Constrain Axis",
            "Enter / LMB: Confirm  |  Esc / RMB: Cancel",
        )
    } else if state.bevel_modal_active {
        (
            "BEVEL",
            "Drag: Bevel Width  |  Wheel: Segments count  |  Type: Exact Width",
            "Enter / LMB: Confirm  |  Esc / RMB: Cancel",
        )
    } else if state.knife_active {
        (
            "KNIFE",
            "LMB: Snap / Add Cut Point along Edge or Face",
            "Enter: Finalize Cut  |  Esc: Cancel Tool",
        )
    } else if state.loop_cut_active {
        (
            "LOOP CUT",
            "Move: Target Edge Ring  |  Wheel: Number of Cuts  |  LMB: Slide Loop",
            "Enter / LMB: Confirm  |  Esc / RMB: Cancel",
        )
    } else if state.edge_crease_modal_active {
        (
            "EDGE CREASE",
            "Drag: Adjust Crease Weight (0.0 Sub-D Soft -> 1.0 Sharp Edge)",
            "Enter / LMB: Confirm  |  Esc / RMB: Cancel",
        )
    } else if state.vertex_crease_modal_active {
        (
            "VERTEX CREASE",
            "Drag: Adjust Semi-Sharp Corner Weight (OpenSubdiv)",
            "Enter / LMB: Confirm  |  Esc / RMB: Cancel",
        )
    } else if state.interaction_mode == InteractionMode::Sculpt {
        (
            "SCULPT",
            "Drag: Sculpt Stroke  |  Ctrl+Drag: Invert (Sub)  |  Shift+Drag: Smooth",
            "[/]: Brush Radius  |  Shift+F: Brush Strength",
        )
    } else if state.interaction_mode == InteractionMode::CurveEdit {
        (
            "CURVE",
            "LMB: Select Knot / Tangent Handle  |  W/E/R: Transform Handle",
            "P: Curve Pen Tool  |  Tab: Return to Object Mode",
        )
    } else if state.interaction_mode == InteractionMode::Edit {
        let level_str = match state.sub_element_level {
            SubElementLevel::Vertex => "1: Vertices",
            SubElementLevel::Edge => "2: Edges",
            SubElementLevel::Face => "3: Faces",
        };
        (
            "EDIT",
            "LMB: Select  |  Shift+LMB: Multi  |  E: Extrude  |  I: Inset  |  Ctrl+B: Bevel  |  K: Knife",
            level_str,
        )
    } else {
        (
            "OBJECT",
            "LMB: Select  |  MMB: Orbit View  |  Shift+MMB: Pan  |  W/E/R: Move/Rot/Scale",
            "Tab: Edit Mode  |  F3: Command Palette  |  F1: Manual",
        )
    };

    let vp = state.egui_ctx.available_rect();

    egui::Area::new(egui::Id::new("viewport_action_hints_area"))
        .pivot(egui::Align2::CENTER_BOTTOM)
        .fixed_pos(egui::pos2(vp.center().x, vp.max.y - 10.0))
        .show(&ctx, |ui| {
            egui::Frame::none()
                .fill(egui::Color32::from_rgba_premultiplied(16, 20, 28, 225))
                .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgba_premultiplied(226, 179, 87, 140)))
                .rounding(egui::Rounding::same(6.0_f32))
                .inner_margin(egui::Margin::symmetric(14.0, 6.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(icon).size(11.0).strong().color(egui::Color32::from_rgb(226, 179, 87)));
                        ui.separator();
                        ui.label(egui::RichText::new(main_hint).size(11.0).color(egui::Color32::from_rgb(230, 238, 250)));
                        ui.separator();
                        ui.label(egui::RichText::new(sub_hint).size(10.5).color(egui::Color32::from_rgb(160, 180, 210)));
                    });
                });
        });
}

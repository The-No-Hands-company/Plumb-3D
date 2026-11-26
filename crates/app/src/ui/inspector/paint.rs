use crate::state::AppState;
use crate::ui::actions::UiActions;
use geo_kernel::{BrushFalloff, PaintBlendMode, PaintChannel};

pub fn draw_vertex_paint_tools(ui: &mut egui::Ui, state: &mut AppState, actions: &mut UiActions) {
    ui.heading("Vertex Paint (Polypaint)");
    let mut rgb = [
        state.vertex_paint_color[0],
        state.vertex_paint_color[1],
        state.vertex_paint_color[2],
    ];
    ui.horizontal(|ui| {
        ui.label("Brush Color:");
        if ui.color_edit_button_rgb(&mut rgb).changed() {
            state.vertex_paint_color[0] = rgb[0];
            state.vertex_paint_color[1] = rgb[1];
            state.vertex_paint_color[2] = rgb[2];
        }
    });

    ui.add(egui::Slider::new(&mut state.vertex_paint_radius, 0.05..=3.0).text("Radius"));
    ui.add(egui::Slider::new(&mut state.vertex_paint_strength, 0.01..=1.0).text("Strength"));

    egui::ComboBox::from_id_salt("vp_falloff_combo")
        .selected_text(format!("{:?}", state.vertex_paint_falloff))
        .show_ui(ui, |ui| {
            ui.selectable_value(&mut state.vertex_paint_falloff, BrushFalloff::Smooth, "Smooth");
            ui.selectable_value(&mut state.vertex_paint_falloff, BrushFalloff::Spherical, "Spherical");
            ui.selectable_value(&mut state.vertex_paint_falloff, BrushFalloff::Linear, "Linear");
            ui.selectable_value(&mut state.vertex_paint_falloff, BrushFalloff::Sharp, "Sharp");
        });

    if ui.button("Fill Mesh with Color").clicked() {
        actions.trigger_fill_vertex_color = true;
    }

    ui.separator();
}

pub fn draw_texture_paint_tools(ui: &mut egui::Ui, state: &mut AppState) {
    ui.heading("PBR Texture Paint (Substance 3D)");

    ui.horizontal(|ui| {
        ui.label("Channel:");
        ui.selectable_value(&mut state.texture_paint_brush.channel, PaintChannel::Albedo, "Albedo");
        ui.selectable_value(&mut state.texture_paint_brush.channel, PaintChannel::Roughness, "Roughness");
        ui.selectable_value(&mut state.texture_paint_brush.channel, PaintChannel::Metallic, "Metallic");
        ui.selectable_value(&mut state.texture_paint_brush.channel, PaintChannel::Height, "Height");
    });

    ui.horizontal(|ui| {
        ui.label("Blend:");
        ui.selectable_value(&mut state.texture_paint_brush.blend_mode, PaintBlendMode::Mix, "Mix");
        ui.selectable_value(&mut state.texture_paint_brush.blend_mode, PaintBlendMode::Add, "Add");
        ui.selectable_value(&mut state.texture_paint_brush.blend_mode, PaintBlendMode::Multiply, "Multiply");
        ui.selectable_value(&mut state.texture_paint_brush.blend_mode, PaintBlendMode::Screen, "Screen");
        ui.selectable_value(&mut state.texture_paint_brush.blend_mode, PaintBlendMode::Erase, "Erase");
    });

    let mut rgb = [
        state.texture_paint_brush.color[0],
        state.texture_paint_brush.color[1],
        state.texture_paint_brush.color[2],
    ];
    ui.horizontal(|ui| {
        ui.label("Color:");
        if ui.color_edit_button_rgb(&mut rgb).changed() {
            state.texture_paint_brush.color[0] = rgb[0];
            state.texture_paint_brush.color[1] = rgb[1];
            state.texture_paint_brush.color[2] = rgb[2];
        }
    });

    ui.add(egui::Slider::new(&mut state.texture_paint_brush.radius, 0.02..=2.0).text("Radius"));
    ui.add(egui::Slider::new(&mut state.texture_paint_brush.strength, 0.01..=1.0).text("Strength"));

    ui.horizontal(|ui| {
        ui.label("Symmetry:");
        ui.toggle_value(&mut state.texture_paint_brush.symmetry_x, "X");
        ui.toggle_value(&mut state.texture_paint_brush.symmetry_y, "Y");
        ui.toggle_value(&mut state.texture_paint_brush.symmetry_z, "Z");
    });

    if let Some(sel_id) = state.selected_node {
        let has_canvas = state.paint_textures.contains_key(&sel_id);
        ui.horizontal(|ui| {
            if !has_canvas {
                if ui.button("Create 1024 Canvas").clicked() {
                    state.paint_textures.insert(sel_id, geo_kernel::PaintTexture::new(1024, 1024, [220, 220, 220, 255]));
                }
                if ui.button("Create 2048 Canvas").clicked() {
                    state.paint_textures.insert(sel_id, geo_kernel::PaintTexture::new(2048, 2048, [220, 220, 220, 255]));
                }
            } else if let Some(tex) = state.paint_textures.get_mut(&sel_id) {
                ui.label(format!("Canvas: {}x{}", tex.width, tex.height));
                if ui.button("Clear Canvas").clicked() {
                    let fill_rgba = [
                        (state.texture_paint_brush.color[0] * 255.0) as u8,
                        (state.texture_paint_brush.color[1] * 255.0) as u8,
                        (state.texture_paint_brush.color[2] * 255.0) as u8,
                        255,
                    ];
                    tex.clear(fill_rgba);
                }
            }
        });
    }

    ui.separator();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paint_channel_and_blend_defaults() {
        let brush = geo_kernel::TexturePaintBrush::default();
        assert_eq!(brush.channel, PaintChannel::Albedo);
        assert_eq!(brush.blend_mode, PaintBlendMode::Mix);
    }
}

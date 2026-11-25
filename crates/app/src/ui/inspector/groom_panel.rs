//! Interactive Hair & Fur Grooming Studio Panel.
//! Provides Yeti/Ornatrix style guide splines, comb/puff/cut/smooth brushes,
//! clumping interpolation, and renderable hair ribbon mesh generation.

use crate::state::AppState;
use core_math::Transform;
use egui::Ui;
use geo_kernel::{GroomBrush, GroomSettings, GroomSystem};
use graph_engine::{NodeData, NodeId, SceneNode};

pub fn draw_groom_panel(ui: &mut Ui, state: &mut AppState, sel_id: NodeId) {
    ui.heading("🦁 Hair & Fur Grooming Studio");
    ui.label(
        egui::RichText::new("Guide curves, directional styling, clumping, and hair ribbon generation.")
            .small()
            .weak(),
    );
    ui.separator();

    let (node_name, base_mesh) = match state.scene.nodes.get(sel_id) {
        Some(n) => match &n.data {
            NodeData::Mesh { base_mesh, .. } => (n.name.clone(), base_mesh.clone()),
            _ => {
                ui.label("Select a polygon mesh to groom hair guides.");
                return;
            }
        },
        None => return,
    };

    // 1. Grooming Brushes
    ui.group(|ui| {
        ui.label(egui::RichText::new("Styling & Grooming Brushes").strong());
        ui.horizontal_wrapped(|ui| {
            ui.selectable_value(&mut state.groom_brush, GroomBrush::Comb, "✋ Comb");
            ui.selectable_value(&mut state.groom_brush, GroomBrush::Puff, "💨 Puff");
            ui.selectable_value(&mut state.groom_brush, GroomBrush::Smooth, "✨ Smooth");
            ui.selectable_value(&mut state.groom_brush, GroomBrush::Length, "📏 Length");
            ui.selectable_value(&mut state.groom_brush, GroomBrush::Cut, "✂ Cut");
        });

        ui.horizontal(|ui| {
            ui.label("Radius:");
            ui.add(egui::Slider::new(&mut state.groom_radius, 0.05..=2.0).text("m"));
            ui.label("Strength:");
            ui.add(egui::Slider::new(&mut state.groom_strength, 0.05..=1.0));
        });
    });

    ui.separator();

    // 2. Guide Curve Generation & Synthesis
    ui.group(|ui| {
        ui.label(egui::RichText::new("Surface Hair Guides").strong());
        ui.label(
            egui::RichText::new("Spawns control guides along surface normals of the active mesh.")
                .small()
                .weak(),
        );

        if ui.button("🌿 Generate Guides From Surface")
            .on_hover_text("Spawns normal-aligned guide splines on all surface vertices")
            .clicked()
        {
            let system = GroomSystem::generate_guides_from_mesh(&base_mesh, 0.35, 5);
            let guide_count = system.guides.len();
            let settings = GroomSettings::default();
            let ribbon_mesh = system.to_ribbon_mesh(&settings);
            let hair_name = format!("{}_HairGroom", node_name);
            let new_node = SceneNode::new_mesh(hair_name, Transform::IDENTITY, ribbon_mesh);
            let new_id = state.scene.add_node(new_node);
            state.selected_node = Some(new_id);
            state.sync_gpu_meshes();
            state.set_status_message(format!("🦁 Generated {} hair guide splines and ribbon mesh.", guide_count));
        }
    });

    ui.separator();

    // 3. Dense Clumping & Strand Synthesis
    ui.group(|ui| {
        ui.label(egui::RichText::new("Clumping & Ribbon Generation").strong());
        ui.label(
            egui::RichText::new("Synthesizes dense hair strands with clumping and tip tapering.")
                .small()
                .weak(),
        );

        if ui.button("✨ Bake Dense Hair Ribbons (10x)")
            .on_hover_text("Interpolates dense clumped strands into camera-facing ribbon polygons")
            .clicked()
        {
            let system = GroomSystem::generate_guides_from_mesh(&base_mesh, 0.4, 6);
            let mut settings = GroomSettings::default();
            settings.strands_per_guide = 12;
            settings.clump_factor = 0.75;
            settings.curl_radius = 0.015;

            let ribbon_mesh = system.to_ribbon_mesh(&settings);
            let strand_count = system.guides.len() * settings.strands_per_guide;
            let hair_name = format!("{}_DenseFurRibbons", node_name);
            let new_node = SceneNode::new_mesh(hair_name, Transform::IDENTITY, ribbon_mesh);
            let new_id = state.scene.add_node(new_node);
            state.selected_node = Some(new_id);
            state.sync_gpu_meshes();
            state.set_status_message(format!("🦁 Synthesized {} dense hair ribbon strands.", strand_count));
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_groom_guide_ribbon_generation() {
        let sphere = geo_kernel::Primitives::create_uv_sphere(1.0, 8, 8);
        let groom = GroomSystem::generate_guides_from_mesh(&sphere, 0.2, 4);
        assert!(!groom.guides.is_empty());
        let settings = GroomSettings::default();
        let ribbons = groom.to_ribbon_mesh(&settings);
        assert!(!ribbons.vertices.is_empty());
        assert!(!ribbons.faces.is_empty());
    }
}

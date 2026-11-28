//! OpenVDB Volumetrics & Atmospheric VFX Inspector Panel.
//! Provides sparse hierarchical grid sampling (VdbGrid), procedural ground fog,
//! cumulus cloud generation, and Marching Cubes isosurface extraction.

use crate::state::AppState;
use core_math::{vec3, Transform};
use egui::Ui;
use geo_kernel::VolumeGenerators;
use graph_engine::SceneNode;

pub fn draw_volumetrics_panel(ui: &mut Ui, state: &mut AppState) {
    ui.heading("OpenVDB Volumetrics & Atmospheric VFX");
    ui.label(
        egui::RichText::new(
            "Sparse hierarchical volume grids for realistic ground fog, clouds, and smoke.",
        )
        .small()
        .weak(),
    );
    ui.separator();

    // 1. Procedural Atmospheric Ground Fog
    ui.group(|ui| {
        ui.label(egui::RichText::new("Atmospheric Ground Fog").strong());
        ui.label(
            egui::RichText::new(
                "Low-lying sparse volume with height falloff and Perlin turbulence.",
            )
            .small()
            .weak(),
        );

        ui.horizontal(|ui| {
            ui.label("Fog Density:");
            ui.add(egui::Slider::new(&mut state.vdb_density, 0.05..=2.0));
        });

        if ui
            .button("Spawn Atmospheric Ground Fog Grid")
            .on_hover_text(
                "Constructs sparse VDB tree with ground fog density and extracts boundary mesh",
            )
            .clicked()
        {
            let grid = VolumeGenerators::create_ground_fog(4.0, 1.2, state.vdb_density, 0.15);
            let active_bricks = grid.active_brick_count();
            let iso_mesh = grid.extract_isosurface(0.08);

            let node = SceneNode::new_mesh("GroundFog_Volume", Transform::IDENTITY, iso_mesh);
            let id = state.scene.add_node(node);
            state.selected_node = Some(id);
            state.sync_gpu_meshes();
            state.set_status_message(format!(
                "Spawned OpenVDB ground fog ({} active bricks, {} KB sparse storage).",
                active_bricks,
                active_bricks * 2
            ));
        }
    });

    ui.separator();

    // 2. Procedural Cumulus Cloud
    ui.group(|ui| {
        ui.label(egui::RichText::new("Procedural Cumulus Cloud").strong());
        ui.label(
            egui::RichText::new(
                "Multi-octave noise turbulence volume representing fluffy cumulus clouds.",
            )
            .small()
            .weak(),
        );

        ui.horizontal(|ui| {
            ui.label("Cloud Radius:");
            ui.add(egui::Slider::new(&mut state.vdb_cloud_radius, 0.5..=5.0).text("m"));
        });

        ui.horizontal(|ui| {
            ui.label("Surface Iso-Level:");
            ui.add(egui::Slider::new(&mut state.vdb_iso_level, 0.05..=0.8));
        });

        if ui
            .button("Generate Cumulus Cloud Volume")
            .on_hover_text(
                "Samples 3D fractal noise into sparse VDB grid and creates cloud isosurface",
            )
            .clicked()
        {
            let center = vec3(0.0, 2.5, 0.0);
            let grid = VolumeGenerators::create_cumulus_cloud(
                center,
                state.vdb_cloud_radius,
                state.vdb_density,
                0.12,
            );
            let active_bricks = grid.active_brick_count();
            let iso_mesh = grid.extract_isosurface(state.vdb_iso_level);

            let node = SceneNode::new_mesh("CumulusCloud_Volume", Transform::IDENTITY, iso_mesh);
            let id = state.scene.add_node(node);
            state.selected_node = Some(id);
            state.sync_gpu_meshes();
            state.set_status_message(format!(
                "Generated Cumulus Cloud ({} active bricks in sparse hierarchical tree).",
                active_bricks
            ));
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vdb_cloud_generation() {
        let grid = VolumeGenerators::create_cumulus_cloud(vec3(0.0, 0.0, 0.0), 0.8, 1.0, 0.2);
        assert!(grid.active_brick_count() > 0);
        let mesh = grid.extract_isosurface(0.1);
        assert!(!mesh.vertices.is_empty());
    }
}

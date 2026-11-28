pub mod camera_gate;
pub mod curvature_comb_overlay;
pub mod cutter_overlay;
pub mod hud;
pub mod mesh_features;
pub mod modals;
pub mod operator_panel;
pub mod pie_menu_overlay;
pub mod push_pull_overlay;
pub mod selection_overlay;
pub mod sketch_overlay;
pub mod tools;
pub mod walkthrough;
pub mod walkthrough_actions;
pub mod walkthrough_crate;

use crate::state::AppState;

pub fn draw_overlays(state: &mut AppState) {
    crate::live_pathtracer::step_live_viewport_pathtracer(state);
    crate::live_pathtracer::render_live_pathtracer_overlay(state);
    camera_gate::render(state);
    cutter_overlay::render(state);
    selection_overlay::render_selection_overlay(state);
    tools::render(state);
    modals::render(state);
    push_pull_overlay::render(state);
    operator_panel::render(state);
    sketch_overlay::render(state);
    curvature_comb_overlay::render(state);
    mesh_features::render(state);
    hud::render(state);
    walkthrough::render(state);
}

mod constraints;
mod groom_panel;
mod materials;
mod mesh_info;
mod mesh_tools;
mod modifiers;
mod motion_panel;
pub mod paint;
mod physics;
mod scene_nodes;
pub mod sculpt;
mod topology_cad;
mod transform;
mod vertex_groups;
mod volumetrics;

pub use constraints::draw_constraints_panel;
pub use groom_panel::draw_groom_panel;
pub use materials::draw_materials_panel;
pub use mesh_info::draw_mesh_info;
pub use mesh_tools::draw_mesh_tools_panel;
pub use modifiers::draw_modifiers_panel;
pub use motion_panel::draw_motion_panel;
pub use paint::{draw_texture_paint_tools, draw_vertex_paint_tools};
pub use physics::{draw_physics_panel, PhysicsSubTab};
pub use scene_nodes::draw_scene_nodes_panel;
pub use sculpt::draw_sculpt_tools;
pub use topology_cad::draw_topology_cad_panel;
pub use transform::draw_transform_panel;
pub use vertex_groups::draw_vertex_groups_panel;
pub use volumetrics::draw_volumetrics_panel;

use crate::state::AppState;
use crate::ui::actions::UiActions;
use crate::ui::workspaces::WorkspaceTemplate;
use graph_engine::{NodeData, NodeId};

pub fn draw_inspector(state: &mut AppState, actions: &mut UiActions) {
    if !state.show_inspector {
        return;
    }

    let egui_ctx = state.egui_ctx.clone();

    egui::SidePanel::right("inspector_panel")
        .resizable(true)
        .default_width(280.0)
        .show(&egui_ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Inspector");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button(">").on_hover_text("Collapse Inspector").clicked() {
                        state.show_inspector = false;
                    }
                });
            });
            ui.separator();
            draw_inspector_inner(state, actions, ui);
        });
}

pub fn draw_inspector_inner(state: &mut AppState, actions: &mut UiActions, ui: &mut egui::Ui) {
    let available_mesh_ids: Vec<(NodeId, String)> = state.scene.nodes.iter()
        .filter_map(|(nid, n)| {
            if matches!(n.data, NodeData::Mesh { .. }) {
                Some((nid, n.name.clone()))
            } else {
                None
            }
        })
        .collect();

    let available_curve_names: Vec<String> = state.scene.nodes.values()
        .filter_map(|n| {
            if matches!(n.data, NodeData::Curve { .. }) {
                Some(n.name.clone())
            } else {
                None
            }
        })
        .collect();

    if let Some(sel_id) = state.selected_node {
        if let Some(node) = state.scene.nodes.get_mut(sel_id) {
            let type_badge = match &node.data {
                NodeData::Mesh { .. } => "Mesh",
                NodeData::Light { .. } => "Light",
                NodeData::Camera { .. } => "Camera",
                NodeData::Joint { .. } => "Bone",
                NodeData::Curve { .. } => "Spline",
                NodeData::Transform => "Group",
            };
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(&node.name).strong().size(14.0));
                ui.label(egui::RichText::new(type_badge).small().color(egui::Color32::from_rgb(140, 150, 175)));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.checkbox(&mut node.visible, "Visible");
                });
            });
        }
        ui.separator();

        ui.horizontal_wrapped(|ui| {
            if ui.selectable_label(state.inspector_tab == crate::types::InspectorTab::Transform, "Node").clicked() {
                state.inspector_tab = crate::types::InspectorTab::Transform;
            }
            if ui.selectable_label(state.inspector_tab == crate::types::InspectorTab::Modifiers, "Modifiers").clicked() {
                state.inspector_tab = crate::types::InspectorTab::Modifiers;
            }
            if ui.selectable_label(state.inspector_tab == crate::types::InspectorTab::Materials, "Shading").clicked() {
                state.inspector_tab = crate::types::InspectorTab::Materials;
            }

            if state.current_workspace != WorkspaceTemplate::Animation {
                let tool_label = match state.current_workspace {
                    WorkspaceTemplate::Sculpting => "Brushes",
                    WorkspaceTemplate::UvRetopo => "UV Tools",
                    _ => "Tools",
                };
                if ui.selectable_label(state.inspector_tab == crate::types::InspectorTab::Tools, tool_label).clicked() {
                    state.inspector_tab = crate::types::InspectorTab::Tools;
                }
            }

            if state.current_workspace == WorkspaceTemplate::Animation {
                if ui.selectable_label(state.inspector_tab == crate::types::InspectorTab::Constraints, "Rigging").clicked() {
                    state.inspector_tab = crate::types::InspectorTab::Constraints;
                }
                if ui.selectable_label(state.inspector_tab == crate::types::InspectorTab::Physics, "Physics").clicked() {
                    state.inspector_tab = crate::types::InspectorTab::Physics;
                }
            }
        });
        ui.separator();

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                match state.inspector_tab {
                    crate::types::InspectorTab::Transform => {
                        draw_transform_panel(ui, state, sel_id);
                        draw_scene_nodes_panel(ui, state, actions, sel_id, &available_mesh_ids);
                        if let Some(node) = state.scene.nodes.get(sel_id) {
                            if let NodeData::Mesh { base_mesh, .. } = &node.data {
                                draw_mesh_info(
                                    ui,
                                    state.interaction_mode,
                                    &state.selected_vertices,
                                    &state.selected_edges,
                                    &state.selected_faces,
                                    base_mesh,
                                );
                            }
                        }
                    }
                    crate::types::InspectorTab::Modifiers => {
                        draw_modifiers_panel(
                            ui,
                            state,
                            actions,
                            sel_id,
                            &available_mesh_ids,
                            &available_curve_names,
                        );
                    }
                    crate::types::InspectorTab::Materials => {
                        draw_materials_panel(ui, state, actions, sel_id);
                    }
                    crate::types::InspectorTab::Tools => {
                        if state.interaction_mode == crate::types::InteractionMode::Sculpt {
                            draw_sculpt_tools(
                                ui,
                                &mut state.sculpt_settings,
                                &mut state.dyntopo,
                                &mut state.sculpt_symmetry_x,
                                &mut state.sculpt_symmetry_y,
                                &mut state.sculpt_symmetry_z,
                                actions,
                            );
                            ui.separator();
                        }
                        if state.interaction_mode == crate::types::InteractionMode::VertexPaint {
                            draw_vertex_paint_tools(ui, state, actions);
                            ui.separator();
                        }
                        if state.interaction_mode == crate::types::InteractionMode::TexturePaint {
                            draw_texture_paint_tools(ui, state);
                            ui.separator();
                        }
                        if state.current_workspace == WorkspaceTemplate::Sculpting {
                            draw_groom_panel(ui, state, sel_id);
                            ui.separator();
                            draw_volumetrics_panel(ui, state);
                            ui.separator();
                        }
                        if state.current_workspace == WorkspaceTemplate::UvRetopo || state.current_workspace == WorkspaceTemplate::CadPrecision {
                            draw_topology_cad_panel(ui, state, actions, sel_id);
                            ui.separator();
                        }
                        draw_mesh_tools_panel(ui, state, actions, sel_id);
                    }
                    crate::types::InspectorTab::Constraints => {
                        draw_vertex_groups_panel(ui, state, actions, sel_id);
                        draw_constraints_panel(ui, state, actions, sel_id);
                    }
                    crate::types::InspectorTab::Physics => {
                        draw_physics_panel(ui, state, sel_id);
                        ui.separator();
                        draw_motion_panel(ui, state);
                    }
                }
            });
    } else {
        ui.vertical_centered(|ui| {
            ui.add_space(30.0);
            ui.label(egui::RichText::new("No Selection").italics().color(egui::Color32::GRAY));
            ui.label("Click an object in the viewport or outliner to inspect properties.");
        });
    }
}

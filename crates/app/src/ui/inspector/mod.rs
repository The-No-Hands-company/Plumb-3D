mod constraints;
mod materials;
mod mesh_info;
mod mesh_tools;
mod modifiers;
mod physics;
mod scene_nodes;
mod transform;
mod vertex_groups;

pub use constraints::draw_constraints_panel;
pub use materials::draw_materials_panel;
pub use mesh_info::draw_mesh_info;
pub use mesh_tools::draw_mesh_tools_panel;
pub use modifiers::draw_modifiers_panel;
pub use physics::draw_physics_panel;
pub use scene_nodes::draw_scene_nodes_panel;
pub use transform::draw_transform_panel;
pub use vertex_groups::draw_vertex_groups_panel;

use crate::state::AppState;
use crate::ui::actions::UiActions;
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
                    if ui.small_button("▶").on_hover_text("Collapse Inspector").clicked() {
                        state.show_inspector = false;
                    }
                });
            });
            ui.separator();

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
                // Header (Name, Type Badge, and Visibility)
                if let Some(node) = state.scene.nodes.get_mut(sel_id) {
                    let type_badge = match &node.data {
                        NodeData::Mesh { .. } => "⬡ Mesh",
                        NodeData::Light { .. } => "💡 Light",
                        NodeData::Camera { .. } => "📷 Camera",
                        NodeData::Joint { .. } => "🦴 Bone",
                        NodeData::Curve { .. } => "〰 Spline",
                        NodeData::Transform => "📁 Group",
                    };
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(&node.name).strong().size(14.0));
                        ui.label(egui::RichText::new(type_badge).small().color(egui::Color32::from_rgb(140, 150, 175)));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.checkbox(&mut node.visible, "👁");
                        });
                    });
                }
                ui.separator();

                // Modern Pill Tab Bar
                ui.horizontal_wrapped(|ui| {
                    if ui.selectable_label(state.inspector_tab == crate::types::InspectorTab::Transform, "📍 Node").clicked() {
                        state.inspector_tab = crate::types::InspectorTab::Transform;
                    }
                    if ui.selectable_label(state.inspector_tab == crate::types::InspectorTab::Modifiers, "⚡ Modifiers").clicked() {
                        state.inspector_tab = crate::types::InspectorTab::Modifiers;
                    }
                    if ui.selectable_label(state.inspector_tab == crate::types::InspectorTab::Materials, "🎨 Shading").clicked() {
                        state.inspector_tab = crate::types::InspectorTab::Materials;
                    }
                    if ui.selectable_label(state.inspector_tab == crate::types::InspectorTab::Tools, "🛠 Tools").clicked() {
                        state.inspector_tab = crate::types::InspectorTab::Tools;
                    }
                    if ui.selectable_label(state.inspector_tab == crate::types::InspectorTab::Constraints, "🔗 Rigging").clicked() {
                        state.inspector_tab = crate::types::InspectorTab::Constraints;
                    }
                    if ui.selectable_label(state.inspector_tab == crate::types::InspectorTab::Physics, "⚛ Physics").clicked() {
                        state.inspector_tab = crate::types::InspectorTab::Physics;
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
                                draw_modifiers_panel(ui, state, actions, sel_id, &available_mesh_ids, &available_curve_names);
                            }
                            crate::types::InspectorTab::Materials => {
                                draw_materials_panel(ui, state, actions, sel_id);
                            }
                            crate::types::InspectorTab::Tools => {
                                draw_mesh_tools_panel(ui, state, actions, sel_id);
                            }
                            crate::types::InspectorTab::Constraints => {
                                draw_vertex_groups_panel(ui, state, actions, sel_id);
                                draw_constraints_panel(ui, state, actions, sel_id);
                            }
                            crate::types::InspectorTab::Physics => {
                                draw_physics_panel(ui, state, sel_id);
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
        });
}

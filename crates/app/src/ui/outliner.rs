use graph_engine::{LightType, NodeData, NodeId, SceneGraph};

use crate::state::AppState;
use crate::ui::actions::UiActions;

pub fn draw_outliner(state: &mut AppState, actions: &mut UiActions) {
    if !state.show_outliner {
        return;
    }

    egui::SidePanel::left("outliner_panel")
        .resizable(true)
        .default_width(220.0)
        .show(&state.egui_ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Scene Outliner");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button("◀").on_hover_text("Collapse Outliner").clicked() {
                        state.show_outliner = false;
                    }
                });
            });
            ui.separator();

                ui.horizontal(|ui| {
                    ui.add(egui::TextEdit::singleline(&mut state.outliner_filter).hint_text("Search outliner..."));
                    if !state.outliner_filter.is_empty() && ui.small_button("✖").clicked() {
                        state.outliner_filter.clear();
                    }
                });
                if ui.button("+ Group / Folder").clicked() {
                    actions.spawn_group = true;
                }
                ui.separator();

                let mut node_to_select = None;
                let mut toggle_vis = None;
                let root_nodes = state.scene.root_nodes.clone();

                egui::ScrollArea::vertical()
                    .max_height(350.0)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        for root_id in root_nodes {
                            draw_outliner_tree_node(
                                ui,
                                &state.scene,
                                root_id,
                                0,
                                state.selected_node,
                                &state.selected_nodes,
                                &mut node_to_select,
                                &mut toggle_vis,
                                &state.outliner_filter,
                            );
                        }
                    });

                if let Some(new_sel) = node_to_select {
                    if state.shift_pressed {
                        if state.selected_nodes.contains(&new_sel) {
                            state.selected_nodes.remove(&new_sel);
                            if state.selected_node == Some(new_sel) {
                                state.selected_node = state.selected_nodes.iter().copied().next();
                            }
                        } else {
                            state.selected_nodes.insert(new_sel);
                            state.previous_selected_node = state.selected_node;
                            state.selected_node = Some(new_sel);
                        }
                    } else {
                        state.selected_nodes.clear();
                        state.selected_nodes.insert(new_sel);
                        if state.selected_node != Some(new_sel) {
                            state.previous_selected_node = state.selected_node;
                            state.selected_node = Some(new_sel);
                        }
                    }
                }
                if let Some(vis_id) = toggle_vis {
                    if let Some(node) = state.scene.nodes.get_mut(vis_id) {
                        node.visible = !node.visible;
                    }
                }

                ui.separator();
                ui.collapsing("Hierarchy & Parenting", |ui| {
                    if let Some(prev) = state.previous_selected_node {
                        let prev_name = state.scene.nodes.get(prev).map(|n| n.name.as_str()).unwrap_or("None");
                        ui.label(format!("Parent Target: {}", prev_name));
                    } else {
                        ui.label("Parent Target: None");
                    }

                    ui.horizontal(|ui| {
                        let can_parent = state.selected_node.is_some()
                            && state.previous_selected_node.is_some()
                            && state.selected_node != state.previous_selected_node;
                        if ui.add_enabled(can_parent, egui::Button::new("Parent (P)")).clicked() {
                            actions.trigger_parent = true;
                        }
                        if ui.add_enabled(state.selected_node.is_some(), egui::Button::new("Unparent (Shift+P)")).clicked() {
                            actions.trigger_unparent = true;
                        }
                        if ui.add_enabled(can_parent, egui::Button::new("Join (Ctrl+J)")).clicked() {
                            actions.trigger_join_meshes = true;
                        }
                    });

                    if ui.button("🗑 Delete Selected (X)").clicked() {
                        actions.delete_selected = true;
                    }
                });
            });

        // Right Panel: 3ds Max style Modifier Stack & Direct Modeling Operators

}

fn has_matching_child(scene: &SceneGraph, node_id: NodeId, filter: &str) -> bool {
    if let Some(node) = scene.nodes.get(node_id) {
        for &child_id in &node.children {
            if let Some(child) = scene.nodes.get(child_id) {
                if child.name.to_lowercase().contains(&filter.to_lowercase())
                    || has_matching_child(scene, child_id, filter)
                {
                    return true;
                }
            }
        }
    }
    false
}

#[allow(clippy::too_many_arguments)]
fn draw_outliner_tree_node(
    ui: &mut egui::Ui,
    scene: &SceneGraph,
    node_id: NodeId,
    depth: usize,
    selected_node: Option<NodeId>,
    selected_nodes: &std::collections::HashSet<NodeId>,
    node_to_select: &mut Option<NodeId>,
    toggle_vis: &mut Option<NodeId>,
    filter: &str,
) {
    if let Some(node) = scene.nodes.get(node_id) {
        let matches_filter = filter.is_empty()
            || node.name.to_lowercase().contains(&filter.to_lowercase())
            || has_matching_child(scene, node_id, filter);

        if !matches_filter {
            return;
        }

        let is_active = selected_node == Some(node_id);
        let is_selected = is_active || selected_nodes.contains(&node_id);
        let indent = "  ".repeat(depth);
        let branch = if node.children.is_empty() {
            " "
        } else {
            "▼"
        };
        let kind_icon = match &node.data {
            NodeData::Mesh { .. } => "⬡",
            NodeData::Light { light_type, .. } => match light_type {
                LightType::Directional => "☀️",
                LightType::Point => "💡",
                LightType::Spot => "🔦",
            },
            NodeData::Camera { .. } => "📷",
            NodeData::Joint { .. } => "🦴",
            NodeData::Curve { .. } => "➰",
            NodeData::Transform => "📁",
        };

        ui.horizontal(|ui| {
            let eye_text = if node.visible { "👁" } else { "⊘" };
            if ui.small_button(eye_text).on_hover_text("Toggle Visibility").clicked() {
                *toggle_vis = Some(node_id);
            }

            let text = format!("{}{} {} {}", indent, branch, kind_icon, node.name);
            let label = if is_active {
                egui::RichText::new(text)
                    .strong()
                    .color(egui::Color32::from_rgb(255, 195, 75))
            } else if is_selected {
                egui::RichText::new(text)
                    .strong()
                    .color(egui::Color32::from_rgb(255, 140, 40))
            } else if !node.visible {
                egui::RichText::new(text).color(egui::Color32::from_rgb(110, 115, 125))
            } else {
                egui::RichText::new(text)
            };

            if ui.selectable_label(is_selected, label).clicked() {
                *node_to_select = Some(node_id);
            }
        });

        for &child_id in &node.children {
            draw_outliner_tree_node(
                ui,
                scene,
                child_id,
                depth + 1,
                selected_node,
                selected_nodes,
                node_to_select,
                toggle_vis,
                filter,
            );
        }
    }
}


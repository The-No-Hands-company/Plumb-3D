use crate::state::AppState;
use crate::ui::actions::UiActions;
use graph_engine::{NodeData, NodeId};

pub fn draw_vertex_groups_panel(
    ui: &mut egui::Ui,
    state: &mut AppState,
    actions: &mut UiActions,
    sel_id: NodeId,
) {
    let selected_vertices = state.selected_vertices.clone();
    let mut weight = state.vertex_group_weight;
    let mut new_name = state.new_vg_name.clone();
    let active_group = state.active_vertex_group.clone();

    if let Some(node) = state.scene.nodes.get_mut(sel_id) {
        ui.heading("Vertex Groups (Weighting)");
        if let NodeData::Mesh { base_mesh, .. } = &mut node.data {
            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut new_name);
                if ui.button("+ Add Group").clicked() && !new_name.is_empty() {
                    base_mesh.add_vertex_group(new_name.clone());
                }
            });

            if !base_mesh.vertex_groups.is_empty() {
                ui.horizontal(|ui| {
                    if ui.button("Normalize All Weights").clicked() {
                        let all_names: Vec<String> = base_mesh
                            .vertex_groups
                            .iter()
                            .map(|g| g.name.clone())
                            .collect();
                        actions.trigger_normalize_vertex_groups = Some(all_names);
                    }
                });
                ui.add(egui::Slider::new(&mut weight, 0.0..=1.0).text("Weight"));
                for vg in &mut base_mesh.vertex_groups {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(format!("- {} ({} verts)", vg.name, vg.weights.len()));
                            if ui.button("Mirror (X)").clicked() {
                                let target = if vg.name.ends_with(".L") {
                                    format!("{}.R", &vg.name[..vg.name.len() - 2])
                                } else if vg.name.ends_with(".R") {
                                    format!("{}.L", &vg.name[..vg.name.len() - 2])
                                } else if vg.name.ends_with("_L") {
                                    format!("{}_R", &vg.name[..vg.name.len() - 2])
                                } else if vg.name.ends_with("_R") {
                                    format!("{}_L", &vg.name[..vg.name.len() - 2])
                                } else {
                                    format!("{}.mirror", vg.name)
                                };
                                actions.trigger_mirror_vertex_group =
                                    Some((vg.name.clone(), target));
                            }
                            if !selected_vertices.is_empty() {
                                if ui.button("Assign").clicked() {
                                    for &vk in &selected_vertices {
                                        vg.set_weight(vk, weight);
                                    }
                                }
                                if ui.button("Remove").clicked() {
                                    for &vk in &selected_vertices {
                                        vg.weights.remove(&vk);
                                    }
                                }
                            }
                        });
                    });
                }
            } else {
                ui.label(egui::RichText::new("No vertex groups defined yet").weak());
            }
        }
    }
    state.vertex_group_weight = weight;
    state.new_vg_name = new_name;
    state.active_vertex_group = active_group;
}

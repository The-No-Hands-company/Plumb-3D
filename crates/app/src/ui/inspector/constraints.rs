use crate::state::AppState;
use crate::ui::actions::UiActions;
use core_math::Vec3;
use geo_kernel::Spline;
use graph_engine::{AimConstraint, Constraint, NodeId, ParentConstraint, PathConstraint};

pub fn draw_constraints_panel(
    ui: &mut egui::Ui,
    state: &mut AppState,
    _actions: &mut UiActions,
    sel_id: NodeId,
) {
    let (scene, constraints) = (&state.scene, &mut state.constraints);
                    ui.collapsing("Scene Constraints (Maya / Max)", |ui| {
                        let mut remove_idx = None;
                        for (i, c) in constraints.constraints.iter_mut().enumerate() {
                            if c.target_node() == sel_id {
                                ui.group(|ui| {
                                    match c {
                                        Constraint::Path(p) => {
                                            ui.horizontal(|ui| {
                                                ui.checkbox(&mut p.enabled, "Path Follow");
                                                if ui.button("✖").clicked() {
                                                    remove_idx = Some(i);
                                                }
                                            });
                                            ui.add(egui::Slider::new(&mut p.u, 0.0..=1.0).text("Param u"));
                                            ui.checkbox(&mut p.follow_tangent, "Orient Tangent");
                                        }
                                        Constraint::Aim(a) => {
                                            ui.horizontal(|ui| {
                                                ui.checkbox(&mut a.enabled, "Aim Target");
                                                if ui.button("✖").clicked() {
                                                    remove_idx = Some(i);
                                                }
                                            });
                                            let current_target_name = scene.nodes.get(a.target_node).map(|n| n.name.as_str()).unwrap_or("None");
                                            egui::ComboBox::from_id_salt(format!("aim_target_{}", i))
                                                .selected_text(current_target_name)
                                                .show_ui(ui, |ui| {
                                                    for (other_id, other_node) in &scene.nodes {
                                                        if other_id != sel_id {
                                                            ui.selectable_value(&mut a.target_node, other_id, &other_node.name);
                                                        }
                                                    }
                                                });
                                        }
                                        Constraint::Parent(pc) => {
                                            ui.horizontal(|ui| {
                                                ui.checkbox(&mut pc.enabled, "Parent Constraint");
                                                if ui.button("✖").clicked() {
                                                    remove_idx = Some(i);
                                                }
                                            });
                                            let current_target_name = scene.nodes.get(pc.target_node).map(|n| n.name.as_str()).unwrap_or("None");
                                            egui::ComboBox::from_id_salt(format!("parent_target_{}", i))
                                                .selected_text(current_target_name)
                                                .show_ui(ui, |ui| {
                                                    for (other_id, other_node) in &scene.nodes {
                                                        if other_id != sel_id {
                                                            ui.selectable_value(&mut pc.target_node, other_id, &other_node.name);
                                                        }
                                                    }
                                                });
                                            ui.checkbox(&mut pc.maintain_offset, "Maintain Offset");
                                        }
                                        Constraint::Ik(ik) => {
                                            ui.horizontal(|ui| {
                                                ui.checkbox(&mut ik.enabled, "Two-Bone IK");
                                                if ui.button("✖").clicked() {
                                                    remove_idx = Some(i);
                                                }
                                            });
                                            let current_goal_name = scene.nodes.get(ik.target).map(|n| n.name.as_str()).unwrap_or("None");
                                            egui::ComboBox::from_id_salt(format!("ik_goal_{}", i))
                                                .selected_text(current_goal_name)
                                                .show_ui(ui, |ui| {
                                                    for (other_id, other_node) in &scene.nodes {
                                                        if other_id != sel_id {
                                                            ui.selectable_value(&mut ik.target, other_id, &other_node.name);
                                                        }
                                                    }
                                                });
                                        }
                                    }
                                });
                            }
                        }

                        if let Some(idx) = remove_idx {
                            constraints.constraints.remove(idx);
                        }

                        ui.add_space(4.0);
                        ui.horizontal_wrapped(|ui| {
                            let other_node_id = scene.nodes.keys().find(|&id| id != sel_id);

                            if let Some(target_id) = other_node_id {
                                if ui.button("+ Aim").clicked() {
                                    constraints.add(Constraint::Aim(AimConstraint::new(sel_id, target_id)));
                                }
                                if ui.button("+ Parent").clicked() {
                                    constraints.add(Constraint::Parent(ParentConstraint::new(sel_id, target_id)));
                                }
                            }
                            if ui.button("+ Path (Circle)").clicked() {
                                let spline = Spline::circle(4.0, Vec3::ZERO);
                                constraints.add(Constraint::Path(PathConstraint::new(sel_id, spline)));
                            }
                            if ui.button("+ Path (Helix)").clicked() {
                                let spline = Spline::helix(3.0, 6.0, 3.0, 12);
                                constraints.add(Constraint::Path(PathConstraint::new(sel_id, spline)));
                            }
                        });
                    });
}

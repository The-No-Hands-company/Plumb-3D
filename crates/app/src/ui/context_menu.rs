use geo_kernel::{MirrorAxis, Primitives};
use graph_engine::{LightType, NodeData};

use crate::state::AppState;
use crate::types::*;
use crate::ui::actions::UiActions;

/// Sleek Context-Sensitive Right-Click (RMB) Floating Menu (Blender / Maya / Plasticity Style)
pub fn draw_context_menu(state: &mut AppState, actions: &mut UiActions) {
    if !state.show_context_menu {
        return;
    }

    let Some(pos) = state.context_menu_pos else {
        return;
    };

    let mut close_menu = false;

    egui::Area::new(egui::Id::new("viewport_rmb_context_menu"))
        .fixed_pos(egui::pos2(pos.0, pos.1))
        .order(egui::Order::Foreground)
        .show(&state.egui_ctx, |ui| {
            let frame = egui::Frame::popup(ui.style())
                .fill(egui::Color32::from_rgba_premultiplied(22, 25, 32, 245))
                .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(60, 68, 85)))
                .rounding(egui::Rounding::same(8.0))
                .inner_margin(egui::Margin::symmetric(10.0, 8.0));

            let response = frame.show(ui, |ui| {
                ui.set_min_width(220.0);

                match state.interaction_mode {
                    // ==========================================
                    // 1. OBJECT MODE CONTEXT MENU
                    // ==========================================
                    InteractionMode::Object => {
                        if let Some(sel_id) = state.selected_node {
                            let node_name = state.scene.nodes.get(sel_id).map(|n| n.name.clone()).unwrap_or_else(|| "Object".into());
                            ui.label(egui::RichText::new(format!("[Object] {}", node_name)).strong().color(egui::Color32::from_rgb(100, 185, 255)));
                            ui.separator();

                            // Mode Switch
                            if ui.button("Enter Edit Mode (Tab)").clicked() {
                                state.interaction_mode = InteractionMode::Edit;
                                close_menu = true;
                            }
                            if ui.button("Focus Camera on Object (F)").clicked() {
                                actions.trigger_frame_selected = true;
                                close_menu = true;
                            }
                            ui.separator();

                            // Shading & Normals
                            ui.label(egui::RichText::new("SHADING & NORMALS").small().color(egui::Color32::GRAY));
                            if ui.button("Shade Smooth / Auto-Smooth").clicked() {
                                actions.trigger_auto_smooth = true;
                                close_menu = true;
                            }
                            if ui.button("Shade Flat").clicked() {
                                state.flat_shading = true;
                                if let Some(n) = state.scene.nodes.get_mut(sel_id) {
                                    n.is_dirty = true;
                                }
                                close_menu = true;
                            }
                            if ui.button("Recalculate Normals Outside").clicked() {
                                actions.trigger_recalculate_normals = Some(false);
                                close_menu = true;
                            }
                            if ui.button("Recalculate Normals Inside").clicked() {
                                actions.trigger_recalculate_normals = Some(true);
                                close_menu = true;
                            }
                            if ui.button("Flip Normals").clicked() {
                                actions.trigger_flip_normals = true;
                                close_menu = true;
                            }
                            ui.separator();

                            // Quick Modifiers
                            ui.label(egui::RichText::new("QUICK MODIFIERS").small().color(egui::Color32::GRAY));
                            ui.horizontal(|ui| {
                                if ui.button("+ Subdiv").clicked() {
                                    actions.add_catmull_clark_to_selected = true;
                                    close_menu = true;
                                }
                                if ui.button("+ Bevel").clicked() {
                                    actions.add_bevel_to_selected = true;
                                    close_menu = true;
                                }
                                if ui.button("+ Solidify").clicked() {
                                    actions.add_solidify_to_selected = true;
                                    close_menu = true;
                                }
                            });
                            ui.horizontal(|ui| {
                                if ui.button("+ Mirror X").clicked() {
                                    actions.add_mirror_to_selected = Some(MirrorAxis::X);
                                    close_menu = true;
                                }
                                if ui.button("Cloth").clicked() {
                                    actions.add_cloth_to_selected = true;
                                    close_menu = true;
                                }
                                if ui.button("Particles").clicked() {
                                    actions.add_particle_system_to_selected = true;
                                    close_menu = true;
                                }
                            });
                            if ui.button("Collapse All Modifiers").clicked() {
                                actions.collapse_modifiers = true;
                                close_menu = true;
                            }
                            ui.separator();

                            // Hierarchy & Organization
                            ui.label(egui::RichText::new("HIERARCHY & OPS").small().color(egui::Color32::GRAY));
                            let can_parent = state.previous_selected_node.is_some() && state.previous_selected_node != state.selected_node;
                            if ui.button("Group Selected (Ctrl+G)").clicked() {
                                actions.trigger_group_nodes = true;
                                close_menu = true;
                            }
                            if ui.button("Duplicate Object (Shift+D)").clicked() {
                                actions.trigger_duplicate_node = true;
                                close_menu = true;
                            }
                            if ui.add_enabled(can_parent, egui::Button::new("Live Boolean Carve (Diff)")).on_hover_text("Carves secondary selected cutter out of active object in real time").clicked() {
                                actions.trigger_boolean_carve = true;
                                close_menu = true;
                            }
                            if ui.add_enabled(can_parent, egui::Button::new("Set Parent (P)")).clicked() {
                                actions.trigger_parent = true;
                                close_menu = true;
                            }
                            if ui.button("Clear Parent (Shift+P)").clicked() {
                                actions.trigger_unparent = true;
                                close_menu = true;
                            }
                            if ui.add_enabled(can_parent, egui::Button::new("Join Objects (J)")).clicked() {
                                actions.trigger_join_meshes = true;
                                close_menu = true;
                            }
                            if ui.button("Separate Loose Parts").clicked() {
                                actions.trigger_separate_loose_parts = true;
                                close_menu = true;
                            }
                            ui.separator();
                            if ui.button(egui::RichText::new("Delete Object (X)").color(egui::Color32::from_rgb(255, 90, 90))).clicked() {
                                actions.delete_selected = true;
                                close_menu = true;
                            }
                        } else {
                            // Empty Viewport Space: Quick Add
                            draw_quick_add_menu(actions, &mut close_menu, ui);
                        }
                    }

                    // ==========================================
                    // 2. EDIT MODE CONTEXT MENU (Mesh Modeling)
                    // ==========================================
                    InteractionMode::Edit => {
                        let level_str = match state.sub_element_level {
                            SubElementLevel::Vertex => "Vertex",
                            SubElementLevel::Edge => "Edge",
                            SubElementLevel::Face => "Face",
                        };
                        ui.label(egui::RichText::new(format!("Edit Mode Context ({})", level_str)).strong().color(egui::Color32::from_rgb(255, 200, 80)));
                        ui.separator();

                        // Face Operations
                        if state.sub_element_level == SubElementLevel::Face || !state.selected_faces.is_empty() {
                            ui.label(egui::RichText::new("FACE OPS").small().color(egui::Color32::GRAY));
                            if ui.button("Extrude Region (E)").clicked() {
                                actions.trigger_extrude = true;
                                close_menu = true;
                            }
                            if ui.button("Inset Faces (I)").clicked() {
                                actions.trigger_inset = true;
                                close_menu = true;
                            }
                            if ui.button("Bevel Faces (Ctrl+B)").clicked() {
                                actions.trigger_bevel = true;
                                close_menu = true;
                            }
                            if ui.button("Un-Bevel / Dissolve Chamfer (Alt+Shift+B)").clicked() {
                                actions.trigger_un_bevel = true;
                                close_menu = true;
                            }
                            if ui.button("Poke Face").clicked() {
                                actions.trigger_poke_face = true;
                                close_menu = true;
                            }
                            if ui.button("Triangulate Faces").clicked() {
                                actions.trigger_triangulate = true;
                                close_menu = true;
                            }
                            if ui.button("Bisect Plane Cut").clicked() {
                                actions.trigger_bisect = true;
                                close_menu = true;
                            }
                            if ui.button("Separate Selected Faces").clicked() {
                                actions.trigger_separate_faces = true;
                                close_menu = true;
                            }
                            if ui.button("Cap Holes (Alt+F)").clicked() {
                                actions.trigger_cap_holes = true;
                                close_menu = true;
                            }
                            if ui.button("Grid Fill").clicked() {
                                actions.trigger_grid_fill = true;
                                close_menu = true;
                            }
                            if ui.button("Flip Normals (Alt+N)").clicked() {
                                actions.trigger_flip_normals = true;
                                close_menu = true;
                            }
                            if ui.button("Recalculate Normals Outside (Shift+N)").clicked() {
                                actions.trigger_recalculate_normals = Some(false);
                                close_menu = true;
                            }
                            if ui.button("Recalculate Normals Inside (Shift+Ctrl+N)").clicked() {
                                actions.trigger_recalculate_normals = Some(true);
                                close_menu = true;
                            }
                            ui.separator();
                        }

                        // Edge Operations
                        if state.sub_element_level == SubElementLevel::Edge || !state.selected_edges.is_empty() {
                            ui.label(egui::RichText::new("EDGE OPS").small().color(egui::Color32::GRAY));
                            if ui.button("Bevel Edges (Ctrl+B)").clicked() {
                                actions.trigger_bevel = true;
                                close_menu = true;
                            }
                            if ui.button("Loop Cut & Slide (Ctrl+R)").clicked() {
                                actions.trigger_loop_cut = true;
                                close_menu = true;
                            }
                            if ui.button("Knife Bisect Tool (K)").clicked() {
                                state.knife_active = true;
                                close_menu = true;
                            }
                            if ui.button("Mark Sharp Edge").clicked() {
                                actions.trigger_mark_sharp = true;
                                close_menu = true;
                            }
                            if ui.button("Clear Sharp Edge").clicked() {
                                actions.trigger_clear_sharp = true;
                                close_menu = true;
                            }
                            if ui.button("Mark UV Seam").clicked() {
                                actions.trigger_mark_seam = true;
                                close_menu = true;
                            }
                            if ui.button("Clear UV Seam").clicked() {
                                actions.trigger_clear_seam = true;
                                close_menu = true;
                            }
                            if ui.button("Slide Edges").clicked() {
                                actions.trigger_slide_edges = true;
                                close_menu = true;
                            }
                            if ui.button("Dissolve Edge").clicked() {
                                actions.trigger_dissolve_edge = true;
                                close_menu = true;
                            }
                            if ui.button("Bridge Edge Loops").clicked() {
                                actions.trigger_bridge_edge_loops = true;
                                close_menu = true;
                            }
                            ui.separator();
                        }

                        // Vertex Operations
                        if state.sub_element_level == SubElementLevel::Vertex || !state.selected_vertices.is_empty() {
                            ui.label(egui::RichText::new("VERTEX OPS").small().color(egui::Color32::GRAY));
                            if ui.button("Bevel Vertex (Shift+Ctrl+B)").clicked() {
                                actions.trigger_bevel_vertex = true;
                                close_menu = true;
                            }
                            if ui.button("Merge at Center (M)").clicked() {
                                actions.trigger_merge_vertices = true;
                                close_menu = true;
                            }
                            if ui.button("Rip Vertices (V)").clicked() {
                                actions.trigger_rip_vertices = true;
                                close_menu = true;
                            }
                            if ui.button("Connect Vertex Pair (J)").clicked() {
                                actions.trigger_connect_vertices = true;
                                close_menu = true;
                            }
                            ui.separator();
                        }

                        // Selection & Mode
                        ui.label(egui::RichText::new("SELECTION").small().color(egui::Color32::GRAY));
                        ui.horizontal(|ui| {
                            if ui.button("All (A)").clicked() {
                                actions.trigger_select_all = true;
                                close_menu = true;
                            }
                            if ui.button("Invert (Ctrl+I)").clicked() {
                                actions.trigger_invert_selection = true;
                                close_menu = true;
                            }
                        });
                        ui.horizontal(|ui| {
                            if ui.button("Loop (Alt+L)").clicked() {
                                actions.trigger_edge_loop = true;
                                close_menu = true;
                            }
                            if ui.button("Grow (+)").clicked() {
                                actions.trigger_grow_selection = true;
                                close_menu = true;
                            }
                        });
                        ui.separator();
                        if ui.button("Delete Selected (X)").clicked() {
                            actions.delete_selected = true;
                            close_menu = true;
                        }
                        if ui.button("Return to Object Mode (Tab)").clicked() {
                            state.interaction_mode = InteractionMode::Object;
                            close_menu = true;
                        }
                    }

                    // ==========================================
                    // 3. SCULPT MODE CONTEXT MENU
                    // ==========================================
                    InteractionMode::Sculpt => {
                        ui.label(egui::RichText::new("Sculpt Brush Context").strong().color(egui::Color32::from_rgb(255, 120, 60)));
                        ui.separator();
                        ui.horizontal(|ui| {
                            ui.label("Radius:");
                            ui.add(egui::Slider::new(&mut state.sculpt_settings.radius, 0.05..=3.0));
                        });
                        ui.horizontal(|ui| {
                            ui.label("Strength:");
                            ui.add(egui::Slider::new(&mut state.sculpt_settings.strength, 0.05..=2.0));
                        });
                        ui.separator();
                        ui.label(egui::RichText::new("SWITCH BRUSH").small().color(egui::Color32::GRAY));
                        if ui.selectable_label(state.sculpt_settings.brush == geo_kernel::SculptBrush::Draw, "Draw").clicked() {
                            state.sculpt_settings.brush = geo_kernel::SculptBrush::Draw;
                            close_menu = true;
                        }
                        if ui.selectable_label(state.sculpt_settings.brush == geo_kernel::SculptBrush::Clay, "Clay").clicked() {
                            state.sculpt_settings.brush = geo_kernel::SculptBrush::Clay;
                            close_menu = true;
                        }
                        if ui.selectable_label(state.sculpt_settings.brush == geo_kernel::SculptBrush::Inflate, "Inflate").clicked() {
                            state.sculpt_settings.brush = geo_kernel::SculptBrush::Inflate;
                            close_menu = true;
                        }
                        if ui.selectable_label(state.sculpt_settings.brush == geo_kernel::SculptBrush::Smooth, "Smooth").clicked() {
                            state.sculpt_settings.brush = geo_kernel::SculptBrush::Smooth;
                            close_menu = true;
                        }
                        if ui.selectable_label(state.sculpt_settings.brush == geo_kernel::SculptBrush::Grab, "Grab / Move").clicked() {
                            state.sculpt_settings.brush = geo_kernel::SculptBrush::Grab;
                            close_menu = true;
                        }
                        if ui.selectable_label(state.sculpt_settings.brush == geo_kernel::SculptBrush::SnakeHook, "Snake Hook").clicked() {
                            state.sculpt_settings.brush = geo_kernel::SculptBrush::SnakeHook;
                            close_menu = true;
                        }
                        if ui.selectable_label(state.sculpt_settings.brush == geo_kernel::SculptBrush::Mask, "Mask (M)").clicked() {
                            state.sculpt_settings.brush = geo_kernel::SculptBrush::Mask;
                            close_menu = true;
                        }
                        ui.separator();
                        ui.label(egui::RichText::new("MASK OPERATIONS").small().color(egui::Color32::GRAY));
                        if ui.button("Invert Mask (Ctrl+I)").clicked() {
                            actions.trigger_invert_sculpt_mask = true;
                            close_menu = true;
                        }
                        if ui.button("Clear Mask (Alt+M)").clicked() {
                            actions.trigger_clear_sculpt_mask = true;
                            close_menu = true;
                        }
                        if ui.button("Smooth Mask").clicked() {
                            actions.trigger_smooth_sculpt_mask = true;
                            close_menu = true;
                        }
                        ui.separator();
                        if ui.button("Exit to Object Mode (Tab)").clicked() {
                            state.interaction_mode = InteractionMode::Object;
                            close_menu = true;
                        }
                    }

                    // ==========================================
                    // 4. CURVE EDIT MODE CONTEXT MENU
                    // ==========================================
                    InteractionMode::CurveEdit => {
                        ui.label(egui::RichText::new("Curve Spline Context").strong().color(egui::Color32::from_rgb(100, 230, 200)));
                        ui.separator();
                        if ui.button("Extrude Knot (E)").clicked() {
                            if let Some(sel_id) = state.selected_node {
                                if let Some(node) = state.scene.nodes.get_mut(sel_id) {
                                    if let NodeData::Curve { spline, .. } = &mut node.data {
                                        let pt = geo_kernel::BezierPoint::new(
                                            core_math::Vec3::new(1.0, 0.0, 0.0),
                                            core_math::Vec3::new(-0.3, 0.0, 0.0),
                                            core_math::Vec3::new(0.3, 0.0, 0.0),
                                        );
                                        spline.add_point(pt);
                                        state.selected_curve_knot = Some(spline.points.len() - 1);
                                        node.is_dirty = true;
                                    }
                                }
                            }
                            close_menu = true;
                        }
                        if ui.button("Convert Curve to Pipe Mesh").clicked() {
                            actions.trigger_convert_curve = true;
                            close_menu = true;
                        }
                        ui.separator();
                        if ui.button("Exit to Object Mode (Tab)").clicked() {
                            state.interaction_mode = InteractionMode::Object;
                            close_menu = true;
                        }
                    }

                    _ => {
                        draw_quick_add_menu(actions, &mut close_menu, ui);
                    }
                }
            });

            // If user clicked outside the menu, dismiss it
            if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                close_menu = true;
            }
            if ui.input(|i| i.pointer.primary_clicked() || i.pointer.secondary_clicked()) {
                if let Some(mouse) = ui.input(|i| i.pointer.interact_pos()) {
                    if !response.response.rect.contains(mouse) {
                        close_menu = true;
                    }
                }
            }
        });

    if close_menu {
        state.show_context_menu = false;
        state.context_menu_pos = None;
    }
}

fn draw_quick_add_menu(actions: &mut UiActions, close_menu: &mut bool, ui: &mut egui::Ui) {
    ui.label(egui::RichText::new("Quick Add to Scene").strong().color(egui::Color32::from_rgb(120, 220, 160)));
    ui.separator();

    ui.label(egui::RichText::new("MESH PRIMITIVES").small().color(egui::Color32::GRAY));
    ui.horizontal(|ui| {
        if ui.button("Cube").clicked() {
            actions.spawn_mesh = Some(("Cube", Primitives::create_cube(2.0)));
            *close_menu = true;
        }
        if ui.button("Sphere").clicked() {
            actions.spawn_mesh = Some(("UVSphere", Primitives::create_uv_sphere(1.0, 16, 16)));
            *close_menu = true;
        }
    });
    ui.horizontal(|ui| {
        if ui.button("Cylinder").clicked() {
            actions.spawn_mesh = Some(("Cylinder", Primitives::create_cylinder(1.0, 2.0, 16)));
            *close_menu = true;
        }
        if ui.button("Plane").clicked() {
            actions.spawn_mesh = Some(("Plane", Primitives::create_plane(2.0, 2.0, 4, 4)));
            *close_menu = true;
        }
    });
    ui.horizontal(|ui| {
        if ui.button("Cone").clicked() {
            actions.spawn_mesh = Some(("Cone", Primitives::create_cone(1.0, 2.0, 16)));
            *close_menu = true;
        }
        if ui.button("Torus").clicked() {
            actions.spawn_mesh = Some(("Torus", Primitives::create_torus(1.0, 0.35, 16, 12)));
            *close_menu = true;
        }
        if ui.button("Plumb Bob").clicked() {
            actions.spawn_mesh = Some(("PlumbBob", Primitives::create_plumb_bob()));
            *close_menu = true;
        }
    });

    ui.separator();
    ui.label(egui::RichText::new("LIGHTS & CAMERAS").small().color(egui::Color32::GRAY));
    ui.horizontal(|ui| {
        if ui.button("Point").clicked() {
            actions.spawn_light = Some(("PointLight", LightType::Point));
            *close_menu = true;
        }
        if ui.button("Sun").clicked() {
            actions.spawn_light = Some(("SunLight", LightType::Directional));
            *close_menu = true;
        }
        if ui.button("Spot").clicked() {
            actions.spawn_light = Some(("SpotLight", LightType::Spot));
            *close_menu = true;
        }
    });
    if ui.button("Add Camera").clicked() {
        actions.spawn_camera = true;
        *close_menu = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ui_actions_defaults() {
        let actions = UiActions::default();
        assert!(actions.spawn_mesh.is_none());
        assert!(actions.spawn_light.is_none());
        assert!(!actions.spawn_camera);
        assert!(!actions.trigger_frame_selected);
        assert!(!actions.trigger_extrude);
        assert!(!actions.trigger_bevel);
        assert!(!actions.trigger_inset);
        assert!(!actions.delete_selected);
    }

    #[test]
    fn test_context_menu_action_setting() {
        let mut actions = UiActions::default();
        let mut close_menu = false;
        assert!(!close_menu);

        // Simulate choosing "Extrude (E)" from the context menu
        actions.trigger_extrude = true;
        close_menu = true;

        assert!(actions.trigger_extrude);
        assert!(close_menu);

        // Simulate choosing Add Cube
        actions.spawn_mesh = Some(("Cube", Primitives::create_cube(2.0)));
        assert!(actions.spawn_mesh.is_some());
        let (name, mesh) = actions.spawn_mesh.unwrap();
        assert_eq!(name, "Cube");
        assert_eq!(mesh.vertices.len(), 8);
        assert_eq!(mesh.faces.len(), 6);
    }
}

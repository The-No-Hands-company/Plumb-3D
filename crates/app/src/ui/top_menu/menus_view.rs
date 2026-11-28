use geo_kernel::*;
use graph_engine::*;
use render_viewport::*;

use crate::state::AppState;
use crate::types::*;
use crate::ui::actions::UiActions;

pub fn draw_mode_menu(state: &mut AppState, ui: &mut egui::Ui) {
    ui.menu_button("Mode", |ui| {
        if ui
            .selectable_label(
                state.interaction_mode == InteractionMode::Object,
                "Object Mode (Tab)",
            )
            .clicked()
        {
            state.interaction_mode = InteractionMode::Object;
            if let Some(sel_id) = state.selected_node {
                state.scene.mark_dirty(sel_id);
            }
            ui.close_menu();
        }
        if ui
            .selectable_label(
                state.interaction_mode == InteractionMode::Edit,
                "Edit Mode (Tab)",
            )
            .clicked()
        {
            state.interaction_mode = InteractionMode::Edit;
            if let Some(sel_id) = state.selected_node {
                state.scene.mark_dirty(sel_id);
            }
            ui.close_menu();
        }
        if ui
            .selectable_label(
                state.interaction_mode == InteractionMode::Sculpt,
                "Sculpt Mode",
            )
            .clicked()
        {
            state.interaction_mode = InteractionMode::Sculpt;
            if let Some(sel_id) = state.selected_node {
                state.scene.mark_dirty(sel_id);
            }
            ui.close_menu();
        }
        if ui
            .selectable_label(
                state.interaction_mode == InteractionMode::WeightPaint,
                "Weight Paint Mode",
            )
            .clicked()
        {
            state.interaction_mode = InteractionMode::WeightPaint;
            if let Some(sel_id) = state.selected_node {
                state.scene.mark_dirty(sel_id);
            }
            ui.close_menu();
        }
        if ui
            .selectable_label(
                state.interaction_mode == InteractionMode::VertexPaint,
                "Vertex Paint Mode",
            )
            .clicked()
        {
            state.interaction_mode = InteractionMode::VertexPaint;
            if let Some(sel_id) = state.selected_node {
                state.scene.mark_dirty(sel_id);
            }
            ui.close_menu();
        }
        if ui
            .selectable_label(
                state.interaction_mode == InteractionMode::TexturePaint,
                "Texture Paint Mode",
            )
            .clicked()
        {
            state.interaction_mode = InteractionMode::TexturePaint;
            if let Some(sel_id) = state.selected_node {
                state.scene.mark_dirty(sel_id);
            }
            ui.close_menu();
        }
        if ui
            .selectable_label(
                state.interaction_mode == InteractionMode::Pose,
                "Pose Mode (Rigging)",
            )
            .clicked()
        {
            state.interaction_mode = InteractionMode::Pose;
            if let Some(sel_id) = state.selected_node {
                state.scene.mark_dirty(sel_id);
            }
            ui.close_menu();
        }
        if ui
            .selectable_label(
                state.interaction_mode == InteractionMode::CurveEdit,
                "Curve Edit Mode",
            )
            .clicked()
        {
            state.interaction_mode = InteractionMode::CurveEdit;
            if let Some(sel_id) = state.selected_node {
                state.scene.mark_dirty(sel_id);
            }
            ui.close_menu();
        }
    });
}

pub fn draw_viewport_menu(state: &mut AppState, actions: &mut UiActions, ui: &mut egui::Ui) {
    ui.menu_button("Viewport", |ui| {
        ui.checkbox(&mut state.show_grid, "Ground Grid");
        ui.checkbox(&mut state.show_wireframe, "Wireframe Overlay");
        ui.checkbox(&mut state.show_subd_cage, "Subd Control Cage");
        ui.checkbox(&mut state.optimal_display, "Optimal Display (Subd)");
        ui.checkbox(&mut state.show_face_normals, "Show Face Normals");
        ui.checkbox(&mut state.show_vertex_normals, "Show Vertex Normals");
        if state.show_face_normals || state.show_vertex_normals {
            ui.add(
                egui::Slider::new(&mut state.normal_display_length, 0.05..=1.0)
                    .text("Normal Length"),
            );
        }
        ui.checkbox(&mut state.show_viewport_hud, "Heads-Up Statistics HUD");
        ui.separator();
        ui.label(
            egui::RichText::new("COLOR & TONEMAPPING")
                .small()
                .color(egui::Color32::GRAY),
        );
        ui.add(egui::Slider::new(&mut state.exposure, 0.1..=4.0).text("Exposure"));
        ui.horizontal(|ui| {
            ui.label("Tonemap:");
            ui.selectable_value(&mut state.tonemapping, TonemappingMode::Aces, "ACES");
            ui.selectable_value(
                &mut state.tonemapping,
                TonemappingMode::Reinhard,
                "Reinhard",
            );
            ui.selectable_value(&mut state.tonemapping, TonemappingMode::None, "Raw");
        });
        ui.horizontal(|ui| {
            ui.checkbox(&mut state.show_cavity, "Cavity & Ridges");
            if state.show_cavity {
                ui.add(egui::Slider::new(&mut state.cavity_intensity, 0.1..=1.5).text("Intensity"));
            }
        });
        ui.separator();
        if ui
            .checkbox(&mut state.flat_shading, "Flat Shading")
            .changed()
        {
            if let Some(node_id) = state.selected_node {
                if let Some(node) = state.scene.nodes.get_mut(node_id) {
                    if let NodeData::Mesh { shading_mode, .. } = &mut node.data {
                        *shading_mode = if state.flat_shading {
                            ShadingMode::Flat
                        } else {
                            ShadingMode::Smooth
                        };
                    }
                }
                state.scene.mark_dirty(node_id);
            }
        }
        ui.separator();
        ui.checkbox(&mut state.timeline_expanded, "Animation Timeline");
        ui.checkbox(&mut state.show_uv_editor, "2D UV Editor Window");
        ui.checkbox(&mut state.show_node_editor, "Geometry Nodes Canvas");
        ui.checkbox(&mut state.show_script_console, "Scripting Console (Rhai)");
        ui.separator();
        let is_persp = state.camera.projection == CameraProjection::Perspective;
        if ui
            .selectable_label(is_persp, "Perspective Camera")
            .clicked()
        {
            state.camera.projection = CameraProjection::Perspective;
            ui.close_menu();
        }
        if ui
            .selectable_label(!is_persp, "Orthographic Camera")
            .clicked()
        {
            state.camera.projection = CameraProjection::Orthographic;
            ui.close_menu();
        }
        ui.separator();
        let in_cam_view = state.active_camera_node.is_some();
        if ui
            .selectable_label(in_cam_view, "Active Camera View (Numpad 0)")
            .clicked()
        {
            if in_cam_view {
                state.active_camera_node = None;
            } else {
                let cam = state
                    .selected_node
                    .filter(|&id| {
                        matches!(
                            state.scene.nodes.get(id).map(|n| &n.data),
                            Some(NodeData::Camera { .. })
                        )
                    })
                    .or_else(|| {
                        state.scene.nodes.iter().find_map(|(id, node)| {
                            if matches!(node.data, NodeData::Camera { .. }) {
                                Some(id)
                            } else {
                                None
                            }
                        })
                    });
                if let Some(cam_id) = cam {
                    state.active_camera_node = Some(cam_id);
                }
            }
            ui.close_menu();
        }
        if ui.button("Align Camera to View (Ctrl+Numpad 0)").clicked() {
            let cam = state
                .active_camera_node
                .or_else(|| {
                    state.selected_node.filter(|&id| {
                        matches!(
                            state.scene.nodes.get(id).map(|n| &n.data),
                            Some(NodeData::Camera { .. })
                        )
                    })
                })
                .or_else(|| {
                    state.scene.nodes.iter().find_map(|(id, node)| {
                        if matches!(node.data, NodeData::Camera { .. }) {
                            Some(id)
                        } else {
                            None
                        }
                    })
                });

            if let Some(cam_id) = cam {
                let eye = state.camera.eye_position();
                let target = state.camera.target;
                let forward = (target - eye).normalize_or_zero();
                let right = forward.cross(core_math::Vec3::Y).normalize_or_zero();
                let up = right.cross(forward).normalize_or_zero();
                let rot_mat = core_math::Mat3::from_cols(right, up, -forward);
                let rot = core_math::Quat::from_mat3(&rot_mat);

                if let Some(cam_node) = state.scene.nodes.get_mut(cam_id) {
                    cam_node.local_transform.translation = eye;
                    cam_node.local_transform.rotation = rot;
                    state.scene.mark_dirty(cam_id);
                }
                state.active_camera_node = Some(cam_id);
            }
            ui.close_menu();
        }
        ui.checkbox(&mut state.lock_camera_to_view, "Lock Camera to View");
        ui.separator();
        if ui.button("Frame Selected (F)").clicked() {
            actions.trigger_frame_selected = true;
            ui.close_menu();
        }
        if ui.button("Reset Camera").clicked() {
            state.camera = ViewportCamera::new();
            state.camera.aspect =
                state.surface_config.width as f32 / state.surface_config.height as f32;
            ui.close_menu();
        }
    });
}

pub fn draw_shading_menu(state: &mut AppState, ui: &mut egui::Ui) {
    ui.menu_button("Shading", |ui| {
        ui.selectable_value(
            &mut state.viewport_shading,
            ViewportShadingMode::Studio,
            "Studio (Three-Point Lighting)",
        );
        ui.selectable_value(
            &mut state.viewport_shading,
            ViewportShadingMode::Pbr,
            "PBR (Physically Based Cook-Torrance)",
        );
        ui.selectable_value(
            &mut state.viewport_shading,
            ViewportShadingMode::MatCapRedWax,
            "MatCap Red Wax (Sculpt)",
        );
        ui.selectable_value(
            &mut state.viewport_shading,
            ViewportShadingMode::MatCapClay,
            "MatCap Clay (Form)",
        );
        ui.selectable_value(
            &mut state.viewport_shading,
            ViewportShadingMode::MatCapChrome,
            "MatCap Chrome (Reflection)",
        );
        ui.selectable_value(
            &mut state.viewport_shading,
            ViewportShadingMode::NormalInspection,
            "Normal Vector Inspection",
        );
        ui.selectable_value(
            &mut state.viewport_shading,
            ViewportShadingMode::Zebra,
            "Zebra Surfacing (Class-A Reflection)",
        );
        ui.selectable_value(
            &mut state.viewport_shading,
            ViewportShadingMode::GaussianCurvature,
            "Gaussian Curvature Heatmap (Class-A)",
        );
        ui.selectable_value(
            &mut state.viewport_shading,
            ViewportShadingMode::MeanCurvature,
            "Mean Curvature Heatmap (Class-A)",
        );
        ui.separator();
        ui.selectable_value(
            &mut state.viewport_shading,
            ViewportShadingMode::Rendered,
            "Rendered (Live Path Tracer)",
        );
        ui.separator();
        ui.checkbox(&mut state.ssr.enabled, "Screen-Space Reflections (SSR)");
        ui.checkbox(&mut state.dof.enabled, "Depth of Field (DoF)");
        if state.dof.enabled {
            ui.add(
                egui::Slider::new(&mut state.dof.focus_distance, 0.5..=20.0).text("Focus Dist (m)"),
            );
            ui.add(egui::Slider::new(&mut state.dof.f_stop, 1.2..=16.0).text("f-stop"));
        }
    });
}

pub fn draw_uv_menu(state: &mut AppState, actions: &mut UiActions, ui: &mut egui::Ui) {
    ui.menu_button("UV", |ui| {
        if ui
            .add_enabled(
                state.selected_node.is_some(),
                egui::Button::new("Unwrap (Seams) (U)"),
            )
            .clicked()
        {
            actions.trigger_unwrap_seams = true;
            ui.close_menu();
        }
        if ui
            .add_enabled(
                state.selected_node.is_some(),
                egui::Button::new("Smart UV Project"),
            )
            .clicked()
        {
            actions.trigger_smart_uv_project = true;
            ui.close_menu();
        }
        ui.separator();
        let can_seam =
            state.interaction_mode == InteractionMode::Edit && !state.selected_edges.is_empty();
        if ui
            .add_enabled(can_seam, egui::Button::new("Mark Seam"))
            .clicked()
        {
            actions.trigger_mark_seam = true;
            ui.close_menu();
        }
        if ui
            .add_enabled(can_seam, egui::Button::new("Clear Seam"))
            .clicked()
        {
            actions.trigger_clear_seam = true;
            ui.close_menu();
        }
        if ui
            .add_enabled(
                state.selected_node.is_some(),
                egui::Button::new("Clear All Seams"),
            )
            .clicked()
        {
            actions.trigger_clear_all_seams = true;
            ui.close_menu();
        }
        ui.separator();
        if ui
            .add_enabled(
                state.selected_node.is_some(),
                egui::Button::new("Planar Projection"),
            )
            .clicked()
        {
            actions.trigger_planar_uv = true;
            ui.close_menu();
        }
        if ui
            .add_enabled(
                state.selected_node.is_some(),
                egui::Button::new("Box / Cube Projection"),
            )
            .clicked()
        {
            actions.trigger_box_uv = true;
            ui.close_menu();
        }
        if ui
            .add_enabled(
                state.selected_node.is_some(),
                egui::Button::new("Cylindrical Projection"),
            )
            .clicked()
        {
            actions.trigger_cylindrical_uv = true;
            ui.close_menu();
        }
        if ui
            .add_enabled(
                state.selected_node.is_some(),
                egui::Button::new("Spherical Projection"),
            )
            .clicked()
        {
            actions.trigger_spherical_uv = true;
            ui.close_menu();
        }
        ui.separator();
        ui.checkbox(&mut state.show_uv_editor, "2D UV Editor Window");
    });
}

pub fn draw_window_menu(state: &mut AppState, ui: &mut egui::Ui) {
    ui.menu_button("Window", |ui| {
        if ui.button("Command Palette (F3)").clicked() {
            state.command_palette.is_open = true;
            state.command_palette.focus_requested = false;
            ui.close_menu();
        }
        if ui.button("Preferences (Ctrl+,)").clicked() {
            state.preferences.show_dialog = true;
            ui.close_menu();
        }
        ui.separator();
        ui.menu_button("Themes", |ui| {
            for preset in crate::ui::theme::ThemePreset::ALL {
                let is_cur = state.current_theme == preset;
                if ui.selectable_label(is_cur, preset.name()).clicked() {
                    state.set_theme(preset);
                    ui.close_menu();
                }
            }
        });
        ui.separator();
        ui.checkbox(
            &mut state.docking_state.is_enabled,
            "Drag-and-Drop Docking Workspace",
        );
        ui.separator();
        ui.checkbox(&mut state.show_outliner, "Scene Outliner");
        ui.checkbox(&mut state.show_inspector, "Properties Inspector");
        ui.checkbox(&mut state.show_left_toolbar, "Left Tool Strip");
        ui.checkbox(&mut state.show_viewport_hud, "Scene Statistics HUD");
        ui.separator();
        ui.checkbox(&mut state.show_node_editor, "Geometry Nodes Canvas");
        ui.checkbox(&mut state.show_uv_editor, "2D UV Editor Window");
        ui.checkbox(
            &mut state.show_script_console,
            "Multi-Engine Scripting Console",
        );
        ui.checkbox(
            &mut state.show_collab_dialog,
            "Live Collaboration Session (Figma for 3D)",
        );
        ui.checkbox(
            &mut state.xr_session.is_active,
            "OpenXR Spatial Computing (VR 6-DoF)",
        );
        ui.checkbox(
            &mut state.production_render.show_dialog,
            "Production Render Result",
        );
        ui.checkbox(&mut state.bake_state.show_dialog, "PBR Texture Baker");
        ui.checkbox(
            &mut state.print_inspector.show_dialog,
            "3D Print Quality Inspector",
        );
        ui.checkbox(&mut state.show_help_dialog, "User Manual & Shortcuts (F1)");
    });
}

pub fn draw_help_menu(state: &mut AppState, ui: &mut egui::Ui) {
    ui.menu_button("Help", |ui| {
        if ui.button("🌕 Lunar Night Sky Walkthrough").clicked() {
            crate::scenes::build_lunar_night_sky_scene(state);
            ui.close_menu();
        }
        ui.separator();
        for (label, tab) in [
            ("User Manual & Shortcuts (F1)", 0),
            ("Beginner Tutorials", 1),
            ("Modeling & Modals Guide", 2),
            ("Geometry Nodes Guide", 3),
            ("Rhai Scripting API", 4),
            ("Path Tracer Rendering", 5),
            ("3D Printing & CAD Interchange", 6),
        ] {
            if ui.button(label).clicked() {
                state.show_help_dialog = true;
                state.help_dialog_tab = tab;
                ui.close_menu();
            }
        }
        ui.separator();
        if ui.button("About Plumb 3D").clicked() {
            state.show_help_dialog = true;
            state.help_dialog_tab = 7;
            ui.close_menu();
        }
    });
}

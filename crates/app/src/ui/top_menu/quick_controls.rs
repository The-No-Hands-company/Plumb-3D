use graph_engine::NodeData;
use render_viewport::{GizmoMode, TransformOrientation, ViewportShadingMode};

use crate::state::AppState;
use crate::types::*;
use crate::ui::workspaces::WorkspaceTemplate;

/// Dedicated Viewport Ribbon Header Bar (Blender 4.x / Plasticity / Unreal Engine layout)
pub fn draw_quick_controls(state: &mut AppState, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        // 1. Interaction Mode Switcher Pills
        ui.label(egui::RichText::new("MODE:").small().strong().color(egui::Color32::from_rgb(140, 150, 170)));
        if ui.selectable_label(state.interaction_mode == InteractionMode::Object, "📦 Object").clicked() {
            state.interaction_mode = InteractionMode::Object;
        }
        if ui.selectable_label(state.interaction_mode == InteractionMode::Edit, "✏ Edit").clicked() {
            state.interaction_mode = InteractionMode::Edit;
        }
        if state.current_workspace == WorkspaceTemplate::Sculpting || state.interaction_mode == InteractionMode::Sculpt {
            if ui.selectable_label(state.interaction_mode == InteractionMode::Sculpt, "🎨 Sculpt").clicked() {
                state.interaction_mode = InteractionMode::Sculpt;
            }
        }
        if ui.selectable_label(state.interaction_mode == InteractionMode::CurveEdit, "〰 Curve").clicked() {
            state.interaction_mode = InteractionMode::CurveEdit;
        }

        ui.separator();

        // 2. Sub-Element Level Pills (in Edit Mode)
        if state.interaction_mode == InteractionMode::Edit {
            if ui.selectable_label(state.sub_element_level == SubElementLevel::Vertex, "⏺ Vert (1)").clicked() {
                state.sub_element_level = SubElementLevel::Vertex;
            }
            if ui.selectable_label(state.sub_element_level == SubElementLevel::Edge, "➖ Edge (2)").clicked() {
                state.sub_element_level = SubElementLevel::Edge;
            }
            if ui.selectable_label(state.sub_element_level == SubElementLevel::Face, "⏹ Face (3)").clicked() {
                state.sub_element_level = SubElementLevel::Face;
            }
            ui.separator();
        }

        // 3. Transform Tool Pills (Translate, Rotate, Scale)
        ui.label(egui::RichText::new("TOOL:").small().strong().color(egui::Color32::from_rgb(140, 150, 170)));
        if ui.selectable_label(state.gizmo.mode == GizmoMode::Translate, "✥ Move (W)").clicked() {
            state.gizmo.mode = GizmoMode::Translate;
        }
        if ui.selectable_label(state.gizmo.mode == GizmoMode::Rotate, "⟲ Rotate (E)").clicked() {
            state.gizmo.mode = GizmoMode::Rotate;
        }
        if ui.selectable_label(state.gizmo.mode == GizmoMode::Scale, "⤢ Scale (R)").clicked() {
            state.gizmo.mode = GizmoMode::Scale;
        }

        // 4. Orientation Space Dropdown Pill
        egui::ComboBox::from_id_salt("quick_orientation_combo")
            .selected_text(match state.transform_orientation {
                TransformOrientation::Global => "🌐 Global",
                TransformOrientation::Local => "📐 Local",
                TransformOrientation::Normal => "⚓ Normal",
                TransformOrientation::View => "👁 View",
            })
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut state.transform_orientation, TransformOrientation::Global, "🌐 Global");
                ui.selectable_value(&mut state.transform_orientation, TransformOrientation::Local, "📐 Local");
                ui.selectable_value(&mut state.transform_orientation, TransformOrientation::Normal, "⚓ Normal");
                ui.selectable_value(&mut state.transform_orientation, TransformOrientation::View, "👁 View");
            });

        ui.separator();

        // 5. Viewport Shading Dropdown Pill
        egui::ComboBox::from_id_salt("quick_shading_combo")
            .selected_text(match state.viewport_shading {
                ViewportShadingMode::Studio => "Studio Shading",
                ViewportShadingMode::Pbr => "PBR Shading",
                ViewportShadingMode::MatCapRedWax => "MatCap Red Wax",
                ViewportShadingMode::MatCapClay => "MatCap Clay",
                ViewportShadingMode::MatCapChrome => "MatCap Chrome",
                ViewportShadingMode::NormalInspection => "Normals Inspection",
                ViewportShadingMode::Zebra => "🦓 Zebra Surfacing",
                ViewportShadingMode::Rendered => "☀ Rendered (Cycles)",
            })
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut state.viewport_shading, ViewportShadingMode::Studio, "Studio");
                ui.selectable_value(&mut state.viewport_shading, ViewportShadingMode::Pbr, "PBR Materials");
                ui.selectable_value(&mut state.viewport_shading, ViewportShadingMode::MatCapRedWax, "MatCap Red Wax");
                ui.selectable_value(&mut state.viewport_shading, ViewportShadingMode::MatCapClay, "MatCap Clay");
                ui.selectable_value(&mut state.viewport_shading, ViewportShadingMode::MatCapChrome, "MatCap Chrome");
                ui.selectable_value(&mut state.viewport_shading, ViewportShadingMode::NormalInspection, "Normals Inspection");
                ui.selectable_value(&mut state.viewport_shading, ViewportShadingMode::Zebra, "🦓 Zebra Surfacing (Class-A)");
                ui.separator();
                ui.selectable_value(&mut state.viewport_shading, ViewportShadingMode::Rendered, "☀ Rendered (Live Path Tracer)");
            });

        // 6. Viewport Options (Wireframe, Grid, Snap)
        if ui.selectable_label(state.show_wireframe, "🔲 Wire").clicked() {
            state.show_wireframe = !state.show_wireframe;
        }
        if ui.selectable_label(state.show_grid, "🌐 Grid").clicked() {
            state.show_grid = !state.show_grid;
        }
        let wp_label = if state.workplane.is_active { "📐 Plane (Active)" } else { "📐 Plane" };
        if ui.selectable_label(state.workplane.is_active, wp_label).on_hover_text("Align CAD Workplane to Face (Alt+W)").clicked() {
            crate::commands::workplane::toggle_workplane_align(state);
        }
        if ui.button("🪞 Sym").on_hover_text("Symmetrize & Bisect-Weld across Workplane (Alt+X)").clicked() {
            crate::commands::workplane::symmetrize_active(state);
        }
        let snap_text = if state.snap_enabled {
            match state.snap_target {
                SnapTarget::Vertex => "🧲 Snap (V)",
                SnapTarget::EdgeMidpoint => "🧲 Edge",
                SnapTarget::FaceCenter => "🧲 Face",
                SnapTarget::Grid => "🧲 Grid",
            }
        } else {
            "🧲 Snap"
        };
        if ui.selectable_label(state.snap_enabled, snap_text).clicked() {
            state.snap_enabled = !state.snap_enabled;
        }

        // Quick snap target & grid step selector
        egui::ComboBox::from_id_salt("quick_snap_selector")
            .selected_text(match state.snap_target {
                SnapTarget::Vertex => "📍 Vertex",
                SnapTarget::EdgeMidpoint => "📏 Midpoint",
                SnapTarget::FaceCenter => "▫ Face Center",
                SnapTarget::Grid => match state.grid_snap_size {
                    s if (s - 0.001).abs() < 1e-4 => "🌐 1mm",
                    s if (s - 0.01).abs() < 1e-4 => "🌐 10mm",
                    s if (s - 0.1).abs() < 1e-4 => "🌐 10cm",
                    s if (s - 1.0).abs() < 1e-4 => "🌐 1m",
                    _ => "🌐 Grid",
                },
            })
            .width(80.0)
            .show_ui(ui, |ui| {
                if ui.selectable_label(state.snap_target == SnapTarget::Vertex, "📍 Vertex (V)").clicked() {
                    state.snap_target = SnapTarget::Vertex;
                    state.snap_enabled = true;
                }
                if ui.selectable_label(state.snap_target == SnapTarget::EdgeMidpoint, "📏 Edge Midpoint").clicked() {
                    state.snap_target = SnapTarget::EdgeMidpoint;
                    state.snap_enabled = true;
                }
                if ui.selectable_label(state.snap_target == SnapTarget::FaceCenter, "▫ Face Center").clicked() {
                    state.snap_target = SnapTarget::FaceCenter;
                    state.snap_enabled = true;
                }
                ui.separator();
                ui.label(egui::RichText::new("Grid Increment").small().weak());
                if ui.selectable_label(state.snap_target == SnapTarget::Grid && (state.grid_snap_size - 0.001).abs() < 1e-4, "🌐 1mm (0.001m)").clicked() {
                    state.snap_target = SnapTarget::Grid;
                    state.grid_snap_size = 0.001;
                    state.snap_enabled = true;
                }
                if ui.selectable_label(state.snap_target == SnapTarget::Grid && (state.grid_snap_size - 0.01).abs() < 1e-4, "🌐 10mm (0.01m)").clicked() {
                    state.snap_target = SnapTarget::Grid;
                    state.grid_snap_size = 0.01;
                    state.snap_enabled = true;
                }
                if ui.selectable_label(state.snap_target == SnapTarget::Grid && (state.grid_snap_size - 0.1).abs() < 1e-4, "🌐 10cm (0.1m)").clicked() {
                    state.snap_target = SnapTarget::Grid;
                    state.grid_snap_size = 0.1;
                    state.snap_enabled = true;
                }
                if ui.selectable_label(state.snap_target == SnapTarget::Grid && (state.grid_snap_size - 1.0).abs() < 1e-4, "🌐 1m (1.0m)").clicked() {
                    state.snap_target = SnapTarget::Grid;
                    state.grid_snap_size = 1.0;
                    state.snap_enabled = true;
                }
            });

        // Camera mode selector
        egui::ComboBox::from_id_salt("quick_camera_combo")
            .selected_text(if let Some(cam_id) = state.active_camera_node {
                state.scene.nodes.get(cam_id).map(|n| format!("📷 {}", n.name)).unwrap_or_else(|| "📷 View".into())
            } else {
                "📷 Free Orbit".into()
            })
            .show_ui(ui, |ui| {
                if ui.selectable_label(state.active_camera_node.is_none(), "Free Orbit Viewport").clicked() {
                    state.active_camera_node = None;
                }
                for (node_id, node) in &state.scene.nodes {
                    if let NodeData::Camera { .. } = &node.data {
                        let is_cur = state.active_camera_node == Some(node_id);
                        if ui.selectable_label(is_cur, format!("📷 {}", node.name)).clicked() {
                            state.active_camera_node = Some(node_id);
                        }
                    }
                }
            });

        // 7. Right Aligned: Collapsible Panel Toggles
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let insp_text = if state.show_inspector { "Inspector ▶" } else { "◀ Inspector" };
            if ui.selectable_label(state.show_inspector, insp_text).clicked() {
                state.show_inspector = !state.show_inspector;
            }

            let outl_text = if state.show_outliner { "◀ Outliner" } else { "Outliner ▶" };
            if ui.selectable_label(state.show_outliner, outl_text).clicked() {
                state.show_outliner = !state.show_outliner;
            }

            if ui.selectable_label(state.show_viewport_hud, "📊 HUD").clicked() {
                state.show_viewport_hud = !state.show_viewport_hud;
            }

            if ui.selectable_label(state.show_left_toolbar, "🛠 Tools").clicked() {
                state.show_left_toolbar = !state.show_left_toolbar;
            }
        });
    });
}

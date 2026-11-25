use core_math::vec3;
use geo_kernel::*;
use graph_engine::*;
use render_viewport::*;

use crate::state::AppState;
use crate::types::*;
use crate::ui::actions::UiActions;

pub fn draw_file_menu(state: &mut AppState, actions: &mut UiActions, ui: &mut egui::Ui) {
    ui.menu_button("File", |ui| {
        if ui.button("✨ Welcome & Starter Hub (F2)").clicked() {
            state.welcome_hub.show_dialog = true;
            ui.close_menu();
        }
        if ui.button("New Scene").clicked() {
            state.scene = SceneGraph::new();
            state.gpu_mesh_cache.clear();
            state.object_colors.clear();
            state.object_materials.clear();
            state.constraints.constraints.clear();
            state.selected_node = None;
            state.previous_selected_node = None;
            state.selected_vertices.clear();
            state.selected_edges.clear();
            state.selected_faces.clear();
            ui.close_menu();
        }
        ui.separator();
        if ui.button("💾 Save Project (Ctrl+S)").clicked() {
            actions.export_plmb = true;
            ui.close_menu();
        }
        if ui.button("💾 Save Project As... (Ctrl+Shift+S)").clicked() {
            actions.export_plmb_as = true;
            ui.close_menu();
        }
        if ui.button("📂 Open Project... (Ctrl+O)").clicked() {
            actions.trigger_open_plmb = true;
            ui.close_menu();
        }
        if ui.button("📥 Import 3D Model... (plmb, obj, stl, ply, gltf)").clicked() {
            actions.trigger_import_dialog = true;
            ui.close_menu();
        }
        ui.separator();
        if ui.button("Export Active Mesh as STL for 3D Printing (.stl)").clicked() { actions.export_stl = true; ui.close_menu(); }
        if ui.button("Export Active Mesh as OBJ (.obj)").clicked() { actions.export_obj = true; ui.close_menu(); }
        if ui.button("Export Active Mesh as glTF 2.0 (.gltf)").clicked() { actions.export_gltf = true; ui.close_menu(); }
        if ui.button("Export Active Mesh as PLY (.ply)").clicked() { actions.export_ply = true; ui.close_menu(); }
        if ui.button("🖨 3D Print & CAD Quality Inspector...").clicked() { state.print_inspector.show_dialog = true; ui.close_menu(); }
        ui.separator();
        if ui.button("Import Sample glTF Cube").clicked() {
            let sample_gltf = Primitives::create_cube(2.0).to_gltf("ImportedCube");
            actions.import_gltf_text = Some(sample_gltf);
            ui.close_menu();
        }
        if ui.button("Import Sample OBJ Cylinder").clicked() {
            let sample_obj = Primitives::create_cylinder(1.5, 3.0, 12).to_obj("ImportedCylinder");
            actions.import_obj_text = Some(sample_obj);
            ui.close_menu();
        }
        if ui.button("Import Sample STL Sphere").clicked() {
            let sample_stl = Primitives::create_uv_sphere(1.2, 16, 16).to_stl_binary();
            actions.import_stl_bytes = Some(sample_stl);
            ui.close_menu();
        }
        if ui.button("Import Sample PLY Torus").clicked() {
            let sample_ply = Primitives::create_torus(1.2, 0.4, 16, 16).to_ply_binary();
            actions.import_ply_bytes = Some(sample_ply);
            ui.close_menu();
        }
    });
}

pub fn draw_edit_menu(state: &mut AppState, actions: &mut UiActions, ui: &mut egui::Ui) {
    ui.menu_button("Edit", |ui| {
        let undo_text = if let Some(name) = state.history.undo_name() {
            format!("Undo {} (Ctrl+Z)", name)
        } else {
            "Undo (Ctrl+Z)".into()
        };
        if ui.add_enabled(state.history.can_undo(), egui::Button::new(undo_text)).clicked() {
            actions.trigger_undo = true;
            ui.close_menu();
        }

        let redo_text = if let Some(name) = state.history.redo_name() {
            format!("Redo {} (Ctrl+Y)", name)
        } else {
            "Redo (Ctrl+Y)".into()
        };
        if ui.add_enabled(state.history.can_redo(), egui::Button::new(redo_text)).clicked() {
            actions.trigger_redo = true;
            ui.close_menu();
        }
        if ui.add_enabled(state.selected_node.is_some(), egui::Button::new("Duplicate Object (Shift+D)")).clicked() {
            actions.trigger_duplicate_node = true;
            ui.close_menu();
        }
        ui.separator();
        let can_join = state.selected_node.is_some()
            && state.previous_selected_node.is_some()
            && state.selected_node != state.previous_selected_node;
        if ui.add_enabled(can_join, egui::Button::new("Join Meshes (Ctrl+J)")).clicked() {
            actions.trigger_join_meshes = true;
            ui.close_menu();
        }
        let can_separate = state.interaction_mode == InteractionMode::Edit && !state.selected_faces.is_empty();
        if ui.add_enabled(can_separate, egui::Button::new("Separate Faces (P)")).clicked() {
            actions.trigger_separate_faces = true;
            ui.close_menu();
        }
        if ui.add_enabled(state.selected_node.is_some(), egui::Button::new("Separate by Material")).clicked() {
            actions.trigger_separate_by_material = true;
            ui.close_menu();
        }
        if ui.add_enabled(state.interaction_mode == InteractionMode::Edit, egui::Button::new("Inset Faces (I)")).clicked() {
            actions.trigger_inset = true;
            ui.close_menu();
        }
        if ui.add_enabled(state.interaction_mode == InteractionMode::Edit, egui::Button::new("Flip Normals (Alt+N)")).clicked() {
            actions.trigger_flip_normals = true;
            ui.close_menu();
        }
        if ui.add_enabled(state.selected_node.is_some(), egui::Button::new("Bisect / Slice Plane")).clicked() {
            actions.trigger_bisect = true;
            ui.close_menu();
        }
        if ui.add_enabled(state.interaction_mode == InteractionMode::Edit, egui::Button::new("Triangulate Faces (Ctrl+T)")).clicked() {
            actions.trigger_triangulate = true;
            ui.close_menu();
        }
        if ui.add_enabled(state.interaction_mode == InteractionMode::Edit, egui::Button::new("Symmetrize Mesh")).clicked() {
            actions.trigger_symmetrize = true;
            ui.close_menu();
        }
        if ui.add_enabled(state.interaction_mode == InteractionMode::Edit, egui::Button::new("Spin Revolve")).clicked() {
            actions.trigger_spin = true;
            ui.close_menu();
        }
        if ui.add_enabled(state.selected_node.is_some(), egui::Button::new("Clean Degenerate Geometry")).clicked() {
            actions.trigger_clean_degenerate = true;
            ui.close_menu();
        }
        if ui.add_enabled(state.interaction_mode == InteractionMode::Edit, egui::Button::new("Cap Holes / Fill Boundaries")).clicked() {
            actions.trigger_cap_holes = true;
            ui.close_menu();
        }
        if ui.add_enabled(state.interaction_mode == InteractionMode::Edit, egui::Button::new("Grid Fill")).clicked() {
            actions.trigger_grid_fill = true;
            ui.close_menu();
        }
        if ui.add_enabled(state.interaction_mode == InteractionMode::Edit && state.selected_vertices.len() >= 2, egui::Button::new("Pick Shortest Path")).clicked() {
            actions.trigger_shortest_path = true;
            ui.close_menu();
        }
        if ui.add_enabled(state.interaction_mode == InteractionMode::Edit && !state.selected_edges.is_empty(), egui::Button::new("Extract Curve from Edges")).clicked() {
            actions.trigger_extract_curve = true;
            ui.close_menu();
        }
        if ui.add_enabled(state.interaction_mode == InteractionMode::Edit && !state.selected_vertices.is_empty(), egui::Button::new("Shrink / Fatten (Alt+S)")).clicked() {
            actions.trigger_shrink_fatten = true;
            ui.close_menu();
        }
        if ui.add_enabled(state.interaction_mode == InteractionMode::Edit && !state.selected_vertices.is_empty(), egui::Button::new("Relax / Smooth Vertices")).clicked() {
            actions.trigger_relax_vertices = true;
            ui.close_menu();
        }
        if ui.add_enabled(state.interaction_mode == InteractionMode::Edit && !state.selected_vertices.is_empty(), egui::Button::new("✂ Rip Vertices (V)")).clicked() {
            actions.trigger_rip_vertices = true;
            ui.close_menu();
        }
        if ui.add_enabled(state.interaction_mode == InteractionMode::Edit, egui::Button::new("✂ Knife Multi-Cut (K)")).clicked() {
            state.knife_active = !state.knife_active;
            if !state.knife_active {
                state.knife_points.clear();
                state.knife_hovered_hit = None;
            }
            ui.close_menu();
        }
        ui.separator();
        ui.checkbox(&mut state.x_mirror_editing, "X-Mirror Symmetry");
    });
}

pub fn draw_add_menu(actions: &mut UiActions, ui: &mut egui::Ui) {
    ui.menu_button("Add", |ui| {
        ui.menu_button("📦 Mesh Primitives", |ui| {
            if ui.button("Box / Cube").clicked() { actions.spawn_mesh = Some(("Cube", Primitives::create_cube(2.0))); ui.close_menu(); }
            if ui.button("Plane Grid").clicked() { actions.spawn_mesh = Some(("Plane", Primitives::create_plane(4.0, 4.0, 4, 4))); ui.close_menu(); }
            if ui.button("UV Sphere").clicked() { actions.spawn_mesh = Some(("Sphere", Primitives::create_uv_sphere(1.2, 16, 16))); ui.close_menu(); }
            if ui.button("Cylinder").clicked() { actions.spawn_mesh = Some(("Cylinder", Primitives::create_cylinder(1.0, 2.5, 16))); ui.close_menu(); }
            if ui.button("Torus / Donut").clicked() { actions.spawn_mesh = Some(("Torus", Primitives::create_torus(1.5, 0.4, 24, 16))); ui.close_menu(); }
            if ui.button("Cone").clicked() { actions.spawn_mesh = Some(("Cone", Primitives::create_cone(1.2, 2.5, 16))); ui.close_menu(); }
            if ui.button("Pyramid").clicked() { actions.spawn_mesh = Some(("Pyramid", Primitives::create_pyramid(2.0, 2.5))); ui.close_menu(); }
            if ui.button("Capsule / Pill").clicked() { actions.spawn_mesh = Some(("Capsule", Primitives::create_capsule(0.8, 1.8, 16, 6))); ui.close_menu(); }
            if ui.button("IcoSphere (Geodesic)").clicked() { actions.spawn_mesh = Some(("IcoSphere", Primitives::create_icosphere(1.2, 2))); ui.close_menu(); }
            if ui.button("📍 Plumb Bob (Mascot)").clicked() { actions.spawn_mesh = Some(("PlumbBob", Primitives::create_plumb_bob())); ui.close_menu(); }
        });
        ui.menu_button("〰 Curves & Splines", |ui| {
            if ui.button("Bézier Path").clicked() { actions.spawn_curve = Some(0); ui.close_menu(); }
            if ui.button("Bézier Circle").clicked() { actions.spawn_curve = Some(1); ui.close_menu(); }
            if ui.button("Helical Spiral").clicked() { actions.spawn_curve = Some(2); ui.close_menu(); }
            ui.separator();
            if ui.button("Lathe Vase (Spline)").clicked() {
                let profile = vec![(0.0, -1.2), (0.7, -1.2), (0.15, -0.3), (0.2, 0.6), (0.8, 1.3), (0.0, 1.3)];
                if let Ok(m) = Spline::lathe(&profile, 24, 2.0 * std::f32::consts::PI) { actions.spawn_mesh = Some(("LatheVase", m)); }
                ui.close_menu();
            }
            if ui.button("Curved Pipe (Spline)").clicked() {
                let circle = Spline::circle_cross_section(0.25, 12);
                let mut spline = Spline::new();
                spline.add_point(BezierPoint::smooth(vec3(-2.0, 0.0, -1.0), vec3(0.0, 1.5, 0.0)));
                spline.add_point(BezierPoint::smooth(vec3(0.0, 2.0, 0.0), vec3(1.5, 0.0, 0.0)));
                spline.add_point(BezierPoint::smooth(vec3(2.0, 0.5, 1.0), vec3(0.0, -1.0, 0.0)));
                let path = spline.sample(12);
                if let Ok(m) = Spline::sweep(&circle, &path, false) { actions.spawn_mesh = Some(("CurvedPipe", m)); }
                ui.close_menu();
            }
            ui.separator();
            if ui.button("Loft Selected Curves (Ctrl+L)").clicked() { actions.trigger_loft_curves = true; ui.close_menu(); }
            if ui.button("✨ Fillet Curve Corners (0.2m)").clicked() { actions.trigger_fillet_curve = Some(0.2); ui.close_menu(); }
            if ui.button("🏺 Revolve / Lathe Curve (360°)").clicked() { actions.trigger_lathe_curve = Some((16, 360.0)); ui.close_menu(); }
        });
        ui.menu_button("💡 Lights", |ui| {
            if ui.button("Directional Light (Sun)").clicked() { actions.spawn_light = Some(("SunLight", LightType::Directional)); ui.close_menu(); }
            if ui.button("Point Light (Omni)").clicked() { actions.spawn_light = Some(("PointLight", LightType::Point)); ui.close_menu(); }
            if ui.button("Spot Light (Cone)").clicked() { actions.spawn_light = Some(("SpotLight", LightType::Spot)); ui.close_menu(); }
        });
        if ui.button("📷 Camera").clicked() { actions.spawn_camera = true; ui.close_menu(); }
        if ui.button("🦴 Joint / Bone Armature").clicked() { actions.spawn_joint = true; ui.close_menu(); }
    });
}

pub fn draw_mode_menu(state: &mut AppState, ui: &mut egui::Ui) {
    ui.menu_button("Mode", |ui| {
        if ui.selectable_label(state.interaction_mode == InteractionMode::Object, "Object Mode (Tab)").clicked() {
            state.interaction_mode = InteractionMode::Object;
            if let Some(sel_id) = state.selected_node { state.scene.mark_dirty(sel_id); }
            ui.close_menu();
        }
        if ui.selectable_label(state.interaction_mode == InteractionMode::Edit, "Edit Mode (Tab)").clicked() {
            state.interaction_mode = InteractionMode::Edit;
            if let Some(sel_id) = state.selected_node { state.scene.mark_dirty(sel_id); }
            ui.close_menu();
        }
        if ui.selectable_label(state.interaction_mode == InteractionMode::Sculpt, "Sculpt Mode").clicked() {
            state.interaction_mode = InteractionMode::Sculpt;
            if let Some(sel_id) = state.selected_node { state.scene.mark_dirty(sel_id); }
            ui.close_menu();
        }
        if ui.selectable_label(state.interaction_mode == InteractionMode::WeightPaint, "Weight Paint Mode").clicked() {
            state.interaction_mode = InteractionMode::WeightPaint;
            if let Some(sel_id) = state.selected_node { state.scene.mark_dirty(sel_id); }
            ui.close_menu();
        }
        if ui.selectable_label(state.interaction_mode == InteractionMode::VertexPaint, "Vertex Paint Mode").clicked() {
            state.interaction_mode = InteractionMode::VertexPaint;
            if let Some(sel_id) = state.selected_node { state.scene.mark_dirty(sel_id); }
            ui.close_menu();
        }
        if ui.selectable_label(state.interaction_mode == InteractionMode::Pose, "Pose Mode (Rigging)").clicked() {
            state.interaction_mode = InteractionMode::Pose;
            if let Some(sel_id) = state.selected_node { state.scene.mark_dirty(sel_id); }
            ui.close_menu();
        }
        if ui.selectable_label(state.interaction_mode == InteractionMode::CurveEdit, "Curve Edit Mode").clicked() {
            state.interaction_mode = InteractionMode::CurveEdit;
            if let Some(sel_id) = state.selected_node { state.scene.mark_dirty(sel_id); }
            ui.close_menu();
        }
    });
}

pub fn draw_viewport_menu(state: &mut AppState, actions: &mut UiActions, ui: &mut egui::Ui) {
    ui.menu_button("Viewport", |ui| {
        ui.checkbox(&mut state.show_grid, "Ground Grid");
        ui.checkbox(&mut state.show_wireframe, "Wireframe Overlay");
        ui.checkbox(&mut state.show_subd_cage, "⛓ Subd Control Cage");
        ui.checkbox(&mut state.optimal_display, "✦ Optimal Display (Subd)");
        ui.checkbox(&mut state.show_face_normals, "📐 Show Face Normals");
        ui.checkbox(&mut state.show_vertex_normals, "📍 Show Vertex Normals");
        if state.show_face_normals || state.show_vertex_normals {
            ui.add(egui::Slider::new(&mut state.normal_display_length, 0.05..=1.0).text("Normal Length"));
        }
        ui.checkbox(&mut state.show_viewport_hud, "Heads-Up Statistics HUD");
        ui.separator();
        ui.label(egui::RichText::new("COLOR & TONEMAPPING").small().color(egui::Color32::GRAY));
        ui.add(egui::Slider::new(&mut state.exposure, 0.1..=4.0).text("Exposure"));
        ui.horizontal(|ui| {
            ui.label("Tonemap:");
            ui.selectable_value(&mut state.tonemapping, TonemappingMode::Aces, "ACES");
            ui.selectable_value(&mut state.tonemapping, TonemappingMode::Reinhard, "Reinhard");
            ui.selectable_value(&mut state.tonemapping, TonemappingMode::None, "Raw");
        });
        ui.horizontal(|ui| {
            ui.checkbox(&mut state.show_cavity, "⚡ Cavity & Ridges");
            if state.show_cavity {
                ui.add(egui::Slider::new(&mut state.cavity_intensity, 0.1..=1.5).text("Intensity"));
            }
        });
        ui.separator();
        if ui.checkbox(&mut state.flat_shading, "Flat Shading").changed() {
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
        ui.checkbox(&mut state.show_node_editor, "🌿 Geometry Nodes Canvas");
        ui.checkbox(&mut state.show_script_console, "📜 Scripting Console (Rhai)");
        ui.separator();
        let is_persp = state.camera.projection == CameraProjection::Perspective;
        if ui.selectable_label(is_persp, "Perspective Camera").clicked() {
            state.camera.projection = CameraProjection::Perspective;
            ui.close_menu();
        }
        if ui.selectable_label(!is_persp, "Orthographic Camera").clicked() {
            state.camera.projection = CameraProjection::Orthographic;
            ui.close_menu();
        }
        ui.separator();
        let in_cam_view = state.active_camera_node.is_some();
        if ui.selectable_label(in_cam_view, "Active Camera View (Numpad 0)").clicked() {
            if in_cam_view {
                state.active_camera_node = None;
            } else {
                let cam = state.selected_node
                    .filter(|&id| matches!(state.scene.nodes.get(id).map(|n| &n.data), Some(NodeData::Camera { .. })))
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
            let cam = state.active_camera_node
                .or_else(|| {
                    state.selected_node.filter(|&id| matches!(state.scene.nodes.get(id).map(|n| &n.data), Some(NodeData::Camera { .. })))
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
            state.camera.aspect = state.surface_config.width as f32 / state.surface_config.height as f32;
            ui.close_menu();
        }
    });
}

pub fn draw_shading_menu(state: &mut AppState, ui: &mut egui::Ui) {
    ui.menu_button("Shading", |ui| {
        ui.selectable_value(&mut state.viewport_shading, ViewportShadingMode::Studio, "Studio (Three-Point Lighting)");
        ui.selectable_value(&mut state.viewport_shading, ViewportShadingMode::Pbr, "PBR (Physically Based Cook-Torrance)");
        ui.selectable_value(&mut state.viewport_shading, ViewportShadingMode::MatCapRedWax, "MatCap Red Wax (Sculpt)");
        ui.selectable_value(&mut state.viewport_shading, ViewportShadingMode::MatCapClay, "MatCap Clay (Form)");
        ui.selectable_value(&mut state.viewport_shading, ViewportShadingMode::MatCapChrome, "MatCap Chrome (Reflection)");
        ui.selectable_value(&mut state.viewport_shading, ViewportShadingMode::NormalInspection, "Normal Vector Inspection");
        ui.selectable_value(&mut state.viewport_shading, ViewportShadingMode::Zebra, "🦓 Zebra Surfacing (Class-A Reflection)");
        ui.separator();
        ui.selectable_value(&mut state.viewport_shading, ViewportShadingMode::Rendered, "☀ Rendered (Live Path Tracer)");
    });
}

pub fn draw_uv_menu(state: &mut AppState, actions: &mut UiActions, ui: &mut egui::Ui) {
    ui.menu_button("UV", |ui| {
        if ui.add_enabled(state.selected_node.is_some(), egui::Button::new("🌐 Unwrap (Seams) (U)")).clicked() {
            actions.trigger_unwrap_seams = true;
            ui.close_menu();
        }
        if ui.add_enabled(state.selected_node.is_some(), egui::Button::new("📐 Smart UV Project")).clicked() {
            actions.trigger_smart_uv_project = true;
            ui.close_menu();
        }
        ui.separator();
        let can_seam = state.interaction_mode == InteractionMode::Edit && !state.selected_edges.is_empty();
        if ui.add_enabled(can_seam, egui::Button::new("✂ Mark Seam")).clicked() {
            actions.trigger_mark_seam = true;
            ui.close_menu();
        }
        if ui.add_enabled(can_seam, egui::Button::new("✖ Clear Seam")).clicked() {
            actions.trigger_clear_seam = true;
            ui.close_menu();
        }
        if ui.add_enabled(state.selected_node.is_some(), egui::Button::new("🗑 Clear All Seams")).clicked() {
            actions.trigger_clear_all_seams = true;
            ui.close_menu();
        }
        ui.separator();
        if ui.add_enabled(state.selected_node.is_some(), egui::Button::new("⬛ Planar Projection")).clicked() {
            actions.trigger_planar_uv = true;
            ui.close_menu();
        }
        if ui.add_enabled(state.selected_node.is_some(), egui::Button::new("📦 Box / Cube Projection")).clicked() {
            actions.trigger_box_uv = true;
            ui.close_menu();
        }
        if ui.add_enabled(state.selected_node.is_some(), egui::Button::new("🥫 Cylindrical Projection")).clicked() {
            actions.trigger_cylindrical_uv = true;
            ui.close_menu();
        }
        if ui.add_enabled(state.selected_node.is_some(), egui::Button::new("🔮 Spherical Projection")).clicked() {
            actions.trigger_spherical_uv = true;
            ui.close_menu();
        }
        ui.separator();
        ui.checkbox(&mut state.show_uv_editor, "2D UV Editor Window");
    });
}

pub fn draw_render_menu(state: &mut AppState, ui: &mut egui::Ui) {
    ui.menu_button("Render", |ui| {
        if ui.button("📷 Render Image (F12)").clicked() {
            state.production_render.show_dialog = true;
            crate::ui::render_dialog::start_production_render(state);
            ui.close_menu();
        }
        if ui.button("⚙ Render Settings...").clicked() {
            state.production_render.show_dialog = true;
            ui.close_menu();
        }
        ui.separator();
        if ui.button("🍳 PBR Texture Baking...").clicked() {
            state.bake_state.show_dialog = true;
            ui.close_menu();
        }
        ui.separator();
        ui.label(egui::RichText::new("Engine: Monte Carlo Path Tracer").size(11.0).color(egui::Color32::GRAY));
        ui.label(egui::RichText::new("Pure Safe Rust + Rayon BVH").size(10.0).color(egui::Color32::DARK_GRAY));
    });
}

pub fn draw_window_menu(state: &mut AppState, ui: &mut egui::Ui) {
    ui.menu_button("Window", |ui| {
        if ui.button("🔍 Command Palette (F3)").clicked() {
            state.command_palette.is_open = true;
            state.command_palette.focus_requested = false;
            ui.close_menu();
        }
        if ui.button("⚙ Preferences (Ctrl+,)").clicked() {
            state.preferences.show_dialog = true;
            ui.close_menu();
        }
        ui.separator();
        ui.menu_button("🎨 Themes", |ui| {
            for preset in crate::ui::theme::ThemePreset::ALL {
                let is_cur = state.current_theme == preset;
                if ui.selectable_label(is_cur, preset.name()).clicked() {
                    state.set_theme(preset);
                    ui.close_menu();
                }
            }
        });
        ui.separator();
        ui.checkbox(&mut state.show_outliner, "Scene Outliner");
        ui.checkbox(&mut state.show_inspector, "Properties Inspector");
        ui.checkbox(&mut state.show_left_toolbar, "Left Tool Strip");
        ui.checkbox(&mut state.show_viewport_hud, "Scene Statistics HUD");
        ui.separator();
        ui.checkbox(&mut state.show_node_editor, "🌿 Geometry Nodes Canvas");
        ui.checkbox(&mut state.show_uv_editor, "🗺 2D UV Editor Window");
        ui.checkbox(&mut state.show_script_console, "📜 Scripting Console (Rhai)");
        ui.checkbox(&mut state.production_render.show_dialog, "📷 Production Render Result");
        ui.checkbox(&mut state.bake_state.show_dialog, "🍳 PBR Texture Baker");
        ui.checkbox(&mut state.print_inspector.show_dialog, "🖨 3D Print Quality Inspector");
        ui.checkbox(&mut state.show_help_dialog, "📖 User Manual & Shortcuts (F1)");
    });
}

pub fn draw_help_menu(state: &mut AppState, ui: &mut egui::Ui) {
    ui.menu_button("Help", |ui| {
        for (label, tab) in [
            ("📖 User Manual & Shortcuts (F1)", 0),
            ("🎓 Beginner Tutorials", 1),
            ("🛠 Modeling & Modals Guide", 2),
            ("🌿 Geometry Nodes Guide", 3),
            ("📜 Rhai Scripting API", 4),
            ("📷 Path Tracer Rendering", 5),
            ("🖨 3D Printing & CAD Interchange", 6),
        ] {
            if ui.button(label).clicked() {
                state.show_help_dialog = true;
                state.help_dialog_tab = tab;
                ui.close_menu();
            }
        }
        ui.separator();
        if ui.button("ℹ About Plumb 3D").clicked() {
            state.show_help_dialog = true;
            state.help_dialog_tab = 7;
            ui.close_menu();
        }
    });
}

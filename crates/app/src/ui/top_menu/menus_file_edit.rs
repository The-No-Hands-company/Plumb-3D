use core_math::vec3;
use geo_kernel::*;
use graph_engine::*;

use crate::state::AppState;
use crate::types::*;
use crate::ui::actions::UiActions;

pub fn draw_file_menu(state: &mut AppState, actions: &mut UiActions, ui: &mut egui::Ui) {
    ui.menu_button("File", |ui| {
        if ui.button("Welcome & Starter Hub (F2)").clicked() {
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
        if ui.button("Save Project (Ctrl+S)").clicked() {
            actions.export_plmb = true;
            ui.close_menu();
        }
        if ui.button("Save Project As... (Ctrl+Shift+S)").clicked() {
            actions.export_plmb_as = true;
            ui.close_menu();
        }
        if ui.button("Open Project... (Ctrl+O)").clicked() {
            actions.trigger_open_plmb = true;
            ui.close_menu();
        }
        if ui
            .button("Import 3D Model... (plmb, obj, stl, ply, gltf, glb, step)")
            .clicked()
        {
            actions.trigger_import_dialog = true;
            ui.close_menu();
        }
        ui.separator();
        if ui
            .button("Export Active Mesh as STL for 3D Printing (.stl)")
            .clicked()
        {
            actions.export_stl = true;
            ui.close_menu();
        }
        if ui.button("Export Active Mesh as OBJ (.obj)").clicked() {
            actions.export_obj = true;
            ui.close_menu();
        }
        if ui
            .button("Export Active Mesh as glTF 2.0 (.gltf)")
            .clicked()
        {
            actions.export_gltf = true;
            ui.close_menu();
        }
        if ui
            .button("Export Active Mesh as Binary glTF (.glb)")
            .clicked()
        {
            actions.export_glb = true;
            ui.close_menu();
        }
        if ui.button("Export Active Mesh as PLY (.ply)").clicked() {
            actions.export_ply = true;
            ui.close_menu();
        }
        if ui
            .button("Export Active Mesh as CAD STEP Solid (.step)")
            .clicked()
        {
            actions.export_step = true;
            ui.close_menu();
        }
        if ui
            .button("Export Active Mesh as CAD IGES Solid (.igs, .iges)")
            .clicked()
        {
            actions.export_iges = true;
            ui.close_menu();
        }
        if ui
            .button("Export SubD Limit Surface as Continuous NURBS CAD STEP (.step)")
            .clicked()
        {
            actions.export_subd_nurbs_step = true;
            ui.close_menu();
        }
        if ui
            .button("Export Full Scene as Pixar OpenUSD (.usda)")
            .clicked()
        {
            actions.export_usd = true;
            ui.close_menu();
        }
        if ui.button("Import Pixar OpenUSD Scene (.usda)...").clicked() {
            actions.import_usd = true;
            ui.close_menu();
        }
        if ui
            .button("Export Active Mesh as Autodesk FBX (.fbx)")
            .clicked()
        {
            actions.export_fbx = true;
            ui.close_menu();
        }
        if ui.button("Import Autodesk FBX (.fbx)...").clicked() {
            actions.import_fbx = true;
            ui.close_menu();
        }
        if ui
            .button("Export Active Mesh as Alembic Cache (.abc)")
            .clicked()
        {
            actions.export_alembic = true;
            ui.close_menu();
        }
        if ui
            .button("Export Sliced G-Code for 3D Printing (.gcode)")
            .clicked()
        {
            actions.export_gcode = true;
            ui.close_menu();
        }
        if ui.button("3D Print & CAD Quality Inspector...").clicked() {
            state.print_inspector.show_dialog = true;
            ui.close_menu();
        }
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
        if ui
            .add_enabled(state.history.can_undo(), egui::Button::new(undo_text))
            .clicked()
        {
            actions.trigger_undo = true;
            ui.close_menu();
        }

        let redo_text = if let Some(name) = state.history.redo_name() {
            format!("Redo {} (Ctrl+Y)", name)
        } else {
            "Redo (Ctrl+Y)".into()
        };
        if ui
            .add_enabled(state.history.can_redo(), egui::Button::new(redo_text))
            .clicked()
        {
            actions.trigger_redo = true;
            ui.close_menu();
        }
        if ui.button("Undo History... (Ctrl+Alt+Z)").clicked() {
            state.show_history_window = true;
            ui.close_menu();
        }
        if ui
            .add_enabled(
                state.selected_node.is_some(),
                egui::Button::new("Duplicate Object (Shift+D)"),
            )
            .clicked()
        {
            actions.trigger_duplicate_node = true;
            ui.close_menu();
        }
        ui.separator();
        let can_join = state.selected_node.is_some()
            && state.previous_selected_node.is_some()
            && state.selected_node != state.previous_selected_node;
        if ui
            .add_enabled(can_join, egui::Button::new("Join Meshes (Ctrl+J)"))
            .clicked()
        {
            actions.trigger_join_meshes = true;
            ui.close_menu();
        }
        let can_separate =
            state.interaction_mode == InteractionMode::Edit && !state.selected_faces.is_empty();
        if ui
            .add_enabled(can_separate, egui::Button::new("Separate Faces (P)"))
            .clicked()
        {
            actions.trigger_separate_faces = true;
            ui.close_menu();
        }
        if ui
            .add_enabled(
                state.selected_node.is_some(),
                egui::Button::new("Separate by Material"),
            )
            .clicked()
        {
            actions.trigger_separate_by_material = true;
            ui.close_menu();
        }
        if ui
            .add_enabled(
                state.interaction_mode == InteractionMode::Edit,
                egui::Button::new("Inset Faces (I)"),
            )
            .clicked()
        {
            actions.trigger_inset = true;
            ui.close_menu();
        }
        if ui
            .add_enabled(
                state.interaction_mode == InteractionMode::Edit,
                egui::Button::new("Flip Normals (Alt+N)"),
            )
            .clicked()
        {
            actions.trigger_flip_normals = true;
            ui.close_menu();
        }
        if ui
            .add_enabled(
                state.selected_node.is_some(),
                egui::Button::new("Bisect / Slice Plane"),
            )
            .clicked()
        {
            actions.trigger_bisect = true;
            ui.close_menu();
        }
        if ui
            .add_enabled(
                state.interaction_mode == InteractionMode::Edit,
                egui::Button::new("Triangulate Faces (Ctrl+T)"),
            )
            .clicked()
        {
            actions.trigger_triangulate = true;
            ui.close_menu();
        }
        if ui
            .add_enabled(
                state.interaction_mode == InteractionMode::Edit,
                egui::Button::new("Symmetrize Mesh"),
            )
            .clicked()
        {
            actions.trigger_symmetrize = true;
            ui.close_menu();
        }
        if ui
            .add_enabled(
                state.interaction_mode == InteractionMode::Edit,
                egui::Button::new("Spin Revolve"),
            )
            .clicked()
        {
            actions.trigger_spin = true;
            ui.close_menu();
        }
        if ui
            .add_enabled(
                state.selected_node.is_some(),
                egui::Button::new("Clean Degenerate Geometry"),
            )
            .clicked()
        {
            actions.trigger_clean_degenerate = true;
            ui.close_menu();
        }
        if ui
            .add_enabled(
                state.interaction_mode == InteractionMode::Edit,
                egui::Button::new("Cap Holes / Fill Boundaries"),
            )
            .clicked()
        {
            actions.trigger_cap_holes = true;
            ui.close_menu();
        }
        if ui
            .add_enabled(
                state.interaction_mode == InteractionMode::Edit,
                egui::Button::new("Grid Fill"),
            )
            .clicked()
        {
            actions.trigger_grid_fill = true;
            ui.close_menu();
        }
        if ui
            .add_enabled(
                state.interaction_mode == InteractionMode::Edit
                    && state.selected_vertices.len() >= 2,
                egui::Button::new("Pick Shortest Path"),
            )
            .clicked()
        {
            actions.trigger_shortest_path = true;
            ui.close_menu();
        }
        if ui
            .add_enabled(
                state.interaction_mode == InteractionMode::Edit && !state.selected_edges.is_empty(),
                egui::Button::new("Extract Curve from Edges"),
            )
            .clicked()
        {
            actions.trigger_extract_curve = true;
            ui.close_menu();
        }
        if ui
            .add_enabled(
                state.interaction_mode == InteractionMode::Edit
                    && !state.selected_vertices.is_empty(),
                egui::Button::new("Shrink / Fatten (Alt+S)"),
            )
            .clicked()
        {
            actions.trigger_shrink_fatten = true;
            ui.close_menu();
        }
        if ui
            .add_enabled(
                state.interaction_mode == InteractionMode::Edit
                    && !state.selected_vertices.is_empty(),
                egui::Button::new("Relax / Smooth Vertices"),
            )
            .clicked()
        {
            actions.trigger_relax_vertices = true;
            ui.close_menu();
        }
        if ui
            .add_enabled(
                state.interaction_mode == InteractionMode::Edit
                    && !state.selected_vertices.is_empty(),
                egui::Button::new("Rip Vertices (V)"),
            )
            .clicked()
        {
            actions.trigger_rip_vertices = true;
            ui.close_menu();
        }
        if ui
            .add_enabled(
                state.interaction_mode == InteractionMode::Edit,
                egui::Button::new("Knife Multi-Cut (K)"),
            )
            .clicked()
        {
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
        ui.menu_button("Mesh Primitives", |ui| {
            if ui.button("Box / Cube").clicked() {
                actions.spawn_mesh = Some(("Cube", Primitives::create_cube(2.0)));
                ui.close_menu();
            }
            if ui.button("Plane Grid").clicked() {
                actions.spawn_mesh = Some(("Plane", Primitives::create_plane(4.0, 4.0, 4, 4)));
                ui.close_menu();
            }
            if ui.button("UV Sphere").clicked() {
                actions.spawn_mesh = Some(("Sphere", Primitives::create_uv_sphere(1.2, 16, 16)));
                ui.close_menu();
            }
            if ui.button("Cylinder").clicked() {
                actions.spawn_mesh = Some(("Cylinder", Primitives::create_cylinder(1.0, 2.5, 16)));
                ui.close_menu();
            }
            if ui.button("Torus / Donut").clicked() {
                actions.spawn_mesh = Some(("Torus", Primitives::create_torus(1.5, 0.4, 24, 16)));
                ui.close_menu();
            }
            if ui.button("Cone").clicked() {
                actions.spawn_mesh = Some(("Cone", Primitives::create_cone(1.2, 2.5, 16)));
                ui.close_menu();
            }
            if ui.button("Pyramid").clicked() {
                actions.spawn_mesh = Some(("Pyramid", Primitives::create_pyramid(2.0, 2.5)));
                ui.close_menu();
            }
            if ui.button("Capsule / Pill").clicked() {
                actions.spawn_mesh = Some(("Capsule", Primitives::create_capsule(0.8, 1.8, 16, 6)));
                ui.close_menu();
            }
            if ui.button("IcoSphere (Geodesic)").clicked() {
                actions.spawn_mesh = Some(("IcoSphere", Primitives::create_icosphere(1.2, 2)));
                ui.close_menu();
            }
            if ui.button("Geodesic Dome").clicked() {
                actions.spawn_mesh =
                    Some(("GeodesicDome", Primitives::create_geodesic_dome(2.0, 2)));
                ui.close_menu();
            }
            if ui.button("Pipe / Screw Thread").clicked() {
                actions.spawn_mesh = Some((
                    "PipeThread",
                    Primitives::create_pipe_thread(1.2, 0.3, 2.0, 16),
                ));
                ui.close_menu();
            }
            if ui.button("Plumb Bob (Mascot)").clicked() {
                actions.spawn_mesh = Some(("PlumbBob", Primitives::create_plumb_bob()));
                ui.close_menu();
            }
        });
        ui.menu_button("Bio-Vegetation (L-System)", |ui| {
            if ui.button("Oak Tree").clicked() {
                if let Ok(m) = LSystem::from_preset(LSystemPreset::OakTree).generate_mesh() {
                    actions.spawn_mesh = Some(("OakTree", m));
                }
                ui.close_menu();
            }
            if ui.button("Fern Plant").clicked() {
                if let Ok(m) = LSystem::from_preset(LSystemPreset::FernPlant).generate_mesh() {
                    actions.spawn_mesh = Some(("FernPlant", m));
                }
                ui.close_menu();
            }
            if ui.button("Ivy Vines").clicked() {
                if let Ok(m) = LSystem::from_preset(LSystemPreset::IvyVines).generate_mesh() {
                    actions.spawn_mesh = Some(("IvyVines", m));
                }
                ui.close_menu();
            }
            if ui.button("Biomechanical Conduit").clicked() {
                if let Ok(m) =
                    LSystem::from_preset(LSystemPreset::SciFiBiomechanicalConduit).generate_mesh()
                {
                    actions.spawn_mesh = Some(("BioConduit", m));
                }
                ui.close_menu();
            }
        });
        ui.menu_button("Curves & Splines", |ui| {
            if ui.button("Bezier Path").clicked() {
                actions.spawn_curve = Some(0);
                ui.close_menu();
            }
            if ui.button("Bezier Circle").clicked() {
                actions.spawn_curve = Some(1);
                ui.close_menu();
            }
            if ui.button("Helical Spiral").clicked() {
                actions.spawn_curve = Some(2);
                ui.close_menu();
            }
            ui.separator();
            if ui.button("Lathe Vase (Spline)").clicked() {
                let profile = vec![
                    (0.0, -1.2),
                    (0.7, -1.2),
                    (0.15, -0.3),
                    (0.2, 0.6),
                    (0.8, 1.3),
                    (0.0, 1.3),
                ];
                if let Ok(m) = Spline::lathe(&profile, 24, 2.0 * std::f32::consts::PI) {
                    actions.spawn_mesh = Some(("LatheVase", m));
                }
                ui.close_menu();
            }
            if ui.button("Curved Pipe (Spline)").clicked() {
                let circle = Spline::circle_cross_section(0.25, 12);
                let mut spline = Spline::new();
                spline.add_point(BezierPoint::smooth(
                    vec3(-2.0, 0.0, -1.0),
                    vec3(0.0, 1.5, 0.0),
                ));
                spline.add_point(BezierPoint::smooth(
                    vec3(0.0, 2.0, 0.0),
                    vec3(1.5, 0.0, 0.0),
                ));
                spline.add_point(BezierPoint::smooth(
                    vec3(2.0, 0.5, 1.0),
                    vec3(0.0, -1.0, 0.0),
                ));
                let path = spline.sample(12);
                if let Ok(m) = Spline::sweep(&circle, &path, false) {
                    actions.spawn_mesh = Some(("CurvedPipe", m));
                }
                ui.close_menu();
            }
            ui.separator();
            if ui.button("Loft Selected Curves (Ctrl+L)").clicked() {
                actions.trigger_loft_curves = true;
                ui.close_menu();
            }
            if ui.button("Fillet Curve Corners (0.2m)").clicked() {
                actions.trigger_fillet_curve = Some(0.2);
                ui.close_menu();
            }
            if ui.button("Revolve / Lathe Curve (360 deg)").clicked() {
                actions.trigger_lathe_curve = Some((16, 360.0));
                ui.close_menu();
            }
        });
        ui.menu_button("OpenVDB Volumetrics", |ui| {
            if ui.button("Cumulus Cloud (Level-Set Mesh)").clicked() {
                let cloud = geo_kernel::VolumeGenerators::create_cumulus_cloud(
                    core_math::Vec3::ZERO,
                    2.0,
                    1.0,
                    0.4,
                );
                let mesh = cloud.extract_isosurface(0.35);
                actions.spawn_mesh = Some(("VdbCumulusCloud", mesh));
                ui.close_menu();
            }
            if ui
                .button("Atmospheric Ground Fog (Level-Set Mesh)")
                .clicked()
            {
                let fog = geo_kernel::VolumeGenerators::create_ground_fog(4.0, 1.5, 0.8, 0.4);
                let mesh = fog.extract_isosurface(0.2);
                actions.spawn_mesh = Some(("VdbGroundFog", mesh));
                ui.close_menu();
            }
        });
        ui.menu_button("Lights", |ui| {
            if ui.button("Directional Light (Sun)").clicked() {
                actions.spawn_light = Some(("SunLight", LightType::Directional));
                ui.close_menu();
            }
            if ui.button("Point Light (Omni)").clicked() {
                actions.spawn_light = Some(("PointLight", LightType::Point));
                ui.close_menu();
            }
            if ui.button("Spot Light (Cone)").clicked() {
                actions.spawn_light = Some(("SpotLight", LightType::Spot));
                ui.close_menu();
            }
        });
        if ui.button("Camera").clicked() {
            actions.spawn_camera = true;
            ui.close_menu();
        }
        if ui.button("Joint / Bone Armature").clicked() {
            actions.spawn_joint = true;
            ui.close_menu();
        }
    });
}


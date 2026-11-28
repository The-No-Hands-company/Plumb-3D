use crate::state::AppState;

#[allow(clippy::type_complexity)]
pub fn draw_tab_tutorials(ui: &mut egui::Ui, state: &mut AppState, query: &str) {
    ui.heading("Beginner 3D Modeling Tutorials");
    ui.label("Hands-on, step-by-step walkthroughs to teach you how to build various styles of 3D models from scratch.");
    ui.add_space(8.0);

    let tutorials: &[(&str, &str, &str, &[(&str, &str)])] = &[
        (
            "1. Sci-Fi Storage Crate (Box Modeling)",
            "Hard-Surface / Game Prop",
            "Learn core polygonal box modeling: Inset, Extrude, Bevel, and Modifier-based edge chamfering.",
            &[
                ("1. Base Primitive", "Start with the Default Cube (or add one via Add -> Primitive -> Cube). Ensure you are in Object Mode, then press Tab to enter Edit Mode."),
                ("2. Face Selection Level", "Press '3' on your keyboard to switch to Face Selection level. Click Left-Mouse (LMB) on the top face of the cube to select it."),
                ("3. Inset Top & Side Panels", "Press 'I' to activate Inset. Drag the mouse inward slightly to create an outer border frame (approx 0.15m). Left-click or press Enter to confirm."),
                ("4. Extrude Recessed Inset", "With the newly inset face still selected, press 'E' to Extrude. Move your mouse down slightly to recess the panel inwards (-0.2m), then Left-click to confirm."),
                ("5. Corner Reinforcements", "Repeat the Inset ('I') and negative Extrude ('E') on the 4 side faces of the cube to create a reinforced outer cage look."),
                ("6. Smooth Edge Chamfers", "In the Inspector on the right under 'Modifier Stack', click '+ Bevel'. Set Width to 0.05m and Segments to 2. Your cube now has pristine beveled industrial edges!"),
                ("7. Materials", "Under 'Material Slots & PBR Shading', set Base Color to Dark Metallic Slate, Metallic to 0.85, and Roughness to 0.25."),
            ],
        ),
        (
            "2. Medieval Castle Tower (Architectural)",
            "Environment & Architecture",
            "Construct a castle battlement tower using Cylinder primitives, double Inset, and Walkway Ring Extrusion.",
            &[
                ("1. Add Cylinder", "Clear the scene (File -> New Scene). Click 'Add Primitive -> Cylinder' in the top ribbon. In Inspector, set Radius to 2.0 and Height to 6.0."),
                ("2. Select Top Cap", "Press Tab to enter Edit Mode. Press '3' for Face mode. Click the top circular cap of the cylinder."),
                ("3. Outer Parapet Wall Inset", "Press 'I' and type '0.3' (or drag inward) to set the outer stone wall parapet thickness (0.3m). Press Enter or click Confirm."),
                ("4. Walkway Inner Boundary Inset", "With the newly inset central face automatically selected, press 'I' again and type '0.8' (or drag inward) to define the walkway width (0.8m) and central tower roof. Confirm."),
                ("5. Extrude Walkway Trench Down", "Hold 'Alt' and click on the circular walkway ring between the two insets (or Shift-click the ring faces) to select the walkway loop. Press 'E' and type '-0.8' to sink the flat walkway trench down between the outer parapet and inner core!"),
                ("6. Battlements (Crenellations)", "Select alternating faces around the top outer rim wall. Press 'E' and type '0.5' to extrude defensive crenel blocks (merlons) upward around the perimeter!"),
                ("7. Stone Material", "In Inspector, add a new Material Slot. Set Base Color to warm granite grey, Roughness to 0.85, and enable Cavity Shading in the Viewport menu for pronounced masonry depth."),
            ],
        ),
        (
            "3. Creature / Character Bust (Organic Sculpting)",
            "Organic Character Art",
            "Sculpt a dynamic creature head using the pure-Rust digital sculpting engine and brush falloffs.",
            &[
                ("1. Base Sphere", "Add a UV Sphere (Add Primitive -> Sphere) with 32 segments and 16 rings. In Inspector, add a Subdivision modifier (level 2) to supply dense sculptable vertices."),
                ("2. Enter Sculpt Mode", "In the top ribbon header, click the 'Sculpt' mode pill (or switch via Mode dropdown)."),
                ("3. Overall Silhouette (Grab Brush)", "Select the 'Grab' brush from the left tool strip or RMB menu. Increase Radius slider to ~1.2m. Click and drag the chin down and the cranium back to establish the skull silhouette."),
                ("4. Facial Features (Clay & Draw)", "Switch to 'Clay' or 'Draw' brush. Decrease radius to 0.3m. Stroke outward along the brow line, cheekbones, and snout to build up anatomical clay mass."),
                ("5. Eye Sockets & Horns", "Use 'Inflate' (or hold Shift to invert) to carve deep orbital eye sockets. Use 'SnakeHook' brush to pull organic horns and tentacles straight out of the skull!"),
                ("6. Surface Tension (Smooth Brush)", "Switch to 'Smooth' brush (or hold Shift while using any brush) and stroke across rough facets to polish organic skin curvature."),
            ],
        ),
        (
            "4. Industrial Pipe & Wiring Harness (Curves)",
            "Precision Industrial Design",
            "Create smooth, sweepable 3D pipes and hydraulic lines using Bezier spline curves.",
            &[
                ("1. Add Spline Curve", "Click 'Add Curve -> 3D Spline Curve'. A parametric Bezier curve appears in the viewport."),
                ("2. Curve Edit Mode", "In the top ribbon, click 'Curve' mode. The spline's control knots and tangent handles become visible."),
                ("3. Shape the Curve", "Select knot points and move them with the 3D Gizmo ('W'). Select the tangent handle tips to rotate and curve the path in 3D space."),
                ("4. Extrude Knots ('E')", "Select the last knot and press 'E' to extrude new sequential knots following your pipe path."),
                ("5. Turn Curve to 3D Pipe Mesh", "In the Inspector under 'Curve Properties', set 'Bevel Radius' to 0.15m and 'Bevel Resolution' to 12. The engine instantly sweeps a watertight pipe mesh along the path!"),
                ("6. Profile Shapes", "Change 'Profile Shape' from Circle to Square, Star, or Hexagon for specialized conduits or architectural moldings."),
            ],
        ),
        (
            "5. Procedural City / Sci-Fi Blocks (Geo Nodes)",
            "Procedural Generation",
            "Generate complex procedural geometry networks using the visual node canvas.",
            &[
                ("1. Open Geo Nodes Canvas", "Click 'Nodes' in the top bar or tool strip. An infinite node editor canvas opens."),
                ("2. Add Nodes", "Right-click the canvas or click '+ Add Node'. Add: 'Grid Primitive', 'Noise Displace', and 'Group Output'."),
                ("3. Wire the Pipeline", "Drag from the green Mesh output socket of the Grid node to the input socket of the Noise Displace node. Connect the output of Noise Displace to the Group Output node."),
                ("4. Procedural Modifiers", "In the 3D Inspector on your object, click 'Modifiers -> + Geo Nodes'. The procedural node graph is now live and non-destructively displacing your object in real time!"),
            ],
        ),
        (
            "6. Studio Lighting & Bokeh Render (Path Tracer)",
            "Production Rendering",
            "Set up 3-point photographic lighting and render with physical optical depth of field.",
            &[
                ("1. Position Subject", "Frame your modeled asset in the center of the viewport. Press 'F' to focus camera."),
                ("2. 3-Point Studio Lights", "Go to 'Add -> Light': Add a Sun light from top-left (Key), a warm Point light on the right (Fill), and a cool Spot light behind the subject (Rim / Hair light)."),
                ("3. Optical Bokeh Camera", "Select the scene Camera in the Outliner. In Inspector, set 'Aperture Radius' to 0.08m and set 'Focal Distance' to the distance to your model to produce authentic optical bokeh."),
                ("4. Render (F12)", "Press 'F12' or select 'Render -> Render Image'. Choose '1080p Full HD' and 128 SPP samples."),
                ("5. Save PNG", "Watch the progressive Monte Carlo ray bounces accumulate clean indirect lighting. Click 'Save Image (PNG)' to export to disk."),
            ],
        ),
        (
            "7. Watertight Geometric Vase for 3D Printing",
            "3D Printing & CAD",
            "Model a hollow, manifold geometric vase and export clean STL for 3D slicers.",
            &[
                ("1. Cylinder Base", "Add a Cylinder with 8 segments (octagon base). Press Tab to enter Edit Mode."),
                ("2. Profile Extrusions", "Select the top face. Press 'E' to extrude upward, 'R' to scale outward, 'E' again, and 'R' to taper inward, forming an elegant vase profile."),
                ("3. Delete Top Cap", "Select only the top face and press Delete (or 'X' -> Delete Faces) to open the top of the vase."),
                ("4. Solidify Modifier", "In Inspector, add the 'Solidify' modifier. Set Thickness to 0.2m. The engine generates uniform inner and outer walls, guaranteeing a 2-manifold watertight mesh."),
                ("5. Export for Slicing", "Click 'File -> Export Active Mesh as STL for 3D Printing (.stl)'. The exported binary STL file is ready to drop directly into Cura, PrusaSlicer, or Bambu Studio!"),
            ],
        ),
        (
            "8. Lunar Astrophotography & Night Sky",
            "Celestial & Astrophotography",
            "Recreate a photorealistic lunar night sky from scratch: 128x128 UV Sphere, NASA 2K cartography maps, Oren-Nayar regolith BRDF, directional sunlight & terminator contrast, 360 ESO photographic Milky Way skybox, and telephoto framing.",
            &[
                ("1. Spawn UV Sphere", "Press 'Shift+A' (or Add -> Mesh -> UV Sphere). In the operator panel or Inspector, set Radius to 3.0m, Segments to 128, and Rings to 128 for a sub-pixel smooth silhouette."),
                ("2. Lunar Regolith PBR Material", "Select the sphere and switch to Inspector 'Shading' tab. Create a material with Metallic 0.0 (dielectric silicate) and Roughness 0.88 (micro-facet Oren-Nayar retro-reflective regolith). Load NASA 2K Albedo and Normal relief maps."),
                ("3. Solar Light & The Terminator", "Add a Directional Sun light (Add -> Light -> Sun Light). Rotate it (-0.2 rad X, 0.65 rad Y) to cast solar illumination across the moon's limb, creating the dramatic lunar terminator where crater shadows emerge."),
                ("4. Authentic 360 Photographic Skybox", "Bind the genuine European Southern Observatory (ESO) 4000x2000 Milky Way 360 equirectangular skybox (eso_milky_way_360.jpg) to surround deep space with real photographic star clusters."),
                ("5. Telephoto Astrophotography Camera", "Add a Camera at (0, 0, 8.5m) with a narrow 40° FOV (telephoto compression). This frames the lunar disc majestically centered against the galactic starfield."),
                ("6. Production Render (F11 / F12)", "Press 'F11' for an instant Direct PBR Snapshot, or 'F12' for progressive Monte Carlo path tracing with ACES Filmic tonemapping, physical regolith retro-reflection, and optical bloom."),
            ],
        ),
    ];

    ui.horizontal_wrapped(|ui| {
        ui.label(egui::RichText::new("Choose Tutorial:").strong());
        for (idx, (title, _, _, _)) in tutorials.iter().enumerate() {
            let short_title = title.split('.').next().unwrap_or(title);
            if ui
                .selectable_label(state.selected_tutorial == idx, short_title)
                .clicked()
            {
                state.selected_tutorial = idx;
            }
        }
    });
    ui.separator();

    let (title, category, summary, steps) =
        tutorials[state.selected_tutorial.min(tutorials.len() - 1)];

    if !query.is_empty()
        && !title.to_lowercase().contains(query)
        && !summary.to_lowercase().contains(query)
        && !steps.iter().any(|(s_title, s_desc)| {
            s_title.to_lowercase().contains(query) || s_desc.to_lowercase().contains(query)
        })
    {
        ui.label(
            egui::RichText::new(
                "No tutorial matches your search query. Clear search to view selected tutorial.",
            )
            .italics()
            .color(egui::Color32::GRAY),
        );
        return;
    }

    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(title)
                .size(18.0)
                .strong()
                .color(egui::Color32::from_rgb(100, 200, 255)),
        );
        ui.label(
            egui::RichText::new(format!("[ {} ]", category))
                .size(11.0)
                .color(egui::Color32::from_rgb(255, 200, 100)),
        );
    });
    ui.label(egui::RichText::new(summary).italics());
    ui.add_space(8.0);

    for (step_title, step_desc) in steps.iter() {
        ui.group(|ui| {
            ui.label(
                egui::RichText::new(*step_title)
                    .strong()
                    .color(egui::Color32::from_rgb(130, 220, 160)),
            );
            ui.label(*step_desc);
        });
        ui.add_space(4.0);
    }

    if state.selected_tutorial == 0 {
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            if ui
                .button(
                    egui::RichText::new("🚀 Start Guided Tutorial (From Scratch)")
                        .strong()
                        .color(egui::Color32::from_rgb(226, 179, 87)),
                )
                .clicked()
            {
                crate::scenes::build_scifi_crate_tutorial_blank(state);
                state.show_help_dialog = false;
            }
            if ui
                .button(
                    egui::RichText::new("📦 Load Completed Crate Scene")
                        .color(egui::Color32::from_rgb(130, 200, 255)),
                )
                .clicked()
            {
                let (scene, camera, id) = crate::scenes::build_scifi_crate_finished_graph();
                state.scene = scene;
                state.camera = camera;
                state.selected_node = Some(id);
                state.interaction_mode = crate::types::InteractionMode::Object;
                state.viewport_shading = render_viewport::ViewportShadingMode::Studio;
                state.show_wireframe = true;
                state.sync_gpu_meshes();
                state.show_help_dialog = false;
            }
        });
    }

    if state.selected_tutorial == 1 {
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            if ui
                .button(
                    egui::RichText::new("🚀 Start Guided Tutorial (From Scratch)")
                        .strong()
                        .color(egui::Color32::from_rgb(226, 179, 87)),
                )
                .clicked()
            {
                crate::scenes::build_castle_tower_tutorial_blank(state);
                state.show_help_dialog = false;
            }
            if ui
                .button(
                    egui::RichText::new("🏰 Load Completed Citadel Scene")
                        .color(egui::Color32::from_rgb(130, 200, 255)),
                )
                .clicked()
            {
                crate::scenes::build_medieval_citadel_scene(state);
                state.show_help_dialog = false;
            }
        });
    }

    if state.selected_tutorial == 7 {
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            if ui
                .button(
                    egui::RichText::new("🚀 Start Guided Tutorial (From Scratch)")
                        .strong()
                        .color(egui::Color32::from_rgb(226, 179, 87)),
                )
                .clicked()
            {
                crate::scenes::build_lunar_night_sky_tutorial_blank(state);
                state.show_help_dialog = false;
            }
            if ui
                .button(
                    egui::RichText::new("🌕 Load Completed Showcase Scene")
                        .color(egui::Color32::from_rgb(130, 200, 255)),
                )
                .clicked()
            {
                crate::scenes::build_lunar_night_sky_scene(state);
                state.show_help_dialog = false;
            }
        });
    }
}

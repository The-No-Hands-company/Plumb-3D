use crate::state::AppState;
use crate::types::*;
use crate::ui::actions::UiActions;
use crate::ui::theme::ThemePreset;
use crate::ui::workspaces::WorkspaceTemplate;
use render_viewport::ViewportShadingMode;

#[derive(Clone, Copy)]
pub enum PaletteAction {
    TriggerAction(fn(&mut AppState, &mut UiActions)),
    SetWorkspace(WorkspaceTemplate),
    SetTheme(ThemePreset),
    SetInteractionMode(InteractionMode),
    SetShading(ViewportShadingMode),
    ToggleWireframe,
    ToggleGrid,
    ToggleMeasure,
    ToggleSnap,
    OpenWelcomeHub,
    OpenPreferences,
    OpenHelp(usize),
}

pub struct PaletteItem {
    pub name: &'static str,
    pub category: &'static str,
    pub shortcut: &'static str,
    pub keywords: &'static [&'static str],
    pub action: PaletteAction,
}

pub const COMMAND_LIST: &[PaletteItem] = &[
    // --- Direct Modeling Operators ---
    PaletteItem {
        name: "Extrude Region / Faces",
        category: "Modeling",
        shortcut: "E",
        keywords: &["extrude", "push", "pull", "expand", "faces"],
        action: PaletteAction::TriggerAction(|state, actions| {
            state.interaction_mode = InteractionMode::Edit;
            actions.trigger_extrude = true;
        }),
    },
    PaletteItem {
        name: "Bevel Edges / Vertices",
        category: "Modeling",
        shortcut: "Ctrl+B",
        keywords: &[
            "bevel",
            "chamfer",
            "round",
            "fillet",
            "smooth corners",
            "edges",
        ],
        action: PaletteAction::TriggerAction(|state, actions| {
            state.interaction_mode = InteractionMode::Edit;
            actions.trigger_bevel = true;
        }),
    },
    PaletteItem {
        name: "Inset Faces",
        category: "Modeling",
        shortcut: "I",
        keywords: &["inset", "inner face", "offset", "border"],
        action: PaletteAction::TriggerAction(|state, actions| {
            state.interaction_mode = InteractionMode::Edit;
            actions.trigger_inset = true;
        }),
    },
    PaletteItem {
        name: "Loop Cut and Slide",
        category: "Modeling",
        shortcut: "Ctrl+R",
        keywords: &["loop cut", "ring cut", "insert edge", "split loop", "slice"],
        action: PaletteAction::TriggerAction(|state, actions| {
            state.interaction_mode = InteractionMode::Edit;
            actions.trigger_loop_cut = true;
        }),
    },
    PaletteItem {
        name: "Knife Topology Cut",
        category: "Modeling",
        shortcut: "K",
        keywords: &["knife", "cut", "slice", "polygon cut", "multi-cut"],
        action: PaletteAction::TriggerAction(|state, _| {
            state.interaction_mode = InteractionMode::Edit;
            state.knife_active = true;
        }),
    },
    PaletteItem {
        name: "Bridge Edge Loops",
        category: "Modeling",
        shortcut: "Alt+B / Edge Menu",
        keywords: &[
            "bridge",
            "edge loops",
            "connect loops",
            "loft quads",
            "bridge rings",
        ],
        action: PaletteAction::TriggerAction(|state, actions| {
            state.interaction_mode = InteractionMode::Edit;
            actions.trigger_bridge_edge_loops = true;
        }),
    },
    PaletteItem {
        name: "Subdivide Mesh / Faces",
        category: "Modeling",
        shortcut: "W -> Subd",
        keywords: &[
            "subdivide",
            "subdiv",
            "smooth",
            "increase density",
            "resolution",
        ],
        action: PaletteAction::TriggerAction(|state, actions| {
            state.interaction_mode = InteractionMode::Edit;
            actions.add_subdivide_to_selected = true;
        }),
    },
    PaletteItem {
        name: "Merge Vertices at Center",
        category: "Modeling",
        shortcut: "M",
        keywords: &["merge", "weld", "collapse", "center", "join vertices"],
        action: PaletteAction::TriggerAction(|state, actions| {
            state.interaction_mode = InteractionMode::Edit;
            actions.trigger_merge_vertices = true;
        }),
    },
    PaletteItem {
        name: "Relax / Smooth Vertices (Laplacian)",
        category: "Modeling",
        shortcut: "Alt+R",
        keywords: &[
            "relax",
            "smooth",
            "laplacian",
            "straighten",
            "distribute evenly",
        ],
        action: PaletteAction::TriggerAction(|state, actions| {
            state.interaction_mode = InteractionMode::Edit;
            actions.trigger_relax_vertices = true;
        }),
    },
    PaletteItem {
        name: "Spin / Revolve Geometry 360 deg",
        category: "Modeling",
        shortcut: "Alt+S",
        keywords: &["spin", "revolve", "lathe", "turn", "radial extrude"],
        action: PaletteAction::TriggerAction(|state, actions| {
            state.interaction_mode = InteractionMode::Edit;
            actions.trigger_spin = true;
        }),
    },
    PaletteItem {
        name: "Bisect Mesh Along Plane",
        category: "Modeling",
        shortcut: "Shift+B",
        keywords: &["bisect", "slice", "plane cut", "half", "cross section"],
        action: PaletteAction::TriggerAction(|state, actions| {
            state.interaction_mode = InteractionMode::Edit;
            actions.trigger_bisect = true;
        }),
    },
    PaletteItem {
        name: "Connect Selected Vertices",
        category: "Modeling",
        shortcut: "J",
        keywords: &["connect", "join vertices", "split face", "cut path"],
        action: PaletteAction::TriggerAction(|state, actions| {
            state.interaction_mode = InteractionMode::Edit;
            actions.trigger_connect_vertices = true;
        }),
    },
    PaletteItem {
        name: "Selection Mode: Marquee Box Select",
        category: "Select",
        shortcut: "B",
        keywords: &["select", "box", "marquee", "rectangle", "window"],
        action: PaletteAction::TriggerAction(|state, _| {
            state.selection_tool_mode = SelectionToolMode::Box;
            state.is_box_selecting = true;
            state.set_status_message("Box Select Mode Activated");
        }),
    },
    PaletteItem {
        name: "Selection Mode: Freehand Lasso Select",
        category: "Select",
        shortcut: "Alt+B",
        keywords: &["select", "lasso", "freehand", "polygon", "outline"],
        action: PaletteAction::TriggerAction(|state, _| {
            state.selection_tool_mode = SelectionToolMode::Lasso;
            state.is_box_selecting = true;
            state.set_status_message("Lasso Select Mode Activated (Drag to outline)");
        }),
    },
    PaletteItem {
        name: "Selection Mode: Circle Paint Select",
        category: "Select",
        shortcut: "Shift+C",
        keywords: &["select", "circle", "paint", "brush", "drag"],
        action: PaletteAction::TriggerAction(|state, _| {
            state.selection_tool_mode = SelectionToolMode::CirclePaint;
            state.is_box_selecting = true;
            state.set_status_message("Circle Paint Select Activated (Drag to paint)");
        }),
    },
    PaletteItem {
        name: "Grid Fill Hole",
        category: "Modeling",
        shortcut: "Alt+F",
        keywords: &["grid fill", "fill hole", "cap loop", "quad fill"],
        action: PaletteAction::TriggerAction(|state, actions| {
            state.interaction_mode = InteractionMode::Edit;
            actions.trigger_grid_fill = true;
        }),
    },
    PaletteItem {
        name: "Symmetrize Mesh (+X to -X)",
        category: "Modeling",
        shortcut: "Alt+M",
        keywords: &["symmetrize", "mirror mesh", "symmetry", "half copy"],
        action: PaletteAction::TriggerAction(|state, actions| {
            state.interaction_mode = InteractionMode::Edit;
            actions.trigger_symmetrize = true;
        }),
    },
    PaletteItem {
        name: "Project Floating Decal & Stamp Normals",
        category: "Modeling",
        shortcut: "Alt+D",
        keywords: &["decal", "project decal", "stamp normal", "floating mesh"],
        action: PaletteAction::TriggerAction(|state, _| {
            state.set_status_message("Press Alt+D to project selected decal onto scene hull");
        }),
    },
    PaletteItem {
        name: "Hardware Kitbash Drop Tool",
        category: "Modeling",
        shortcut: "K / Shift+K",
        keywords: &[
            "kitbash",
            "bolt",
            "screw",
            "vent",
            "latch",
            "insert hardware",
        ],
        action: PaletteAction::TriggerAction(|state, _| {
            crate::modals::kitbash::start(state);
        }),
    },
    // --- CAD & Precision Inspection ---
    PaletteItem {
        name: "Toggle CAD Caliper / 3D Measure",
        category: "CAD",
        shortcut: "M (Measure)",
        keywords: &[
            "measure",
            "caliper",
            "dimension",
            "distance",
            "cad",
            "distance between",
        ],
        action: PaletteAction::ToggleMeasure,
    },
    PaletteItem {
        name: "Recalculate Surface Normals Outside",
        category: "CAD",
        shortcut: "Shift+N",
        keywords: &[
            "recalculate normals",
            "flip",
            "facing",
            "outside",
            "inverted faces",
        ],
        action: PaletteAction::TriggerAction(|_, actions| {
            actions.trigger_recalculate_normals = Some(true);
        }),
    },
    PaletteItem {
        name: "3D Print & CAD Quality Inspector",
        category: "CAD",
        shortcut: "File > Print",
        keywords: &[
            "3d print",
            "slicer",
            "stl",
            "manifold",
            "watertight check",
            "wall thickness",
        ],
        action: PaletteAction::TriggerAction(|state, _| {
            state.print_inspector.show_dialog = true;
        }),
    },
    PaletteItem {
        name: "Fillet Spline Corners (G2 Curvature)",
        category: "CAD",
        shortcut: "Curve > Fillet",
        keywords: &[
            "fillet",
            "curvature",
            "g2",
            "spline",
            "smooth corners",
            "plasticity",
        ],
        action: PaletteAction::TriggerAction(|state, _| {
            if let Some(id) = state.selected_node {
                let cmd = Box::new(crate::commands::FilletCurveCommand::with_continuity(
                    id,
                    0.5,
                    None,
                    geo_kernel::FilletContinuity::G2Curvature,
                ));
                let _ = state.history.execute(cmd, &mut state.scene);
                state.set_status_message("Fillet G2 Curvature Applied to Spline");
            }
        }),
    },
    PaletteItem {
        name: "Spawn CAD 3-Point Circular Arc",
        category: "Add",
        shortcut: "Add > Curve",
        keywords: &["arc", "3-point", "circular", "cad", "spline", "curve"],
        action: PaletteAction::TriggerAction(|state, _| {
            let p1 = core_math::Vec3::new(-1.0, 0.0, 0.0);
            let p2 = core_math::Vec3::new(0.0, 1.0, 0.0);
            let p3 = core_math::Vec3::new(1.0, 0.0, 0.0);
            if let Some(arc) = geo_kernel::Spline::arc_3point(p1, p2, p3) {
                let cmd = Box::new(crate::commands::CreateCurveNodeCommand {
                    name: "Arc3Point".into(),
                    spline: arc,
                    created_node_id: None,
                });
                let _ = state.history.execute(cmd, &mut state.scene);
                state.set_status_message("CAD 3-Point Circular Arc Created");
            }
        }),
    },
    // --- File & Project ---
    PaletteItem {
        name: "Save Plumb Project (.plmb)",
        category: "File",
        shortcut: "Ctrl+S",
        keywords: &["save", "plmb", "project", "store", "export plmb"],
        action: PaletteAction::TriggerAction(|_, a| {
            a.export_plmb = true;
        }),
    },
    PaletteItem {
        name: "Export Active Mesh as glTF 2.0 (.gltf)",
        category: "File",
        shortcut: "Ctrl+E",
        keywords: &["export", "gltf", "web 3d", "game engine"],
        action: PaletteAction::TriggerAction(|_, a| {
            a.export_gltf = true;
        }),
    },
    PaletteItem {
        name: "Export Active Mesh as Binary glTF (.glb)",
        category: "File",
        shortcut: "Ctrl+Shift+G",
        keywords: &["export", "glb", "binary gltf", "unreal", "unity", "godot"],
        action: PaletteAction::TriggerAction(|_, a| {
            a.export_glb = true;
        }),
    },
    PaletteItem {
        name: "Export Active Mesh as STL for 3D Printing",
        category: "File",
        shortcut: "Ctrl+Shift+S",
        keywords: &["stl", "3d print", "export stl", "slicer", "bambu", "prusa"],
        action: PaletteAction::TriggerAction(|_, a| {
            a.export_stl = true;
        }),
    },
    PaletteItem {
        name: "Export Active Mesh as CAD STEP Solid (.step)",
        category: "File",
        shortcut: "File > Export",
        keywords: &[
            "step",
            "stp",
            "cad solid",
            "brep",
            "solidworks",
            "cnc",
            "cam",
        ],
        action: PaletteAction::TriggerAction(|_, a| {
            a.export_step = true;
        }),
    },
    PaletteItem {
        name: "Export Full Scene as Pixar OpenUSD (.usda)",
        category: "File",
        shortcut: "File > Export",
        keywords: &[
            "usd",
            "usda",
            "openusd",
            "pixar",
            "houdini",
            "unreal",
            "materialx",
        ],
        action: PaletteAction::TriggerAction(|_, a| {
            a.export_usd = true;
        }),
    },
    PaletteItem {
        name: "Import Pixar OpenUSD Scene (.usda)",
        category: "File",
        shortcut: "File > Import",
        keywords: &["usd", "usda", "import", "openusd", "pixar"],
        action: PaletteAction::TriggerAction(|_, a| {
            a.import_usd = true;
        }),
    },
    PaletteItem {
        name: "Export Sliced 3D Print G-code (.gcode)",
        category: "File",
        shortcut: "File > Export",
        keywords: &[
            "gcode", "slicer", "toolpath", "3d print", "bambu", "prusa", "creality",
        ],
        action: PaletteAction::TriggerAction(|_, a| {
            a.export_gcode = true;
        }),
    },
    PaletteItem {
        name: "Toggle OpenXR Spatial Computing (VR 6-DoF)",
        category: "View",
        shortcut: "Window > OpenXR",
        keywords: &[
            "xr",
            "openxr",
            "vr",
            "spatial",
            "headset",
            "immersive",
            "vision pro",
            "quest",
        ],
        action: PaletteAction::TriggerAction(|s, _| {
            s.xr_session.is_active = !s.xr_session.is_active;
        }),
    },
    PaletteItem {
        name: "Step Inertial Motion Matching Locomotion",
        category: "Animation",
        shortcut: "Animation > Motion",
        keywords: &[
            "motion matching",
            "locomotion",
            "sprint",
            "run",
            "walk",
            "inertialization",
        ],
        action: PaletteAction::TriggerAction(|s, _| {
            s.motion_matching
                .step(0.016, core_math::Vec2::new(0.0, 1.0));
        }),
    },
    PaletteItem {
        name: "Welcome & Quick-Start Hub",
        category: "App",
        shortcut: "F2",
        keywords: &[
            "welcome",
            "splash",
            "start",
            "templates",
            "new project",
            "hub",
        ],
        action: PaletteAction::OpenWelcomeHub,
    },
    PaletteItem {
        name: "Preferences & UI Density",
        category: "App",
        shortcut: "Ctrl+,",
        keywords: &[
            "preferences",
            "settings",
            "density",
            "keymap",
            "options",
            "configure",
        ],
        action: PaletteAction::OpenPreferences,
    },
    // --- Workspaces ---
    PaletteItem {
        name: "Switch to Layout Workspace",
        category: "Workspace",
        shortcut: "1",
        keywords: &["workspace", "layout"],
        action: PaletteAction::SetWorkspace(WorkspaceTemplate::General),
    },
    PaletteItem {
        name: "Switch to Hard-Surface Workspace",
        category: "Workspace",
        shortcut: "2",
        keywords: &["workspace", "hard-surface"],
        action: PaletteAction::SetWorkspace(WorkspaceTemplate::HardSurface),
    },
    PaletteItem {
        name: "Switch to CAD Precision Workspace",
        category: "Workspace",
        shortcut: "3",
        keywords: &["workspace", "cad"],
        action: PaletteAction::SetWorkspace(WorkspaceTemplate::CadPrecision),
    },
    PaletteItem {
        name: "Switch to Digital Sculpting Workspace",
        category: "Workspace",
        shortcut: "4",
        keywords: &["workspace", "sculpt"],
        action: PaletteAction::SetWorkspace(WorkspaceTemplate::Sculpting),
    },
    PaletteItem {
        name: "Switch to UV & Retopo Workspace",
        category: "Workspace",
        shortcut: "5",
        keywords: &["workspace", "uv"],
        action: PaletteAction::SetWorkspace(WorkspaceTemplate::UvRetopo),
    },
    PaletteItem {
        name: "Switch to LookDev Workspace",
        category: "Workspace",
        shortcut: "6",
        keywords: &["workspace", "lookdev"],
        action: PaletteAction::SetWorkspace(WorkspaceTemplate::LookDev),
    },
    // --- Procedural Generation & Bio ---
    PaletteItem {
        name: "Spawn Procedural Oak Tree (L-System)",
        category: "Add",
        shortcut: "Add > Bio",
        keywords: &[
            "tree",
            "lsystem",
            "plant",
            "organic",
            "foliage",
            "procedural",
        ],
        action: PaletteAction::TriggerAction(|_, a| {
            if let Ok(m) =
                geo_kernel::LSystem::from_preset(geo_kernel::LSystemPreset::OakTree).generate_mesh()
            {
                a.spawn_mesh = Some(("OakTree", m));
            }
        }),
    },
    PaletteItem {
        name: "Spawn Biomechanical Conduit (L-System)",
        category: "Add",
        shortcut: "Add > Bio",
        keywords: &[
            "conduit",
            "pipe",
            "cable",
            "biomechanical",
            "sci-fi",
            "lsystem",
        ],
        action: PaletteAction::TriggerAction(|_, a| {
            if let Ok(m) = geo_kernel::LSystem::from_preset(
                geo_kernel::LSystemPreset::SciFiBiomechanicalConduit,
            )
            .generate_mesh()
            {
                a.spawn_mesh = Some(("BioConduit", m));
            }
        }),
    },
    PaletteItem {
        name: "Spawn Geodesic Dome",
        category: "Add",
        shortcut: "Add > Mesh",
        keywords: &["geodesic", "dome", "hemisphere", "sphere", "architecture"],
        action: PaletteAction::TriggerAction(|_, a| {
            a.spawn_mesh = Some((
                "GeodesicDome",
                geo_kernel::Primitives::create_geodesic_dome(2.0, 2),
            ));
        }),
    },
    PaletteItem {
        name: "Spawn Pipe / Screw Thread",
        category: "Add",
        shortcut: "Add > Mesh",
        keywords: &["pipe", "thread", "screw", "helix", "cad", "mechanical"],
        action: PaletteAction::TriggerAction(|_, a| {
            a.spawn_mesh = Some((
                "PipeThread",
                geo_kernel::Primitives::create_pipe_thread(1.2, 0.3, 2.0, 16),
            ));
        }),
    },
    // --- Themes ---
    PaletteItem {
        name: "Theme: Plumb Slate (Default Gold)",
        category: "Theme",
        shortcut: "",
        keywords: &["theme", "slate", "dark", "gold", "default"],
        action: PaletteAction::SetTheme(ThemePreset::PlumbSlate),
    },
    PaletteItem {
        name: "Theme: Blender Charcoal",
        category: "Theme",
        shortcut: "",
        keywords: &["theme", "blender", "charcoal", "cyan"],
        action: PaletteAction::SetTheme(ThemePreset::BlenderDark),
    },
    PaletteItem {
        name: "Theme: Maya Neutral Gray",
        category: "Theme",
        shortcut: "",
        keywords: &["theme", "maya", "gray", "orange"],
        action: PaletteAction::SetTheme(ThemePreset::MayaGray),
    },
    PaletteItem {
        name: "Theme: CAD Blueprint Navy",
        category: "Theme",
        shortcut: "",
        keywords: &["theme", "cad", "blueprint", "navy", "cyan"],
        action: PaletteAction::SetTheme(ThemePreset::CadBlueprint),
    },
    PaletteItem {
        name: "Theme: Paper Light Drafting",
        category: "Theme",
        shortcut: "",
        keywords: &["theme", "light", "white", "paper", "drafting", "daylight"],
        action: PaletteAction::SetTheme(ThemePreset::PaperLight),
    },
    // --- Viewport Toggles ---
    PaletteItem {
        name: "Toggle Viewport Wireframe Overlay",
        category: "Viewport",
        shortcut: "Shift+Z",
        keywords: &["wireframe", "edges", "overlay", "lines"],
        action: PaletteAction::ToggleWireframe,
    },
    PaletteItem {
        name: "Toggle Construction Grid",
        category: "Viewport",
        shortcut: "G",
        keywords: &["grid", "ground plane", "axes", "floor"],
        action: PaletteAction::ToggleGrid,
    },
    PaletteItem {
        name: "Toggle Transform Snapping",
        category: "Viewport",
        shortcut: "Shift+Tab",
        keywords: &["snap", "grid snap", "magnet", "align to vertex"],
        action: PaletteAction::ToggleSnap,
    },
    PaletteItem {
        name: "Viewport Shading: MatCap Clay",
        category: "Shading",
        shortcut: "",
        keywords: &["matcap", "clay", "sculpt", "shading"],
        action: PaletteAction::SetShading(ViewportShadingMode::MatCapClay),
    },
    PaletteItem {
        name: "Viewport Shading: Studio",
        category: "Shading",
        shortcut: "",
        keywords: &["studio", "default shading", "smooth"],
        action: PaletteAction::SetShading(ViewportShadingMode::Studio),
    },
    PaletteItem {
        name: "Viewport Shading: Live Path Tracer",
        category: "Shading",
        shortcut: "F12",
        keywords: &[
            "render",
            "path tracer",
            "raytracing",
            "production renderer",
            "global illumination",
        ],
        action: PaletteAction::SetShading(ViewportShadingMode::Rendered),
    },
    PaletteItem {
        name: "Direct PBR Snapshot Render",
        category: "Rendering",
        shortcut: "F11",
        keywords: &[
            "render", "snapshot", "pbr", "instant", "hardware", "viewport",
        ],
        action: PaletteAction::TriggerAction(|state, _| {
            state.production_render.engine = crate::ui::render_dialog::RenderEngine::DirectPbr;
            state.production_render.show_dialog = true;
            crate::ui::render_dialog::start_production_render(state);
        }),
    },
    // --- Help ---
    PaletteItem {
        name: "User Manual & Shortcuts Guide",
        category: "Help",
        shortcut: "F1",
        keywords: &["help", "manual", "guide", "documentation", "shortcuts"],
        action: PaletteAction::OpenHelp(0),
    },
    PaletteItem {
        name: "Beginner Interactive Tutorials",
        category: "Help",
        shortcut: "F1 -> Tab 1",
        keywords: &["beginner", "tutorial", "first time", "learning", "start"],
        action: PaletteAction::OpenHelp(1),
    },
];

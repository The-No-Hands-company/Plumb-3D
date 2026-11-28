use super::items::{PaletteAction, PaletteItem};
use crate::ui::theme::ThemePreset;
use crate::ui::workspaces::WorkspaceTemplate;
use render_viewport::ViewportShadingMode;

pub const SCENE_COMMANDS: &[PaletteItem] = &[
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
        name: "Viewport Shading: Zebra Surfacing",
        category: "Shading",
        shortcut: "",
        keywords: &["zebra", "reflection lines", "class-a", "surfacing", "continuity"],
        action: PaletteAction::SetShading(ViewportShadingMode::Zebra),
    },
    PaletteItem {
        name: "Viewport Shading: Gaussian Curvature Heatmap",
        category: "Shading",
        shortcut: "",
        keywords: &["gaussian", "curvature", "heatmap", "diagnostics", "inflection", "class-a"],
        action: PaletteAction::SetShading(ViewportShadingMode::GaussianCurvature),
    },
    PaletteItem {
        name: "Viewport Shading: Mean Curvature Heatmap",
        category: "Shading",
        shortcut: "",
        keywords: &["mean", "curvature", "heatmap", "diagnostics", "laplacian", "class-a"],
        action: PaletteAction::SetShading(ViewportShadingMode::MeanCurvature),
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

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

#[derive(Clone, Copy)]
pub struct PaletteItem {
    pub name: &'static str,
    pub category: &'static str,
    pub shortcut: &'static str,
    pub keywords: &'static [&'static str],
    pub action: PaletteAction,
}

pub const MODELING_COMMANDS: &[PaletteItem] = &[
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
        name: "Push-Pull: Modal Carve (Boss/Cavity)",
        category: "Modeling",
        shortcut: "P",
        keywords: &[
            "push",
            "pull",
            "carve",
            "boss",
            "cavity",
            "pocket",
            "plasticity",
        ],
        action: PaletteAction::TriggerAction(|state, _| {
            state.interaction_mode = InteractionMode::Edit;
            if !state.selected_faces.is_empty() {
                let mouse_pos = (
                    state.surface_config.width as f64 * 0.5,
                    state.surface_config.height as f64 * 0.5,
                );
                crate::modals::push_pull::start(state, mouse_pos);
            }
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
    // --- Advanced Retopology & Pixar SubD Limit ---
    PaletteItem {
        name: "Mesh: Field-Aligned Quad Remesh",
        category: "Modeling",
        shortcut: "Mesh > Remesh",
        keywords: &[
            "quad",
            "remesh",
            "retopology",
            "field",
            "curvature",
            "zremesher",
            "instant meshes",
            "flow",
        ],
        action: PaletteAction::TriggerAction(|state, _| {
            if let Some(sel_id) = state.selected_node {
                let settings = geo_kernel::FieldRemeshSettings {
                    target_edge_length: state.field_remesh_edge_len,
                    curvature_influence: state.field_remesh_curvature_influence,
                    smooth_passes: 4,
                };
                let cmd = Box::new(crate::commands::FieldRemeshCommand::new(sel_id, settings));
                let _ = state.history.execute(cmd, &mut state.scene);
                state.sync_gpu_meshes();
                state.set_status_message("Mesh remeshed to field-aligned quads.");
            }
        }),
    },
    PaletteItem {
        name: "Mesh: Evaluate SubD Limit Surface",
        category: "Modeling",
        shortcut: "Mesh > SubD",
        keywords: &[
            "subd",
            "subdivision",
            "limit",
            "surface",
            "pixar",
            "opensubdiv",
            "stencil",
        ],
        action: PaletteAction::TriggerAction(|state, _| {
            if let Some(sel_id) = state.selected_node {
                let cmd = Box::new(crate::commands::SubdLimitCommand::new(sel_id));
                let _ = state.history.execute(cmd, &mut state.scene);
                state.sync_gpu_meshes();
                state.set_status_message("Mesh projected to exact Pixar SubD limit surface.");
            }
        }),
    },
];

pub static COMMAND_LIST: std::sync::LazyLock<Vec<PaletteItem>> = std::sync::LazyLock::new(|| {
    let mut list = MODELING_COMMANDS.to_vec();
    list.extend_from_slice(super::items_scene::SCENE_COMMANDS);
    list
});

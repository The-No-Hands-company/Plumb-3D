pub use render_viewport::TransformOrientation;

/// Interaction modes for the DCC Suite viewport (Blender / Maya style).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InteractionMode {
    Object,
    Edit,
    Sculpt,
    WeightPaint,
    VertexPaint,
    Pose,
    CurveEdit,
}

/// Sub-element selection level in Mesh Edit Mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubElementLevel {
    Vertex,
    Edge,
    Face,
}

/// Interactive snapping targets for 3D element alignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SnapTarget {
    #[default]
    Grid,
    Vertex,
    EdgeMidpoint,
    FaceCenter,
}

/// Spline knot and tangent handle selection component.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CurveHandleSelection {
    Knot,
    HandleIn,
    HandleOut,
}

/// Animation timeline visualization mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TimelineViewMode {
    #[default]
    DopeSheet,
    FCurve,
}

/// Weight paint tool modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeightPaintTool {
    Brush,
    LinearGradient,
}

/// Reference point alignment mode for Object Align Dialog (3ds Max / Maya).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AlignPosMode {
    Pivot,
    #[default]
    Center,
    Minimum,
    Maximum,
}

/// UV projection mapping algorithms.
#[derive(Debug, Clone, Copy)]
pub enum UVProjectionType {
    Planar(geo_kernel::ProjectionAxis),
    Box,
    Cylindrical(geo_kernel::ProjectionAxis),
    Spherical,
}

/// Active tab in the right-hand Inspector panel (Blender / Cinema4D style).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InspectorTab {
    #[default]
    Transform,
    Modifiers,
    Materials,
    Tools,
    Constraints,
    Physics,
}

/// Recorded parameters of the most recent modeling operation for the interactive "Adjust Last Operation" panel (Blender F9).
#[derive(Debug, Clone)]
pub enum LastOperatorInfo {
    Inset {
        faces: Vec<geo_kernel::FaceKey>,
        thickness: f32,
        factor: f32,
        individual: bool,
        initial_radius: f32,
    },
    Extrude {
        faces: Vec<geo_kernel::FaceKey>,
        distance: f32,
        axis: Option<render_viewport::GizmoAxis>,
    },
    Bevel {
        edges: Vec<geo_kernel::EdgeKey>,
        width: f32,
        segments: usize,
    },
    LoopCut {
        edge: geo_kernel::EdgeKey,
        num_cuts: usize,
        factor: f32,
    },
}

/// Interface layout density setting for screen real estate optimization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UiDensity {
    /// Condensed margins and smaller padding for high-density power modeling on laptops.
    Compact,
    /// Ergonomic spacing and legible typography for standard drafting and 4K displays.
    #[default]
    Comfortable,
}

/// Input keymap style preset across 3D navigation and transform operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum KeymapStyle {
    /// Modern Plumb 3D native workflow (W/E/R transforms, Tab toggle, F3 search).
    #[default]
    PlumbStandard,
    /// Industry classic Blender workflow (G Grab, S Scale, R Rotate, Shift+A Add).
    BlenderVeteran,
    /// Studio production Maya workflow (Alt+LMB Orbit, Alt+MMB Pan, W/E/R).
    MayaStudio,
}




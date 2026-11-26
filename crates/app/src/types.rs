pub use render_viewport::TransformOrientation;

/// Interaction modes for the DCC Suite viewport (Blender / Maya style).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InteractionMode {
    Object,
    Edit,
    Sculpt,
    WeightPaint,
    VertexPaint,
    TexturePaint,
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
    AddPrimitive {
        node_id: graph_engine::NodeId,
        params: PrimitiveParams,
    },
    Bridge {
        edges: Vec<geo_kernel::EdgeKey>,
        twist: i32,
        cuts: usize,
    },
}

/// Parametric definitions for live primitive adjustment in the F9 operator panel.
#[derive(Debug, Clone)]
pub enum PrimitiveParams {
    Box { width: f32, height: f32, depth: f32, seg_x: u32, seg_y: u32, seg_z: u32 },
    Plane { width: f32, length: f32, sub_x: u32, sub_z: u32 },
    UvSphere { radius: f32, segments: u32, rings: u32 },
    Cylinder { radius: f32, height: f32, segments: u32, cap_fill: geo_kernel::CapFillType },
    Cone { radius_bottom: f32, radius_top: f32, height: f32, segments: u32, cap_fill: geo_kernel::CapFillType },
    Torus { major_radius: f32, minor_radius: f32, major_segments: u32, minor_segments: u32 },
    Pyramid { base_width: f32, base_depth: f32, height: f32 },
    Capsule { radius: f32, cylinder_height: f32, segments: u32, hemisphere_rings: u32 },
    IcoSphere { radius: f32, subdivisions: u32 },
    PlumbBob { segments: usize },
    GeodesicDome { radius: f32, frequency: u32 },
    PipeThread { major_diameter: f32, pitch: f32, thread_length: f32, segments_per_turn: u32 },
}

impl PrimitiveParams {
    pub fn name(&self) -> &str {
        match self {
            Self::Box { .. } => "Add Box / Cube",
            Self::Plane { .. } => "Add Plane Grid",
            Self::UvSphere { .. } => "Add UV Sphere",
            Self::Cylinder { .. } => "Add Cylinder",
            Self::Cone { .. } => "Add Cone / Frustum",
            Self::Torus { .. } => "Add Torus / Donut",
            Self::Pyramid { .. } => "Add Pyramid",
            Self::Capsule { .. } => "Add Capsule / Pill",
            Self::IcoSphere { .. } => "Add IcoSphere",
            Self::PlumbBob { .. } => "Add Plumb Bob",
            Self::GeodesicDome { .. } => "Add Geodesic Dome",
            Self::PipeThread { .. } => "Add Pipe / Screw Thread",
        }
    }

    pub fn generate_mesh(&self) -> geo_kernel::Mesh {
        match self {
            Self::Box { width, height, depth, seg_x, seg_y, seg_z } => {
                geo_kernel::Primitives::create_box_parametric(*width, *height, *depth, *seg_x, *seg_y, *seg_z)
            }
            Self::Plane { width, length, sub_x, sub_z } => {
                geo_kernel::Primitives::create_plane(*width, *length, *sub_x, *sub_z)
            }
            Self::UvSphere { radius, segments, rings } => {
                geo_kernel::Primitives::create_uv_sphere(*radius, *segments, *rings)
            }
            Self::Cylinder { radius, height, segments, cap_fill } => {
                geo_kernel::Primitives::create_cylinder_parametric(*radius, *height, *segments, *cap_fill)
            }
            Self::Cone { radius_bottom, radius_top, height, segments, cap_fill } => {
                geo_kernel::Primitives::create_cone_parametric(*radius_bottom, *radius_top, *height, *segments, *cap_fill)
            }
            Self::Torus { major_radius, minor_radius, major_segments, minor_segments } => {
                geo_kernel::Primitives::create_torus(*major_radius, *minor_radius, *major_segments, *minor_segments)
            }
            Self::Pyramid { base_width, base_depth, height } => {
                geo_kernel::Primitives::create_pyramid_parametric(*base_width, *base_depth, *height)
            }
            Self::Capsule { radius, cylinder_height, segments, hemisphere_rings } => {
                geo_kernel::Primitives::create_capsule(*radius, *cylinder_height, *segments, *hemisphere_rings)
            }
            Self::IcoSphere { radius, subdivisions } => {
                geo_kernel::Primitives::create_icosphere(*radius, *subdivisions)
            }
            Self::PlumbBob { segments } => {
                geo_kernel::Primitives::create_plumb_bob_detailed(*segments)
            }
            Self::GeodesicDome { radius, frequency } => {
                geo_kernel::Primitives::create_geodesic_dome(*radius, *frequency)
            }
            Self::PipeThread { major_diameter, pitch, thread_length, segments_per_turn } => {
                geo_kernel::Primitives::create_pipe_thread(*major_diameter, *pitch, *thread_length, *segments_per_turn)
            }
        }
    }
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

/// Multi-element viewport selection tool mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SelectionToolMode {
    /// Rectangular marquee bounding box (Hotkey B).
    #[default]
    Box,
    /// Freehand polygonal lasso lassoing sub-elements (Hotkey Alt+B).
    Lasso,
    /// Continuous circular paint brush selection (Hotkey C / Drag).
    CirclePaint,
}




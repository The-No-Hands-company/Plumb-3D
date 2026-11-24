use core_math::{Transform, Vec3};
use geo_kernel::{
    BrushFalloff, CutPoint, EdgeKey, FaceKey, Mesh, MirrorAxis, NgonMethod,
    Primitives, QuadMethod, SculptSettings, SymmetrizeDirection, VertexKey,
};
use graph_engine::{
    AnimationManager, ConstraintStack, HistoryManager,
    NodeId, SceneGraph, SceneNode,
};
use render_viewport::{
    GizmoAxis, GpuMesh, TonemappingMode,
    TransformGizmo, ViewportCamera, ViewportRenderer, ViewportShadingMode,
};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Instant;
use winit::window::Window;

use crate::types::*;

pub struct AppState {
    pub window: Arc<Window>,
    pub device: Arc<wgpu::Device>,
    pub queue: Arc<wgpu::Queue>,
    pub surface: wgpu::Surface<'static>,
    pub surface_config: wgpu::SurfaceConfiguration,
    pub depth_texture: wgpu::Texture,
    pub depth_view: wgpu::TextureView,

    // Viewport Renderer & Camera
    pub renderer: ViewportRenderer,
    pub camera: ViewportCamera,
    pub active_camera_node: Option<NodeId>,
    pub lock_camera_to_view: bool,
    pub gpu_mesh_cache: HashMap<NodeId, GpuMesh>,
    pub gpu_texture_bind_groups: HashMap<String, wgpu::BindGroup>,

    // 3D Transform Gizmo
    pub gizmo: TransformGizmo,
    pub transform_orientation: TransformOrientation,
    pub hovered_gizmo_axis: Option<GizmoAxis>,
    pub active_gizmo_axis: Option<GizmoAxis>,
    pub drag_start_transform: Option<Transform>,

    // Scene & Selection
    pub scene: SceneGraph,
    pub selected_node: Option<NodeId>,
    pub selected_nodes: HashSet<NodeId>,
    pub previous_selected_node: Option<NodeId>,
    pub selected_vertices: HashSet<VertexKey>,
    pub selected_edges: HashSet<EdgeKey>,
    pub selected_faces: HashSet<FaceKey>,
    pub interaction_mode: InteractionMode,
    pub sub_element_level: SubElementLevel,
    pub shift_pressed: bool,
    pub ctrl_pressed: bool,
    pub alt_pressed: bool,

    // Proportional Editing (Soft Selection)
    pub proportional_editing: bool,
    pub proportional_radius: f32,

    // Box / Marquee Selection (Blender 'B' / Drag-select)
    pub is_box_selecting: bool,
    pub box_select_start: Option<(f32, f32)>,
    pub box_select_current: Option<(f32, f32)>,

    // Snapping (Blender Shift+Tab / Grid, Vertex, Edge, Face)
    pub snap_enabled: bool,
    pub snap_target: SnapTarget,
    pub snap_to_grid: bool,
    pub grid_snap_size: f32,
    pub snap_angle: bool,
    pub angle_snap_degrees: f32,
    pub snap_distance_threshold: f32,
    pub last_snap_hit: Option<(Vec3, &'static str)>,

    // Interactive Knife Tool (Blender 'K' / Maya Multi-Cut)
    pub knife_active: bool,
    pub knife_points: Vec<(CutPoint, Vec3, FaceKey)>,
    pub knife_hovered_hit: Option<(CutPoint, Vec3, FaceKey)>,

    // Interactive Loop Cut Modal (Blender Ctrl+R / Maya Insert Edge Loop)
    pub loop_cut_active: bool,
    pub loop_cut_sliding: bool,
    pub loop_cut_hovered_edge: Option<EdgeKey>,
    pub loop_cut_factor: f32,
    pub loop_cut_num_cuts: usize,
    pub loop_cut_mouse_start_x: f64,

    // Interactive Extrude Modal (Blender 'E' / Maya Extrude)
    pub extrude_modal_active: bool,
    pub extrude_modal_start_mouse: (f64, f64),
    pub extrude_modal_distance: f32,
    pub extrude_modal_faces: Vec<FaceKey>,
    pub extrude_modal_axis_constraint: Option<GizmoAxis>,

    // Interactive Bevel Modal (Blender Ctrl+B / Maya Bevel)
    pub bevel_modal_active: bool,
    pub bevel_modal_start_mouse: (f64, f64),
    pub bevel_modal_width: f32,
    pub bevel_modal_segments: usize,
    pub bevel_modal_edges: Vec<EdgeKey>,
    pub bevel_modal_faces: Vec<FaceKey>,

    // Interactive Inset Modal (Blender 'I' / Maya Inset)
    pub inset_modal_active: bool,
    pub inset_modal_start_mouse: (f64, f64),
    pub inset_modal_factor: f32,
    pub inset_modal_thickness: f32,
    pub inset_modal_initial_radius: f32,
    pub inset_modal_individual: bool,
    pub inset_modal_faces: Vec<FaceKey>,
    pub modal_numeric_buffer: String,
    pub last_operator: Option<LastOperatorInfo>,
    pub show_last_operator_panel: bool,

    // Interactive Edge Crease Modal (Blender Shift+E / Maya Crease)
    pub edge_crease_modal_active: bool,
    pub edge_crease_modal_weight: f32,
    pub edge_crease_modal_start_mouse: (f64, f64),
    pub edge_crease_modal_edges: Vec<EdgeKey>,
    pub edge_crease_modal_original_weights: Vec<(EdgeKey, f32)>,

    // Interactive Vertex Crease Modal (Blender Shift+V / OpenSubdiv Semi-Sharp Corners)
    pub vertex_crease_modal_active: bool,
    pub vertex_crease_modal_weight: f32,
    pub vertex_crease_modal_start_mouse: (f64, f64),
    pub vertex_crease_modal_vertices: Vec<VertexKey>,
    pub vertex_crease_modal_original_weights: Vec<(VertexKey, f32)>,

    // UV Editor Window & Tools (Blender / Maya Style)
    pub show_uv_editor: bool,
    pub smart_uv_angle_threshold: f32,
    pub smart_uv_margin: f32,

    // Viewport Shelves & Sidebars
    pub show_outliner: bool,
    pub show_inspector: bool,
    pub show_left_toolbar: bool,

    // RMB Context Menu (Blender / Maya / Plasticity Style)
    pub show_context_menu: bool,
    pub context_menu_pos: Option<(f32, f32)>,

    // In-Software Help Manual & Shortcuts Reference (F1)
    pub show_help_dialog: bool,
    pub help_dialog_tab: usize,
    pub selected_tutorial: usize,
    pub help_search_query: String,

    // Visual Geometry Nodes (Blender Geo Nodes / Unreal Blueprints)
    pub geo_node_graph: geo_nodes::GeoNodeGraph,
    pub node_canvas_state: crate::ui::node_editor::NodeCanvasState,
    pub show_node_editor: bool,

    // Embedded Rhai Scripting Engine & Console
    pub script_console_state: crate::scripting::ScriptConsoleState,
    pub show_script_console: bool,

    // PBR Texture Map Baker
    pub bake_state: crate::ui::bake_dialog::BakeDialogState,

    // 3D Printing & CAD Quality Inspector
    pub print_inspector: crate::ui::print_inspector::PrintInspectorState,

    // Sculpt Mode Settings & Stroke State
    pub sculpt_settings: SculptSettings,
    pub is_sculpting: bool,
    pub sculpt_stroke_start_mesh: Option<Mesh>,
    pub last_sculpt_hit: Option<(Vec3, Vec3)>,
    pub sculpt_symmetry_x: bool, pub sculpt_symmetry_y: bool, pub sculpt_symmetry_z: bool,

    // Custom Object Colors (Albedo Tint)
    pub object_colors: HashMap<NodeId, [f32; 3]>,
    pub object_materials: HashMap<NodeId, [f32; 4]>,
    pub active_material_slot: usize,

    // History (Undo / Redo)
    pub history: HistoryManager,

    // Viewport Options & Live Path Tracer
    pub show_grid: bool, pub show_wireframe: bool, pub show_subd_cage: bool, pub optimal_display: bool, pub flat_shading: bool,
    pub viewport_shading: ViewportShadingMode,
    pub show_viewport_hud: bool,
    pub show_face_normals: bool, pub show_vertex_normals: bool, pub normal_display_length: f32,
    pub exposure: f32, pub tonemapping: TonemappingMode,
    pub show_cavity: bool, pub cavity_intensity: f32,
    pub live_pathtracer_buffer: Option<render_pathtracer::RenderBuffer>,
    pub live_pathtracer_scene: Option<render_pathtracer::PathTracerScene>,
    pub live_pathtracer_camera: Option<render_pathtracer::PathtracerCamera>,
    pub live_pathtracer_last_eye_target: (Vec3, Vec3),
    pub live_pathtracer_max_samples: u32,
    pub live_pathtracer_tex_handle: Option<egui::TextureHandle>,

    // Mouse Interaction for Viewport
    pub last_mouse_pos: Option<(f64, f64)>,
    pub mouse_drag_distance: f32,
    pub is_orbiting: bool,
    pub is_panning: bool,

    // Direct modeling parameters
    pub extrude_distance: f32,
    pub bevel_offset: f32,
    pub bevel_inset: f32,
    pub loop_cut_ratio: f32,
    pub inset_factor: f32,
    pub inset_individual: bool,
    pub bisect_plane_axis: usize,
    pub bisect_clear_positive: bool,
    pub bisect_clear_negative: bool,
    pub bisect_fill_cut: bool,
    pub triangulate_quad_method: QuadMethod,
    pub triangulate_ngon_method: NgonMethod,
    pub triangulate_min_vertices: usize,
    pub vertex_bevel_offset: f32,
    pub wireframe_thickness: f32,
    pub edge_slide_factor: f32,
    pub auto_smooth_angle_degrees: f32,
    pub x_mirror_editing: bool,
    pub symmetrize_axis: MirrorAxis,
    pub symmetrize_direction: SymmetrizeDirection,
    pub spin_angle_degrees: f32,
    pub spin_steps: usize,
    pub clean_min_edge_len: f32,
    pub clean_min_area: f32,
    pub cap_holes_triangulate: bool,
    pub grid_fill_span_count: usize,

    // Animation & Keyframe Timeline (Maya / Blender Style)
    pub animation: AnimationManager,
    pub timeline_expanded: bool,
    pub timeline_mode: TimelineViewMode,
    pub fcurve_show_tx: bool, pub fcurve_show_ty: bool, pub fcurve_show_tz: bool,
    pub fcurve_show_sx: bool, pub fcurve_show_sy: bool, pub fcurve_show_sz: bool,
    pub fcurve_show_rot: bool, pub fcurve_val_min: f32, pub fcurve_val_max: f32,

    // Spline Curve Editing Mode (Maya / 3ds Max / Blender Style)
    pub selected_curve_knot: Option<usize>,
    pub selected_curve_handle: Option<CurveHandleSelection>,

    // Scene Constraints & Physics Dynamics (Maya / 3ds Max / Blender Style)
    pub constraints: ConstraintStack,
    pub physics: graph_engine::PhysicsWorld,

    // Vertex Group & Weight Painting Mode (Blender / Maya Style)
    pub vertex_group_weight: f32,
    pub new_vg_name: String,
    pub active_vertex_group: Option<String>,
    pub weight_brush_radius: f32,
    pub weight_brush_strength: f32,
    pub weight_brush_target: f32,
    pub weight_brush_falloff: BrushFalloff,
    pub is_weight_painting: bool,
    pub weight_paint_tool: WeightPaintTool,
    pub weight_gradient_start: Option<Vec3>,
    pub weight_gradient_end: Option<Vec3>,
    pub weight_gradient_start_w: f32,
    pub weight_gradient_end_w: f32,
    pub weight_paint_stroke_start_mesh: Option<Mesh>,

    // Vertex Color / Polypaint Mode (ZBrush / Blender Style)
    pub vertex_paint_color: [f32; 4],
    pub vertex_paint_radius: f32,
    pub vertex_paint_strength: f32,
    pub vertex_paint_falloff: BrushFalloff,
    pub is_vertex_painting: bool,

    // 3D Measurement Tool (Maya Distance / Blender Measure)
    pub measure_mode: bool,
    pub measure_points: Vec<Vec3>,

    // Interactive 3D Bézier Curve Pen Tool (Plasticity / Illustrator Pen)
    pub curve_pen_active: bool,
    pub curve_pen_points: Vec<geo_kernel::BezierPoint>,
    pub curve_pen_is_closed: bool,
    pub curve_pen_drag_origin: Option<Vec3>,

    // Direct Edit Mode Operators
    pub shrink_fatten_offset: f32,
    pub relax_iterations: usize,
    pub relax_factor: f32,

    // Object Align Tool (Maya Align / 3ds Max Align Tool Alt+A)
    pub show_align_dialog: bool,
    pub align_pos_x: bool, pub align_pos_y: bool, pub align_pos_z: bool, pub align_pos_mode: AlignPosMode,
    pub align_rot_x: bool, pub align_rot_y: bool, pub align_rot_z: bool,
    pub align_scale_x: bool, pub align_scale_y: bool, pub align_scale_z: bool,

    // Outliner Search / Filtering
    pub outliner_filter: String,
    pub inspector_tab: InspectorTab,

    // Offline Production Path Tracer (Cycles / Arnold)
    pub production_render: crate::ui::render_dialog::ProductionRenderState,

    // UI, Theming & UX Customization
    pub current_theme: crate::ui::theme::ThemePreset,
    pub current_workspace: crate::ui::workspaces::WorkspaceTemplate,
    pub custom_workspaces: Vec<crate::ui::workspaces::CustomWorkspace>,
    pub command_palette: crate::ui::command_palette::CommandPaletteState,
    pub welcome_hub: crate::ui::welcome_hub::WelcomeHubState,
    pub preferences: crate::ui::preferences::PreferencesState,
    pub egui_ctx: egui::Context,
    pub egui_state: egui_winit::State,
    pub egui_renderer: egui_wgpu::Renderer,

    // Timing
    pub last_frame_time: Instant,
    pub fps: f32,
}

impl AppState {
    pub fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();
        let width = size.width.max(1);
        let height = size.height.max(1);

        let instance = wgpu::Instance::default();
        let surface = unsafe {
            let target = wgpu::SurfaceTargetUnsafe::from_window(&window).unwrap();
            instance.create_surface_unsafe(target).unwrap()
        };

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .expect("Failed to find suitable GPU adapter");

        let (raw_device, raw_queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("DCC Device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: Default::default(),
            },
            None,
        ))
        .expect("Failed to create GPU device");

        let device = Arc::new(raw_device);
        let queue = Arc::new(raw_queue);

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width,
            height,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &surface_config);

        let (depth_texture, depth_view) = Self::create_depth_texture(&device, &surface_config);

        let renderer = ViewportRenderer::new(device.clone(), queue.clone(), surface_format);

        let mut camera = ViewportCamera::new();
        camera.aspect = width as f32 / height as f32;

        let egui_ctx = egui::Context::default();
        crate::ui::apply_dcc_theme(&egui_ctx);
        let viewport_id = egui_ctx.viewport_id();
        let egui_state =
            egui_winit::State::new(egui_ctx.clone(), viewport_id, &window, None, None, None);
        let egui_renderer = egui_wgpu::Renderer::new(&device, surface_format, None, 1, false);

        let mut scene = SceneGraph::new();
        let cube_mesh = Primitives::create_cube(2.0);
        let first_vertex = cube_mesh.vertices.keys().next();
        let first_edge = cube_mesh.edges.keys().next();
        let first_face = cube_mesh.faces.keys().next();
        let default_cube =
            scene.add_node(SceneNode::new_mesh("DefaultCube", Transform::IDENTITY, cube_mesh));

        let mut object_colors = HashMap::new();
        object_colors.insert(default_cube, [0.72, 0.73, 0.75]);
        let mut object_materials = HashMap::new();
        object_materials.insert(default_cube, [0.0, 0.5, 0.0, 0.0]);

        let mut selected_vertices = HashSet::new();
        if let Some(vk) = first_vertex {
            selected_vertices.insert(vk);
        }
        let mut selected_edges = HashSet::new();
        if let Some(ek) = first_edge {
            selected_edges.insert(ek);
        }
        let mut selected_faces = HashSet::new();
        if let Some(fk) = first_face {
            selected_faces.insert(fk);
        }

        let mut app_state = Self {
            window,
            device,
            queue,
            surface,
            surface_config,
            depth_texture,
            depth_view,
            renderer,
            camera,
            active_camera_node: None,
            lock_camera_to_view: false,
            gpu_mesh_cache: HashMap::new(),
            gpu_texture_bind_groups: HashMap::new(),
            gizmo: TransformGizmo::new(),
            transform_orientation: TransformOrientation::Global,
            hovered_gizmo_axis: None,
            active_gizmo_axis: None,
            drag_start_transform: None,
            scene,
            selected_node: Some(default_cube),
            selected_nodes: [default_cube].into_iter().collect(),
            previous_selected_node: None,
            selected_vertices,
            selected_edges,
            selected_faces,
            interaction_mode: InteractionMode::Object,
            sub_element_level: SubElementLevel::Face,
            shift_pressed: false,
            ctrl_pressed: false,
            alt_pressed: false,
            proportional_editing: false,
            proportional_radius: 1.5,
            is_box_selecting: false,
            box_select_start: None,
            box_select_current: None,
            snap_enabled: false,
            snap_target: SnapTarget::Grid,
            snap_to_grid: false,
            grid_snap_size: 0.5,
            snap_angle: false,
            angle_snap_degrees: 15.0,
            snap_distance_threshold: 0.4,
            last_snap_hit: None,
            knife_active: false,
            knife_points: Vec::new(),
            knife_hovered_hit: None,
            loop_cut_active: false,
            loop_cut_sliding: false,
            loop_cut_hovered_edge: None,
            loop_cut_factor: 0.5,
            loop_cut_num_cuts: 1,
            loop_cut_mouse_start_x: 0.0,
            extrude_modal_active: false,
            extrude_modal_start_mouse: (0.0, 0.0),
            extrude_modal_distance: 0.0,
            extrude_modal_faces: Vec::new(),
            extrude_modal_axis_constraint: None,
            bevel_modal_active: false,
            bevel_modal_start_mouse: (0.0, 0.0),
            bevel_modal_width: 0.15,
            bevel_modal_segments: 1,
            bevel_modal_edges: Vec::new(),
            bevel_modal_faces: Vec::new(),
            inset_modal_active: false,
            inset_modal_start_mouse: (0.0, 0.0),
            inset_modal_factor: 0.8,
            inset_modal_thickness: 0.2,
            inset_modal_initial_radius: 1.0,
            inset_modal_individual: false,
            inset_modal_faces: Vec::new(),
            modal_numeric_buffer: String::new(),
            last_operator: None,
            show_last_operator_panel: true,
            edge_crease_modal_active: false,
            edge_crease_modal_weight: 0.0,
            edge_crease_modal_start_mouse: (0.0, 0.0),
            edge_crease_modal_edges: Vec::new(),
            edge_crease_modal_original_weights: Vec::new(),
            vertex_crease_modal_active: false,
            vertex_crease_modal_weight: 0.0,
            vertex_crease_modal_start_mouse: (0.0, 0.0),
            vertex_crease_modal_vertices: Vec::new(),
            vertex_crease_modal_original_weights: Vec::new(),
            show_uv_editor: false,
            smart_uv_angle_threshold: 66.0,
            smart_uv_margin: 0.02,
            show_outliner: true,
            show_inspector: true,
            show_left_toolbar: true,
            show_context_menu: false,
            context_menu_pos: None,
            show_help_dialog: false,
            help_dialog_tab: 0,
            selected_tutorial: 0,
            help_search_query: String::new(),
            geo_node_graph: geo_nodes::GeoNodeGraph::default(),
            node_canvas_state: crate::ui::node_editor::NodeCanvasState::default(),
            show_node_editor: false,
            script_console_state: crate::scripting::ScriptConsoleState::default(),
            show_script_console: false,
            sculpt_settings: SculptSettings::default(),
            is_sculpting: false,
            sculpt_stroke_start_mesh: None,
            last_sculpt_hit: None,
            sculpt_symmetry_x: true, sculpt_symmetry_y: false, sculpt_symmetry_z: false,
            object_colors,
            object_materials,
            active_material_slot: 0,
            history: HistoryManager::new(),
            show_grid: true,
            show_wireframe: true,
            show_subd_cage: true,
            optimal_display: true,
            flat_shading: false,
            viewport_shading: ViewportShadingMode::Studio,
            show_viewport_hud: true,
            show_face_normals: false, show_vertex_normals: false, normal_display_length: 0.25,
            exposure: 1.0, tonemapping: TonemappingMode::Aces,
            show_cavity: true, cavity_intensity: 0.5,
            live_pathtracer_buffer: None,
            live_pathtracer_scene: None,
            live_pathtracer_camera: None,
            live_pathtracer_last_eye_target: (Vec3::ZERO, Vec3::ZERO),
            live_pathtracer_max_samples: 64,
            live_pathtracer_tex_handle: None,
            last_mouse_pos: None,
            mouse_drag_distance: 0.0,
            is_orbiting: false,
            is_panning: false,
            extrude_distance: 0.75,
            bevel_offset: 0.2,
            bevel_inset: 0.2,
            loop_cut_ratio: 0.5,
            inset_factor: 0.8,
            inset_individual: false,
            bisect_plane_axis: 1,
            bisect_clear_positive: false,
            bisect_clear_negative: false,
            bisect_fill_cut: true,
            triangulate_quad_method: QuadMethod::Beauty,
            triangulate_ngon_method: NgonMethod::EarClipping,
            triangulate_min_vertices: 4,
            vertex_bevel_offset: 0.2,
            wireframe_thickness: 0.05,
            edge_slide_factor: 0.2,
            auto_smooth_angle_degrees: 30.0,
            x_mirror_editing: false,
            symmetrize_axis: MirrorAxis::X, symmetrize_direction: SymmetrizeDirection::PositiveToNegative,
            spin_angle_degrees: 360.0, spin_steps: 12, clean_min_edge_len: 0.001, clean_min_area: 1e-5,
            cap_holes_triangulate: false, grid_fill_span_count: 2,
            animation: AnimationManager::new(),
            timeline_expanded: true,
            timeline_mode: TimelineViewMode::default(),
            fcurve_show_tx: true, fcurve_show_ty: true, fcurve_show_tz: true,
            fcurve_show_sx: false, fcurve_show_sy: false, fcurve_show_sz: false,
            fcurve_show_rot: false, fcurve_val_min: -5.0, fcurve_val_max: 5.0,
            selected_curve_knot: None, selected_curve_handle: None,
            constraints: ConstraintStack::new(),
            physics: graph_engine::PhysicsWorld::new(),
            vertex_group_weight: 1.0,
            new_vg_name: "Group.001".to_string(),
            active_vertex_group: None,
            weight_brush_radius: 0.8,
            weight_brush_strength: 0.5,
            weight_brush_target: 1.0,
            weight_brush_falloff: BrushFalloff::Smooth, is_weight_painting: false,
            weight_paint_tool: WeightPaintTool::Brush, weight_gradient_start: None, weight_gradient_end: None,
            weight_gradient_start_w: 1.0, weight_gradient_end_w: 0.0,
            weight_paint_stroke_start_mesh: None,
            vertex_paint_color: [1.0, 0.2, 0.2, 1.0],
            vertex_paint_radius: 0.8,
            vertex_paint_strength: 0.5,
            vertex_paint_falloff: BrushFalloff::Smooth,
            is_vertex_painting: false,
            measure_mode: false,
            measure_points: Vec::new(),
            curve_pen_active: false,
            curve_pen_points: Vec::new(),
            curve_pen_is_closed: false,
            curve_pen_drag_origin: None,
            shrink_fatten_offset: 0.1,
            relax_iterations: 3,
            relax_factor: 0.3,
            show_align_dialog: false,
            align_pos_x: true, align_pos_y: true, align_pos_z: true,
            align_pos_mode: AlignPosMode::Center,
            align_rot_x: false, align_rot_y: false, align_rot_z: false,
            align_scale_x: false, align_scale_y: false, align_scale_z: false,
            outliner_filter: String::new(),
            inspector_tab: InspectorTab::default(),
            production_render: crate::ui::render_dialog::ProductionRenderState::default(),
            bake_state: crate::ui::bake_dialog::BakeDialogState::default(),
            print_inspector: crate::ui::print_inspector::PrintInspectorState::default(),
            current_theme: crate::ui::theme::ThemePreset::PlumbSlate,
            current_workspace: crate::ui::workspaces::WorkspaceTemplate::General,
            custom_workspaces: Vec::new(),
            command_palette: crate::ui::command_palette::CommandPaletteState::default(),
            welcome_hub: crate::ui::welcome_hub::WelcomeHubState::default(),
            preferences: crate::ui::preferences::PreferencesState::default(),
            egui_ctx,
            egui_state,
            egui_renderer,
            last_frame_time: Instant::now(),
            fps: 60.0,
        };

        app_state.sync_gpu_meshes();
        app_state
    }

    pub fn set_theme(&mut self, theme: crate::ui::theme::ThemePreset) {
        self.current_theme = theme;
        crate::ui::theme::apply_theme(&self.egui_ctx, theme);
    }
}

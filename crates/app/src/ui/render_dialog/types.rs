use render_pathtracer::{PathTracerScene, PathtracerCamera, RenderBuffer, RenderSettings};
use std::time::Instant;

/// Preset output resolutions for production rendering
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolutionPreset {
    Preview360p,
    Hd720p,
    FullHd1080p,
    Qhd1440p,
    Custom,
}

impl ResolutionPreset {
    pub fn dimensions(self) -> (u32, u32) {
        match self {
            Self::Preview360p => (640, 360),
            Self::Hd720p => (1280, 720),
            Self::FullHd1080p => (1920, 1080),
            Self::Qhd1440p => (2560, 1440),
            Self::Custom => (640, 360),
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Preview360p => "640 x 360 (Draft Preview)",
            Self::Hd720p => "1280 x 720 (HD)",
            Self::FullHd1080p => "1920 x 1080 (Full HD)",
            Self::Qhd1440p => "2560 x 1440 (2K QHD)",
            Self::Custom => "Custom Resolution",
        }
    }
}

/// Preset target sample counts for Monte Carlo path tracing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SamplePreset {
    Draft16,
    Preview64,
    Medium128,
    High256,
    Ultra512,
    Custom,
}

impl SamplePreset {
    pub fn target_samples(self) -> u32 {
        match self {
            Self::Draft16 => 16,
            Self::Preview64 => 64,
            Self::Medium128 => 128,
            Self::High256 => 256,
            Self::Ultra512 => 512,
            Self::Custom => 64,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Draft16 => "16 spp (Fast Draft)",
            Self::Preview64 => "64 spp (Balanced Preview)",
            Self::Medium128 => "128 spp (Clean)",
            Self::High256 => "256 spp (Production)",
            Self::Ultra512 => "512 spp (Ultra High Quality)",
            Self::Custom => "Custom Samples",
        }
    }
}

/// Selected rendering engine in Plumb 3D
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderEngine {
    /// Progressive Monte Carlo Path Tracer with physical global illumination & multi-bounce BVH
    PathTracer,
    /// Direct PBR Hardware evaluation (instant, zero Monte Carlo noise, ideal for space / LookDev)
    DirectPbr,
}

impl RenderEngine {
    pub fn label(self) -> &'static str {
        match self {
            Self::PathTracer => "Production Path Tracer (Ray Tracing GI)",
            Self::DirectPbr => "Direct PBR / Viewport (Instant / Zero Noise)",
        }
    }
}

/// State for the offline Production Path Tracer dialog and execution
pub struct ProductionRenderState {
    pub engine: RenderEngine,
    pub show_dialog: bool,
    pub settings: RenderSettings,
    pub resolution_preset: ResolutionPreset,
    pub sample_preset: SamplePreset,
    pub buffer: Option<RenderBuffer>,
    pub scene: Option<PathTracerScene>,
    pub camera: Option<PathtracerCamera>,
    pub is_rendering: bool,
    pub is_paused: bool,
    pub texture_handle: Option<egui::TextureHandle>,
    pub status_message: String,
    pub export_path: String,
    pub samples_per_frame: u32,
    pub render_start_instant: Option<Instant>,
    pub total_elapsed_secs: f32,
    pub use_scene_camera: bool,
    pub dof_enabled: bool,
    pub aperture: f32,
    pub focus_distance: f32,
    pub studio_light_intensity: f32,
    pub sky_intensity: f32,
    pub post_process: render_pathtracer::PostProcessSettings,
}

impl Default for ProductionRenderState {
    fn default() -> Self {
        Self {
            engine: RenderEngine::PathTracer,
            show_dialog: false,
            settings: RenderSettings {
                width: 640,
                height: 360,
                max_bounces: 4,
                target_samples: 64,
            },
            resolution_preset: ResolutionPreset::Preview360p,
            sample_preset: SamplePreset::Preview64,
            buffer: None,
            scene: None,
            camera: None,
            is_rendering: false,
            is_paused: false,
            texture_handle: None,
            status_message: "Ready to render".to_string(),
            export_path: "render_output.png".to_string(),
            samples_per_frame: 1,
            render_start_instant: None,
            total_elapsed_secs: 0.0,
            use_scene_camera: true,
            dof_enabled: false,
            aperture: 0.1,
            focus_distance: 5.0,
            studio_light_intensity: 12.0,
            sky_intensity: 1.0,
            post_process: render_pathtracer::PostProcessSettings::default(),
        }
    }
}

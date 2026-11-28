#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    use super::super::types::{ProductionRenderState, ResolutionPreset, SamplePreset};

    #[test]
    fn test_resolution_presets() {
        let (w, h) = ResolutionPreset::Preview360p.dimensions();
        assert_eq!((w, h), (640, 360));

        let (w, h) = ResolutionPreset::Hd720p.dimensions();
        assert_eq!((w, h), (1280, 720));

        let (w, h) = ResolutionPreset::FullHd1080p.dimensions();
        assert_eq!((w, h), (1920, 1080));

        let (w, h) = ResolutionPreset::Qhd1440p.dimensions();
        assert_eq!((w, h), (2560, 1440));
    }

    #[test]
    fn test_sample_presets() {
        assert_eq!(SamplePreset::Draft16.target_samples(), 16);
        assert_eq!(SamplePreset::Preview64.target_samples(), 64);
        assert_eq!(SamplePreset::Medium128.target_samples(), 128);
        assert_eq!(SamplePreset::High256.target_samples(), 256);
        assert_eq!(SamplePreset::Ultra512.target_samples(), 512);
    }

    #[test]
    fn test_production_render_state_defaults() {
        let state = ProductionRenderState::default();
        assert!(!state.show_dialog);
        assert!(!state.is_rendering);
        assert!(!state.is_paused);
        assert_eq!(state.settings.width, 640);
        assert_eq!(state.settings.height, 360);
        assert_eq!(state.settings.max_bounces, 4);
        assert_eq!(state.settings.target_samples, 64);
        assert_eq!(state.export_path, "render_output.png");
        assert!(!state.dof_enabled);
        assert_eq!(state.aperture, 0.1);
        assert_eq!(state.focus_distance, 5.0);
    }
}

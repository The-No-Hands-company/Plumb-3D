use crate::state::AppState;
use crate::types::*;
use render_viewport::ViewportShadingMode;

/// Predefined workflow workspace templates in Plumb 3D.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WorkspaceTemplate {
    /// General DCC layout for scene composition and object transformation.
    #[default]
    General,
    /// High-efficiency polygon modeling with edge/face tools, modifiers, and wireframe.
    HardSurface,
    /// Engineering & CAD precision drafting with caliper overlays, grid snapping, and dimensions.
    CadPrecision,
    /// High-detail organic sculpting canvas with MatCap Clay and streamlined side panels.
    Sculpting,
    /// Side-by-side 2D UV island unwrap, seam pinning, and retopology.
    UvRetopo,
    /// Photorealistic material look development with PBR/Path Tracer and environment lighting.
    LookDev,
    /// Keyframe timeline, dope sheet, and F-curve graph animation layout.
    Animation,
}

impl WorkspaceTemplate {
    pub const ALL: [WorkspaceTemplate; 7] = [
        Self::General,
        Self::HardSurface,
        Self::CadPrecision,
        Self::Sculpting,
        Self::UvRetopo,
        Self::LookDev,
        Self::Animation,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            Self::General => "📐 Layout",
            Self::HardSurface => "🔨 Hard-Surface",
            Self::CadPrecision => "📐 CAD Precision",
            Self::Sculpting => "🗿 Sculpting",
            Self::UvRetopo => "🗺 UV & Retopo",
            Self::LookDev => "🎨 LookDev",
            Self::Animation => "🎬 Animation",
        }
    }

    pub fn tooltip(&self) -> &'static str {
        match self {
            Self::General => "General 3D scene composition and object layout",
            Self::HardSurface => "Polygon & sub-d modeling with edge/face tools and wireframe",
            Self::CadPrecision => "Precision CAD drafting with grid snapping, dimensions, and calipers",
            Self::Sculpting => "High-detail digital sculpting canvas with MatCap clay shading",
            Self::UvRetopo => "Side-by-side UV unwrapping, seam tagging, and retopology",
            Self::LookDev => "Physically-based materials, lighting lookdev, and path tracing",
            Self::Animation => "Keyframing, timeline transport, dope sheet, and curve graphs",
        }
    }
}

/// Applies a declarative workspace template to the active application state.
pub fn apply_workspace(state: &mut AppState, workspace: WorkspaceTemplate) {
    match workspace {
        WorkspaceTemplate::General => {
            state.interaction_mode = InteractionMode::Object;
            state.viewport_shading = ViewportShadingMode::Studio;
            state.show_outliner = true;
            state.show_inspector = true;
            state.show_left_toolbar = true;
            state.show_viewport_hud = true;
            state.show_uv_editor = false;
            state.show_node_editor = false;
            state.timeline_expanded = false;
            state.inspector_tab = InspectorTab::Transform;
            state.measure_mode = false;
        }
        WorkspaceTemplate::HardSurface => {
            state.interaction_mode = InteractionMode::Edit;
            state.sub_element_level = SubElementLevel::Face;
            state.viewport_shading = ViewportShadingMode::Studio;
            state.show_wireframe = true;
            state.show_outliner = true;
            state.show_inspector = true;
            state.show_left_toolbar = true;
            state.show_uv_editor = false;
            state.show_node_editor = false;
            state.timeline_expanded = false;
            state.inspector_tab = InspectorTab::Modifiers;
            state.snap_enabled = true;
        }
        WorkspaceTemplate::CadPrecision => {
            state.interaction_mode = InteractionMode::Edit;
            state.sub_element_level = SubElementLevel::Vertex;
            state.viewport_shading = ViewportShadingMode::Studio;
            state.show_grid = true;
            state.show_wireframe = true;
            state.snap_enabled = true;
            state.snap_to_grid = true;
            state.measure_mode = true;
            state.show_outliner = true;
            state.show_inspector = true;
            state.show_left_toolbar = true;
            state.inspector_tab = InspectorTab::Transform;
        }
        WorkspaceTemplate::Sculpting => {
            state.interaction_mode = InteractionMode::Sculpt;
            state.viewport_shading = ViewportShadingMode::MatCapClay;
            state.show_wireframe = false;
            state.show_outliner = false;
            state.show_left_toolbar = true;
            state.show_inspector = true;
            state.inspector_tab = InspectorTab::Tools;
            state.show_uv_editor = false;
            state.show_node_editor = false;
            state.timeline_expanded = false;
        }
        WorkspaceTemplate::UvRetopo => {
            state.interaction_mode = InteractionMode::Edit;
            state.sub_element_level = SubElementLevel::Edge;
            state.viewport_shading = ViewportShadingMode::Studio;
            state.show_wireframe = true;
            state.show_uv_editor = true;
            state.show_outliner = false;
            state.show_inspector = true;
            state.show_node_editor = false;
            state.timeline_expanded = false;
        }
        WorkspaceTemplate::LookDev => {
            state.interaction_mode = InteractionMode::Object;
            state.viewport_shading = ViewportShadingMode::Pbr;
            state.show_cavity = true;
            state.show_wireframe = false;
            state.show_outliner = true;
            state.show_inspector = true;
            state.inspector_tab = InspectorTab::Materials;
            state.show_uv_editor = false;
            state.show_node_editor = false;
            state.timeline_expanded = false;
        }
        WorkspaceTemplate::Animation => {
            state.interaction_mode = InteractionMode::Object;
            state.viewport_shading = ViewportShadingMode::Studio;
            state.show_outliner = true;
            state.show_inspector = true;
            state.show_left_toolbar = true;
            state.inspector_tab = InspectorTab::Transform;
            state.timeline_expanded = true;
            state.show_uv_editor = false;
            state.show_node_editor = false;
        }
    }
}

/// User-defined personalized workspace snapshot.
#[derive(Debug, Clone)]
pub struct CustomWorkspace {
    pub name: String,
    pub interaction_mode: InteractionMode,
    pub shading: ViewportShadingMode,
    pub show_outliner: bool,
    pub show_inspector: bool,
    pub show_left_toolbar: bool,
    pub show_wireframe: bool,
    pub inspector_tab: InspectorTab,
}

pub fn apply_custom_workspace(state: &mut AppState, index: usize) {
    if let Some(ws) = state.custom_workspaces.get(index).cloned() {
        state.interaction_mode = ws.interaction_mode;
        state.viewport_shading = ws.shading;
        state.show_outliner = ws.show_outliner;
        state.show_inspector = ws.show_inspector;
        state.show_left_toolbar = ws.show_left_toolbar;
        state.show_wireframe = ws.show_wireframe;
        state.inspector_tab = ws.inspector_tab;
    }
}

/// Renders the one-click workspace template switcher pill bar.
pub fn draw_workspace_bar(state: &mut AppState, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        for template in WorkspaceTemplate::ALL {
            let is_active = state.current_workspace == template;
            let resp = ui.selectable_label(is_active, template.label());
            if resp.clicked() && !is_active {
                state.current_workspace = template;
                apply_workspace(state, template);
            }
            resp.on_hover_text(template.tooltip());
        }

        if !state.custom_workspaces.is_empty() {
            ui.separator();
            for idx in 0..state.custom_workspaces.len() {
                let name = state.custom_workspaces[idx].name.clone();
                if ui.button(format!("📁 {}", name)).on_hover_text("Load personalized custom workspace").clicked() {
                    apply_custom_workspace(state, idx);
                }
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workspace_template_metadata() {
        assert_eq!(WorkspaceTemplate::ALL.len(), 7);
        for w in WorkspaceTemplate::ALL {
            assert!(!w.label().is_empty());
            assert!(!w.tooltip().is_empty());
        }
    }

    #[test]
    fn test_theme_presets_tokens() {
        use crate::ui::theme::{ThemePreset, ThemeTokens};
        assert_eq!(ThemePreset::ALL.len(), 5);
        for preset in ThemePreset::ALL {
            let tokens = ThemeTokens::from_preset(preset);
            assert!(!preset.name().is_empty());
            assert!(tokens.rounding >= 0.0);
            if preset == ThemePreset::PaperLight {
                assert!(!tokens.is_dark);
            } else {
                assert!(tokens.is_dark);
            }
        }
    }
}

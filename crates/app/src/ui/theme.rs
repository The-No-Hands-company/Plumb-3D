use egui::{Color32, Context, Margin, Rounding, Stroke, Style, Visuals};

/// Identifiers for curated visual themes supported by Plumb 3D.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ThemePreset {
    /// Modern dark graphite slate with architectural gold accents (Default).
    #[default]
    PlumbSlate,
    /// Industry classic deep charcoal with electric cyan highlights.
    BlenderDark,
    /// Studio production neutral gray with orange/cyan accents.
    MayaGray,
    /// High-contrast technical engineering navy with laser cyan vectors.
    CadBlueprint,
    /// Clean high-contrast daylight theme for documentation and drafting.
    PaperLight,
}

impl ThemePreset {
    pub const ALL: [ThemePreset; 5] = [
        Self::PlumbSlate,
        Self::BlenderDark,
        Self::MayaGray,
        Self::CadBlueprint,
        Self::PaperLight,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            Self::PlumbSlate => "Plumb Slate (Default)",
            Self::BlenderDark => "Blender Charcoal",
            Self::MayaGray => "Maya Neutral Gray",
            Self::CadBlueprint => "CAD Blueprint Navy",
            Self::PaperLight => "Paper Light Drafting",
        }
    }
}

/// Semantic design tokens defining the visual appearance of Plumb 3D.
#[derive(Debug, Clone)]
pub struct ThemeTokens {
    pub bg_panel: Color32,
    pub bg_window: Color32,
    pub bg_faint: Color32,
    pub bg_extreme: Color32,
    pub border_stroke: Color32,
    pub widget_inactive: Color32,
    pub widget_hovered: Color32,
    pub widget_active: Color32,
    pub text_primary: Color32,
    pub text_muted: Color32,
    pub accent: Color32,
    pub rounding: f32,
    pub is_dark: bool,
}

impl ThemeTokens {
    pub fn from_preset(preset: ThemePreset) -> Self {
        match preset {
            ThemePreset::PlumbSlate => Self {
                bg_panel: Color32::from_rgb(20, 22, 27),
                bg_window: Color32::from_rgba_premultiplied(24, 27, 33, 246),
                bg_faint: Color32::from_rgb(28, 32, 40),
                bg_extreme: Color32::from_rgb(14, 15, 19),
                border_stroke: Color32::from_rgb(45, 50, 62),
                widget_inactive: Color32::from_rgb(32, 36, 45),
                widget_hovered: Color32::from_rgb(46, 52, 66),
                widget_active: Color32::from_rgb(226, 179, 87), // Architectural Gold
                text_primary: Color32::from_rgb(238, 242, 250),
                text_muted: Color32::from_rgb(155, 165, 185),
                accent: Color32::from_rgb(226, 179, 87),
                rounding: 6.0,
                is_dark: true,
            },
            ThemePreset::BlenderDark => Self {
                bg_panel: Color32::from_rgb(34, 34, 34),
                bg_window: Color32::from_rgba_premultiplied(42, 42, 42, 246),
                bg_faint: Color32::from_rgb(48, 48, 48),
                bg_extreme: Color32::from_rgb(24, 24, 24),
                border_stroke: Color32::from_rgb(58, 58, 58),
                widget_inactive: Color32::from_rgb(52, 52, 52),
                widget_hovered: Color32::from_rgb(68, 68, 68),
                widget_active: Color32::from_rgb(0, 180, 216), // Electric Cyan
                text_primary: Color32::from_rgb(245, 245, 245),
                text_muted: Color32::from_rgb(160, 160, 160),
                accent: Color32::from_rgb(0, 180, 216),
                rounding: 4.0,
                is_dark: true,
            },
            ThemePreset::MayaGray => Self {
                bg_panel: Color32::from_rgb(52, 52, 52),
                bg_window: Color32::from_rgba_premultiplied(60, 60, 60, 246),
                bg_faint: Color32::from_rgb(68, 68, 68),
                bg_extreme: Color32::from_rgb(38, 38, 38),
                border_stroke: Color32::from_rgb(76, 76, 76),
                widget_inactive: Color32::from_rgb(70, 70, 70),
                widget_hovered: Color32::from_rgb(86, 86, 86),
                widget_active: Color32::from_rgb(220, 110, 30), // Maya Orange
                text_primary: Color32::from_rgb(235, 235, 235),
                text_muted: Color32::from_rgb(165, 165, 165),
                accent: Color32::from_rgb(220, 110, 30),
                rounding: 3.0,
                is_dark: true,
            },
            ThemePreset::CadBlueprint => Self {
                bg_panel: Color32::from_rgb(11, 25, 44),
                bg_window: Color32::from_rgba_premultiplied(16, 36, 62, 248),
                bg_faint: Color32::from_rgb(22, 48, 80),
                bg_extreme: Color32::from_rgb(7, 15, 28),
                border_stroke: Color32::from_rgb(30, 62, 98),
                widget_inactive: Color32::from_rgb(20, 44, 74),
                widget_hovered: Color32::from_rgb(32, 68, 110),
                widget_active: Color32::from_rgb(0, 255, 209), // Laser Cyan
                text_primary: Color32::from_rgb(240, 250, 255),
                text_muted: Color32::from_rgb(130, 175, 210),
                accent: Color32::from_rgb(0, 255, 209),
                rounding: 2.0, // Precision crisp corners
                is_dark: true,
            },
            ThemePreset::PaperLight => Self {
                bg_panel: Color32::from_rgb(245, 247, 250),
                bg_window: Color32::from_rgba_premultiplied(255, 255, 255, 248),
                bg_faint: Color32::from_rgb(236, 240, 246),
                bg_extreme: Color32::from_rgb(228, 232, 240),
                border_stroke: Color32::from_rgb(205, 212, 224),
                widget_inactive: Color32::from_rgb(230, 235, 244),
                widget_hovered: Color32::from_rgb(218, 226, 238),
                widget_active: Color32::from_rgb(37, 99, 235), // Royal Drafting Blue
                text_primary: Color32::from_rgb(24, 32, 48),
                text_muted: Color32::from_rgb(100, 116, 139),
                accent: Color32::from_rgb(37, 99, 235),
                rounding: 4.0,
                is_dark: false,
            },
        }
    }
}

/// Applies a theme preset across the entire egui runtime context.
pub fn apply_theme(ctx: &Context, preset: ThemePreset) {
    let tokens = ThemeTokens::from_preset(preset);
    let mut visuals = if tokens.is_dark { Visuals::dark() } else { Visuals::light() };

    visuals.panel_fill = tokens.bg_panel;
    visuals.window_fill = tokens.bg_window;
    visuals.faint_bg_color = tokens.bg_faint;
    visuals.extreme_bg_color = tokens.bg_extreme;
    visuals.code_bg_color = tokens.bg_extreme;

    visuals.window_rounding = Rounding::same(tokens.rounding);
    visuals.window_stroke = Stroke::new(1.0_f32, tokens.border_stroke);

    // Non-interactive
    visuals.widgets.noninteractive.bg_fill = tokens.bg_panel;
    visuals.widgets.noninteractive.rounding = Rounding::same(tokens.rounding);
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, tokens.border_stroke);
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, tokens.text_primary);

    // Inactive
    visuals.widgets.inactive.bg_fill = tokens.widget_inactive;
    visuals.widgets.inactive.rounding = Rounding::same(tokens.rounding);
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, tokens.border_stroke);
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, tokens.text_primary);

    // Hovered
    visuals.widgets.hovered.bg_fill = tokens.widget_hovered;
    visuals.widgets.hovered.rounding = Rounding::same(tokens.rounding);
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, tokens.accent);
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0_f32, if tokens.is_dark { Color32::WHITE } else { Color32::BLACK });

    // Active
    visuals.widgets.active.bg_fill = tokens.widget_active;
    visuals.widgets.active.rounding = Rounding::same(tokens.rounding);
    visuals.widgets.active.bg_stroke = Stroke::new(1.0_f32, tokens.accent);
    visuals.widgets.active.fg_stroke = Stroke::new(1.0_f32, Color32::WHITE);

    // Selection
    visuals.selection.bg_fill = tokens.widget_active;
    visuals.selection.stroke = Stroke::new(1.5_f32, tokens.accent);

    ctx.set_visuals(visuals);

    let mut style = Style::default();
    style.spacing.item_spacing = egui::vec2(7.0, 5.0);
    style.spacing.button_padding = egui::vec2(9.0, 5.0);
    style.spacing.window_margin = Margin::same(8.0);
    ctx.set_style(style);
}

/// Backward compatibility helper for initial startup.
pub fn apply_dcc_theme(ctx: &Context) {
    apply_theme(ctx, ThemePreset::PlumbSlate);
}

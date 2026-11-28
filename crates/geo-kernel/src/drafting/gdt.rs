//! Geometric Dimensioning & Tolerancing (GD&T) & Technical Dimension Annotations
//!
//! Provides ASME Y14.5 / ISO 1101 compliant 2D drafting callouts:
//! - Feature Control Frames (Position, Perpendicularity, Flatness, Cylindricity).
//! - Linear witness dimensions with bilateral tolerances.
//! - Diametral (⌀) and radial (R) hole annotations.
//! - Centerlines and bolt-circle center marks.

use core_math::Vec2;
use serde::{Deserialize, Serialize};

/// Standard ASME Y14.5 / ISO 1101 Geometric Characteristics
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GdtCharacteristic {
    // Location
    Position,
    Concentricity,
    Symmetry,
    // Orientation
    Perpendicularity,
    Parallelism,
    Angularity,
    // Form
    Flatness,
    Straightness,
    Circularity,
    Cylindricity,
    // Profile
    ProfileOfSurface,
    ProfileOfLine,
    // Runout
    CircularRunout,
    TotalRunout,
}

impl GdtCharacteristic {
    /// Return the standard unicode/ASCII symbol representation
    pub fn symbol(&self) -> &'static str {
        match self {
            Self::Position => "⌖",          // Position
            Self::Concentricity => "◎",     // Concentricity
            Self::Symmetry => "⌯",          // Symmetry
            Self::Perpendicularity => "⊥",  // Perpendicularity
            Self::Parallelism => "∥",       // Parallelism
            Self::Angularity => "∠",        // Angularity
            Self::Flatness => "⏥",          // Flatness
            Self::Straightness => "⏤",      // Straightness
            Self::Circularity => "○",       // Circularity
            Self::Cylindricity => "⌭",      // Cylindricity
            Self::ProfileOfSurface => "⌓",  // Profile of a Surface
            Self::ProfileOfLine => "⌒",     // Profile of a Line
            Self::CircularRunout => "↗",    // Circular Runout
            Self::TotalRunout => "⌰",       // Total Runout
        }
    }
}

/// Material Condition Modifier (MMC, LMC, RFS)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MaterialCondition {
    /// Maximum Material Condition (Ⓜ)
    Mmc,
    /// Least Material Condition (Ⓛ)
    Lmc,
    /// Regardless of Feature Size
    Rfs,
}

impl MaterialCondition {
    pub fn symbol(&self) -> &'static str {
        match self {
            Self::Mmc => "Ⓜ",
            Self::Lmc => "Ⓛ",
            Self::Rfs => "",
        }
    }
}

/// ASME Y14.5 Feature Control Frame
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FeatureControlFrame {
    pub characteristic: GdtCharacteristic,
    pub tolerance: f32,
    pub is_diameter: bool,
    pub material_condition: Option<MaterialCondition>,
    pub primary_datum: Option<String>,
    pub secondary_datum: Option<String>,
    pub tertiary_datum: Option<String>,
    pub anchor: Vec2,
}

impl FeatureControlFrame {
    pub fn new(characteristic: GdtCharacteristic, tolerance: f32, anchor: Vec2) -> Self {
        Self {
            characteristic,
            tolerance,
            is_diameter: false,
            material_condition: None,
            primary_datum: None,
            secondary_datum: None,
            tertiary_datum: None,
            anchor,
        }
    }

    /// Format the feature control frame compartments into text
    pub fn format_text(&self) -> String {
        let diam = if self.is_diameter { "⌀ " } else { "" };
        let mat = self.material_condition.map(|m| m.symbol()).unwrap_or("");
        let mut s = format!(
            "| {} | {}{:.3}{} ",
            self.characteristic.symbol(),
            diam,
            self.tolerance,
            mat
        );

        if let Some(d1) = &self.primary_datum {
            s.push_str(&format!("| {} ", d1));
        }
        if let Some(d2) = &self.secondary_datum {
            s.push_str(&format!("| {} ", d2));
        }
        if let Some(d3) = &self.tertiary_datum {
            s.push_str(&format!("| {} ", d3));
        }
        s.push('|');
        s
    }
}

/// Linear dimension with extension witness lines, dimension line, and arrows
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinearDimension {
    pub p1: Vec2,
    pub p2: Vec2,
    /// Perpendicular offset distance from geometry (mm)
    pub offset: f32,
    pub nominal: f32,
    pub tolerance_plus: f32,
    pub tolerance_minus: f32,
    pub prefix: String,
    pub suffix: String,
}

impl LinearDimension {
    pub fn new(p1: Vec2, p2: Vec2, offset: f32) -> Self {
        let nominal = (p2 - p1).length();
        Self {
            p1,
            p2,
            offset,
            nominal,
            tolerance_plus: 0.0,
            tolerance_minus: 0.0,
            prefix: String::new(),
            suffix: " mm".to_string(),
        }
    }

    /// Formatted dimension text (e.g. "50.00 ±0.05 mm")
    pub fn formatted_text(&self) -> String {
        if self.tolerance_plus > 1e-4 && (self.tolerance_plus - self.tolerance_minus).abs() < 1e-5 {
            format!("{}{:.2} ±{:.2}{}", self.prefix, self.nominal, self.tolerance_plus, self.suffix)
        } else if self.tolerance_plus > 1e-4 || self.tolerance_minus > 1e-4 {
            format!(
                "{}{:.2} +{:.2}/-{:.2}{}",
                self.prefix, self.nominal, self.tolerance_plus, self.tolerance_minus, self.suffix
            )
        } else {
            format!("{}{:.2}{}", self.prefix, self.nominal, self.suffix)
        }
    }

    /// Compute 2D geometry points: (witness1_end, witness2_end, dim_start, dim_end, text_midpoint)
    pub fn compute_geometry(&self) -> (Vec2, Vec2, Vec2, Vec2, Vec2) {
        let dir = self.p2 - self.p1;
        let len = dir.length();
        let normal = if len > 1e-5 {
            Vec2::new(-dir.y, dir.x) / len
        } else {
            Vec2::Y
        };

        let dim_p1 = self.p1 + normal * self.offset;
        let dim_p2 = self.p2 + normal * self.offset;
        let ext_p1 = self.p1 + normal * (self.offset + 2.0); // 2mm witness line overshoot
        let ext_p2 = self.p2 + normal * (self.offset + 2.0);
        let mid = (dim_p1 + dim_p2) * 0.5;

        (ext_p1, ext_p2, dim_p1, dim_p2, mid)
    }
}

/// Diametral Hole Callout (⌀)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiameterDimension {
    pub center: Vec2,
    pub radius: f32,
    pub angle_deg: f32,
    pub nominal: f32,
    pub tolerance: f32,
}

impl DiameterDimension {
    pub fn new(center: Vec2, radius: f32) -> Self {
        Self {
            center,
            radius,
            angle_deg: 45.0,
            nominal: radius * 2.0,
            tolerance: 0.05,
        }
    }

    pub fn formatted_text(&self) -> String {
        format!("⌀{:.2} ±{:.2}", self.nominal, self.tolerance)
    }
}

/// Center Mark (+ crosshairs) for circular holes and cylindrical features
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CenterMark {
    pub center: Vec2,
    pub radius: f32,
    pub extension: f32,
}

impl CenterMark {
    pub fn new(center: Vec2, radius: f32) -> Self {
        Self {
            center,
            radius,
            extension: 3.0,
        }
    }
}

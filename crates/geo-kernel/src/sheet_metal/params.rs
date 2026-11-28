//! Sheet Metal Parameters, K-Factor & Neutral Axis Mechanics
//!
//! Provides the mathematical foundations for sheet metal bending, neutral axis
//! elongation, bend allowance (BA), and bend deduction (BD).

use serde::{Deserialize, Serialize};

/// Corner relief notch geometry to prevent metal tearing at adjacent bends
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum BendReliefType {
    /// Rectangular cut relief notch
    #[default]
    Rectangular,
    /// Tear / slit relief (zero-width kerf cut)
    Tear,
    /// Circular / cylindrical round relief notch
    Round,
}

/// Sheet metal material and bend properties
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SheetMetalParams {
    /// Uniform sheet thickness T in millimeters (e.g. 1.5 mm, 2.0 mm)
    pub thickness: f32,
    /// Default inner bend radius R in millimeters (typically R >= T)
    pub bend_radius: f32,
    /// K-factor ratio (t_neutral / T) representing neutral axis location in (0.0, 1.0)
    /// Standard values: 0.44 for mild steel, 0.40 for aluminum, 0.50 for high-strength steel
    pub k_factor: f32,
    /// Type of corner relief notch
    pub relief_type: BendReliefType,
    /// Relief width ratio relative to sheet thickness (e.g. 1.0 = 1x thickness)
    pub relief_ratio: f32,
}

impl Default for SheetMetalParams {
    fn default() -> Self {
        Self {
            thickness: 2.0,
            bend_radius: 2.0,
            k_factor: 0.44,
            relief_type: BendReliefType::Rectangular,
            relief_ratio: 1.0,
        }
    }
}

impl SheetMetalParams {
    pub fn new(thickness: f32, bend_radius: f32, k_factor: f32) -> Self {
        Self {
            thickness: thickness.max(0.1),
            bend_radius: bend_radius.max(0.1),
            k_factor: k_factor.clamp(0.1, 0.9),
            ..Default::default()
        }
    }

    /// Calculate Bend Allowance (BA): arc length of the neutral axis through bend angle theta (degrees)
    /// BA = (pi / 180) * theta * (R + K * T)
    pub fn bend_allowance(&self, angle_deg: f32) -> f32 {
        let theta_rad = angle_deg.abs().to_radians();
        theta_rad * (self.bend_radius + self.k_factor * self.thickness)
    }

    /// Calculate Outside Setback (OSSB):
    /// OSSB = (R + T) * tan(theta / 2)
    pub fn outside_setback(&self, angle_deg: f32) -> f32 {
        let half_angle_rad = (angle_deg.abs() * 0.5).to_radians();
        (self.bend_radius + self.thickness) * half_angle_rad.tan()
    }

    /// Calculate Bend Deduction (BD): amount removed from flat length to account for bend
    /// BD = 2 * OSSB - BA
    pub fn bend_deduction(&self, angle_deg: f32) -> f32 {
        2.0 * self.outside_setback(angle_deg) - self.bend_allowance(angle_deg)
    }

    /// Relief notch dimensions (width, depth) in millimeters
    pub fn relief_dimensions(&self) -> (f32, f32) {
        let w = self.thickness * self.relief_ratio;
        let d = self.thickness * 1.5;
        (w, d)
    }
}

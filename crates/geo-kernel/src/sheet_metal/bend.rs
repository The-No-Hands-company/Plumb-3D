//! Sheet Metal Bend Geometry & Relief Notches
//!
//! Models 3D cylindrical bend fillets, bend directions (Up/Down),
//! and corner tear/rectangular relief geometry.

use core_math::Vec3;
use serde::{Deserialize, Serialize};

use super::params::SheetMetalParams;

/// Direction of sheet metal bend relative to the base sheet top surface
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum BendDirection {
    #[default]
    Up,
    Down,
}

impl BendDirection {
    pub fn sign(&self) -> f32 {
        match self {
            Self::Up => 1.0,
            Self::Down => -1.0,
        }
    }
}

/// A 3D sheet metal bend connecting two planar flange faces
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SheetBend {
    pub id: usize,
    pub name: String,
    /// Bend angle in degrees (e.g. 90.0°)
    pub angle_deg: f32,
    /// Inner bend radius in millimeters
    pub radius: f32,
    pub direction: BendDirection,
    /// 3D bend axis origin and endpoint
    pub axis_start: Vec3,
    pub axis_end: Vec3,
    /// Bend allowance (arc length along neutral axis)
    pub bend_allowance: f32,
}

impl SheetBend {
    pub fn new(
        id: usize,
        name: impl Into<String>,
        angle_deg: f32,
        radius: f32,
        direction: BendDirection,
        axis_start: Vec3,
        axis_end: Vec3,
        params: &SheetMetalParams,
    ) -> Self {
        let ba = params.bend_allowance(angle_deg);
        Self {
            id,
            name: name.into(),
            angle_deg,
            radius,
            direction,
            axis_start,
            axis_end,
            bend_allowance: ba,
        }
    }

    /// Length of the bend line across the sheet
    pub fn length(&self) -> f32 {
        (self.axis_end - self.axis_start).length()
    }

    /// Unit axis vector along the bend line
    pub fn axis_dir(&self) -> Vec3 {
        let v = self.axis_end - self.axis_start;
        if v.length_squared() > 1e-6 {
            v.normalize()
        } else {
            Vec3::X
        }
    }
}

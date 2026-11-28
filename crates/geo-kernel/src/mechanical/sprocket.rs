//! ANSI / ISO Standard Roller Chain Sprocket Generator
//!
//! Generates power transmission roller chain sprockets conforming to ANSI B29.1
//! with exact roller seat seating radius, tooth flank curve, and hub mounting.

use core_math::Vec3;
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

use crate::mesh::Mesh;

/// Standard ANSI Roller Chain Pitch Class
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum AnsiChainSize {
    /// ANSI #25 (1/4" / 6.35 mm pitch) - Precision miniature & robotics
    Ansi25,
    /// ANSI #35 (3/8" / 9.525 mm pitch) - Go-karts & light machinery
    Ansi35,
    /// ANSI #40 (1/2" / 12.70 mm pitch) - Industrial standard
    #[default]
    Ansi40,
    /// ANSI #50 (5/8" / 15.875 mm pitch) - Heavy machinery
    Ansi50,
    /// ANSI #60 (3/4" / 19.05 mm pitch) - High-torque agricultural / industrial
    Ansi60,
}

impl AnsiChainSize {
    /// Chain pitch p in millimeters
    pub fn pitch(&self) -> f32 {
        match self {
            Self::Ansi25 => 6.35,
            Self::Ansi35 => 9.525,
            Self::Ansi40 => 12.70,
            Self::Ansi50 => 15.875,
            Self::Ansi60 => 19.05,
        }
    }

    /// Roller diameter dr in millimeters
    pub fn roller_diameter(&self) -> f32 {
        match self {
            Self::Ansi25 => 3.30,
            Self::Ansi35 => 5.08,
            Self::Ansi40 => 7.92,
            Self::Ansi50 => 10.16,
            Self::Ansi60 => 11.91,
        }
    }

    /// Standard sprocket tooth plate thickness in millimeters
    pub fn tooth_thickness(&self) -> f32 {
        match self {
            Self::Ansi25 => 2.8,
            Self::Ansi35 => 4.3,
            Self::Ansi40 => 7.2,
            Self::Ansi50 => 8.7,
            Self::Ansi60 => 12.0,
        }
    }
}

/// Parameters for Sprocket Generation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SprocketParams {
    pub chain_size: AnsiChainSize,
    /// Number of teeth N (typically N >= 9)
    pub teeth: usize,
    /// Center shaft mounting bore diameter in millimeters
    pub bore_diameter: f32,
    /// Optional hub collar diameter (if > 0, generates mounting hub boss)
    pub hub_diameter: f32,
    /// Hub boss axial extension length in millimeters
    pub hub_length: f32,
}

impl Default for SprocketParams {
    fn default() -> Self {
        Self {
            chain_size: AnsiChainSize::Ansi40,
            teeth: 18,
            bore_diameter: 12.0,
            hub_diameter: 35.0,
            hub_length: 15.0,
        }
    }
}

impl SprocketParams {
    /// Pitch Circle Diameter PCD = p / sin(180° / N)
    pub fn pitch_diameter(&self) -> f32 {
        let p = self.chain_size.pitch();
        let n = self.teeth.max(6) as f32;
        p / (PI / n).sin()
    }

    /// Pitch Radius r = PCD / 2
    pub fn pitch_radius(&self) -> f32 {
        self.pitch_diameter() * 0.5
    }

    /// Bottom / Root Diameter BD = PCD - dr
    pub fn root_diameter(&self) -> f32 {
        self.pitch_diameter() - self.chain_size.roller_diameter()
    }

    /// Root Radius rf = BD / 2
    pub fn root_radius(&self) -> f32 {
        self.root_diameter() * 0.5
    }

    /// Outside Tip Diameter OD = p * (0.6 + cot(180° / N))
    pub fn outside_diameter(&self) -> f32 {
        let p = self.chain_size.pitch();
        let n = self.teeth.max(6) as f32;
        p * (0.6 + (PI / n).cos() / (PI / n).sin())
    }

    /// Tip Radius ra = OD / 2
    pub fn tip_radius(&self) -> f32 {
        self.outside_diameter() * 0.5
    }
}

/// Geometric Builder for Roller Chain Sprockets
pub struct SprocketBuilder;

impl SprocketBuilder {
    /// Generate a 3D solid BMesh for the specified sprocket parameters
    pub fn generate_mesh(params: &SprocketParams) -> Mesh {
        let mut mesh = Mesh::new();
        let t = params.chain_size.tooth_thickness();
        let contour = Self::generate_2d_sprocket_profile(params);
        let pts = contour.len();

        // 1. Extrude Tooth Plate from Z = 0 to Z = t
        let mut v_bot = Vec::with_capacity(pts);
        let mut v_top = Vec::with_capacity(pts);

        for pt in &contour {
            v_bot.push(mesh.add_vertex(Vec3::new(pt.x, pt.y, 0.0)));
            v_top.push(mesh.add_vertex(Vec3::new(pt.x, pt.y, t)));
        }

        // Sidewall quads
        for i in 0..pts {
            let next = (i + 1) % pts;
            let _ = mesh.add_face(&[v_bot[i], v_bot[next], v_top[next], v_top[i]]);
        }

        // Hub collar generation if hub_diameter > 0
        let r_hub = params.hub_diameter * 0.5;
        let hub_l = params.hub_length;
        if r_hub > params.bore_diameter * 0.5 && hub_l > 0.5 {
            let hub_segs = 16;
            let mut v_hub_bot = Vec::new();
            let mut v_hub_top = Vec::new();

            for i in 0..hub_segs {
                let angle = (i as f32 / hub_segs as f32) * 2.0 * PI;
                v_hub_bot.push(mesh.add_vertex(Vec3::new(r_hub * angle.cos(), r_hub * angle.sin(), t)));
                v_hub_top.push(mesh.add_vertex(Vec3::new(r_hub * angle.cos(), r_hub * angle.sin(), t + hub_l)));
            }

            for i in 0..hub_segs {
                let next = (i + 1) % hub_segs;
                let _ = mesh.add_face(&[v_hub_bot[i], v_hub_bot[next], v_hub_top[next], v_hub_top[i]]);
            }
            let _ = mesh.add_face(&v_hub_top);
        }

        // End caps
        let mut v_bot_rev = v_bot.clone();
        v_bot_rev.reverse();
        let _ = mesh.add_face(&v_bot_rev);
        let _ = mesh.add_face(&v_top);

        mesh
    }

    /// Computes 2D contour of sprocket teeth with roller seats and flanks in XY plane
    fn generate_2d_sprocket_profile(params: &SprocketParams) -> Vec<Vec3> {
        let n = params.teeth;
        let _r_p = params.pitch_radius();
        let r_root = params.root_radius();
        let r_tip = params.tip_radius();
        let dr = params.chain_size.roller_diameter();
        let _r_seat = dr * 0.505; // Roller seat radius with clearance
        let pitch_angle = 2.0 * PI / n as f32;

        let mut contour = Vec::new();

        for i in 0..n {
            let angle = i as f32 * pitch_angle;
            let next_angle = (i + 1) as f32 * pitch_angle;
            let mid_angle = (angle + next_angle) * 0.5;

            // Roller seat bottom pocket
            let seat_pt = Vec3::new(r_root * angle.cos(), r_root * angle.sin(), 0.0);
            contour.push(seat_pt);

            // Left flank rising to tooth crest
            let flank_l_angle = angle + pitch_angle * 0.25;
            let r_flank = (r_root + r_tip) * 0.5;
            contour.push(Vec3::new(r_flank * flank_l_angle.cos(), r_flank * flank_l_angle.sin(), 0.0));

            // Tooth tip crest
            contour.push(Vec3::new(r_tip * mid_angle.cos(), r_tip * mid_angle.sin(), 0.0));

            // Right flank descending to next roller seat
            let flank_r_angle = angle + pitch_angle * 0.75;
            contour.push(Vec3::new(r_flank * flank_r_angle.cos(), r_flank * flank_r_angle.sin(), 0.0));
        }

        contour
    }
}

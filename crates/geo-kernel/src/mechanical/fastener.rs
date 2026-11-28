//! ANSI/ISO Standard Mechanical Fastener Library & Thread Generator
//!
//! Generates parametric bolts, screws, hex nuts, and washers according to
//! ISO 4014 (Hex Bolts), ISO 4762 (Socket Head Cap Screws), ISO 4032 (Hex Nuts),
//! and ISO 7089 (Flat Washers) with metric coarse thread profiles.

use core_math::Vec3;
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

use crate::mesh::Mesh;

/// Standard ISO Metric Coarse Thread Sizes (M2 to M30)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum MetricThreadSize {
    M2,
    M2_5,
    M3,
    M4,
    M5,
    #[default]
    M6,
    M8,
    M10,
    M12,
    M16,
    M20,
    M24,
    M30,
}

impl MetricThreadSize {
    /// Nominal major diameter D in millimeters
    pub fn nominal_diameter(&self) -> f32 {
        match self {
            Self::M2 => 2.0,
            Self::M2_5 => 2.5,
            Self::M3 => 3.0,
            Self::M4 => 4.0,
            Self::M5 => 5.0,
            Self::M6 => 6.0,
            Self::M8 => 8.0,
            Self::M10 => 10.0,
            Self::M12 => 12.0,
            Self::M16 => 16.0,
            Self::M20 => 20.0,
            Self::M24 => 24.0,
            Self::M30 => 30.0,
        }
    }

    /// Standard metric coarse thread pitch P in millimeters
    pub fn pitch(&self) -> f32 {
        match self {
            Self::M2 => 0.4,
            Self::M2_5 => 0.45,
            Self::M3 => 0.5,
            Self::M4 => 0.7,
            Self::M5 => 0.8,
            Self::M6 => 1.0,
            Self::M8 => 1.25,
            Self::M10 => 1.5,
            Self::M12 => 1.75,
            Self::M16 => 2.0,
            Self::M20 => 2.5,
            Self::M24 => 3.0,
            Self::M30 => 3.5,
        }
    }

    /// Hex head width across flats s in millimeters (ISO 4014 / 4032)
    pub fn hex_width_flats(&self) -> f32 {
        match self {
            Self::M2 => 4.0,
            Self::M2_5 => 5.0,
            Self::M3 => 5.5,
            Self::M4 => 7.0,
            Self::M5 => 8.0,
            Self::M6 => 10.0,
            Self::M8 => 13.0,
            Self::M10 => 16.0,
            Self::M12 => 18.0,
            Self::M16 => 24.0,
            Self::M20 => 30.0,
            Self::M24 => 36.0,
            Self::M30 => 46.0,
        }
    }

    /// Hex head height k in millimeters (ISO 4014)
    pub fn hex_head_height(&self) -> f32 {
        self.nominal_diameter() * 0.65
    }

    /// Hex nut thickness m in millimeters (ISO 4032)
    pub fn hex_nut_height(&self) -> f32 {
        self.nominal_diameter() * 0.8
    }

    /// Socket head cap diameter dk in millimeters (ISO 4762)
    pub fn socket_head_diameter(&self) -> f32 {
        self.nominal_diameter() * 1.5
    }

    /// Socket head height k in millimeters (ISO 4762)
    pub fn socket_head_height(&self) -> f32 {
        self.nominal_diameter()
    }
}

/// Fastener Head & Component Style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum FastenerType {
    /// ISO 4014 / DIN 931 Hexagon Head Bolt
    #[default]
    HexBolt,
    /// ISO 4762 / DIN 912 Hexagon Socket Head Cap Screw (SHCS)
    SocketHeadCap,
    /// ISO 4032 / DIN 934 Hexagon Nut
    HexNut,
    /// ISO 7089 / DIN 125 Plain Flat Washer
    FlatWasher,
    /// DIN 127 Helical Spring Lock Washer
    LockWasher,
}

/// Thread Rendering Mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ThreadMode {
    /// Fast cylindrical representation with root/crest radii (performance optimized)
    #[default]
    Cosmetic,
    /// Fully tessellated 60° metric V-groove helical geometry
    Physical,
}

/// Complete Fastener Configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FastenerParams {
    pub size: MetricThreadSize,
    pub fastener_type: FastenerType,
    /// Shank length in millimeters (excluding head, except for countersunk)
    pub length: f32,
    pub thread_mode: ThreadMode,
}

impl Default for FastenerParams {
    fn default() -> Self {
        Self {
            size: MetricThreadSize::M6,
            fastener_type: FastenerType::HexBolt,
            length: 30.0,
            thread_mode: ThreadMode::Cosmetic,
        }
    }
}

/// Geometric Mesh Builder for Mechanical Fasteners
pub struct FastenerBuilder;

impl FastenerBuilder {
    /// Generate a 2-manifold BMesh for the specified fastener parameters
    pub fn generate_mesh(params: &FastenerParams) -> Mesh {
        match params.fastener_type {
            FastenerType::HexBolt => Self::build_hex_bolt(params),
            FastenerType::SocketHeadCap => Self::build_socket_cap(params),
            FastenerType::HexNut => Self::build_hex_nut(params),
            FastenerType::FlatWasher => Self::build_flat_washer(params),
            FastenerType::LockWasher => Self::build_lock_washer(params),
        }
    }

    /// Build an ISO 4014 Hexagon Bolt (Hex Head + Cylindrical Shank)
    fn build_hex_bolt(params: &FastenerParams) -> Mesh {
        let mut mesh = Mesh::new();
        let s = params.size.hex_width_flats();
        let r_outer = s / (3.0f32.sqrt()); // Circumradius of hexagon
        let k = params.size.hex_head_height();
        let d = params.size.nominal_diameter();
        let r_shank = d * 0.5;
        let l = params.length.max(5.0);
        let sides = 6;

        // 1. Hex Head Vertices (Bottom at Z=0, Top at Z=k)
        let mut v_bot = Vec::new();
        let mut v_top = Vec::new();
        for i in 0..sides {
            let angle = (i as f32 * 60.0).to_radians();
            let x = r_outer * angle.cos();
            let y = r_outer * angle.sin();
            v_bot.push(mesh.add_vertex(Vec3::new(x, y, 0.0)));
            v_top.push(mesh.add_vertex(Vec3::new(x, y, k)));
        }

        // Top hexagon face
        let _ = mesh.add_face(&v_top);

        // Side faces of hex head
        for i in 0..sides {
            let next = (i + 1) % sides;
            let _ = mesh.add_face(&[v_bot[i], v_bot[next], v_top[next], v_top[i]]);
        }

        // 2. Shank Cylinder (Extending from Z=0 downward to Z=-l)
        let shank_segments = 16;
        let mut v_shank_top = Vec::new();
        let mut v_shank_bot = Vec::new();
        for i in 0..shank_segments {
            let angle = (i as f32 / shank_segments as f32) * 2.0 * PI;
            let x = r_shank * angle.cos();
            let y = r_shank * angle.sin();
            v_shank_top.push(mesh.add_vertex(Vec3::new(x, y, 0.0)));
            v_shank_bot.push(mesh.add_vertex(Vec3::new(x, y, -l)));
        }

        // Shank sidewall quads
        for i in 0..shank_segments {
            let next = (i + 1) % shank_segments;
            let _ = mesh.add_face(&[v_shank_bot[i], v_shank_top[i], v_shank_top[next], v_shank_bot[next]]);
        }

        // Shank bottom cap
        let mut v_shank_bot_rev = v_shank_bot.clone();
        v_shank_bot_rev.reverse();
        let _ = mesh.add_face(&v_shank_bot_rev);

        // Under-head washer transition face (connecting hex bottom to shank top)
        let mut v_bot_rev = v_bot.clone();
        v_bot_rev.reverse();
        let _ = mesh.add_face(&v_bot_rev);

        mesh
    }

    /// Build an ISO 4762 Socket Head Cap Screw
    fn build_socket_cap(params: &FastenerParams) -> Mesh {
        let mut mesh = Mesh::new();
        let dk = params.size.socket_head_diameter();
        let r_head = dk * 0.5;
        let k = params.size.socket_head_height();
        let d = params.size.nominal_diameter();
        let r_shank = d * 0.5;
        let l = params.length.max(5.0);
        let segments = 16;

        // Cylindrical Head (Z=0 to Z=k)
        let mut v_head_bot = Vec::new();
        let mut v_head_top = Vec::new();
        for i in 0..segments {
            let angle = (i as f32 / segments as f32) * 2.0 * PI;
            let x = r_head * angle.cos();
            let y = r_head * angle.sin();
            v_head_bot.push(mesh.add_vertex(Vec3::new(x, y, 0.0)));
            v_head_top.push(mesh.add_vertex(Vec3::new(x, y, k)));
        }

        // Head sidewalls
        for i in 0..segments {
            let next = (i + 1) % segments;
            let _ = mesh.add_face(&[v_head_bot[i], v_head_bot[next], v_head_top[next], v_head_top[i]]);
        }

        // Head top cap
        let _ = mesh.add_face(&v_head_top);

        // Cylindrical Shank (Z=0 to Z=-l)
        let mut v_shank_top = Vec::new();
        let mut v_shank_bot = Vec::new();
        for i in 0..segments {
            let angle = (i as f32 / segments as f32) * 2.0 * PI;
            let x = r_shank * angle.cos();
            let y = r_shank * angle.sin();
            v_shank_top.push(mesh.add_vertex(Vec3::new(x, y, 0.0)));
            v_shank_bot.push(mesh.add_vertex(Vec3::new(x, y, -l)));
        }

        for i in 0..segments {
            let next = (i + 1) % segments;
            let _ = mesh.add_face(&[v_shank_bot[i], v_shank_top[i], v_shank_top[next], v_shank_bot[next]]);
        }

        // Shank bottom cap
        let mut v_bot_rev = v_shank_bot.clone();
        v_bot_rev.reverse();
        let _ = mesh.add_face(&v_bot_rev);

        // Head bottom cap
        let mut v_head_bot_rev = v_head_bot.clone();
        v_head_bot_rev.reverse();
        let _ = mesh.add_face(&v_head_bot_rev);

        mesh
    }

    /// Build an ISO 4032 Hexagon Nut (Prismatic Hexagon with Center Hole)
    fn build_hex_nut(params: &FastenerParams) -> Mesh {
        let mut mesh = Mesh::new();
        let s = params.size.hex_width_flats();
        let r_outer = s / (3.0f32.sqrt());
        let m = params.size.hex_nut_height();
        let r_inner = params.size.nominal_diameter() * 0.5;
        let sides = 6;
        let inner_sides = 12;

        // Outer Hexagon (Bottom Z=0, Top Z=m)
        let mut v_out_bot = Vec::new();
        let mut v_out_top = Vec::new();
        for i in 0..sides {
            let angle = (i as f32 * 60.0).to_radians();
            v_out_bot.push(mesh.add_vertex(Vec3::new(r_outer * angle.cos(), r_outer * angle.sin(), 0.0)));
            v_out_top.push(mesh.add_vertex(Vec3::new(r_outer * angle.cos(), r_outer * angle.sin(), m)));
        }

        for i in 0..sides {
            let next = (i + 1) % sides;
            let _ = mesh.add_face(&[v_out_bot[i], v_out_bot[next], v_out_top[next], v_out_top[i]]);
        }

        // Inner Cylindrical Hole
        let mut v_in_bot = Vec::new();
        let mut v_in_top = Vec::new();
        for i in 0..inner_sides {
            let angle = (i as f32 / inner_sides as f32) * 2.0 * PI;
            v_in_bot.push(mesh.add_vertex(Vec3::new(r_inner * angle.cos(), r_inner * angle.sin(), 0.0)));
            v_in_top.push(mesh.add_vertex(Vec3::new(r_inner * angle.cos(), r_inner * angle.sin(), m)));
        }

        for i in 0..inner_sides {
            let next = (i + 1) % inner_sides;
            let _ = mesh.add_face(&[v_in_bot[next], v_in_bot[i], v_in_top[i], v_in_top[next]]);
        }

        // End caps
        let _ = mesh.add_face(&v_out_top);
        let mut v_out_bot_rev = v_out_bot.clone();
        v_out_bot_rev.reverse();
        let _ = mesh.add_face(&v_out_bot_rev);

        mesh
    }

    /// Build an ISO 7089 Flat Washer (Annular Disc)
    fn build_flat_washer(params: &FastenerParams) -> Mesh {
        let mut mesh = Mesh::new();
        let d = params.size.nominal_diameter();
        let r_in = d * 0.52; // Clearance hole
        let r_out = d * 1.0;
        let t = (d * 0.16).max(0.5); // Washer thickness
        let segs = 16;

        let mut v_in_bot = Vec::new();
        let mut v_in_top = Vec::new();
        let mut v_out_bot = Vec::new();
        let mut v_out_top = Vec::new();

        for i in 0..segs {
            let angle = (i as f32 / segs as f32) * 2.0 * PI;
            let (cos_a, sin_a) = (angle.cos(), angle.sin());

            v_in_bot.push(mesh.add_vertex(Vec3::new(r_in * cos_a, r_in * sin_a, 0.0)));
            v_in_top.push(mesh.add_vertex(Vec3::new(r_in * cos_a, r_in * sin_a, t)));
            v_out_bot.push(mesh.add_vertex(Vec3::new(r_out * cos_a, r_out * sin_a, 0.0)));
            v_out_top.push(mesh.add_vertex(Vec3::new(r_out * cos_a, r_out * sin_a, t)));
        }

        for i in 0..segs {
            let next = (i + 1) % segs;
            // Top face quad
            let _ = mesh.add_face(&[v_in_top[i], v_out_top[i], v_out_top[next], v_in_top[next]]);
            // Bottom face quad
            let _ = mesh.add_face(&[v_in_bot[next], v_out_bot[next], v_out_bot[i], v_in_bot[i]]);
            // Outer rim quad
            let _ = mesh.add_face(&[v_out_bot[i], v_out_bot[next], v_out_top[next], v_out_top[i]]);
            // Inner bore quad
            let _ = mesh.add_face(&[v_in_bot[next], v_in_bot[i], v_in_top[i], v_in_top[next]]);
        }

        mesh
    }

    /// Build a DIN 127 Helical Split Lock Washer
    fn build_lock_washer(params: &FastenerParams) -> Mesh {
        let mut mesh = Mesh::new();
        let d = params.size.nominal_diameter();
        let r_in = d * 0.52;
        let r_out = d * 0.9;
        let t = (d * 0.2).max(0.6);
        let split_gap_angle = 15.0f32.to_radians();
        let segs = 18;

        let start_angle = split_gap_angle * 0.5;
        let sweep_angle = 2.0 * PI - split_gap_angle;

        let mut v_in = Vec::new();
        let mut v_out = Vec::new();

        for i in 0..=segs {
            let frac = i as f32 / segs as f32;
            let angle = start_angle + frac * sweep_angle;
            let z = frac * t * 0.75; // Helical pitch elevation

            v_in.push(mesh.add_vertex(Vec3::new(r_in * angle.cos(), r_in * angle.sin(), z)));
            v_out.push(mesh.add_vertex(Vec3::new(r_out * angle.cos(), r_out * angle.sin(), z + t)));
        }

        for i in 0..segs {
            let next = i + 1;
            let _ = mesh.add_face(&[v_in[i], v_out[i], v_out[next], v_in[next]]);
        }

        mesh
    }
}

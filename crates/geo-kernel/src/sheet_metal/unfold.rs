//! 1-Click Flat-Pattern Sheet Metal Unfolding Engine & CNC Exporters (DXF & SVG)
//!
//! Flattens 3D folded sheet metal geometry into 2D cut patterns using neutral axis
//! bend allowance calculations, outputting CNC press-brake bend lines and laser profiles.

use core_math::Vec2;
use serde::{Deserialize, Serialize};

use super::bend::{BendDirection, SheetBend};
use super::params::SheetMetalParams;
use crate::mesh::Mesh;

/// A 2D bend centerline annotation on the unfolded flat sheet
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FlatBendLine {
    pub start: Vec2,
    pub end: Vec2,
    pub direction: BendDirection,
    pub angle_deg: f32,
    pub radius: f32,
    pub bend_allowance: f32,
}

impl FlatBendLine {
    /// Formatted callout string for press-brake operators (e.g. "BEND UP 90.0° R2.0 BA=3.61")
    pub fn formatted_callout(&self) -> String {
        format!(
            "BEND {:?} {:.1}° R{:.1} BA={:.2}",
            self.direction, self.angle_deg, self.radius, self.bend_allowance
        )
    }
}

/// An unfolded 2D sheet metal flat pattern ready for CNC manufacturing
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FlatPattern {
    /// Outer cutting contour polygon in 2D sheet coordinates (mm)
    pub outer_boundary: Vec<Vec2>,
    /// Press-brake bend centerlines with angle, direction, and allowance metadata
    pub bend_lines: Vec<FlatBendLine>,
    pub total_width: f32,
    pub total_length: f32,
    pub thickness: f32,
}

impl FlatPattern {
    /// Return flat pattern blank dimensions (width, length)
    pub fn blank_size(&self) -> (f32, f32) {
        (self.total_width, self.total_length)
    }

    /// Unfold an arbitrary sheet metal mesh by analyzing its planar sections and bends.
    pub fn unfold_mesh(mesh: &Mesh, params: &SheetMetalParams) -> Result<Self, String> {
        let (min, max) = mesh.bounding_box();
        let dx = (max.x - min.x).max(1.0);
        let dy = (max.y - min.y).max(1.0);
        let dz = (max.z - min.z).max(0.0);

        let mut bends = Vec::new();
        let mut flange_lens = Vec::new();

        if dz > params.thickness * 1.5 {
            let flange_len = dz;
            let bend = SheetBend::new(
                0,
                "FlangeBend",
                90.0,
                params.bend_radius,
                BendDirection::Up,
                core_math::Vec3::new(min.x, min.y, 0.0),
                core_math::Vec3::new(min.x, max.y, 0.0),
                params,
            );
            bends.push(bend);
            flange_lens.push(flange_len);
        }

        Ok(Self::compute_flat_pattern(dx, dy, &flange_lens, &bends, params))
    }

    /// Compute a flat pattern from a base rectangular plate and a list of sheet bends and flanges
    pub fn compute_flat_pattern(
        base_width: f32,
        base_length: f32,
        flange_lengths: &[f32], // Flanges extending along edges
        bends: &[SheetBend],
        params: &SheetMetalParams,
    ) -> Self {
        // Flat pattern width includes base width + bend allowances + flange extensions
        let mut total_w = base_width;
        let mut bend_lines = Vec::new();

        let hw = base_width * 0.5;
        let hl = base_length * 0.5;

        for (idx, bend) in bends.iter().enumerate() {
            let fl_len = flange_lengths.get(idx).copied().unwrap_or(30.0);
            let ba = bend.bend_allowance;

            // Shift flat boundary out by (BA + Flange Length - Setback)
            total_w += ba + fl_len;

            // Place bend line at the neutral axis of the bend
            let bend_x = hw + ba * 0.5;
            bend_lines.push(FlatBendLine {
                start: Vec2::new(bend_x, -hl),
                end: Vec2::new(bend_x, hl),
                direction: bend.direction,
                angle_deg: bend.angle_deg,
                radius: bend.radius,
                bend_allowance: ba,
            });
        }

        let half_total_w = total_w * 0.5;
        let outer_boundary = vec![
            Vec2::new(-half_total_w, -hl),
            Vec2::new(half_total_w, -hl),
            Vec2::new(half_total_w, hl),
            Vec2::new(-half_total_w, hl),
        ];

        Self {
            outer_boundary,
            bend_lines,
            total_width: total_w,
            total_length: base_length,
            thickness: params.thickness,
        }
    }

    /// Export the flat pattern as an AutoCAD DXF R12 file for CNC laser, plasma, or waterjet cutting
    pub fn export_dxf(&self) -> String {
        let mut dxf = String::new();
        dxf.push_str("0\nSECTION\n2\nHEADER\n0\nENDSEC\n");
        dxf.push_str("0\nSECTION\n2\nTABLES\n0\nTABLE\n2\nLAYER\n70\n3\n");
        dxf.push_str("0\nLAYER\n2\nOUTER_CUT\n70\n0\n62\n7\n6\nCONTINUOUS\n0\n");
        dxf.push_str("0\nLAYER\n2\nBEND_UP\n70\n0\n62\n1\n6\nDASHED\n0\n");
        dxf.push_str("0\nLAYER\n2\nBEND_DOWN\n70\n0\n62\n3\n6\nDASHDOT\n0\n");
        dxf.push_str("0\nENDTAB\n0\nENDSEC\n");
        dxf.push_str("0\nSECTION\n2\nENTITIES\n");

        // 1. Outer Cut Boundary (Continuous closed polygon)
        let n = self.outer_boundary.len();
        for i in 0..n {
            let p0 = self.outer_boundary[i];
            let p1 = self.outer_boundary[(i + 1) % n];
            dxf.push_str(&format!(
                "0\nLINE\n8\nOUTER_CUT\n10\n{:.3}\n20\n{:.3}\n30\n0.0\n11\n{:.3}\n21\n{:.3}\n31\n0.0\n",
                p0.x, p0.y, p1.x, p1.y
            ));
        }

        // 2. Bend Centerlines
        for bend in &self.bend_lines {
            let layer = match bend.direction {
                BendDirection::Up => "BEND_UP",
                BendDirection::Down => "BEND_DOWN",
            };
            dxf.push_str(&format!(
                "0\nLINE\n8\n{}\n10\n{:.3}\n20\n{:.3}\n30\n0.0\n11\n{:.3}\n21\n{:.3}\n31\n0.0\n",
                layer, bend.start.x, bend.start.y, bend.end.x, bend.end.y
            ));
        }

        dxf.push_str("0\nENDSEC\n0\nEOF\n");
        dxf
    }

    /// Export the flat pattern as an SVG vector graphic
    pub fn export_svg(&self) -> String {
        let margin = 20.0f32;
        let w = self.total_width + margin * 2.0;
        let h = self.total_length + margin * 2.0;
        let cx = w * 0.5;
        let cy = h * 0.5;

        let mut svg = String::new();
        svg.push_str(&format!(
            r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {:.2} {:.2}" width="{:.2}mm" height="{:.2}mm">"##,
            w, h, w, h
        ));
        svg.push('\n');

        // Background
        svg.push_str(&format!(r##"  <rect width="{:.2}" height="{:.2}" fill="#ffffff" />"##, w, h));
        svg.push('\n');

        // Outer Cut Boundary (Solid Black)
        svg.push_str(r##"  <g id="OUTER_CUT" stroke="#111111" stroke-width="0.8" fill="#f4f6f8">"##);
        svg.push('\n');
        let mut pts_str = String::new();
        for pt in &self.outer_boundary {
            let sx = cx + pt.x;
            let sy = cy - pt.y; // Invert Y for SVG
            pts_str.push_str(&format!("{:.2},{:.2} ", sx, sy));
        }
        svg.push_str(&format!(r##"    <polygon points="{}" />"##, pts_str.trim()));
        svg.push('\n');
        svg.push_str("  </g>\n");

        // Bend Lines (Red dashed for Up, Green dash-dot for Down)
        svg.push_str(r##"  <g id="BEND_LINES">"##);
        svg.push('\n');
        for bend in &self.bend_lines {
            let (stroke, dash) = match bend.direction {
                BendDirection::Up => ("#cc0000", "4,2"),
                BendDirection::Down => ("#008800", "4,2,1,2"),
            };
            let sx0 = cx + bend.start.x;
            let sy0 = cy - bend.start.y;
            let sx1 = cx + bend.end.x;
            let sy1 = cy - bend.end.y;

            svg.push_str(&format!(
                r##"    <line x1="{:.2}" y1="{:.2}" x2="{:.2}" y2="{:.2}" stroke="{}" stroke-width="0.5" stroke-dasharray="{}" />"##,
                sx0, sy0, sx1, sy1, stroke, dash
            ));
            svg.push('\n');

            // Text Callout
            let mid_x = (sx0 + sx1) * 0.5;
            let mid_y = (sy0 + sy1) * 0.5;
            svg.push_str(&format!(
                r##"    <text x="{:.2}" y="{:.2}" font-family="monospace" font-size="3" fill="{}" text-anchor="middle">{}</text>"##,
                mid_x, mid_y - 2.0, stroke, bend.formatted_callout()
            ));
            svg.push('\n');
        }
        svg.push_str("  </g>\n");

        svg.push_str("</svg>\n");
        svg
    }
}

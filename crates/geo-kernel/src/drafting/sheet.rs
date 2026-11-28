//! Technical 2D Drawing Sheet Layout & Multi-Format Exporter (SVG & DXF)
//!
//! Provides ISO/ANSI engineering drawing sheet layouts, automated multi-view
//! placement (Front, Top, Right, Isometric), title blocks, and vector export.

use core_math::Vec2;
use serde::{Deserialize, Serialize};

use super::gdt::{CenterMark, FeatureControlFrame, LinearDimension};
use super::hlr::{HlrEdgeSegment, HlrEngine, HlrLineStyle};
use super::projection::{ProjectionStandard, ViewCamera, ViewType};
use crate::mesh::Mesh;

/// Standard ISO 216 & ANSI/ASME Y14.1 Drawing Sheet Formats
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SheetFormat {
    #[default]
    IsoA4, // 297 x 210 mm (Landscape)
    IsoA3, // 420 x 297 mm
    IsoA2, // 594 x 420 mm
    IsoA1, // 841 x 594 mm
    IsoA0, // 1189 x 841 mm
    AnsiA, // 279.4 x 215.9 mm (11" x 8.5")
    AnsiB, // 431.8 x 279.4 mm (17" x 11")
    AnsiC, // 558.8 x 431.8 mm (22" x 17")
    AnsiD, // 863.6 x 558.8 mm (34" x 22")
}

impl SheetFormat {
    /// Sheet width and height in millimeters (Landscape orientation)
    pub fn dimensions_mm(&self) -> (f32, f32) {
        match self {
            Self::IsoA4 => (297.0, 210.0),
            Self::IsoA3 => (420.0, 297.0),
            Self::IsoA2 => (594.0, 420.0),
            Self::IsoA1 => (841.0, 594.0),
            Self::IsoA0 => (1189.0, 841.0),
            Self::AnsiA => (279.4, 215.9),
            Self::AnsiB => (431.8, 279.4),
            Self::AnsiC => (558.8, 431.8),
            Self::AnsiD => (863.6, 558.8),
        }
    }

    /// Border margin from paper edge (mm)
    pub fn margin_mm(&self) -> f32 {
        10.0
    }
}

/// Engineering drawing title block metadata
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TitleBlock {
    pub title: String,
    pub drawing_number: String,
    pub revision: String,
    pub author: String,
    pub date: String,
    pub scale_str: String,
    pub projection: ProjectionStandard,
}

impl Default for TitleBlock {
    fn default() -> Self {
        Self {
            title: "ENGINEERING PART".to_string(),
            drawing_number: "PLMB-CAD-001".to_string(),
            revision: "A".to_string(),
            author: "PLUMB 3D".to_string(),
            date: "2026-11-28".to_string(),
            scale_str: "1:1".to_string(),
            projection: ProjectionStandard::ThirdAngle,
        }
    }
}

/// A 2D orthographic or isometric view projected on the drawing sheet
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DrawingView {
    pub view_type: ViewType,
    pub title: String,
    pub center: Vec2,
    pub scale: f32,
    pub segments: Vec<HlrEdgeSegment>,
}

/// A complete 2D Technical Engineering Drawing Sheet
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DrawingSheet {
    pub format: SheetFormat,
    pub title_block: TitleBlock,
    pub views: Vec<DrawingView>,
    pub dimensions: Vec<LinearDimension>,
    pub gdt_frames: Vec<FeatureControlFrame>,
    pub center_marks: Vec<CenterMark>,
}

impl Default for DrawingSheet {
    fn default() -> Self {
        Self {
            format: SheetFormat::IsoA3,
            title_block: TitleBlock::default(),
            views: Vec::new(),
            dimensions: Vec::new(),
            gdt_frames: Vec::new(),
            center_marks: Vec::new(),
        }
    }
}

impl DrawingSheet {
    pub fn new(format: SheetFormat, projection: ProjectionStandard) -> Self {
        let mut title_block = TitleBlock::default();
        title_block.projection = projection;
        Self {
            format,
            title_block,
            views: Vec::new(),
            dimensions: Vec::new(),
            gdt_frames: Vec::new(),
            center_marks: Vec::new(),
        }
    }

    /// Automatically generate standard 4-view layout (Front, Top, Right, Isometric)
    pub fn generate_multi_view(&mut self, mesh: &Mesh, scale: f32) {
        self.views.clear();
        self.dimensions.clear();

        let (w, h) = self.format.dimensions_mm();
        let target = mesh.compute_aabb().center();

        // Calculate view centers according to projection standard
        let (front_c, top_c, right_c, iso_c) = match self.title_block.projection {
            ProjectionStandard::ThirdAngle => {
                (
                    Vec2::new(w * 0.30, h * 0.38),
                    Vec2::new(w * 0.30, h * 0.72),
                    Vec2::new(w * 0.65, h * 0.38),
                    Vec2::new(w * 0.65, h * 0.72),
                )
            }
            ProjectionStandard::FirstAngle => {
                (
                    Vec2::new(w * 0.30, h * 0.72),
                    Vec2::new(w * 0.30, h * 0.38),
                    Vec2::new(w * 0.65, h * 0.72),
                    Vec2::new(w * 0.65, h * 0.38),
                )
            }
        };

        let view_defs = [
            (ViewType::Front, "FRONT VIEW", front_c),
            (ViewType::Top, "TOP VIEW", top_c),
            (ViewType::Right, "RIGHT VIEW", right_c),
            (ViewType::Isometric, "ISOMETRIC VIEW (30°)", iso_c),
        ];

        for (vt, title, center) in view_defs {
            let camera = ViewCamera::new(vt, center, scale, target);
            let segments = HlrEngine::extract_hlr_edges(mesh, &camera, 30.0);
            self.views.push(DrawingView {
                view_type: vt,
                title: title.to_string(),
                center,
                scale,
                segments,
            });
        }

        // Add overall bounding dimensions on Front view
        let aabb = mesh.compute_aabb();
        let ext = aabb.extents();
        let front_min = front_c - Vec2::new(ext.x * 0.5 * scale, ext.y * 0.5 * scale);
        let front_max = front_c + Vec2::new(ext.x * 0.5 * scale, ext.y * 0.5 * scale);

        // Width dimension (bottom)
        let mut dim_w = LinearDimension::new(
            Vec2::new(front_min.x, front_min.y),
            Vec2::new(front_max.x, front_min.y),
            12.0,
        );
        dim_w.nominal = ext.x;
        self.dimensions.push(dim_w);

        // Height dimension (left)
        let mut dim_h = LinearDimension::new(
            Vec2::new(front_min.x, front_max.y),
            Vec2::new(front_min.x, front_min.y),
            12.0,
        );
        dim_h.nominal = ext.y;
        self.dimensions.push(dim_h);
    }

    /// Export the drawing sheet as standards-compliant Scalable Vector Graphics (SVG)
    pub fn export_svg(&self) -> String {
        let (w, h) = self.format.dimensions_mm();
        let margin = self.format.margin_mm();
        let mut svg = String::new();

        svg.push_str(&format!(
            r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {:.2} {:.2}" width="{:.2}mm" height="{:.2}mm">"##,
            w, h, w, h
        ));
        svg.push('\n');

        // Background white sheet
        svg.push_str(&format!(
            r##"  <rect width="{:.2}" height="{:.2}" fill="#ffffff" stroke="none" />"##,
            w, h
        ));
        svg.push('\n');

        // 1. Drawing Border & Title Block
        svg.push_str(r##"  <g id="BORDER_AND_TITLE_BLOCK" stroke="#111111" stroke-width="0.7">"##);
        svg.push('\n');
        // Outer border
        svg.push_str(&format!(
            r##"    <rect x="{:.2}" y="{:.2}" width="{:.2}" height="{:.2}" fill="none" stroke-width="0.7" />"##,
            margin, margin, w - margin * 2.0, h - margin * 2.0
        ));
        svg.push('\n');

        // Title Block in bottom-right corner (120 x 36 mm)
        let tb_w = 120.0f32;
        let tb_h = 36.0f32;
        let tb_x = w - margin - tb_w;
        let tb_y = h - margin - tb_h;
        svg.push_str(&format!(
            r##"    <rect x="{:.2}" y="{:.2}" width="{:.2}" height="{:.2}" fill="#fcfcfc" stroke-width="0.5" />"##,
            tb_x, tb_y, tb_w, tb_h
        ));
        svg.push('\n');
        svg.push_str(&format!(
            r##"    <text x="{:.2}" y="{:.2}" font-family="monospace" font-size="5" font-weight="bold" fill="#111">{}</text>"##,
            tb_x + 4.0, tb_y + 8.0, self.title_block.title
        ));
        svg.push('\n');
        svg.push_str(&format!(
            r##"    <text x="{:.2}" y="{:.2}" font-family="monospace" font-size="3.5" fill="#444">DWG: {}  REV: {}</text>"##,
            tb_x + 4.0, tb_y + 16.0, self.title_block.drawing_number, self.title_block.revision
        ));
        svg.push('\n');
        svg.push_str(&format!(
            r##"    <text x="{:.2}" y="{:.2}" font-family="monospace" font-size="3.5" fill="#444">SCALE: {}  PROJ: {:?}</text>"##,
            tb_x + 4.0, tb_y + 24.0, self.title_block.scale_str, self.title_block.projection
        ));
        svg.push('\n');
        svg.push_str(&format!(
            r##"    <text x="{:.2}" y="{:.2}" font-family="monospace" font-size="3.5" fill="#444">DATE: {}  BY: {}</text>"##,
            tb_x + 4.0, tb_y + 32.0, self.title_block.date, self.title_block.author
        ));
        svg.push('\n');
        svg.push_str("  </g>\n");

        // 2. Visible Feature Edges (Solid 0.5mm)
        svg.push_str(r##"  <g id="VISIBLE_EDGES" stroke="#111111" stroke-width="0.5" stroke-linecap="round">"##);
        svg.push('\n');
        for view in &self.views {
            // View Title
            svg.push_str(&format!(
                r##"    <text x="{:.2}" y="{:.2}" font-family="sans-serif" font-size="4" font-weight="bold" fill="#333" text-anchor="middle">{}</text>"##,
                view.center.x, h - (view.center.y - 30.0), view.title
            ));
            svg.push('\n');

            for seg in &view.segments {
                if seg.style == HlrLineStyle::Visible || seg.style == HlrLineStyle::Silhouette {
                    let y0 = h - seg.p0.y;
                    let y1 = h - seg.p1.y;
                    svg.push_str(&format!(
                        r##"    <line x1="{:.2}" y1="{:.2}" x2="{:.2}" y2="{:.2}" />"##,
                        seg.p0.x, y0, seg.p1.x, y1
                    ));
                    svg.push('\n');
                }
            }
        }
        svg.push_str("  </g>\n");

        // 3. Hidden Edges (Dashed 0.25mm)
        svg.push_str(r##"  <g id="HIDDEN_EDGES" stroke="#666666" stroke-width="0.25" stroke-dasharray="2.5,1.5" stroke-linecap="butt">"##);
        svg.push('\n');
        for view in &self.views {
            for seg in &view.segments {
                if seg.style == HlrLineStyle::Hidden {
                    let y0 = h - seg.p0.y;
                    let y1 = h - seg.p1.y;
                    svg.push_str(&format!(
                        r##"    <line x1="{:.2}" y1="{:.2}" x2="{:.2}" y2="{:.2}" />"##,
                        seg.p0.x, y0, seg.p1.x, y1
                    ));
                    svg.push('\n');
                }
            }
        }
        svg.push_str("  </g>\n");

        // 4. Dimensions & Callouts
        svg.push_str(r##"  <g id="DIMENSIONS" stroke="#0044aa" stroke-width="0.35" fill="none">"##);
        svg.push('\n');
        for dim in &self.dimensions {
            let (ext1, ext2, d1, d2, mid) = dim.compute_geometry();
            let y_ext1 = h - ext1.y;
            let y_ext2 = h - ext2.y;
            let y_d1 = h - d1.y;
            let y_d2 = h - d2.y;
            let y_p1 = h - dim.p1.y;
            let y_p2 = h - dim.p2.y;
            let y_mid = h - mid.y;

            svg.push_str(&format!(r##"    <line x1="{:.2}" y1="{:.2}" x2="{:.2}" y2="{:.2}" />"##, dim.p1.x, y_p1, ext1.x, y_ext1));
            svg.push('\n');
            svg.push_str(&format!(r##"    <line x1="{:.2}" y1="{:.2}" x2="{:.2}" y2="{:.2}" />"##, dim.p2.x, y_p2, ext2.x, y_ext2));
            svg.push('\n');
            svg.push_str(&format!(r##"    <line x1="{:.2}" y1="{:.2}" x2="{:.2}" y2="{:.2}" />"##, d1.x, y_d1, d2.x, y_d2));
            svg.push('\n');
            svg.push_str(&format!(
                r##"    <text x="{:.2}" y="{:.2}" font-family="sans-serif" font-size="3.5" font-weight="bold" fill="#0044aa" text-anchor="middle">{}</text>"##,
                mid.x, y_mid - 1.5, dim.formatted_text()
            ));
            svg.push('\n');
        }
        svg.push_str("  </g>\n");

        svg.push_str("</svg>\n");
        svg
    }

    /// Export the drawing sheet as an AutoCAD DXF (R12/2000) ASCII stream
    pub fn export_dxf(&self) -> String {
        let mut dxf = String::new();
        dxf.push_str("0\nSECTION\n2\nHEADER\n0\nENDSEC\n");
        dxf.push_str("0\nSECTION\n2\nTABLES\n0\nTABLE\n2\nLAYER\n70\n3\n");
        dxf.push_str("0\nLAYER\n2\nVISIBLE\n70\n0\n62\n7\n6\nCONTINUOUS\n0\n");
        dxf.push_str("0\nLAYER\n2\nHIDDEN\n70\n0\n62\n8\n6\nHIDDEN\n0\n");
        dxf.push_str("0\nLAYER\n2\nDIMENSIONS\n70\n0\n62\n5\n6\nCONTINUOUS\n0\n");
        dxf.push_str("0\nENDTAB\n0\nENDSEC\n");
        dxf.push_str("0\nSECTION\n2\nENTITIES\n");

        for view in &self.views {
            for seg in &view.segments {
                let layer = if seg.style == HlrLineStyle::Hidden {
                    "HIDDEN"
                } else {
                    "VISIBLE"
                };
                dxf.push_str(&format!(
                    "0\nLINE\n8\n{}\n10\n{:.3}\n20\n{:.3}\n30\n0.0\n11\n{:.3}\n21\n{:.3}\n31\n0.0\n",
                    layer, seg.p0.x, seg.p0.y, seg.p1.x, seg.p1.y
                ));
            }
        }

        dxf.push_str("0\nENDSEC\n0\nEOF\n");
        dxf
    }
}

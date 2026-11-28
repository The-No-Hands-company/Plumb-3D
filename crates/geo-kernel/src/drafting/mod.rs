//! Technical 2D Engineering Drawings, Hidden Line Removal (HLR) & GD&T
//!
//! Provides the complete technical drafting system for Plumb 3D:
//! - [`projection`]: Orthographic camera projection engines (First-Angle ISO / Third-Angle ANSI).
//! - [`hlr`]: Analytical Hidden Line Removal (HLR) edge classifier (visible solid vs hidden dashed).
//! - [`gdt`]: Geometric Dimensioning & Tolerancing (GD&T) and linear/diametral callouts.
//! - [`sheet`]: Standard drawing sheet layout (ISO A4–A0 / ANSI A–E) and vector export (SVG & DXF).

pub mod gdt;
pub mod hlr;
pub mod projection;
pub mod sheet;

pub use gdt::{
    CenterMark, DiameterDimension, FeatureControlFrame, GdtCharacteristic, LinearDimension,
    MaterialCondition,
};
pub use hlr::{HlrEdgeSegment, HlrEngine, HlrLineStyle};
pub use projection::{ProjectionStandard, ViewCamera, ViewType};
pub use sheet::{DrawingSheet, DrawingView, SheetFormat, TitleBlock};

#[cfg(test)]
mod tests {
    use super::*;
    use core_math::{Vec2, Vec3};
    use crate::primitives::Primitives;

    #[test]
    fn test_orthographic_camera_projections() {
        let center = Vec2::new(100.0, 100.0);
        let cam_front = ViewCamera::new(ViewType::Front, center, 1.0, Vec3::ZERO);
        let (p_front, z_front) = cam_front.project_point(Vec3::new(10.0, 20.0, 30.0));
        assert!((p_front.x - 110.0).abs() < 1e-4);
        assert!((p_front.y - 120.0).abs() < 1e-4);
        assert!((z_front - (-30.0)).abs() < 1e-4);

        let cam_top = ViewCamera::new(ViewType::Top, center, 1.0, Vec3::ZERO);
        let (p_top, _) = cam_top.project_point(Vec3::new(10.0, 20.0, 30.0));
        assert!((p_top.x - 110.0).abs() < 1e-4);
        assert!((p_top.y - 70.0).abs() < 1e-4); // Looking down from +Y, -Z projects upwards
    }

    #[test]
    fn test_hlr_cube_edges_classification() {
        let cube = Primitives::create_cube(10.0);
        let center = Vec2::new(150.0, 100.0);
        let cam = ViewCamera::new(ViewType::Front, center, 1.0, Vec3::ZERO);

        let segments = HlrEngine::extract_hlr_edges(&cube, &cam, 30.0);
        assert!(!segments.is_empty(), "HLR must extract edges from cube");

        let visible_count = segments
            .iter()
            .filter(|s| s.style == HlrLineStyle::Visible || s.style == HlrLineStyle::Silhouette)
            .count();
        let hidden_count = segments
            .iter()
            .filter(|s| s.style == HlrLineStyle::Hidden)
            .count();

        assert!(visible_count > 0, "Cube front view must have visible silhouette edges");
        assert!(hidden_count > 0, "Cube front view must have occluded hidden back edges");
    }

    #[test]
    fn test_gdt_feature_control_frame_formatting() {
        let mut fcf = FeatureControlFrame::new(
            GdtCharacteristic::Position,
            0.05,
            Vec2::new(50.0, 50.0),
        );
        fcf.is_diameter = true;
        fcf.material_condition = Some(MaterialCondition::Mmc);
        fcf.primary_datum = Some("A".to_string());
        fcf.secondary_datum = Some("B".to_string());

        let text = fcf.format_text();
        assert!(text.contains("⌖"));
        assert!(text.contains("⌀"));
        assert!(text.contains("0.050"));
        assert!(text.contains("Ⓜ"));
        assert!(text.contains("| A | B |"));
    }

    #[test]
    fn test_drawing_sheet_multi_view_and_export() {
        let cube = Primitives::create_cube(20.0);
        let mut sheet = DrawingSheet::new(SheetFormat::IsoA3, ProjectionStandard::ThirdAngle);

        sheet.generate_multi_view(&cube, 1.0);

        assert_eq!(sheet.views.len(), 4, "Must generate Front, Top, Right, and Isometric views");
        assert!(!sheet.dimensions.is_empty(), "Must generate bounding dimensions");

        // Verify SVG export output
        let svg = sheet.export_svg();
        assert!(svg.contains("<svg"), "Must output valid SVG root element");
        assert!(svg.contains("BORDER_AND_TITLE_BLOCK"), "Must contain title block");
        assert!(svg.contains("VISIBLE_EDGES"), "Must contain visible layer");
        assert!(svg.contains("HIDDEN_EDGES"), "Must contain hidden dashed layer");
        assert!(svg.contains("DIMENSIONS"), "Must contain dimension callouts layer");

        // Verify DXF export output
        let dxf = sheet.export_dxf();
        assert!(dxf.contains("SECTION\n2\nHEADER"), "Must output DXF header");
        assert!(dxf.contains("LAYER\n2\nVISIBLE"), "Must output DXF visible layer");
        assert!(dxf.contains("LAYER\n2\nHIDDEN"), "Must output DXF hidden layer");
        assert!(dxf.contains("0\nEOF"), "Must terminate with EOF");
    }
}

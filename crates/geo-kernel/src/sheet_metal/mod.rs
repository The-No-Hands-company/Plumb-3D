//! Sheet Metal Modeling & Flat-Pattern Unfolding Engine
//!
//! Provides the complete sheet metal discipline for Plumb 3D:
//! - [`params`]: Sheet thickness, inner bend radius, K-factor neutral axis, and bend deduction.
//! - [`bend`]: 3D cylindrical bend transitions and bend directions (Up/Down).
//! - [`flange`]: Base flange solid plates and angled edge flange extensions.
//! - [`unfold`]: 1-click flat-pattern blank generation and CNC DXF/SVG exporters.

pub mod bend;
pub mod flange;
pub mod params;
pub mod unfold;

pub use bend::{BendDirection, SheetBend};
pub use flange::{BaseFlangeSettings, EdgeFlangeSettings, SheetMetalBuilder};
pub use params::{BendReliefType, SheetMetalParams};
pub use unfold::{FlatBendLine, FlatPattern};

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn test_k_factor_bend_allowance_and_deduction() {
        let params = SheetMetalParams::new(2.0, 2.0, 0.44);

        // For a 90° bend:
        // BA = (PI / 2) * (2.0 + 0.44 * 2.0) = 1.5707963 * 2.88 = 4.523893
        let ba_90 = params.bend_allowance(90.0);
        let expected_ba = (PI * 0.5) * (2.0 + 0.44 * 2.0);
        assert!((ba_90 - expected_ba).abs() < 1e-4);

        // OSSB = (2.0 + 2.0) * tan(45°) = 4.0 * 1.0 = 4.0
        let ossb_90 = params.outside_setback(90.0);
        assert!((ossb_90 - 4.0).abs() < 1e-4);

        // BD = 2 * OSSB - BA = 8.0 - 4.523893 = 3.4761
        let bd_90 = params.bend_deduction(90.0);
        assert!((bd_90 - (8.0 - expected_ba)).abs() < 1e-4);
    }

    #[test]
    fn test_base_flange_solid_creation() {
        let settings = BaseFlangeSettings {
            width: 80.0,
            length: 120.0,
            params: SheetMetalParams::new(1.5, 1.5, 0.44),
        };
        let mesh = SheetMetalBuilder::create_base_flange(&settings);

        assert_eq!(mesh.vertex_count(), 8, "Base flange plate has 8 vertices");
        assert_eq!(mesh.face_count(), 6, "Base flange plate has 6 faces");

        let aabb = mesh.compute_aabb();
        assert!((aabb.extents().x - 80.0).abs() < 1e-4);
        assert!((aabb.extents().y - 120.0).abs() < 1e-4);
        assert!((aabb.extents().z - 1.5).abs() < 1e-4);
    }

    #[test]
    fn test_edge_flange_and_bend_generation() {
        let base_settings = BaseFlangeSettings {
            width: 100.0,
            length: 80.0,
            params: SheetMetalParams::new(2.0, 2.0, 0.44),
        };
        let base_mesh = SheetMetalBuilder::create_base_flange(&base_settings);

        let flange_settings = EdgeFlangeSettings {
            length: 25.0,
            angle_deg: 90.0,
            bend_radius: Some(2.0),
            direction: BendDirection::Up,
        };

        let (flanged_mesh, bend) = SheetMetalBuilder::add_edge_flange(
            &base_mesh,
            &flange_settings,
            &base_settings.params,
            50.0, // right edge
            80.0, // length along edge
        );

        assert!(flanged_mesh.vertex_count() > base_mesh.vertex_count());
        assert_eq!(bend.angle_deg, 90.0);
        assert_eq!(bend.direction, BendDirection::Up);
        assert!(bend.bend_allowance > 0.0);
    }

    #[test]
    fn test_flat_pattern_unfolding_and_dxf_export() {
        let params = SheetMetalParams::new(2.0, 2.0, 0.44);
        let bend = SheetBend::new(
            0,
            "Bend1",
            90.0,
            2.0,
            BendDirection::Up,
            core_math::Vec3::new(50.0, -40.0, 0.0),
            core_math::Vec3::new(50.0, 40.0, 0.0),
            &params,
        );

        let flat = FlatPattern::compute_flat_pattern(
            100.0,
            80.0,
            &[30.0],
            &[bend],
            &params,
        );

        assert!(flat.total_width > 100.0, "Unfolded width includes flange and bend allowance");
        assert_eq!(flat.bend_lines.len(), 1, "Must contain 1 bend centerline");
        assert_eq!(flat.bend_lines[0].direction, BendDirection::Up);

        // Test DXF generation
        let dxf = flat.export_dxf();
        assert!(dxf.contains("SECTION\n2\nHEADER"));
        assert!(dxf.contains("LAYER\n2\nOUTER_CUT"));
        assert!(dxf.contains("LAYER\n2\nBEND_UP"));
        assert!(dxf.contains("0\nEOF"));

        // Test SVG generation
        let svg = flat.export_svg();
        assert!(svg.contains("<svg"));
        assert!(svg.contains("OUTER_CUT"));
        assert!(svg.contains("BEND_LINES"));
        assert!(svg.contains("BEND Up 90.0°"));
    }
}

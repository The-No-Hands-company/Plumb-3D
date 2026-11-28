//! Mechanical Part Generators & Power Transmission Library
//!
//! Provides parametric ISO/ANSI standard Fasteners (bolts, screws, nuts, washers),
//! analytical Involute Gears (Spur, Helical, Bevel), and Roller Chain Sprockets.

pub mod fastener;
pub mod gear;
pub mod sprocket;

pub use fastener::{
    FastenerBuilder, FastenerParams, FastenerType, MetricThreadSize, ThreadMode,
};
pub use gear::{GearParams, GearType, InvoluteGearBuilder};
pub use sprocket::{AnsiChainSize, SprocketBuilder, SprocketParams};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fastener_generation_hex_bolt() {
        let params = FastenerParams {
            size: MetricThreadSize::M6,
            fastener_type: FastenerType::HexBolt,
            length: 25.0,
            thread_mode: ThreadMode::Cosmetic,
        };

        let mesh = FastenerBuilder::generate_mesh(&params);
        let (min, max) = mesh.bounding_box();

        assert!(mesh.vertices.len() >= 32);
        assert!(mesh.faces.len() >= 20);
        // Head height is ~0.65 * 6 = 3.9 mm, shank length is 25 mm -> total Z span ~28.9 mm
        let z_span = max.z - min.z;
        assert!((z_span - 28.9).abs() < 1.0);
    }

    #[test]
    fn test_fastener_generation_hex_nut_and_washer() {
        let nut_params = FastenerParams {
            size: MetricThreadSize::M8,
            fastener_type: FastenerType::HexNut,
            length: 0.0,
            thread_mode: ThreadMode::Cosmetic,
        };
        let nut_mesh = FastenerBuilder::generate_mesh(&nut_params);
        assert!(!nut_mesh.faces.is_empty());

        let washer_params = FastenerParams {
            size: MetricThreadSize::M8,
            fastener_type: FastenerType::FlatWasher,
            length: 0.0,
            thread_mode: ThreadMode::Cosmetic,
        };
        let washer_mesh = FastenerBuilder::generate_mesh(&washer_params);
        let (min, max) = washer_mesh.bounding_box();
        let xy_span = (max.x - min.x).max(max.y - min.y);
        // Outer radius for M8 washer is 8 * 1.0 = 8 -> diameter 16 mm
        assert!((xy_span - 16.0).abs() < 1.0);
    }

    #[test]
    fn test_involute_gear_spur_and_helical() {
        let spur_params = GearParams {
            module: 2.0,
            teeth: 20,
            pressure_angle_deg: 20.0,
            face_width: 12.0,
            helix_angle_deg: 0.0,
            bore_diameter: 8.0,
            gear_type: GearType::Spur,
        };

        assert_eq!(spur_params.pitch_diameter(), 40.0);
        assert_eq!(spur_params.addendum(), 2.0);
        assert_eq!(spur_params.dedendum(), 2.5);
        assert_eq!(spur_params.tip_radius(), 22.0);
        assert_eq!(spur_params.root_radius(), 17.5);

        let spur_mesh = InvoluteGearBuilder::generate_mesh(&spur_params);
        let (min, max) = spur_mesh.bounding_box();
        assert!((max.z - min.z - 12.0).abs() < 1e-3);
        assert!((max.x - min.x - 44.0).abs() < 2.0); // Tip diameter is 44 mm

        let helical_params = GearParams {
            helix_angle_deg: 20.0,
            gear_type: GearType::Helical,
            ..spur_params
        };
        let helical_mesh = InvoluteGearBuilder::generate_mesh(&helical_params);
        assert!(helical_mesh.vertices.len() > spur_mesh.vertices.len());
    }

    #[test]
    fn test_sprocket_generation_ansi40() {
        let params = SprocketParams {
            chain_size: AnsiChainSize::Ansi40,
            teeth: 18,
            bore_diameter: 12.0,
            hub_diameter: 30.0,
            hub_length: 10.0,
        };

        let pcd = params.pitch_diameter();
        assert!((pcd - 73.14).abs() < 1.0);

        let mesh = SprocketBuilder::generate_mesh(&params);
        let (min, max) = mesh.bounding_box();
        // Tooth thickness for ANSI 40 is 7.2 mm + hub length 10 mm = 17.2 mm
        assert!((max.z - min.z - 17.2).abs() < 1e-3);
    }
}

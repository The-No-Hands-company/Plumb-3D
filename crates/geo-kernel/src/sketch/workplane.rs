//! 3D Workplanes and Coordinate Framing for 2D CAD Sketching.
//!
//! Re-exports and integrates the unified `core_math::Workplane` for CAD sketching.

pub use core_math::Workplane;

#[cfg(test)]
mod tests {
    use super::*;
    use core_math::{vec2, vec3, Vec3};

    #[test]
    fn test_workplane_projections() {
        let wp = Workplane::xy();
        let uv = vec2(15.0, -25.0);
        let world = wp.to_3d(uv);
        assert_eq!(world, vec3(15.0, -25.0, 0.0));

        let uv_back = wp.to_2d(world);
        assert!((uv_back - uv).length() < 1e-5);
    }

    #[test]
    fn test_workplane_offset_and_arbitrary_normal() {
        let wp = Workplane::from_point_and_normal(vec3(0.0, 10.0, 0.0), vec3(0.0, 1.0, 0.0));
        assert!((wp.normal - Vec3::Y).length() < 1e-5);

        let offset_wp = wp.offset(5.0);
        assert_eq!(offset_wp.origin, vec3(0.0, 15.0, 0.0));

        let pt = vec3(4.0, 15.0, 8.0);
        let uv = offset_wp.to_2d(pt);
        let pt_reconstructed = offset_wp.to_3d(uv);
        assert!((pt_reconstructed - pt).length() < 1e-5);
    }
}

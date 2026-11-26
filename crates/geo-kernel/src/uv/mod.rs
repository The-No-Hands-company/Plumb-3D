pub mod pack;
pub mod projection;
pub mod unwrap;

pub use projection::ProjectionAxis;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::Primitives;

    #[test]
    fn test_uv_projections() {
        let mut cube = Primitives::create_cube(2.0);
        cube.generate_planar_uv(ProjectionAxis::Y);
        for v in cube.vertices.values() {
            assert!(v.uv.x >= 0.0 && v.uv.x <= 1.0);
            assert!(v.uv.y >= 0.0 && v.uv.y <= 1.0);
        }

        cube.generate_spherical_uv();
        for v in cube.vertices.values() {
            assert!(v.uv.x >= 0.0 && v.uv.x <= 1.0);
            assert!(v.uv.y >= 0.0 && v.uv.y <= 1.0);
        }
    }
}

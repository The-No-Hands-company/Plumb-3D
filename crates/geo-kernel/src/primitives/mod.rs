//! Standard and Parametric 3D Mesh Primitives.

pub mod architectural;
pub mod box_plane;
pub mod curved;
pub mod polyhedral;

pub use architectural::{create_geodesic_dome, create_pipe_thread};
pub use box_plane::{create_box_parametric, create_cube, create_plane};
pub use curved::{
    create_capsule, create_cone, create_cone_parametric, create_cylinder, create_cylinder_parametric,
    create_icosphere, create_torus, create_uv_sphere, CapFillType,
};
pub use polyhedral::{create_plumb_bob, create_plumb_bob_detailed, create_pyramid, create_pyramid_parametric};

use crate::mesh::Mesh;

/// Namespace struct for backward compatibility with existing `Primitives::create_*` call sites.
pub struct Primitives;

impl Primitives {
    #[inline]
    pub fn create_cube(size: f32) -> Mesh {
        create_cube(size)
    }

    #[inline]
    pub fn create_box_parametric(width: f32, height: f32, depth: f32, seg_x: u32, seg_y: u32, seg_z: u32) -> Mesh {
        create_box_parametric(width, height, depth, seg_x, seg_y, seg_z)
    }

    #[inline]
    pub fn create_plane(width: f32, length: f32, sub_x: u32, sub_z: u32) -> Mesh {
        create_plane(width, length, sub_x, sub_z)
    }

    #[inline]
    pub fn create_uv_sphere(radius: f32, segments: u32, rings: u32) -> Mesh {
        create_uv_sphere(radius, segments, rings)
    }

    #[inline]
    pub fn create_cylinder(radius: f32, height: f32, segments: u32) -> Mesh {
        create_cylinder(radius, height, segments)
    }

    #[inline]
    pub fn create_cylinder_parametric(radius: f32, height: f32, segments: u32, cap_fill: CapFillType) -> Mesh {
        create_cylinder_parametric(radius, height, segments, cap_fill)
    }

    #[inline]
    pub fn create_cone(radius: f32, height: f32, segments: u32) -> Mesh {
        create_cone(radius, height, segments)
    }

    #[inline]
    pub fn create_cone_parametric(radius_bottom: f32, radius_top: f32, height: f32, segments: u32, cap_fill: CapFillType) -> Mesh {
        create_cone_parametric(radius_bottom, radius_top, height, segments, cap_fill)
    }

    #[inline]
    pub fn create_torus(major_radius: f32, minor_radius: f32, major_segments: u32, minor_segments: u32) -> Mesh {
        create_torus(major_radius, minor_radius, major_segments, minor_segments)
    }

    #[inline]
    pub fn create_pyramid(base_size: f32, height: f32) -> Mesh {
        create_pyramid(base_size, height)
    }

    #[inline]
    pub fn create_pyramid_parametric(base_width: f32, base_depth: f32, height: f32) -> Mesh {
        create_pyramid_parametric(base_width, base_depth, height)
    }

    #[inline]
    pub fn create_capsule(radius: f32, cylinder_height: f32, segments: u32, hemisphere_rings: u32) -> Mesh {
        create_capsule(radius, cylinder_height, segments, hemisphere_rings)
    }

    #[inline]
    pub fn create_icosphere(radius: f32, subdivisions: u32) -> Mesh {
        create_icosphere(radius, subdivisions)
    }

    #[inline]
    pub fn create_plumb_bob() -> Mesh {
        create_plumb_bob()
    }

    #[inline]
    pub fn create_plumb_bob_detailed(segments: usize) -> Mesh {
        create_plumb_bob_detailed(segments)
    }

    #[inline]
    pub fn create_geodesic_dome(radius: f32, frequency: u32) -> Mesh {
        create_geodesic_dome(radius, frequency)
    }

    #[inline]
    pub fn create_pipe_thread(
        major_diameter: f32,
        pitch: f32,
        thread_length: f32,
        segments_per_turn: u32,
    ) -> Mesh {
        create_pipe_thread(major_diameter, pitch, thread_length, segments_per_turn)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parametric_box_subdivisions() {
        let b = Primitives::create_box_parametric(2.0, 3.0, 4.0, 2, 2, 2);
        assert_eq!(b.faces.len(), 24); // 6 faces * 4 quads each
        assert!(b.validate().is_ok());
    }

    #[test]
    fn test_parametric_cone_frustum() {
        let frustum = Primitives::create_cone_parametric(2.0, 1.0, 3.0, 16, CapFillType::Ngon);
        assert!(frustum.validate().is_ok());
        assert_eq!(frustum.faces.len(), 18); // 16 sides + 2 caps
    }

    #[test]
    fn test_geodesic_dome() {
        let dome = Primitives::create_geodesic_dome(2.0, 2);
        assert!(!dome.vertices.is_empty());
        assert!(!dome.faces.is_empty());
        assert!(dome.validate().is_ok());
    }

    #[test]
    fn test_pipe_thread() {
        let thread = Primitives::create_pipe_thread(1.0, 0.2, 0.6, 16);
        assert!(!thread.vertices.is_empty());
        assert!(!thread.faces.is_empty());
        assert!(thread.validate().is_ok());
    }
}

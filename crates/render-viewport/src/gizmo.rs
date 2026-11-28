use core_math::{vec3, Quat, Ray, Vec3};

/// Transform manipulation axes and planar handles
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GizmoAxis {
    X,
    Y,
    Z,
    XY,
    XZ,
    YZ,
    Screen,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GizmoMode {
    Translate,
    Rotate,
    Scale,
}

/// Transform orientation / coordinate space (Global, Local, Normal, View)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TransformOrientation {
    #[default]
    Global,
    Local,
    Normal,
    View,
}

pub struct TransformGizmo {
    pub mode: GizmoMode,
    pub length: f32,
    pub hit_radius: f32,
}

impl Default for TransformGizmo {
    fn default() -> Self {
        Self {
            mode: GizmoMode::Translate,
            length: 1.5,
            hit_radius: 0.22,
        }
    }
}

impl TransformGizmo {
    pub fn new() -> Self {
        Self::default()
    }

    /// Compute closest distance between a Ray and a 3D line segment (p0 to p1)
    fn ray_segment_distance(ray: &Ray, p0: Vec3, p1: Vec3) -> (f32, f32) {
        let u = ray.direction;
        let v = p1 - p0;
        let w = ray.origin - p0;

        let a = u.dot(u);
        let b = u.dot(v);
        let c = v.dot(v);
        let d = u.dot(w);
        let e = v.dot(w);

        let denom = a * c - b * b;
        let (s, mut t) = if denom.abs() < 1e-6 {
            (0.0, if b > c { d / b } else { e / c })
        } else {
            ((b * e - c * d) / denom, (a * e - b * d) / denom)
        };

        t = t.clamp(0.0, 1.0);
        let s = s.max(0.0);

        let pt_ray = ray.origin + u * s;
        let pt_seg = p0 + v * t;

        ((pt_ray - pt_seg).length(), t)
    }

    /// Distance between ray and a point
    fn ray_point_distance(ray: &Ray, point: Vec3) -> f32 {
        let w = point - ray.origin;
        let c1 = w.dot(ray.direction);
        if c1 <= 0.0 {
            return w.length();
        }
        let b = ray.origin + ray.direction * c1;
        (point - b).length()
    }

    /// Test if a screen ray hovers/hits any gizmo axis or plane handle
    pub fn hit_test(&self, ray: &Ray, origin: Vec3, orientation: Quat) -> Option<GizmoAxis> {
        self.hit_test_scaled(ray, origin, orientation, 1.0)
    }

    /// Test if a screen ray hovers/hits gizmo with adaptive screen-space scaling
    pub fn hit_test_scaled(
        &self,
        ray: &Ray,
        origin: Vec3,
        orientation: Quat,
        scale: f32,
    ) -> Option<GizmoAxis> {
        let len = self.length * scale;
        let hit_r = self.hit_radius * scale;

        // 1. Center screen handle test
        let center_dist = Self::ray_point_distance(ray, origin);
        if center_dist < 0.10 * scale {
            return Some(GizmoAxis::Screen);
        }

        // 2. Axis rays test (priority)
        let x_end = origin + orientation * vec3(len, 0.0, 0.0);
        let y_end = origin + orientation * vec3(0.0, len, 0.0);
        let z_end = origin + orientation * vec3(0.0, 0.0, len);

        let (dist_x, _) = Self::ray_segment_distance(ray, origin, x_end);
        let (dist_y, _) = Self::ray_segment_distance(ray, origin, y_end);
        let (dist_z, _) = Self::ray_segment_distance(ray, origin, z_end);

        let mut closest = None;
        let mut min_dist = hit_r;

        if dist_x < min_dist {
            min_dist = dist_x;
            closest = Some(GizmoAxis::X);
        }
        if dist_y < min_dist {
            min_dist = dist_y;
            closest = Some(GizmoAxis::Y);
        }
        if dist_z < min_dist {
            closest = Some(GizmoAxis::Z);
        }

        if closest.is_some() {
            return closest;
        }

        // 3. Planar square handles test (Translate & Scale modes)
        if self.mode != GizmoMode::Rotate {
            let plane_offset = 0.35 * len;
            let plane_size = 0.14 * len;

            let xy_center = origin + orientation * Vec3::new(plane_offset, plane_offset, 0.0);
            let xz_center = origin + orientation * Vec3::new(plane_offset, 0.0, plane_offset);
            let yz_center = origin + orientation * Vec3::new(0.0, plane_offset, plane_offset);

            let d_xy = Self::ray_point_distance(ray, xy_center);
            let d_xz = Self::ray_point_distance(ray, xz_center);
            let d_yz = Self::ray_point_distance(ray, yz_center);

            if d_xy < plane_size && d_xy < d_xz && d_xy < d_yz {
                return Some(GizmoAxis::XY);
            }
            if d_xz < plane_size && d_xz < d_xy && d_xz < d_yz {
                return Some(GizmoAxis::XZ);
            }
            if d_yz < plane_size {
                return Some(GizmoAxis::YZ);
            }
        }

        None
    }

    /// Project mouse drag vector along the given axis or plane for Translation oriented by `orientation`.
    pub fn calculate_translation_delta(
        axis: GizmoAxis,
        orientation: Quat,
        camera_right: Vec3,
        camera_up: Vec3,
        dx: f32,
        dy: f32,
        distance_to_camera: f32,
    ) -> Vec3 {
        let mouse_world_motion = camera_right * dx - camera_up * dy;
        let scale = distance_to_camera * 0.0015;

        match axis {
            GizmoAxis::X => {
                let axis_dir = orientation * Vec3::X;
                let projected = mouse_world_motion.dot(axis_dir);
                axis_dir * (projected * scale)
            }
            GizmoAxis::Y => {
                let axis_dir = orientation * Vec3::Y;
                let projected = mouse_world_motion.dot(axis_dir);
                axis_dir * (projected * scale)
            }
            GizmoAxis::Z => {
                let axis_dir = orientation * Vec3::Z;
                let projected = mouse_world_motion.dot(axis_dir);
                axis_dir * (projected * scale)
            }
            GizmoAxis::XY => {
                let dir_x = orientation * Vec3::X;
                let dir_y = orientation * Vec3::Y;
                let proj_x = mouse_world_motion.dot(dir_x);
                let proj_y = mouse_world_motion.dot(dir_y);
                (dir_x * proj_x + dir_y * proj_y) * scale
            }
            GizmoAxis::XZ => {
                let dir_x = orientation * Vec3::X;
                let dir_z = orientation * Vec3::Z;
                let proj_x = mouse_world_motion.dot(dir_x);
                let proj_z = mouse_world_motion.dot(dir_z);
                (dir_x * proj_x + dir_z * proj_z) * scale
            }
            GizmoAxis::YZ => {
                let dir_y = orientation * Vec3::Y;
                let dir_z = orientation * Vec3::Z;
                let proj_y = mouse_world_motion.dot(dir_y);
                let proj_z = mouse_world_motion.dot(dir_z);
                (dir_y * proj_y + dir_z * proj_z) * scale
            }
            GizmoAxis::Screen => mouse_world_motion * scale,
        }
    }

    /// Calculate rotation delta quaternion from mouse drag around the oriented axis.
    pub fn calculate_rotation_delta(axis: GizmoAxis, orientation: Quat, dx: f32, dy: f32) -> Quat {
        let angle = (dx + dy) * 0.015;
        let local_axis = match axis {
            GizmoAxis::X | GizmoAxis::YZ => Vec3::X,
            GizmoAxis::Y | GizmoAxis::XZ => Vec3::Y,
            GizmoAxis::Z | GizmoAxis::XY | GizmoAxis::Screen => Vec3::Z,
        };
        let world_axis = (orientation * local_axis).normalize_or_zero();
        Quat::from_axis_angle(world_axis, angle)
    }

    /// Calculate scaling factor delta from mouse drag
    pub fn calculate_scale_delta(axis: GizmoAxis, dx: f32, dy: f32) -> Vec3 {
        let delta = (dx - dy) * 0.01;
        match axis {
            GizmoAxis::X => vec3(delta, 0.0, 0.0),
            GizmoAxis::Y => vec3(0.0, delta, 0.0),
            GizmoAxis::Z => vec3(0.0, 0.0, delta),
            GizmoAxis::XY => vec3(delta, delta, 0.0),
            GizmoAxis::XZ => vec3(delta, 0.0, delta),
            GizmoAxis::YZ => vec3(0.0, delta, delta),
            GizmoAxis::Screen => Vec3::splat(delta),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core_math::{Quat, Ray, Vec3};

    #[test]
    fn test_gizmo_translation_delta_global_vs_oriented() {
        let cam_right = Vec3::new(1.0, 0.0, 1.0).normalize();
        let cam_up = Vec3::Y;

        let delta_global = TransformGizmo::calculate_translation_delta(
            GizmoAxis::X,
            Quat::IDENTITY,
            cam_right,
            cam_up,
            10.0,
            0.0,
            10.0,
        );
        assert!(delta_global.x > 0.0);
        assert_eq!(delta_global.y, 0.0);
        assert_eq!(delta_global.z, 0.0);

        let rot_y_90 = Quat::from_rotation_y(std::f32::consts::FRAC_PI_2);
        let delta_oriented = TransformGizmo::calculate_translation_delta(
            GizmoAxis::X,
            rot_y_90,
            cam_right,
            cam_up,
            10.0,
            0.0,
            10.0,
        );
        assert!(delta_oriented.z.abs() > 0.0);
    }

    #[test]
    fn test_gizmo_rotation_delta_oriented() {
        let rot_id =
            TransformGizmo::calculate_rotation_delta(GizmoAxis::X, Quat::IDENTITY, 1.0, 1.0);
        assert!((rot_id.x.abs() - 0.0149).abs() < 1e-3);

        let rot_y_90 = Quat::from_rotation_y(std::f32::consts::FRAC_PI_2);
        let rot_oriented =
            TransformGizmo::calculate_rotation_delta(GizmoAxis::X, rot_y_90, 1.0, 1.0);
        assert!(rot_oriented.z.abs() > 0.0);
    }

    #[test]
    fn test_gizmo_hit_test_oriented() {
        let gizmo = TransformGizmo::new();
        let origin = Vec3::ZERO;
        let ray_x = Ray::new(Vec3::new(0.5, 0.05, 0.05), Vec3::new(0.0, 0.0, -1.0));
        assert_eq!(
            gizmo.hit_test(&ray_x, origin, Quat::IDENTITY),
            Some(GizmoAxis::X)
        );

        let rot_y_90 = Quat::from_rotation_y(std::f32::consts::FRAC_PI_2);
        let ray_rot = Ray::new(Vec3::new(0.05, 0.05, -0.5), Vec3::new(1.0, 0.0, 0.0));
        assert_eq!(
            gizmo.hit_test(&ray_rot, origin, rot_y_90),
            Some(GizmoAxis::X)
        );
    }
}

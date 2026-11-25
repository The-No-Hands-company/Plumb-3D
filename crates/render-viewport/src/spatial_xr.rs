//! OpenXR Spatial Computing & 6-DoF Immersive Modeling Engine
//!
//! Provides stereoscopic dual-eye rendering matrices, 6-DoF motion controller tracking,
//! in-air 3D curve sketching, and 1:1 room-scale spatial inspection.

use core_math::{Mat4, Ray, Transform, Vec2, Vec3};

/// Active spatial tool in the immersive VR workspace
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SpatialToolMode {
    #[default]
    Inspect1To1,
    AirSketchCurve,
    SpatialTransform,
    LaserPointer,
}

impl SpatialToolMode {
    pub fn name(&self) -> &'static str {
        match self {
            SpatialToolMode::Inspect1To1 => "1:1 Real-Scale Inspection",
            SpatialToolMode::AirSketchCurve => "In-Air Curve Sketcher",
            SpatialToolMode::SpatialTransform => "6-DoF Direct Grab & Move",
            SpatialToolMode::LaserPointer => "Far-Field Laser Pointer",
        }
    }
}

/// 6-DoF spatial motion controller state
#[derive(Debug, Clone)]
pub struct SpatialController {
    /// 3D pose of the controller relative to room origin
    pub pose: Transform,
    /// Analog trigger depression in [0, 1]
    pub trigger: f32,
    /// Grip squeeze value in [0, 1]
    pub grip: f32,
    /// 2-axis thumbstick input
    pub thumbstick: Vec2,
    /// Primary face button (A / X)
    pub primary_button: bool,
    /// Secondary face button (B / Y)
    pub secondary_button: bool,
    /// Currently recording an in-air stroke
    pub is_drawing: bool,
    /// Points captured during the active in-air stroke
    pub stroke_points: Vec<Vec3>,
}

impl Default for SpatialController {
    fn default() -> Self {
        Self {
            pose: Transform::IDENTITY,
            trigger: 0.0,
            grip: 0.0,
            thumbstick: Vec2::ZERO,
            primary_button: false,
            secondary_button: false,
            is_drawing: false,
            stroke_points: Vec::new(),
        }
    }
}

impl SpatialController {
    /// Computes the pointing laser ray originating from the controller nose
    pub fn pointing_ray(&self) -> Ray {
        let origin = self.pose.translation;
        let direction = (self.pose.rotation * Vec3::NEG_Z).normalize();
        Ray::new(origin, direction)
    }
}

/// Stereoscopic dual-eye camera rig for OpenXR HMDs
#[derive(Debug, Clone)]
pub struct StereoRig {
    /// Interpupillary distance (IPD) in meters (default: 63mm)
    pub ipd_meters: f32,
    /// Head center pose in room space
    pub head_pose: Transform,
    /// Left motion controller
    pub left_controller: SpatialController,
    /// Right motion controller
    pub right_controller: SpatialController,
    /// Computed left eye world position
    pub left_eye_pos: Vec3,
    /// Computed right eye world position
    pub right_eye_pos: Vec3,
    /// Left eye view matrix
    pub left_view_matrix: Mat4,
    /// Right eye view matrix
    pub right_view_matrix: Mat4,
    /// Stereoscopic projection matrix
    pub projection_matrix: Mat4,
}

impl Default for StereoRig {
    fn default() -> Self {
        let mut rig = Self {
            ipd_meters: 0.063,
            head_pose: Transform::from_translation(Vec3::new(0.0, 1.7, 0.0)), // Average eye height 1.7m
            left_controller: SpatialController {
                pose: Transform::from_translation(Vec3::new(-0.25, 1.2, -0.4)),
                ..Default::default()
            },
            right_controller: SpatialController {
                pose: Transform::from_translation(Vec3::new(0.25, 1.2, -0.4)),
                ..Default::default()
            },
            left_eye_pos: Vec3::ZERO,
            right_eye_pos: Vec3::ZERO,
            left_view_matrix: Mat4::IDENTITY,
            right_view_matrix: Mat4::IDENTITY,
            projection_matrix: Mat4::IDENTITY,
        };
        rig.update_stereo_views(1.0, std::f32::consts::FRAC_PI_2); // 90° FOV per eye
        rig
    }
}

impl StereoRig {
    /// Recomputes stereoscopic view and projection matrices
    pub fn update_stereo_views(&mut self, aspect_ratio: f32, fov_y_rad: f32) {
        let head_rot = self.head_pose.rotation;
        let right_vec = head_rot * Vec3::X;
        let half_ipd = right_vec * (self.ipd_meters * 0.5);

        self.left_eye_pos = self.head_pose.translation - half_ipd;
        self.right_eye_pos = self.head_pose.translation + half_ipd;

        let forward = head_rot * Vec3::NEG_Z;
        let up = head_rot * Vec3::Y;

        self.left_view_matrix = Mat4::look_at_rh(self.left_eye_pos, self.left_eye_pos + forward, up);
        self.right_view_matrix = Mat4::look_at_rh(self.right_eye_pos, self.right_eye_pos + forward, up);
        self.projection_matrix = Mat4::perspective_rh(fov_y_rad, aspect_ratio, 0.05, 500.0);
    }
}

/// Manages an active OpenXR spatial computing session
#[derive(Debug, Clone)]
pub struct SpatialModelingSession {
    /// Whether VR / OpenXR stereoscopic mode is active
    pub is_active: bool,
    /// Dual-eye and 6-DoF controller spatial rig
    pub rig: StereoRig,
    /// Origin offset in virtual room space
    pub room_origin: Vec3,
    /// World scale multiplier (1.0 = 1:1 true scale, 0.1 = tabletop miniature)
    pub world_scale: f32,
    /// Active spatial modeling tool
    pub active_tool: SpatialToolMode,
    /// Completed in-air sketched 3D spline curves
    pub sketched_curves: Vec<Vec<Vec3>>,
    /// Laser pointer collision point on scene geometry
    pub laser_hit: Option<Vec3>,
}

impl Default for SpatialModelingSession {
    fn default() -> Self {
        Self {
            is_active: false,
            rig: StereoRig::default(),
            room_origin: Vec3::ZERO,
            world_scale: 1.0,
            active_tool: SpatialToolMode::Inspect1To1,
            sketched_curves: Vec::new(),
            laser_hit: None,
        }
    }
}

impl SpatialModelingSession {
    /// Creates a new spatial modeling session
    pub fn new() -> Self {
        Self::default()
    }

    /// Toggles stereoscopic VR spatial computing mode
    pub fn toggle_xr(&mut self) -> bool {
        self.is_active = !self.is_active;
        self.is_active
    }

    /// Sets world scale factor (e.g. 1.0 for life-size CAD inspection, 0.1 for miniature)
    pub fn set_world_scale(&mut self, scale: f32) {
        self.world_scale = scale.clamp(0.01, 100.0);
    }

    /// Advances spatial inputs, tracking in-air curve sketching and triggers
    pub fn update_inputs(&mut self, trigger_pressed: bool) {
        if self.active_tool == SpatialToolMode::AirSketchCurve {
            let ctrl = &mut self.rig.right_controller;
            ctrl.trigger = if trigger_pressed { 1.0 } else { 0.0 };

            if trigger_pressed {
                let pt = ctrl.pose.translation;
                if !ctrl.is_drawing {
                    ctrl.is_drawing = true;
                    ctrl.stroke_points.clear();
                    ctrl.stroke_points.push(pt);
                } else {
                    // Append point if moved more than 1.5 cm to avoid redundant vertices
                    if let Some(&last) = ctrl.stroke_points.last() {
                        if (pt - last).length() > 0.015 {
                            ctrl.stroke_points.push(pt);
                        }
                    }
                }
            } else if ctrl.is_drawing {
                ctrl.is_drawing = false;
                if ctrl.stroke_points.len() >= 2 {
                    self.sketched_curves.push(ctrl.stroke_points.clone());
                }
                ctrl.stroke_points.clear();
            }
        }
    }

    /// Converts finished in-air sketched strokes to a polygonal mesh
    pub fn generate_air_curve_mesh(&self, stroke_idx: usize, tube_radius: f32) -> Option<geo_kernel::Mesh> {
        let pts = self.sketched_curves.get(stroke_idx)?;
        if pts.len() < 2 {
            return None;
        }

        let mut spline = geo_kernel::Spline::new();
        for &p in pts {
            spline.add_point(geo_kernel::BezierPoint::smooth(p, Vec3::ZERO));
        }

        let circle = geo_kernel::Spline::circle_cross_section(tube_radius, 8);
        let sampled = spline.sample(8 * pts.len());
        geo_kernel::Spline::sweep(&circle, &sampled, false).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stereo_rig_ipd_eye_separation() {
        let mut rig = StereoRig::default();
        rig.ipd_meters = 0.065; // 65mm
        rig.head_pose = Transform::from_translation(Vec3::new(0.0, 1.8, 0.0));
        rig.update_stereo_views(1.0, 1.57);

        assert!((rig.left_eye_pos.x - (-0.0325)).abs() < 1e-4);
        assert!((rig.right_eye_pos.x - 0.0325).abs() < 1e-4);
        assert_eq!(rig.left_eye_pos.y, 1.8);
        assert_eq!(rig.right_eye_pos.y, 1.8);
    }

    #[test]
    fn test_in_air_curve_sketching() {
        let mut session = SpatialModelingSession::new();
        session.active_tool = SpatialToolMode::AirSketchCurve;

        session.rig.right_controller.pose.translation = Vec3::new(0.0, 1.2, -0.5);
        session.update_inputs(true);

        session.rig.right_controller.pose.translation = Vec3::new(0.1, 1.2, -0.5);
        session.update_inputs(true);

        session.rig.right_controller.pose.translation = Vec3::new(0.2, 1.2, -0.5);
        session.update_inputs(true);

        session.update_inputs(false); // Release trigger

        assert_eq!(session.sketched_curves.len(), 1);
        assert_eq!(session.sketched_curves[0].len(), 3);

        let mesh = session.generate_air_curve_mesh(0, 0.02).unwrap();
        assert!(mesh.vertex_count() > 0);
        assert!(mesh.face_count() > 0);
    }

    #[test]
    fn test_spatial_scale_adjustment() {
        let mut session = SpatialModelingSession::new();
        assert_eq!(session.world_scale, 1.0);

        session.set_world_scale(0.1);
        assert_eq!(session.world_scale, 0.1);

        session.set_world_scale(0.0001); // Clamped
        assert_eq!(session.world_scale, 0.01);
    }
}

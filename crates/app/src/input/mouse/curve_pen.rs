//! Interactive Bézier Curve Pen Tool input handler.
//! Allows clicking and dragging to place smooth or sharp 3D Bézier spline curves in the viewport.

use crate::commands::CreateCurveNodeCommand;
use crate::state::AppState;
use core_math::Vec3;
use geo_kernel::{BezierPoint, Spline};

impl AppState {
    /// Commits the in-progress Curve Pen Bézier knots into a new Curve SceneNode.
    pub fn commit_curve_pen(&mut self) {
        if self.curve_pen_points.len() < 2 {
            self.curve_pen_active = false;
            self.curve_pen_points.clear();
            self.curve_pen_drag_origin = None;
            return;
        }

        let pts = std::mem::take(&mut self.curve_pen_points);
        let spline = Spline::with_points(pts, self.curve_pen_is_closed);
        let cmd = Box::new(CreateCurveNodeCommand {
            name: "Pen_Curve".into(),
            spline,
            created_node_id: None,
        });

        if let Ok(()) = self.history.execute(cmd, &mut self.scene) {
            if let Some(id) = self.scene.nodes.keys().last() {
                self.selected_node = Some(id);
            }
        }

        self.curve_pen_active = false;
        self.curve_pen_is_closed = false;
        self.curve_pen_drag_origin = None;
        self.window.request_redraw();
    }

    /// Handles mouse clicks when Curve Pen Tool is active.
    pub fn handle_curve_pen_click(&mut self, mx: f64, my: f64) {
        let ray = self.get_screen_ray(mx, my);
        let pt = if self.live_surface_snap {
            let mut best_hit = None;
            for node in self.scene.nodes.values() {
                if let graph_engine::NodeData::Mesh { base_mesh, .. } = &node.data {
                    if let Some(hit) = base_mesh.raycast(&ray, &node.world_matrix) {
                        if best_hit.as_ref().is_none_or(|b: &geo_kernel::RaycastHit| hit.distance < b.distance) {
                            best_hit = Some(hit);
                        }
                    }
                }
            }
            if let Some(hit) = best_hit {
                hit.point + hit.normal * self.live_surface_offset
            } else if ray.direction.y.abs() > 1e-4 {
                let t = -ray.origin.y / ray.direction.y;
                if t > 0.0 { ray.origin + ray.direction * t } else { ray.origin + ray.direction * 5.0 }
            } else {
                ray.origin + ray.direction * 5.0
            }
        } else if ray.direction.y.abs() > 1e-4 {
            let t = -ray.origin.y / ray.direction.y;
            if t > 0.0 {
                ray.origin + ray.direction * t
            } else {
                ray.origin + ray.direction * 5.0
            }
        } else {
            ray.origin + ray.direction * 5.0
        };

        // If snapping is active, snap to 0.5m grid increments
        let pt = if self.snap_enabled ^ self.ctrl_pressed {
            Vec3::new((pt.x * 2.0).round() * 0.5, 0.0, (pt.z * 2.0).round() * 0.5)
        } else {
            pt
        };

        // If clicking within 0.35m of the first point, close loop and commit
        if self.curve_pen_points.len() >= 2
            && (pt - self.curve_pen_points[0].position).length() < 0.35
        {
            self.curve_pen_is_closed = true;
            self.commit_curve_pen();
        } else {
            self.curve_pen_points.push(BezierPoint::sharp(pt));
            self.curve_pen_drag_origin = Some(pt);
            self.window.request_redraw();
        }
    }

    /// Updates tangent handles while dragging newly placed curve anchor.
    pub fn handle_curve_pen_drag(&mut self, mx: f64, my: f64) {
        if let Some(origin) = self.curve_pen_drag_origin {
            let ray = self.get_screen_ray(mx, my);
            let pt = if ray.direction.y.abs() > 1e-4 {
                let t = -ray.origin.y / ray.direction.y;
                if t > 0.0 {
                    ray.origin + ray.direction * t
                } else {
                    ray.origin + ray.direction * 5.0
                }
            } else {
                ray.origin + ray.direction * 5.0
            };

            let handle = pt - origin;
            if let Some(last_pt) = self.curve_pen_points.last_mut() {
                last_pt.handle_out = handle;
                last_pt.handle_in = -handle;
                self.window.request_redraw();
            }
        }
    }
}

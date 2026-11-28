//! Mouse interaction handler for 2D CAD Constraint Sketcher.
//!
//! Handles viewport clicks and dragging to draw lines, rectangles, circles,
//! and place live dimensions on the active workplane.

use core_math::vec2;
use geo_kernel::ConstraintType;
use graph_engine::NodeData;

use crate::commands::modeling::sketch::{
    AddSketchConstraintCommand, AddSketchLineCommand, CreateSketchNodeCommand,
};
use crate::state::AppState;
use crate::types::SketchTool;
use crate::ui::overlays::sketch_overlay::find_active_sketch_node;

impl AppState {
    /// Handles mouse clicks when CAD Sketcher is active.
    /// Returns true if the click was consumed by the sketcher.
    pub fn handle_sketch_click(&mut self, mx: f64, my: f64) -> bool {
        let node_id = match find_active_sketch_node(self) {
            Some(id) => id,
            None => {
                // Automatically initialize a new sketch node on the active workplane
                let cmd = Box::new(CreateSketchNodeCommand {
                    name: "Sketch_1".into(),
                    workplane: self.workplane,
                    created_node_id: None,
                });
                if self.history.execute(cmd, &mut self.scene).is_ok() {
                    if let Some(id) = self.scene.nodes.keys().last() {
                        self.selected_node = Some(id);
                        id
                    } else {
                        return false;
                    }
                } else {
                    return false;
                }
            }
        };

        let wp = {
            let node = match self.scene.nodes.get(node_id) {
                Some(n) => n,
                None => return false,
            };
            match &node.data {
                NodeData::Sketch { sketch, .. } => sketch.workplane,
                _ => return false,
            }
        };

        let ray = self.get_screen_ray(mx, my);
        let raw_uv = match wp.intersect_ray_uv(&ray) {
            Some(uv) => uv,
            None => return false,
        };

        // Grid snapping (5mm increments if snap is enabled)
        let uv = if self.snap_enabled ^ self.ctrl_pressed {
            vec2(
                (raw_uv.x / 5.0).round() * 5.0,
                (raw_uv.y / 5.0).round() * 5.0,
            )
        } else {
            raw_uv
        };

        match self.sketch_tool {
            SketchTool::Select => {
                self.sketch_in_progress_point = None;
                false
            }

            SketchTool::Line => {
                if let Some(p_start) = self.sketch_in_progress_point.take() {
                    let cmd = Box::new(AddSketchLineCommand::new(node_id, p_start, uv));
                    let _ = self.history.execute(cmd, &mut self.scene);
                    self.set_status_message(format!(
                        "CAD Line added: ({:.1}, {:.1}) -> ({:.1}, {:.1})",
                        p_start.x, p_start.y, uv.x, uv.y
                    ));
                    self.window.request_redraw();
                } else {
                    self.sketch_in_progress_point = Some(uv);
                    self.set_status_message(format!("Start point: ({:.1}, {:.1})", uv.x, uv.y));
                    self.window.request_redraw();
                }
                true
            }

            SketchTool::Rectangle => {
                if let Some(p_min) = self.sketch_in_progress_point.take() {
                    let p_max = uv;
                    let p0 = p_min;
                    let p1 = vec2(p_max.x, p_min.y);
                    let p2 = p_max;
                    let p3 = vec2(p_min.x, p_max.y);

                    // Add 4 lines
                    let _ = self.history.execute(
                        Box::new(AddSketchLineCommand::new(node_id, p0, p1)),
                        &mut self.scene,
                    );
                    let _ = self.history.execute(
                        Box::new(AddSketchLineCommand::new(node_id, p1, p2)),
                        &mut self.scene,
                    );
                    let _ = self.history.execute(
                        Box::new(AddSketchLineCommand::new(node_id, p2, p3)),
                        &mut self.scene,
                    );
                    let _ = self.history.execute(
                        Box::new(AddSketchLineCommand::new(node_id, p3, p0)),
                        &mut self.scene,
                    );

                    self.set_status_message(format!(
                        "CAD Rectangle added: {:.1}mm x {:.1}mm",
                        (p_max.x - p_min.x).abs(),
                        (p_max.y - p_min.y).abs()
                    ));
                    self.window.request_redraw();
                } else {
                    self.sketch_in_progress_point = Some(uv);
                    self.set_status_message(format!(
                        "Rectangle origin: ({:.1}, {:.1})",
                        uv.x, uv.y
                    ));
                    self.window.request_redraw();
                }
                true
            }

            SketchTool::Circle => {
                if let Some(center) = self.sketch_in_progress_point.take() {
                    let radius = (uv - center).length().max(1.0);
                    if let Some(node) = self.scene.nodes.get_mut(node_id) {
                        if let NodeData::Sketch {
                            sketch,
                            cached_render_mesh,
                        } = &mut node.data
                        {
                            let cp = sketch.add_point(center);
                            sketch.add_circle(cp, radius);
                            *cached_render_mesh = None;
                            self.scene.mark_dirty(node_id);
                        }
                    }
                    self.set_status_message(format!("CAD Circle added: R = {:.1}mm", radius));
                    self.window.request_redraw();
                } else {
                    self.sketch_in_progress_point = Some(uv);
                    self.set_status_message(format!("Circle center: ({:.1}, {:.1})", uv.x, uv.y));
                    self.window.request_redraw();
                }
                true
            }

            SketchTool::Dimension => {
                // Automatically add a distance dimension between the last two points
                if let Some(node) = self.scene.nodes.get(node_id) {
                    if let NodeData::Sketch { sketch, .. } = &node.data {
                        let n_pts = sketch.points.len();
                        if n_pts >= 2 {
                            let pt_a = sketch.points[n_pts - 2].id;
                            let pt_b = sketch.points[n_pts - 1].id;
                            let pa = sketch.points[n_pts - 2].pos;
                            let pb = sketch.points[n_pts - 1].pos;
                            let dist = (pb - pa).length();

                            let c_type = ConstraintType::Distance {
                                pt_a,
                                pt_b,
                                distance: dist,
                            };
                            let cmd = Box::new(AddSketchConstraintCommand::new(node_id, c_type));
                            let _ = self.history.execute(cmd, &mut self.scene);
                            self.set_status_message(format!("Live Dimension added: {:.2}mm", dist));
                            self.window.request_redraw();
                        }
                    }
                }
                true
            }

            _ => false,
        }
    }
}

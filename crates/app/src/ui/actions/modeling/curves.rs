use geo_kernel::{EdgeKey, ProfileShape};

use crate::commands::*;
use crate::state::AppState;
use crate::types::InteractionMode;
use crate::ui::actions::UiActions;

pub fn handle_curve_actions(state: &mut AppState, actions: &mut UiActions) {
    // Handle Convert Curve to Mesh
    if actions.trigger_convert_curve {
        if let Some(sel_id) = state.selected_node {
            let cmd = Box::new(ConvertCurveToMeshCommand {
                node_id: sel_id,
                old_spline: None,
                old_bevel_radius: 0.05,
                old_bevel_resolution: 8,
                old_profile_shape: ProfileShape::Circle,
            });
            if let Err(e) = state.history.execute(cmd, &mut state.scene) {
                eprintln!("Convert curve failed: {}", e);
            } else {
                state.interaction_mode = InteractionMode::Object;
            }
        }
    }

    // Handle Extract Curve from Edges
    if actions.trigger_extract_curve {
        if let Some(sel_id) = state.selected_node {
            let edges: Vec<EdgeKey> = state.selected_edges.iter().copied().collect();
            if !edges.is_empty() {
                let cmd = Box::new(ExtractCurveFromEdgesCommand {
                    source_node_id: sel_id,
                    edges,
                    created_node_id: None,
                });
                if let Err(e) = state.history.execute(cmd, &mut state.scene) {
                    eprintln!("Extract curve failed: {}", e);
                }
            }
        }
    }

    // Handle CAD Fillet Curve Corners
    if let Some(radius) = actions.trigger_fillet_curve.take() {
        if let Some(sel_id) = state.selected_node {
            let cmd = Box::new(FilletCurveCommand::all(sel_id, radius));
            if let Err(e) = state.history.execute(cmd, &mut state.scene) {
                eprintln!("Fillet curve failed: {}", e);
            }
        }
    }

    // Handle CAD Fillet Selected Curve Knot
    if let Some(radius) = actions.trigger_fillet_selected_curve.take() {
        if let Some(sel_id) = state.selected_node {
            if let Some(sel_knot) = state.selected_curve_knot {
                let cmd = Box::new(FilletCurveCommand::selected(sel_id, radius, vec![sel_knot]));
                if let Err(e) = state.history.execute(cmd, &mut state.scene) {
                    eprintln!("Fillet selected knot failed: {}", e);
                }
            }
        }
    }

    // Handle CAD Revolve / Lathe Curve
    if let Some((segments, angle_deg)) = actions.trigger_lathe_curve.take() {
        if let Some(sel_id) = state.selected_node {
            let cmd = Box::new(LatheCurveCommand {
                curve_node_id: sel_id,
                segments,
                angle_degrees: angle_deg,
                created_node_id: None,
            });
            if let Err(e) = state.history.execute(cmd, &mut state.scene) {
                eprintln!("Lathe curve failed: {}", e);
            }
        }
    }

    // Handle CAD Boolean Curve Trimming
    if let Some(keep_inside) = actions.trigger_trim_curves.take() {
        let curve_ids: Vec<_> = state
            .selected_nodes
            .iter()
            .copied()
            .filter(|&id| {
                state
                    .scene
                    .nodes
                    .get(id)
                    .is_some_and(|n| matches!(n.data, graph_engine::NodeData::Curve { .. }))
            })
            .collect();

        if curve_ids.len() >= 2 {
            let target_id = curve_ids[0];
            let cutter_id = curve_ids[1];
            let cmd = Box::new(TrimCurveCommand::new(
                target_id,
                cutter_id,
                keep_inside,
                0.05,
            ));
            match state.history.execute(cmd, &mut state.scene) {
                Ok(()) => {
                    state.set_status_message(format!(
                        "CAD Curve Trimming applied (keep_inside: {})! (Ctrl+Z to Undo)",
                        keep_inside
                    ));
                }
                Err(e) => {
                    state.set_status_message(format!("Curve Trim failed: {}", e));
                }
            }
        } else {
            state.set_status_message("Select 2 curves (target and cutter) to trim");
        }
    }

    // Handle CAD Connect Tangent Arc
    if actions.trigger_connect_tangent_arc {
        let curve_ids: Vec<_> = state
            .selected_nodes
            .iter()
            .copied()
            .filter(|&id| {
                state
                    .scene
                    .nodes
                    .get(id)
                    .is_some_and(|n| matches!(n.data, graph_engine::NodeData::Curve { .. }))
            })
            .collect();

        if curve_ids.len() >= 2 {
            let cmd = Box::new(ConnectTangentArcCommand::new(
                curve_ids[0],
                usize::MAX,
                curve_ids[1],
                0,
            ));
            match state.history.execute(cmd, &mut state.scene) {
                Ok(()) => {
                    state.set_status_message(
                        "CAD Tangent Arc connected with G1 continuity! (Ctrl+Z to Undo)",
                    );
                }
                Err(e) => {
                    state.set_status_message(format!("Tangent Arc failed: {}", e));
                }
            }
        } else if let Some(sel_id) = state.selected_node {
            let knot_a = state.selected_curve_knot.unwrap_or(0);
            let knot_b = knot_a.saturating_add(1);
            let cmd = Box::new(ConnectTangentArcCommand::new(
                sel_id, knot_a, sel_id, knot_b,
            ));
            match state.history.execute(cmd, &mut state.scene) {
                Ok(()) => {
                    state.set_status_message("CAD Tangent Arc connected! (Ctrl+Z to Undo)");
                }
                Err(e) => {
                    state.set_status_message(format!("Tangent Arc failed: {}", e));
                }
            }
        }
    }
}

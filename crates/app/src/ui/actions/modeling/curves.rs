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
            let cmd = Box::new(FilletCurveCommand {
                node_id: sel_id,
                radius,
                old_spline: None,
            });
            if let Err(e) = state.history.execute(cmd, &mut state.scene) {
                eprintln!("Fillet curve failed: {}", e);
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
}

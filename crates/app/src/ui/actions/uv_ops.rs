use geo_kernel::{EdgeKey, ProjectionAxis};
use graph_engine::NodeData;

use super::UiActions;
use crate::commands::{
    FlipUvsCommand, MarkSeamCommand, PackUvIslandsCommand, RelaxUvsCommand, RotateUvsCommand, ScaleUvsCommand,
    SmartUvProjectCommand, UVProjectionCommand, UnwrapSeamsCommand,
};
use crate::state::AppState;
use crate::types::UVProjectionType;

pub fn handle_uv_ops(state: &mut AppState, actions: &mut UiActions) {
    // Handle UV Projections
    if actions.trigger_planar_uv {
        if let Some(sel_id) = state.selected_node {
            let cmd = Box::new(UVProjectionCommand {
                node_id: sel_id,
                projection: UVProjectionType::Planar(ProjectionAxis::Y),
                old_mesh: None,
            });
            let _ = state.history.execute(cmd, &mut state.scene);
        }
    }
    if actions.trigger_box_uv {
        if let Some(sel_id) = state.selected_node {
            let cmd = Box::new(UVProjectionCommand {
                node_id: sel_id,
                projection: UVProjectionType::Box,
                old_mesh: None,
            });
            let _ = state.history.execute(cmd, &mut state.scene);
        }
    }
    if actions.trigger_cylindrical_uv {
        if let Some(sel_id) = state.selected_node {
            let cmd = Box::new(UVProjectionCommand {
                node_id: sel_id,
                projection: UVProjectionType::Cylindrical(ProjectionAxis::Y),
                old_mesh: None,
            });
            let _ = state.history.execute(cmd, &mut state.scene);
        }
    }
    if actions.trigger_spherical_uv {
        if let Some(sel_id) = state.selected_node {
            let cmd = Box::new(UVProjectionCommand {
                node_id: sel_id,
                projection: UVProjectionType::Spherical,
                old_mesh: None,
            });
            let _ = state.history.execute(cmd, &mut state.scene);
        }
    }
    if actions.trigger_unwrap_seams {
        if let Some(sel_id) = state.selected_node {
            let cmd = Box::new(UnwrapSeamsCommand {
                node_id: sel_id,
                margin: state.smart_uv_margin,
                old_mesh: None,
            });
            let _ = state.history.execute(cmd, &mut state.scene);
        }
    }
    if actions.trigger_smart_uv_project {
        if let Some(sel_id) = state.selected_node {
            let cmd = Box::new(SmartUvProjectCommand {
                node_id: sel_id,
                angle_threshold_deg: state.smart_uv_angle_threshold,
                margin: state.smart_uv_margin,
                old_mesh: None,
            });
            let _ = state.history.execute(cmd, &mut state.scene);
        }
    }
    if actions.trigger_pack_uv_islands {
        if let Some(sel_id) = state.selected_node {
            let cmd = Box::new(PackUvIslandsCommand {
                node_id: sel_id,
                margin: state.smart_uv_margin,
                old_mesh: None,
            });
            let _ = state.history.execute(cmd, &mut state.scene);
        }
    }
    if actions.trigger_rotate_uv_cw {
        if let Some(sel_id) = state.selected_node {
            let cmd = Box::new(RotateUvsCommand {
                node_id: sel_id,
                clockwise: true,
                old_mesh: None,
            });
            let _ = state.history.execute(cmd, &mut state.scene);
        }
    }
    if actions.trigger_rotate_uv_ccw {
        if let Some(sel_id) = state.selected_node {
            let cmd = Box::new(RotateUvsCommand {
                node_id: sel_id,
                clockwise: false,
                old_mesh: None,
            });
            let _ = state.history.execute(cmd, &mut state.scene);
        }
    }
    if actions.trigger_flip_uv_h {
        if let Some(sel_id) = state.selected_node {
            let cmd = Box::new(FlipUvsCommand {
                node_id: sel_id,
                horizontal: true,
                old_mesh: None,
            });
            let _ = state.history.execute(cmd, &mut state.scene);
        }
    }
    if actions.trigger_flip_uv_v {
        if let Some(sel_id) = state.selected_node {
            let cmd = Box::new(FlipUvsCommand {
                node_id: sel_id,
                horizontal: false,
                old_mesh: None,
            });
            let _ = state.history.execute(cmd, &mut state.scene);
        }
    }
    if let Some(factor) = actions.trigger_scale_uv {
        if let Some(sel_id) = state.selected_node {
            let cmd = Box::new(ScaleUvsCommand {
                node_id: sel_id,
                factor,
                old_mesh: None,
            });
            let _ = state.history.execute(cmd, &mut state.scene);
        }
    }
    if actions.trigger_relax_uvs {
        if let Some(sel_id) = state.selected_node {
            let cmd = Box::new(RelaxUvsCommand {
                node_id: sel_id,
                iterations: 5,
                factor: 0.5,
                old_mesh: None,
            });
            let _ = state.history.execute(cmd, &mut state.scene);
        }
    }
    if actions.trigger_mark_seam {
        if let Some(sel_id) = state.selected_node {
            let edges: Vec<EdgeKey> = state.selected_edges.iter().copied().collect();
            if !edges.is_empty() {
                let cmd = Box::new(MarkSeamCommand {
                    node_id: sel_id,
                    edges,
                    is_seam: true,
                    old_seam_states: Vec::new(),
                });
                let _ = state.history.execute(cmd, &mut state.scene);
            }
        }
    }
    if actions.trigger_clear_seam {
        if let Some(sel_id) = state.selected_node {
            let edges: Vec<EdgeKey> = state.selected_edges.iter().copied().collect();
            if !edges.is_empty() {
                let cmd = Box::new(MarkSeamCommand {
                    node_id: sel_id,
                    edges,
                    is_seam: false,
                    old_seam_states: Vec::new(),
                });
                let _ = state.history.execute(cmd, &mut state.scene);
            }
        }
    }
    if actions.trigger_clear_all_seams {
        if let Some(sel_id) = state.selected_node {
            if let Some(node) = state.scene.nodes.get(sel_id) {
                if let NodeData::Mesh { base_mesh, .. } = &node.data {
                    let seams = base_mesh.get_seams();
                    if !seams.is_empty() {
                        let cmd = Box::new(MarkSeamCommand {
                            node_id: sel_id,
                            edges: seams,
                            is_seam: false,
                            old_seam_states: Vec::new(),
                        });
                        let _ = state.history.execute(cmd, &mut state.scene);
                    }
                }
            }
        }
    }
}

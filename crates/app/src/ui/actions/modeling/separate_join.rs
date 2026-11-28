use core_math::Transform;
use geo_kernel::FaceKey;

use crate::commands::*;
use crate::state::AppState;
use crate::ui::actions::UiActions;

pub fn handle_separate_join_actions(state: &mut AppState, actions: &mut UiActions) {
    // Handle Join Meshes (Blender Ctrl+J / Maya Combine)
    if actions.trigger_join_meshes {
        if let (Some(target_id), Some(source_id)) =
            (state.selected_node, state.previous_selected_node)
        {
            if target_id != source_id {
                let cmd = Box::new(JoinMeshesCommand {
                    target_id,
                    source_id,
                    old_target_mesh: None,
                    source_name: String::new(),
                    source_transform: Transform::IDENTITY,
                    source_base_mesh: None,
                });
                if let Err(e) = state.history.execute(cmd, &mut state.scene) {
                    eprintln!("Join meshes failed: {}", e);
                } else {
                    state.previous_selected_node = None;
                }
            }
        }
    }

    // Handle Separate Faces (Blender P / Maya Extract)
    if actions.trigger_separate_faces {
        if let Some(sel_id) = state.selected_node {
            if !state.selected_faces.is_empty() {
                let faces: Vec<FaceKey> = state.selected_faces.iter().copied().collect();
                let cmd = Box::new(SeparateFacesCommand {
                    source_id: sel_id,
                    faces_to_extract: faces,
                    created_node_id: None,
                    old_source_mesh: None,
                });
                if let Err(e) = state.history.execute(cmd, &mut state.scene) {
                    eprintln!("Separate faces failed: {}", e);
                } else {
                    state.selected_faces.clear();
                }
            }
        }
    }

    // Handle Separate Loose Parts (Blender P -> By Loose Parts / Maya Separate)
    if actions.trigger_separate_loose_parts {
        if let Some(sel_id) = state.selected_node {
            let cmd = Box::new(SeparateLoosePartsCommand {
                source_id: sel_id,
                created_node_ids: Vec::new(),
                old_source_mesh: None,
            });
            if let Err(e) = state.history.execute(cmd, &mut state.scene) {
                eprintln!("Separate loose parts failed: {}", e);
            } else {
                state.selected_faces.clear();
                state.selected_edges.clear();
                state.selected_vertices.clear();
            }
        }
    }

    // Handle Separate by Material (Blender P -> By Material)
    if actions.trigger_separate_by_material {
        if let Some(sel_id) = state.selected_node {
            let cmd = Box::new(SeparateByMaterialCommand {
                source_id: sel_id,
                created_node_ids: Vec::new(),
                old_source_mesh: None,
            });
            if let Err(e) = state.history.execute(cmd, &mut state.scene) {
                eprintln!("Separate by material failed: {}", e);
            } else {
                state.selected_faces.clear();
                state.selected_edges.clear();
                state.selected_vertices.clear();
            }
        }
    }
}

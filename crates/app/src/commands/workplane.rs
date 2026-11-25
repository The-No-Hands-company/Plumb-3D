use core_math::{Vec3, Workplane};
use graph_engine::NodeData;

use crate::state::AppState;
use crate::types::InteractionMode;

/// Toggles or snaps the CAD Workplane coplanar to the active selection (Plasticity / Rhino style).
pub fn toggle_workplane_align(state: &mut AppState) {
    if state.workplane.is_active {
        state.workplane = Workplane::default();
        state.set_status_message("Workplane reset to Global Ground (Z=0)");
        state.window.request_redraw();
        return;
    }

    if let Some(sel_id) = state.selected_node {
        if let Some(node) = state.scene.nodes.get(sel_id) {
            if let NodeData::Mesh { base_mesh, .. } = &node.data {
                // If a face is selected in Edit mode, align directly to that face
                if state.interaction_mode == InteractionMode::Edit && !state.selected_faces.is_empty() {
                    let &fk = state.selected_faces.iter().next().unwrap();
                    let local_centroid = base_mesh.face_centroid(fk);
                    let local_normal = base_mesh.faces[fk].normal;

                    let world_mat = node.local_transform.to_matrix();
                    let world_pos = world_mat.transform_point3(local_centroid);
                    let world_normal = (node.local_transform.rotation * local_normal).normalize_or_zero();

                    state.workplane = Workplane::from_point_and_normal(world_pos, world_normal);
                    state.set_status_message("Workplane aligned to selected Face (Alt+W to reset)");
                    state.window.request_redraw();
                    return;
                }

                // Otherwise align to object origin and Y-normal
                let world_pos = node.local_transform.translation;
                let world_normal = (node.local_transform.rotation * Vec3::Y).normalize_or_zero();
                state.workplane = Workplane::from_point_and_normal(world_pos, world_normal);
                state.set_status_message("Workplane aligned to Object Origin (Alt+W to reset)");
                state.window.request_redraw();
                return;
            }
        }
    }

    state.workplane = Workplane::from_point_and_normal(Vec3::ZERO, Vec3::Y);
    state.set_status_message("Workplane activated at Center (Alt+W to reset)");
    state.window.request_redraw();
}

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
                if state.interaction_mode == InteractionMode::Edit
                    && !state.selected_faces.is_empty()
                {
                    let &fk = state.selected_faces.iter().next().unwrap();
                    let local_centroid = base_mesh.face_centroid(fk);
                    let local_normal = base_mesh.faces[fk].normal;

                    let world_mat = node.local_transform.to_matrix();
                    let world_pos = world_mat.transform_point3(local_centroid);
                    let world_normal =
                        (node.local_transform.rotation * local_normal).normalize_or_zero();

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

/// Symmetrizes and bisect-welds the selected mesh across the active CAD workplane or local X-axis (MeshMachine / HardOps Alt+X).
pub fn symmetrize_active(state: &mut AppState) {
    let sel_id = match state.selected_node {
        Some(id) => id,
        None => {
            state.set_status_message("Select an object to symmetrize (Alt+X)");
            return;
        }
    };

    let (plane_origin, plane_normal) = if state.workplane.is_active {
        // Transform world workplane into node local space
        if let Some(node) = state.scene.nodes.get(sel_id) {
            let inv_mat = node.local_transform.to_matrix().inverse();
            let local_orig = inv_mat.transform_point3(state.workplane.origin);
            let local_norm = (node.local_transform.rotation.conjugate() * state.workplane.normal)
                .normalize_or_zero();
            (local_orig, local_norm)
        } else {
            (Vec3::ZERO, Vec3::X)
        }
    } else {
        (Vec3::ZERO, Vec3::X)
    };

    if let Some(node) = state.scene.nodes.get_mut(sel_id) {
        if let NodeData::Mesh { base_mesh, .. } = &mut node.data {
            if let Ok(()) = base_mesh.symmetrize_plane(plane_origin, plane_normal) {
                state.sync_gpu_meshes();
                state.scene.mark_dirty(sel_id);
                state.set_status_message(if state.workplane.is_active {
                    "Symmetrized & Welded across CAD Workplane (Alt+X)! (Ctrl+Z to Undo)"
                } else {
                    "Symmetrized & Welded across X-Axis (Alt+X)! (Ctrl+Z to Undo)"
                });
            }
        }
    }

    state.window.request_redraw();
}

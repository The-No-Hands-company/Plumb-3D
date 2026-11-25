pub mod input;
pub mod shapes;

pub use input::*;
pub use shapes::*;

use core_math::{Vec2, Vec3, Workplane};
use geo_kernel::{BooleanOp, MirrorAxis};
use graph_engine::NodeData;

use crate::state::AppState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CutterStage {
    DrawFootprint,
    ExtrudeDepth,
}

#[derive(Debug, Clone)]
pub struct CutterModalState {
    pub shape: CutterShape,
    pub stage: CutterStage,
    pub workplane: Workplane,
    pub start_pos: Vec3,
    pub current_pos: Vec3,
    pub start_uv: Vec2,
    pub current_uv: Vec2,
    pub polygon_points: Vec<Vec2>,
    pub depth: f32,
    pub operation: BooleanOp,
    pub drag_start_mouse_y: f64,
    pub radial_count: usize,
    pub symmetry_mirror: Option<MirrorAxis>,
    pub is_ghost: bool,
}

impl Default for CutterModalState {
    fn default() -> Self {
        Self {
            shape: CutterShape::Box,
            stage: CutterStage::DrawFootprint,
            workplane: Workplane::default(),
            start_pos: Vec3::ZERO,
            current_pos: Vec3::ZERO,
            start_uv: Vec2::ZERO,
            current_uv: Vec2::ZERO,
            polygon_points: Vec::new(),
            depth: 0.2,
            operation: BooleanOp::Difference,
            drag_start_mouse_y: 0.0,
            radial_count: 1,
            symmetry_mirror: None,
            is_ghost: false,
        }
    }
}

pub fn start(state: &mut AppState) {
    if state.selected_node.is_none() {
        state.set_status_message("Select an object first to perform precision cuts");
        return;
    }

    let workplane = if state.workplane.is_active {
        state.workplane
    } else if let Some(sel_id) = state.selected_node {
        if let Some(node) = state.scene.nodes.get(sel_id) {
            if let NodeData::Mesh { base_mesh, .. } = &node.data {
                if !state.selected_faces.is_empty() {
                    let &fk = state.selected_faces.iter().next().unwrap();
                    let world_mat = node.local_transform.to_matrix();
                    let pos = world_mat.transform_point3(base_mesh.face_centroid(fk));
                    let n = (node.local_transform.rotation * base_mesh.faces[fk].normal)
                        .normalize_or_zero();
                    Workplane::from_point_and_normal(pos, n)
                } else {
                    Workplane::from_point_and_normal(
                        node.local_transform.translation,
                        node.local_transform.rotation * Vec3::Y,
                    )
                }
            } else {
                Workplane::from_point_and_normal(Vec3::ZERO, Vec3::Y)
            }
        } else {
            Workplane::from_point_and_normal(Vec3::ZERO, Vec3::Y)
        }
    } else {
        Workplane::from_point_and_normal(Vec3::ZERO, Vec3::Y)
    };

    state.cutter_modal = Some(CutterModalState {
        workplane,
        ..Default::default()
    });

    state.set_status_message(
        "Cutter Active (C): Drag footprint | Tab: Shape (Box/Circle/N-Gon) | M: Symmetry | Scroll/R: Radial | G: Ghost",
    );
    state.window.request_redraw();
}

pub fn cancel(state: &mut AppState) {
    state.cutter_modal = None;
    state.window.request_redraw();
}

pub fn commit(state: &mut AppState) {
    let cutter = match state.cutter_modal.take() {
        Some(c) => c,
        None => return,
    };

    let sel_id = match state.selected_node {
        Some(id) => id,
        None => return,
    };

    let cutter_mesh = match build_compound_cutter(
        &cutter.workplane,
        cutter.shape,
        cutter.start_uv,
        cutter.current_uv,
        &cutter.polygon_points,
        cutter.depth,
        cutter.radial_count,
        cutter.symmetry_mirror,
    ) {
        Ok(m) => m,
        Err(_) => return,
    };

    if !cutter.is_ghost {
        // Destructive Direct Boolean Cut
        if let Some(node) = state.scene.nodes.get_mut(sel_id) {
            if let NodeData::Mesh { base_mesh, .. } = &mut node.data {
                let inv_mat = node.local_transform.to_matrix().inverse();
                let mut local_cutter = cutter_mesh.clone();
                for v in local_cutter.vertices.values_mut() {
                    v.position = inv_mat.transform_point3(v.position);
                }
                local_cutter.update_normals();

                base_mesh.stash_normals();
                if let Ok(mut new_mesh) = base_mesh.boolean(&local_cutter, cutter.operation) {
                    new_mesh.stashed_normals = base_mesh.stashed_normals.clone();
                    new_mesh.transfer_stashed_normals(0.25);
                    new_mesh.update_face_weighted_normals(true);
                    *base_mesh = new_mesh;
                    state.sync_gpu_meshes();
                    state.scene.mark_dirty(sel_id);
                    state.set_status_message(if cutter.radial_count > 1 {
                        format!(
                            "Radial Array Cut (N={}) Applied! (Ctrl+Z to Undo)",
                            cutter.radial_count
                        )
                    } else {
                        "Hard-Surface Cut Applied! (Ctrl+Z to Undo)".to_string()
                    });
                }
            }
        }
    } else {
        // Non-Destructive Ghost Cutter (Live Movable Cut)
        let target_name = state
            .scene
            .nodes
            .get(sel_id)
            .map(|n| n.name.clone())
            .unwrap_or_else(|| "Target".to_string());
        let cutter_name = format!("{}_GhostCutter", target_name);

        let mut centroid = Vec3::ZERO;
        let v_count = cutter_mesh.vertices.len().max(1) as f32;
        for v in cutter_mesh.vertices.values() {
            centroid += v.position;
        }
        centroid /= v_count;

        let mut local_cutter = cutter_mesh.clone();
        for v in local_cutter.vertices.values_mut() {
            v.position -= centroid;
        }
        local_cutter.update_normals();

        let mut cutter_node = graph_engine::SceneNode::new_mesh(
            &cutter_name,
            core_math::Transform::from_translation(centroid),
            local_cutter,
        );
        if let NodeData::Mesh { materials, .. } = &mut cutter_node.data {
            materials.clear();
            materials.push(graph_engine::Material {
                name: "GhostCutter_Material".to_string(),
                base_color: [1.0, 0.45, 0.05, 0.4],
                metallic: 0.0,
                roughness: 0.2,
                emissive: 0.3,
                texture_pattern: 0,
                albedo_texture_path: None,
                normal_texture_path: None,
                roughness_metallic_texture_path: None,
            });
        }
        let cutter_id = state.scene.add_node(cutter_node);

        if let Some(target_node) = state.scene.nodes.get_mut(sel_id) {
            if let NodeData::Mesh {
                modifier_stack,
                cached_render_mesh,
                ..
            } = &mut target_node.data
            {
                let bool_mod = geo_kernel::BooleanModifier::with_target(
                    geo_kernel::Mesh::new(),
                    cutter.operation,
                    cutter_name,
                );
                modifier_stack.add_modifier(Box::new(bool_mod));
                *cached_render_mesh = None;
                state.scene.mark_dirty(sel_id);
            }
        }

        state.scene.sync_booleans();
        state.sync_gpu_meshes();
        state.selected_node = Some(cutter_id);
        state.set_status_message(
            "👻 Ghost Cutter Active! Move with W/E/R to adjust live cut | Modifiers in Inspector",
        );
    }

    state.window.request_redraw();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_box_cutter_mesh_generation() {
        let wp = Workplane::from_point_and_normal(Vec3::ZERO, Vec3::Y);
        let cutter = build_box_cutter(&wp, Vec2::new(-0.5, -0.5), Vec2::new(0.5, 0.5), 0.3)
            .expect("box cutter mesh");
        assert_eq!(cutter.vertices.len(), 8);
        assert_eq!(cutter.faces.len(), 6);
        assert_eq!(cutter.euler_characteristic(), 2);
    }

    #[test]
    fn test_circle_cutter_mesh_generation() {
        let wp = Workplane::from_point_and_normal(Vec3::new(1.0, 2.0, 3.0), Vec3::Z);
        let cutter =
            build_circle_cutter(&wp, Vec2::ZERO, 0.4, 0.5, 16).expect("circle cutter mesh");
        assert_eq!(cutter.vertices.len(), 32);
        assert_eq!(cutter.faces.len(), 18);
        assert_eq!(cutter.euler_characteristic(), 2);
    }

    #[test]
    fn test_radial_compound_cutter() {
        let wp = Workplane::from_point_and_normal(Vec3::ZERO, Vec3::Y);
        let compound = build_compound_cutter(
            &wp,
            CutterShape::Circle,
            Vec2::ZERO,
            Vec2::new(0.5, 0.0),
            &[],
            0.2,
            6,
            None,
        )
        .expect("radial circle cutter");
        assert_eq!(compound.vertices.len(), 192);
        assert_eq!(compound.faces.len(), 108);

        let box_compound = build_compound_cutter(
            &wp,
            CutterShape::Box,
            Vec2::ZERO,
            Vec2::new(0.6, 0.0),
            &[],
            0.2,
            4,
            None,
        )
        .expect("radial box cutter");
        assert_eq!(box_compound.vertices.len(), 32);
        assert_eq!(box_compound.faces.len(), 24);
    }

    #[test]
    fn test_ngon_cutter_and_symmetry_mirror() {
        let wp = Workplane::from_point_and_normal(Vec3::ZERO, Vec3::Y);
        let poly = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 0.0),
            Vec2::new(1.0, 1.0),
            Vec2::new(0.0, 1.0),
            Vec2::new(-0.5, 0.5),
        ];

        let ngon = build_ngon_cutter(&wp, &poly, 0.3).expect("ngon cutter mesh");
        assert_eq!(ngon.vertices.len(), 10);
        assert_eq!(ngon.faces.len(), 7); // 2 caps + 5 sides

        // Test with X symmetry mirror
        let mirrored = build_compound_cutter(
            &wp,
            CutterShape::Ngon,
            Vec2::ZERO,
            Vec2::new(1.0, 1.0),
            &poly,
            0.3,
            1,
            Some(MirrorAxis::X),
        )
        .expect("mirrored ngon cutter");
        // 10 orig + 10 mirrored = 20 vertices
        assert_eq!(mirrored.vertices.len(), 20);
        assert_eq!(mirrored.faces.len(), 14);
    }
}

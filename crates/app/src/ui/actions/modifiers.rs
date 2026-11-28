use geo_kernel::*;
use graph_engine::NodeData;

use super::UiActions;
use crate::commands::{ApplyModifierCommand, CollapseStackCommand};
use crate::state::AppState;

pub fn handle_modifiers(state: &mut AppState, actions: &mut UiActions) {
    if actions.add_subdivide_to_selected {
        if let Some(sel_id) = state.selected_node {
            if let Some(node) = state.scene.nodes.get_mut(sel_id) {
                if let NodeData::Mesh { modifier_stack, .. } = &mut node.data {
                    modifier_stack.add_modifier(Box::new(SubdivideModifier::new(1)));
                }
            }
            state.scene.mark_dirty(sel_id);
        }
    }

    if actions.add_catmull_clark_to_selected {
        if let Some(sel_id) = state.selected_node {
            if let Some(node) = state.scene.nodes.get_mut(sel_id) {
                if let NodeData::Mesh { modifier_stack, .. } = &mut node.data {
                    modifier_stack.add_modifier(Box::new(CatmullClarkModifier::new(1)));
                }
            }
            state.scene.mark_dirty(sel_id);
        }
    }

    if actions.add_geometry_nodes_to_selected {
        if let Some(sel_id) = state.selected_node {
            if let Some(node) = state.scene.nodes.get_mut(sel_id) {
                if let NodeData::Mesh { modifier_stack, .. } = &mut node.data {
                    let modifier = geo_nodes::GeometryNodesModifier::new(
                        "GeometryNodes",
                        state.geo_node_graph.clone(),
                    );
                    modifier_stack.add_modifier(Box::new(modifier));
                }
            }
            state.scene.mark_dirty(sel_id);
        }
    }

    if actions.add_displace_to_selected {
        if let Some(sel_id) = state.selected_node {
            if let Some(node) = state.scene.nodes.get_mut(sel_id) {
                if let NodeData::Mesh { modifier_stack, .. } = &mut node.data {
                    modifier_stack.add_modifier(Box::new(DisplaceModifier::new(0.3, 1.5)));
                }
            }
            state.scene.mark_dirty(sel_id);
        }
    }

    if actions.add_taper_to_selected {
        if let Some(sel_id) = state.selected_node {
            if let Some(node) = state.scene.nodes.get_mut(sel_id) {
                if let NodeData::Mesh { modifier_stack, .. } = &mut node.data {
                    modifier_stack.add_modifier(Box::new(TaperModifier::new(0.6)));
                }
            }
            state.scene.mark_dirty(sel_id);
        }
    }

    if actions.add_boolean_diff {
        if let Some(sel_id) = state.selected_node {
            if let Some(node) = state.scene.nodes.get_mut(sel_id) {
                if let NodeData::Mesh { modifier_stack, .. } = &mut node.data {
                    // CSG Boolean difference with a nested cylindrical hole
                    let cutter = Primitives::create_cylinder(0.7, 3.0, 12);
                    modifier_stack.add_modifier(Box::new(BooleanModifier::new(
                        cutter,
                        BooleanOp::Difference,
                    )));
                }
            }
            state.scene.mark_dirty(sel_id);
        }
    }

    if actions.add_decimate_to_selected {
        if let Some(sel_id) = state.selected_node {
            if let Some(node) = state.scene.nodes.get_mut(sel_id) {
                if let NodeData::Mesh { modifier_stack, .. } = &mut node.data {
                    modifier_stack.add_modifier(Box::new(DecimateModifier::new(0.5)));
                }
            }
            state.scene.mark_dirty(sel_id);
        }
    }

    if let Some(axis) = actions.add_mirror_to_selected.take() {
        if let Some(sel_id) = state.selected_node {
            if let Some(node) = state.scene.nodes.get_mut(sel_id) {
                if let NodeData::Mesh { modifier_stack, .. } = &mut node.data {
                    modifier_stack.add_modifier(Box::new(MirrorModifier::new(axis)));
                }
            }
            state.scene.mark_dirty(sel_id);
        }
    }

    if actions.add_weld_to_selected {
        if let Some(sel_id) = state.selected_node {
            if let Some(node) = state.scene.nodes.get_mut(sel_id) {
                if let NodeData::Mesh { modifier_stack, .. } = &mut node.data {
                    modifier_stack.add_modifier(Box::new(WeldModifier::new(0.01)));
                }
            }
            state.scene.mark_dirty(sel_id);
        }
    }

    if actions.add_bevel_to_selected {
        if let Some(sel_id) = state.selected_node {
            if let Some(node) = state.scene.nodes.get_mut(sel_id) {
                if let NodeData::Mesh { modifier_stack, .. } = &mut node.data {
                    modifier_stack.add_modifier(Box::new(BevelModifier::new(0.05)));
                }
            }
            state.scene.mark_dirty(sel_id);
        }
    }

    if actions.add_lattice_to_selected {
        if let Some(sel_id) = state.selected_node {
            if let Some(node) = state.scene.nodes.get_mut(sel_id) {
                if let NodeData::Mesh {
                    base_mesh,
                    modifier_stack,
                    ..
                } = &mut node.data
                {
                    let mut lattice = LatticeModifier::from_mesh_bounds(base_mesh, (3, 3, 3));
                    lattice.squash_and_stretch(1.15);
                    modifier_stack.add_modifier(Box::new(lattice));
                }
            }
            state.scene.mark_dirty(sel_id);
        }
    }

    if actions.add_curve_mod_to_selected {
        if let Some(sel_id) = state.selected_node {
            if let Some(node) = state.scene.nodes.get_mut(sel_id) {
                if let NodeData::Mesh { modifier_stack, .. } = &mut node.data {
                    let spline = Spline::helix(1.5, 4.0, 1.5, 8);
                    modifier_stack.add_modifier(Box::new(CurveModifier::new(spline)));
                }
            }
            state.scene.mark_dirty(sel_id);
        }
    }

    if actions.add_solidify_to_selected {
        if let Some(sel_id) = state.selected_node {
            if let Some(node) = state.scene.nodes.get_mut(sel_id) {
                if let NodeData::Mesh { modifier_stack, .. } = &mut node.data {
                    modifier_stack.add_modifier(Box::new(SolidifyModifier::new(0.2)));
                }
            }
            state.scene.mark_dirty(sel_id);
        }
    }

    if actions.add_array_to_selected {
        if let Some(sel_id) = state.selected_node {
            if let Some(node) = state.scene.nodes.get_mut(sel_id) {
                if let NodeData::Mesh { modifier_stack, .. } = &mut node.data {
                    modifier_stack.add_modifier(Box::new(ArrayModifier::new(3)));
                }
            }
            state.scene.mark_dirty(sel_id);
        }
    }

    if actions.add_smooth_to_selected {
        if let Some(sel_id) = state.selected_node {
            if let Some(node) = state.scene.nodes.get_mut(sel_id) {
                if let NodeData::Mesh { modifier_stack, .. } = &mut node.data {
                    modifier_stack.add_modifier(Box::new(SmoothModifier::new(0.5, 5)));
                }
            }
            state.scene.mark_dirty(sel_id);
        }
    }

    if actions.add_shrinkwrap_to_selected {
        if let Some(sel_id) = state.selected_node {
            if let Some(node) = state.scene.nodes.get_mut(sel_id) {
                if let NodeData::Mesh { modifier_stack, .. } = &mut node.data {
                    modifier_stack.add_modifier(Box::new(ShrinkwrapModifier::new()));
                }
            }
            state.scene.mark_dirty(sel_id);
        }
    }

    if actions.add_simple_deform_to_selected {
        if let Some(sel_id) = state.selected_node {
            if let Some(node) = state.scene.nodes.get_mut(sel_id) {
                if let NodeData::Mesh { modifier_stack, .. } = &mut node.data {
                    modifier_stack.add_modifier(Box::new(SimpleDeformModifier::new(
                        SimpleDeformMethod::Twist,
                        0.5,
                    )));
                }
            }
            state.scene.mark_dirty(sel_id);
        }
    }

    if actions.add_screw_to_selected {
        if let Some(sel_id) = state.selected_node {
            if let Some(node) = state.scene.nodes.get_mut(sel_id) {
                if let NodeData::Mesh { modifier_stack, .. } = &mut node.data {
                    modifier_stack.add_modifier(Box::new(ScrewModifier::new(1.0, 1)));
                }
            }
            state.scene.mark_dirty(sel_id);
        }
    }

    if actions.add_triangulate_to_selected {
        if let Some(sel_id) = state.selected_node {
            if let Some(node) = state.scene.nodes.get_mut(sel_id) {
                if let NodeData::Mesh { modifier_stack, .. } = &mut node.data {
                    modifier_stack.add_modifier(Box::new(TriangulateModifier::new()));
                }
            }
            state.scene.mark_dirty(sel_id);
        }
    }

    if actions.add_cloth_to_selected {
        if let Some(sel_id) = state.selected_node {
            if let Some(node) = state.scene.nodes.get_mut(sel_id) {
                if let NodeData::Mesh {
                    modifier_stack,
                    base_mesh,
                    ..
                } = &mut node.data
                {
                    let mut cloth = ClothModifier::new();
                    cloth.initialize(base_mesh);
                    modifier_stack.add_modifier(Box::new(cloth));
                }
            }
            state.scene.mark_dirty(sel_id);
        }
    }

    if actions.add_particle_system_to_selected {
        if let Some(sel_id) = state.selected_node {
            if let Some(node) = state.scene.nodes.get_mut(sel_id) {
                if let NodeData::Mesh { modifier_stack, .. } = &mut node.data {
                    let mut particles = ParticleSystemModifier::new();
                    particles.reset();
                    modifier_stack.add_modifier(Box::new(particles));
                }
            }
            state.scene.mark_dirty(sel_id);
        }
    }

    if actions.add_wireframe_to_selected {
        if let Some(sel_id) = state.selected_node {
            if let Some(node) = state.scene.nodes.get_mut(sel_id) {
                if let NodeData::Mesh { modifier_stack, .. } = &mut node.data {
                    modifier_stack
                        .add_modifier(Box::new(WireframeModifier::new(state.wireframe_thickness)));
                }
            }
            state.scene.mark_dirty(sel_id);
        }
    }

    if actions.add_skin_envelope_to_selected {
        if let Some(sel_id) = state.selected_node {
            if let Some(node) = state.scene.nodes.get_mut(sel_id) {
                if let NodeData::Mesh { modifier_stack, .. } = &mut node.data {
                    modifier_stack
                        .add_modifier(Box::new(geo_kernel::SkinEnvelopeModifier::default()));
                }
            }
            state.scene.mark_dirty(sel_id);
        }
    }

    if actions.add_remesh_to_selected {
        if let Some(sel_id) = state.selected_node {
            if let Some(node) = state.scene.nodes.get_mut(sel_id) {
                if let NodeData::Mesh { modifier_stack, .. } = &mut node.data {
                    modifier_stack.add_modifier(Box::new(geo_kernel::RemeshModifier::default()));
                }
            }
            state.scene.mark_dirty(sel_id);
        }
    }

    if actions.add_weighted_normal_to_selected {
        if let Some(sel_id) = state.selected_node {
            if let Some(node) = state.scene.nodes.get_mut(sel_id) {
                if let NodeData::Mesh { modifier_stack, .. } = &mut node.data {
                    modifier_stack.add_modifier(Box::new(WeightedNormalModifier::default()));
                }
            }
            state.scene.mark_dirty(sel_id);
        }
    }

    if actions.add_morpher_to_selected {
        if let Some(sel_id) = state.selected_node {
            if let Some(node) = state.scene.nodes.get_mut(sel_id) {
                if let NodeData::Mesh { modifier_stack, .. } = &mut node.data {
                    modifier_stack.add_modifier(Box::new(MorpherModifier::new()));
                }
            }
            state.scene.mark_dirty(sel_id);
        }
    }

    if actions.trigger_capture_morph_target {
        if let (Some(sel_id), Some(prev_id)) = (state.selected_node, state.previous_selected_node) {
            let target_mesh = if let Some(p_node) = state.scene.nodes.get(prev_id) {
                if let NodeData::Mesh { base_mesh, .. } = &p_node.data {
                    Some((p_node.name.clone(), base_mesh.clone()))
                } else {
                    None
                }
            } else {
                None
            };

            if let Some((target_name, t_mesh)) = target_mesh {
                if let Some(node) = state.scene.nodes.get_mut(sel_id) {
                    if let NodeData::Mesh {
                        base_mesh,
                        modifier_stack,
                        ..
                    } = &mut node.data
                    {
                        for m in modifier_stack.modifiers_mut() {
                            if let Some(morpher) = m.as_morpher_mut() {
                                let _ = morpher.add_target_from_mesh(
                                    target_name.clone(),
                                    base_mesh,
                                    &t_mesh,
                                );
                            }
                        }
                    }
                }
                state.scene.mark_dirty(sel_id);
            }
        }
    }

    if let Some((idx, mod_name)) = actions.apply_modifier_idx.take() {
        if let Some(sel_id) = state.selected_node {
            if let Some(node) = state.scene.nodes.get(sel_id) {
                if let NodeData::Mesh { base_mesh, .. } = &node.data {
                    let cmd = Box::new(ApplyModifierCommand {
                        node_id: sel_id,
                        modifier_idx: idx,
                        modifier_name: format!("Apply {}", mod_name),
                        old_base_mesh: base_mesh.clone(),
                    });
                    let _ = state.history.execute(cmd, &mut state.scene);
                }
            }
        }
    }

    if let Some(idx) = actions.remove_modifier_idx.take() {
        if let Some(sel_id) = state.selected_node {
            if let Some(node) = state.scene.nodes.get_mut(sel_id) {
                if let NodeData::Mesh {
                    modifier_stack,
                    cached_render_mesh,
                    ..
                } = &mut node.data
                {
                    if idx < modifier_stack.modifiers().len() {
                        modifier_stack.remove_modifier(idx);
                        *cached_render_mesh = None;
                        node.is_dirty = true;
                    }
                }
            }
            state.scene.mark_dirty(sel_id);
        }
    }

    if let Some((from, to)) = actions.reorder_modifier.take() {
        if let Some(sel_id) = state.selected_node {
            if let Some(node) = state.scene.nodes.get_mut(sel_id) {
                if let NodeData::Mesh {
                    modifier_stack,
                    cached_render_mesh,
                    ..
                } = &mut node.data
                {
                    modifier_stack.move_modifier(from, to);
                    *cached_render_mesh = None;
                    node.is_dirty = true;
                }
            }
            state.scene.mark_dirty(sel_id);
            state.sync_gpu_meshes();
            state.window.request_redraw();
        }
    }

    if actions.collapse_modifiers {
        if let Some(sel_id) = state.selected_node {
            if let Some(node) = state.scene.nodes.get(sel_id) {
                if let NodeData::Mesh { base_mesh, .. } = &node.data {
                    let cmd = Box::new(CollapseStackCommand {
                        node_id: sel_id,
                        old_base_mesh: base_mesh.clone(),
                    });
                    let _ = state.history.execute(cmd, &mut state.scene);
                }
            }
        }
    }
}

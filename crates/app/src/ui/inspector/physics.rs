use crate::state::AppState;
use core_math::{vec3, Transform};
use egui::Ui;
use geo_kernel::{CableGenerator, SphFluidSimulator, SphSettings};
use graph_engine::{CollisionShape, NodeData, NodeId, RigidBodyComponent, RigidBodyType, SceneNode};

/// Selected physics simulation subsystem tab in the Inspector
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PhysicsSubTab {
    #[default]
    RigidBody,
    SphFluids,
    XpbdCloth,
    CatenaryCables,
}

pub fn draw_physics_panel(ui: &mut Ui, state: &mut AppState, sel_id: NodeId) {
    ui.heading("⚛ Dynamics & Simulation Hub");
    ui.separator();

    // Subsystem Tabs
    ui.horizontal(|ui| {
        if ui.selectable_label(state.physics_sub_tab == PhysicsSubTab::RigidBody, "⚛ Rigid Bodies").clicked() {
            state.physics_sub_tab = PhysicsSubTab::RigidBody;
        }
        if ui.selectable_label(state.physics_sub_tab == PhysicsSubTab::SphFluids, "💧 SPH Fluids").clicked() {
            state.physics_sub_tab = PhysicsSubTab::SphFluids;
        }
        if ui.selectable_label(state.physics_sub_tab == PhysicsSubTab::XpbdCloth, "🧵 XPBD Cloth").clicked() {
            state.physics_sub_tab = PhysicsSubTab::XpbdCloth;
        }
        if ui.selectable_label(state.physics_sub_tab == PhysicsSubTab::CatenaryCables, "〰 Cables").clicked() {
            state.physics_sub_tab = PhysicsSubTab::CatenaryCables;
        }
    });
    ui.separator();

    match state.physics_sub_tab {
        PhysicsSubTab::RigidBody => draw_rigid_body_tab(ui, state, sel_id),
        PhysicsSubTab::SphFluids => draw_sph_fluids_tab(ui, state),
        PhysicsSubTab::XpbdCloth => draw_xpbd_cloth_tab(ui, state),
        PhysicsSubTab::CatenaryCables => draw_catenary_cables_tab(ui, state),
    }
}

fn draw_rigid_body_tab(ui: &mut Ui, state: &mut AppState, sel_id: NodeId) {
    let has_body = state.physics.bodies.contains_key(&sel_id);

    if !has_body {
        ui.label("No Rigid Body component attached to this node.");
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            if ui.button("➕ Add Active Rigid Body").on_hover_text("Adds dynamic rigid body influenced by gravity and impulses").clicked() {
                if let Some(node) = state.scene.nodes.get(sel_id) {
                    let shape = match &node.data {
                        NodeData::Mesh { base_mesh, .. } => {
                            let (min, max) = base_mesh.bounding_box();
                            let half_extents = (max - min) * 0.5;
                            CollisionShape::Box { half_extents }
                        }
                        _ => CollisionShape::Sphere { radius: 1.0 },
                    };
                    let body = RigidBodyComponent::new_active(
                        shape,
                        1.0,
                        node.local_transform.translation,
                        node.local_transform.rotation,
                    );
                    state.physics.add_body(sel_id, body);
                }
            }
            if ui.button("➕ Add Passive Obstacle").on_hover_text("Adds static collision obstacle (e.g. floor/wall)").clicked() {
                if let Some(node) = state.scene.nodes.get(sel_id) {
                    let shape = match &node.data {
                        NodeData::Mesh { base_mesh, .. } => {
                            let (min, max) = base_mesh.bounding_box();
                            let half_extents = (max - min) * 0.5;
                            CollisionShape::Box { half_extents }
                        }
                        _ => CollisionShape::Box { half_extents: core_math::Vec3::splat(1.0) },
                    };
                    let body = RigidBodyComponent::new_passive(
                        shape,
                        node.local_transform.translation,
                        node.local_transform.rotation,
                    );
                    state.physics.add_body(sel_id, body);
                }
            }
        });
    } else {
        let mut remove_requested = false;
        if let Some(body) = state.physics.bodies.get_mut(&sel_id) {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Rigid Body Settings").strong());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button("🗑 Remove").clicked() {
                            remove_requested = true;
                        }
                    });
                });
                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("Type:");
                    let is_active = body.body_type == RigidBodyType::Active;
                    if ui.selectable_label(is_active, "Active (Dynamic)").clicked() {
                        body.body_type = RigidBodyType::Active;
                        if body.mass <= 0.0 {
                            body.mass = 1.0;
                        }
                    }
                    if ui.selectable_label(!is_active, "Passive (Static)").clicked() {
                        body.body_type = RigidBodyType::Passive;
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Collision Shape:");
                    let is_box = matches!(body.shape, CollisionShape::Box { .. });
                    let is_sphere = matches!(body.shape, CollisionShape::Sphere { .. });
                    let is_cyl = matches!(body.shape, CollisionShape::Cylinder { .. });
                    if ui.selectable_label(is_box, "Box").clicked() {
                        body.shape = CollisionShape::Box { half_extents: core_math::Vec3::splat(0.5) };
                    }
                    if ui.selectable_label(is_sphere, "Sphere").clicked() {
                        body.shape = CollisionShape::Sphere { radius: 0.5 };
                    }
                    if ui.selectable_label(is_cyl, "Cylinder").clicked() {
                        body.shape = CollisionShape::Cylinder { radius: 1.0, half_height: 2.0 };
                    }
                });

                if body.body_type == RigidBodyType::Active {
                    ui.horizontal(|ui| {
                        ui.label("Mass (kg):");
                        ui.add(egui::DragValue::new(&mut body.mass).speed(0.1).range(0.01..=1000.0));
                    });
                    if ui.button("🚀 Apply Siege Impact Impulse").on_hover_text("Applies horizontal impulse (8 N·s) simulating a boulder hit").clicked() {
                        body.apply_impulse(core_math::Vec3::new(8.0, 3.0, 0.0));
                    }
                }

                ui.horizontal(|ui| {
                    ui.label("Restitution (Bounce):");
                    ui.add(egui::Slider::new(&mut body.restitution, 0.0..=1.0));
                });

                ui.horizontal(|ui| {
                    ui.label("Friction:");
                    ui.add(egui::Slider::new(&mut body.friction, 0.0..=1.0));
                });

                ui.checkbox(&mut body.enabled, "Enabled in Simulation");
            });
        }

        if remove_requested {
            state.physics.remove_body(sel_id);
        }
    }

    ui.add_space(8.0);
    ui.separator();
    ui.heading("🌍 World Physics Settings");
    ui.checkbox(&mut state.physics.is_simulating, "Active Simulation on Play");

    let mut has_ground = state.physics.ground_plane_y.is_some();
    if ui.checkbox(&mut has_ground, "Floor Collision Plane (Y = 0)").changed() {
        state.physics.ground_plane_y = if has_ground { Some(0.0) } else { None };
    }

    ui.horizontal(|ui| {
        ui.label("Gravity Y (m/s²):");
        ui.add(egui::DragValue::new(&mut state.physics.gravity.y).speed(0.1).range(-50.0..=50.0));
    });

    ui.horizontal(|ui| {
        if ui.button("🔄 Reset Positions").clicked() {
            state.physics.reset_simulation(&mut state.scene);
        }
        if ui.button("🍳 Bake to Keyframes").clicked() {
            let start = state.animation.start_frame as u32;
            let end = state.animation.end_frame as u32;
            let fps = state.animation.fps;
            state.physics.bake_to_keyframes(&mut state.scene, &mut state.animation, start, end, fps);
        }
    });
}

fn draw_sph_fluids_tab(ui: &mut Ui, state: &mut AppState) {
    ui.heading("💧 SPH Liquid Fluid Dynamics");
    ui.label(egui::RichText::new("Smoothed Particle Hydrodynamics (SPH) RealFlow-grade fluid dynamics with surface meshing.").small().weak());
    ui.separator();

    ui.horizontal(|ui| {
        ui.label("Fluid Viscosity:");
        ui.add(egui::Slider::new(&mut state.fluid_viscosity, 0.01..=1.0).text("Pa·s"));
    });

    ui.horizontal(|ui| {
        ui.label("Rest Density:");
        ui.add(egui::Slider::new(&mut state.fluid_rest_density, 500.0..=2000.0).text("kg/m³"));
    });

    ui.horizontal(|ui| {
        ui.label("Grid Resolution:");
        ui.add(egui::Slider::new(&mut state.fluid_grid_step, 0.1..=0.4).text("m"));
    });

    ui.add_space(6.0);
    ui.horizontal(|ui| {
        if ui.button("💧 Spawn Fluid Dam-Break Block").on_hover_text("Spawns 4x4x4 fluid particle cluster").clicked() {
            let mut sim = SphFluidSimulator::new(SphSettings {
                viscosity: state.fluid_viscosity,
                rest_density: state.fluid_rest_density,
                ..Default::default()
            });
            sim.spawn_block(vec3(-0.3, 1.0, -0.3), vec3(0.6, 0.6, 0.6), 0.15);
            state.active_fluid_sim = Some(sim);
            state.set_status_message("💧 Spawned 64 SPH fluid particles.");
        }

        if let Some(ref mut sim) = state.active_fluid_sim {
            if ui.button("▶ Step Physics (16ms)").clicked() {
                sim.step(0.016);
            }
        }
    });

    if let Some(ref sim) = state.active_fluid_sim {
        ui.label(format!("Active Particles: {}", sim.particles.len()));
        if ui.button("🧊 Reconstruct Watertight Fluid Mesh").on_hover_text("Marching Cubes level-set polygonization").clicked() {
            let mesh = sim.reconstruct_surface_mesh(state.fluid_grid_step, 0.5);
            let node = SceneNode::new_mesh("FluidSurfaceMesh", Transform::IDENTITY, mesh);
            let id = state.scene.add_node(node);
            state.selected_node = Some(id);
            state.sync_gpu_meshes();
            state.set_status_message("🧊 Reconstructed watertight fluid surface mesh.");
        }
    }
}

fn draw_xpbd_cloth_tab(ui: &mut Ui, state: &mut AppState) {
    ui.heading("🧵 XPBD Cloth & Soft-Body Dynamics");
    ui.label(egui::RichText::new("Extended Position Based Dynamics (XPBD) with distance and bending compliance.").small().weak());
    ui.separator();

    ui.horizontal(|ui| {
        ui.label("Stretch Compliance:");
        ui.add(egui::Slider::new(&mut state.cloth_compliance, 0.0..=0.01).text("m/N"));
    });

    ui.horizontal(|ui| {
        ui.label("Wind Force Vector:");
        ui.add(egui::DragValue::new(&mut state.cloth_wind.x).speed(0.1).prefix("X: "));
        ui.add(egui::DragValue::new(&mut state.cloth_wind.y).speed(0.1).prefix("Y: "));
        ui.add(egui::DragValue::new(&mut state.cloth_wind.z).speed(0.1).prefix("Z: "));
    });

    ui.add_space(6.0);
    if ui.button("🧵 Spawn XPBD Cloth Sheet (4x4m)").clicked() {
        let cloth_mesh = geo_kernel::Primitives::create_plane(4.0, 4.0, 8, 8);
        let node = SceneNode::new_mesh("XpbdClothSheet", Transform::from_translation(vec3(0.0, 3.0, 0.0)), cloth_mesh);
        let id = state.scene.add_node(node);
        state.selected_node = Some(id);
        state.sync_gpu_meshes();
        state.set_status_message("🧵 Spawned dynamic XPBD cloth plane.");
    }
}

fn draw_catenary_cables_tab(ui: &mut Ui, state: &mut AppState) {
    ui.heading("〰 Dynamic Catenary Hydraulic Cables");
    ui.label(egui::RichText::new("Generates physical catenary droop cables, braided harnesses, and pneumatic couplings.").small().weak());
    ui.separator();

    ui.horizontal(|ui| {
        ui.label("Gravity Sag Factor:");
        ui.add(egui::Slider::new(&mut state.cable_slack, 0.05..=1.5).text("sag"));
    });

    ui.horizontal(|ui| {
        ui.label("Hose Radius:");
        ui.add(egui::Slider::new(&mut state.cable_radius, 0.01..=0.2).text("m"));
    });

    ui.add_space(6.0);
    if ui.button("〰 Drop Catenary Cable Between Anchors").on_hover_text("Generates hydraulic hose spanning Port A to Port B with hex fittings").clicked() {
        let p_a = vec3(-2.0, 2.5, 0.0);
        let p_b = vec3(2.0, 2.0, 0.0);
        let cable = CableGenerator::create_cable(p_a, p_b, state.cable_radius, state.cable_slack, 24, 8);
        let node = SceneNode::new_mesh("HydraulicCatenaryCable", Transform::IDENTITY, cable);
        let id = state.scene.add_node(node);
        state.selected_node = Some(id);
        state.sync_gpu_meshes();
        state.set_status_message("〰 Generated dynamic catenary hydraulic cable.");
    }
}

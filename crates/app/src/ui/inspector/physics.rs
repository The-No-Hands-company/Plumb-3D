use egui::Ui;
use graph_engine::{CollisionShape, NodeData, NodeId, RigidBodyComponent, RigidBodyType};
use crate::state::AppState;

pub fn draw_physics_panel(ui: &mut Ui, state: &mut AppState, sel_id: NodeId) {
    ui.heading("⚛ Rigid Body Dynamics");
    ui.separator();

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
                    if ui.button("🚀 Apply Siege Impact Impulse").on_hover_text("Applies horizontal impulse (8 N·s) simulating a catapult/trebuchet boulder hit").clicked() {
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

                ui.horizontal(|ui| {
                    ui.label("Linear Drag:");
                    ui.add(egui::Slider::new(&mut body.linear_damping, 0.0..=1.0));
                });

                ui.horizontal(|ui| {
                    ui.label("Angular Drag:");
                    ui.add(egui::Slider::new(&mut body.angular_damping, 0.0..=1.0));
                });

                ui.checkbox(&mut body.enabled, "Enabled in Simulation");
            });
        }

        if remove_requested {
            state.physics.remove_body(sel_id);
        }
    }

    ui.add_space(10.0);
    ui.separator();
    ui.heading("🌍 World Physics Settings");
    ui.add_space(4.0);

    ui.checkbox(&mut state.physics.is_simulating, "Active Simulation on Play");

    let mut has_ground = state.physics.ground_plane_y.is_some();
    if ui.checkbox(&mut has_ground, "Floor Collision Plane (Y = 0)").changed() {
        state.physics.ground_plane_y = if has_ground { Some(0.0) } else { None };
    }

    ui.horizontal(|ui| {
        ui.label("Gravity Y (m/s²):");
        ui.add(egui::DragValue::new(&mut state.physics.gravity.y).speed(0.1).range(-50.0..=50.0));
    });

    ui.add_space(6.0);
    ui.horizontal(|ui| {
        if ui.button("🔄 Reset Positions").on_hover_text("Resets all active rigid bodies to their start transforms").clicked() {
            state.physics.reset_simulation(&mut state.scene);
        }
        if ui.button("🍳 Bake to Keyframes").on_hover_text("Bakes physics across timeline range into permanent animation keyframes").clicked() {
            let start = state.animation.start_frame as u32;
            let end = state.animation.end_frame as u32;
            let fps = state.animation.fps;
            state.physics.bake_to_keyframes(&mut state.scene, &mut state.animation, start, end, fps);
        }
    });
}

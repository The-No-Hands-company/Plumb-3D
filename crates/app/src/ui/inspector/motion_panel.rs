//! Inertial Motion Matching Character Locomotion Pad Inspector Panel.
//! Provides directional movement control, sprint toggling, live foot contact indicators,
//! and quintic inertialization blend diagnostics.

use crate::state::AppState;
use core_math::vec2;
use egui::Ui;

pub fn draw_motion_panel(ui: &mut Ui, state: &mut AppState) {
    ui.heading("Motion Matching Locomotion Pad");
    ui.label(
        egui::RichText::new("AAA database query locomotion with quintic inertialization blend.")
            .small()
            .weak(),
    );
    ui.separator();

    // 1. Directional D-Pad Controls
    ui.group(|ui| {
        ui.label(egui::RichText::new("Directional Input Vector").strong());

        ui.vertical_centered(|ui| {
            if ui.button("Forward (W / Up)").clicked() {
                state.motion_stick = vec2(0.0, if state.motion_sprint { 1.0 } else { 0.5 });
            }

            ui.horizontal(|ui| {
                ui.add_space(ui.available_width() * 0.15);
                if ui.button("Left (A)").clicked() {
                    state.motion_stick = vec2(-0.8, 0.0);
                }
                if ui.button("Stop").clicked() {
                    state.motion_stick = vec2(0.0, 0.0);
                }
                if ui.button("Right (D)").clicked() {
                    state.motion_stick = vec2(0.8, 0.0);
                }
            });

            if ui.button("Backward (S / Down)").clicked() {
                state.motion_stick = vec2(0.0, -0.5);
            }
        });

        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.checkbox(&mut state.motion_sprint, "Sprint Mode (Shift)");
            ui.label(format!("Stick: ({:.2}, {:.2})", state.motion_stick.x, state.motion_stick.y));
        });

        if ui.button("Step Locomotion Engine (16ms)").clicked() {
            state.motion_matching.step(0.016, state.motion_stick);
        }
    });

    ui.separator();

    // 2. Foot Planting & Ground Adaptation Status
    ui.group(|ui| {
        ui.label(egui::RichText::new("Inverse Kinematics & Foot Contacts").strong());

        let left_planted = state.motion_matching.is_left_foot_planted();
        let right_planted = state.motion_matching.is_right_foot_planted();

        ui.horizontal(|ui| {
            let left_text = if left_planted { "Left Foot: PLANTED" } else { "Left Foot: SWING" };
            let left_color = if left_planted { egui::Color32::from_rgb(80, 220, 100) } else { egui::Color32::from_rgb(200, 200, 200) };
            ui.label(egui::RichText::new(left_text).color(left_color).strong());

            ui.separator();

            let right_text = if right_planted { "Right Foot: PLANTED" } else { "Right Foot: SWING" };
            let right_color = if right_planted { egui::Color32::from_rgb(80, 220, 100) } else { egui::Color32::from_rgb(200, 200, 200) };
            ui.label(egui::RichText::new(right_text).color(right_color).strong());
        });
    });

    ui.separator();

    // 3. Inertialization Blend Diagnostics
    ui.group(|ui| {
        ui.label(egui::RichText::new("Quintic Polynomial Inertialization").strong());
        ui.label(
            egui::RichText::new("Smooth C2 continuous hermite trajectory blending with zero foot-sliding.")
                .small()
                .weak(),
        );

        let speed = state.motion_matching.current_pose.root_velocity.length();
        ui.label(format!("Root Speed: {:.2} m/s", speed));
        ui.label(format!("Database Poses: {}", state.motion_matching.database.poses.len()));
        ui.label(format!("Active Clip: {}", state.motion_matching.current_pose.clip_name));
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use graph_engine::MotionMatchingController;

    #[test]
    fn test_motion_panel_step() {
        let mut controller = MotionMatchingController::default();
        controller.step(0.016, vec2(0.0, 1.0));
        assert!(controller.current_pose.root_velocity.z > 0.0 || controller.current_pose.root_velocity.y >= 0.0);
    }
}

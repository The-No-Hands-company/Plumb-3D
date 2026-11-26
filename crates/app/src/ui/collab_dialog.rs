//! Real-Time Multi-User Collaboration Dialog ("Figma for 3D")
//!
//! Visualizes active peers, cursor positions, synchronizes node selections,
//! and provides in-session live team chat.

use crate::state::AppState;

/// Renders the floating Live Collaboration Session dialog
pub fn draw_collab_dialog(state: &mut AppState) {
    if !state.show_collab_dialog {
        return;
    }

    let mut open = state.show_collab_dialog;
    egui::Window::new("Live Collaboration Session (Figma for 3D)")
        .open(&mut open)
        .default_size([460.0, 520.0])
        .min_size([380.0, 360.0])
        .show(&state.egui_ctx.clone(), |ui| {
            // Header: Status and room input
            ui.horizontal(|ui| {
                if state.collab_session.is_connected {
                    ui.colored_label(egui::Color32::from_rgb(80, 220, 100), "● Connected");
                    ui.label(format!("(Room: '{}')", state.collab_session.room_name));
                    if ui.button("Disconnect").clicked() {
                        state.collab_session.disconnect();
                    }
                } else {
                    ui.colored_label(egui::Color32::from_rgb(220, 80, 80), "○ Offline");
                    ui.label("Room:");
                    ui.text_edit_singleline(&mut state.collab_session.room_name);
                    if ui.button("Connect").clicked() {
                        let room = state.collab_session.room_name.clone();
                        state.collab_session.connect(&room);
                    }
                }
            });

            ui.separator();

            // Local user card
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    let c = state.collab_session.local_user.color;
                    let color = egui::Color32::from_rgb((c[0] * 255.0) as u8, (c[1] * 255.0) as u8, (c[2] * 255.0) as u8);
                    ui.colored_label(color, "■");
                    ui.strong(format!("{} (You)", state.collab_session.local_user.name));
                    ui.label(format!("| Tool: {}", state.collab_session.local_user.active_tool));
                });
            });

            ui.separator();

            // Active Peers
            ui.strong(format!("Active Peers ({})", state.collab_session.peers.len()));
            if state.collab_session.peers.is_empty() {
                ui.label(egui::RichText::new("No other peers in room. Invite colleagues to collaborate!").italics());
            } else {
                egui::ScrollArea::vertical()
                    .id_salt("collab_peers_scroll")
                    .max_height(100.0)
                    .show(ui, |ui| {
                        for peer in state.collab_session.peers.values() {
                            ui.horizontal(|ui| {
                                let c = peer.color;
                                let color = egui::Color32::from_rgb((c[0] * 255.0) as u8, (c[1] * 255.0) as u8, (c[2] * 255.0) as u8);
                                ui.colored_label(color, "●");
                                ui.label(&peer.name);
                                ui.colored_label(egui::Color32::GRAY, format!("pos: [{:.1}, {:.1}, {:.1}]", peer.cursor_pos.x, peer.cursor_pos.y, peer.cursor_pos.z));
                            });
                        }
                    });
            }

            ui.separator();

            // In-Session Live Chat
            ui.strong("Team Session Chat");
            let available_height = ui.available_height() - 40.0;
            egui::ScrollArea::vertical()
                .id_salt("collab_chat_scroll")
                .max_height(available_height.max(120.0))
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    for entry in &state.collab_session.chat_history {
                        ui.horizontal(|ui| {
                            let c = entry.color;
                            let color = egui::Color32::from_rgb((c[0] * 255.0) as u8, (c[1] * 255.0) as u8, (c[2] * 255.0) as u8);
                            ui.colored_label(color, format!("[{}]", entry.user_name));
                            ui.label(&entry.text);
                        });
                    }
                });

            // Chat input row
            ui.horizontal(|ui| {
                let response = ui.text_edit_singleline(&mut state.collab_chat_input);
                let enter_pressed = response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                if (ui.button("Send").clicked() || enter_pressed) && !state.collab_chat_input.trim().is_empty() {
                    let msg = state.collab_chat_input.trim().to_string();
                    state.collab_session.send_chat(&msg);
                    state.collab_chat_input.clear();
                    response.request_focus();
                }
            });
        });

    state.show_collab_dialog = open;
}

//! Real-Time Multi-User Collaborative Session Engine ("Figma for 3D")
//!
//! Provides CRDT-inspired operational synchronization, multi-user cursor tracking,
//! live selection highlights, synchronized DAG transform cascades, and in-app chat.

use crate::node::NodeId;
use crate::scene::SceneGraph;
use core_math::{Transform, Vec3};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Metadata for a collaborative peer in the 3D session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollabUser {
    pub id: String,
    pub name: String,
    pub color: [f32; 3],
    pub cursor_pos: Vec3,
    pub selected_node_id: Option<NodeId>,
    pub active_tool: String,
    pub last_heartbeat_ms: u64,
}

impl CollabUser {
    pub fn new(id: impl Into<String>, name: impl Into<String>, color: [f32; 3]) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            color,
            cursor_pos: Vec3::ZERO,
            selected_node_id: None,
            active_tool: "Select".to_string(),
            last_heartbeat_ms: 0,
        }
    }
}

/// Collaborative network messages broadcast between peers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CollabMessage {
    /// Peer joined the session
    UserJoin(CollabUser),
    /// Peer left the session
    UserLeave { user_id: String },
    /// 3D Viewport cursor position update
    CursorMove { user_id: String, pos: Vec3 },
    /// Selection change in the DAG scene hierarchy
    Selection {
        user_id: String,
        node_id: Option<NodeId>,
    },
    /// Live transform tweak applied to a scene node
    NodeTransform {
        user_id: String,
        node_id: NodeId,
        transform: Transform,
    },
    /// In-session chat message
    Chat {
        user_id: String,
        user_name: String,
        text: String,
        timestamp_str: String,
    },
}

/// Chat log entry in the collaborative workspace
#[derive(Debug, Clone)]
pub struct CollabChatEntry {
    pub user_name: String,
    pub text: String,
    pub time_str: String,
    pub color: [f32; 3],
}

/// Manages a live multi-user collaborative session
#[derive(Debug)]
pub struct CollabSession {
    pub local_user: CollabUser,
    pub peers: HashMap<String, CollabUser>,
    pub chat_history: Vec<CollabChatEntry>,
    pub is_connected: bool,
    pub room_name: String,
    pub server_url: String,
}

impl CollabSession {
    pub fn new(user_name: &str) -> Self {
        let user_id = format!("user_{:04x}", (user_name.len() * 31 + 42) & 0xffff);
        let color = generate_user_color(user_name);
        Self {
            local_user: CollabUser::new(user_id, user_name, color),
            peers: HashMap::new(),
            chat_history: Vec::new(),
            is_connected: false,
            room_name: "StudioMainRoom".to_string(),
            server_url: "wss://collab.plumb3d.io/v1".to_string(),
        }
    }

    /// Connects to a collaborative session room
    pub fn connect(&mut self, room: &str) {
        self.room_name = room.to_string();
        self.is_connected = true;
        self.chat_history.push(CollabChatEntry {
            user_name: "System".to_string(),
            text: format!("Connected to live collaboration room: '{}'", room),
            time_str: "Now".to_string(),
            color: [0.4, 0.8, 1.0],
        });
    }

    /// Disconnects from the current session
    pub fn disconnect(&mut self) {
        self.is_connected = false;
        self.peers.clear();
        self.chat_history.push(CollabChatEntry {
            user_name: "System".to_string(),
            text: "Disconnected from collaboration session.".to_string(),
            time_str: "Now".to_string(),
            color: [1.0, 0.4, 0.4],
        });
    }

    /// Updates local user's 3D cursor position
    pub fn update_cursor(&mut self, pos: Vec3) -> CollabMessage {
        self.local_user.cursor_pos = pos;
        CollabMessage::CursorMove {
            user_id: self.local_user.id.clone(),
            pos,
        }
    }

    /// Updates local selection
    pub fn update_selection(&mut self, node_id: Option<NodeId>) -> CollabMessage {
        self.local_user.selected_node_id = node_id;
        CollabMessage::Selection {
            user_id: self.local_user.id.clone(),
            node_id,
        }
    }

    /// Broadcasts local node transformation
    pub fn broadcast_transform(&self, node_id: NodeId, transform: Transform) -> CollabMessage {
        CollabMessage::NodeTransform {
            user_id: self.local_user.id.clone(),
            node_id,
            transform,
        }
    }

    /// Sends a chat message
    pub fn send_chat(&mut self, text: &str) -> CollabMessage {
        let entry = CollabChatEntry {
            user_name: self.local_user.name.clone(),
            text: text.to_string(),
            time_str: "Just now".to_string(),
            color: self.local_user.color,
        };
        self.chat_history.push(entry);

        CollabMessage::Chat {
            user_id: self.local_user.id.clone(),
            user_name: self.local_user.name.clone(),
            text: text.to_string(),
            timestamp_str: "Just now".to_string(),
        }
    }

    /// Processes an incoming message from a remote peer
    pub fn process_incoming_message(&mut self, scene: &mut SceneGraph, msg: CollabMessage) {
        match msg {
            CollabMessage::UserJoin(user) => {
                let name = user.name.clone();
                let color = user.color;
                self.peers.insert(user.id.clone(), user);
                self.chat_history.push(CollabChatEntry {
                    user_name: "System".to_string(),
                    text: format!("{} joined the session.", name),
                    time_str: "Now".to_string(),
                    color,
                });
            }
            CollabMessage::UserLeave { user_id } => {
                if let Some(user) = self.peers.remove(&user_id) {
                    self.chat_history.push(CollabChatEntry {
                        user_name: "System".to_string(),
                        text: format!("{} left the session.", user.name),
                        time_str: "Now".to_string(),
                        color: [0.7, 0.7, 0.7],
                    });
                }
            }
            CollabMessage::CursorMove { user_id, pos } => {
                if let Some(peer) = self.peers.get_mut(&user_id) {
                    peer.cursor_pos = pos;
                }
            }
            CollabMessage::Selection { user_id, node_id } => {
                if let Some(peer) = self.peers.get_mut(&user_id) {
                    peer.selected_node_id = node_id;
                }
            }
            CollabMessage::NodeTransform {
                node_id, transform, ..
            } => {
                if let Some(node) = scene.nodes.get_mut(node_id) {
                    node.local_transform = transform;
                    scene.mark_dirty(node_id);
                }
            }
            CollabMessage::Chat {
                user_id,
                user_name,
                text,
                timestamp_str,
            } => {
                let color = self
                    .peers
                    .get(&user_id)
                    .map(|p| p.color)
                    .unwrap_or([0.8, 0.8, 0.8]);
                self.chat_history.push(CollabChatEntry {
                    user_name,
                    text,
                    time_str: timestamp_str,
                    color,
                });
            }
        }
    }

    /// Number of active participants in the session (including local user)
    pub fn total_participants(&self) -> usize {
        1 + self.peers.len()
    }
}

/// Generates a pleasant distinctive RGB color for a username
fn generate_user_color(name: &str) -> [f32; 3] {
    let mut hash: u32 = 5381;
    for b in name.bytes() {
        hash = ((hash << 5).wrapping_add(hash)).wrapping_add(b as u32);
    }

    let h = (hash % 360) as f32 / 360.0;
    // HSV to RGB with S=0.7, V=0.9
    hsv_to_rgb(h, 0.75, 0.95)
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> [f32; 3] {
    let i = (h * 6.0).floor() as i32;
    let f = h * 6.0 - i as f32;
    let p = v * (1.0 - s);
    let q = v * (1.0 - f * s);
    let t = v * (1.0 - (1.0 - f) * s);

    match i % 6 {
        0 => [v, t, p],
        1 => [q, v, p],
        2 => [p, v, t],
        3 => [p, q, v],
        4 => [t, p, v],
        _ => [v, p, q],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use geo_kernel::Primitives;

    #[test]
    fn test_collab_join_and_cursor_sync() {
        let mut session = CollabSession::new("Alice");
        session.connect("StudioRoomA");
        assert!(session.is_connected);
        assert_eq!(session.total_participants(), 1);

        let bob = CollabUser::new("user_bob", "Bob", [0.2, 0.8, 0.3]);
        let mut scene = SceneGraph::new();
        session.process_incoming_message(&mut scene, CollabMessage::UserJoin(bob));
        assert_eq!(session.total_participants(), 2);

        // Move Bob's cursor
        session.process_incoming_message(
            &mut scene,
            CollabMessage::CursorMove {
                user_id: "user_bob".to_string(),
                pos: Vec3::new(10.0, 5.0, -2.0),
            },
        );

        let bob_peer = session.peers.get("user_bob").unwrap();
        assert_eq!(bob_peer.cursor_pos, Vec3::new(10.0, 5.0, -2.0));
    }

    #[test]
    fn test_collab_node_transform_sync() {
        let mut session = CollabSession::new("Alice");
        let mut scene = SceneGraph::new();
        let cube = Primitives::create_cube(1.0);
        let node = crate::node::SceneNode::new_mesh("SyncMesh", Transform::IDENTITY, cube);
        let node_id = scene.add_node(node);

        let new_transform = Transform::from_translation(Vec3::new(5.0, 0.0, 0.0));
        session.process_incoming_message(
            &mut scene,
            CollabMessage::NodeTransform {
                user_id: "user_remote".to_string(),
                node_id,
                transform: new_transform,
            },
        );

        let updated_node = scene.nodes.get(node_id).unwrap();
        assert_eq!(
            updated_node.local_transform.translation,
            Vec3::new(5.0, 0.0, 0.0)
        );
    }

    #[test]
    fn test_collab_chat() {
        let mut session = CollabSession::new("Alice");
        let msg = session.send_chat("Hello from Plumb 3D!");
        if let CollabMessage::Chat { text, .. } = msg {
            assert_eq!(text, "Hello from Plumb 3D!");
        } else {
            panic!("Expected Chat message");
        }
        assert_eq!(session.chat_history.len(), 1);
    }
}

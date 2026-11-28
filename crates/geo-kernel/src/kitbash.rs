//! Procedural Hardware Kitbash Primitives for Plumb 3D DCC.
//! Provides production-ready mechanical hardware: Hex Bolts, Allen Cap Screws,
//! Slotted Cooling Vents, and Industrial Panel Latches.

use crate::mesh::Mesh;
use core_math::{vec2, vec3};
use std::f32::consts::PI;

/// Mechanical kitbash hardware type selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum KitbashHardwareType {
    #[default]
    HexBolt,
    AllenScrew,
    SlottedVent,
    LatchHandle,
}

impl KitbashHardwareType {
    pub const ALL: [KitbashHardwareType; 4] = [
        Self::HexBolt,
        Self::AllenScrew,
        Self::SlottedVent,
        Self::LatchHandle,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            Self::HexBolt => "🔩 Hex Bolt",
            Self::AllenScrew => "⚙ Allen Cap Screw",
            Self::SlottedVent => "▦ Slotted Vent",
            Self::LatchHandle => "🔒 Latch Handle",
        }
    }

    pub fn create_mesh(&self, scale: f32) -> Mesh {
        match self {
            Self::HexBolt => Kitbash::create_hex_bolt(0.1 * scale, 0.08 * scale, 0.15 * scale),
            Self::AllenScrew => Kitbash::create_allen_screw(0.1 * scale, 0.1 * scale, 0.15 * scale),
            Self::SlottedVent => {
                Kitbash::create_slotted_vent(0.3 * scale, 0.3 * scale, 0.05 * scale, 4)
            }
            Self::LatchHandle => {
                Kitbash::create_latch_handle(0.25 * scale, 0.12 * scale, 0.04 * scale)
            }
        }
    }
}

pub struct Kitbash;

impl Kitbash {
    /// Creates a 6-sided mechanical Hex Bolt head with washer collar and thread stud.
    pub fn create_hex_bolt(radius: f32, head_height: f32, thread_len: f32) -> Mesh {
        let mut mesh = Mesh::new();
        let r = radius.max(0.001);
        let h = head_height.max(0.001);
        let tl = thread_len.max(0.001);

        // 1. Hex Head Solid (6 vertices top, 6 bottom)
        let mut top_hex = Vec::with_capacity(6);
        let mut bot_hex = Vec::with_capacity(6);
        for i in 0..6 {
            let theta = (i as f32) * 2.0 * PI / 6.0;
            let x = r * theta.cos();
            let z = r * theta.sin();
            top_hex.push(mesh.add_vertex_with_uv(
                vec3(x, h, z),
                vec2(0.5 + 0.5 * theta.cos(), 0.5 + 0.5 * theta.sin()),
            ));
            bot_hex.push(mesh.add_vertex_with_uv(
                vec3(x, 0.0, z),
                vec2(0.5 + 0.5 * theta.cos(), 0.5 + 0.5 * theta.sin()),
            ));
        }

        // Top hex cap (CCW from above)
        let _ = mesh.add_face(&top_hex);

        // 6 Side quads of hex head [t0, b0, b1, t1]
        for i in 0..6 {
            let next = (i + 1) % 6;
            let _ = mesh.add_face(&[top_hex[i], bot_hex[i], bot_hex[next], top_hex[next]]);
        }

        // Bottom hex cap (reversed CCW from below)
        let mut base_rev = bot_hex.clone();
        base_rev.reverse();
        let _ = mesh.add_face(&base_rev);

        // 2. Thread Stud (cylindrical stud below head)
        let stud_r = r * 0.7;
        let mut stud_top = Vec::with_capacity(8);
        let mut stud_bot = Vec::with_capacity(8);
        for i in 0..8 {
            let theta = (i as f32) * 2.0 * PI / 8.0;
            let x = stud_r * theta.cos();
            let z = stud_r * theta.sin();
            stud_top.push(mesh.add_vertex_with_uv(vec3(x, 0.0, z), vec2((i as f32) / 8.0, 1.0)));
            stud_bot.push(mesh.add_vertex_with_uv(vec3(x, -tl, z), vec2((i as f32) / 8.0, 0.0)));
        }

        for i in 0..8 {
            let next = (i + 1) % 8;
            let _ = mesh.add_face(&[stud_top[i], stud_bot[i], stud_bot[next], stud_top[next]]);
        }
        let _ = mesh.add_face(&stud_top);
        let mut stud_bot_rev = stud_bot.clone();
        stud_bot_rev.reverse();
        let _ = mesh.add_face(&stud_bot_rev);

        mesh.update_face_weighted_normals(true);
        mesh
    }

    /// Creates an Allen Cap Screw with cylindrical knurled head and recessed hex socket.
    pub fn create_allen_screw(radius: f32, head_height: f32, thread_len: f32) -> Mesh {
        let mut mesh = Mesh::new();
        let r = radius.max(0.001);
        let h = head_height.max(0.001);
        let tl = thread_len.max(0.001);
        let segs = 12;

        // Outer cylinder
        let mut top_outer = Vec::with_capacity(segs);
        let mut bot_outer = Vec::with_capacity(segs);
        for i in 0..segs {
            let theta = (i as f32) * 2.0 * PI / (segs as f32);
            let x = r * theta.cos();
            let z = r * theta.sin();
            top_outer.push(
                mesh.add_vertex_with_uv(vec3(x, h, z), vec2((i as f32) / (segs as f32), 1.0)),
            );
            bot_outer.push(
                mesh.add_vertex_with_uv(vec3(x, 0.0, z), vec2((i as f32) / (segs as f32), 0.0)),
            );
        }

        for i in 0..segs {
            let next = (i + 1) % segs;
            let _ = mesh.add_face(&[top_outer[i], bot_outer[i], bot_outer[next], top_outer[next]]);
        }
        let _ = mesh.add_face(&top_outer);
        let mut bot_rev = bot_outer.clone();
        bot_rev.reverse();
        let _ = mesh.add_face(&bot_rev);

        // Recessed hex socket on top
        let hex_r = r * 0.55;
        let hex_depth = h * 0.5;
        let mut socket_top = Vec::with_capacity(6);
        let mut socket_bot = Vec::with_capacity(6);
        for i in 0..6 {
            let theta = (i as f32) * 2.0 * PI / 6.0;
            let x = hex_r * theta.cos();
            let z = hex_r * theta.sin();
            socket_top.push(mesh.add_vertex(vec3(x, h + 0.0001, z)));
            socket_bot.push(mesh.add_vertex(vec3(x, h - hex_depth, z)));
        }
        for i in 0..6 {
            let next = (i + 1) % 6;
            let _ = mesh.add_face(&[
                socket_top[i],
                socket_bot[i],
                socket_bot[next],
                socket_top[next],
            ]);
        }
        let _ = mesh.add_face(&socket_top);
        let mut s_bot_rev = socket_bot.clone();
        s_bot_rev.reverse();
        let _ = mesh.add_face(&s_bot_rev);

        // Thread stud cylinder
        let stud_r = r * 0.75;
        let mut stud_top = Vec::with_capacity(8);
        let mut stud_bot = Vec::with_capacity(8);
        for i in 0..8 {
            let theta = (i as f32) * 2.0 * PI / 8.0;
            let x = stud_r * theta.cos();
            let z = stud_r * theta.sin();
            stud_top.push(mesh.add_vertex_with_uv(vec3(x, 0.0, z), vec2((i as f32) / 8.0, 1.0)));
            stud_bot.push(mesh.add_vertex_with_uv(vec3(x, -tl, z), vec2((i as f32) / 8.0, 0.0)));
        }
        for i in 0..8 {
            let next = (i + 1) % 8;
            let _ = mesh.add_face(&[stud_top[i], stud_bot[i], stud_bot[next], stud_top[next]]);
        }
        let _ = mesh.add_face(&stud_top);
        let mut stud_bot_rev = stud_bot.clone();
        stud_bot_rev.reverse();
        let _ = mesh.add_face(&stud_bot_rev);

        mesh.update_face_weighted_normals(true);
        mesh
    }

    /// Creates an industrial Slotted Cooling Vent panel with louvers.
    pub fn create_slotted_vent(width: f32, length: f32, depth: f32, slots: usize) -> Mesh {
        let mut mesh = Mesh::new();
        let w = width.max(0.01) * 0.5;
        let l = length.max(0.01) * 0.5;
        let d = depth.max(0.005);
        let n_slots = slots.clamp(2, 10);

        // Outer chassis box
        let p0 = mesh.add_vertex_with_uv(vec3(-w, 0.0, -l), vec2(0.0, 0.0));
        let p1 = mesh.add_vertex_with_uv(vec3(w, 0.0, -l), vec2(1.0, 0.0));
        let p2 = mesh.add_vertex_with_uv(vec3(w, 0.0, l), vec2(1.0, 1.0));
        let p3 = mesh.add_vertex_with_uv(vec3(-w, 0.0, l), vec2(0.0, 1.0));

        let b0 = mesh.add_vertex_with_uv(vec3(-w, -d, -l), vec2(0.0, 0.0));
        let b1 = mesh.add_vertex_with_uv(vec3(w, -d, -l), vec2(1.0, 0.0));
        let b2 = mesh.add_vertex_with_uv(vec3(w, -d, l), vec2(1.0, 1.0));
        let b3 = mesh.add_vertex_with_uv(vec3(-w, -d, l), vec2(0.0, 1.0));

        // Outer walls & bottom
        let _ = mesh.add_face(&[b0, b1, b2, b3]);
        let _ = mesh.add_face(&[p0, p1, b1, b0]);
        let _ = mesh.add_face(&[p1, p2, b2, b1]);
        let _ = mesh.add_face(&[p2, p3, b3, b2]);
        let _ = mesh.add_face(&[p3, p0, b0, b3]);

        // Angled louver slats
        let step = (l * 1.6) / (n_slots as f32);
        let slot_w = w * 0.75;
        let slot_thick = step * 0.45;

        for s in 0..n_slots {
            let z_start = -l * 0.8 + (s as f32) * step;
            let z_end = z_start + slot_thick;

            let l0 = mesh.add_vertex(vec3(-slot_w, 0.001, z_start));
            let l1 = mesh.add_vertex(vec3(slot_w, 0.001, z_start));
            let l2 = mesh.add_vertex(vec3(slot_w, -d * 0.8, z_end));
            let l3 = mesh.add_vertex(vec3(-slot_w, -d * 0.8, z_end));
            let _ = mesh.add_face(&[l0, l1, l2, l3]);
        }

        let _ = mesh.add_face(&[p0, p3, p2, p1]);
        mesh.update_face_weighted_normals(true);
        mesh
    }

    /// Creates an industrial toggle latch / recessed cabinet handle.
    pub fn create_latch_handle(width: f32, height: f32, depth: f32) -> Mesh {
        let mut mesh = Mesh::new();
        let w = width.max(0.01) * 0.5;
        let h = height.max(0.01) * 0.5;
        let d = depth.max(0.005);

        // Mounting bezel base
        let v0 = mesh.add_vertex(vec3(-w, 0.0, -h));
        let v1 = mesh.add_vertex(vec3(w, 0.0, -h));
        let v2 = mesh.add_vertex(vec3(w, 0.0, h));
        let v3 = mesh.add_vertex(vec3(-w, 0.0, h));

        let b0 = mesh.add_vertex(vec3(-w, -d, -h));
        let b1 = mesh.add_vertex(vec3(w, -d, -h));
        let b2 = mesh.add_vertex(vec3(w, -d, h));
        let b3 = mesh.add_vertex(vec3(-w, -d, h));

        let _ = mesh.add_face(&[b0, b1, b2, b3]);
        let _ = mesh.add_face(&[v0, v1, b1, b0]);
        let _ = mesh.add_face(&[v1, v2, b2, b1]);
        let _ = mesh.add_face(&[v2, v3, b3, b2]);
        let _ = mesh.add_face(&[v3, v0, b0, b3]);

        // Raised toggle handle lever
        let hw = w * 0.45;
        let hh = h * 0.65;
        let hd = d * 1.5;

        let t0 = mesh.add_vertex(vec3(-hw, hd, -hh));
        let t1 = mesh.add_vertex(vec3(hw, hd, -hh));
        let t2 = mesh.add_vertex(vec3(hw, hd * 0.4, hh));
        let t3 = mesh.add_vertex(vec3(-hw, hd * 0.4, hh));

        let _ = mesh.add_face(&[t0, t1, t2, t3]);
        let _ = mesh.add_face(&[v0, v3, v2, v1]);

        mesh.update_face_weighted_normals(true);
        mesh
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kitbash_generators() {
        let hex = Kitbash::create_hex_bolt(0.1, 0.08, 0.15);
        assert!(!hex.vertices.is_empty());
        assert!(!hex.faces.is_empty());

        let screw = Kitbash::create_allen_screw(0.1, 0.1, 0.15);
        assert!(!screw.vertices.is_empty());
        assert!(!screw.faces.is_empty());

        let vent = Kitbash::create_slotted_vent(0.3, 0.3, 0.05, 4);
        assert!(!vent.vertices.is_empty());
        assert!(!vent.faces.is_empty());

        let latch = Kitbash::create_latch_handle(0.25, 0.12, 0.04);
        assert!(!latch.vertices.is_empty());
        assert!(!latch.faces.is_empty());
    }

    #[test]
    fn test_hardware_types() {
        for t in KitbashHardwareType::ALL {
            let m = t.create_mesh(1.0);
            assert!(m.vertices.len() >= 8);
            assert!(!t.label().is_empty());
        }
    }
}

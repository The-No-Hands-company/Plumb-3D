//! Dynamic Hydraulic Pipe & Braided Wiring Harness Generator for Plumb 3D DCC.
//! Generates physical catenary droop cables, pneumatic hoses, and braided harnesses
//! with industrial end fittings and couplings between ports.

use crate::mesh::Mesh;
use core_math::Vec3;
use std::f32::consts::PI;

pub struct CableGenerator;

impl CableGenerator {
    /// Generates a complete 3D hydraulic hose / braided cable spanning Port A to Port B.
    ///
    /// - Includes mechanical hex coupling nuts at both ends.
    /// - Calculates catenary gravity sag: $y(t) = -4 \cdot \text{slack} \cdot L \cdot t(1 - t)$.
    /// - Generates corrugated braided quad rings along the hose body.
    pub fn create_cable(
        port_a: Vec3,
        port_b: Vec3,
        radius: f32,
        slack: f32,
        segments: usize,
        radial_segments: usize,
    ) -> Mesh {
        let mut mesh = Mesh::new();
        let segs = segments.max(4);
        let radial = radial_segments.max(6);
        let r = radius.max(0.002);

        let ab = port_b - port_a;
        let length = ab.length().max(0.01);

        // 1. Compute catenary curve spine stations
        let mut spine_points = Vec::with_capacity(segs + 1);
        for i in 0..=segs {
            let t = i as f32 / segs as f32;
            let linear_p = port_a + ab * t;

            // Physical catenary gravity droop
            let droop = -4.0 * slack.max(0.0) * length * t * (1.0 - t);
            let p = linear_p + Vec3::new(0.0, droop, 0.0);
            spine_points.push(p);
        }

        // 2. Generate radial vertex rings along the spine
        let mut rings: Vec<Vec<_>> = Vec::with_capacity(segs + 1);
        for i in 0..=segs {
            let p = spine_points[i];
            let tangent = if i == 0 {
                (spine_points[1] - spine_points[0]).normalize_or_zero()
            } else if i == segs {
                (spine_points[segs] - spine_points[segs - 1]).normalize_or_zero()
            } else {
                (spine_points[i + 1] - spine_points[i - 1]).normalize_or_zero()
            };

            let up = if tangent.y.abs() < 0.9 { Vec3::Y } else { Vec3::Z };
            let binormal = tangent.cross(up).normalize_or_zero();
            let normal = binormal.cross(tangent).normalize_or_zero();

            // Slight corrugated rib oscillation for braided industrial texture
            let rib_mod = if i % 2 == 1 && i > 0 && i < segs { 1.08 } else { 1.0 };
            let cur_r = r * rib_mod;

            let mut ring = Vec::with_capacity(radial);
            for s in 0..radial {
                let theta = (s as f32) * 2.0 * PI / (radial as f32);
                let offset = binormal * (theta.cos() * cur_r) + normal * (theta.sin() * cur_r);
                ring.push(mesh.add_vertex(p + offset));
            }
            rings.push(ring);
        }

        // 3. Connect quad bands along hose body
        for i in 0..segs {
            for s in 0..radial {
                let next_s = (s + 1) % radial;
                let v0 = rings[i][s];
                let v1 = rings[i][next_s];
                let v2 = rings[i + 1][next_s];
                let v3 = rings[i + 1][s];
                let _ = mesh.add_face(&[v0, v1, v2, v3]);
            }
        }

        // 4. End cap Port A
        let mut cap_a = rings[0].clone();
        cap_a.reverse();
        let _ = mesh.add_face(&cap_a);

        // End cap Port B
        let _ = mesh.add_face(&rings[segs]);

        // 5. Hex Fitting Couplings at Port A and Port B
        Self::add_hex_fitting(&mut mesh, port_a, (spine_points[1] - port_a).normalize_or_zero(), r * 1.5, r * 1.8);
        Self::add_hex_fitting(&mut mesh, port_b, (spine_points[segs - 1] - port_b).normalize_or_zero(), r * 1.5, r * 1.8);

        mesh.update_face_weighted_normals(true);
        mesh
    }

    /// Adds a 6-sided mechanical hex coupling nut at a port terminal.
    fn add_hex_fitting(mesh: &mut Mesh, origin: Vec3, dir: Vec3, nut_radius: f32, nut_length: f32) {
        let up = if dir.y.abs() < 0.9 { Vec3::Y } else { Vec3::Z };
        let binormal = dir.cross(up).normalize_or_zero();
        let normal = binormal.cross(dir).normalize_or_zero();

        let end = origin + dir * nut_length;

        let mut ring_start = Vec::with_capacity(6);
        let mut ring_end = Vec::with_capacity(6);

        for i in 0..6 {
            let theta = (i as f32) * 2.0 * PI / 6.0;
            let offset = binormal * (theta.cos() * nut_radius) + normal * (theta.sin() * nut_radius);
            ring_start.push(mesh.add_vertex(origin + offset));
            ring_end.push(mesh.add_vertex(end + offset));
        }

        for i in 0..6 {
            let next = (i + 1) % 6;
            let _ = mesh.add_face(&[ring_start[i], ring_start[next], ring_end[next], ring_end[i]]);
        }
        let _ = mesh.add_face(&ring_end);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core_math::vec3;

    #[test]
    fn test_create_hydraulic_cable() {
        let port_a = vec3(-2.0, 1.0, 0.0);
        let port_b = vec3(2.0, 1.0, 0.0);

        let cable = CableGenerator::create_cable(port_a, port_b, 0.05, 0.2, 10, 8);
        assert!(!cable.vertices.is_empty());
        assert!(!cable.faces.is_empty());
        assert!(cable.validate().is_ok());
    }

    #[test]
    fn test_straight_conduit_zero_slack() {
        let p_a = vec3(0.0, 0.0, 0.0);
        let p_b = vec3(0.0, 3.0, 0.0);

        let conduit = CableGenerator::create_cable(p_a, p_b, 0.04, 0.0, 6, 6);
        assert!(conduit.faces.len() >= 36);
        assert!(conduit.validate().is_ok());
    }
}

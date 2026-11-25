//! Variable Radius Fillet Operator for Plumb 3D DCC.
//! Enables smooth tapered chamfers ($R_1 \to R_2$) along edge chains
//! with $G^1$ (tangent) and $G^2$ (curvature continuous) profile transitions.

use crate::mesh::{EdgeKey, FaceKey, Mesh, MeshError};
use crate::operators::bevel::BevelProfile;
use core_math::Vec3;
use std::f32::consts::PI;

/// Geometric continuity mode for variable radius filleting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FilletContinuity {
    /// Linear radius blend along edge chain
    Linear,
    /// $G^1$ Tangent continuous (Hermite cubic smoothstep: $3t^2 - 2t^3$)
    #[default]
    G1Tangent,
    /// $G^2$ Curvature continuous (Quintic smoothstep: $6t^5 - 15t^4 + 10t^3$)
    G2Curvature,
}

impl FilletContinuity {
    /// Evaluates blending factor $s \in [0.0, 1.0]$ for parametric distance $t \in [0.0, 1.0]$.
    pub fn blend(&self, t: f32) -> f32 {
        let clamped = t.clamp(0.0, 1.0);
        match self {
            Self::Linear => clamped,
            Self::G1Tangent => clamped * clamped * (3.0 - 2.0 * clamped),
            Self::G2Curvature => {
                clamped * clamped * clamped * (clamped * (clamped * 6.0 - 15.0) + 10.0)
            }
        }
    }
}

impl Mesh {
    /// Bevels an edge with variable start radius $R_1$ and end radius $R_2$.
    ///
    /// Constructs a tapered quad fillet strip along the edge with $G^1$ or $G^2$ continuity.
    pub fn bevel_edge_variable_radius(
        &mut self,
        edge_key: EdgeKey,
        r1: f32,
        r2: f32,
        segments: usize,
        continuity: FilletContinuity,
        profile: BevelProfile,
    ) -> Result<Vec<FaceKey>, MeshError> {
        let seg_count = segments.max(1);
        let edge = self
            .edges
            .get(edge_key)
            .ok_or_else(|| MeshError::InvariantFailed("Edge not found".into()))?;
        let h1_key = edge.half_edge;
        let h2_key = self.half_edges[h1_key].twin;

        let f1_key = self.half_edges[h1_key]
            .face
            .ok_or_else(|| MeshError::InvariantFailed("Edge requires two faces to bevel".into()))?;
        let f2_key = self.half_edges[h2_key]
            .face
            .ok_or_else(|| MeshError::InvariantFailed("Edge requires two faces to bevel".into()))?;

        let v0 = self.half_edges[h1_key].origin;
        let v1 = self.half_edges[h1_key].target;

        let p0 = self.vertices[v0].position;
        let p1 = self.vertices[v1].position;
        let edge_dir = (p1 - p0).normalize_or_zero();

        let n1 = self.faces[f1_key].normal;
        let n2 = self.faces[f2_key].normal;

        // Inward perpendiculars at v0 (scaled by r1) and v1 (scaled by r2)
        let perp1_0 = edge_dir.cross(n1).normalize_or_zero() * r1;
        let perp2_0 = (-edge_dir).cross(n2).normalize_or_zero() * r1;

        let perp1_1 = edge_dir.cross(n1).normalize_or_zero() * r2;
        let perp2_1 = (-edge_dir).cross(n2).normalize_or_zero() * r2;

        let mut v0_strip = Vec::with_capacity(seg_count + 1);
        let mut v1_strip = Vec::with_capacity(seg_count + 1);

        let outward_dir_0 = -(perp1_0 + perp2_0).normalize_or_zero();
        let outward_dir_1 = -(perp1_1 + perp2_1).normalize_or_zero();

        let bump_factor = match profile {
            BevelProfile::Round(tension) => tension.clamp(0.0, 1.0) * 0.828,
            BevelProfile::Concave => -0.414,
            BevelProfile::Stepped => 0.0,
        };

        for i in 0..=seg_count {
            let t_raw = i as f32 / seg_count as f32;
            let t = continuity.blend(t_raw);

            let bump_0 = if seg_count > 1 {
                match profile {
                    BevelProfile::Stepped => {
                        if i % 2 == 1 { outward_dir_0 * (r1 * 0.25) } else { Vec3::ZERO }
                    }
                    _ => outward_dir_0 * (r1 * bump_factor * (t * PI).sin()),
                }
            } else {
                Vec3::ZERO
            };

            let bump_1 = if seg_count > 1 {
                match profile {
                    BevelProfile::Stepped => {
                        if i % 2 == 1 { outward_dir_1 * (r2 * 0.25) } else { Vec3::ZERO }
                    }
                    _ => outward_dir_1 * (r2 * bump_factor * (t * PI).sin()),
                }
            } else {
                Vec3::ZERO
            };

            let pt0 = p0 + (1.0 - t) * perp1_0 + t * perp2_0 + bump_0;
            let pt1 = p1 + (1.0 - t) * perp1_1 + t * perp2_1 + bump_1;

            v0_strip.push(self.add_vertex(pt0));
            v1_strip.push(self.add_vertex(pt1));
        }

        let v0_f1 = v0_strip[0];
        let v1_f1 = v1_strip[0];
        let v0_f2 = v0_strip[seg_count];
        let v1_f2 = v1_strip[seg_count];

        // Update face 1: replace v0, v1 with v0_f1, v1_f1
        let mut f1_verts = self.face_vertices(f1_key);
        if let Some(pos) = f1_verts.iter().position(|&v| v == v0) {
            let next_idx = (pos + 1) % f1_verts.len();
            if f1_verts[next_idx] == v1 {
                f1_verts[pos] = v0_f1;
                f1_verts[next_idx] = v1_f1;
            }
        }

        // Update face 2: replace v1, v0 with v1_f2, v0_f2
        let mut f2_verts = self.face_vertices(f2_key);
        if let Some(pos) = f2_verts.iter().position(|&v| v == v1) {
            let next_idx = (pos + 1) % f2_verts.len();
            if f2_verts[next_idx] == v0 {
                f2_verts[pos] = v1_f2;
                f2_verts[next_idx] = v0_f2;
            }
        }

        // Detach old faces
        for he_key in self.face_half_edges(f1_key) {
            self.half_edges[he_key].face = None;
        }
        self.faces.remove(f1_key);

        for he_key in self.face_half_edges(f2_key) {
            self.half_edges[he_key].face = None;
        }
        self.faces.remove(f2_key);

        // Remove old edge and half-edges
        self.vertices[v0].half_edges.retain(|he| *he != h1_key);
        self.vertices[v1].half_edges.retain(|he| *he != h2_key);
        self.half_edges.remove(h1_key);
        self.half_edges.remove(h2_key);
        self.edges.remove(edge_key);

        // Re-add updated adjacent faces
        self.add_face(&f1_verts)?;
        self.add_face(&f2_verts)?;

        // Add the new variable chamfer/fillet quad faces
        let mut created_faces = Vec::with_capacity(seg_count);
        for s in 0..seg_count {
            let fk = self.add_face(&[
                v1_strip[s],
                v0_strip[s],
                v0_strip[s + 1],
                v1_strip[s + 1],
            ])?;
            created_faces.push(fk);
        }

        self.update_face_weighted_normals(true);
        Ok(created_faces)
    }

    /// Bevels an ordered chain of edges with smoothly varying radius $R_1 \to R_2$.
    pub fn bevel_edge_chain_variable(
        &mut self,
        edge_chain: &[EdgeKey],
        r1: f32,
        r2: f32,
        segments: usize,
        continuity: FilletContinuity,
        profile: BevelProfile,
    ) -> Result<Vec<FaceKey>, MeshError> {
        if edge_chain.is_empty() {
            return Ok(Vec::new());
        }

        let total = edge_chain.len();
        let mut all_created = Vec::new();

        for (i, &edge_key) in edge_chain.iter().enumerate() {
            let t_start = i as f32 / total as f32;
            let t_end = (i + 1) as f32 / total as f32;

            let blend_start = continuity.blend(t_start);
            let blend_end = continuity.blend(t_end);

            let cur_r1 = r1 + (r2 - r1) * blend_start;
            let cur_r2 = r1 + (r2 - r1) * blend_end;

            if let Ok(faces) = self.bevel_edge_variable_radius(
                edge_key,
                cur_r1,
                cur_r2,
                segments,
                continuity,
                profile,
            ) {
                all_created.extend(faces);
            }
        }

        Ok(all_created)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::Primitives;

    #[test]
    fn test_variable_radius_fillet_cube_edge() {
        let mut cube = Primitives::create_cube(2.0);
        let edge_key = cube.edges.keys().next().unwrap();

        let faces = cube
            .bevel_edge_variable_radius(
                edge_key,
                0.1,
                0.3,
                2,
                FilletContinuity::G2Curvature,
                BevelProfile::Round(0.5),
            )
            .expect("variable radius bevel");

        assert_eq!(faces.len(), 2);
        assert!(!cube.faces.is_empty());
        assert!(cube.validate().is_ok());
    }

    #[test]
    fn test_continuity_blending() {
        let g1 = FilletContinuity::G1Tangent;
        let g2 = FilletContinuity::G2Curvature;

        assert!((g1.blend(0.0) - 0.0).abs() < 1e-4);
        assert!((g1.blend(1.0) - 1.0).abs() < 1e-4);
        assert!((g1.blend(0.5) - 0.5).abs() < 1e-4);

        assert!((g2.blend(0.0) - 0.0).abs() < 1e-4);
        assert!((g2.blend(1.0) - 1.0).abs() < 1e-4);
        assert!((g2.blend(0.5) - 0.5).abs() < 1e-4);
    }
}

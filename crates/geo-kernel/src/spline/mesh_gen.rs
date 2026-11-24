use std::collections::{HashMap, HashSet};
use std::f32::consts::PI;
use core_math::{vec2, Vec2, Vec3};
use crate::mesh::{EdgeKey, Mesh, MeshError, VertexKey};

use super::profiles::ProfileShape;
use super::types::{BezierPoint, Spline};

impl Spline {
    /// Helper to generate a regular polygon / circular 2D cross-section with `subdivisions` points.
    pub fn circle_cross_section(radius: f32, subdivisions: usize) -> Vec<Vec2> {
        let n = subdivisions.max(3);
        (0..n)
            .map(|i| {
                let angle = (i as f32 / n as f32) * 2.0 * PI;
                vec2(radius * angle.cos(), radius * angle.sin())
            })
            .collect()
    }

    /// Converts the spline directly into a 3D cylindrical pipe/tube mesh.
    pub fn to_pipe_mesh(&self, radius: f32, radial_segments: usize, samples_per_segment: usize) -> Result<Mesh, MeshError> {
        self.to_extruded_mesh(ProfileShape::Circle, radius, radial_segments, samples_per_segment)
    }

    /// Converts the spline directly into a 3D extruded mesh using the specified cross-section profile.
    pub fn to_extruded_mesh(
        &self,
        profile: ProfileShape,
        radius: f32,
        radial_segments: usize,
        samples_per_segment: usize,
    ) -> Result<Mesh, MeshError> {
        if self.points.len() < 2 {
            return Err(MeshError::DegenerateFace(self.points.len()));
        }
        let cs = profile.generate_cross_section(radius.max(1e-4), radial_segments.max(3));
        let sampled = self.sample(samples_per_segment.max(2));
        Self::sweep(&cs, &sampled, self.is_closed)
    }
}

impl Mesh {
    /// Extracts a smooth 3D cubic Bézier spline from a chain or loop of selected mesh edges.
    pub fn extract_curve_from_edges(&self, edges: &[EdgeKey]) -> Result<Spline, MeshError> {
        if edges.is_empty() {
            return Err(MeshError::DegenerateFace(0));
        }

        // 1. Build adjacency graph for the selected edges
        let mut adj: HashMap<VertexKey, Vec<(VertexKey, EdgeKey)>> = HashMap::new();
        for &ek in edges {
            let edge = self.edges.get(ek).ok_or(MeshError::InvalidEdge(ek))?;
            let he = &self.half_edges[edge.half_edge];
            let v0 = he.origin;
            let v1 = he.target;
            adj.entry(v0).or_default().push((v1, ek));
            adj.entry(v1).or_default().push((v0, ek));
        }

        // 2. Identify start vertex: endpoint (degree == 1) if open, or any vertex if closed
        let start_vk = adj.iter()
            .find(|(_, neighbors)| neighbors.len() == 1)
            .map(|(&vk, _)| vk)
            .or_else(|| adj.keys().next().copied())
            .ok_or(MeshError::DegenerateFace(0))?;

        // 3. Walk along unvisited edges to form an ordered sequence of vertices
        let mut ordered_verts = Vec::new();
        let mut visited_edges = HashSet::new();
        let mut current_vk = start_vk;
        ordered_verts.push(current_vk);

        let mut is_closed = false;
        loop {
            // Find next unvisited edge incident to current_vk
            let next_step = adj.get(&current_vk).and_then(|neighbors| {
                neighbors.iter().find(|(_, ek)| !visited_edges.contains(ek))
            });

            if let Some(&(next_vk, ek)) = next_step {
                visited_edges.insert(ek);
                if next_vk == start_vk {
                    is_closed = true;
                    break;
                }
                ordered_verts.push(next_vk);
                current_vk = next_vk;
            } else {
                break;
            }
        }

        if ordered_verts.len() < 2 {
            return Err(MeshError::DegenerateFace(ordered_verts.len()));
        }

        // 4. Construct cubic Bézier control points with smooth tangents
        let n = ordered_verts.len();
        let mut spline_points = Vec::with_capacity(n);

        for i in 0..n {
            let pos = self.vertices[ordered_verts[i]].position;

            let tan = if is_closed {
                let prev_pos = self.vertices[ordered_verts[(i + n - 1) % n]].position;
                let next_pos = self.vertices[ordered_verts[(i + 1) % n]].position;
                (next_pos - prev_pos) * (0.5 * 0.333)
            } else if i == 0 {
                let next_pos = self.vertices[ordered_verts[1]].position;
                (next_pos - pos) * 0.333
            } else if i == n - 1 {
                let prev_pos = self.vertices[ordered_verts[n - 2]].position;
                (pos - prev_pos) * 0.333
            } else {
                let prev_pos = self.vertices[ordered_verts[i - 1]].position;
                let next_pos = self.vertices[ordered_verts[i + 1]].position;
                (next_pos - prev_pos) * (0.5 * 0.333)
            };

            let (h_in, h_out) = if !is_closed && i == 0 {
                (Vec3::ZERO, tan)
            } else if !is_closed && i == n - 1 {
                (-tan, Vec3::ZERO)
            } else {
                (-tan, tan)
            };

            spline_points.push(BezierPoint::new(pos, h_in, h_out));
        }

        Ok(Spline::with_points(spline_points, is_closed))
    }
}

//! Mesh Generation & 3D BMesh Conversion for 2D CAD Sketches.
//!
//! Provides solid extrusion, revolve/lathe, and 3D Spline extraction from solved 2D sketches.

use super::entities::{Sketch, SketchEntity};
use crate::mesh::Mesh;
use crate::spline::{BezierPoint, Spline};
use core_math::Vec2;

/// Extrudes a closed 2D polygon profile from a sketch along the workplane normal into a 2-manifold BMesh.
pub fn extrude_sketch(sketch: &Sketch, depth: f32) -> Result<Mesh, String> {
    let loops = extract_closed_polygon_loops(sketch);
    if loops.is_empty() {
        return Err("No closed polygon loops found in sketch to extrude".into());
    }

    let mut mesh = Mesh::new();
    let normal = sketch.workplane.normal;
    let offset_3d = normal * depth;

    for poly in loops {
        let n = poly.len();
        if n < 3 {
            continue;
        }

        let mut bottom_verts = Vec::with_capacity(n);
        let mut top_verts = Vec::with_capacity(n);

        // Create bottom and top vertices in 3D
        for &uv in &poly {
            let p_bottom = sketch.workplane.to_3d(uv);
            let p_top = p_bottom + offset_3d;
            bottom_verts.push(mesh.add_vertex(p_bottom));
            top_verts.push(mesh.add_vertex(p_top));
        }

        // Create quad side walls
        for i in 0..n {
            let next = (i + 1) % n;
            let v0 = bottom_verts[i];
            let v1 = bottom_verts[next];
            let v2 = top_verts[next];
            let v3 = top_verts[i];

            if depth >= 0.0 {
                mesh.add_face(&[v0, v1, v2, v3])
                    .map_err(|e| format!("Failed to create side wall: {:?}", e))?;
            } else {
                mesh.add_face(&[v3, v2, v1, v0])
                    .map_err(|e| format!("Failed to create side wall: {:?}", e))?;
            }
        }

        // Create bottom cap (reversed winding for outward normal)
        let mut bottom_reversed = bottom_verts.clone();
        bottom_reversed.reverse();
        if depth >= 0.0 {
            let _ = mesh.add_face(&bottom_reversed);
        } else {
            let _ = mesh.add_face(&bottom_verts);
        }

        // Create top cap
        if depth >= 0.0 {
            let _ = mesh.add_face(&top_verts);
        } else {
            let mut top_reversed = top_verts.clone();
            top_reversed.reverse();
            let _ = mesh.add_face(&top_reversed);
        }
    }

    let _ = mesh.recalculate_face_normals(false);
    Ok(mesh)
}

/// Revolves a 2D sketch profile around an axis on the workplane by `angle_rad`.
pub fn revolve_sketch(
    sketch: &Sketch,
    axis_origin: Vec2,
    axis_dir: Vec2,
    angle_rad: f32,
    segments: usize,
) -> Result<Mesh, String> {
    let loops = extract_closed_polygon_loops(sketch);
    if loops.is_empty() {
        return Err("No closed polygon loops found in sketch to revolve".into());
    }

    let segs = segments.max(4);
    let mut mesh = Mesh::new();
    let norm_axis = axis_dir.normalize_or_zero();
    let perp_axis = Vec2::new(-norm_axis.y, norm_axis.x);

    for poly in loops {
        let n = poly.len();
        if n < 3 {
            continue;
        }

        let mut ring_vertices = Vec::with_capacity(segs + 1);

        for step in 0..=segs {
            let theta = (step as f32 / segs as f32) * angle_rad;
            let cos_t = theta.cos();
            let sin_t = theta.sin();

            let mut current_ring = Vec::with_capacity(n);
            for &p2d in &poly {
                let rel = p2d - axis_origin;
                let u_dist = rel.dot(norm_axis);
                let r_dist = rel.dot(perp_axis);

                let rotated_2d = axis_origin + norm_axis * u_dist + perp_axis * (r_dist * cos_t);
                let mut p3d = sketch.workplane.to_3d(rotated_2d);
                p3d += sketch.workplane.normal * (r_dist * sin_t);

                current_ring.push(mesh.add_vertex(p3d));
            }
            ring_vertices.push(current_ring);
        }

        // Bridge rings with quad faces
        for s in 0..segs {
            for i in 0..n {
                let next = (i + 1) % n;
                let v0 = ring_vertices[s][i];
                let v1 = ring_vertices[s][next];
                let v2 = ring_vertices[s + 1][next];
                let v3 = ring_vertices[s + 1][i];

                let _ = mesh.add_face(&[v0, v1, v2, v3]);
            }
        }
    }

    let _ = mesh.recalculate_face_normals(false);
    Ok(mesh)
}

/// Converts sketch entities into 3D cubic Bézier splines.
pub fn sketch_to_splines(sketch: &Sketch) -> Vec<Spline> {
    let mut splines = Vec::new();

    for entity in &sketch.entities {
        if entity.is_construction() {
            continue;
        }

        match entity {
            SketchEntity::Line(l) => {
                if let (Some(p1), Some(p2)) = (sketch.get_point(l.start), sketch.get_point(l.end)) {
                    let w1 = sketch.workplane.to_3d(p1.pos);
                    let w2 = sketch.workplane.to_3d(p2.pos);
                    let spline = Spline::with_points(
                        vec![BezierPoint::sharp(w1), BezierPoint::sharp(w2)],
                        false,
                    );
                    splines.push(spline);
                }
            }
            SketchEntity::Circle(c) => {
                if let Some(cp) = sketch.get_point(c.center) {
                    let center_3d = sketch.workplane.to_3d(cp.pos);
                    let circle_spline = Spline::circle(c.radius, center_3d);
                    splines.push(circle_spline);
                }
            }
            SketchEntity::Arc(a) => {
                if let (Some(cp), Some(sp), Some(ep)) = (
                    sketch.get_point(a.center),
                    sketch.get_point(a.start),
                    sketch.get_point(a.end),
                ) {
                    let c3 = sketch.workplane.to_3d(cp.pos);
                    let s3 = sketch.workplane.to_3d(sp.pos);
                    let e3 = sketch.workplane.to_3d(ep.pos);
                    let mid3 = (s3 + e3) * 0.5;
                    let spline = Spline::with_points(
                        vec![
                            BezierPoint::smooth(s3, (mid3 - s3) * 0.5),
                            BezierPoint::smooth(e3, (e3 - mid3) * 0.5),
                        ],
                        false,
                    );
                    let _ = c3;
                    splines.push(spline);
                }
            }
        }
    }

    splines
}

/// Helper to trace closed connected loops of sketch line segments.
fn extract_closed_polygon_loops(sketch: &Sketch) -> Vec<Vec<Vec2>> {
    let mut lines = Vec::new();
    for entity in &sketch.entities {
        if let SketchEntity::Line(l) = entity {
            if !l.is_construction {
                if let (Some(p1), Some(p2)) = (sketch.get_point(l.start), sketch.get_point(l.end)) {
                    lines.push((p1.pos, p2.pos));
                }
            }
        }
    }

    if lines.len() < 3 {
        return Vec::new();
    }

    let mut loops = Vec::new();
    let mut visited = vec![false; lines.len()];

    for i in 0..lines.len() {
        if visited[i] {
            continue;
        }

        let mut current_loop = vec![lines[i].0, lines[i].1];
        visited[i] = true;
        let mut loop_closed = false;

        while !loop_closed {
            let last_pt = *current_loop.last().unwrap();
            let mut found_next = false;

            for j in 0..lines.len() {
                if visited[j] {
                    continue;
                }

                if (lines[j].0 - last_pt).length() < 1e-4 {
                    visited[j] = true;
                    if (lines[j].1 - current_loop[0]).length() < 1e-4 {
                        loop_closed = true;
                    } else {
                        current_loop.push(lines[j].1);
                    }
                    found_next = true;
                    break;
                } else if (lines[j].1 - last_pt).length() < 1e-4 {
                    visited[j] = true;
                    if (lines[j].0 - current_loop[0]).length() < 1e-4 {
                        loop_closed = true;
                    } else {
                        current_loop.push(lines[j].0);
                    }
                    found_next = true;
                    break;
                }
            }

            if !found_next {
                break;
            }
        }

        if loop_closed && current_loop.len() >= 3 {
            loops.push(current_loop);
        }
    }

    loops
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sketch::workplane::Workplane;
    use core_math::vec2;

    #[test]
    fn test_extrude_sketch_rectangle() {
        let mut sketch = Sketch::new("BoxSketch", Workplane::xy());
        sketch.add_rectangle(vec2(0.0, 0.0), vec2(20.0, 10.0));

        let mesh = extrude_sketch(&sketch, 15.0).expect("Extrusion must succeed");
        assert_eq!(mesh.vertex_count(), 8);
        assert_eq!(mesh.face_count(), 6);
        assert_eq!(mesh.euler_characteristic(), 2);
    }
}

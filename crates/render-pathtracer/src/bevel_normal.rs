use crate::scene::PathTracerScene;
use core_math::Vec3;

/// Computes distance squared from point `p` to line segment `ab`.
#[inline]
pub fn dist_sq_point_segment(p: Vec3, a: Vec3, b: Vec3) -> f32 {
    let ab = b - a;
    let len_sq = ab.length_squared();
    if len_sq < 1e-8 {
        return (p - a).length_squared();
    }
    let t = ((p - a).dot(ab) / len_sq).clamp(0.0, 1.0);
    (p - (a + ab * t)).length_squared()
}

/// Computes minimum distance squared from point `p` to the perimeter edges of triangle `(v0, v1, v2)`.
#[inline]
pub fn dist_sq_point_triangle_edges(p: Vec3, v0: Vec3, v1: Vec3, v2: Vec3) -> f32 {
    let d0 = dist_sq_point_segment(p, v0, v1);
    let d1 = dist_sq_point_segment(p, v1, v2);
    let d2 = dist_sq_point_segment(p, v2, v0);
    d0.min(d1).min(d2)
}

/// Evaluates a micro-chamfer / rounded edge normal perturbation.
///
/// Simulates physical edge beveling and corner fillets at render time without adding geometry.
/// For points within `bevel_radius` of adjacent non-coplanar faces, the shading normal is
/// smoothly perturbed towards the bisector normal, creating pristine highlight chamfers.
pub fn compute_beveled_normal(
    scene: &PathTracerScene,
    hit_pos: Vec3,
    base_normal: Vec3,
    bevel_radius: f32,
) -> Vec3 {
    if bevel_radius <= 1e-5 {
        return base_normal;
    }

    let mut nearby_triangles = Vec::new();
    scene
        .bvh
        .query_sphere(hit_pos, bevel_radius, &mut nearby_triangles);

    if nearby_triangles.is_empty() {
        return base_normal;
    }

    let radius_sq = bevel_radius * bevel_radius;
    let mut accumulated_normal = base_normal;
    let mut total_weight = 1.0_f32;

    for tri in nearby_triangles {
        let edge1 = tri.v1 - tri.v0;
        let edge2 = tri.v2 - tri.v0;
        let tri_normal = edge1.cross(edge2).normalize_or_zero();

        // Check if adjacent face is non-coplanar (dot < 0.999) and not completely back-to-back (dot > -0.9)
        let dot = base_normal.dot(tri_normal);
        if dot < 0.999 && dot > -0.9 {
            let d_sq = dist_sq_point_triangle_edges(hit_pos, tri.v0, tri.v1, tri.v2);
            if d_sq < radius_sq {
                let dist = d_sq.sqrt();
                let factor = (1.0 - (dist / bevel_radius)).clamp(0.0, 1.0);
                // Hermite smoothstep for natural curvature roll-off
                let weight = factor * factor * (3.0 - 2.0 * factor);

                let bisector = (base_normal + tri_normal).normalize_or_zero();
                accumulated_normal += bisector * weight;
                total_weight += weight;
            }
        }
    }

    (accumulated_normal / total_weight).normalize_or_zero()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bvh::Triangle;
    use core_math::{vec2, vec3};

    #[test]
    fn test_point_segment_distance() {
        let a = vec3(0.0, 0.0, 0.0);
        let b = vec3(2.0, 0.0, 0.0);
        let p = vec3(1.0, 1.0, 0.0);
        let d_sq = dist_sq_point_segment(p, a, b);
        assert!((d_sq - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_micro_chamfer_normal_perturbation_corner() {
        // Create 2 orthogonal faces meeting at an edge along Z-axis: x=0
        // Top face in Y-plane: normal = (0, 1, 0), sharing edge (1, 1, 0)->(1, 1, 1)
        let t_top = Triangle::new(
            vec3(0.0, 1.0, 0.0),
            vec3(1.0, 1.0, 1.0),
            vec3(1.0, 1.0, 0.0),
            vec3(0.0, 1.0, 0.0),
            vec3(0.0, 1.0, 0.0),
            vec3(0.0, 1.0, 0.0),
            vec2(0.0, 0.0),
            vec2(1.0, 1.0),
            vec2(1.0, 0.0),
            0,
        );

        // Right face in X-plane: normal = (1, 0, 0), sharing edge (1, 1, 0)->(1, 1, 1)
        let t_right = Triangle::new(
            vec3(1.0, 0.0, 0.0),
            vec3(1.0, 1.0, 0.0),
            vec3(1.0, 1.0, 1.0),
            vec3(1.0, 0.0, 0.0),
            vec3(1.0, 0.0, 0.0),
            vec3(1.0, 0.0, 0.0),
            vec2(0.0, 0.0),
            vec2(0.0, 1.0),
            vec2(1.0, 1.0),
            0,
        );

        let scene = PathTracerScene::new(
            vec![t_top, t_right],
            vec![crate::Material::lambertian(vec3(0.8, 0.8, 0.8))],
        );

        // Point right near the edge seam at (0.98, 1.0, 0.5) with base_normal = (0, 1, 0)
        let hit_pos = vec3(0.98, 1.0, 0.5);
        let base_normal = vec3(0.0, 1.0, 0.0);
        let beveled = compute_beveled_normal(&scene, hit_pos, base_normal, 0.1);

        // The beveled normal must be pushed towards the (1, 1, 0) bisector (positive X component)
        assert!(
            beveled.x > 0.05,
            "Beveled normal X should bend towards adjacent face: {}",
            beveled.x
        );
        assert!(
            beveled.y > 0.5,
            "Beveled normal Y should remain strong: {}",
            beveled.y
        );

        // Far away from edge at (0.2, 1.0, 0.5), normal should remain untouched
        let far_pos = vec3(0.2, 1.0, 0.5);
        let untouched = compute_beveled_normal(&scene, far_pos, base_normal, 0.1);
        assert!((untouched - base_normal).length() < 1e-4);
    }
}

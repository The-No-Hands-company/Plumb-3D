//! Multi-Body CAD Assemblies, Kinematic Mates & Interference Clash Detection
//!
//! Provides the core mechanical CAD assembly module for Plumb 3D:
//! - [`mate`]: Kinematic mate connectors and joint types (Coincident, Concentric, Revolute, Prismatic, Cylindrical, Gear, Fixed).
//! - [`clash`]: Real-time 3D interference and clash detection (broadphase AABB, narrowphase triangle-triangle SAT).
//! - [`solver`]: Rigid assembly kinematics solver, DOF resolution, and joint parameter actuation.
//! - [`exploded_view`]: Exploded view sequence generator and 3D schematic trail lines.

pub mod clash;
pub mod exploded_view;
pub mod mate;
pub mod solver;

pub use clash::{
    check_mesh_clash, triangle_triangle_intersection, ClashResult, ClashType, CollisionMesh,
};
pub use exploded_view::{ExplodedStep, ExplodedTrailLine, ExplodedView};
pub use mate::{KinematicMate, MateAlignment, MateConnector, MateType};
pub use solver::{AssemblyBody, AssemblySolver};

#[cfg(test)]
mod tests {
    use super::*;
    use core_math::{Mat4, Transform, Vec3};
    use std::f32::consts::PI;

    #[test]
    fn test_mate_connector_orthogonality() {
        let origin = Vec3::new(10.0, 5.0, 0.0);
        let normal = Vec3::new(0.0, 0.0, 1.0);
        let tangent = Vec3::new(1.0, 0.0, 0.0);
        let conn = MateConnector::new(origin, normal, tangent);

        assert!((conn.normal.dot(conn.tangent)).abs() < 1e-5);
        assert!((conn.normal.dot(conn.binormal())).abs() < 1e-5);
        assert!((conn.tangent.dot(conn.binormal())).abs() < 1e-5);
        assert!((conn.normal.length() - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_fixed_mate_solver() {
        let mut solver = AssemblySolver::new();
        let body0 = AssemblyBody::grounded(0, "Base", Transform::IDENTITY);
        let body1 = AssemblyBody::new(1, "Bracket", Transform::from_translation(Vec3::new(50.0, 50.0, 50.0)));
        solver.add_body(body0);
        solver.add_body(body1);

        let conn0 = MateConnector::new(Vec3::new(10.0, 0.0, 0.0), Vec3::Z, Vec3::X);
        let conn1 = MateConnector::new(Vec3::ZERO, Vec3::Z, Vec3::X);
        let mut mate = KinematicMate::new(0, "FixedBracket", 0, conn0, 1, conn1, MateType::Fixed);
        mate.alignment = MateAlignment::Aligned;
        solver.add_mate(mate);

        solver.solve();

        let b1 = solver.get_body(1).unwrap();
        assert!((b1.transform.translation - Vec3::new(10.0, 0.0, 0.0)).length() < 1e-4);
        assert_eq!(b1.translation_dof, 0);
        assert_eq!(b1.rotation_dof, 0);
    }

    #[test]
    fn test_revolute_joint_actuation() {
        let mut solver = AssemblySolver::new();
        let body0 = AssemblyBody::grounded(0, "HingeBase", Transform::IDENTITY);
        let body1 = AssemblyBody::new(1, "HingeArm", Transform::IDENTITY);
        solver.add_body(body0);
        solver.add_body(body1);

        let conn0 = MateConnector::new(Vec3::new(0.0, 0.0, 5.0), Vec3::Z, Vec3::X);
        let conn1 = MateConnector::new(Vec3::ZERO, Vec3::Z, Vec3::X);
        let mut mate = KinematicMate::new(
            0,
            "RevoluteHinge",
            0,
            conn0,
            1,
            conn1,
            MateType::Revolute { limits: None },
        );
        mate.alignment = MateAlignment::Aligned;
        solver.add_mate(mate);

        // Actuate 90 degrees (PI/2 radians)
        solver.actuate_mate(0, PI * 0.5);

        let b1 = solver.get_body(1).unwrap();
        assert_eq!(b1.translation_dof, 0);
        assert_eq!(b1.rotation_dof, 1);
        assert!((b1.transform.translation - Vec3::new(0.0, 0.0, 5.0)).length() < 1e-4);

        // A test point on local X (1, 0, 0) should now rotate to (0, 1, 0) in world space
        let local_arm_pt = Vec3::new(1.0, 0.0, 0.0);
        let world_pt = b1.transform.transform_point(local_arm_pt);
        assert!((world_pt.x - 0.0).abs() < 1e-4);
        assert!((world_pt.y - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_prismatic_joint_sliding() {
        let mut solver = AssemblySolver::new();
        let body0 = AssemblyBody::grounded(0, "RailBase", Transform::IDENTITY);
        let body1 = AssemblyBody::new(1, "SliderCarriage", Transform::IDENTITY);
        solver.add_body(body0);
        solver.add_body(body1);

        let conn0 = MateConnector::new(Vec3::ZERO, Vec3::Z, Vec3::X);
        let conn1 = MateConnector::new(Vec3::ZERO, Vec3::Z, Vec3::X);
        let mut mate = KinematicMate::new(
            0,
            "Slider",
            0,
            conn0,
            1,
            conn1,
            MateType::Prismatic { limits: Some((-50.0, 50.0)) },
        );
        mate.alignment = MateAlignment::Aligned;
        solver.add_mate(mate);

        // Actuate carriage by 25.0 mm along X tangent
        solver.actuate_mate(0, 25.0);

        let b1 = solver.get_body(1).unwrap();
        assert_eq!(b1.translation_dof, 1);
        assert_eq!(b1.rotation_dof, 0);
        assert!((b1.transform.translation.x - 25.0).abs() < 1e-4);
    }

    fn make_test_cube_mesh(size: f32) -> CollisionMesh {
        let h = size * 0.5;
        let vertices = vec![
            Vec3::new(-h, -h, -h), // 0
            Vec3::new(h, -h, -h),  // 1
            Vec3::new(h, h, -h),   // 2
            Vec3::new(-h, h, -h),  // 3
            Vec3::new(-h, -h, h),  // 4
            Vec3::new(h, -h, h),   // 5
            Vec3::new(h, h, h),    // 6
            Vec3::new(-h, h, h),   // 7
        ];
        let indices = vec![
            // Front (+Z)
            [4, 5, 6], [4, 6, 7],
            // Back (-Z)
            [1, 0, 3], [1, 3, 2],
            // Top (+Y)
            [3, 2, 6], [3, 6, 7],
            // Bottom (-Y)
            [0, 1, 5], [0, 5, 4],
            // Right (+X)
            [5, 1, 2], [5, 2, 6],
            // Left (-X)
            [0, 4, 7], [0, 7, 3],
        ];
        CollisionMesh::from_raw(vertices, indices)
    }

    #[test]
    fn test_clash_detection_interference_vs_clearance() {
        let cube1 = make_test_cube_mesh(2.0);
        let cube2 = make_test_cube_mesh(2.0);

        // Case 1: Overlapping cubes (interpenetration along X)
        let tf1 = Mat4::from_translation(Vec3::new(0.0, 0.0, 0.0));
        let tf2 = Mat4::from_translation(Vec3::new(1.0, 0.0, 0.0)); // overlapping by 1.0 unit

        let clash_overlap = check_mesh_clash(0, &cube1, &tf1, 1, &cube2, &tf2, 0.01);
        assert!(clash_overlap.is_interfering(), "Cubes with centers at 0 and 1 must interfere");
        assert!(clash_overlap.penetration_depth() > 0.0);

        // Case 2: Separated cubes with clear distance
        let tf3 = Mat4::from_translation(Vec3::new(10.0, 0.0, 0.0));
        let clash_clear = check_mesh_clash(0, &cube1, &tf1, 2, &cube2, &tf3, 0.01);
        assert!(!clash_clear.is_interfering(), "Cubes at 0 and 10 must not interfere");
        match clash_clear.clash_type {
            ClashType::Clearance { min_distance } => {
                assert!(min_distance > 7.0, "Expected clearance > 7.0, got {}", min_distance);
            }
            _ => panic!("Expected clearance"),
        }
    }

    #[test]
    fn test_exploded_view_interpolation_and_trails() {
        let mut exp = ExplodedView::new("AssemblyExplosion");
        exp.add_step(ExplodedStep::new(0, "ExplodeArm", vec![1], Vec3::X, 50.0));
        exp.add_step(ExplodedStep::new(1, "ExplodePin", vec![2], Vec3::Z, 30.0));

        let assembled = vec![
            (0, Transform::IDENTITY),
            (1, Transform::from_translation(Vec3::new(10.0, 0.0, 0.0))),
            (2, Transform::from_translation(Vec3::new(0.0, 0.0, 5.0))),
        ];

        // At 0% explosion: no motion, no trail lines
        exp.set_factor(0.0);
        let (tf_0, trails_0) = exp.evaluate(&assembled);
        assert_eq!(trails_0.len(), 0);
        assert!((tf_0[&1].translation - Vec3::new(10.0, 0.0, 0.0)).length() < 1e-4);

        // At 100% explosion: full motion and trail lines generated
        exp.set_factor(1.0);
        let (tf_100, trails_100) = exp.evaluate(&assembled);
        assert_eq!(trails_100.len(), 2);
        assert!((tf_100[&1].translation - Vec3::new(60.0, 0.0, 0.0)).length() < 1e-4);
        assert!((tf_100[&2].translation - Vec3::new(0.0, 0.0, 35.0)).length() < 1e-4);
    }
}

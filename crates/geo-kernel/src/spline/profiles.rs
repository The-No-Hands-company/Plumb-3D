use std::f32::consts::PI;
use core_math::{vec2, Vec2};

use super::types::Spline;

/// 2D cross-section profiles for sweeping/extruding along 3D splines (3ds Max Loft / Blender Curve Profile)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ProfileShape {
    #[default]
    Circle,
    Square,
    Rectangle,
    Star,
    Triangle,
    Hexagon,
}

impl ProfileShape {
    pub fn generate_cross_section(&self, radius: f32, subdivisions: usize) -> Vec<Vec2> {
        match self {
            ProfileShape::Circle => Spline::circle_cross_section(radius, subdivisions),
            ProfileShape::Square => {
                let r = radius;
                vec![
                    vec2(-r, -r),
                    vec2(r, -r),
                    vec2(r, r),
                    vec2(-r, r),
                ]
            }
            ProfileShape::Rectangle => {
                let rx = radius;
                let ry = radius * 0.5;
                vec![
                    vec2(-rx, -ry),
                    vec2(rx, -ry),
                    vec2(rx, ry),
                    vec2(-rx, ry),
                ]
            }
            ProfileShape::Triangle => {
                (0..3)
                    .map(|i| {
                        let a = (i as f32 / 3.0) * 2.0 * PI - PI * 0.5;
                        vec2(radius * a.cos(), radius * a.sin())
                    })
                    .collect()
            }
            ProfileShape::Hexagon => {
                (0..6)
                    .map(|i| {
                        let a = (i as f32 / 6.0) * 2.0 * PI;
                        vec2(radius * a.cos(), radius * a.sin())
                    })
                    .collect()
            }
            ProfileShape::Star => {
                let points = 5;
                let r_inner = radius * 0.45;
                let r_outer = radius;
                let total = points * 2;
                (0..total)
                    .map(|i| {
                        let a = (i as f32 / total as f32) * 2.0 * PI - PI * 0.5;
                        let r = if i % 2 == 0 { r_outer } else { r_inner };
                        vec2(r * a.cos(), r * a.sin())
                    })
                    .collect()
            }
        }
    }
}

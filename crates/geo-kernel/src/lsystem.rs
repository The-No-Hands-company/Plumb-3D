use crate::mesh::{Mesh, MeshError, VertexKey};
use core_math::{vec2, vec3, Quat, Vec3};
use std::collections::HashMap;
use std::f32::consts::PI;

/// Turtle graphics state for 3D L-System branch growth
#[derive(Debug, Clone)]
struct TurtleState {
    position: Vec3,
    heading: Vec3,
    up: Vec3,
    right: Vec3,
    length: f32,
    radius: f32,
    depth: usize,
    last_ring: Option<Vec<VertexKey>>,
}

/// Popular natural and biomechanical growth presets for procedural generation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LSystemPreset {
    OakTree,
    FernPlant,
    IvyVines,
    SciFiBiomechanicalConduit,
}

/// Procedural 3D Lindenmayer System (L-System) grammar and vegetation generator (SpeedTree / Houdini style)
#[derive(Debug, Clone)]
pub struct LSystem {
    pub axiom: String,
    pub rules: HashMap<char, String>,
    pub iterations: usize,
    pub angle_deg: f32,
    pub initial_length: f32,
    pub length_decay: f32,
    pub initial_radius: f32,
    pub radius_decay: f32,
    pub radial_segments: usize,
    pub phototropism: Vec3,
    pub gravity_droop: f32,
    pub generate_leaves: bool,
}

impl Default for LSystem {
    fn default() -> Self {
        Self::from_preset(LSystemPreset::OakTree)
    }
}

impl LSystem {
    /// Constructs an L-System configured for a specific natural or procedural style
    pub fn from_preset(preset: LSystemPreset) -> Self {
        let mut rules = HashMap::new();
        match preset {
            LSystemPreset::OakTree => {
                rules.insert('X', "F[+&X][/&-X][\\^X]FX".into());
                rules.insert('F', "FF".into());
                Self {
                    axiom: "X".into(),
                    rules,
                    iterations: 3,
                    angle_deg: 26.5,
                    initial_length: 0.8,
                    length_decay: 0.72,
                    initial_radius: 0.12,
                    radius_decay: 0.68,
                    radial_segments: 6,
                    phototropism: vec3(0.0, 0.4, 0.0),
                    gravity_droop: 0.05,
                    generate_leaves: true,
                }
            }
            LSystemPreset::FernPlant => {
                rules.insert('X', "F[+X]F[-X]+X".into());
                rules.insert('F', "FF".into());
                Self {
                    axiom: "X".into(),
                    rules,
                    iterations: 4,
                    angle_deg: 25.0,
                    initial_length: 0.35,
                    length_decay: 0.80,
                    initial_radius: 0.04,
                    radius_decay: 0.82,
                    radial_segments: 5,
                    phototropism: vec3(0.0, 0.2, 0.0),
                    gravity_droop: 0.08,
                    generate_leaves: false,
                }
            }
            LSystemPreset::IvyVines => {
                rules.insert('F', "F[+F]F[-F][&F]".into());
                Self {
                    axiom: "F".into(),
                    rules,
                    iterations: 3,
                    angle_deg: 32.0,
                    initial_length: 0.5,
                    length_decay: 0.75,
                    initial_radius: 0.05,
                    radius_decay: 0.75,
                    radial_segments: 5,
                    phototropism: vec3(0.1, 0.1, 0.0),
                    gravity_droop: 0.12,
                    generate_leaves: true,
                }
            }
            LSystemPreset::SciFiBiomechanicalConduit => {
                rules.insert('F', "FF+[+F-F]-[-F+F]".into());
                Self {
                    axiom: "F".into(),
                    rules,
                    iterations: 3,
                    angle_deg: 45.0,
                    initial_length: 0.7,
                    length_decay: 0.70,
                    initial_radius: 0.10,
                    radius_decay: 0.70,
                    radial_segments: 8,
                    phototropism: Vec3::ZERO,
                    gravity_droop: 0.0,
                    generate_leaves: false,
                }
            }
        }
    }

    /// Recursively expands the grammar string according to production rules
    pub fn expand_grammar(&self) -> String {
        let mut current = self.axiom.clone();
        for _ in 0..self.iterations {
            let mut next = String::with_capacity(current.len() * 2);
            for c in current.chars() {
                if let Some(replacement) = self.rules.get(&c) {
                    next.push_str(replacement);
                } else {
                    next.push(c);
                }
            }
            current = next;
        }
        current
    }

    /// Generates a manifold BMesh 3D geometry representing the grown structure
    pub fn generate_mesh(&self) -> Result<Mesh, MeshError> {
        let grammar = self.expand_grammar();
        let mut mesh = Mesh::new();
        let rad_angle = self.angle_deg * (PI / 180.0);

        let mut turtle = TurtleState {
            position: Vec3::ZERO,
            heading: vec3(0.0, 1.0, 0.0),
            up: vec3(0.0, 0.0, 1.0),
            right: vec3(1.0, 0.0, 0.0),
            length: self.initial_length,
            radius: self.initial_radius,
            depth: 0,
            last_ring: None,
        };

        let mut stack: Vec<TurtleState> = Vec::new();

        // Create base root ring
        let base_ring = create_ring(&mut mesh, turtle.position, turtle.heading, turtle.radius, self.radial_segments);
        turtle.last_ring = Some(base_ring);

        for ch in grammar.chars() {
            match ch {
                'F' => {
                    // Apply environmental forces (phototropism and gravity)
                    let force = self.phototropism + vec3(0.0, -self.gravity_droop, 0.0);
                    turtle.heading = (turtle.heading + force * 0.3).normalize();
                    turtle.right = turtle.heading.cross(turtle.up).normalize();
                    turtle.up = turtle.right.cross(turtle.heading).normalize();

                    let next_pos = turtle.position + turtle.heading * turtle.length;
                    let next_radius = (turtle.radius * self.radius_decay).max(0.005);
                    let next_ring = create_ring(&mut mesh, next_pos, turtle.heading, next_radius, self.radial_segments);

                    // Connect rings with quad strip
                    if let Some(ref prev_ring) = turtle.last_ring {
                        bridge_rings(&mut mesh, prev_ring, &next_ring);
                    }

                    turtle.position = next_pos;
                    turtle.radius = next_radius;
                    turtle.length *= self.length_decay;
                    turtle.last_ring = Some(next_ring);
                }
                '+' => {
                    let axis = turtle.up;
                    rotate_turtle(&mut turtle, axis, rad_angle);
                }
                '-' => {
                    let axis = turtle.up;
                    rotate_turtle(&mut turtle, axis, -rad_angle);
                }
                '&' => {
                    let axis = turtle.right;
                    rotate_turtle(&mut turtle, axis, rad_angle);
                }
                '^' => {
                    let axis = turtle.right;
                    rotate_turtle(&mut turtle, axis, -rad_angle);
                }
                '\\' => {
                    let axis = turtle.heading;
                    rotate_turtle(&mut turtle, axis, rad_angle);
                }
                '/' => {
                    let axis = turtle.heading;
                    rotate_turtle(&mut turtle, axis, -rad_angle);
                }
                '|' => {
                    let axis = turtle.up;
                    rotate_turtle(&mut turtle, axis, PI);
                }
                '[' => {
                    stack.push(turtle.clone());
                    turtle.depth += 1;
                    turtle.length *= self.length_decay;
                    turtle.radius *= self.radius_decay;
                }
                ']' => {
                    if let Some(popped) = stack.pop() {
                        turtle = popped;
                    }
                }
                'L' if self.generate_leaves => {
                    create_leaf(&mut mesh, turtle.position, turtle.heading, turtle.radius * 2.5);
                }
                _ => {}
            }
        }

        mesh.update_normals();
        Ok(mesh)
    }
}

fn rotate_turtle(turtle: &mut TurtleState, axis: Vec3, angle: f32) {
    let rot = Quat::from_axis_angle(axis.normalize(), angle);
    turtle.heading = rot.mul_vec3(turtle.heading).normalize();
    turtle.up = rot.mul_vec3(turtle.up).normalize();
    turtle.right = turtle.heading.cross(turtle.up).normalize();
}

fn create_ring(mesh: &mut Mesh, center: Vec3, normal: Vec3, radius: f32, segments: usize) -> Vec<VertexKey> {
    let n = normal.normalize();
    let arbitrary = if n.y.abs() < 0.9 { vec3(0.0, 1.0, 0.0) } else { vec3(1.0, 0.0, 0.0) };
    let t = n.cross(arbitrary).normalize();
    let b = n.cross(t).normalize();

    let mut keys = Vec::with_capacity(segments);
    for i in 0..segments {
        let theta = 2.0 * PI * (i as f32) / segments as f32;
        let pos = center + (t * theta.cos() + b * theta.sin()) * radius;
        let uv = vec2((i as f32) / segments as f32, center.y);
        keys.push(mesh.add_vertex_with_uv(pos, uv));
    }
    keys
}

fn bridge_rings(mesh: &mut Mesh, r0: &[VertexKey], r1: &[VertexKey]) {
    let count = r0.len().min(r1.len());
    for i in 0..count {
        let next = (i + 1) % count;
        let v0 = r0[i];
        let v1 = r0[next];
        let v2 = r1[next];
        let v3 = r1[i];
        let _ = mesh.add_face(&[v0, v1, v2, v3]);
    }
}

fn create_leaf(mesh: &mut Mesh, tip: Vec3, dir: Vec3, size: f32) {
    let up = vec3(0.0, 1.0, 0.0);
    let right = dir.cross(up).normalize() * (size * 0.5);
    let forward = dir.normalize() * size;

    let v0 = mesh.add_vertex(tip);
    let v1 = mesh.add_vertex(tip + right + forward * 0.5);
    let v2 = mesh.add_vertex(tip + forward);
    let v3 = mesh.add_vertex(tip - right + forward * 0.5);

    let _ = mesh.add_face(&[v0, v1, v2, v3]);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lsystem_grammar_expansion() {
        let lsystem = LSystem::from_preset(LSystemPreset::OakTree);
        let expanded = lsystem.expand_grammar();
        assert!(expanded.len() > lsystem.axiom.len());
        assert!(expanded.contains('F'));
        assert!(expanded.contains('['));
        assert!(expanded.contains(']'));
    }

    #[test]
    fn test_lsystem_tree_mesh_generation() {
        let lsystem = LSystem::from_preset(LSystemPreset::OakTree);
        let mesh = lsystem.generate_mesh().expect("Generate LSystem tree mesh");

        assert!(!mesh.vertices.is_empty());
        assert!(!mesh.faces.is_empty());
        let aabb = mesh.compute_aabb();
        assert!(aabb.max.y > 0.5); // Tree grows upwards
    }

    #[test]
    fn test_lsystem_presets() {
        for preset in [
            LSystemPreset::OakTree,
            LSystemPreset::FernPlant,
            LSystemPreset::IvyVines,
            LSystemPreset::SciFiBiomechanicalConduit,
        ] {
            let lsystem = LSystem::from_preset(preset);
            let mesh = lsystem.generate_mesh().expect("Generate preset mesh");
            assert!(mesh.vertices.len() >= 6);
            assert!(mesh.faces.len() >= 4);
        }
    }
}

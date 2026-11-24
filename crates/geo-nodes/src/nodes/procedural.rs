use crate::nodes::traits::GeoNodeOp;
use crate::types::*;
use core_math::Vec3;

/// 3D Perlin / Simplex fractal gradient noise generator.
#[derive(Debug, Clone)]
pub struct NoiseTextureNode;

impl GeoNodeOp for NoiseTextureNode {
    fn name(&self) -> &str {
        "Noise Texture"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::Math
    }

    fn inputs(&self) -> Vec<NodeSocket> {
        vec![
            NodeSocket::vector("Vector", Vec3::ZERO),
            NodeSocket::float("Scale", 5.0),
            NodeSocket::int("Detail", 2),
            NodeSocket::float("Roughness", 0.5),
        ]
    }

    fn outputs(&self) -> Vec<NodeSocket> {
        vec![
            NodeSocket::float("Fac", 0.0),
            NodeSocket::vector("Color", Vec3::splat(0.5)),
        ]
    }

    fn execute(&self, inputs: &[SocketValue]) -> Result<Vec<SocketValue>, NodeGraphError> {
        let pos = inputs.first().map(|v| v.as_vec3()).unwrap_or(Vec3::ZERO);
        let scale = inputs.get(1).map(|v| v.as_float()).unwrap_or(5.0).max(0.001);
        let detail = inputs.get(2).map(|v| v.as_int()).unwrap_or(2).clamp(1, 6) as usize;
        let roughness = inputs.get(3).map(|v| v.as_float()).unwrap_or(0.5).clamp(0.0, 1.0);

        let noise_val = fbm_noise_3d(pos * scale, detail, roughness);
        let col = Vec3::new(
            fbm_noise_3d(pos * scale + Vec3::new(17.1, 4.3, 8.5), detail, roughness),
            fbm_noise_3d(pos * scale + Vec3::new(31.7, 12.9, 19.4), detail, roughness),
            noise_val,
        );

        Ok(vec![
            SocketValue::Float(noise_val),
            SocketValue::Vector3(col),
        ])
    }

    fn clone_box(&self) -> Box<dyn GeoNodeOp> {
        Box::new(self.clone())
    }
}

/// Displaces mesh surface vertices along normals using a scalar or vector field.
#[derive(Debug, Clone)]
pub struct DisplaceGeometryNode;

impl GeoNodeOp for DisplaceGeometryNode {
    fn name(&self) -> &str {
        "Displace Geometry"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::GeometryOps
    }

    fn inputs(&self) -> Vec<NodeSocket> {
        vec![
            NodeSocket::geometry("Geometry"),
            NodeSocket::float("Displacement", 0.2),
            NodeSocket::float("Strength", 1.0),
            NodeSocket::float("Midlevel", 0.0),
        ]
    }

    fn outputs(&self) -> Vec<NodeSocket> {
        vec![NodeSocket::geometry("Geometry")]
    }

    fn execute(&self, inputs: &[SocketValue]) -> Result<Vec<SocketValue>, NodeGraphError> {
        let Some(mut mesh) = inputs.first().and_then(|v| v.as_geometry()).cloned() else {
            return Ok(vec![SocketValue::Geometry(None)]);
        };

        let disp = inputs.get(1).map(|v| v.as_float()).unwrap_or(0.2);
        let strength = inputs.get(2).map(|v| v.as_float()).unwrap_or(1.0);
        let midlevel = inputs.get(3).map(|v| v.as_float()).unwrap_or(0.0);

        let factor = (disp - midlevel) * strength;

        for v in mesh.vertices.values_mut() {
            v.position += v.normal * factor;
        }

        mesh.update_normals();
        Ok(vec![SocketValue::Geometry(Some(mesh))])
    }

    fn clone_box(&self) -> Box<dyn GeoNodeOp> {
        Box::new(self.clone())
    }
}

/// Entry socket node representing the base geometry fed into a procedural modifier graph.
#[derive(Debug, Clone)]
pub struct GroupInputNode {
    pub input_mesh: Option<geo_kernel::Mesh>,
}

impl Default for GroupInputNode {
    fn default() -> Self {
        Self { input_mesh: None }
    }
}

impl GeoNodeOp for GroupInputNode {
    fn name(&self) -> &str {
        "Group Input"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::Inputs
    }

    fn inputs(&self) -> Vec<NodeSocket> {
        vec![]
    }

    fn outputs(&self) -> Vec<NodeSocket> {
        vec![NodeSocket::geometry("Geometry")]
    }

    fn execute(&self, _inputs: &[SocketValue]) -> Result<Vec<SocketValue>, NodeGraphError> {
        Ok(vec![SocketValue::Geometry(self.input_mesh.clone())])
    }

    fn as_any_mut(&mut self) -> Option<&mut dyn std::any::Any> {
        Some(self)
    }

    fn clone_box(&self) -> Box<dyn GeoNodeOp> {
        Box::new(self.clone())
    }
}

/// Fractal Brownian Motion 3D Perlin Gradient Noise
fn fbm_noise_3d(pos: Vec3, octaves: usize, roughness: f32) -> f32 {
    let mut total = 0.0;
    let mut frequency = 1.0;
    let mut amplitude = 1.0;
    let mut max_val = 0.0;

    for _ in 0..octaves {
        total += perlin_noise_3d(pos * frequency) * amplitude;
        max_val += amplitude;
        amplitude *= roughness;
        frequency *= 2.0;
    }

    (total / max_val.max(0.001)).clamp(0.0, 1.0)
}

/// Deterministic 3D gradient hash noise
fn perlin_noise_3d(p: Vec3) -> f32 {
    let xi = p.x.floor() as i32;
    let yi = p.y.floor() as i32;
    let zi = p.z.floor() as i32;

    let xf = p.x - p.x.floor();
    let yf = p.y - p.y.floor();
    let zf = p.z - p.z.floor();

    // Quintic Hermite interpolant: 6t^5 - 15t^4 + 10t^3
    let u = xf * xf * xf * (xf * (xf * 6.0 - 15.0) + 10.0);
    let v = yf * yf * yf * (yf * (yf * 6.0 - 15.0) + 10.0);
    let w = zf * zf * zf * (zf * (zf * 6.0 - 15.0) + 10.0);

    let h000 = grad_dot(xi, yi, zi, xf, yf, zf);
    let h100 = grad_dot(xi + 1, yi, zi, xf - 1.0, yf, zf);
    let h010 = grad_dot(xi, yi + 1, zi, xf, yf - 1.0, zf);
    let h110 = grad_dot(xi + 1, yi + 1, zi, xf - 1.0, yf - 1.0, zf);

    let h001 = grad_dot(xi, yi, zi + 1, xf, yf, zf - 1.0);
    let h101 = grad_dot(xi + 1, yi, zi + 1, xf - 1.0, yf, zf - 1.0);
    let h011 = grad_dot(xi, yi + 1, zi + 1, xf, yf - 1.0, zf - 1.0);
    let h111 = grad_dot(xi + 1, yi + 1, zi + 1, xf - 1.0, yf - 1.0, zf - 1.0);

    let x00 = lerp(h000, h100, u);
    let x10 = lerp(h010, h110, u);
    let x01 = lerp(h001, h101, u);
    let x11 = lerp(h011, h111, u);

    let y0 = lerp(x00, x10, v);
    let y1 = lerp(x01, x11, v);

    let z = lerp(y0, y1, w);
    (z * 0.5 + 0.5).clamp(0.0, 1.0)
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + t * (b - a)
}

fn grad_dot(ix: i32, iy: i32, iz: i32, fx: f32, fy: f32, fz: f32) -> f32 {
    let mut h = (ix as u32).wrapping_mul(374761393)
        ^ (iy as u32).wrapping_mul(668265263)
        ^ (iz as u32).wrapping_mul(921473957);
    h = h.wrapping_mul(1274126177);
    h = h ^ (h >> 16);

    let gx = match h & 3 {
        0 => 1.0,
        1 => -1.0,
        _ => 0.0,
    };
    let gy = match (h >> 2) & 3 {
        0 => 1.0,
        1 => -1.0,
        _ => 0.0,
    };
    let gz = match (h >> 4) & 3 {
        0 => 1.0,
        1 => -1.0,
        _ => 0.0,
    };

    gx * fx + gy * fy + gz * fz
}

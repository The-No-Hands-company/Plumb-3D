use crate::nodes::traits::GeoNodeOp;
use crate::types::*;
use core_math::{Quat, Vec3};
use geo_kernel::Mesh;

/// Distributes 3D point instances across mesh polygon surfaces.
#[derive(Debug, Clone)]
pub struct DistributePointsOnFacesNode;

impl GeoNodeOp for DistributePointsOnFacesNode {
    fn name(&self) -> &str {
        "Distribute Points on Faces"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::PointOps
    }

    fn inputs(&self) -> Vec<NodeSocket> {
        vec![
            NodeSocket::geometry("Geometry"),
            NodeSocket::float("Density", 2.0),
            NodeSocket::int("Seed", 42),
        ]
    }

    fn outputs(&self) -> Vec<NodeSocket> {
        vec![NodeSocket::points("Points")]
    }

    fn execute(&self, inputs: &[SocketValue]) -> Result<Vec<SocketValue>, NodeGraphError> {
        let Some(mesh) = inputs.first().and_then(|v| v.as_geometry()) else {
            return Ok(vec![SocketValue::Points(Vec::new())]);
        };

        let density = inputs.get(1).map(|v| v.as_float()).unwrap_or(2.0).max(0.01);
        let seed = inputs.get(2).map(|v| v.as_int()).unwrap_or(42) as u64;

        let mut points = Vec::new();
        let mut rng = SimpleLcgRng::new(seed);

        for fk in mesh.faces.keys() {
            let v_keys = mesh.face_vertices(fk);
            if v_keys.len() < 3 {
                continue;
            }

            let normal = mesh.faces[fk].normal;
            let centroid = mesh.face_centroid(fk);

            // Compute face area using fan triangulation
            let mut face_area = 0.0_f32;
            let v0 = mesh.vertices[v_keys[0]].position;
            for i in 1..(v_keys.len() - 1) {
                let v1 = mesh.vertices[v_keys[i]].position;
                let v2 = mesh.vertices[v_keys[i + 1]].position;
                face_area += ((v1 - v0).cross(v2 - v0)).length() * 0.5;
            }

            let target_point_count = (face_area * density).round() as usize;
            let count = target_point_count.max(1).min(50);

            for _ in 0..count {
                // Sample point on face using random jitter around centroid
                let jitter_x = (rng.next_f32() - 0.5) * 0.8;
                let jitter_z = (rng.next_f32() - 0.5) * 0.8;
                let tangent = if normal.y.abs() < 0.99 {
                    normal.cross(Vec3::Y).normalize()
                } else {
                    normal.cross(Vec3::X).normalize()
                };
                let bitangent = normal.cross(tangent).normalize();

                let pos = centroid + tangent * (jitter_x * face_area.sqrt()) + bitangent * (jitter_z * face_area.sqrt());
                points.push(PointInstance::new(pos, normal));
            }
        }

        Ok(vec![SocketValue::Points(points)])
    }

    fn clone_box(&self) -> Box<dyn GeoNodeOp> {
        Box::new(self.clone())
    }
}

/// Instantiates copies of geometry onto scattered point locations.
#[derive(Debug, Clone)]
pub struct InstanceOnPointsNode;

impl GeoNodeOp for InstanceOnPointsNode {
    fn name(&self) -> &str {
        "Instance on Points"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::PointOps
    }

    fn inputs(&self) -> Vec<NodeSocket> {
        vec![
            NodeSocket::points("Points"),
            NodeSocket::geometry("Instance"),
            NodeSocket::vector("Scale", Vec3::ONE),
        ]
    }

    fn outputs(&self) -> Vec<NodeSocket> {
        vec![NodeSocket::geometry("Geometry")]
    }

    fn execute(&self, inputs: &[SocketValue]) -> Result<Vec<SocketValue>, NodeGraphError> {
        let points = inputs.first().and_then(|v| v.as_points()).unwrap_or(&[]);
        let Some(instance_mesh) = inputs.get(1).and_then(|v| v.as_geometry()) else {
            return Ok(vec![SocketValue::Geometry(None)]);
        };

        let user_scale = inputs.get(2).map(|v| v.as_vec3()).unwrap_or(Vec3::ONE);

        let mut combined_mesh = Mesh::new();

        for pt in points {
            let scale = pt.scale * user_scale;
            // Compute rotation aligning +Y to point normal
            let rot = Quat::from_rotation_arc(Vec3::Y, pt.normal);
            let transform = core_math::Mat4::from_scale_rotation_translation(scale, rot, pt.position);

            let _ = combined_mesh.join(instance_mesh, transform);
        }

        combined_mesh.update_normals();
        Ok(vec![SocketValue::Geometry(Some(combined_mesh))])
    }

    fn clone_box(&self) -> Box<dyn GeoNodeOp> {
        Box::new(self.clone())
    }
}

/// Minimal deterministic Linear Congruential Generator for reproducible procedural sampling
struct SimpleLcgRng {
    state: u64,
}

impl SimpleLcgRng {
    fn new(seed: u64) -> Self {
        Self {
            state: seed.max(1),
        }
    }

    fn next_u32(&mut self) -> u32 {
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (self.state >> 32) as u32
    }

    fn next_f32(&mut self) -> f32 {
        (self.next_u32() as f32) / (u32::MAX as f32)
    }
}

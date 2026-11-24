use crate::nodes::traits::GeoNodeOp;
use crate::types::*;
use core_math::{EulerRot, Quat, Transform, Vec3};
use geo_kernel::{decimate_mesh, BooleanOp, CatmullClarkModifier, Modifier, SolidifyModifier};

#[derive(Debug, Clone)]
pub struct TransformGeometryNode;

impl GeoNodeOp for TransformGeometryNode {
    fn name(&self) -> &str {
        "Transform Geometry"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::GeometryOps
    }

    fn inputs(&self) -> Vec<NodeSocket> {
        vec![
            NodeSocket::geometry("Geometry"),
            NodeSocket::vector("Translation", Vec3::ZERO),
            NodeSocket::vector("Rotation", Vec3::ZERO),
            NodeSocket::vector("Scale", Vec3::ONE),
        ]
    }

    fn outputs(&self) -> Vec<NodeSocket> {
        vec![NodeSocket::geometry("Geometry")]
    }

    fn execute(&self, inputs: &[SocketValue]) -> Result<Vec<SocketValue>, NodeGraphError> {
        let Some(mut mesh) = inputs.get(0).and_then(|v| v.as_geometry()).cloned() else {
            return Ok(vec![SocketValue::Geometry(None)]);
        };

        let trans = inputs.get(1).map(|v| v.as_vec3()).unwrap_or(Vec3::ZERO);
        let rot_deg = inputs.get(2).map(|v| v.as_vec3()).unwrap_or(Vec3::ZERO);
        let scale = inputs.get(3).map(|v| v.as_vec3()).unwrap_or(Vec3::ONE);

        let rot = Quat::from_euler(
            EulerRot::YXZ,
            rot_deg.y.to_radians(),
            rot_deg.x.to_radians(),
            rot_deg.z.to_radians(),
        );

        let transform = Transform {
            translation: trans,
            rotation: rot,
            scale,
        };
        let mat = transform.to_matrix();

        for v in mesh.vertices.values_mut() {
            v.position = mat.transform_point3(v.position);
        }
        mesh.update_normals();

        Ok(vec![SocketValue::Geometry(Some(mesh))])
    }

    fn clone_box(&self) -> Box<dyn GeoNodeOp> {
        Box::new(self.clone())
    }
}


#[derive(Debug, Clone)]
pub struct SubdivideNode;

impl GeoNodeOp for SubdivideNode {
    fn name(&self) -> &str {
        "Subdivide"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::GeometryOps
    }

    fn inputs(&self) -> Vec<NodeSocket> {
        vec![
            NodeSocket::geometry("Geometry"),
            NodeSocket::int("Level", 1),
        ]
    }

    fn outputs(&self) -> Vec<NodeSocket> {
        vec![NodeSocket::geometry("Geometry")]
    }

    fn execute(&self, inputs: &[SocketValue]) -> Result<Vec<SocketValue>, NodeGraphError> {
        let Some(mesh) = inputs.get(0).and_then(|v| v.as_geometry()).cloned() else {
            return Ok(vec![SocketValue::Geometry(None)]);
        };

        let levels = inputs.get(1).map(|v| v.as_int()).unwrap_or(1).clamp(1, 4) as u32;
        let modifier = CatmullClarkModifier::new(levels);
        let mesh = modifier.apply(&mesh).map_err(|e| NodeGraphError::ExecutionError(e.to_string()))?;

        Ok(vec![SocketValue::Geometry(Some(mesh))])
    }

    fn clone_box(&self) -> Box<dyn GeoNodeOp> {
        Box::new(self.clone())
    }
}


#[derive(Debug, Clone)]
pub struct ExtrudeNode;

impl GeoNodeOp for ExtrudeNode {
    fn name(&self) -> &str {
        "Extrude Mesh"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::GeometryOps
    }

    fn inputs(&self) -> Vec<NodeSocket> {
        vec![
            NodeSocket::geometry("Geometry"),
            NodeSocket::float("Offset", 0.5),
        ]
    }

    fn outputs(&self) -> Vec<NodeSocket> {
        vec![NodeSocket::geometry("Geometry")]
    }

    fn execute(&self, inputs: &[SocketValue]) -> Result<Vec<SocketValue>, NodeGraphError> {
        let Some(mut mesh) = inputs.get(0).and_then(|v| v.as_geometry()).cloned() else {
            return Ok(vec![SocketValue::Geometry(None)]);
        };

        let offset = inputs.get(1).map(|v| v.as_float()).unwrap_or(0.5);
        let faces: Vec<_> = mesh.faces.keys().collect();

        for fk in faces {
            let _ = mesh.extrude_face(fk, offset);
        }
        mesh.update_normals();

        Ok(vec![SocketValue::Geometry(Some(mesh))])
    }

    fn clone_box(&self) -> Box<dyn GeoNodeOp> {
        Box::new(self.clone())
    }
}


#[derive(Debug, Clone)]
pub struct BooleanNode;

impl GeoNodeOp for BooleanNode {
    fn name(&self) -> &str {
        "Boolean"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::GeometryOps
    }

    fn inputs(&self) -> Vec<NodeSocket> {
        vec![
            NodeSocket::geometry("Mesh A"),
            NodeSocket::geometry("Mesh B"),
            NodeSocket::int("Operation (0:Diff, 1:Union, 2:Intersect)", 0),
        ]
    }

    fn outputs(&self) -> Vec<NodeSocket> {
        vec![NodeSocket::geometry("Geometry")]
    }

    fn execute(&self, inputs: &[SocketValue]) -> Result<Vec<SocketValue>, NodeGraphError> {
        let Some(mesh_a) = inputs.get(0).and_then(|v| v.as_geometry()) else {
            return Ok(vec![SocketValue::Geometry(None)]);
        };
        let Some(mesh_b) = inputs.get(1).and_then(|v| v.as_geometry()) else {
            return Ok(vec![SocketValue::Geometry(Some(mesh_a.clone()))]);
        };

        let op_code = inputs.get(2).map(|v| v.as_int()).unwrap_or(0);
        let op = match op_code {
            1 => BooleanOp::Union,
            2 => BooleanOp::Intersection,
            _ => BooleanOp::Difference,
        };

        let res = mesh_a.boolean(mesh_b, op).map_err(|e| NodeGraphError::ExecutionError(e.to_string()))?;
        Ok(vec![SocketValue::Geometry(Some(res))])
    }

    fn clone_box(&self) -> Box<dyn GeoNodeOp> {
        Box::new(self.clone())
    }
}


#[derive(Debug, Clone)]
pub struct SolidifyNode;

impl GeoNodeOp for SolidifyNode {
    fn name(&self) -> &str {
        "Solidify"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::GeometryOps
    }

    fn inputs(&self) -> Vec<NodeSocket> {
        vec![
            NodeSocket::geometry("Geometry"),
            NodeSocket::float("Thickness", 0.2),
        ]
    }

    fn outputs(&self) -> Vec<NodeSocket> {
        vec![NodeSocket::geometry("Geometry")]
    }

    fn execute(&self, inputs: &[SocketValue]) -> Result<Vec<SocketValue>, NodeGraphError> {
        let Some(mesh) = inputs.get(0).and_then(|v| v.as_geometry()) else {
            return Ok(vec![SocketValue::Geometry(None)]);
        };

        let thickness = inputs.get(1).map(|v| v.as_float()).unwrap_or(0.2);
        let modifier = SolidifyModifier::new(thickness);
        let res = modifier.apply(mesh).map_err(|e| NodeGraphError::ExecutionError(e.to_string()))?;

        Ok(vec![SocketValue::Geometry(Some(res))])
    }

    fn clone_box(&self) -> Box<dyn GeoNodeOp> {
        Box::new(self.clone())
    }
}


#[derive(Debug, Clone)]
pub struct DecimateNode;

impl GeoNodeOp for DecimateNode {
    fn name(&self) -> &str {
        "Decimate (QEM)"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::GeometryOps
    }

    fn inputs(&self) -> Vec<NodeSocket> {
        vec![
            NodeSocket::geometry("Geometry"),
            NodeSocket::float("Ratio", 0.5),
        ]
    }

    fn outputs(&self) -> Vec<NodeSocket> {
        vec![NodeSocket::geometry("Geometry")]
    }

    fn execute(&self, inputs: &[SocketValue]) -> Result<Vec<SocketValue>, NodeGraphError> {
        let Some(mesh) = inputs.get(0).and_then(|v| v.as_geometry()) else {
            return Ok(vec![SocketValue::Geometry(None)]);
        };

        let ratio = inputs.get(1).map(|v| v.as_float()).unwrap_or(0.5).clamp(0.01, 1.0);
        let res = decimate_mesh(mesh, ratio).map_err(|e| NodeGraphError::ExecutionError(e.to_string()))?;

        Ok(vec![SocketValue::Geometry(Some(res))])
    }

    fn clone_box(&self) -> Box<dyn GeoNodeOp> {
        Box::new(self.clone())
    }
}


#[derive(Debug, Clone)]
pub struct WeldNode;

impl GeoNodeOp for WeldNode {
    fn name(&self) -> &str {
        "Merge by Distance"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::GeometryOps
    }

    fn inputs(&self) -> Vec<NodeSocket> {
        vec![
            NodeSocket::geometry("Geometry"),
            NodeSocket::float("Distance", 0.001),
        ]
    }

    fn outputs(&self) -> Vec<NodeSocket> {
        vec![NodeSocket::geometry("Geometry")]
    }

    fn execute(&self, inputs: &[SocketValue]) -> Result<Vec<SocketValue>, NodeGraphError> {
        let Some(mesh) = inputs.get(0).and_then(|v| v.as_geometry()) else {
            return Ok(vec![SocketValue::Geometry(None)]);
        };

        let dist = inputs.get(1).map(|v| v.as_float()).unwrap_or(0.001).max(1e-5);
        let res = mesh.weld_vertices_by_distance(dist).map_err(|e| NodeGraphError::ExecutionError(e.to_string()))?;

        Ok(vec![SocketValue::Geometry(Some(res))])
    }

    fn clone_box(&self) -> Box<dyn GeoNodeOp> {
        Box::new(self.clone())
    }
}

#[derive(Debug, Clone)]
pub struct VoxelRemeshNode;

impl GeoNodeOp for VoxelRemeshNode {
    fn name(&self) -> &str {
        "Voxel Remesh (DynaMesh)"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::GeometryOps
    }

    fn inputs(&self) -> Vec<NodeSocket> {
        vec![
            NodeSocket::geometry("Geometry"),
            NodeSocket::float("Voxel Size", 0.1),
            NodeSocket::int("Smooth Iterations", 2),
        ]
    }

    fn outputs(&self) -> Vec<NodeSocket> {
        vec![NodeSocket::geometry("Geometry")]
    }

    fn execute(&self, inputs: &[SocketValue]) -> Result<Vec<SocketValue>, NodeGraphError> {
        let Some(mesh) = inputs.get(0).and_then(|v| v.as_geometry()) else {
            return Ok(vec![SocketValue::Geometry(None)]);
        };

        let voxel_size = inputs.get(1).map(|v| v.as_float()).unwrap_or(0.1).clamp(0.01, 2.0);
        let smooth_iters = inputs.get(2).map(|v| v.as_int()).unwrap_or(2).clamp(0, 10) as usize;

        let mut remeshed = mesh.voxel_remesh(voxel_size)
            .map_err(|e| NodeGraphError::ExecutionError(e.to_string()))?;
        if smooth_iters > 0 {
            let all_verts: Vec<_> = remeshed.vertices.keys().collect();
            let _ = remeshed.relax_vertices(&all_verts, smooth_iters, 0.25);
        }

        Ok(vec![SocketValue::Geometry(Some(remeshed))])
    }

    fn clone_box(&self) -> Box<dyn GeoNodeOp> {
        Box::new(self.clone())
    }
}


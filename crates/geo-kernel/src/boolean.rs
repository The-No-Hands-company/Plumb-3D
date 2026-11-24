use crate::mesh::{Mesh, MeshError, VertexKey};
use core_math::{Ray, Vec2, Vec3};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BooleanOp {
    Union,
    Difference,
    Intersection,
}

const PLANE_EPSILON: f32 = 1e-5;

#[derive(Debug, Clone, Copy)]
pub struct BspVertex {
    pub pos: Vec3,
    pub normal: Vec3,
    pub uv: Vec2,
}

impl BspVertex {
    pub fn new(pos: Vec3, normal: Vec3, uv: Vec2) -> Self {
        Self { pos, normal, uv }
    }

    pub fn lerp(&self, other: &Self, t: f32) -> Self {
        Self {
            pos: self.pos + (other.pos - self.pos) * t,
            normal: (self.normal + (other.normal - self.normal) * t).normalize(),
            uv: self.uv + (other.uv - self.uv) * t,
        }
    }

    pub fn flip(&mut self) {
        self.normal = -self.normal;
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BspPlane {
    pub normal: Vec3,
    pub w: f32,
}

impl BspPlane {
    pub fn from_points(a: Vec3, b: Vec3, c: Vec3) -> Option<Self> {
        let n = (b - a).cross(c - a);
        let len = n.length();
        if len < 1e-7 {
            return None;
        }
        let normal = n / len;
        let w = normal.dot(a);
        Some(Self { normal, w })
    }

    pub fn flip(&mut self) {
        self.normal = -self.normal;
        self.w = -self.w;
    }

    pub fn split_polygon(
        &self,
        polygon: &BspPolygon,
        coplanar_front: &mut Vec<BspPolygon>,
        coplanar_back: &mut Vec<BspPolygon>,
        front: &mut Vec<BspPolygon>,
        back: &mut Vec<BspPolygon>,
    ) {
        const COPLANAR: u8 = 0;
        const FRONT: u8 = 1;
        const BACK: u8 = 2;
        const SPANNING: u8 = 3;

        let mut polygon_type = 0;
        let mut types = Vec::with_capacity(polygon.vertices.len());

        for v in &polygon.vertices {
            let t = self.normal.dot(v.pos) - self.w;
            let v_type = if t < -PLANE_EPSILON { BACK } else if t > PLANE_EPSILON { FRONT } else { COPLANAR };
            polygon_type |= v_type;
            types.push(v_type);
        }

        match polygon_type {
            COPLANAR => {
                if self.normal.dot(polygon.plane.normal) > 0.0 { coplanar_front.push(polygon.clone()); }
                else { coplanar_back.push(polygon.clone()); }
            }
            FRONT => front.push(polygon.clone()),
            BACK => back.push(polygon.clone()),
            SPANNING => {
                let (mut f_verts, mut b_verts) = (Vec::new(), Vec::new());
                let n = polygon.vertices.len();
                for i in 0..n {
                    let j = (i + 1) % n;
                    let (ti, tj) = (types[i], types[j]);
                    let (vi, vj) = (&polygon.vertices[i], &polygon.vertices[j]);

                    if ti != BACK { f_verts.push(*vi); }
                    if ti != FRONT { b_verts.push(*vi); }
                    if (ti | tj) == SPANNING {
                        let denom = self.normal.dot(vj.pos - vi.pos);
                        if denom.abs() > 1e-8 {
                            let t = ((self.w - self.normal.dot(vi.pos)) / denom).clamp(0.0, 1.0);
                            let v = vi.lerp(vj, t);
                            f_verts.push(v);
                            b_verts.push(v);
                        }
                    }
                }
                if f_verts.len() >= 3 {
                    if let Some(p) = BspPolygon::from_vertices(f_verts) { front.push(p); }
                }
                if b_verts.len() >= 3 {
                    if let Some(p) = BspPolygon::from_vertices(b_verts) { back.push(p); }
                }
            }
            _ => {}
        }
    }
}

#[derive(Debug, Clone)]
pub struct BspPolygon {
    pub vertices: Vec<BspVertex>,
    pub plane: BspPlane,
}

impl BspPolygon {
    pub fn from_vertices(vertices: Vec<BspVertex>) -> Option<Self> {
        if vertices.len() < 3 {
            return None;
        }
        let p0 = vertices[0].pos;
        let mut plane = None;
        for i in 1..(vertices.len() - 1) {
            let p1 = vertices[i].pos;
            let p2 = vertices[i + 1].pos;
            if let Some(pl) = BspPlane::from_points(p0, p1, p2) {
                plane = Some(pl);
                break;
            }
        }
        let plane = plane?;
        if Self::polygon_area(&vertices) < 1e-7 {
            return None;
        }
        Some(Self { vertices, plane })
    }

    pub fn polygon_area(vertices: &[BspVertex]) -> f32 {
        if vertices.len() < 3 {
            return 0.0;
        }
        let p0 = vertices[0].pos;
        let mut total = Vec3::ZERO;
        for i in 1..(vertices.len() - 1) {
            let p1 = vertices[i].pos;
            let p2 = vertices[i + 1].pos;
            total += (p1 - p0).cross(p2 - p0);
        }
        0.5 * total.length()
    }

    pub fn flip(&mut self) {
        self.vertices.reverse();
        for v in &mut self.vertices {
            v.flip();
        }
        self.plane.flip();
    }
}

#[derive(Debug, Default)]
pub struct BspNode {
    pub plane: Option<BspPlane>,
    pub front: Option<Box<BspNode>>,
    pub back: Option<Box<BspNode>>,
    pub polygons: Vec<BspPolygon>,
}

impl BspNode {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_polygons(polygons: Vec<BspPolygon>) -> Self {
        let mut node = Self::new();
        node.build(polygons);
        node
    }

    pub fn invert(&mut self) {
        for p in &mut self.polygons {
            p.flip();
        }
        if let Some(pl) = &mut self.plane {
            pl.flip();
        }
        if let Some(f) = &mut self.front {
            f.invert();
        }
        if let Some(b) = &mut self.back {
            b.invert();
        }
        std::mem::swap(&mut self.front, &mut self.back);
    }

    pub fn clip_polygons(&self, polygons: &[BspPolygon]) -> Vec<BspPolygon> {
        let plane = match &self.plane {
            Some(pl) => pl,
            None => return polygons.to_vec(),
        };

        let mut cp_f = Vec::new();
        let mut cp_b = Vec::new();
        let mut f = Vec::new();
        let mut b = Vec::new();

        for poly in polygons {
            plane.split_polygon(poly, &mut cp_f, &mut cp_b, &mut f, &mut b);
        }

        let mut res_f = if let Some(front) = &self.front {
            front.clip_polygons(&f)
        } else {
            f
        };

        let res_b = if let Some(back) = &self.back {
            back.clip_polygons(&b)
        } else {
            Vec::new()
        };

        res_f.extend(cp_f);
        res_f.extend(res_b);
        res_f
    }

    pub fn clip_to(&mut self, other: &BspNode) {
        self.polygons = other.clip_polygons(&self.polygons);
        if let Some(front) = &mut self.front {
            front.clip_to(other);
        }
        if let Some(back) = &mut self.back {
            back.clip_to(other);
        }
    }

    pub fn all_polygons(&self, out: &mut Vec<BspPolygon>) {
        out.extend(self.polygons.clone());
        if let Some(front) = &self.front {
            front.all_polygons(out);
        }
        if let Some(back) = &self.back {
            back.all_polygons(out);
        }
    }

    pub fn build(&mut self, polygons: Vec<BspPolygon>) {
        if polygons.is_empty() {
            return;
        }

        if self.plane.is_none() {
            self.plane = Some(polygons[0].plane);
        }
        let plane = self.plane.unwrap();

        let mut cp_f = Vec::new();
        let mut cp_b = Vec::new();
        let mut f = Vec::new();
        let mut b = Vec::new();

        for poly in &polygons {
            plane.split_polygon(poly, &mut cp_f, &mut cp_b, &mut f, &mut b);
        }

        self.polygons.extend(cp_f);
        self.polygons.extend(cp_b);

        if !f.is_empty() {
            if self.front.is_none() {
                self.front = Some(Box::new(BspNode::new()));
            }
            self.front.as_mut().unwrap().build(f);
        }
        if !b.is_empty() {
            if self.back.is_none() {
                self.back = Some(Box::new(BspNode::new()));
            }
            self.back.as_mut().unwrap().build(b);
        }
    }
}

impl Mesh {
    /// Test whether a 3D point is strictly inside a closed, watertight 2-manifold mesh.
    /// Uses 3D Jordan ray intersection parity with jittered rays to prevent edge grazing.
    pub fn is_point_inside(&self, point: Vec3) -> bool {
        let aabb = self.compute_aabb();
        if point.x < aabb.min.x || point.x > aabb.max.x
            || point.y < aabb.min.y || point.y > aabb.max.y
            || point.z < aabb.min.z || point.z > aabb.max.z
        {
            return false;
        }

        let ray_dir = Vec3::new(1.0, 0.000137, 0.000259).normalize();
        let ray = Ray::new(point, ray_dir);
        let mut hit_count = 0;

        for (fk, _) in &self.faces {
            let v_keys = self.face_vertices(fk);
            if v_keys.len() < 3 {
                continue;
            }

            let p0 = self.vertices[v_keys[0]].position;
            for i in 1..(v_keys.len() - 1) {
                let p1 = self.vertices[v_keys[i]].position;
                let p2 = self.vertices[v_keys[i + 1]].position;

                let edge1 = p1 - p0;
                let edge2 = p2 - p0;
                let h = ray.direction.cross(edge2);
                let a = edge1.dot(h);

                if a.abs() < 1e-7 {
                    continue;
                }

                let f = 1.0 / a;
                let s = ray.origin - p0;
                let u = f * s.dot(h);
                if !(0.0..=1.0).contains(&u) {
                    continue;
                }

                let q = s.cross(edge1);
                let v = f * ray.direction.dot(q);
                if v < 0.0 || u + v > 1.0 {
                    continue;
                }

                let t = f * edge2.dot(q);
                if t > 1e-5 {
                    hit_count += 1;
                }
            }
        }

        hit_count % 2 == 1
    }

    /// Convert mesh into a list of planar BSP polygons (preserving convex quads).
    pub fn to_bsp_polygons(&self) -> Vec<BspPolygon> {
        let mut polygons = Vec::new();
        for (fk, _) in &self.faces {
            let v_keys = self.face_vertices(fk);
            if v_keys.len() < 3 {
                continue;
            }
            let verts: Vec<BspVertex> = v_keys
                .iter()
                .map(|&vk| {
                    let v = &self.vertices[vk];
                    BspVertex::new(v.position, v.normal, v.uv)
                })
                .collect();

            if verts.len() <= 4 {
                if let Some(poly) = BspPolygon::from_vertices(verts.clone()) {
                    polygons.push(poly);
                    continue;
                }
            }
            for i in 1..(verts.len() - 1) {
                let tri = vec![verts[0], verts[i], verts[i + 1]];
                if let Some(poly) = BspPolygon::from_vertices(tri) {
                    polygons.push(poly);
                }
            }
        }
        polygons
    }

    /// Reconstruct a manifold mesh from BSP polygons with O(1) vertex welding and degenerate culling.
    pub fn from_bsp_polygons(polygons: &[BspPolygon]) -> Result<Mesh, MeshError> {
        let mut mesh = Mesh::new();
        let mut pos_map: HashMap<(i32, i32, i32), VertexKey> = HashMap::new();
        let quantize = |p: Vec3| -> (i32, i32, i32) {
            (
                (p.x * 10000.0).round() as i32,
                (p.y * 10000.0).round() as i32,
                (p.z * 10000.0).round() as i32,
            )
        };

        for poly in polygons {
            if poly.vertices.len() < 3 {
                continue;
            }
            if poly.vertices.len() <= 4 {
                let mut vks = Vec::with_capacity(poly.vertices.len());
                for v in &poly.vertices {
                    let key = quantize(v.pos);
                    let vk = *pos_map.entry(key).or_insert_with(|| {
                        mesh.add_vertex_with_uv(v.pos, v.uv)
                    });
                    if vks.is_empty() || *vks.last().unwrap() != vk {
                        vks.push(vk);
                    }
                }
                if vks.len() > 1 && vks.first() == vks.last() {
                    vks.pop();
                }
                if vks.len() >= 3 {
                    let mut unique = true;
                    for a in 0..vks.len() {
                        for b in (a + 1)..vks.len() {
                            if vks[a] == vks[b] {
                                unique = false;
                                break;
                            }
                        }
                    }
                    if unique {
                        let _ = mesh.add_face(&vks);
                        continue;
                    }
                }
            }

            let n = poly.vertices.len();
            for i in 1..(n - 1) {
                let tri = [poly.vertices[0], poly.vertices[i], poly.vertices[i + 1]];
                let mut tri_keys = Vec::with_capacity(3);
                for v in &tri {
                    let key = quantize(v.pos);
                    let vk = *pos_map.entry(key).or_insert_with(|| {
                        mesh.add_vertex_with_uv(v.pos, v.uv)
                    });
                    if tri_keys.is_empty() || *tri_keys.last().unwrap() != vk {
                        tri_keys.push(vk);
                    }
                }
                if tri_keys.len() == 3 && tri_keys[0] != tri_keys[2] {
                    let p0 = mesh.vertices[tri_keys[0]].position;
                    let p1 = mesh.vertices[tri_keys[1]].position;
                    let p2 = mesh.vertices[tri_keys[2]].position;
                    if (p1 - p0).cross(p2 - p0).length_squared() > 1e-10 {
                        let _ = mesh.add_face(&tri_keys);
                    }
                }
            }
        }

        let _ = mesh.clean_degenerate_geometry(1e-6, 1e-4);
        mesh.update_normals();
        Ok(mesh)
    }

    /// Constructive Solid Geometry (CSG) Boolean operation between self (A) and other (B).
    /// Hardened BSP-tree engine: watertight clipping, coplanar face handling, seam welding.
    pub fn boolean(&self, other: &Mesh, op: BooleanOp) -> Result<Mesh, MeshError> {
        if self.faces.is_empty() {
            return match op {
                BooleanOp::Union => Ok(other.clone()),
                _ => Ok(Mesh::new()),
            };
        }
        if other.faces.is_empty() {
            return match op {
                BooleanOp::Union | BooleanOp::Difference => Ok(self.clone()),
                BooleanOp::Intersection => Ok(Mesh::new()),
            };
        }

        let aabb_a = self.compute_aabb();
        let aabb_b = other.compute_aabb();

        // Check if bounding boxes overlap with epsilon margin
        let margin = 1e-4;
        let overlaps = aabb_a.min.x <= aabb_b.max.x + margin && aabb_a.max.x >= aabb_b.min.x - margin
            && aabb_a.min.y <= aabb_b.max.y + margin && aabb_a.max.y >= aabb_b.min.y - margin
            && aabb_a.min.z <= aabb_b.max.z + margin && aabb_a.max.z >= aabb_b.min.z - margin;

        if !overlaps {
            return match op {
                BooleanOp::Union => {
                    let mut res = self.clone();
                    for (fk, _) in &other.faces {
                        let v_keys = other.face_vertices(fk);
                        let mut new_vks = Vec::with_capacity(v_keys.len());
                        for vk in v_keys {
                            let v = &other.vertices[vk];
                            new_vks.push(res.add_vertex_with_uv(v.position, v.uv));
                        }
                        let _ = res.add_face(&new_vks);
                    }
                    res.update_normals();
                    Ok(res)
                }
                BooleanOp::Difference => Ok(self.clone()),
                BooleanOp::Intersection => Ok(Mesh::new()),
            };
        }

        // Check if other is nested entirely inside self without boundary contact
        let b_inside_a = aabb_b.min.x > aabb_a.min.x + margin && aabb_b.max.x < aabb_a.max.x - margin
            && aabb_b.min.y > aabb_a.min.y + margin && aabb_b.max.y < aabb_a.max.y - margin
            && aabb_b.min.z > aabb_a.min.z + margin && aabb_b.max.z < aabb_a.max.z - margin;

        if b_inside_a {
            match op {
                BooleanOp::Union => return Ok(self.clone()),
                BooleanOp::Difference => {
                    let mut res = self.clone();
                    for (fk, _) in &other.faces {
                        let v_keys = other.face_vertices(fk);
                        let mut new_vks = Vec::with_capacity(v_keys.len());
                        for vk in v_keys {
                            let v = &other.vertices[vk];
                            new_vks.push(res.add_vertex_with_uv(v.position, v.uv));
                        }
                        new_vks.reverse(); // Invert normal for internal cavity
                        let _ = res.add_face(&new_vks);
                    }
                    res.update_normals();
                    return Ok(res);
                }
                BooleanOp::Intersection => return Ok(other.clone()),
            }
        }

        let a_polys = self.to_bsp_polygons();
        let b_polys = other.to_bsp_polygons();

        let mut a = BspNode::from_polygons(a_polys);
        let mut b = BspNode::from_polygons(b_polys);

        let mut result_polygons = Vec::new();

        match op {
            BooleanOp::Union => {
                a.clip_to(&b);
                b.clip_to(&a);
                b.invert();
                b.clip_to(&a);
                b.invert();
                let mut b_all = Vec::new();
                b.all_polygons(&mut b_all);
                a.build(b_all);
                a.all_polygons(&mut result_polygons);
            }
            BooleanOp::Difference => {
                a.invert();
                a.clip_to(&b);
                b.clip_to(&a);
                b.invert();
                b.clip_to(&a);
                b.invert();
                let mut b_all = Vec::new();
                b.all_polygons(&mut b_all);
                a.build(b_all);
                a.invert();
                a.all_polygons(&mut result_polygons);
            }
            BooleanOp::Intersection => {
                a.invert();
                b.clip_to(&a);
                b.invert();
                a.clip_to(&b);
                b.clip_to(&a);
                let mut b_all = Vec::new();
                b.all_polygons(&mut b_all);
                a.build(b_all);
                a.invert();
                a.all_polygons(&mut result_polygons);
            }
        }

        let mut mesh = Self::from_bsp_polygons(&result_polygons)?;
        if mesh.faces.is_empty() && (op == BooleanOp::Union || op == BooleanOp::Difference) {
            return Ok(self.clone());
        }
        mesh.update_normals();
        Ok(mesh)
    }
}

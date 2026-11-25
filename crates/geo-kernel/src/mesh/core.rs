use core_math::{Vec2, Vec3};
use std::collections::HashSet;

use super::types::{Edge, EdgeKey, Face, FaceKey, HalfEdge, HalfEdgeKey, Mesh, MeshError, Vertex, VertexGroup, VertexKey};

impl Mesh {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    pub fn face_count(&self) -> usize {
        self.faces.len()
    }

    pub fn half_edge_count(&self) -> usize {
        self.half_edges.len()
    }

    /// Add a new vertex at the given 3D position.
    pub fn add_vertex(&mut self, position: Vec3) -> VertexKey {
        self.vertices.insert(Vertex::new(position))
    }

    /// Add a new vertex with 3D position and UV coordinates.
    pub fn add_vertex_with_uv(&mut self, position: Vec3, uv: Vec2) -> VertexKey {
        self.vertices.insert(Vertex::with_uv(position, uv))
    }

    /// Add a new vertex with 3D position, UV coordinates, and RGBA color.
    pub fn add_vertex_with_uv_and_color(&mut self, position: Vec3, uv: Vec2, color: [f32; 4]) -> VertexKey {
        self.vertices.insert(Vertex::with_uv_and_color(position, uv, color))
    }

    /// Set RGBA color for an existing vertex.
    pub fn set_vertex_color(&mut self, vertex: VertexKey, color: [f32; 4]) {
        if let Some(v) = self.vertices.get_mut(vertex) {
            v.color = color;
        }
    }

    /// Query RGBA color of an existing vertex. Returns white `[1.0, 1.0, 1.0, 1.0]` if vertex is missing.
    pub fn get_vertex_color(&self, vertex: VertexKey) -> [f32; 4] {
        self.vertices.get(vertex).map(|v| v.color).unwrap_or([1.0, 1.0, 1.0, 1.0])
    }

    /// Fill all vertices of the mesh with a uniform color.
    pub fn fill_vertex_colors(&mut self, color: [f32; 4]) {
        for v in self.vertices.values_mut() {
            v.color = color;
        }
    }

    /// Add a vertex group with a given name, returning its index.
    pub fn add_vertex_group(&mut self, name: impl Into<String>) -> usize {
        let name_str = name.into();
        if let Some(pos) = self.vertex_groups.iter().position(|vg| vg.name == name_str) {
            return pos;
        }
        self.vertex_groups.push(VertexGroup::new(name_str));
        self.vertex_groups.len() - 1
    }

    /// Retrieve a vertex group by name.
    pub fn get_vertex_group(&self, name: &str) -> Option<&VertexGroup> {
        self.vertex_groups.iter().find(|vg| vg.name == name)
    }

    /// Retrieve a mutable vertex group by name.
    pub fn get_vertex_group_mut(&mut self, name: &str) -> Option<&mut VertexGroup> {
        self.vertex_groups.iter_mut().find(|vg| vg.name == name)
    }

    /// Set a vertex weight in the named vertex group.
    pub fn set_vertex_weight(&mut self, group_name: &str, vertex: VertexKey, weight: f32) {
        if let Some(vg) = self.get_vertex_group_mut(group_name) {
            vg.set_weight(vertex, weight);
        } else {
            let mut vg = VertexGroup::new(group_name);
            vg.set_weight(vertex, weight);
            self.vertex_groups.push(vg);
        }
    }

    /// Query a vertex weight in the named vertex group. Returns 0.0 if not assigned.
    pub fn get_vertex_weight(&self, group_name: &str, vertex: VertexKey) -> f32 {
        self.get_vertex_group(group_name)
            .map(|vg| vg.get_weight(vertex))
            .unwrap_or(0.0)
    }

    /// Remove a vertex group by name.
    pub fn remove_vertex_group(&mut self, name: &str) -> bool {
        if let Some(pos) = self.vertex_groups.iter().position(|vg| vg.name == name) {
            self.vertex_groups.remove(pos);
            true
        } else {
            false
        }
    }

    /// Add a polygon face defined by a slice of vertex keys (counter-clockwise order).
    /// Supports triangles, quads, and general N-gons (N >= 3).
    pub fn add_face(&mut self, v_keys: &[VertexKey]) -> Result<FaceKey, MeshError> {
        let n = v_keys.len();
        if n < 3 {
            return Err(MeshError::DegenerateFace(n));
        }

        // Validate vertex existence and uniqueness
        for (i, &vk) in v_keys.iter().enumerate() {
            if !self.vertices.contains_key(vk) {
                return Err(MeshError::InvalidVertex(vk));
            }
            for &other in &v_keys[i + 1..] {
                if vk == other {
                    return Err(MeshError::DuplicateVertexInFace(vk));
                }
            }
        }

        // Check for non-manifold conditions
        for i in 0..n {
            let v_curr = v_keys[i];
            let v_next = v_keys[(i + 1) % n];
            if let Some(existing_he) = self.find_half_edge(v_curr, v_next) {
                if self.half_edges[existing_he].face.is_some() {
                    return Err(MeshError::NonManifoldEdge(v_curr, v_next));
                }
            }
        }

        // Reserve face key
        let face_key = self.faces.insert_with_key(|_fk| Face::new(HalfEdgeKey::default()));
        let mut loop_he_keys = Vec::with_capacity(n);

        for i in 0..n {
            let v_curr = v_keys[i];
            let v_next = v_keys[(i + 1) % n];

            if let Some(existing_he) = self.find_half_edge(v_curr, v_next) {
                // Claim the boundary half-edge for this face
                self.half_edges[existing_he].face = Some(face_key);
                loop_he_keys.push(existing_he);
            } else {
                // Create brand new edge and pair of half-edges
                let edge_key = self.edges.insert_with_key(|_ek| Edge::new(HalfEdgeKey::default()));

                let he_key = self.half_edges.insert_with_key(|_k| HalfEdge {
                    origin: v_curr,
                    target: v_next,
                    edge: edge_key,
                    face: Some(face_key),
                    next: HalfEdgeKey::default(),
                    prev: HalfEdgeKey::default(),
                    twin: HalfEdgeKey::default(),
                });

                let twin_key = self.half_edges.insert_with_key(|_tk| HalfEdge {
                    origin: v_next,
                    target: v_curr,
                    edge: edge_key,
                    face: None,
                    next: HalfEdgeKey::default(),
                    prev: HalfEdgeKey::default(),
                    twin: he_key,
                });

                self.half_edges[he_key].twin = twin_key;
                self.edges[edge_key].half_edge = he_key;

                self.vertices[v_curr].half_edges.push(he_key);
                self.vertices[v_next].half_edges.push(twin_key);

                loop_he_keys.push(he_key);
            }
        }

        // Link next and prev pointers in the face cycle
        for i in 0..n {
            let curr = loop_he_keys[i];
            let next = loop_he_keys[(i + 1) % n];
            let prev = loop_he_keys[(i + n - 1) % n];

            self.half_edges[curr].next = next;
            self.half_edges[curr].prev = prev;
        }

        // Point face to first half-edge
        self.faces[face_key].half_edge = loop_he_keys[0];

        // Link any boundary half-edges around vertices so cycles are closed
        self.link_boundary_half_edges();

        // Compute face normal using Newell's method
        let normal = self.compute_face_normal_newell(v_keys);
        self.faces[face_key].normal = normal;

        Ok(face_key)
    }

    /// Adds an undirected wireframe edge between two vertices without creating a face.
    pub fn add_edge(&mut self, origin: VertexKey, target: VertexKey) -> Result<EdgeKey, MeshError> {
        if origin == target {
            return Err(MeshError::InvariantFailed("Self-loop edge not allowed".into()));
        }
        if !self.vertices.contains_key(origin) {
            return Err(MeshError::InvalidVertex(origin));
        }
        if !self.vertices.contains_key(target) {
            return Err(MeshError::InvalidVertex(target));
        }
        if let Some(existing) = self.find_edge(origin, target) {
            return Ok(existing);
        }

        let edge_key = self.edges.insert_with_key(|_ek| Edge::new(HalfEdgeKey::default()));
        let he_key = self.half_edges.insert_with_key(|_k| HalfEdge {
            origin,
            target,
            edge: edge_key,
            face: None,
            next: HalfEdgeKey::default(),
            prev: HalfEdgeKey::default(),
            twin: HalfEdgeKey::default(),
        });
        let twin_key = self.half_edges.insert_with_key(|_tk| HalfEdge {
            origin: target,
            target: origin,
            edge: edge_key,
            face: None,
            next: HalfEdgeKey::default(),
            prev: HalfEdgeKey::default(),
            twin: he_key,
        });

        self.half_edges[he_key].twin = twin_key;
        self.edges[edge_key].half_edge = he_key;

        self.vertices[origin].half_edges.push(he_key);
        self.vertices[target].half_edges.push(twin_key);

        Ok(edge_key)
    }

    /// Link open boundary half-edges (face == None) into continuous cycles.
    fn link_boundary_half_edges(&mut self) {
        let boundary_keys: Vec<HalfEdgeKey> = self
            .half_edges
            .iter()
            .filter(|(_, he)| he.face.is_none())
            .map(|(k, _)| k)
            .collect();

        // Reset next and prev on all boundary half-edges first
        for &k in &boundary_keys {
            self.half_edges[k].next = HalfEdgeKey::default();
            self.half_edges[k].prev = HalfEdgeKey::default();
        }

        let mut used_out: HashSet<HalfEdgeKey> = HashSet::new();

        for he_key in boundary_keys {
            let target = self.half_edges[he_key].target;
            // Find an unused outgoing half-edge from target that is also on the boundary
            if let Some(v) = self.vertices.get(target) {
                for &out_he in &v.half_edges {
                    if self.half_edges[out_he].face.is_none() && !used_out.contains(&out_he) {
                        self.half_edges[he_key].next = out_he;
                        self.half_edges[out_he].prev = he_key;
                        used_out.insert(out_he);
                        break;
                    }
                }
            }
        }
    }

    /// Find an existing half-edge directed from `origin` to `target`.
    pub fn find_half_edge(&self, origin: VertexKey, target: VertexKey) -> Option<HalfEdgeKey> {
        let v = self.vertices.get(origin)?;
        (&v.half_edges).into_iter().find(|&&he_key| self.half_edges[he_key].target == target).copied()
    }

    /// Calculate face normal using Newell's method for arbitrary N-gons.
    pub fn compute_face_normal_newell(&self, v_keys: &[VertexKey]) -> Vec3 {
        let n = v_keys.len();
        let mut normal = Vec3::ZERO;

        for i in 0..n {
            let p0 = self.vertices[v_keys[i]].position;
            let p1 = self.vertices[v_keys[(i + 1) % n]].position;

            normal.x += (p0.y - p1.y) * (p0.z + p1.z);
            normal.y += (p0.z - p1.z) * (p0.x + p1.x);
            normal.z += (p0.x - p1.x) * (p0.y + p1.y);
        }

        normal.normalize_or_zero()
    }

    /// Retrieve all vertices of a face in counter-clockwise order.
    pub fn face_vertices(&self, face_key: FaceKey) -> Vec<VertexKey> {
        let mut vertices = Vec::new();
        if let Some(face) = self.faces.get(face_key) {
            let start = face.half_edge;
            let mut curr = start;
            loop {
                let he = &self.half_edges[curr];
                vertices.push(he.origin);
                curr = he.next;
                if curr == start || !self.half_edges.contains_key(curr) {
                    break;
                }
            }
        }
        vertices
    }

    /// Calculate the geometric center (centroid) of a face.
    pub fn face_centroid(&self, face_key: FaceKey) -> Vec3 {
        let f_verts = self.face_vertices(face_key);
        if f_verts.is_empty() {
            return Vec3::ZERO;
        }
        let mut sum = Vec3::ZERO;
        for &vk in &f_verts {
            if let Some(v) = self.vertices.get(vk) {
                sum += v.position;
            }
        }
        sum / (f_verts.len() as f32)
    }

    /// Calculate the 3D surface area of a polygon face.
    pub fn face_area(&self, face_key: FaceKey) -> f32 {
        let v_keys = self.face_vertices(face_key);
        let n = v_keys.len();
        if n < 3 {
            return 0.0;
        }
        let mut unnorm = Vec3::ZERO;
        for i in 0..n {
            let p0 = self.vertices[v_keys[i]].position;
            let p1 = self.vertices[v_keys[(i + 1) % n]].position;
            unnorm.x += (p0.y - p1.y) * (p0.z + p1.z);
            unnorm.y += (p0.z - p1.z) * (p0.x + p1.x);
            unnorm.z += (p0.x - p1.x) * (p0.y + p1.y);
        }
        unnorm.length() * 0.5
    }

    /// Deletes a face from the mesh and detaches its incident half-edges, turning adjacent edges into boundaries.
    pub fn delete_face(&mut self, face_key: FaceKey) {
        if self.faces.contains_key(face_key) {
            let face_hes = self.face_half_edges(face_key);
            for &he_key in &face_hes {
                if let Some(he) = self.half_edges.get_mut(he_key) {
                    he.face = None;
                }
            }
            self.faces.remove(face_key);
        }
    }

    /// Retrieve all half-edges belonging to a face loop.
    pub fn face_half_edges(&self, face_key: FaceKey) -> Vec<HalfEdgeKey> {
        let mut hes = Vec::new();
        if let Some(face) = self.faces.get(face_key) {
            let start = face.half_edge;
            let mut curr = start;
            loop {
                hes.push(curr);
                curr = self.half_edges[curr].next;
                if curr == start || !self.half_edges.contains_key(curr) {
                    break;
                }
            }
        }
        hes
    }

    /// Retrieve all 1-ring neighbor vertices connected to a given vertex.
    pub fn vertex_neighbors(&self, vertex_key: VertexKey) -> Vec<VertexKey> {
        let mut neighbors = Vec::new();
        if let Some(v) = self.vertices.get(vertex_key) {
            for &he_key in &v.half_edges {
                neighbors.push(self.half_edges[he_key].target);
            }
        }
        neighbors
    }

    /// Retrieve all faces incident to a given vertex.
    pub fn vertex_faces(&self, vertex_key: VertexKey) -> Vec<FaceKey> {
        let mut faces = Vec::new();
        if let Some(v) = self.vertices.get(vertex_key) {
            for &he_key in &v.half_edges {
                if let Some(fk) = self.half_edges[he_key].face {
                    if !faces.contains(&fk) {
                        faces.push(fk);
                    }
                }
            }
        }
        faces
    }

    /// Finds the edge connecting two vertices, if one exists.
    pub fn find_edge(&self, v0: VertexKey, v1: VertexKey) -> Option<EdgeKey> {
        if let Some(he_key) = self.find_half_edge(v0, v1) {
            return Some(self.half_edges[he_key].edge);
        }
        if let Some(he_key) = self.find_half_edge(v1, v0) {
            return Some(self.half_edges[he_key].edge);
        }
        None
    }

    /// Retrieve the faces incident to an edge (at most 2 for 2-manifold).
    pub fn edge_faces(&self, edge_key: EdgeKey) -> Vec<FaceKey> {
        let mut faces = Vec::with_capacity(2);
        if let Some(edge) = self.edges.get(edge_key) {
            let he = &self.half_edges[edge.half_edge];
            if let Some(fk) = he.face {
                faces.push(fk);
            }
            let twin = &self.half_edges[he.twin];
            if let Some(fk) = twin.face {
                faces.push(fk);
            }
        }
        faces
    }

    /// Sets the crease weight on an edge in [0.0, 1.0].
    pub fn set_edge_crease(&mut self, edge_key: EdgeKey, weight: f32) {
        if let Some(edge) = self.edges.get_mut(edge_key) {
            edge.crease_weight = weight.clamp(0.0, 1.0);
            edge.is_sharp = edge.crease_weight > 0.0;
        }
    }

    /// Gets the crease weight of an edge.
    pub fn get_edge_crease(&self, edge_key: EdgeKey) -> f32 {
        self.edges.get(edge_key).map(|e| e.crease_weight).unwrap_or(0.0)
    }

    /// Sets the crease weight on a vertex in [0.0, 1.0].
    pub fn set_vertex_crease(&mut self, vertex_key: VertexKey, weight: f32) {
        if let Some(vertex) = self.vertices.get_mut(vertex_key) {
            vertex.crease_weight = weight.clamp(0.0, 1.0);
        }
    }

    /// Gets the crease weight of a vertex.
    pub fn get_vertex_crease(&self, vertex_key: VertexKey) -> f32 {
        self.vertices.get(vertex_key).map(|v| v.crease_weight).unwrap_or(0.0)
    }

    /// Returns all creased vertices with their crease weight.
    pub fn creased_vertices(&self) -> Vec<(VertexKey, f32)> {
        self.vertices
            .iter()
            .filter(|(_, v)| v.crease_weight > 0.0)
            .map(|(k, v)| (k, v.crease_weight))
            .collect()
    }

    /// Sets whether an edge is marked sharp for normal generation and flat shading.
    pub fn set_edge_sharp(&mut self, edge_key: EdgeKey, sharp: bool) {
        if let Some(edge) = self.edges.get_mut(edge_key) {
            edge.is_sharp = sharp;
        }
    }

    /// Sets whether an edge is marked as a UV seam.
    pub fn set_edge_seam(&mut self, edge_key: EdgeKey, is_seam: bool) {
        if let Some(edge) = self.edges.get_mut(edge_key) {
            edge.is_seam = is_seam;
        }
    }

    /// Gets whether an edge is marked as a UV seam.
    pub fn is_edge_seam(&self, edge_key: EdgeKey) -> bool {
        self.edges.get(edge_key).is_some_and(|e| e.is_seam)
    }

    /// Mark or unmark a list of edges as UV seams.
    pub fn mark_seams(&mut self, edges: &[EdgeKey], is_seam: bool) {
        for &ek in edges {
            self.set_edge_seam(ek, is_seam);
        }
    }

    /// Clears all marked UV seams across the entire mesh.
    pub fn clear_all_seams(&mut self) {
        for edge in self.edges.values_mut() {
            edge.is_seam = false;
        }
    }

    /// Returns all edges currently marked as UV seams.
    pub fn get_seams(&self) -> Vec<EdgeKey> {
        self.edges
            .iter()
            .filter_map(|(k, e)| if e.is_seam { Some(k) } else { None })
            .collect()
    }

    /// Returns all creased edges with their crease weight.
    pub fn creased_edges(&self) -> Vec<(EdgeKey, f32)> {
        self.edges
            .iter()
            .filter(|(_, e)| e.crease_weight > 0.0)
            .map(|(k, e)| (k, e.crease_weight))
            .collect()
    }
}

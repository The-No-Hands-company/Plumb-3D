use crate::mesh::{Mesh, MeshError};

impl Mesh {
    /// Verify all topological invariants of the Half-Edge data structure:
    /// - Next/Prev dual pointer symmetry
    /// - Twin pointer symmetry
    /// - Origin/Target endpoint matching across twins
    /// - Cycle closure for face loops
    pub fn validate(&self) -> Result<(), MeshError> {
        for (he_key, he) in &self.half_edges {
            // 1. Next/prev cycle symmetry
            let next_he = self.half_edges.get(he.next).ok_or_else(|| {
                MeshError::InvariantFailed(format!("Half-edge {:?} next points to non-existent key {:?}", he_key, he.next))
            })?;
            if next_he.prev != he_key {
                return Err(MeshError::InvariantFailed(format!(
                    "Half-edge {:?}.next.prev ({:?}) != this ({:?})",
                    he_key, next_he.prev, he_key
                )));
            }

            let prev_he = self.half_edges.get(he.prev).ok_or_else(|| {
                MeshError::InvariantFailed(format!("Half-edge {:?} prev points to non-existent key {:?}", he_key, he.prev))
            })?;
            if prev_he.next != he_key {
                return Err(MeshError::InvariantFailed(format!(
                    "Half-edge {:?}.prev.next ({:?}) != this ({:?})",
                    he_key, prev_he.next, he_key
                )));
            }

            // 2. Twin symmetry
            let twin_he = self.half_edges.get(he.twin).ok_or_else(|| {
                MeshError::InvariantFailed(format!("Half-edge {:?} twin points to non-existent key {:?}", he_key, he.twin))
            })?;
            if twin_he.twin != he_key {
                return Err(MeshError::InvariantFailed(format!(
                    "Half-edge {:?}.twin.twin ({:?}) != this ({:?})",
                    he_key, twin_he.twin, he_key
                )));
            }

            // 3. Geometric endpoint matching
            if he.origin != twin_he.target || he.target != twin_he.origin {
                return Err(MeshError::InvariantFailed(format!(
                    "Half-edge {:?} endpoints ({:?} -> {:?}) mismatch twin {:?} ({:?} -> {:?})",
                    he_key, he.origin, he.target, he.twin, twin_he.origin, twin_he.target
                )));
            }

            // 4. Edge consistency
            if he.edge != twin_he.edge {
                return Err(MeshError::InvariantFailed(format!(
                    "Half-edge {:?} and twin {:?} point to different edges ({:?} vs {:?})",
                    he_key, he.twin, he.edge, twin_he.edge
                )));
            }
        }

        // Validate face loops
        for (f_key, face) in &self.faces {
            let start = face.half_edge;
            let mut curr = start;
            let mut count = 0;
            loop {
                let he = self.half_edges.get(curr).ok_or_else(|| {
                    MeshError::InvariantFailed(format!("Face {:?} contains invalid half-edge {:?}", f_key, curr))
                })?;
                if he.face != Some(f_key) {
                    return Err(MeshError::InvariantFailed(format!(
                        "Face {:?} half-edge {:?} references face {:?}",
                        f_key, curr, he.face
                    )));
                }
                count += 1;
                curr = he.next;
                if curr == start {
                    break;
                }
                if count > self.half_edges.len() {
                    return Err(MeshError::InvariantFailed(format!(
                        "Face {:?} half-edge loop does not close (infinite loop detected)",
                        f_key
                    )));
                }
            }
            if count < 3 {
                return Err(MeshError::DegenerateFace(count));
            }
        }

        Ok(())
    }

    /// Calculate the Euler characteristic: $\chi = V - E + F$.
    /// For a connected, closed orientable 2-manifold (sphere, cube), $\chi = 2$.
    pub fn euler_characteristic(&self) -> i32 {
        self.vertices.len() as i32 - self.edges.len() as i32 + self.faces.len() as i32
    }
}

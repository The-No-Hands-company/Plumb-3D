//! ISO 10303-21 (STEP AP203/AP214) CAD B-Rep Importer for Plumb 3D DCC.
//! Parses solid boundary representation (B-Rep) geometry from SolidWorks, Siemens NX,
//! Autodesk Fusion 360, FreeCAD, and Rhino into watertight 2-manifold BMesh surfaces.

use crate::mesh::{Mesh, MeshError, VertexKey};
use core_math::Vec3;
use std::collections::HashMap;

/// Parses an ISO 10303-21 STEP ASCII solid CAD file and reconstructs the BMesh solid.
pub fn import_step_str(content: &str) -> Result<Mesh, MeshError> {
    let mut mesh = Mesh::new();

    // Check for STEP header signature
    if !content.contains("ISO-10303-21") {
        return Err(MeshError::InvariantFailed(
            "Missing ISO-10303-21 STEP header signature".into(),
        ));
    }

    // 1. Check unit scaling: default is millimeters (scale by 0.001 to convert to meters)
    let is_milli = content.contains("MILLI") || !content.contains(".METRE.");
    let scale = if is_milli { 0.001 } else { 1.0 };

    // 2. Parse CARTESIAN_POINT entities: #id=CARTESIAN_POINT('',(x,y,z));
    let mut cartesian_points: HashMap<usize, Vec3> = HashMap::new();
    // 3. Parse VERTEX_POINT entities: #id=VERTEX_POINT('',#cp_id);
    let mut vertex_points: HashMap<usize, usize> = HashMap::new();
    // 4. Parse EDGE_CURVE entities: #id=EDGE_CURVE('',#vp1,#vp2,#curve_id,.T.);
    let mut edge_curves: HashMap<usize, (usize, usize)> = HashMap::new();
    // 5. Parse ORIENTED_EDGE entities: #id=ORIENTED_EDGE('',*,*,#edge_curve_id,.T./.F.);
    let mut oriented_edges: HashMap<usize, (usize, bool)> = HashMap::new();
    // 6. Parse EDGE_LOOP entities: #id=EDGE_LOOP('',(#oe1,#oe2,...));
    let mut edge_loops: HashMap<usize, Vec<usize>> = HashMap::new();
    // 7. Parse FACE_OUTER_BOUND entities: #id=FACE_OUTER_BOUND('',#loop_id,.T.);
    let mut outer_bounds: HashMap<usize, usize> = HashMap::new();
    // 8. Parse ADVANCED_FACE entities: #id=ADVANCED_FACE('',(#bound_id,...),#surface_id,.T.);
    let mut face_outer_loops: Vec<usize> = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with('#') {
            continue;
        }

        let eq_idx = match trimmed.find('=') {
            Some(i) => i,
            None => continue,
        };

        let id_str = &trimmed[1..eq_idx].trim();
        let id: usize = match id_str.parse() {
            Ok(v) => v,
            Err(_) => continue,
        };

        let rest = &trimmed[eq_idx + 1..];

        if rest.starts_with("CARTESIAN_POINT") {
            if let Some(coords) = parse_coordinate_triplet(rest) {
                cartesian_points.insert(id, coords * scale);
            }
        } else if rest.starts_with("VERTEX_POINT") {
            if let Some(target_cp) = parse_single_ref(rest) {
                vertex_points.insert(id, target_cp);
            }
        } else if rest.starts_with("EDGE_CURVE") {
            if let Some((v1, v2)) = parse_double_ref(rest) {
                edge_curves.insert(id, (v1, v2));
            }
        } else if rest.starts_with("ORIENTED_EDGE") {
            if let Some((edge_ref, forward)) = parse_oriented_edge_ref(rest) {
                oriented_edges.insert(id, (edge_ref, forward));
            }
        } else if rest.starts_with("EDGE_LOOP") {
            let refs = parse_ref_list(rest);
            edge_loops.insert(id, refs);
        } else if rest.starts_with("FACE_OUTER_BOUND") || rest.starts_with("FACE_BOUND") {
            if let Some(loop_id) = parse_single_ref(rest) {
                outer_bounds.insert(id, loop_id);
            }
        } else if rest.starts_with("ADVANCED_FACE") {
            let bound_refs = parse_ref_list(rest);
            for b in bound_refs {
                if let Some(&loop_id) = outer_bounds.get(&b) {
                    face_outer_loops.push(loop_id);
                } else if edge_loops.contains_key(&b) {
                    face_outer_loops.push(b);
                }
            }
        }
    }

    // Map 3D positions to mesh vertices using spatial quantization
    let mut vert_lut: HashMap<[i32; 3], VertexKey> = HashMap::new();

    for loop_id in &face_outer_loops {
        let oe_list = match edge_loops.get(loop_id) {
            Some(l) => l,
            None => continue,
        };

        let mut face_vks = Vec::new();

        for &oe_id in oe_list {
            let (ec_id, forward) = match oriented_edges.get(&oe_id) {
                Some(oe) => *oe,
                None => (oe_id, true),
            };

            let (vp_start, vp_end) = match edge_curves.get(&ec_id) {
                Some(ec) => *ec,
                None => continue,
            };

            let chosen_vp = if forward { vp_start } else { vp_end };
            let cp_id = match vertex_points.get(&chosen_vp) {
                Some(&cp) => cp,
                None => chosen_vp,
            };

            let pt = match cartesian_points.get(&cp_id) {
                Some(&p) => p,
                None => continue,
            };

            let key = [
                (pt.x * 10000.0).round() as i32,
                (pt.y * 10000.0).round() as i32,
                (pt.z * 10000.0).round() as i32,
            ];

            let vk = *vert_lut.entry(key).or_insert_with(|| mesh.add_vertex(pt));

            if face_vks.last() != Some(&vk) {
                face_vks.push(vk);
            }
        }

        // Remove closing duplicate if any
        if face_vks.len() > 3 && face_vks.first() == face_vks.last() {
            face_vks.pop();
        }

        if face_vks.len() >= 3 {
            let _ = mesh.add_face(&face_vks);
        }
    }

    let _ = mesh.weld_vertices_by_distance(0.0001);
    mesh.update_face_weighted_normals(true);
    Ok(mesh)
}

fn parse_coordinate_triplet(s: &str) -> Option<Vec3> {
    let open_paren = s.find('(')?;
    let coords_start = s[open_paren + 1..].find('(')? + open_paren + 1;
    let coords_end = s[coords_start..].find(')')? + coords_start;
    let parts: Vec<&str> = s[coords_start + 1..coords_end].split(',').collect();
    if parts.len() >= 3 {
        let x: f32 = parts[0].trim().parse().ok()?;
        let y: f32 = parts[1].trim().parse().ok()?;
        let z: f32 = parts[2].trim().parse().ok()?;
        Some(Vec3::new(x, y, z))
    } else {
        None
    }
}

fn parse_single_ref(s: &str) -> Option<usize> {
    let hash = s.find('#')?;
    let rest = &s[hash + 1..];
    let end = rest
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(rest.len());
    rest[..end].parse().ok()
}

fn parse_double_ref(s: &str) -> Option<(usize, usize)> {
    let h1 = s.find('#')?;
    let rest1 = &s[h1 + 1..];
    let end1 = rest1.find(|c: char| !c.is_ascii_digit())?;
    let r1: usize = rest1[..end1].parse().ok()?;

    let h2 = rest1[end1..].find('#')? + end1;
    let rest2 = &rest1[h2 + 1..];
    let end2 = rest2
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(rest2.len());
    let r2: usize = rest2[..end2].parse().ok()?;

    Some((r1, r2))
}

fn parse_oriented_edge_ref(s: &str) -> Option<(usize, bool)> {
    let hash = s.find('#')?;
    let rest = &s[hash + 1..];
    let end = rest.find(|c: char| !c.is_ascii_digit())?;
    let edge_ref: usize = rest[..end].parse().ok()?;
    let forward = !s.contains(".F.");
    Some((edge_ref, forward))
}

fn parse_ref_list(s: &str) -> Vec<usize> {
    let mut refs = Vec::new();
    let mut cur = s;
    while let Some(hash) = cur.find('#') {
        let rest = &cur[hash + 1..];
        let end = rest
            .find(|c: char| !c.is_ascii_digit())
            .unwrap_or(rest.len());
        if let Ok(num) = rest[..end].parse::<usize>() {
            refs.push(num);
        }
        cur = &rest[end..];
    }
    refs
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::Primitives;

    #[test]
    fn test_step_export_and_import_roundtrip_cube() {
        let cube = Primitives::create_cube(2.0);
        let mut step_bytes = Vec::new();
        cube.export_step(&mut step_bytes, "RoundtripCube")
            .expect("export step");

        let step_str = String::from_utf8(step_bytes).expect("valid utf8");
        let imported = import_step_str(&step_str).expect("import step");

        assert!(!imported.vertices.is_empty());
        assert_eq!(imported.face_count(), 6);
    }
}

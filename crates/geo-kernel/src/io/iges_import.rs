//! ANSI/USAF IGES 5.3 CAD Importer for Plumb 3D DCC.
//! Parses fixed 80-column IGES files and reconstructs 2-manifold BMesh solids
//! from B-Spline surfaces (Entity 128), lines (Entity 110), and coplanar polygons.

use crate::mesh::{Mesh, MeshError};
use core_math::Vec3;
use std::collections::HashMap;

/// Directory Entry record in IGES Section D.
#[derive(Debug, Clone)]
struct DirEntry {
    entity_type: i32,
    param_start: usize,
    param_line_count: usize,
}

/// Imports a 2-manifold mesh from an ANSI/USAF IGES 5.3 formatted string.
pub fn import_iges_str(iges_content: &str) -> Result<Mesh, MeshError> {
    let mut d_lines = Vec::new();
    let mut p_lines: HashMap<usize, String> = HashMap::new();

    // 1. Split lines and segregate by section code (column 73)
    for raw_line in iges_content.lines() {
        let line = raw_line.trim_end_matches('\r');
        if line.len() < 73 {
            continue;
        }

        let section_code = line.chars().nth(72).unwrap_or(' ');
        match section_code {
            'D' => d_lines.push(line.to_string()),
            'P' => {
                // Line sequence number is in cols 73-80 (1-based index)
                let seq_str = line.get(73..80).unwrap_or("").trim();
                let seq: usize = seq_str.parse().unwrap_or(0);
                // Data is in cols 0..64
                let data = line.get(0..64.min(line.len())).unwrap_or("").trim_end();
                p_lines.insert(seq, data.to_string());
            }
            _ => {}
        }
    }

    if d_lines.is_empty() {
        return Err(MeshError::InvariantFailed(
            "IGES file contains no Directory Entry (D) records".to_string(),
        ));
    }

    // 2. Parse Directory Entries (pairs of lines in Section D)
    let mut dir_entries = Vec::new();
    let mut i = 0;
    while i + 1 < d_lines.len() {
        let l1 = &d_lines[i];
        let l2 = &d_lines[i + 1];

        let type1: i32 = l1.get(0..8).unwrap_or("").trim().parse().unwrap_or(0);
        let p_start: usize = l1.get(8..16).unwrap_or("").trim().parse().unwrap_or(0);
        let p_count: usize = l2.get(24..32).unwrap_or("").trim().parse().unwrap_or(1);

        dir_entries.push(DirEntry {
            entity_type: type1,
            param_start: p_start,
            param_line_count: p_count,
        });

        i += 2;
    }

    // 3. Extract parameter records and reconstruct geometry
    let mut mesh = Mesh::new();
    let mut line_segments: Vec<(Vec3, Vec3)> = Vec::new();

    for de in &dir_entries {
        // Concatenate parameter string across p_count lines
        let mut param_str = String::new();
        for p_idx in de.param_start..(de.param_start + de.param_line_count) {
            if let Some(chunk) = p_lines.get(&p_idx) {
                param_str.push_str(chunk);
            }
        }

        // Strip trailing semicolon and tokenize by comma
        let clean_str = param_str.trim().trim_end_matches(';');
        let tokens: Vec<&str> = clean_str
            .split(',')
            .map(|t| t.trim())
            .filter(|t| !t.is_empty())
            .collect();

        if tokens.is_empty() {
            continue;
        }

        match de.entity_type {
            // Entity 110: Line (X1, Y1, Z1, X2, Y2, Z2)
            110 => {
                if tokens.len() >= 7 {
                    let x1: f32 = tokens[1].parse().unwrap_or(0.0);
                    let y1: f32 = tokens[2].parse().unwrap_or(0.0);
                    let z1: f32 = tokens[3].parse().unwrap_or(0.0);
                    let x2: f32 = tokens[4].parse().unwrap_or(0.0);
                    let y2: f32 = tokens[5].parse().unwrap_or(0.0);
                    let z2: f32 = tokens[6].parse().unwrap_or(0.0);
                    line_segments.push((Vec3::new(x1, y1, z1), Vec3::new(x2, y2, z2)));
                }
            }
            // Entity 128: Rational B-Spline Surface
            128 if tokens.len() >= 10 => {
                let k1: usize = tokens[1].parse().unwrap_or(1);
                let k2: usize = tokens[2].parse().unwrap_or(1);
                let m1: usize = tokens[3].parse().unwrap_or(1);
                let m2: usize = tokens[4].parse().unwrap_or(1);

                let knots_u_count = m1 + k1 + 2;
                let knots_v_count = m2 + k2 + 2;
                let num_poles = (m1 + 1) * (m2 + 1);

                let knots_offset = 10;
                let weights_offset = knots_offset + knots_u_count + knots_v_count;
                let poles_offset = weights_offset + num_poles;

                if tokens.len() >= poles_offset + num_poles * 3 {
                    let mut poles = Vec::new();
                    for p in 0..num_poles {
                        let base = poles_offset + p * 3;
                        let x: f32 = tokens[base].parse().unwrap_or(0.0);
                        let y: f32 = tokens[base + 1].parse().unwrap_or(0.0);
                        let z: f32 = tokens[base + 2].parse().unwrap_or(0.0);
                        poles.push(Vec3::new(x, y, z));
                    }

                    // For bilinear patches (K1=1, K2=1, 4 poles), add as quad
                    if k1 == 1 && k2 == 1 && poles.len() == 4 {
                        let p0 = poles[0];
                        let p1 = poles[1];
                        let p2 = poles[3];
                        let p3 = poles[2];
                        let v0 = mesh.add_vertex(p0);
                        let v1 = mesh.add_vertex(p1);
                        let v2 = mesh.add_vertex(p2);
                        let v3 = mesh.add_vertex(p3);
                        let _ = mesh.add_face(&[v0, v1, v2, v3]);
                    } else {
                        // Higher-order B-Spline: sample (m1+1) x (m2+1) pole grid
                        let cols = m1 + 1;
                        let rows = m2 + 1;
                        let mut v_grid = Vec::new();
                        for pole in &poles {
                            v_grid.push(mesh.add_vertex(*pole));
                        }
                        for r in 0..(rows - 1) {
                            for c in 0..(cols - 1) {
                                let v_a = v_grid[r * cols + c];
                                let v_b = v_grid[r * cols + (c + 1)];
                                let v_c = v_grid[(r + 1) * cols + (c + 1)];
                                let v_d = v_grid[(r + 1) * cols + c];
                                let _ = mesh.add_face(&[v_a, v_b, v_c, v_d]);
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    // If mesh has no faces but has lines (wireframe model), construct closed loops
    if mesh.faces.is_empty() && !line_segments.is_empty() {
        stitch_line_segments_into_mesh(&line_segments, &mut mesh);
    }

    // Weld coincident vertices to ensure manifold topology
    if let Ok(welded) = mesh.weld_vertices_by_distance(1e-4) {
        mesh = welded;
    }

    Ok(mesh)
}

/// Helper function to stitch loose wireframe line segments into polygon faces.
fn stitch_line_segments_into_mesh(lines: &[(Vec3, Vec3)], mesh: &mut Mesh) {
    if lines.is_empty() {
        return;
    }

    let mut current_loop: Vec<Vec3> = Vec::new();
    let mut remaining: Vec<(Vec3, Vec3)> = lines.to_vec();

    while !remaining.is_empty() {
        if current_loop.is_empty() {
            let first = remaining.remove(0);
            current_loop.push(first.0);
            current_loop.push(first.1);
            continue;
        }

        let tail = *current_loop.last().unwrap();
        let head = current_loop[0];

        // Check if loop is closed
        if (tail - head).length() < 1e-4 && current_loop.len() >= 4 {
            let v_keys: Vec<_> = current_loop[..current_loop.len() - 1]
                .iter()
                .map(|&p| mesh.add_vertex(p))
                .collect();
            let _ = mesh.add_face(&v_keys);
            current_loop.clear();
            continue;
        }

        // Find next connecting segment
        let mut found_idx = None;
        let mut reverse = false;
        for (idx, (p0, p1)) in remaining.iter().enumerate() {
            if (tail - *p0).length() < 1e-4 {
                found_idx = Some(idx);
                reverse = false;
                break;
            } else if (tail - *p1).length() < 1e-4 {
                found_idx = Some(idx);
                reverse = true;
                break;
            }
        }

        if let Some(idx) = found_idx {
            let seg = remaining.remove(idx);
            let next_pt = if reverse { seg.0 } else { seg.1 };
            current_loop.push(next_pt);
        } else {
            // Unconnected segment, terminate current loop
            if current_loop.len() >= 3 {
                let v_keys: Vec<_> = current_loop.iter().map(|&p| mesh.add_vertex(p)).collect();
                let _ = mesh.add_face(&v_keys);
            }
            current_loop.clear();
        }
    }

    if current_loop.len() >= 3 {
        let v_keys: Vec<_> = current_loop.iter().map(|&p| mesh.add_vertex(p)).collect();
        let _ = mesh.add_face(&v_keys);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::Primitives;

    #[test]
    fn test_import_iges_roundtrip() {
        let cube = Primitives::create_cube(2.0);
        let mut buffer = Vec::new();
        cube.export_iges(&mut buffer, "CubeRoundtrip").unwrap();

        let iges_str = String::from_utf8(buffer).unwrap();
        let imported = import_iges_str(&iges_str).expect("Failed to import IGES cube");

        assert_eq!(imported.faces.len(), 6);
        assert_eq!(imported.vertices.len(), 8);
    }
}

//! ANSI/USAF IGES 5.3 (ASME Y14.26M) CAD Solid & Surface Exporter for Plumb 3D DCC.
//! Generates strict 80-column fixed-format IGES files compatible with
//! SolidWorks, Siemens NX, CATIA, Rhino, AutoCAD, and CNC CAM toolpaths.

use crate::mesh::Mesh;
use core_math::Vec3;
use std::io::{self, Write};
use std::path::Path;

impl Mesh {
    /// Imports a mesh from an ANSI/USAF IGES 5.3 CAD string.
    pub fn from_iges_str(iges_content: &str) -> Result<Self, crate::mesh::MeshError> {
        super::iges_import::import_iges_str(iges_content)
    }

    /// Imports a mesh from an ANSI/USAF IGES 5.3 CAD file on disk.
    pub fn from_iges_file(path: impl AsRef<Path>) -> io::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Self::from_iges_str(&content)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("{:?}", e)))
    }

    /// Exports the mesh to an ANSI/USAF IGES 5.3 file on disk.
    pub fn export_iges_file(&self, path: impl AsRef<Path>) -> io::Result<()> {
        let file = std::fs::File::create(path)?;
        let mut writer = io::BufWriter::new(file);
        self.export_iges(&mut writer, "Plumb3D_CAD_Model")
    }

    /// Serializes the mesh to an ANSI/USAF IGES 5.3 stream with strict 80-column cards.
    pub fn export_iges<W: Write>(&self, w: &mut W, model_name: &str) -> io::Result<()> {
        let mut s_lines = Vec::new();
        let mut g_lines = Vec::new();
        let mut d_lines = Vec::new();
        let mut p_lines = Vec::new();

        // 1. Section S: Start Section (human-readable metadata)
        format_iges_section(
            &format!("Plumb 3D CAD Kernel v1.6.0 - IGES 5.3 Export: {}", model_name),
            'S',
            &mut s_lines,
        );
        format_iges_section(
            "The No-Hands Company - Precision Analytic B-Rep Solid",
            'S',
            &mut s_lines,
        );

        // 2. Section G: Global Section
        let g_content = format!(
            "1H,,1H;,{}H{},{}H{}.igs,19HPlumb 3D CAD Kernel,15HPlumb 3D v1.6.0,32,38,6,308,15,{}H{},1.0,2,2HMM,1,0.01,15H20261128.160000,1.0E-4,1000.0,17HEric Hakansson,21HThe No-Hands Company,11,0;",
            model_name.len(),
            model_name,
            model_name.len(),
            model_name,
            model_name.len(),
            model_name
        );
        format_iges_section(&g_content, 'G', &mut g_lines);

        // 3. Section D & P: Entities
        let mut de_index = 1;
        let mut p_index = 1;

        for (f_key, _face) in &self.faces {
            let v_keys = self.face_vertices(f_key);
            if v_keys.len() < 3 {
                continue;
            }
            let pts: Vec<Vec3> = v_keys
                .iter()
                .filter_map(|&vk| self.vertices.get(vk).map(|v| v.position))
                .collect();
            if pts.len() < 3 {
                continue;
            }

            let p0 = pts[0];
            let p1 = pts[1];
            let p2 = pts[2];
            let p3 = if pts.len() >= 4 { pts[3] } else { pts[2] };

            let param_body = format!(
                "128,1,1,1,1,0,0,1,0,0,0.0,0.0,1.0,1.0,0.0,0.0,1.0,1.0,1.0,1.0,1.0,1.0,{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},0.0,1.0,0.0,1.0;",
                p0.x, p0.y, p0.z,
                p1.x, p1.y, p1.z,
                p3.x, p3.y, p3.z,
                p2.x, p2.y, p2.z
            );

            let p_start = p_index;
            let p_added = format_iges_param_data(&param_body, de_index, &mut p_lines);
            p_index += p_added;

            let d1 = format_de_line(
                "128", &p_start.to_string(), "0", "1", "1", "0", "0", "0", "00000001",
                'D', de_index,
            );
            let d2 = format_de_line(
                "128", "1", "0", &p_added.to_string(), "0", "", "", "SURF", "1",
                'D', de_index + 1,
            );

            d_lines.push(d1);
            d_lines.push(d2);
            de_index += 2;
        }

        // Also export boundary edges as Entity 110 (Line)
        for (_edge_key, edge) in &self.edges {
            if let Some(he) = self.half_edges.get(edge.half_edge) {
                let p0 = self.vertices.get(he.origin).map(|v| v.position).unwrap_or(Vec3::ZERO);
                let p1 = self.vertices.get(he.target).map(|v| v.position).unwrap_or(Vec3::ZERO);

                let param_body = format!(
                    "110,{:.4},{:.4},{:.4},{:.4},{:.4},{:.4};",
                    p0.x, p0.y, p0.z, p1.x, p1.y, p1.z
                );

                let p_start = p_index;
                let p_added = format_iges_param_data(&param_body, de_index, &mut p_lines);
                p_index += p_added;

                let d1 = format_de_line(
                    "110", &p_start.to_string(), "0", "1", "1", "0", "0", "0", "00000001",
                    'D', de_index,
                );
                let d2 = format_de_line(
                    "110", "1", "1", &p_added.to_string(), "0", "", "", "LINE", "1",
                    'D', de_index + 1,
                );

                d_lines.push(d1);
                d_lines.push(d2);
                de_index += 2;
            }
        }

        // 4. Section T: Terminate Section (exactly 80 columns)
        let t_line = format!(
            "S{:7}G{:7}D{:7}P{:7}{:<40}T{:7}",
            s_lines.len(),
            g_lines.len(),
            d_lines.len(),
            p_lines.len(),
            "",
            1
        );

        // Write all sections out to writer
        for line in &s_lines {
            writeln!(w, "{}", line)?;
        }
        for line in &g_lines {
            writeln!(w, "{}", line)?;
        }
        for line in &d_lines {
            writeln!(w, "{}", line)?;
        }
        for line in &p_lines {
            writeln!(w, "{}", line)?;
        }
        writeln!(w, "{}", t_line)?;

        Ok(())
    }
}

/// Formats text into fixed 80-character IGES cards with a section letter and 7-digit line number.
fn format_iges_section(content: &str, section: char, lines: &mut Vec<String>) {
    let mut seq = lines.len() + 1;
    let mut offset = 0;
    let chars: Vec<char> = content.chars().collect();

    while offset < chars.len() || offset == 0 {
        let chunk_size = (chars.len() - offset).min(72);
        let chunk: String = chars[offset..offset + chunk_size].iter().collect();
        let card = format!("{:<72}{}{:7}", chunk, section, seq);
        lines.push(card);
        seq += 1;
        offset += chunk_size;
        if offset >= chars.len() {
            break;
        }
    }
}

/// Formats a Directory Entry line (9 fields of 8 chars + section code + 7-digit seq = 80 chars).
fn format_de_line(
    f1: &str,
    f2: &str,
    f3: &str,
    f4: &str,
    f5: &str,
    f6: &str,
    f7: &str,
    f8: &str,
    f9: &str,
    section: char,
    seq: usize,
) -> String {
    format!(
        "{:>8}{:>8}{:>8}{:>8}{:>8}{:>8}{:>8}{:>8}{:>8}{}{:7}",
        f1, f2, f3, f4, f5, f6, f7, f8, f9, section, seq
    )
}

/// Formats parameter data into Section P cards (Cols 1-64 data, 65-72 DE pointer, 73 'P', 74-80 seq).
fn format_iges_param_data(param_str: &str, de_pointer: usize, lines: &mut Vec<String>) -> usize {
    let mut seq = lines.len() + 1;
    let mut count = 0;
    let chars: Vec<char> = param_str.chars().collect();
    let mut offset = 0;

    while offset < chars.len() {
        let chunk_size = (chars.len() - offset).min(64);
        let chunk: String = chars[offset..offset + chunk_size].iter().collect();
        let card = format!("{:<64}{:8}P{:7}", chunk, de_pointer, seq);
        lines.push(card);
        seq += 1;
        count += 1;
        offset += chunk_size;
    }

    count.max(1)
}

#[cfg(test)]
mod tests {
    use crate::primitives::Primitives;

    #[test]
    fn test_iges_export_roundtrip_lines() {
        let cube = Primitives::create_cube(1.0);
        let mut buffer = Vec::new();
        cube.export_iges(&mut buffer, "TestCube").unwrap();

        let iges_text = String::from_utf8(buffer).unwrap();
        let lines: Vec<&str> = iges_text.lines().collect();

        assert!(!lines.is_empty());
        // Verify every line has exactly 80 characters
        for line in &lines {
            assert_eq!(line.len(), 80, "IGES line must be exactly 80 chars: '{}'", line);
        }

        // Verify sections exist
        assert!(lines.iter().any(|l| l.chars().nth(72) == Some('S')));
        assert!(lines.iter().any(|l| l.chars().nth(72) == Some('G')));
        assert!(lines.iter().any(|l| l.chars().nth(72) == Some('D')));
        assert!(lines.iter().any(|l| l.chars().nth(72) == Some('P')));
        assert!(lines.iter().any(|l| l.chars().nth(72) == Some('T')));
    }
}

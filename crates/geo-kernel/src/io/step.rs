//! ISO 10303-21 (STEP AP214) CAD B-Rep Exporter for Plumb 3D DCC.
//! Exports 2-manifold polygon meshes directly as solid B-Rep CAD geometry
//! compatible with SolidWorks, Siemens NX, CATIA, FreeCAD, and CNC CAM toolpaths.

use crate::mesh::{Mesh, VertexKey};
use core_math::Vec3;
use std::collections::HashMap;
use std::io::{self, Write};
use std::path::Path;

impl Mesh {
    /// Imports a mesh from a standard ISO 10303-21 STEP CAD string.
    pub fn from_step_str(step_content: &str) -> Result<Self, crate::mesh::MeshError> {
        super::step_import::import_step_str(step_content)
    }

    /// Imports a mesh from a standard ISO 10303-21 STEP CAD file on disk.
    pub fn from_step_file(path: impl AsRef<Path>) -> io::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Self::from_step_str(&content)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("{:?}", e)))
    }

    /// Exports the mesh to a standard ISO 10303-21 STEP AP214 solid CAD file.
    pub fn export_step_file(&self, path: impl AsRef<Path>) -> io::Result<()> {
        let file = std::fs::File::create(path)?;
        let mut writer = io::BufWriter::new(file);
        self.export_step(&mut writer, "Plumb3D_Solid_Model")
    }

    /// Serializes the mesh to an ISO 10303-21 STEP B-Rep stream.
    pub fn export_step<W: Write>(&self, w: &mut W, model_name: &str) -> io::Result<()> {
        writeln!(w, "ISO-10303-21;")?;
        writeln!(w, "HEADER;")?;
        writeln!(
            w,
            "FILE_DESCRIPTION(('Plumb 3D CAD Solid B-Rep Geometry'),'2;1');"
        )?;
        writeln!(
            w,
            "FILE_NAME('{}.step','2026-11-25T18:30:00',('Plumb 3D Designer'),('The No-Hands Company'),'Plumb 3D v1.0.0','Plumb 3D DCC Engine','');",
            model_name
        )?;
        writeln!(
            w,
            "FILE_SCHEMA(('AUTOMOTIVE_DESIGN {{ 1 0 10303 214 1 1 1 1 }}'));"
        )?;
        writeln!(w, "ENDSEC;")?;
        writeln!(w, "DATA;")?;

        let mut id = 1usize;
        let mut next_id = || {
            let cur = id;
            id += 1;
            cur
        };

        // Standard Application Context
        let app_ctx = next_id();
        writeln!(
            w,
            "#{}=APPLICATION_CONTEXT('core data for automotive mechanical design');",
            app_ctx
        )?;
        let app_proto = next_id();
        writeln!(
            w,
            "#{}=APPLICATION_PROTOCOL_DEFINITION('international standard','automotive_design',2000,#{});",
            app_proto, app_ctx
        )?;

        // Length & Angle Units Context
        let len_unit = next_id();
        writeln!(
            w,
            "#{}=(LENGTH_UNIT()NAMED_UNIT(*)SI_UNIT(.MILLI.,.METRE.));",
            len_unit
        )?;
        let plane_ang_unit = next_id();
        writeln!(
            w,
            "#{}=(NAMED_UNIT(*)PLANE_ANGLE_UNIT()SI_UNIT($,.RADIAN.));",
            plane_ang_unit
        )?;
        let uncert = next_id();
        let global_ctx = next_id();
        writeln!(
            w,
            "#{}=UNCERTAINTY_MEASURE_WITH_UNIT(LENGTH_MEASURE(1.E-05),#{},'DISTANCE_ACCURACY_VALUE','confusion accuracy');",
            uncert, len_unit
        )?;
        writeln!(
            w,
            "#{}=(GEOMETRIC_REPRESENTATION_CONTEXT(3)GLOBAL_UNCERTAINTY_ASSIGNED_CONTEXT((#{}))GLOBAL_UNIT_ASSIGNED_CONTEXT((#{},#{}))REPRESENTATION_CONTEXT('Context #1','3D'));",
            global_ctx, uncert, len_unit, plane_ang_unit
        )?;

        // Map vertices to STEP CARTESIAN_POINT and VERTEX_POINT entities
        let mut vert_pt_ids: HashMap<VertexKey, (usize, usize)> = HashMap::new();
        for (vk, v) in &self.vertices {
            let cp_id = next_id();
            // Scale meters to millimeters for standard CAD conventions
            let x = v.position.x * 1000.0;
            let y = v.position.y * 1000.0;
            let z = v.position.z * 1000.0;
            writeln!(
                w,
                "#{}=CARTESIAN_POINT('',({:.6},{:.6},{:.6}));",
                cp_id, x, y, z
            )?;
            let vp_id = next_id();
            writeln!(w, "#{}=VERTEX_POINT('',#{});", vp_id, cp_id)?;
            vert_pt_ids.insert(vk, (cp_id, vp_id));
        }

        // Write Direction vectors for Face Planes
        let mut face_ids = Vec::new();
        for (fk, face) in &self.faces {
            let v_keys = self.face_vertices(fk);
            if v_keys.len() < 3 {
                continue;
            }

            // Normal and Origin
            let centroid = self.face_centroid(fk) * 1000.0;
            let norm = face.normal.normalize_or_zero();

            let norm_dir_id = next_id();
            writeln!(
                w,
                "#{}=DIRECTION('',({:.6},{:.6},{:.6}));",
                norm_dir_id, norm.x, norm.y, norm.z
            )?;

            // Perpendicular axis for placement
            let ref_up = if norm.y.abs() < 0.9 { Vec3::Y } else { Vec3::Z };
            let ref_dir = norm.cross(ref_up).normalize_or_zero();
            let ref_dir_id = next_id();
            writeln!(
                w,
                "#{}=DIRECTION('',({:.6},{:.6},{:.6}));",
                ref_dir_id, ref_dir.x, ref_dir.y, ref_dir.z
            )?;

            let axis_pt_id = next_id();
            writeln!(
                w,
                "#{}=CARTESIAN_POINT('',({:.6},{:.6},{:.6}));",
                axis_pt_id, centroid.x, centroid.y, centroid.z
            )?;

            let axis2_id = next_id();
            writeln!(
                w,
                "#{}=AXIS2_PLACEMENT_3D('',#{},#{},#{});",
                axis2_id, axis_pt_id, norm_dir_id, ref_dir_id
            )?;

            let plane_id = next_id();
            writeln!(w, "#{}=PLANE('',#{});", plane_id, axis2_id)?;

            // Construct edges and EDGE_LOOP for this face
            let mut oriented_edge_ids = Vec::new();
            for i in 0..v_keys.len() {
                let v_start = v_keys[i];
                let v_end = v_keys[(i + 1) % v_keys.len()];

                let (_, vp_start) = vert_pt_ids[&v_start];
                let (_, vp_end) = vert_pt_ids[&v_end];

                let p_start = self.vertices[v_start].position * 1000.0;
                let p_end = self.vertices[v_end].position * 1000.0;
                let edge_vec = (p_end - p_start).normalize_or_zero();

                let line_dir_id = next_id();
                writeln!(
                    w,
                    "#{}=DIRECTION('',({:.6},{:.6},{:.6}));",
                    line_dir_id, edge_vec.x, edge_vec.y, edge_vec.z
                )?;

                let line_vec_id = next_id();
                writeln!(w, "#{}=VECTOR('',#{},1.0);", line_vec_id, line_dir_id)?;

                let line_pt_id = next_id();
                writeln!(
                    w,
                    "#{}=CARTESIAN_POINT('',({:.6},{:.6},{:.6}));",
                    line_pt_id, p_start.x, p_start.y, p_start.z
                )?;

                let line_id = next_id();
                writeln!(w, "#{}=LINE('',#{},#{});", line_id, line_pt_id, line_vec_id)?;

                let edge_curve_id = next_id();
                writeln!(
                    w,
                    "#{}=EDGE_CURVE('',#{},#{},#{},.T.);",
                    edge_curve_id, vp_start, vp_end, line_id
                )?;

                let oriented_edge_id = next_id();
                writeln!(
                    w,
                    "#{}=ORIENTED_EDGE('',*,*,#{},.T.);",
                    oriented_edge_id, edge_curve_id
                )?;

                oriented_edge_ids.push(oriented_edge_id);
            }

            let edge_list_str: Vec<String> = oriented_edge_ids
                .iter()
                .map(|eid| format!("#{}", eid))
                .collect();
            let edge_loop_id = next_id();
            writeln!(
                w,
                "#{}=EDGE_LOOP('',({}));",
                edge_loop_id,
                edge_list_str.join(",")
            )?;

            let face_bound_id = next_id();
            writeln!(
                w,
                "#{}=FACE_BOUND('',#{},.T.);",
                face_bound_id, edge_loop_id
            )?;

            let adv_face_id = next_id();
            writeln!(
                w,
                "#{}=ADVANCED_FACE('',(#{})#{},.T.);",
                adv_face_id, face_bound_id, plane_id
            )?;

            face_ids.push(adv_face_id);
        }

        // Closed Shell & Manifold Solid B-Rep
        let face_list_str: Vec<String> = face_ids.iter().map(|fid| format!("#{}", fid)).collect();
        let shell_id = next_id();
        writeln!(
            w,
            "#{}=CLOSED_SHELL('',({}));",
            shell_id,
            face_list_str.join(",")
        )?;

        let solid_id = next_id();
        writeln!(
            w,
            "#{}=MANIFOLD_SOLID_BREP('{}',#{});",
            solid_id, model_name, shell_id
        )?;

        // Top-level Shape Definition Representation
        let shape_rep_id = next_id();
        writeln!(
            w,
            "#{}=ADVANCED_BREP_SHAPE_REPRESENTATION('{}',(#{},#{}),#{});",
            shape_rep_id, model_name, solid_id, app_ctx, global_ctx
        )?;

        writeln!(w, "ENDSEC;")?;
        writeln!(w, "END-ISO-10303-21;")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::primitives::Primitives;

    #[test]
    fn test_export_step_cube() {
        let cube = Primitives::create_cube(2.0);
        let mut buffer = Vec::new();
        cube.export_step(&mut buffer, "TestCubeSolid")
            .expect("export step");

        let content = String::from_utf8(buffer).expect("valid utf8");
        assert!(content.contains("ISO-10303-21;"));
        assert!(content.contains("MANIFOLD_SOLID_BREP('TestCubeSolid'"));
        assert!(content.contains("CLOSED_SHELL("));
        assert!(content.contains("ADVANCED_FACE("));
        assert!(content.contains("EDGE_LOOP("));
        assert!(content.contains("CARTESIAN_POINT("));
        assert!(content.contains("END-ISO-10303-21;"));
    }
}

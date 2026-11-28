use crate::mesh::Mesh;
use std::collections::HashMap;
use std::fmt::Write as FmtWrite;
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;

impl Mesh {
    /// Serializes the mesh to Pixar Universal Scene Description ASCII format (.usda)
    /// compatible with Pixar USD, Autodesk Maya, SideFX Houdini, Unreal Engine 5, and Omniverse.
    pub fn export_usda(&self, mesh_name: &str) -> String {
        let clean_name = if mesh_name.is_empty() {
            "PlumbMesh"
        } else {
            mesh_name
        };

        let mut out = String::with_capacity(32 * 1024);

        // 1. Stage Header and Metadata
        writeln!(&mut out, "#usda 1.0").unwrap();
        writeln!(&mut out, "(").unwrap();
        writeln!(&mut out, "    defaultPrim = \"World\"").unwrap();
        writeln!(&mut out, "    doc = \"Exported by Plumb 3D Studio\"").unwrap();
        writeln!(&mut out, "    metersPerUnit = 1.0").unwrap();
        writeln!(&mut out, "    upAxis = \"Y\"").unwrap();
        writeln!(&mut out, ")").unwrap();
        writeln!(&mut out).unwrap();

        // 2. World Root Prim
        writeln!(&mut out, "def Xform \"World\"").unwrap();
        writeln!(&mut out, "{{").unwrap();

        // 3. Materials Scope (UsdPreviewSurface / MaterialX specification)
        writeln!(&mut out, "    def Scope \"Materials\"").unwrap();
        writeln!(&mut out, "    {{").unwrap();
        writeln!(&mut out, "        def Material \"DefaultMaterial\"").unwrap();
        writeln!(&mut out, "        {{").unwrap();
        writeln!(&mut out, "            def Shader \"PBRPreviewShader\"").unwrap();
        writeln!(&mut out, "            {{").unwrap();
        writeln!(
            &mut out,
            "                uniform token info:id = \"UsdPreviewSurface\""
        )
        .unwrap();
        writeln!(
            &mut out,
            "                color3f inputs:diffuseColor = (0.72, 0.73, 0.75)"
        )
        .unwrap();
        writeln!(&mut out, "                float inputs:metallic = 0.0").unwrap();
        writeln!(&mut out, "                float inputs:roughness = 0.5").unwrap();
        writeln!(&mut out, "                float inputs:opacity = 1.0").unwrap();
        writeln!(&mut out, "                token outputs:surface").unwrap();
        writeln!(&mut out, "            }}").unwrap();
        writeln!(&mut out, "            token outputs:surface.connect = </World/Materials/DefaultMaterial/PBRPreviewShader.outputs:surface>").unwrap();
        writeln!(&mut out, "        }}").unwrap();
        writeln!(&mut out, "    }}").unwrap();
        writeln!(&mut out).unwrap();

        // 4. Mesh Prim
        writeln!(&mut out, "    def Mesh \"{}\" (", clean_name).unwrap();
        writeln!(
            &mut out,
            "        prepend apiSchemas = [\"MaterialBindingAPI\"]"
        )
        .unwrap();
        writeln!(&mut out, "    )").unwrap();
        writeln!(&mut out, "    {{").unwrap();
        writeln!(
            &mut out,
            "        rel material:binding = </World/Materials/DefaultMaterial>"
        )
        .unwrap();
        writeln!(
            &mut out,
            "        uniform token subdivisionScheme = \"none\""
        )
        .unwrap();

        // Vertex Index Mapping
        let mut key_to_idx = HashMap::new();
        let mut points = Vec::with_capacity(self.vertices.len());
        for (i, (vk, v)) in self.vertices.iter().enumerate() {
            key_to_idx.insert(vk, i);
            points.push(v.position);
        }

        // Bounding Box Extent
        let aabb = self.compute_aabb();
        writeln!(
            &mut out,
            "        float3[] extent = [({}, {}, {}), ({}, {}, {})]",
            format_f32(aabb.min.x),
            format_f32(aabb.min.y),
            format_f32(aabb.min.z),
            format_f32(aabb.max.x),
            format_f32(aabb.max.y),
            format_f32(aabb.max.z)
        )
        .unwrap();

        // Points
        writeln!(&mut out, "        point3f[] points = [").unwrap();
        for (i, p) in points.iter().enumerate() {
            let comma = if i + 1 < points.len() { "," } else { "" };
            writeln!(
                &mut out,
                "            ({}, {}, {}){}",
                format_f32(p.x),
                format_f32(p.y),
                format_f32(p.z),
                comma
            )
            .unwrap();
        }
        writeln!(&mut out, "        ]").unwrap();

        // Face Vertex Counts and Indices
        let mut face_counts = Vec::with_capacity(self.faces.len());
        let mut face_indices = Vec::with_capacity(self.faces.len() * 4);
        let mut face_normals = Vec::with_capacity(self.faces.len() * 4);
        let mut face_uvs = Vec::with_capacity(self.faces.len() * 4);

        for fk in self.faces.keys() {
            let verts = self.face_vertices(fk);
            if verts.len() >= 3 {
                face_counts.push(verts.len());
                for &vk in &verts {
                    face_indices.push(key_to_idx[&vk]);
                    let v = &self.vertices[vk];
                    face_normals.push(v.normal);
                    face_uvs.push(v.uv);
                }
            }
        }

        // Output faceVertexCounts
        write!(&mut out, "        int[] faceVertexCounts = [").unwrap();
        for (i, count) in face_counts.iter().enumerate() {
            if i % 20 == 0 {
                write!(&mut out, "\n            ").unwrap();
            }
            write!(
                &mut out,
                "{}{}",
                count,
                if i + 1 < face_counts.len() { ", " } else { "" }
            )
            .unwrap();
        }
        writeln!(&mut out, "\n        ]").unwrap();

        // Output faceVertexIndices
        write!(&mut out, "        int[] faceVertexIndices = [").unwrap();
        for (i, idx) in face_indices.iter().enumerate() {
            if i % 20 == 0 {
                write!(&mut out, "\n            ").unwrap();
            }
            write!(
                &mut out,
                "{}{}",
                idx,
                if i + 1 < face_indices.len() { ", " } else { "" }
            )
            .unwrap();
        }
        writeln!(&mut out, "\n        ]").unwrap();

        // Output primvars:normals (faceVarying)
        writeln!(&mut out, "        normal3f[] primvars:normals = [").unwrap();
        for (i, n) in face_normals.iter().enumerate() {
            let comma = if i + 1 < face_normals.len() { "," } else { "" };
            writeln!(
                &mut out,
                "            ({}, {}, {}){}",
                format_f32(n.x),
                format_f32(n.y),
                format_f32(n.z),
                comma
            )
            .unwrap();
        }
        writeln!(&mut out, "        ] (").unwrap();
        writeln!(&mut out, "            interpolation = \"faceVarying\"").unwrap();
        writeln!(&mut out, "        )").unwrap();

        // Output primvars:st (UV texture coordinates)
        writeln!(&mut out, "        texCoord2f[] primvars:st = [").unwrap();
        for (i, uv) in face_uvs.iter().enumerate() {
            let comma = if i + 1 < face_uvs.len() { "," } else { "" };
            writeln!(
                &mut out,
                "            ({}, {}){}",
                format_f32(uv.x),
                format_f32(uv.y),
                comma
            )
            .unwrap();
        }
        writeln!(&mut out, "        ] (").unwrap();
        writeln!(&mut out, "            interpolation = \"faceVarying\"").unwrap();
        writeln!(&mut out, "        )").unwrap();

        // Close prims
        writeln!(&mut out, "    }}").unwrap();
        writeln!(&mut out, "}}").unwrap();

        out
    }

    /// Exports the mesh to a .usda file on disk
    pub fn export_usda_file(&self, path: impl AsRef<Path>, mesh_name: &str) -> io::Result<()> {
        let content = self.export_usda(mesh_name);
        let mut file = File::create(path)?;
        file.write_all(content.as_bytes())?;
        file.flush()
    }
}

fn format_f32(val: f32) -> String {
    if val.abs() < 1e-6 {
        "0.0".to_string()
    } else {
        let s = format!("{:.5}", val);
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

#[cfg(test)]
mod tests {
    use crate::primitives::Primitives;

    #[test]
    fn test_export_usda_cube() {
        let cube = Primitives::create_cube(2.0);
        let usda = cube.export_usda("CubeMesh");

        assert!(usda.starts_with("#usda 1.0"));
        assert!(usda.contains("def Xform \"World\""));
        assert!(usda.contains("def Mesh \"CubeMesh\""));
        assert!(usda.contains("UsdPreviewSurface"));
        assert!(usda.contains("point3f[] points = ["));
        assert!(usda.contains("int[] faceVertexCounts = ["));
        assert!(usda.contains("int[] faceVertexIndices = ["));
        assert!(usda.contains("normal3f[] primvars:normals = ["));
        assert!(usda.contains("texCoord2f[] primvars:st = ["));
    }
}

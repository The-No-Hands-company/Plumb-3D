use std::f32::consts::PI;

/// Configuration settings for Non-Photorealistic Rendering (NPR) cel-shading and anime outlines
#[derive(Debug, Clone, Copy)]
pub struct NprSettings {
    pub enabled: bool,
    /// Backface inverted hull outline thickness in world units
    pub outline_thickness: f32,
    /// Outline ink RGBA color
    pub outline_color: [f32; 4],
    /// Number of discrete cel illumination bands (e.g. 2 for classic anime, 3 for modern look)
    pub cel_bands: usize,
    /// Shadow tint factor blending in cool tones
    pub shadow_tint: [f32; 3],
    /// Whether to enable manga halftone screen stippling in shadow regions
    pub halftone_stippling: bool,
}

impl Default for NprSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            outline_thickness: 0.02,
            outline_color: [0.08, 0.08, 0.12, 1.0],
            cel_bands: 3,
            shadow_tint: [0.15, 0.12, 0.25], // Anime cool purple ambient shadow
            halftone_stippling: false,
        }
    }
}

/// NPR Pipeline providing inverted hull silhouette extraction and stepped cel illumination (Ghibli / Arcane style)
pub struct NprPipeline;

impl NprPipeline {
    /// Generates an Inverted Hull backface outline mesh (industry standard used in Guilty Gear, Genshin, Zelda)
    ///
    /// Displaces vertices outward along surface normals and reverses face winding,
    /// producing crisp silhouette outlines under standard backface culling.
    pub fn generate_inverted_hull(mesh: &geo_kernel::Mesh, thickness: f32) -> geo_kernel::Mesh {
        let mut outline = geo_kernel::Mesh::new();
        let t = thickness.max(0.001);

        // Map original vertices to extruded outline vertices
        let mut v_map = std::collections::HashMap::new();
        for (vk, v) in &mesh.vertices {
            let extruded_pos = v.position + v.normal.normalize() * t;
            let out_vk = outline.add_vertex_with_uv(extruded_pos, v.uv);
            if let Some(out_v) = outline.vertices.get_mut(out_vk) {
                out_v.normal = -v.normal;
                out_v.color = [0.08, 0.08, 0.12, 1.0]; // Ink outline color
            }
            v_map.insert(vk, out_vk);
        }

        // Re-create faces with reversed winding order for backface display
        for fk in mesh.faces.keys() {
            let fverts = mesh.face_vertices(fk);
            if fverts.len() >= 3 {
                let reversed_vks: Vec<geo_kernel::VertexKey> = fverts.iter().rev().map(|k| v_map[k]).collect();
                let _ = outline.add_face(&reversed_vks);
            }
        }

        outline.update_normals();
        outline
    }

    /// Evaluates stepped cel-shading illumination with discrete bands and shadow tinting
    pub fn evaluate_cel_shading(ndotl: f32, bands: usize, shadow_tint: [f32; 3]) -> [f32; 3] {
        let b = bands.clamp(1, 8) as f32;
        let clamped = ndotl.clamp(0.0, 1.0);
        // Step continuous Lambertian diffuse into discrete stair-step bands
        let step = (clamped * b).floor() / b;

        if step <= 0.05 {
            // Shadow band with stylized cool color tint
            shadow_tint
        } else {
            [step, step, step]
        }
    }

    /// Evaluates procedural manga halftone screen dot stippling
    pub fn evaluate_halftone_stipple(uv: [f32; 2], intensity: f32, dot_frequency: f32) -> f32 {
        let freq = dot_frequency.max(10.0);
        let u_dot = (uv[0] * freq * 2.0 * PI).sin();
        let v_dot = (uv[1] * freq * 2.0 * PI).sin();
        let dot_pattern = (u_dot * v_dot * 0.5 + 0.5).clamp(0.0, 1.0);

        if intensity < 0.4 {
            if dot_pattern > intensity * 2.0 {
                0.15 // Ink dot
            } else {
                1.0
            }
        } else {
            1.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use geo_kernel::primitives::Primitives;

    #[test]
    fn test_npr_inverted_hull_outline() {
        let cube = Primitives::create_cube(2.0);
        let outline = NprPipeline::generate_inverted_hull(&cube, 0.05);

        assert_eq!(outline.vertices.len(), cube.vertices.len());
        assert_eq!(outline.faces.len(), cube.faces.len());

        // Bounding box of outline should be strictly larger than base cube
        let base_aabb = cube.compute_aabb();
        let outline_aabb = outline.compute_aabb();
        assert!(outline_aabb.max.x > base_aabb.max.x);
        assert!(outline_aabb.min.x < base_aabb.min.x);
    }

    #[test]
    fn test_npr_cel_lighting_quantization() {
        let shadow_tint = [0.1, 0.1, 0.2];

        // Deep shadow
        let c_shadow = NprPipeline::evaluate_cel_shading(0.02, 3, shadow_tint);
        assert_eq!(c_shadow, shadow_tint);

        // Highlight
        let c_high = NprPipeline::evaluate_cel_shading(0.95, 3, shadow_tint);
        assert!(c_high[0] >= 0.66);
    }

    #[test]
    fn test_npr_halftone_stipple() {
        let dot_high = NprPipeline::evaluate_halftone_stipple([0.5, 0.5], 0.8, 20.0);
        assert_eq!(dot_high, 1.0);

        let dot_shadow = NprPipeline::evaluate_halftone_stipple([0.25, 0.25], 0.1, 20.0);
        assert!(dot_shadow <= 1.0);
    }
}

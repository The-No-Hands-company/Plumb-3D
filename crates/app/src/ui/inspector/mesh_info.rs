use crate::types::InteractionMode;
use geo_kernel::{EdgeKey, FaceKey, Mesh, VertexKey};
use std::collections::HashSet;

/// Precision Architectural CAD Measurement & Mesh Information Panel
pub fn draw_mesh_info(
    ui: &mut egui::Ui,
    interaction_mode: InteractionMode,
    selected_vertices: &HashSet<VertexKey>,
    selected_edges: &HashSet<EdgeKey>,
    selected_faces: &HashSet<FaceKey>,
    base_mesh: &Mesh,
) {
    ui.collapsing("📐 CAD Dimensions & Measurement", |ui| {
        // Overall Bounding Box & Element Metrics
        ui.horizontal(|ui| {
            ui.label(format!("Verts: {}", base_mesh.vertices.len()));
            ui.separator();
            ui.label(format!("Edges: {}", base_mesh.edges.len()));
            ui.separator();
            ui.label(format!("Faces: {}", base_mesh.faces.len()));
        });

        let dims = base_mesh.bounding_dimensions();
        let center = base_mesh.bounding_center();
        ui.label(format!("Bounding Box: [{:.3}m × {:.3}m × {:.3}m]", dims.x, dims.y, dims.z));
        ui.label(format!("Geometric Center: [{:.3}, {:.3}, {:.3}]", center.x, center.y, center.z));

        // 2-Manifold Topology Health Report
        let report = base_mesh.check_manifold();
        ui.horizontal(|ui| {
            if report.is_watertight {
                ui.label(egui::RichText::new("🟢 Watertight Solid").color(egui::Color32::from_rgb(100, 220, 140)));
            } else {
                ui.label(egui::RichText::new(format!("🟠 Open Shell ({} Holes)", report.boundary_edge_count)).color(egui::Color32::from_rgb(255, 170, 80)));
            }
            ui.separator();
            ui.label(format!("Euler χ: {} | Genus: {}", report.euler_characteristic, report.genus));
        });

        ui.separator();

        // Interactive Sub-Element CAD Caliper / Tape Measure
        if interaction_mode == InteractionMode::Edit {
            ui.label(egui::RichText::new("Precision Measurement:").strong().color(egui::Color32::from_rgb(255, 215, 100)));

            if selected_vertices.len() == 2 {
                let mut iter = selected_vertices.iter();
                let v0 = *iter.next().unwrap();
                let v1 = *iter.next().unwrap();
                if let (Some(vert0), Some(vert1)) = (base_mesh.vertices.get(v0), base_mesh.vertices.get(v1)) {
                    let p0 = vert0.position;
                    let p1 = vert1.position;
                    let delta = p1 - p0;
                    let dist = delta.length();
                    let dist_xz = (delta.x * delta.x + delta.z * delta.z).sqrt();
                    let incline_deg = delta.y.atan2(dist_xz).to_degrees();

                    ui.group(|ui| {
                        ui.label(egui::RichText::new(format!("📏 3D Distance: {:.4} m", dist)).strong().color(egui::Color32::from_rgb(100, 220, 160)));
                        ui.label(format!("ΔX: {:.4} m | ΔY: {:.4} m (Height) | ΔZ: {:.4} m", delta.x.abs(), delta.y.abs(), delta.z.abs()));
                        ui.label(format!("Ground Run (XZ): {:.4} m | Incline: {:.2}°", dist_xz, incline_deg));
                    });
                }
            } else if selected_vertices.len() == 3 {
                let v_list: Vec<_> = selected_vertices.iter().copied().collect();
                if let (Some(va), Some(vb), Some(vc)) = (
                    base_mesh.vertices.get(v_list[0]),
                    base_mesh.vertices.get(v_list[1]),
                    base_mesh.vertices.get(v_list[2]),
                ) {
                    let ba = va.position - vb.position;
                    let bc = vc.position - vb.position;
                    let len_prod = ba.length() * bc.length();
                    let angle_deg = if len_prod > 1e-6 {
                        (ba.dot(bc) / len_prod).clamp(-1.0, 1.0).acos().to_degrees()
                    } else {
                        0.0
                    };
                    ui.group(|ui| {
                        ui.label(egui::RichText::new(format!("📐 Vertex Angle (∠ABC): {:.2}°", angle_deg)).strong().color(egui::Color32::from_rgb(130, 200, 255)));
                        ui.label(format!("Leg 1 (BA): {:.4} m | Leg 2 (BC): {:.4} m", ba.length(), bc.length()));
                    });
                }
            } else if selected_edges.len() == 1 {
                let ek = *selected_edges.iter().next().unwrap();
                if let Some(edge) = base_mesh.edges.get(ek) {
                    let he = &base_mesh.half_edges[edge.half_edge];
                    let p0 = base_mesh.vertices[he.origin].position;
                    let p1 = base_mesh.vertices[he.target].position;
                    let len = (p1 - p0).length();
                    ui.group(|ui| {
                        ui.label(egui::RichText::new(format!("📏 Edge Length: {:.4} m", len)).strong().color(egui::Color32::from_rgb(100, 220, 160)));
                        let dy = (p1.y - p0.y).abs();
                        ui.label(format!("Vertical Rise (ΔY): {:.4} m", dy));
                    });
                }
            } else if selected_edges.len() == 2 {
                let e_keys: Vec<_> = selected_edges.iter().copied().collect();
                if let (Some(e0), Some(e1)) = (base_mesh.edges.get(e_keys[0]), base_mesh.edges.get(e_keys[1])) {
                    let he0 = &base_mesh.half_edges[e0.half_edge];
                    let p0a = base_mesh.vertices[he0.origin].position;
                    let p0b = base_mesh.vertices[he0.target].position;
                    let mid0 = (p0a + p0b) * 0.5;
                    let dir0 = (p0b - p0a).normalize_or_zero();

                    let he1 = &base_mesh.half_edges[e1.half_edge];
                    let p1a = base_mesh.vertices[he1.origin].position;
                    let p1b = base_mesh.vertices[he1.target].position;
                    let mid1 = (p1a + p1b) * 0.5;
                    let dir1 = (p1b - p1a).normalize_or_zero();

                    let angle_deg = dir0.dot(dir1).clamp(-1.0, 1.0).acos().to_degrees();
                    let midpoint_dist = (mid1 - mid0).length();

                    ui.group(|ui| {
                        ui.label(egui::RichText::new(format!("📐 Edge Angle: {:.2}°", angle_deg)).strong().color(egui::Color32::from_rgb(130, 200, 255)));
                        ui.label(format!("Midpoint Distance: {:.4} m", midpoint_dist));
                    });
                }
            } else if selected_faces.len() == 1 {
                let fk = *selected_faces.iter().next().unwrap();
                if let Some(face) = base_mesh.faces.get(fk) {
                    let v_keys = base_mesh.face_vertices(fk);
                    let mut perimeter = 0.0_f32;
                    let mut area = 0.0_f32;
                    if v_keys.len() >= 3 {
                        for i in 0..v_keys.len() {
                            let next_i = (i + 1) % v_keys.len();
                            let pa = base_mesh.vertices[v_keys[i]].position;
                            let pb = base_mesh.vertices[v_keys[next_i]].position;
                            perimeter += (pb - pa).length();
                        }
                        let p0 = base_mesh.vertices[v_keys[0]].position;
                        for i in 1..(v_keys.len() - 1) {
                            let p1 = base_mesh.vertices[v_keys[i]].position;
                            let p2 = base_mesh.vertices[v_keys[i + 1]].position;
                            area += ((p1 - p0).cross(p2 - p0)).length() * 0.5;
                        }
                    }
                    ui.group(|ui| {
                        ui.label(egui::RichText::new(format!("⬡ Face Area: {:.4} m²", area)).strong().color(egui::Color32::from_rgb(255, 180, 100)));
                        ui.label(format!("Perimeter: {:.4} m | Vertices: {}", perimeter, v_keys.len()));
                        ui.label(format!("Surface Normal: [{:.2}, {:.2}, {:.2}]", face.normal.x, face.normal.y, face.normal.z));
                    });
                }
            } else if selected_faces.len() == 2 {
                let f_keys: Vec<_> = selected_faces.iter().copied().collect();
                if let (Some(f0), Some(f1)) = (base_mesh.faces.get(f_keys[0]), base_mesh.faces.get(f_keys[1])) {
                    let dot = f0.normal.dot(f1.normal).clamp(-1.0, 1.0);
                    let dihedral_deg = dot.acos().to_degrees();
                    ui.group(|ui| {
                        ui.label(egui::RichText::new(format!("📐 Dihedral Angle: {:.2}°", dihedral_deg)).strong().color(egui::Color32::from_rgb(255, 200, 120)));
                        ui.label(format!("Normal 1: [{:.2}, {:.2}, {:.2}]", f0.normal.x, f0.normal.y, f0.normal.z));
                        ui.label(format!("Normal 2: [{:.2}, {:.2}, {:.2}]", f1.normal.x, f1.normal.y, f1.normal.z));
                    });
                }
            } else if selected_faces.len() > 2 {
                let mut total_area = 0.0_f32;
                for &fk in selected_faces {
                    let v_keys = base_mesh.face_vertices(fk);
                    if v_keys.len() >= 3 {
                        let p0 = base_mesh.vertices[v_keys[0]].position;
                        for i in 1..(v_keys.len() - 1) {
                            let p1 = base_mesh.vertices[v_keys[i]].position;
                            let p2 = base_mesh.vertices[v_keys[i + 1]].position;
                            total_area += ((p1 - p0).cross(p2 - p0)).length() * 0.5;
                        }
                    }
                }
                ui.group(|ui| {
                    ui.label(egui::RichText::new(format!("⬡ Total Selection Area: {:.4} m²", total_area)).strong().color(egui::Color32::from_rgb(255, 180, 100)));
                    ui.label(format!("Selected Faces Count: {}", selected_faces.len()));
                });
            } else {
                ui.label(egui::RichText::new("Select 2 vertices, 1-2 edges, or 1+ faces to measure precision CAD dimensions.").italics().color(egui::Color32::GRAY));
            }
        }
    });
    ui.separator();
}

#[cfg(test)]
mod tests {
    use geo_kernel::Primitives;

    #[test]
    fn test_cad_distance_and_angle_calculations() {
        let cube = Primitives::create_cube(2.0);
        let dims = cube.bounding_dimensions();
        assert!((dims.x - 2.0).abs() < 1e-4);
        assert!((dims.y - 2.0).abs() < 1e-4);
        assert!((dims.z - 2.0).abs() < 1e-4);

        // Distance between two corners
        let v0 = cube.vertices.keys().next().unwrap();
        let v1 = cube.vertices.keys().nth(1).unwrap();
        let p0 = cube.vertices[v0].position;
        let p1 = cube.vertices[v1].position;
        let dist = (p1 - p0).length();
        assert!(dist > 0.0);

        // Check manifold report on cube
        let report = cube.check_manifold();
        assert!(report.is_watertight);
        assert!(report.is_2_manifold);
        assert_eq!(report.euler_characteristic, 2);
        assert_eq!(report.genus, 0);
    }
}

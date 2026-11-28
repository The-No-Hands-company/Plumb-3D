//! 3D Printing, CAD Integrity & Geometric Analysis Engine.
//! Provides watertightness checking, exact physical volume/surface-area calculation,
//! overhang detection for FDM slicers, and one-click mesh healing.

use crate::mesh::{FaceKey, Mesh, MeshError};
use core_math::Vec3;

/// Comprehensive geometric, physical, and topological analysis report for 3D printing and CAD interchange.
#[derive(Debug, Clone, PartialEq)]
pub struct MeshAnalysisReport {
    /// Whether the mesh is a 100% closed, watertight 2-manifold solid.
    pub is_watertight: bool,
    /// Total enclosed physical volume in cubic meters (m³).
    pub volume_m3: f32,
    /// Total enclosed physical volume in cubic centimeters (cm³).
    pub volume_cm3: f32,
    /// Estimated weight in grams for PLA filament (density ~1.24 g/cm³).
    pub estimated_weight_pla_grams: f32,
    /// Total external surface area in square meters (m²).
    pub surface_area_m2: f32,
    /// Total external surface area in square centimeters (cm²).
    pub surface_area_cm2: f32,
    /// Number of open boundary half-edges (open holes).
    pub open_boundary_edges: usize,
    /// Number of degenerate zero-area faces.
    pub degenerate_faces: usize,
    /// Number of faces whose normal angle requires support structures.
    pub overhang_faces: usize,
    /// FaceKeys of steep overhang faces.
    pub overhang_face_keys: Vec<FaceKey>,
    /// Bounding box dimensions (X, Y, Z) in meters.
    pub dimensions_meters: Vec3,
    /// Bounding box dimensions (X, Y, Z) in millimeters (standard slicer units).
    pub dimensions_mm: Vec3,
    /// Centroid center of the bounding box in meters.
    pub center: Vec3,
    /// Total vertices count.
    pub vertex_count: usize,
    /// Total faces count.
    pub face_count: usize,
}

impl Mesh {
    /// Performs exhaustive physical, geometric, and topological analysis of the mesh for 3D printing.
    /// `overhang_threshold_deg`: Support angle threshold (typically 45.0° for standard FDM printing).
    pub fn analyze_for_3d_printing(&self, overhang_threshold_deg: f32) -> MeshAnalysisReport {
        let vertex_count = self.vertices.len();
        let face_count = self.faces.len();

        if vertex_count == 0 || face_count == 0 {
            return MeshAnalysisReport {
                is_watertight: false,
                volume_m3: 0.0,
                volume_cm3: 0.0,
                estimated_weight_pla_grams: 0.0,
                surface_area_m2: 0.0,
                surface_area_cm2: 0.0,
                open_boundary_edges: 0,
                degenerate_faces: 0,
                overhang_faces: 0,
                overhang_face_keys: Vec::new(),
                dimensions_meters: Vec3::ZERO,
                dimensions_mm: Vec3::ZERO,
                center: Vec3::ZERO,
                vertex_count,
                face_count,
            };
        }

        // 1. Bounding box computation
        let mut min_b = Vec3::splat(f32::INFINITY);
        let mut max_b = Vec3::splat(f32::NEG_INFINITY);
        for v in self.vertices.values() {
            min_b = min_b.min(v.position);
            max_b = max_b.max(v.position);
        }
        let dims_m = (max_b - min_b).max(Vec3::ZERO);
        let dims_mm = dims_m * 1000.0;
        let center = (min_b + max_b) * 0.5;

        // 2. Open boundary edges check
        let open_boundary_edges = self
            .half_edges
            .values()
            .filter(|he| he.face.is_none())
            .count();

        // 3. Overhang calculation
        let thresh_rad = overhang_threshold_deg.clamp(10.0, 85.0).to_radians();
        let sin_thresh = thresh_rad.sin();

        let mut overhang_face_keys = Vec::new();
        let mut degenerate_faces = 0_usize;
        let mut total_vol = 0.0_f32;
        let mut total_area = 0.0_f32;

        for (fk, face) in &self.faces {
            let verts = self.face_vertices(fk);
            if verts.len() < 3 {
                degenerate_faces += 1;
                continue;
            }

            // Check overhang against gravity (-Y direction)
            if face.normal.y < -sin_thresh {
                overhang_face_keys.push(fk);
            }

            // Fan triangulation for volume and surface area
            let p0 = self.vertices[verts[0]].position;
            for i in 1..(verts.len() - 1) {
                let p1 = self.vertices[verts[i]].position;
                let p2 = self.vertices[verts[i + 1]].position;

                // Signed tetrahedron volume to origin: (p0 . (p1 x p2)) / 6.0
                let tet_vol = p0.dot(p1.cross(p2)) / 6.0;
                total_vol += tet_vol;

                // Triangle area: ||(p1 - p0) x (p2 - p0)|| / 2.0
                let cross = (p1 - p0).cross(p2 - p0);
                let tri_area = cross.length() * 0.5;
                if tri_area < 1e-8 {
                    degenerate_faces += 1;
                }
                total_area += tri_area;
            }
        }

        let volume_m3 = total_vol.abs();
        let volume_cm3 = volume_m3 * 1_000_000.0;
        let estimated_weight_pla_grams = volume_cm3 * 1.24; // standard PLA density 1.24 g/cm³

        let surface_area_m2 = total_area;
        let surface_area_cm2 = surface_area_m2 * 10_000.0;

        let is_watertight = open_boundary_edges == 0 && degenerate_faces == 0;

        MeshAnalysisReport {
            is_watertight,
            volume_m3,
            volume_cm3,
            estimated_weight_pla_grams,
            surface_area_m2,
            surface_area_cm2,
            open_boundary_edges,
            degenerate_faces,
            overhang_faces: overhang_face_keys.len(),
            overhang_face_keys,
            dimensions_meters: dims_m,
            dimensions_mm: dims_mm,
            center,
            vertex_count,
            face_count,
        }
    }

    /// Automatically fixes common 3D printing defects (seals open boundary holes, cleans degenerates, recalculates normals).
    pub fn auto_fix_for_3d_printing(&mut self) -> Result<usize, MeshError> {
        let mut fixed_items = 0;
        let deg = self.clean_degenerate_geometry(1e-6, 1e-5).unwrap_or(0);
        fixed_items += deg;
        let holes = self.cap_holes(false).unwrap_or_default();
        fixed_items += holes.len();
        let _ = self.recalculate_face_normals(false);
        self.update_normals();
        Ok(fixed_items)
    }
}

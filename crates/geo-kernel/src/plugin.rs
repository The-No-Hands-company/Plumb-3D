//! C-ABI Foreign Function Interface (FFI) boundary and plugin execution pipeline for Plumb 3D.
//!
//! Provides a versioned, zero-cost, C-compatible geometry buffer and plugin descriptor
//! allowing native C, C++, and Rust dynamic libraries (`.so`, `.dll`, `.dylib`) as well
//! as WebAssembly runtimes to execute bare-metal geometry generation and modifiers.

use crate::mesh::Mesh;
use core_math::{vec2, vec3};
use std::ffi::CStr;

/// Magic signature bytes for Plumb 3D native plugins (`b"PLUG"`).
pub const PLUMB_PLUGIN_MAGIC: [u8; 4] = *b"PLUG";

/// Current stable major version of the Plumb 3D C-ABI Plugin specification.
pub const PLUMB_PLUGIN_API_VERSION: u32 = 1;

/// Classification of native plugins.
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlumbPluginType {
    /// Generates new mesh geometry from procedural algorithms.
    Generator = 0,
    /// Non-destructive mesh modifier / topological transform.
    Modifier = 1,
    /// Read-only CAD inspection, validation, or spatial analysis.
    Analysis = 2,
}

/// Header embedded in every native Plumb 3D plugin descriptor.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlumbPluginHeader {
    /// Must match [`PLUMB_PLUGIN_MAGIC`].
    pub magic: [u8; 4],
    /// Must match [`PLUMB_PLUGIN_API_VERSION`].
    pub api_version: u32,
    /// [`PlumbPluginType`] classification.
    pub plugin_type: u32,
    /// Capability flags reserved for future extensions.
    pub flags: u32,
}

impl Default for PlumbPluginHeader {
    fn default() -> Self {
        Self {
            magic: PLUMB_PLUGIN_MAGIC,
            api_version: PLUMB_PLUGIN_API_VERSION,
            plugin_type: PlumbPluginType::Modifier as u32,
            flags: 0,
        }
    }
}

/// Contiguous, C-compatible geometry buffer representing 2-manifold or polygonal meshes.
#[repr(C)]
pub struct PlumbMeshBuffer {
    /// Number of vertices in the buffer.
    pub vertex_count: u32,
    /// Flat array of 3D positions `[x0, y0, z0, x1, y1, z1, ...]`. Length: `vertex_count * 3`.
    pub positions: *mut f32,
    /// Flat array of surface normals `[nx0, ny0, nz0, ...]`. Length: `vertex_count * 3` or null.
    pub normals: *mut f32,
    /// Flat array of texture UV coordinates `[u0, v0, u1, v1, ...]`. Length: `vertex_count * 2` or null.
    pub uvs: *mut f32,
    /// Total number of polygonal faces.
    pub face_count: u32,
    /// Number of vertices per polygon face (e.g. 3 for tris, 4 for quads). Length: `face_count`.
    pub face_vertex_counts: *mut u32,
    /// Total count of face index references across all faces.
    pub total_indices: u32,
    /// Flattened indices into the vertex buffer defining face winding order. Length: `total_indices`.
    pub face_vertex_indices: *mut u32,
}

/// Errors returned by the plugin boundary and FFI conversion layer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluginError {
    NullBufferPointer,
    InvalidMagic,
    VersionMismatch { expected: u32, actual: u32 },
    InvalidFaceIndices,
    ExecutionFailed(i32),
}

impl std::fmt::Display for PluginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NullBufferPointer => write!(f, "Mesh buffer contains a null pointer"),
            Self::InvalidMagic => write!(f, "Invalid plugin magic signature"),
            Self::VersionMismatch { expected, actual } => {
                write!(
                    f,
                    "Plugin API version mismatch: expected {}, found {}",
                    expected, actual
                )
            }
            Self::InvalidFaceIndices => write!(f, "Face indices exceed vertex buffer bounds"),
            Self::ExecutionFailed(code) => {
                write!(f, "Plugin execution returned error code: {}", code)
            }
        }
    }
}

impl std::error::Error for PluginError {}

impl PlumbMeshBuffer {
    /// Converts an existing Plumb [`Mesh`] into an allocated, C-compatible [`PlumbMeshBuffer`].
    pub fn from_mesh(mesh: &Mesh) -> Self {
        let vertex_count = mesh.vertices.len() as u32;
        let face_count = mesh.faces.len() as u32;

        let mut vert_map = std::collections::HashMap::with_capacity(mesh.vertices.len());
        let mut pos_vec = Vec::with_capacity(mesh.vertices.len() * 3);
        let mut norm_vec = Vec::with_capacity(mesh.vertices.len() * 3);
        let mut uv_vec = Vec::with_capacity(mesh.vertices.len() * 2);

        for (idx, (vk, v)) in mesh.vertices.iter().enumerate() {
            vert_map.insert(vk, idx as u32);
            pos_vec.push(v.position.x);
            pos_vec.push(v.position.y);
            pos_vec.push(v.position.z);

            norm_vec.push(v.normal.x);
            norm_vec.push(v.normal.y);
            norm_vec.push(v.normal.z);

            uv_vec.push(v.uv.x);
            uv_vec.push(v.uv.y);
        }

        let mut face_counts = Vec::with_capacity(face_count as usize);
        let mut face_indices = Vec::new();

        for (fk, _face) in &mesh.faces {
            let fv = mesh.face_vertices(fk);
            face_counts.push(fv.len() as u32);
            for vk in &fv {
                if let Some(&i) = vert_map.get(vk) {
                    face_indices.push(i);
                }
            }
        }

        let total_indices = face_indices.len() as u32;

        // Leak into raw boxes to transfer ownership across C ABI
        let positions = Box::into_raw(pos_vec.into_boxed_slice()) as *mut f32;
        let normals = Box::into_raw(norm_vec.into_boxed_slice()) as *mut f32;
        let uvs = Box::into_raw(uv_vec.into_boxed_slice()) as *mut f32;
        let face_vertex_counts = Box::into_raw(face_counts.into_boxed_slice()) as *mut u32;
        let face_vertex_indices = Box::into_raw(face_indices.into_boxed_slice()) as *mut u32;

        Self {
            vertex_count,
            positions,
            normals,
            uvs,
            face_count,
            face_vertex_counts,
            total_indices,
            face_vertex_indices,
        }
    }

    /// Converts a [`PlumbMeshBuffer`] back into a fully topological [`Mesh`].
    ///
    /// # Safety
    /// Pointers in `self` must be valid, properly aligned, and match specified lengths.
    pub unsafe fn to_mesh(&self) -> Result<Mesh, PluginError> {
        if self.positions.is_null()
            || self.face_vertex_counts.is_null()
            || self.face_vertex_indices.is_null()
        {
            return Err(PluginError::NullBufferPointer);
        }

        let mut mesh = Mesh::new();
        let mut vertex_keys = Vec::with_capacity(self.vertex_count as usize);

        for i in 0..self.vertex_count as usize {
            let x = *self.positions.add(i * 3);
            let y = *self.positions.add(i * 3 + 1);
            let z = *self.positions.add(i * 3 + 2);
            let vk = mesh.add_vertex(vec3(x, y, z));

            if !self.normals.is_null() {
                let nx = *self.normals.add(i * 3);
                let ny = *self.normals.add(i * 3 + 1);
                let nz = *self.normals.add(i * 3 + 2);
                if let Some(v) = mesh.vertices.get_mut(vk) {
                    v.normal = vec3(nx, ny, nz);
                }
            }

            if !self.uvs.is_null() {
                let u = *self.uvs.add(i * 2);
                let v_coord = *self.uvs.add(i * 2 + 1);
                if let Some(v) = mesh.vertices.get_mut(vk) {
                    v.uv = vec2(u, v_coord);
                }
            }

            vertex_keys.push(vk);
        }

        let mut index_offset = 0usize;
        for f in 0..self.face_count as usize {
            let count = *self.face_vertex_counts.add(f) as usize;
            if index_offset + count > self.total_indices as usize {
                return Err(PluginError::InvalidFaceIndices);
            }

            let mut face_verts = Vec::with_capacity(count);
            for idx_in_face in 0..count {
                let v_idx = *self.face_vertex_indices.add(index_offset + idx_in_face) as usize;
                if v_idx >= vertex_keys.len() {
                    return Err(PluginError::InvalidFaceIndices);
                }
                face_verts.push(vertex_keys[v_idx]);
            }
            index_offset += count;

            let _ = mesh.add_face(&face_verts);
        }

        Ok(mesh)
    }

    /// Deallocates memory owned by this [`PlumbMeshBuffer`] that was allocated via `from_mesh`.
    ///
    /// # Safety
    /// Must only be called once on buffers created by [`PlumbMeshBuffer::from_mesh`].
    pub unsafe fn free_buffer(self) {
        if !self.positions.is_null() {
            let _ = Box::from_raw(std::ptr::slice_from_raw_parts_mut(
                self.positions,
                self.vertex_count as usize * 3,
            ));
        }
        if !self.normals.is_null() {
            let _ = Box::from_raw(std::ptr::slice_from_raw_parts_mut(
                self.normals,
                self.vertex_count as usize * 3,
            ));
        }
        if !self.uvs.is_null() {
            let _ = Box::from_raw(std::ptr::slice_from_raw_parts_mut(
                self.uvs,
                self.vertex_count as usize * 2,
            ));
        }
        if !self.face_vertex_counts.is_null() {
            let _ = Box::from_raw(std::ptr::slice_from_raw_parts_mut(
                self.face_vertex_counts,
                self.face_count as usize,
            ));
        }
        if !self.face_vertex_indices.is_null() {
            let _ = Box::from_raw(std::ptr::slice_from_raw_parts_mut(
                self.face_vertex_indices,
                self.total_indices as usize,
            ));
        }
    }
}

/// Function pointer type for C-ABI mesh modifier plugins.
pub type PlumbProcessMeshFn = unsafe extern "C" fn(*mut PlumbMeshBuffer) -> i32;

/// Stable C-compatible descriptor structure exported by native dynamic plugins.
#[repr(C)]
pub struct PlumbPluginDescriptor {
    /// Standardized header with magic bytes and versioning.
    pub header: PlumbPluginHeader,
    /// Null-terminated UTF-8 plugin display name (max 64 bytes).
    pub name: [u8; 64],
    /// Null-terminated UTF-8 author name (max 64 bytes).
    pub author: [u8; 64],
    /// Null-terminated UTF-8 SemVer version string (e.g. `b"1.0.0\0"`).
    pub version: [u8; 32],
    /// Optional C-ABI callback function to process a mesh buffer.
    pub process_mesh: Option<PlumbProcessMeshFn>,
}

impl PlumbPluginDescriptor {
    /// Creates a new plugin descriptor with UTF-8 strings truncated and null-terminated safely.
    pub fn new(
        name: &str,
        author: &str,
        version: &str,
        process_mesh: Option<PlumbProcessMeshFn>,
    ) -> Self {
        let mut d = Self {
            header: PlumbPluginHeader::default(),
            name: [0; 64],
            author: [0; 64],
            version: [0; 32],
            process_mesh,
        };
        copy_cstr(name, &mut d.name);
        copy_cstr(author, &mut d.author);
        copy_cstr(version, &mut d.version);
        d
    }

    /// Returns the UTF-8 plugin name.
    pub fn name_str(&self) -> &str {
        CStr::from_bytes_until_nul(&self.name)
            .ok()
            .and_then(|c| c.to_str().ok())
            .unwrap_or("Unknown")
    }

    /// Returns the author string.
    pub fn author_str(&self) -> &str {
        CStr::from_bytes_until_nul(&self.author)
            .ok()
            .and_then(|c| c.to_str().ok())
            .unwrap_or("Unknown")
    }

    /// Returns the version string.
    pub fn version_str(&self) -> &str {
        CStr::from_bytes_until_nul(&self.version)
            .ok()
            .and_then(|c| c.to_str().ok())
            .unwrap_or("0.0.0")
    }

    /// Validates the plugin header and compatibility.
    pub fn validate(&self) -> Result<(), PluginError> {
        if self.header.magic != PLUMB_PLUGIN_MAGIC {
            return Err(PluginError::InvalidMagic);
        }
        if self.header.api_version != PLUMB_PLUGIN_API_VERSION {
            return Err(PluginError::VersionMismatch {
                expected: PLUMB_PLUGIN_API_VERSION,
                actual: self.header.api_version,
            });
        }
        Ok(())
    }

    /// Safely executes this plugin against a [`Mesh`] across the C-ABI boundary.
    pub fn execute(&self, mesh: &Mesh) -> Result<Mesh, PluginError> {
        self.validate()?;
        let process_fn = self.process_mesh.ok_or(PluginError::ExecutionFailed(-1))?;

        let mut buffer = PlumbMeshBuffer::from_mesh(mesh);
        let ret = unsafe { process_fn(&mut buffer as *mut PlumbMeshBuffer) };
        if ret != 0 {
            unsafe {
                buffer.free_buffer();
            }
            return Err(PluginError::ExecutionFailed(ret));
        }

        let new_mesh = unsafe { buffer.to_mesh() };
        unsafe {
            buffer.free_buffer();
        }
        new_mesh
    }
}

fn copy_cstr(src: &str, dst: &mut [u8]) {
    let bytes = src.as_bytes();
    let max = dst.len().saturating_sub(1).min(bytes.len());
    dst[..max].copy_from_slice(&bytes[..max]);
    dst[max] = 0;
}

/// First-class, idiomatic, 100% pure safe Rust plugin interface.
///
/// Third-party Rust developers can write plugins purely in Rust without unsafe code,
/// working directly with [`Mesh`] and standard safe Rust types.
pub trait PlumbPlugin: Send + Sync {
    /// Human-readable display name of the plugin.
    fn name(&self) -> &str;
    /// Author or organization.
    fn author(&self) -> &str {
        "Community"
    }
    /// SemVer version string.
    fn version(&self) -> &str {
        "1.0.0"
    }
    /// Classification of this plugin.
    fn plugin_type(&self) -> PlumbPluginType {
        PlumbPluginType::Modifier
    }
    /// Executes the plugin operation directly on a [`Mesh`] using 100% safe Rust.
    fn execute(&self, mesh: &mut Mesh) -> Result<(), PluginError>;
}

/// Dynamic registry of active plugins in Plumb 3D.
#[derive(Default)]
pub struct PluginRegistry {
    plugins: Vec<Box<dyn PlumbPlugin>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a pure safe Rust plugin.
    pub fn register(&mut self, plugin: Box<dyn PlumbPlugin>) {
        self.plugins.push(plugin);
    }

    pub fn len(&self) -> usize {
        self.plugins.len()
    }

    pub fn is_empty(&self) -> bool {
        self.plugins.is_empty()
    }

    pub fn get(&self, index: usize) -> Option<&dyn PlumbPlugin> {
        self.plugins.get(index).map(|p| p.as_ref())
    }

    pub fn find_by_name(&self, name: &str) -> Option<&dyn PlumbPlugin> {
        self.plugins
            .iter()
            .find(|p| p.name() == name)
            .map(|p| p.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::Primitives;

    unsafe extern "C" fn mock_scale_plugin(buf: *mut PlumbMeshBuffer) -> i32 {
        if buf.is_null() || (*buf).positions.is_null() {
            return -1;
        }
        let count = (*buf).vertex_count as usize;
        for i in 0..count {
            let x = (*buf).positions.add(i * 3);
            let y = (*buf).positions.add(i * 3 + 1);
            let z = (*buf).positions.add(i * 3 + 2);
            *x *= 2.0;
            *y *= 2.0;
            *z *= 2.0;
        }
        0
    }

    #[test]
    fn test_plumb_mesh_buffer_roundtrip() {
        let cube = Primitives::create_cube(2.0);
        let buffer = PlumbMeshBuffer::from_mesh(&cube);

        assert_eq!(buffer.vertex_count, 8);
        assert_eq!(buffer.face_count, 6);
        assert_eq!(buffer.total_indices, 24);

        let restored = unsafe { buffer.to_mesh().expect("Valid roundtrip mesh") };
        assert_eq!(restored.vertices.len(), 8);
        assert_eq!(restored.faces.len(), 6);

        unsafe {
            buffer.free_buffer();
        }
    }

    #[test]
    fn test_native_plugin_c_abi_execution() {
        let cube = Primitives::create_cube(1.0);
        let descriptor = PlumbPluginDescriptor::new(
            "DoubleScalePlugin",
            "Plumb Core Team",
            "1.0.0",
            Some(mock_scale_plugin),
        );

        assert_eq!(descriptor.name_str(), "DoubleScalePlugin");
        assert_eq!(descriptor.author_str(), "Plumb Core Team");
        assert_eq!(descriptor.version_str(), "1.0.0");

        let scaled = descriptor
            .execute(&cube)
            .expect("Plugin execution succeeds");
        assert_eq!(scaled.vertices.len(), 8);
        assert_eq!(scaled.faces.len(), 6);

        // Verify all vertices scaled by 2.0
        for (_, v) in &scaled.vertices {
            assert!(v.position.x.abs() > 0.99);
            assert!(v.position.y.abs() > 0.99);
            assert!(v.position.z.abs() > 0.99);
        }
    }

    #[test]
    fn test_plugin_version_validation() {
        let mut descriptor = PlumbPluginDescriptor::new("Incompatible", "Unknown", "0.1.0", None);
        descriptor.header.api_version = 999;
        let err = descriptor.validate().unwrap_err();
        match err {
            PluginError::VersionMismatch { expected, actual } => {
                assert_eq!(expected, PLUMB_PLUGIN_API_VERSION);
                assert_eq!(actual, 999);
            }
            _ => panic!("Expected VersionMismatch"),
        }
    }

    struct PureRustInvertPlugin;
    impl PlumbPlugin for PureRustInvertPlugin {
        fn name(&self) -> &str {
            "PureRustInvert"
        }
        fn author(&self) -> &str {
            "Rustaceans"
        }
        fn execute(&self, mesh: &mut Mesh) -> Result<(), PluginError> {
            for (_, v) in &mut mesh.vertices {
                v.position = -v.position;
            }
            Ok(())
        }
    }

    #[test]
    fn test_pure_rust_plugin_execution_and_registry() {
        let mut registry = PluginRegistry::new();
        registry.register(Box::new(PureRustInvertPlugin));

        assert_eq!(registry.len(), 1);
        let plugin = registry
            .find_by_name("PureRustInvert")
            .expect("Plugin found");
        assert_eq!(plugin.author(), "Rustaceans");

        let mut cube = Primitives::create_cube(2.0);
        let orig_first = cube.vertices.values().next().unwrap().position;
        plugin
            .execute(&mut cube)
            .expect("Pure Rust plugin execution");
        let inverted_first = cube.vertices.values().next().unwrap().position;

        assert_eq!(inverted_first, -orig_first);
    }
}

# DCC Suite Developer Guide & Architectural Manual

This document is the authoritative engineering manual for developers contributing to or maintaining the DCC Suite codebase.

---

## 1. System Architecture & Crate Decomposition

The workspace is organized as a clean multi-crate Cargo workspace adhering to strict **Separation of Concerns (SoC)** and the **Zero Foreign C/C++ FFI** principle.

```text
                               ┌─────────────┐
                               │     app     │ (Winit, egui, Modals, Commands, Rhai)
                               └──────┬──────┘
                                      │
         ┌────────────────────────────┼───────────────────────────┐
         │                            │                           │
         ▼                            ▼                           ▼
┌──────────────────┐        ┌──────────────────┐        ┌──────────────────┐
│ render-viewport  │        │render-pathtracer │        │    geo-nodes     │
│ (wgpu 23 Raster) │        │ (Rayon CPU BVH)  │        │ (Visual DAG)     │
└────────┬─────────┘        └────────┬─────────┘        └────────┬─────────┘
         │                           │                           │
         └───────────────────────────┼───────────────────────────┘
                                     │
                                     ▼
                          ┌─────────────────────┐
                          │    graph-engine     │ (Maya DAG, Constraints, IK)
                          └──────────┬──────────┘
                                     │
                                     ▼
                          ┌─────────────────────┐
                          │     geo-kernel      │ (BMesh, Modifiers, Sculpt, IO)
                          └──────────┬──────────┘
                                     │
                                     ▼
                          ┌─────────────────────┐
                          │      core-math      │ (glam, Ray, AABB, Transforms)
                          └─────────────────────┘
```

### Engineering Rules & Code Invariants
1. **Zero Foreign C/C++ FFI**: All algorithms, data structures, math, windowing, and rendering pipelines are 100% safe pure Rust.
2. **File Length Discipline (~600-700 Line Limit)**: No operational source file may exceed ~650-700 lines. Monolithic files must be decomposed into focused submodules with explicit single responsibilities.
3. **Zero Compiler Warnings**: All crates must pass `cargo check --workspace` and `cargo test --workspace` with **0 warnings** and **0 errors**.
4. **Deterministic Topologic Invariants**: Every topological mutation must preserve 2-manifold half-edge integrity and Euler characteristics.

---

## 2. The 2-Manifold BMesh Half-Edge Kernel (`geo-kernel`)

The mesh representation in [`geo-kernel::mesh`](../crates/geo-kernel/src/mesh/) is an index-based **BMesh Half-Edge** data structure:

```rust
pub struct Mesh {
    pub vertices: SlotMap<VertexKey, Vertex>,
    pub edges: SlotMap<EdgeKey, Edge>,
    pub half_edges: SlotMap<HalfEdgeKey, HalfEdge>,
    pub faces: SlotMap<FaceKey, Face>,
    // ...
}
```

### Structural Invariants
- **Vertices**: Store `position: Vec3`, `normal: Vec3`, optional `uv: Vec2`, and an outgoing `half_edge: HalfEdgeKey`.
- **Half-Edges**: Form a directed loop around each polygonal face:
  - `origin: VertexKey`: Starting vertex of this oriented segment.
  - `target: VertexKey`: Ending vertex.
  - `face: FaceKey`: Face bounded by this half-edge.
  - `twin: HalfEdgeKey`: Opposite half-edge on the adjacent face (or boundary).
  - `next: HalfEdgeKey`: Next half-edge in the counter-clockwise face loop.
  - `prev: HalfEdgeKey`: Preceding half-edge in the face loop.
- **Edges**: Abstract connection between two vertices, holding a pointer to one of its two constituent twin half-edges.
- **Euler Characteristic Verification**:
  For any closed, orientable 2-manifold surface of genus $g$:
  $$V - E + F = 2(1 - g)$$
  For a standard sphere/cube ($g = 0$): $V - E + F = 2$.
  Topology operators run validation passes in [`validation.rs`](../crates/geo-kernel/src/validation.rs) to verify twin symmetry:
  $$\forall h, \quad \text{twin}(\text{twin}(h)) = h$$

---

## 3. Parametric Modifier System

Modifiers in [`geo-kernel::modifiers`](../crates/geo-kernel/src/modifiers/) implement the non-destructive `Modifier` trait:

```rust
pub trait Modifier: std::fmt::Debug + Send + Sync {
    fn name(&self) -> &str;
    fn apply(&self, mesh: &mut Mesh);
    fn is_enabled(&self) -> bool;
    fn set_enabled(&mut self, enabled: bool);
    fn clone_box(&self) -> Box<dyn Modifier>;
}
```

### Adding a New Modifier
1. **Define the Struct**: Create a new file under `crates/geo-kernel/src/modifiers/generate/` or `deform/`:
   ```rust
   #[derive(Debug, Clone)]
   pub struct MyCustomModifier {
       pub factor: f32,
       pub enabled: bool,
   }
   ```
2. **Implement the `Modifier` Trait**:
   ```rust
   impl Modifier for MyCustomModifier {
       fn name(&self) -> &str { "Custom Modifier" }
       fn apply(&self, mesh: &mut Mesh) {
           if !self.enabled { return; }
           // Perform parametric mutation on mesh
       }
       fn is_enabled(&self) -> bool { self.enabled }
       fn set_enabled(&mut self, enabled: bool) { self.enabled = enabled; }
       fn clone_box(&self) -> Box<dyn Modifier> { Box::new(self.clone()) }
   }
   ```
3. **Expose Downcasting (if inspector controls are needed)**:
   Add `as_my_custom_mut(&mut self) -> Option<&mut MyCustomModifier>` to the `Modifier` trait.
4. **Register in UI**: Add UI sliders inside [`crates/app/src/ui/inspector/modifiers.rs`](../crates/app/src/ui/inspector/modifiers.rs).

---

## 4. Maya DAG Scene Graph & Rigging (`graph-engine`)

The scene graph in [`graph-engine`](../crates/graph-engine/) evaluates a Directed Acyclic Graph:

- **Node Hierarchy**: Each `Node` contains `parent: Option<NodeId>`, `children: Vec<NodeId>`, `local_transform: Transform`, and `world_matrix: Mat4`.
- **Dirty Flagging**: Modifications to a node's local transform mark its dirty flag and recursively propagate to all descendants.
- **Evaluation Pipeline**:
  $$\mathbf{M}_{\text{world}} = \mathbf{M}_{\text{parent\_world}} \times \mathbf{M}_{\text{local}}$$
- **Constraints Stack**: Evaluated sequentially:
  - `AimConstraint`: Computes rotation quaternion orienting forward axis toward target node with an up-vector.
  - `ParentConstraint`: Replicates parent-child transform tracking with offset preservation.
  - `PathConstraint`: Parameterizes position along a spline curve.
- **IK Solvers**:
  - `Analytic2BoneIK`: Closed-form trigonometric law-of-cosines joint calculation for humanoid limbs.
  - `CyclicCoordinateDescentIK (CCD)`: Iterative gradient-free multi-bone chain solver.

---

## 5. Visual Geometry Nodes (`geo-nodes`)

The visual node editor in [`crates/geo-nodes`](../crates/geo-nodes/) allows node graph composition:

- **Graph Structure**: `GeoNodeGraph` stores `nodes: HashMap<NodeId, GeoNode>` and `connections: Vec<NodeConnection>`.
- **Topological Sorting**: Evaluates dependencies using **Kahn's Algorithm**. Cycles are rejected at connection time via DFS cycle detection (`can_connect`).
- **Dataflow Sockets**:
  - `SocketType::Geometry`: Carries a complete `geo_kernel::Mesh`.
  - `SocketType::Vector`: Carries `core_math::Vec3`.
  - `SocketType::Float`: Carries `f32`.
  - `SocketType::Transform`: Carries `core_math::Transform`.
- **Modifier Bridge**: The `GeometryNodesModifier` holds an internal `GeoNodeGraph` and executes it during the object's modifier stack pass.

---

## 6. Real-Time Viewport Pipeline (`render-viewport`)

The interactive viewport uses **wgpu 23**:

- **Forward Rasterization**:
  - Vertex layout: `[position: [f32; 3], normal: [f32; 3], uv: [f32; 2], color: [f32; 4]]`.
  - Shaders written in WGSL (`shaders.rs`).
- **Uniform Bind Groups**:
  - `@group(0) @binding(0)`: `CameraUniform` (`view_proj`, `eye_pos`, `viewport_params`).
  - `@group(1) @binding(0)`: `ModelUniform` (`model`, `color`, `material`).
- **3D Transform Gizmos**:
  - Scaled adaptively in screen space: $\text{scale} \propto \|\mathbf{p}_{\text{eye}} - \mathbf{p}_{\text{gizmo}}\|$.
  - Rendered with depth test enabled but depth write disabled to maintain clear overlay visibility without self-occlusion artifacts.

---

## 7. Production Path Tracer (`render-pathtracer`)

The offline Monte Carlo path tracer operates headlessly on CPU threads using **Rayon**:

- **Spatial Acceleration**:
  - Surface Area Heuristic (SAH) Bounding Volume Hierarchy (**BVH**).
  - Traverses ray intersections in $O(\log N)$ time against scene triangles using the Möller-Trumbore algorithm.
- **Microfacet BSDF**:
  - Cook-Torrance specular distribution with GGX normal distribution function (NDF):
    $$D(h) = \frac{\alpha^2}{\pi ((n \cdot h)^2 (\alpha^2 - 1) + 1)^2}$$
  - Smith geometric shadowing/masking function: $G(l, v, h)$.
  - Schlick Fresnel approximation: $F(\theta) = F_0 + (1 - F_0)(1 - \cos \theta)^5$.
  - Dielectric glass transmission refraction via Snell's Law.
- **Thin-Lens Physical Camera**:
  - Pinhole rays are jittered across a disk aperture with radius $R_{\text{lens}}$ and converged at focal plane distance $d_{\text{focus}}$ to simulate optical depth of field and bokeh.
- **Progressive Accumulation**:
  - Progressive accumulation buffer storing high-dynamic-range floating-point values:
    $$\bar{C}_{N} = \frac{N-1}{N} \bar{C}_{N-1} + \frac{1}{N} C_{\text{sample}}$$

---

## 8. Modal State Machine Pattern (`app::modals`)

Interactive viewport operations (Extrude, Bevel, Inset, Knife, LoopCut, Crease) are implemented as isolated modal state machines:

1. Hotkey triggers modal (e.g. `KeyCode::KeyE` sets `extrude_modal_active = true`).
2. Input routers in [`crates/app/src/input/`](../crates/app/src/input/) check `crate::modals::handle_mouse_input` and `crate::modals::handle_cursor_moved` first.
3. If an interactive modal is active, it consumes the event and short-circuits general viewport navigation.
4. Dragging adjusts parameters dynamically.
5. `LMB` / `Enter` commits the operation by pushing an undoable `Command` to the history stack.
6. `Esc` / `RMB` cancels the operation and reverts geometry.

---

## 9. Testing & Quality Assurance

Run the test suite across the entire workspace:

```bash
# Run all unit and integration tests
cargo test --workspace

# Validate zero compiler warnings
cargo check --workspace --all-targets
```
All unit tests must pass before opening a pull request.

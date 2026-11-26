# Plumb 3D Developer Guide & Architectural Manual

This document is the authoritative engineering manual for developers contributing to or maintaining the Plumb 3D codebase.

---

## 1. System Architecture & Crate Decomposition

The workspace is organized as a clean multi-crate Cargo workspace adhering to strict **Separation of Concerns (SoC)**, the **Zero Foreign C/C++ FFI** principle, and 100% Pure Safe Rust.

```text
                               ┌─────────────┐
                               │     app     │ (Winit, egui, egui_dock, Modals, Commands, Rhai)
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
                          │    graph-engine     │ (Maya DAG, Constraints, 2-Bone IK, CCD)
                          └──────────┬──────────┘
                                     │
                                     ▼
                          ┌─────────────────────┐
                          │     geo-kernel      │ (BMesh, Modifiers, Sculpt, Splines, UV)
                          └──────────┬──────────┘
                                     │
                                     ▼
                          ┌─────────────────────┐
                          │      core-math      │ (glam, Ray, AABB, Transforms)
                          └─────────────────────┘
```

### Engineering Rules & Code Invariants
1. **100% Pure Safe Rust (`zero unsafe`)**: All algorithms, data structures, math, windowing, and rendering pipelines are pure safe Rust. Zero `unsafe` blocks across all 7 workspace crates.
2. **Strict Zajfan Engineering Line Ceiling ($\le 630$ Lines)**: Every single `.rs` source file must strictly adhere to the $\le 630$ line ceiling. Monolithic files must be decomposed into focused submodules with explicit single responsibilities.
3. **Zero Compiler & Clippy Warnings**: All crates must pass `cargo check --workspace`, `cargo test --workspace`, and `cargo clippy --workspace -- -D warnings` with **0 warnings** and **0 errors**.
4. **Deterministic Topologic Invariants**: Every topological mutation must preserve 2-manifold half-edge integrity and Euler characteristics ($V - E + F = 2(1 - g)$).
5. **Clean Typography**: 100% 7-bit ASCII typography throughout the entire UI subsystem to prevent font fallback tofu glyphs (`[ ]`).

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
   pub struct MyModifier {
       pub parameter: f32,
       pub enabled: bool,
   }
   ```
2. **Implement `Modifier`**: Implement `name()`, `apply()`, `is_enabled()`, `set_enabled()`, and `clone_box()`.
3. **Register in Stack**: Register the new modifier variant in `ModifierType` enum and the inspector UI.

---

## 4. Testing & Verification

Plumb 3D maintains a rigorous test suite of **281 passing unit and integration tests**:

```bash
# Run all workspace unit & integration tests
cargo test --workspace

# Run Clippy with zero warnings allowed
cargo clippy --workspace -- -D warnings

# Check file line limits
python3 -c '
import os
for root, _, files in os.walk("crates"):
    for f in files:
        if f.endswith(".rs"):
            p = os.path.join(root, f)
            with open(p) as fl:
                n = len(fl.readlines())
                if n > 630: print(f"VIOLATION: {n} lines: {p}")
'
```

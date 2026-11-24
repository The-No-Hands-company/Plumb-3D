# Plumb 3D

A next-generation, high-performance 3D modeling, procedural generation, animation, and rendering suite engineered entirely in **100% Safe, Pure Rust**.

Inspired by the precision modeling workflows of **Plasticity**, the topological power of **Blender**, the hierarchical DAG and constraint rigging of **Autodesk Maya**, and the parametric modifier stack of **Autodesk 3ds Max**, Plumb 3D delivers a unified, zero-C/C++-dependency platform for artists, engineers, and technical directors.

---

## Core Architectural Pillars

### 1. Deterministic `.plmb` Project Container & Versioning
- **Native Binary Container (`.plmb`)**: Chunk-based binary archive (`PLMB`) with IEEE 802.3 CRC32 data integrity validation.
- **Embedded Semantic Versioning**: Forward/backward compatible chunk parser preserving geometry, UV seams, creasing weights, vertex groups, and metadata without lossy external formats.

### 2. High-Efficiency UX & Declarative Workspaces
- **Workspace Templates**: One-click curated workflows (`📐 Layout`, `🔨 Hard-Surface`, `📐 CAD Precision`, `🗿 Organic Sculpting`, `🗺 UV & Retopo`, `🎨 LookDev`, `🎬 Animation`).
- **Interactive Welcome & Quick-Start Hub (`F2`)**: Instant project starter templates, navigation keymaps, and tutorials.
- **Omnipresent Command Palette (`F3` / `Ctrl+K`)**: Fuzzy search across 30+ modeling operators, modifiers, workspaces, and tools.
- **Dynamic Viewport Action Hints**: Context-sensitive HUD overlay guiding mouse and keyboard actions in real-time.
- **Hot-Reloadable Design Tokens (Themes)**: Instant runtime theme switching (`Plumb Slate`, `Blender Charcoal`, `Maya Neutral Gray`, `CAD Blueprint Navy`, `Paper Light Drafting`).

### 3. Stable Versioned Plugin Architecture
- **Rust-First Plugins**: Safe, idiomatic `PlumbPlugin` trait and `PluginRegistry` for community tool authoring with zero unsafe code.
- **Zero-Cost C-ABI Kernel Boundary**: `PlumbMeshBuffer` (`#[repr(C)]`) and `PlumbPluginDescriptor` allowing external dynamic libraries (`.so`, `.dll`, `.dylib`) and WebAssembly modules to execute bare-metal geometry modifiers without ABI drift.

---

## Key Highlights & Architectural Features

### 1. Robust 2-Manifold BMesh Half-Edge Kernel (`geo-kernel`)
- **Topological Integrity**: Strict twin half-edge pointer graph maintaining 2-manifold orientation and Euler characteristic invariants across all operations ($V - E + F = 2(1 - g)$).
- **Interactive Modals**: Blender-style smooth modal operators with mouse drag feedback:
  - `E`: Extrude region & faces along surface normals.
  - `Ctrl + B`: Bevel edges and faces with interactive mouse-wheel segment tessellation.
  - `I`: Inset faces with contiguous region and individual face modes.
  - `K`: Interactive Knife bisect tool with surface snap crosshairs.
  - `Ctrl + R`: Quad loop cut preview and edge slide placement.
  - `Shift + E` / `Shift + V`: Semi-sharp edge & vertex creasing for Catmull-Clark surfaces.
  - `J` & `V`: Connect vertices across faces and topological vertex ripping.
  - `Alt + M`: Merge vertices at center, cursor, or by distance weld.

### 2. Parametric Non-Destructive Modifier Stack
- **Mesh Generation**: Array, Bevel, Solidify, Screw (Lathe), Subdivide, Catmull-Clark Semi-Sharp Creasing, Triangulate, Wireframe, Weighted Normal auto-smoothing.
- **Deformers**: Simple Deform (Bend, Twist, Taper, Stretch), Lattice FFD, Curve Deform (Path sweep), Shrinkwrap, Morpher (Blendshapes), Linear Blend Skinning (Armatures).
- **Physical Simulation**:
  - `ClothModifier`: Verlet integration with distance constraints, ground collision, and vertex group pinning.
  - `ParticleSystemModifier`: Dynamic physical particle simulation with gravity, normal velocity, bounce restitution, and billboard/tetrahedron mesh generation.
- **Garland-Heckbert Quadric Error Metric (QEM) Decimation**: Non-destructive topology simplification preserving silhouettes and boundary edges.

### 3. Maya DAG Scene Graph & Rigging (`graph-engine`)
- **Hierarchical Propagation**: Directed Acyclic Graph (DAG) with dirty-flag transformation cascades.
- **Parametric Constraints**: Aim (Look-At with World Up vector), Parent Constraint (transform offset maintaining), and Path Constraint (Spline curve following).
- **Inverse Kinematics (IK)**: Analytic 2-Bone IK solver and iterative multi-joint Cyclic Coordinate Descent (CCD) IK.
- **Full History**: Transactional Undo/Redo command stack (`Ctrl+Z` / `Ctrl+Y`).

### 4. Visual Geometry Nodes Canvas (`geo-nodes`)
- **Procedural DAG Graph**: Unreal Blueprints / Blender Geo Nodes style node editor.
- **Data Flow Types**: Geometry (BMesh Half-Edge), Vectors, Floats, Transforms.
- **Node Library**: Primitives (Cube, Cylinder, Sphere, Grid), Transforms, Noise Displacement, Point Scattering / Instancing, Boolean CSG, and Group Outputs.
- **Modifier Bridge**: Run any geometry node graph as a live modifier in an object's stack.

### 5. Embedded Rhai Scripting Engine
- **Sandboxed Execution**: 100% safe Rust scripting runtime without unsafe C FFI (Lua/Python).
- **First-Class DCC API**: Direct bindings for vectors, matrices, transforms, primitive generators, subdivision, extrusion, scene querying, and light spawning.
- **Interactive Console (`📜 Rhai`)**: Live multi-line script editor and REPL.

### 6. Dual Rendering Engines
- **Interactive Real-Time Viewport (`render-viewport`)**:
  - Hardware-accelerated forward rasterizer on **wgpu 23** (Vulkan, Metal, DirectX 12).
  - Studio MatCaps, flat shading, PBR preview, infinite anti-aliased ground grid.
  - Interactive 6-Axis navigation orbit sphere (`+X`, `-X`, `+Y`, `-Y`, `+Z`, `-Z`).
  - Professional 3D Transform Gizmos: 3D cones for translation, smooth rings for rotation, cubes for scaling, and planar drag tiles ($XY, XZ, YZ$).
- **Offline Production Path Tracer (`render-pathtracer`)**:
  - Multi-threaded CPU Monte Carlo path tracing engine powered by **Rayon**.
  - Surface Area Heuristic (SAH) Bounding Volume Hierarchy ($O(\log N)$ BVH).
  - Cook-Torrance Microfacet specular BRDF, Lambertian diffuse, dielectric Fresnel glass transmission, and metallic reflections.
  - Thin-lens physical optical camera with concentric disk aperture sampling for realistic optical depth-of-field bokeh.
  - F12 progressive render accumulation window with PNG export.

### 7. CAD, 3D Printing & Point Cloud Interchange
- **STL (ASCII & Binary)**: Watertight export for Cura, PrusaSlicer, and Bambu Studio.
- **PLY (ASCII & Binary)**: Point cloud and colored mesh interchange.
- **glTF 2.0 & OBJ**: Standard game engine exchange with material slots and UV texture maps.

---

## Workspace Architecture

The repository is cleanly decomposed into 7 decoupled, pure-Rust crates under `crates/`:

| Crate | Responsibilities | Dependencies |
| :--- | :--- | :--- |
| **`core-math`** | Vector/matrix math (`glam`), Spatial Transforms, Raycasting, AABB, Geometric intersections. | `glam`, `serde` |
| **`geo-kernel`** | 2-Manifold BMesh Half-Edge kernel, Modals, Modifiers, Sculpting, Splines, UV Unwrapping, IO (STL/PLY/OBJ/glTF). | `core-math`, `rayon` |
| **`graph-engine`** | Maya DAG SceneGraph, World matrix cascades, Constraints, IK Solvers, Animation Tracks. | `core-math`, `geo-kernel` |
| **`geo-nodes`** | Visual geometry node graph evaluation, socket typing, topological sort, cycle prevention. | `core-math`, `geo-kernel` |
| **`render-viewport`** | Real-time wgpu 23 forward rasterizer, camera, grid, actor glyphs, 3D transform gizmos. | `core-math`, `geo-kernel`, `wgpu` |
| **`render-pathtracer`**| Multi-threaded Monte Carlo path tracer, BVH spatial tree, Cook-Torrance BSDF, bokeh camera. | `core-math`, `geo-kernel`, `rayon`, `image` |
| **`app`** | Application lifecycle (`winit`), UI panels (`egui`), modal state machines, undo/redo commands, Rhai console. | All crates, `egui`, `winit`, `rhai` |

---

## Quick Start

### Prerequisites
- [Rust Toolchain](https://rustup.rs/) (1.78+ stable recommended).
- Vulkan, Metal, or DirectX 12 compatible graphics drivers.

### Build and Run
```bash
# Clone the repository
git clone https://github.com/The-No-Hands-company/Plumb-3D.git
cd Plumb-3D

# Run unit tests across all 7 workspace crates
cargo test --workspace

# Launch the interactive DCC application
cargo run -p app --release
```

---

## In-Software Documentation (Press `F1`)
Press **`F1`** inside the running application or click **`Help -> User Manual & Shortcuts`** to open the built-in, searchable interactive documentation browser covering hotkeys, modeling workflows, geometry nodes, Rhai scripting examples, and path tracing settings.

---

## Documentation Sitemap

Detailed documentation is available in the [`docs/`](docs/) directory:

- 📖 [**User Guide**](docs/USER_GUIDE.md): End-to-end interface overview, modeling tutorials, modifier guide, sculpting, and rendering.
- 🎓 [**Beginner Tutorials**](docs/TUTORIALS.md): 7 hands-on step-by-step modeling tutorials (Sci-Fi Crate, Castle Tower, Organic Creature Bust, Industrial Pipes, Geometry Nodes, Studio Bokeh Render, 3D Printing Vase).
- ⌨ [**Keyboard Shortcuts**](docs/KEYBOARD_SHORTCUTS.md): Full categorized index of all hotkeys and mouse gestures.
- 📜 [**Rhai Scripting API Reference**](docs/RHAI_SCRIPTING_API.md): Procedural scripting guide, syntax, types, methods, and code snippets.
- 🏗 [**Developer Guide & Architecture**](docs/DEVELOPER_GUIDE.md): Deep-dive engineering guide, BMesh invariants, DAG lifecycle, and crate extension tutorials.
- 🤝 [**Contributing Guidelines**](docs/CONTRIBUTING.md): Coding standards, line-limit policy, pure-Rust invariants, and testing rules.

---

## License

Engineered with pride by The No-hands Company. Distributed under the MIT License.

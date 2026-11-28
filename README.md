# Plumb 3D

<div align="center">

![Medieval Citadel Hero Production Render](docs/images/castle_citadel_hero.png)

*Architectural Medieval Citadel & Fortress modeled in Plumb 3D with parametric modifier stacks, PBR materials, torchlight illumination, and rendered with the pure-Rust Monte Carlo Path Tracer + Post-Production LookDev pipeline (Bloom, Vignette, ACES Filmic Tone Mapping).*

[![CI/CD Build & Test](https://github.com/The-No-Hands-company/Plumb-3D/actions/workflows/ci.yml/badge.svg)](https://github.com/The-No-Hands-company/Plumb-3D/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Pure Safe Rust](https://img.shields.io/badge/Rust-100%25%20Safe-brightgreen.svg)](#architectural-compliance--quality-standards)
[![Tests Passing](https://img.shields.io/badge/Tests-438%20Passed-success.svg)](#architectural-compliance--quality-standards)
[![Zero Clippy Warnings](https://img.shields.io/badge/Clippy-0%20Warnings-success.svg)](#architectural-compliance--quality-standards)

</div>

A next-generation, high-performance 3D modeling, procedural generation, animation, and rendering suite engineered entirely in **100% Safe, Pure Rust**.

Inspired by the precision modeling workflows of **Plasticity**, the topological power of **Blender**, the hierarchical DAG and constraint rigging of **Autodesk Maya**, and the parametric modifier stack of **Autodesk 3ds Max**, Plumb 3D delivers a unified, zero-C/C++-dependency platform for artists, engineers, and technical directors.

---

## Screenshots & Interface

<div align="center">

### Interactive Welcome & Quick-Start Hub (`F2`)
![Plumb 3D Welcome Hub](docs/images/plumb_3d_welcome_hub.jpg)

### Real-Time Modeling Viewport & Workspace Navigation
![Plumb 3D Modeling Viewport](docs/images/plumb_3d_viewport.jpg)

### Signature Plumb Bob Mascot Studio Render
![Plumb Bob Mascot Hero Shot](docs/images/plumb_bob_splash.png)

</div>

---

## Key Highlights & Subsystem Architecture (25 Hardened Domains)

Plumb 3D is hardened across **25 systematic subsystems** with full unit test coverage and production UI integration:

### 1. Mesh Primitives & Parametric Generators
- 10 Parametric Primitives: Cube, Cylinder, Cone, UV Sphere, IcoSphere, Torus, Planar Grid, Pyramid, Capsule, and the mascot Plumb Bob.
- Fully procedural dimensions, segment tessellation, UV coordinate unwrapping, and outward normal recalculation.

### 2. Sub-Element Selection & Mesh Inspection
- Discrete sub-element levels: Vertices (`1`), Edges (`2`), Faces (`3`), and Object Mode (`Tab`).
- Advanced selection algorithms: Topological Edge Loops (`Alt+L`), Edge Rings, Manifold Linked Shells (`L`), Marquee Box Select (`B`), Shortest Path Dijkstra, and Selection Inversion (`Ctrl+I`).
- CAD Caliper Inspection: Real-time Euclidean distance, face-to-face angles, and bounding box dimensions.

### 3. Polygonal Modeling & Direct Edit Operators
- Industry-standard BMesh Half-Edge modal operators with smooth mouse-drag feedback:
  - **Extrusion (`E`)**: Region and individual face extrusions constructing watertight sidewalls.
  - **Direct Push-Pull (`Shift+E`)**: Face click-drag extrusion with automatic boolean union/subtraction.
  - **Bevel (`Ctrl+B`)**: Multi-segment edge chamfering with mouse-wheel segment tessellation.
  - **Inset (`I`)**: Contiguous perimeter patch inset and individual polygonal inset modes.
  - **Interactive Knife (`K`)**: Ray-cast polygon bisection with edge and vertex snapping crosshairs.
  - **Loop Cut & Slide (`Ctrl+R`)**: Continuous quad-loop subdivision with interactive slide positioning.
  - **Bridge Edge Loops**: Multi-edge lofting between boundary loops with twist and segment controls.
  - **Topological Cleanup**: Tris-to-Quads (`Alt+J`), Grid Fill, Hole Capping, Normal Flipping, and Edge Flipping.

### 4. CAD Precision Drafting & NURBS Curves
- **3D Cubic Bézier Splines**: De Casteljau subdivision, tangent handle constraints (Free, Aligned, Collinear).
- **Interactive Bézier Pen Tool (`P`)**: Click-and-drag drawing of 3D curves directly on geometry and the workplane.
- **Boolean Spline Trimming & Splitting**: Parametric curve-curve intersection solving, knot insertion, and polygon region trimming.
- **Continuous Tangent Arc Fillets ($G^1$)**: Dynamic fillet arcs and cursor tangent snapping for precision CAD drafting.
- **Revolve (Lathe) & Pipe Sweeps**: Circular profile revolution and cross-sectional sweeps along 3D guide curves.
- **Dynamic CAD Workplane (`Alt+W`)**: Align construction grid to arbitrary faces; mirror symmetrize (`Alt+X`).
- **NURBS & STEP (ISO 10303-21) Export**: Bicubic B-spline surface patches with standard STEP format interchange.

### 5. Hard-Surface Modals & Precision Cutters
- **Direct Cutter Modal (`C`)**: Interactive Box, Circle, Ngon, and Radial Compound cutting tools with live ghost mesh preview.
- **Conformal Decals**: Ray-projected surface decals with normal displacement offsets.
- **Weld Beads & Un-Bevel**: Procedural welding seam generation and chamfer reconstruction along sharp corners.

### 6. Live Non-Destructive Modifiers (3ds Max Style Stack)
- 16 Live Modifiers: SubD Catmull-Clark (with semi-sharp crease weights), Bevel Chamfer, Mirror, Boolean CSG (Union, Difference, Intersection), Solidify, Array, QEM Mesh Decimation, Shrinkwrap, Curve Deform, Lattice FFD, Morpher (Blendshapes), and Linear Blend Skinning.
- Live stack reordering, visibility toggling, parameter tweaking, and one-click modifier collapsing.

### 7. Digital Sculpting & Dynamic Topology (DynTopo)
- 10 Sculpting Brushes: Clay, Grab, Crease, Smooth, Inflate, Flatten, Pinch, Scrape, SnakeHook, and DynTopo edge split/collapse.
- Mirror symmetry across $X, Y, Z$ planes with circular brush screen-space HUD overlay.

### 8. Texture Painting & Vertex Colors
- Vertex Color RGBA painting with smooth radial brush falloff.
- Weight Painting Mode with Linear Gradient tool for armature bone weights and particle density masks.
- Dynamic Cavity & Curvature heatmap visualization for pronounced surface relief inspection.

### 9. UV Island Parameterization & Unwrapping
- Conformal Tutte Disk Harmonic parameterization, LSCM unwrap, and Smart UV Angle Projection.
- **ABF++ Distortion Heatmap**: Real-time per-face angle distortion and area stretch analysis (Blue = zero distortion, Red = high distortion).
- **Interactive 2D Seam Cut Mode**: Click edges directly in the 2D UV canvas to cut or clear UV seams.
- UV Island Packing with area weighting for uniform texel density across texture maps.

### 10. Physics, Dynamics & Fluids
- **XPBD Cloth Simulator**: Extended Position-Based Dynamics with stretch/shear constraints, isometric bending, pneumatic pressure, ground collision, and **Spatial Hash Grid Particle Self-Collision**.
- **SPH Fluid Dynamics**: Smoothed-Particle Hydrodynamics with density pressure, viscosity, and surface tension.
- **Rigid Body Simulation**: Gravity, angular velocity, and restitution impulse floor collisions.
- **Physics Keyframe Baker**: Bake live physical simulations directly into transform keyframe tracks.

### 11. Skeletal Rigging, Constraints & IK
- Automatic Armature Bone Binding with heat diffusion weights.
- **Analytic 2-Bone IK Solver**: Law of Cosines solver with **Pole Vector target**, **Pole Twist Angle ($ -180^\circ$ to $+180^\circ$)**, and **FK/IK Weight Blend**.
- Cyclic Coordinate Descent (CCD) IK solver for arbitrary N-bone kinematic chains (spines, tentacles, robot arms).
- Parametric Constraints: Aim (Look-At with World Up), Parent, and Spline Path Follow.

### 12. Animation, Timeline & Motion Matching
- Interactive Timeline with frame scrubbing, playback loop, keyframe markers, and FPS controls.
- Keyframe interpolation (Linear, Step, Smooth Bézier Spline) with Dopesheet and Graph Curve editor.
- **Inertialization Smooth Decay**: Transition smoothing between conflicting animation states.
- **Motion Matching Database**: Real-time trajectory prediction and feature cost querying for responsive character locomotion.

### 13. Procedural Geometry Nodes (GeoNodes)
- Node Graph Editor: Visual Directed Acyclic Graph (DAG) with type-safe socket connections.
- Node Library: Primitives, Transforms, Math Operators, Procedural Noise Displacement, Point Scattering / Instancing, Curve Fillet/Revolve, and Boolean CSG.
- Live Modifier Bridge: Host any Geometry Node graph directly inside an object's modifier stack.

### 14. Multi-Engine Scripting & Studio Automation
- **Native Rhai Scripting Engine**: 100% pure-safe Rust scripting runtime with first-class DCC bindings, memory caps, and infinite loop protection.
- **Python 3 & Lua 5.4 Runners**: Multi-engine script execution pipeline.
- **Interactive Script Console**: REPL with syntax coloring, multiline editor, and stdout capturing.

### 15. Viewport Rendering, Shading & NPR
- Hardware-accelerated rasterization on **wgpu 23** (Vulkan, Metal, DirectX 12).
- 10 Viewport Shading Modes: Studio MatCap (Red Wax, Clay, Chrome), PBR Preview, Normals Inspection, Zebra Surfacing, NPR Cel-Lighting, NPR Halftone Stippling, and Ray-Traced Ambient Occlusion (RTAO) contact shadows.
- 6-Axis navigation orbit compass and multi-axis 3D transform gizmos.

### 16. Production Spectral Path Tracer (`render-pathtracer`)
- Multi-threaded CPU Monte Carlo path tracing engine powered by **Rayon**.
- Surface Area Heuristic (SAH) Bounding Volume Hierarchy ($O(\log N)$ BVH).
- Cook-Torrance Microfacet specular BRDF, Lambertian diffuse, dielectric Fresnel glass transmission, and metallic reflections.
- Thin-lens optical camera with physical bokeh depth-of-field.
- **Built-in LookDev Compositor**: Multi-pass separable Gaussian Bloom glow, corner vignette falloff, saturation/contrast curve grading, and ACES Filmic tone mapping.
- **Hardware RT Acceleration**: Multi-million polygon BLAS partitioning into 64K-triangle chunks with strict 256-byte alignment and 64-byte TLAS instance descriptors.
- PBR Texture Map Baker: Tangent-space Normal, Ambient Occlusion, and Displacement baking.

### 17. File Formats & CAD Interchange (I/O)
- **Native `.plmb` Container**: IEEE 802.3 CRC32-validated chunk-based binary project container.
- **CAD & CAM Interchange**: ISO 10303-21 STEP, Continuous NURBS STEP, G-code Slicing output.
- **Standard 3D Formats**: glTF 2.0 / GLB, Wavefront OBJ, Binary STL, PLY, and USDA (Universal Scene Description).

### 18. Modern UI/UX, Drag-and-Drop Docking & Collaboration
- **Drag-and-Drop Docking Workspace (`egui_dock`)**: Full tab rearranging and panel docking for Outliner, Inspector, Geometry Nodes, 2D UV Editor, Timeline, Console, and 3D Viewport.
- **Collapsible Side Panels**: Cleanly isolated left Outliner and right Inspector with automatic vertical scrolling.
- **Command Palette (`F3` / `Ctrl+K`)**: Omnipresent fuzzy search across all tools, modifiers, and operators.
- **Welcome & Project Hub (`F2`)**: Quick-start starter templates and shortcuts manual.
- **CAM 3D Print Quality Inspector**: Watertightness validation, overhang angle detection, layer slicing preview, and G-code export.
- **Live Collaboration Session**: Multi-client scene node synchronization (Figma for 3D).

### 19. Precision B-Rep, Trimmed NURBS & SSI
- **Analytic Trimmed NURBS Surface Patches**: Parametric surfaces bounded by 2D trimming curves in UV space ($[0,1] \times [0,1]$).
- **Surface-Surface Intersection (SSI)**: Robust marching solver calculating transverse intersection curves between intersecting NURBS sheets.
- **STEP (ISO 10303-21) Solid B-Rep Importer**: Full parsing and boundary representation reconstruction from industry-standard STEP AP203/AP214 files.
- **8-Directional Radial Pie Menu (`Space` / `Tab`)**: Ultra-fast gestural modeling shortcuts for extrude, inset, bevel, knife, and loop cut.

### 20. Class-A Surface Diagnostics & Universal CAD Interop
- **Dynamic Zebra Reflection Striping**: Screen-space normal reflection striping with adjustable stripe frequency and rotation angle for $G^1$ tangency and $G^2$ curvature inspection.
- **Screen-Space Curvature Heatmaps**: Real-time Gaussian curvature ($K = \kappa_1 \kappa_2$) and Mean curvature ($H = \frac{\kappa_1 + \kappa_2}{2}$) colormaps (Turbo, Cool-to-Warm, Jet).
- **$G^0 / G^1 / G^2$ Seam Continuity Diagnostics**: Automated boundary inspection detecting positional gaps, angular tangency kinks, and curvature radius jumps.
- **Porcupine Curvature Combs**: Needle vectors along 3D curves indicating curvature radius and inflection points.
- **ANSI/USAF IGES 5.3 Solid Interop**: Native import and export for Entity 128 (B-Spline Surface), Entity 126 (B-Spline Curve), and Entity 110 (Line).

### 21. Advanced B-Rep Solids & Boundary Patch Filling
- **Variable-Radius Rolling-Ball & Chordal Fillet Engine**: True rolling-ball fillet blending along 3D edge rails smoothly tapering from radius $r_1 \to r_2$ with circular $G^1$ and curvature-continuous $G^2$ transitions.
- **$N$-Sided Trim Patch Filler (Coons & Gordon Networks)**: Fills arbitrary 3-sided, 5-sided, or $N$-sided holes with watertight patches maintaining tangent matching with adjacent surfaces.
- **CAD Solid Shell & Uniform Thicken Operator**: Hollows solid bodies leaving uniform wall thickness, thickens open surface sheets, and miters sharp corners.

### 22. Multi-Body CAD Assemblies & Kinematics
- **Kinematic Mate Connectors**: Precise local coordinate frames with Origin, Up, and Normal alignment.
- **7 Kinematic Joint Types**: Coincident, Concentric, Revolute (hinge), Prismatic (slider), Cylindrical, Gear, and Fixed with `Aligned` vs `AntiAligned` flips.
- **Separating Axis Theorem (SAT) 3D Clash Detection**: Fast AABB broadphase + exact triangle-triangle SAT narrowphase computing intersection volumes and penetration depth.
- **Exploded View Sequencer**: Interactive progress scrubber ($t \in [0.0, 1.0]$) and procedural 3D dashed trail lines.

### 23. Technical 2D Engineering Drawings & Sheet Metal Flat-Pattern
- **First & Third Angle Orthographic Projection**: Front, Top, Right, Left, Bottom, Rear, and Isometric ($30^\circ/30^\circ$) projections.
- **Analytical Vector Hidden Line Removal (HLR)**: Rigorous edge-surface occlusion solver classifying solid visible lines, dashed hidden lines, and silhouette outlines.
- **ASME Y14.5 GD&T Callouts**: Feature Control Frames (Position, Flatness, Perpendicularity, Runout) with MMC/LMC modifiers, linear witness dimensions, and center marks.
- **Standard Drawing Sheet Exporters**: ISO A4–A0 and ANSI A–E sheets with formal title blocks, exporting to AutoCAD DXF and styled vector SVG.
- **Sheet Metal & CNC Flat Pattern Unfolder**: DIN 6935 K-factor bend deduction engine, parametric base & edge flanges, and 1-click flat pattern unfolding exporting to multi-layer DXF (`OUTER_CUT`, `BEND_UP`, `BEND_DOWN`) and SVG.

### 24. Mechanical Part Generators, Fastener Library & Involute Gears
- **Parametric ANSI/ISO Fasteners**: Full metric range from M2 through M30 with ISO 4014 Hex Bolts, ISO 4762 Socket Head Cap Screws (SHCS), ISO 4032 Hex Nuts, ISO 7089 Flat Washers, and DIN 127 Spring Lock Washers.
- **Involute Gear Transmission Engine**: True involute tooth flank curve generator ($x(\phi), y(\phi)$) for Spur Gears, Helical Gears (axial helix angle $\beta$ with continuous tooth twist), and Bevel Gears (pitch cone angle taper).
- **Roller Chain Sprockets**: ANSI B29.1 standard roller chain sprockets (ANSI #25, #35, #40, #50, #60) with exact roller seating pockets, tooth crests, and mounting hub collars.

### 25. Finite Element Analysis (FEA) Structural Simulation & Stress Tensor Heatmaps
- **Linear Elasticity Solver & Tet4 Meshing**: Direct linear elasticity solver using linear 4-node tetrahedral elements (Tet4) with analytical volume evaluation, strain-displacement matrix $[B]$, and element stiffness $[k_e] = [B]^T [D] [B] V_e$.
- **Preconditioned Conjugate Gradient (PCG)**: Solves large sparse $[K] \{U\} = \{F\}$ systems for nodal displacement vectors.
- **Cauchy Stress Recovery & Von Mises Yield Criteria**: Recovers full 3D stress tensors across nodes, calculates Von Mises equivalent stress, and evaluates structural Factor of Safety ($\text{FoS} = \sigma_y / \sigma_v$).
- **Boundary Anchors & Body Loads**: Fixed/pinned support constraints, mechanical load vectors (N), gravitational body loads, and exaggerated deformation displacement overlays ($1\times - 1000\times$).

---

## Workspace Architecture

The codebase is organized into 7 decoupled, pure-Rust crates under `crates/`:

| Crate | Responsibilities | Key Dependencies |
| :--- | :--- | :--- |
| **`core-math`** | Primitive types (`Vec2`, `Vec3`, `Quat`, `Mat4`, `Ray`, `Aabb`), spatial transforms, geometric intersections. | `glam`, `serde` |
| **`geo-kernel`** | 2-Manifold BMesh Half-Edge kernel, Modals, Modifiers, Sculpting, Splines, UV Unwrapping, XPBD Cloth, SPH Fluids, STEP/STL/PLY/OBJ/glTF IO. | `core-math`, `rayon` |
| **`graph-engine`** | Maya DAG SceneGraph, World matrix cascades, Constraints, Analytic 2-Bone IK + Pole Angle, CCD IK, Animation Tracks, Motion Matching. | `core-math`, `geo-kernel` |
| **`geo-nodes`** | Visual geometry node graph evaluation, socket typing, topological sort, cycle prevention, procedural modifiers. | `core-math`, `geo-kernel` |
| **`render-viewport`** | Real-time wgpu 23 forward rasterizer, infinite grid, 3D transform gizmos, RTAO, NPR shaders, Spatial XR rig. | `core-math`, `geo-kernel`, `wgpu` |
| **`render-pathtracer`**| Multi-threaded Monte Carlo path tracer, SAH BVH, Principled BSDF, Bokeh camera, Hardware RT BLAS/TLAS buffers, Texture baking. | `core-math`, `geo-kernel`, `rayon`, `image` |
| **`app`** | Application lifecycle (`winit`), egui UI panels, drag-and-drop docking (`egui_dock`), commands, modals, Rhai/Python/Lua scripting console. | All crates, `egui`, `egui_dock`, `winit`, `rhai` |

---

## Architectural Compliance & Quality Standards

- **100% Pure Safe Rust**: Zero `unsafe` code blocks across the entire codebase (`zero unsafe` policy).
- **Zajfan Engineering Standard**: Every single `.rs` file strictly adheres to the $\le 630$ line ceiling. Monolithic files are broken into clean, cohesive submodules.
- **Zero Warnings**: Compiles completely clean under `cargo clippy --workspace -- -D warnings`.
- **Comprehensive Verification**: **438+ passing unit and integration tests** verifying geometry invariants, CAD precision, path tracing, and scene graph evaluation.
- **Clean Typography**: 100% 7-bit ASCII typography throughout the entire UI subsystem, preventing font fallback tofu glyphs.

---

## Quick Start

### Prerequisites
- [Rust Toolchain](https://rustup.rs/) (1.78+ stable).
- Vulkan, Metal, or DirectX 12 compatible graphics drivers.

### Build and Test
```bash
# Clone the repository
git clone https://github.com/The-No-Hands-company/Plumb-3D.git
cd Plumb-3D

# Run unit tests across all 7 workspace crates
cargo test --workspace

# Run Clippy with zero-warning enforcement
cargo clippy --workspace -- -D warnings

# Launch the interactive DCC studio application
cargo run -p app --release
```

---

## Essential Keyboard Shortcuts

| Category | Shortcut | Action |
| :--- | :--- | :--- |
| **Navigation** | `Alt + LMB` / `MMB` | Orbit camera around pivot |
| | `Alt + MMB` / `Shift + MMB` | Pan camera laterally |
| | `Mouse Wheel` | Smooth zoom in/out |
| | `F` / `Numpad .` | Frame camera view on selection |
| | `Numpad 1 / 3 / 7` | Front / Right / Top orthographic views |
| | `Numpad 5` | Toggle Perspective / Orthographic |
| | `Numpad 0` | Toggle Active Camera View / Free Orbit |
| **Selection & Mode** | `Tab` | Toggle Object Mode / Edit Mode |
| | `Space` / `Tab (Hold)` | 8-Directional Gestural Radial Pie Menu |
| | `1 / 2 / 3` | Vertex / Edge / Face selection levels |
| | `Shift + LMB` | Multi-select toggle |
| | `B` | Marquee box select |
| | `A` / `Alt + A` | Select All / Deselect All |
| | `L` / `Alt + L` | Select Linked Shell / Select Edge Loop |
| | `Ctrl + I` | Invert selection |
| **Transformation** | `W / E / R` | Translate / Rotate / Scale gizmos |
| | `,` (Comma) | Cycle Orientation (Global / Local / Normal / View) |
| | `V` | Precision Vertex Snapping toggle |
| | `Alt + W` | Align CAD Workplane to active face |
| | `Alt + X` | Symmetrize active geometry across Workplane |
| **Modeling Operators** | `E` | Extrude Region along normals |
| | `Shift + E` | Direct Push-Pull extrusion |
| | `Ctrl + B` | Bevel edges / faces (Mouse wheel adjusts segments) |
| | `I` | Inset faces (Press `I` to toggle Region/Individual) |
| | `K` | Interactive Knife bisect tool |
| | `Ctrl + R` | Loop Cut and Slide |
| | `C` | Direct Cutter modal (Box, Circle, Ngon, Radial) |
| | `K` / `Shift + K` | Hardware Kitbash modal tool |
| | `P` | 3D Bézier Curve Pen tool |
| | `Shift + E` / `Shift + V` | Edge Crease / Vertex Crease for SubD |
| | `J` / `V` | Connect vertices across face / Rip vertex |
| | `Alt + M` | Merge vertices at center |
| | `Delete` / `X` | Delete selected elements |
| **CAD & Engineering** | `Shift + Space` | 2D CAD Constraint Sketcher mode |
| | `Inspector > Assembly` | Multi-Body Mates, SAT Clash Check & Exploded View |
| | `Inspector > Drafting` | ANSI/ISO 2D Drawing Sheets, Vector HLR & GD&T |
| | `Inspector > Sheet Metal` | DIN 6935 K-Factor Flanges & 1-Click Flat-Pattern Unfolding |
| | `Inspector > Mechanical` | ISO Fasteners, Involute Gears & Roller Sprockets |
| | `Inspector > FEA` | Linear Elasticity, Von Mises Stress & Factor of Safety |
| **Windows & Studio** | `F1` | In-Software User Manual & Tutorials Hub |
| | `F2` | Welcome & Quick-Start Project Hub |
| | `F3` / `Ctrl + K` | Omnipresent Command Palette search |
| | `F12` | Production Path Tracer window |
| | `Ctrl + Z` / `Ctrl + Y` | Transactional Undo / Redo |
| | `Ctrl + S` | Save `.plmb` Project |

---

## Product Scope & Release Roadmap

Plumb 3D enforces a strict separation of concerns across major product generations. **Version 1.x is strictly The Modeler**, maintaining laser focus on pure modeling speed, CAD precision, and rock-solid geometry:

### Completed Releases (v1.0.0 – v1.12.0)
- **v1.0.0 (Commercial Hardened Foundation)**: 2-manifold BMesh Half-Edge kernel, 16-modifier stack, G2 CAD splines, STEP export, UV unwrapping, native `.plmb` container, 5 semantic design-token themes, arbitrary docking (`egui_dock`), and 3 interactive walkthroughs.
- **v1.1.0 (Precision CAD & Curve Drafting)**: 2D constraint sketcher on workplanes, live dimension callouts, and multi-rail curve lofts.
- **v1.2.0 (Universal Interop & Production Shading)**: Native glTF 2.0 / GLB container export & import with PBR materials, and micro-chamfer normal perturbation shader.
- **v1.3.0 (High-Speed Direct Modeling UX)**: Coplanar face-split polygon inscription and modal bi-directional push-pull carving (boss/cavity).
- **v1.4.0 (Field-Aligned Quad Retopology & Pixar SubD Limit Math)**: 4-RoSy cross-field curvature tensor alignment, OpenSubdiv fractional semi-sharp creases ($w \in [0.0, 10.0]$), and closed-form Catmull-Clark limit surface stencils.
- **v1.5.0 (Precision B-Rep & Cognitive Velocity)**: Analytic Trimmed NURBS surface patches, Surface-Surface Intersection (SSI) engine, STEP CAD solid importer, and 8-direction modal gesture Radial Pie Menu.
- **v1.6.0 (Class-A Surface Diagnostics & Universal CAD Interop)**: Dynamic Zebra reflection striping, real-time screen-space Gaussian & Mean curvature heatmaps, G0/G1/G2 surface continuity seam analysis, porcupine curvature combs, and ANSI/USAF IGES 5.3 CAD solid B-Rep importer and exporter.
- **v1.7.0 (Advanced B-Rep Operations & Class-A Surface Solids)**: Variable-radius rolling-ball & chordal fillet engine, N-sided Coons boundary patch filler with G1 Hermite boundary matching, and CAD solid shell hollowing and uniform thicken operator with corner mitering.
- **v1.8.0 (Multi-Body CAD Assemblies & Kinematics)**: MateConnector coordinate frames, 7 kinematic joint types (Revolute, Prismatic, Concentric, etc.), SAT 3D interference clash detection with penetration depth, and exploded view sequencer with 3D dashed trail lines.
- **v1.9.0 (Technical 2D Engineering Drawings, Vector HLR & GD&T)**: First & Third Angle orthographic projections, analytical vector Hidden Line Removal (HLR), ASME Y14.5 GD&T feature control frames, ISO A4–A0 & ANSI A–E sheets, and vector SVG & AutoCAD DXF exporters.
- **v1.10.0 (Sheet Metal Engine & Flat-Pattern Unfolding)**: DIN 6935 K-factor bend physics, parametric base & edge flanges, 1-click flat pattern unfolding, and multi-layer CNC DXF (`OUTER_CUT`, `BEND_UP`, `BEND_DOWN`) + SVG exports.
- **v1.11.0 (Mechanical Part Generators & Fastener Library)**: Parametric ISO/ANSI metric fasteners (M2–M30 bolts, screws, nuts, washers), analytical Involute Gears (Spur, Helical, Bevel), and ANSI roller chain sprockets (#25–#60).
- **v1.12.0 (Finite Element Analysis & Stress Tensor Heatmaps)**: Linear elasticity solver using Tet4 elements, Cauchy stress tensor recovery, Von Mises equivalent stress, and structural Factor of Safety (FoS) failure analysis.

### Target Releases to 100% CAD Modeler Completeness (v1.13.0 – v1.16.0)
- **v1.13.0**: *CAM & CNC G-Code Toolpath Generator* — Facing, 2.5D adaptive clearing roughing, ball-end surfacing, lead-in/lead-out arcs, and RS-274 standard G-Code export.
- **v1.14.0**: *Parametric Design Tables & Global Expression Variables* — Mathematical expression parser for dimensions (`Width * 0.5 + 2.0`) and CSV/TSV configuration tables for part families.
- **v1.15.0**: *Generative Design & Topology Optimization* — SIMP material penalization method, compliance minimization under volume constraints, and organic load-path lattice generation.
- **v1.16.0**: *100% CAD Modeler Feature Completeness & Final Polish* — Comprehensive CAD interop certification, multi-body kinematics verification, and performance benchmarks.

### Long-Term Generational Roadmap (v2.0+)
- **Plumb 3D v2.0: The Animator & DCC Suite** — Digital Clay Sculpting Engine, Armature skeletal hierarchies, 2-Bone/CCD IK solvers, weight painting, NLA clip mixing, dope sheet, and direct MP4/H.264 video container export.
- **Plumb 3D v3.0: The Renderer** — Full procedural material shading DAG node network (OpenPBR standard) and hardware GPU ray tracing pipeline (Vulkan RT / DXR).
- **Plumb 3D v4.0: The Simulator** — XPBD cloth & soft-body tearing, rigid-body fracturing, and FLIP liquid solvers.
- **Plumb 3D v5.0: The Studio Suite** — Pixar OpenUSD Stage referencing, Alembic point cache streaming, and multi-user collaborative networking.

---

## Documentation Index

- [User Guide](docs/USER_GUIDE.md): In-depth manual covering all tools, panels, modifiers, and workflows.
- [Developer Guide](docs/DEVELOPER_GUIDE.md): Codebase manual, architectural standards, and contributor guidelines.
- [Keyboard Shortcuts](docs/KEYBOARD_SHORTCUTS.md): Complete list of hotkeys, modals, and viewport gestures.
- [Beginner Tutorials](docs/TUTORIALS.md): 7 hands-on walkthroughs from hard-surface crates to organic sculpting.
- [Medieval Citadel Masterclass](docs/CITADEL_TUTORIAL.md): Complete architectural production masterclass.
- [Rhai Scripting API](docs/RHAI_SCRIPTING_API.md): API documentation for procedural scripting.
- [System Architecture](docs/architecture.md): Topology, crate graphs, and kernel guarantees.

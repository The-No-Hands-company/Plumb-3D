# Changelog

All notable changes to the Plumb 3D DCC Suite are documented in this file.
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2026-11-28 - Plumb 3D Modeler v1.0.0: The Hardened Commercial Release

### Core Architecture & Release Scope
- **Frozen Commercial Scope**: Plumb 3D Modeler v1.0.0 is dedicated and laser-focused strictly on **3D Modeling & LookDev**. Character animation, GPU ray tracing pipelines, and physics solvers are officially designated for major generational releases (`v2.0` Animator, `v3.0` Renderer, `v4.0` Simulator, `v5.0` Studio Suite).
- **Native `.plmb` Project Container**:
  - Deterministic chunked binary archive with `PLMB` magic bytes header (`*b"PLMB"`), embedded semantic versioning (`major.minor.patch`), and CRC32 integrity validation.
  - Zero-corruption crash recovery session autosaving (`~/.plumb3d/autosave/recovery_session.plmb`).
- **Interactive In-Viewport Walkthrough Triumvirate**:
  - **Sci-Fi Storage Crate (Box Modeling)**: 7-step guided hard-surface masterclass (Cube -> Face Selection -> Individual Inset -> Negative Extrude -> Bevel Chamfer -> Gunmetal PBR -> Beauty Snapshot).
  - **Medieval Castle Tower (Architectural)**: 7-step guided masonry workflow (Cylinder -> Parapet Inset -> Sunken Walkway -> Crenellations -> Weathered Granite PBR).
  - **Lunar Night Sky (LookDev & Astrophotography)**: High-resolution UV sphere, genuine NASA 2K cartographic regolith normal maps, sunlight terminator, and 360 ESO Milky Way skybox.
- **Welcome Hub Redesign**:
  - Replaced sprawling vertical template lists with a sleek, compact dropdown and live template preview card with 1-click `🚀 Start Project` launcher.
- **Production Studio LookDev Hardening**:
  - Direct PBR multi-pass stratified instant beauty snapshot (<0.15s, zero grain).
  - Automatic studio ground plane contact shadow catcher.
  - Automatic 3-point softbox studio lighting rig (Key, Fill, Rim).
  - Built-in À-Trous Wavelet denoiser enabled by default.
- **Modal Status Bar & Numeric Readouts**:
  - Live millimeter, angle, and distance feedback during active transforms, extrusions, and insets.
  - Real-time numerical keyboard typing buffer with Enter confirmation.
- **High-Poly Stress Benchmarking**:
  - Automated stress test suite verifying 100k+ polygon memory safety, 2-manifold BMesh invariants, sub-millisecond undo/redo throughput, and 60 FPS hover raycast picking.
- **Codebase Quality & Hardening**:
  - 330 automated tests passing with 0 failures across all 7 workspace crates.
  - 0 compiler warnings, 0 clippy warnings under `cargo clippy --workspace -- -D warnings`.
  - 100% pure safe Rust (`zero unsafe` blocks).
  - All files strictly formatted under `cargo fmt --all -- --check`.

### Added
- **Drag-and-Drop Dockable Workspace (`egui_dock`)**:
  - Full tab rearranging and panel docking across Viewport, Outliner, Inspector, Geometry Nodes, 2D UV Editor, Timeline, and Script Console.
  - Transparent 3D viewport canvas integration preventing render-pass occlusions.
- **UV Island Parameterization & ABF++ Distortion Analysis**:
  - Per-face angular and area stretch distortion heatmap visualization directly on 2D UV charts.
  - Interactive 2D Seam Cut Mode: Click edges directly in the 2D UV canvas to cut or clear seams in real-time.
- **CAD Precision Splines & NURBS**:
  - Multi-segment 3D cubic Bézier boolean spline trimming and parametric knot splitting.
  - Interactive tangent arc snapping ($G^1$ continuity) and dynamic workplane alignment (`Alt+W`).
  - STEP (ISO 10303-21) and Continuous NURBS export.
- **XPBD Cloth Self-Collision Hardening**:
  - $O(N)$ 3D spatial hash grid particle self-collision resolution preventing fabric self-penetration.
- **Skeletal Rigging & IK Hardening**:
  - Analytic 2-Bone IK solver with Pole Vector target, Pole Twist Angle ($-180^\circ$ to $+180^\circ$), and FK/IK weight blend.
- **Hardware Ray Tracing Acceleration**:
  - Multi-million polygon BLAS partitioning into 64K-triangle chunks with strict 256-byte alignment and 64-byte TLAS instance descriptors.
- **UI/UX Collision & Glyph Hardening**:
  - Moved outliner parenting inside the master scroll area, eliminating status bar overflow.
  - Replaced all non-ASCII emoji glyphs with standard 7-bit ASCII badges, eliminating font fallback tofu boxes.
  - Right-anchored panel toggles and isolated ribbon horizontal scroll.
  - 281 passing unit and integration tests across 7 workspace crates.
  - Strict Zajfan Engineering Line Ceiling ($\le 630$ lines per `.rs` file) and 100% pure safe Rust (`zero unsafe`).

## [1.0.0] - 2026-11-25 - Official Launch Release

### Added
- **Native OS File Dialogs**: Integrated native file choosers via `rfd` for Open, Save, Save As, and 3D Model Import.
- **Autosave & Crash Recovery**: Background session snapshot engine writing to `~/.plumb3d/autosave/` every 180 seconds with gold recovery banner on unexpected exit.
- **Hard-Surface Boolean Engine**: Watertight BSP-tree Constructive Solid Geometry (`Union`, `Difference`, `Intersection`) with $O(1)$ seam welding and degenerate polygon culling.
- **CAD Precision Snapping**:
  - Vertex magnetic snapping (`V` key toggle) with HUD coordinates and cyan glow ring.
  - Edge midpoint and polygon face centroid snapping.
  - Grid increment presets ($1\,\text{mm}$, $10\,\text{mm}$, $10\,\text{cm}$, $1\,\text{m}$) accessible via ribbon bar and inspector.
  - Rotation angle constraints holding `Ctrl` ($15^\circ$), `Shift` ($45^\circ$), or `Ctrl+Shift` ($90^\circ$).
- **Occlusion Culling**: Raycast depth and surface normal testing in solid shading mode preventing accidental click-throughs to back-facing geometry.
- **Cross-Platform Release Automation**: Added canonical scripts (`build.sh`, `build.cmd`, `build.ps1`, `package_release.sh`) and GitHub CI/CD workflows.

## [0.9.5] - 2026-11-24
- Hard-surface CSG Boolean hardening with coplanar plane clipping.
- CAD snapping indicators and angle constraint presets.

## [0.9.0] - 2026-11-24
- Native OS dialog integration for STL, OBJ, glTF, and PLY exports.
- Viewport sub-element occlusion culling and back-face filtering in marquee select.
- Background autosave manager and session restore prompt.

## [0.8.5] - 2026-11-23
- 2-Manifold BMesh Half-Edge geometric kernel.
- WGPU 23 hardware-accelerated viewport with MatCap, Studio PBR, and Film LookDev tonemapping.
- Pure-Rust Monte Carlo path tracer with BVH traversal and thin-lens bokeh depth of field.

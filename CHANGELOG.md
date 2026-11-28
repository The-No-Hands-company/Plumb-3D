# Changelog

All notable changes to the Plumb 3D DCC Suite are documented in this file.
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.12.0] - 2026-11-28 - Finite Element Analysis (FEA) Structural Simulation & Stress Tensor Heatmaps

### Added
- **Linear Elasticity Solver & Tet4 Finite Elements**:
  - Direct linear elasticity solver using 4-node tetrahedral elements (Tet4) with analytical volume evaluation, strain-displacement matrix $[B]$, and element stiffness $[k_e] = [B]^T [D] [B] V_e$.
  - Preconditioned Conjugate Gradient (PCG) solver for global linear systems $[K] \{U\} = \{F\}$.
- **Standard Structural Materials**:
  - Isotropic material models with Young's Modulus ($E$), Poisson's Ratio ($\nu$), Yield Strength ($\sigma_y$), and Density ($\rho$).
  - Built-in presets for ASTM A36 Structural Steel, Aluminum 6061-T6, Titanium Ti-6Al-4V Grade 5, PLA Plastic (3D print), and ABS Plastic.
- **Cauchy Stress Tensor Recovery & Failure Analysis**:
  - Recovers 3D Cauchy stress tensor $\boldsymbol{\sigma} = [\sigma_{xx}, \sigma_{yy}, \sigma_{zz}, \tau_{xy}, \tau_{yz}, \tau_{zx}]$ across elements and nodes.
  - Computes Von Mises equivalent stress $\sigma_v = \sqrt{\frac{1}{2}\left[(\sigma_{xx}-\sigma_{yy})^2 + (\sigma_{yy}-\sigma_{zz})^2 + (\sigma_{zz}-\sigma_{xx})^2 + 6(\tau_{xy}^2 + \tau_{yz}^2 + \tau_{zx}^2)\right]}$.
  - Evaluates structural Factor of Safety ($\text{FoS} = \sigma_y / \sigma_v$) and detects plastic yielding.
- **Structural Boundary Conditions & Body Loads**:
  - Fixed / pinned support constraints locking translational degrees of freedom.
  - Point and distributed mechanical load vectors (Newtons).
  - Gravitational body forces ($\vec{g} = [0, 0, -9.81]\,\text{m/s}^2$).
  - Exaggerated deformation displacement visualization ($1\times$ to $1000\times$).
- **Inspector UI & Commands**:
  - Dedicated **FEA** Inspector tab with live material presets, boundary condition setups, and safety factor evaluation.
  - `RunFeaSimulationCommand` with full undo/redo.
  - Command Palette shortcut for instant structural simulation execution.

## [1.11.0] - 2026-11-28 - Mechanical Part Generators & Fastener Library

### Added
- **ISO / ANSI Standard Fastener Library**:
  - Full parametric dimensional generator covering standard metric thread sizes from M2 through M30.
  - Supports ISO 4014 / DIN 931 Hex Head Bolts, ISO 4762 / DIN 912 Hexagon Socket Head Cap Screws (SHCS), ISO 4032 / DIN 934 Hex Nuts, ISO 7089 / DIN 125 Flat Washers, and DIN 127 Helical Spring Lock Washers.
  - Standard head width across flats ($s$), head heights ($k$), nominal diameters, and metric coarse pitches.
  - Cosmetic and physical thread rendering modes.
- **Involute Gear & Power Transmission Engine**:
  - Analytical involute tooth flank curve generator ($x(\phi), y(\phi)$) ensuring true meshing contact without undercut.
  - Supports Spur Gears (straight teeth), Helical Gears (axial helix angle $\beta$ with continuous tooth twist), and Bevel Gears (pitch cone angle taper for $90^\circ$ shaft drives).
  - Parametric Module $m$, teeth $Z$, pressure angle $\alpha$ ($20^\circ / 14.5^\circ$), face width $b$, and shaft bore diameter.
  - Automatic calculation of pitch diameter, base diameter, addendum, dedendum, and root fillet curves.
- **ANSI Roller Chain Sprockets**:
  - Conforms to ANSI B29.1 standard roller chain pitches: ANSI #25 ($1/4"$), ANSI #35 ($3/8"$), ANSI #40 ($1/2"$), ANSI #50 ($5/8"$), and ANSI #60 ($3/4"$).
  - Exact roller seat seating radius, tooth flank curve, outside diameter, and optional mounting hub boss collar.
- **Inspector UI & Commands**:
  - Dedicated **Mechanical** Inspector tab with 3 sub-categories (Fasteners, Involute Gears, Sprockets), live kinematic readouts, and 1-click scene spawners.
  - Undoable commands: `SpawnFastenerCommand`, `SpawnGearCommand`, and `SpawnSprocketCommand`.
  - Command Palette shortcuts for instant mechanical component creation.

## [1.10.0] - 2026-11-28 - Sheet Metal Engine & Flat-Pattern Unfolding (K-Factor CNC Manufacturing)

### Added
- **DIN 6935 K-Factor & Neutral Axis Analytical Mechanics**:
  - Exact neutral axis shift $y = K \cdot T$, Bend Allowance ($\text{BA} = \theta \cdot \frac{\pi}{180}(R + KT)$), Outside Setback ($\text{OSSB} = (R+T)\tan(\frac{\theta}{2})$), and Bend Deduction ($\text{BD} = 2 \cdot \text{OSSB} - \text{BA}$).
  - Built-in material presets for Mild Steel ($K=0.44$), Aluminum ($K=0.40$), and Stainless/High-Strength Steel ($K=0.50$).
  - Corner relief notch configurations: Rectangular, Tear (slit), and Round.
- **Parametric Sheet Metal Builder**:
  - `SheetMetalBuilder::create_base_flange`: constructs uniform thickness solid plates with topological quads.
  - `SheetMetalBuilder::add_edge_flange`: generates smooth cylindrical arc bends with inner and outer concentric radius tessellation, followed by angled planar flange extensions.
- **1-Click Flat-Pattern Unfolding Engine**:
  - Unfolds 3D sheet metal brackets into 2D cut blanks compensated by neutral-axis elongation.
  - Extracts outer cutting boundary polygon + bend centerlines with angle, direction (`Up` vs `Down`), radius, and allowance metadata.
  - **AutoCAD DXF Exporter**: multi-layer DXF with `OUTER_CUT`, `BEND_UP`, and `BEND_DOWN`.
  - **SVG Exporter**: styled vector graphic with color-coded dashed bend lines and callout badges.
- **Interactive UI & Commands**:
  - Dedicated `SheetMetal` Inspector tab and Command Palette shortcuts.
  - `CreateBaseFlangeCommand`, `AddEdgeFlangeCommand`, and `UnfoldSheetMetalCommand` with full undo/redo.

## [1.9.0] - 2026-11-28 - Technical 2D Engineering Drawings, Hidden Line Removal (HLR) & GD&T

### Added
- **First-Angle (ISO) & Third-Angle (ANSI) Orthographic Projections**:
  - Analytical camera view projections for Front, Top, Right, Left, Bottom, Rear, and Isometric ($30^\circ / 30^\circ$) views.
- **Analytical Hidden Line Removal (HLR) Vector Engine**:
  - Rigorous face-edge occlusion classification separating visible solid lines (`HlrLineStyle::VisibleSolid`) from hidden dashed lines (`HlrLineStyle::HiddenDashed`) and silhouette outlines (`HlrLineStyle::SilhouetteOutline`).
- **ASME Y14.5 Geometric Dimensioning & Tolerancing (GD&T)**:
  - Feature Control Frames supporting Position, Flatness, Perpendicularity, Concentricity, and Runout with Maximum/Least Material Condition callouts.
  - Linear witness dimensions with arrowheads, diametral callouts, and center marks.
- **Standard Drawing Sheet Exporter**:
  - ISO A4–A0 and ANSI A–E standard sheets with formal ASME title blocks and automated 4-view generation.
  - Production-ready vector SVG and AutoCAD DXF multi-view drawing sheet exporters.

## [1.8.0] - 2026-11-28 - Multi-Body CAD Assemblies, Kinematic Mates & Interference Clash Detection

### Added
- **Multi-Body CAD Assembly Kinematics**:
  - `MateConnector` coordinate frame attachments (Origin, Up, Normal).
  - Kinematic mate types: Coincident, Concentric, Revolute, Prismatic, Cylindrical, Gear, and Fixed with alignment options (`Aligned` vs `AntiAligned`).
  - Degree-of-Freedom (DOF) solver with interactive joint actuation.
- **Separating Axis Theorem (SAT) 3D Interference Clash Detection**:
  - AABB broadphase bounding volume acceleration + narrowphase 3D triangle-triangle SAT intersection.
  - Penetration depth computation and clash categorization (`Interference` collision vs `Clearance` hazard).
- **Interactive Exploded View Sequencer**:
  - Exploded view sequencing with interactive progress scrubber ($t \in [0.0, 1.0]$) and 3D dashed trail line generation.

## [1.7.0] - 2026-11-28 - Advanced B-Rep Operations & Class-A Surface Solids

### Added
- **Variable-Radius Rolling-Ball & Chordal Fillet Engine**:
  - Edge rail filleting smoothly tapering from radius $r_1 \to r_2$ with $G^1$ circular rolling-ball and $G^2$ curvature-continuous blend surface transitions.
- **N-Sided Trim Patch Filler (Coons & Gordon Surface Networks)**:
  - Creates watertight patches over 3-sided, 5-sided, or $N$-sided curve boundaries with tangent matching.
- **CAD Solid Shell Hollowing & Uniform Thicken Operator**:
  - Shell hollowing with interior cavity offsetting, open surface sheet thickening, and mitered corner stitching.

## [1.6.0] - 2026-11-28 - Class-A Surface Diagnostics & Universal CAD Interop

### Added
- **ANSI/USAF IGES 5.3 Solid B-Rep Interop**:
  - Native IGES 5.3 importer and exporter supporting Entity 128 (Rational B-Spline Surfaces), Entity 126 (B-Spline Curves), and Entity 110 (Lines).
- **Class-A Surface Diagnostics Suite**:
  - Real-time dynamic Zebra reflection striping for surface tangency ($G^1$) and curvature ($G^2$) inspection.
  - Real-time screen-space Gaussian & Mean curvature heatmaps.
  - $G^0 / G^1 / G^2$ surface continuity seam analysis.
  - Curvature needle combs (porcupine plots) displaying surface normal inflection and curvature spikes.

## [1.5.0] - 2026-11-28 - Precision B-Rep & Cognitive Velocity

### Added
- **Analytic Trimmed NURBS Surface Engine**:
  - Trimmed NURBS surface patches with 2D trimming curve boundary loops in UV parametric space ($[0,1] \times [0,1]$).
  - Surface-Surface Intersection (SSI) engine computing transverse intersection curves between adjacent NURBS patches.
- **STEP (ISO 10303-21) Solid B-Rep Importer**:
  - Parses and reconstructs STEP manifold solid B-Rep face networks into native geometry.
- **8-Directional Modal Gesture Radial Pie Menu**:
  - Instant circular pie menu triggered via `Space` / `Tab` with directional flick gestures for Extrude, Inset, Bevel, Knife, Push-Pull, Loop Cut, and Sub-element selection.

## [1.4.0] - 2026-11-28 - Field-Aligned Quad Retopology & Pixar OpenSubdiv Limit Math

### Added
- **4-RoSy Cross-Field Curvature Tensor Alignment**:
  - Principal curvature estimation and field-guided quad remeshing aligning quad edges with underlying principal stress lines.
- **Pixar OpenSubdiv Limit Math & Fractional Creasing**:
  - Fractional semi-sharp creasing ($w \in [0.0, 10.0]$) with closed-form Catmull-Clark limit surface stencils.

## [1.3.0] - 2026-11-28 - High-Speed Direct Modeling UX

### Added
- **Coplanar Face-Split Polygon Inscriber**:
  - Inscribes quads, circles, and regular $N$-gons on planar faces with automatic edge splitting.
- **Modal Bi-Directional Push-Pull Carving**:
  - Dynamic boss extrusion and cavity subtraction with automatic boundary boolean stitching.

## [1.2.0] - 2026-11-28 - Universal Interop & Production Shading

### Added
- **Native glTF 2.0 / GLB Container Import & Export**:
  - Full binary glTF roundtrip with embedded PBR metallic-roughness materials and texture maps.
- **Micro-Chamfer Normal Perturbation Shader**:
  - Real-time procedural edge rounding shader providing synthetic fillets on hard edges without polygon count increases.

## [1.1.0] - 2026-11-28 - Precision CAD & Curve Drafting

### Added
- **2D Parametric Constraint Sketcher**:
  - Workplane drafting engine supporting Points, Lines, Circles, Arcs, and geometric constraints.
  - Multi-rail curve lofting and skinning.
  - Live dimension callouts and distance inspection overlays.

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

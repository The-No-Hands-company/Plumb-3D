# Plumb 3D User Guide

Welcome to the **Plumb 3D User Guide**! This comprehensive manual walks you through the concepts, tools, shortcuts, and workflows of this next-generation 3D modeling and rendering suite engineered in 100% Safe Pure Rust.

---

## Table of Contents
1. [User Interface Layout & Docking System](#1-user-interface-layout--docking-system)
2. [Viewport Navigation & Camera Controls](#2-viewport-navigation--camera-controls)
3. [Selection & Transform Gizmos](#3-selection--transform-gizmos)
4. [Mesh Modeling & Interactive Modals](#4-mesh-modeling--interactive-modals)
5. [CAD Precision Drafting & NURBS Splines](#5-cad-precision-drafting--nurbs-splines)
6. [Hard-Surface Cutters & Kitbash Modals](#6-hard-surface-cutters--kitbash-modals)
7. [3ds Max-Style Parametric Modifier Stack](#7-3ds-max-style-parametric-modifier-stack)
8. [Digital Sculpting & Dynamic Topology](#8-digital-sculpting--dynamic-topology)
9. [Texture Painting & Polypaint](#9-texture-painting--polypaint)
10. [UV Unwrapping & ABF++ Distortion Analysis](#10-uv-unwrapping--abf-distortion-analysis)
11. [Physics, XPBD Cloth & Fluid Particles](#11-physics-xpbd-cloth--fluid-particles)
12. [Skeletal Rigging, Constraints & 2-Bone IK](#12-skeletal-rigging-constraints--2-bone-ik)
13. [Animation Timeline & Motion Matching](#13-animation-timeline--motion-matching)
14. [Visual Geometry Nodes](#14-visual-geometry-nodes)
15. [Multi-Engine Scripting (Rhai, Python, Lua)](#15-multi-engine-scripting-rhai-python-lua)
16. [Production Path Tracer & Hardware RT](#16-production-path-tracer--hardware-rt)
17. [CAM 3D Print Quality Inspector & Slicing](#17-cam-3d-print-quality-inspector--slicing)
18. [Class-A Surface Diagnostics & Curvature Analysis](#18-class-a-surface-diagnostics--curvature-analysis)
19. [Multi-Body CAD Assemblies & Kinematics](#19-multi-body-cad-assemblies--kinematics)
20. [Technical 2D Engineering Drawings & GD&T](#20-technical-2d-engineering-drawings--gdt)
21. [Sheet Metal Design & CNC Flat-Pattern Unfolding](#21-sheet-metal-design--cnc-flat-pattern-unfolding)
22. [Gestural Modeling & 8-Directional Radial Pie Menu](#22-gestural-modeling--8-directional-radial-pie-menu)
23. [Mechanical Part Generators, Fasteners & Involute Gears](#23-mechanical-part-generators-fasteners--involute-gears)

---

## 1. User Interface Layout & Docking System

The workspace is organized into clean, focused functional areas designed to eliminate panel overlap and visual clutter:

- **Top App Menu Bar**: Desktop menus (`File`, `Edit`, `Add`, `Render`, `Window`, `Help`) along with quick-access buttons (`Render F12`, `Help F1`, `Welcome F2`, `Search F3`).
- **Workspace Ribbon Header Bar**: Quick-access pills for Interaction Modes (`Object`, `Edit`, `Sculpt`, `Curve`), Sub-Element levels (`Vert (1)`, `Edge (2)`, `Face (3)`), Transform tools (`Move (W)`, `Rotate (E)`, `Scale (R)`), Coordinate spaces (`Global`, `Local`, `Normal`, `View`), Shading modes (`Studio`, `PBR`, `Normals`, `Zebra`, `Rendered`), Snapping presets, and panel toggles.
- **Drag-and-Drop Docking Workspace (`egui_dock`)**:
  - Toggle via **Window -> Drag-and-Drop Docking Workspace**.
  - Allows clicking and dragging any tab to rearrange the workspace into custom split panes.
  - Supported dockable tabs: `3D Viewport`, `Scene Outliner`, `Inspector`, `Geometry Nodes`, `2D UV Editor`, `Animation Timeline`, and `Script Console`.
- **Scene Outliner (Left Panel)**: Hierarchical DAG scene graph displaying Meshes, Lights, Cameras, Joints, and Splines. Features group parenting (`P`), unparenting (`Shift+P`), mesh joining (`Ctrl+J`), and integrated vertical scrolling that prevents status bar clipping.
- **Properties Inspector (Right Panel)**: Deep inspector providing access to Transform parameters, 3ds Max Modifier Stack, PBR Shading, Brushes, Rigging Constraints, and SubD NURBS topology.
- **Status Bar (Footer)**: Real-time telemetry: FPS, interaction mode, gizmo space, camera projection, vertex/edge/face counts, and selection counts.

---

## 2. Viewport Navigation & Camera Controls

- **Orbiting**: Hold `Alt + Left Mouse Button` or press `Middle Mouse Button (MMB)` and drag to tumble around the active pivot.
- **Panning**: Hold `Alt + MMB` or `Shift + MMB` and drag to pan laterally.
- **Zooming**: Scroll the `Mouse Wheel` up/down to zoom smoothly.
- **Focusing (`F`)**: Press `F` or `Numpad .` to instantly frame the camera view onto the selected object or sub-elements.
- **Orthographic Snapping**:
  - `Numpad 1`: Front View
  - `Numpad 3`: Right View
  - `Numpad 7`: Top View
  - `Numpad 5`: Toggle Perspective / Orthographic projection
  - `Numpad 0`: Toggle Active Camera View / Free Orbit
- **6-Axis Navigation Compass**: Click the colored axes ($+X, -X, +Y, -Y, +Z, -Z$) in the top-right corner to snap to orthogonal projections; drag the sphere to orbit.

---

## 3. Selection & Transform Gizmos

### Selection Modes
- **Object Mode (`Tab`)**: Click any model to select it. The active object receives a bright selection wireframe and corner bounds.
- **Edit Mode (`Tab`)**: Sub-element selection levels:
  - `1`: Vertex selection level.
  - `2`: Edge selection level.
  - `3`: Face selection level.
- **Multi-Select**: Hold `Shift` while clicking to add or remove elements.
- **Box Select (`B`)**: Click and drag a marquee rectangle across the screen.
- **Select All (`A`) / Deselect All (`Alt + A`)**.
- **Select Linked (`L`)**: Hover over geometry and press `L` to select all connected manifold geometry.
- **Edge Loop (`Alt + L`)**: Select continuous topological quad edge rings.
- **Invert Selection (`Ctrl + I`)**: Invert the current selection set.

### Transform Gizmos
- `W` (Translate): 3D arrowhead cones for axial movement, planar tiles ($XY, XZ, YZ$), and center ring for screen-space movement.
- `E` (Rotate): Smooth rings around each principal axis with angle readout.
- `R` (Scale): 3D cubes on axis tips for axial scaling, and a center cube for uniform scaling.
- **Coordinate Space (`,`)**: Press `,` (Comma) to cycle between Global, Local, Normal, and View coordinates.
- **Precision Snapping (`V`)**: Toggle vertex, midpoint, face center, and grid magnetic snapping.

---

## 4. Mesh Modeling & Interactive Modals

All modeling tools operate on a **2-Manifold BMesh Half-Edge kernel** with complete undo/redo history (`Ctrl+Z` / `Ctrl+Y`).

### Extrude (`E`)
Select faces (or edges) and press `E`. Drag the mouse to pull new geometry outward along surface normals. Watertight side-wall polygons are generated automatically. Click `LMB` or press `Enter` to confirm.

### Direct Push-Pull (`Shift + E`)
Select a face and press `Shift+E` (or click-drag a face directly). Extruding outward adds volume; pushing inward automatically creates recessed pockets or boolean cutouts.

### Bevel (`Ctrl + B`)
Select edges or faces and press `Ctrl+B`. Drag the mouse to expand bevel width. **Scroll the Mouse Wheel** while dragging to increase or decrease the number of bevel profile segments.

### Inset (`I`)
Select faces and press `I`. Drag inward to create an inset boundary ring. Press `I` again during the drag to toggle between:
- **Region Inset**: Insets the outer perimeter of contiguous selected faces as a single patch.
- **Individual Inset**: Insets each selected face independently.

### Interactive Knife Tool (`K`)
Press `K` to enter Knife cut mode. The crosshair snaps to vertices and edges. Click `LMB` to place cut points. Press `Enter` or `Space` to commit the cut; the engine bisects polygons and heals half-edge twin pointers cleanly.

### Loop Cut & Slide (`Ctrl + R`)
Press `Ctrl+R` and hover over a quad face. A preview cut ring appears perpendicular to the hovered edge. Click `LMB` to confirm the cut ring, then slide the mouse along the loop to position the cut. Click `LMB` again to finalize.

### Bridge Edge Loops
Select two boundary edge loops and trigger **Bridge Edge Loops** in the Mesh menu or context menu to loft connecting quad strips with twist and segment controls.

---

## 5. CAD Precision Drafting & NURBS Splines

- **Interactive Bézier Pen Tool (`P`)**: Click to place anchor points; drag to adjust smooth tangent handles. Press `C` to close the curve or `Enter` to commit to the scene graph.
- **Boolean Spline Trimming**: Intersect arbitrary 3D cubic Bézier curves; the solver computes exact intersection parameters, inserts subdivision knots, and trims segments inside or outside boundaries.
- **Continuous Tangent Arc Fillets ($G^1$)**: Replaces sharp curve corners with circular fillet arcs ensuring first-derivative continuity.
- **Cursor Tangent Snapping**: Cursor automatically snaps to curve tangents and normal projections for precision CAD drafting.
- **Pipe Sweeps & Revolve (Lathe)**: Generate hollow tubes or solid pipes along 3D splines, or revolve profiles around axes into rotationally symmetric vessels.
- **CAD Construction Workplane (`Alt+W`)**: Align the dynamic workplane to any polygonal face. Draw curves and transform objects directly in the aligned construction frame. Press `Alt+X` to symmetrize across the workplane.
- **STEP (ISO 10303-21) Export**: Export bicubic B-spline NURBS surfaces directly into industry-standard CAD formats.

---

## 6. Hard-Surface Cutters & Kitbash Modals

- **Direct Cutter Modal (`C`)**:
  - **Box Cutter**: Drag a rectangular cutting profile across the surface; push inward to create clean subtractive boolean cavities.
  - **Circle Cutter**: Draw circular counter-bores and through-holes.
  - **Ngon Cutter**: Click custom polygonal boundary points to cut complex architectural recesses.
  - **Radial Compound Cutter**: Circular array cutting for bolt patterns, vents, and turbine casings.
  - **Ghost Cutter Preview**: Move and rotate cutting shapes in 3D space with `W/E/R` before committing booleans.
- **Hardware Kitbash Tool (`K` / `Shift + K`)**:
  - Drop industrial hex bolts, allen screws, slotted vents, and latches directly onto surfaces with surface-normal alignment.
- **Conformal Decals & Weld Beads**: Project floating surface details and procedural welding beads along sharp seam edges.

---

## 7. 3ds Max-Style Parametric Modifier Stack

Located in the Properties Inspector on the right, the non-destructive modifier stack allows stacking procedural operations:

- **Generate**: SubD Catmull-Clark (with semi-sharp edge/vertex crease weights `Shift+E` / `Shift+V`), Bevel Chamfer, Array, Solidify, Screw (Lathe), Triangulate, Wireframe.
- **Boolean**: Real-time Constructive Solid Geometry (`Union`, `Difference`, `Intersection`) referencing other scene nodes.
- **Deform**: Simple Deform (Bend, Twist, Taper, Stretch), Lattice FFD, Curve Deform, Shrinkwrap, Morpher (Blendshapes), Linear Blend Skinning.
- **Decimate**: Garland-Heckbert Quadric Error Metric (QEM) polygon reduction preserving silhouettes.
- **Simulation**: XPBD Cloth, SPH Fluid Particles, Rigid Bodies.

---

## 8. Digital Sculpting & Dynamic Topology

Switch to **Sculpt Mode** via the mode selector in the top ribbon:

- **Brushes**: Clay, Grab (Move), Crease, Smooth, Inflate, Flatten, Pinch, Scrape, SnakeHook.
- **Dynamic Topology (DynTopo)**: Automatically tessellates and subdivides edges under the brush stroke, allowing unlimited freeform sculpting without pre-subdividing the base mesh.
- **Symmetry**: Live mirror sculpting across $X, Y, Z$ planes with multi-point visual feedback.
- **Brush Controls**: Adjust Radius with `[` and `]`, or interactively in the Inspector.

---

## 9. Texture Painting & Polypaint

- **Vertex Color Painting**: Paint RGBA colors directly onto mesh vertices with smooth spherical falloff brushes.
- **Weight Painting**: Assign bone influence weights or modifier density masks with color-coded heatmaps (Blue $= 0.0 \to$ Red $= 1.0$).
- **Linear Gradient Tool**: Click-and-drag linear falloff gradients across vertices.
- **Cavity & Curvature Shading**: Real-time geometric curvature preview emphasizing crevices and high-frequency edges.

---

## 10. UV Unwrapping & ABF++ Distortion Analysis

Open the **2D UV Editor** via **Window -> 2D UV Editor**:

- **Unwrapping Algorithms**: Conformal Tutte Disk Harmonic parameterization, LSCM unwrap, and Smart UV Angle Projection.
- **ABF++ Distortion Heatmap**: Check **Heatmap** to render per-face distortion gradients directly over the 2D canvas:
  - Deep Blue ($D=0.0$): Perfect isometric / conformal match.
  - Cyan / Green: Low distortion.
  - Yellow / Red: High angle shear or area compression.
- **Interactive 2D Seam Cut Mode**: Click **Cut Seams [ON]** to interactively click edges in the 2D UV view to cut or clear seams.
- **UV Island Packing & Tools**: Pack Islands with uniform texel density, Rotate CW/CCW, Flip Horizontal/Vertical, Scale, and Laplacian UV Relaxation.

---

## 11. Physics, XPBD Cloth & Fluid Particles

- **XPBD Cloth Simulator**:
  - Extended Position-Based Dynamics with stretch/shear constraints, isometric bending, pneumatic pressure inflation, and ground plane collision.
  - **Spatial Hash Grid Self-Collision**: $O(N)$ neighborhood search prevents cloth from penetrating through its own folds.
- **SPH Fluid Dynamics**: Smoothed-Particle Hydrodynamics simulating water droplets and viscous fluids.
- **Rigid Body Dynamics**: Gravity, linear impulse, and restitution collisions against ground and obstacle colliders.
- **Simulation Baking**: One-click bake of physics simulations into keyframed animation tracks.

---

## 12. Skeletal Rigging, Constraints & 2-Bone IK

- **Armatures & Bones**: Add joint hierarchies and automatically bind skin envelopes with heat diffusion weighting.
- **Analytic 2-Bone IK Solver**:
  - Law of Cosines solver with **Pole Vector target**, **Pole Twist Angle ($ -180^\circ$ to $+180^\circ$)**, and **FK/IK Weight Blend**.
- **Cyclic Coordinate Descent (CCD) IK**: Solves multi-joint kinematic chains with damping and target reach tolerance.
- **Parametric Constraints**: Aim (Look-At with World Up), Parent (offset maintaining), and Spline Path Follow.

---

## 13. Animation Timeline & Motion Matching

- **Timeline Controls**: Frame scrubbing, playback/loop, start/end frame range, and current frame readout.
- **Keyframe Tracks**: Dopesheet view with Linear, Step, and Smooth Bézier spline interpolation curves.
- **Inertialization Smooth Decay**: Blends jarring animation transitions into natural continuous motion.
- **Motion Matching Database**: Real-time trajectory prediction and feature cost querying for responsive character locomotion.

---

## 14. Visual Geometry Nodes

Open the **Geometry Nodes Canvas** via **Window -> Geometry Nodes**:

- Visual Directed Acyclic Graph (DAG) for procedural modeling.
- **Nodes**: Primitives (Cube, Cylinder, Sphere, Grid), Transforms, Noise Displacement, Point Scattering / Instancing, Curve Fillet/Revolve, Math operations, and Boolean CSG.
- Host any node graph as a live modifier in an object's stack.

---

## 15. Multi-Engine Scripting (Rhai, Python, Lua)

Open the **Script Console** via **Window -> Script Console**:

- **Native Rhai**: 100% pure-safe Rust scripting runtime with memory caps and instruction step limits. First-class access to math vectors, meshes, and scene nodes.
- **Python 3 & Lua 5.4**: Integrated multi-engine runners for external pipeline scripts.
- **Interactive REPL**: Multiline script editor with stdout capture and execution timing.

---

## 16. Production Path Tracer & Hardware RT

Press **`F12`** or select **Render -> Production Path Tracer**:

- Multi-threaded CPU Monte Carlo path tracing engine powered by **Rayon**.
- Cook-Torrance Microfacet specular BRDF, Lambertian diffuse, dielectric Fresnel glass transmission, and metallic reflections.
- Thin-lens optical camera with physical bokeh depth-of-field.
- **Hardware RT Acceleration**: Multi-million polygon BLAS partitioning into 64K-triangle chunks with strict 256-byte alignment and 64-byte TLAS instance descriptors.
- **Built-in LookDev Compositor**: Multi-pass separable Gaussian Bloom glow, corner vignette falloff, saturation/contrast curve grading, and ACES Filmic tone mapping.
- PBR Texture Map Baker: Tangent-space Normal, Ambient Occlusion, and Displacement baking.

---

## 17. CAM 3D Print Quality Inspector & Slicing

Open via **Render -> 3D Print Quality Inspector**:

- **Watertightness & Manifold Validation**: Detects non-manifold boundary edges and flipped face normals.
- **Overhang Angle Heatmap**: Highlights faces exceeding support thresholds ($>45^\circ$).
- **Volume & Weight Estimation**: Real-time calculations of enclosed volume ($\text{cm}^3$) and PLA filament weight.
- **CAM Layer Slicing & G-code Export**: Computes horizontal layer contours, infill paths, and exports standard G-code for 3D printers.

---

## 18. Class-A Surface Diagnostics & Curvature Analysis

Accessed via the **CAD Precision & Diagnostics** inspector or Shading menu:

- **Dynamic Zebra Reflection Striping**: Simulates parallel fluorescent lighting tubes reflected across surfaces.
  - Aligned, unbroken stripes indicate **$G^1$ Tangency**.
  - Smoothly curving, non-kinked stripes verify **$G^2$ Curvature Continuity**.
  - Adjustable stripe frequency, angle, and high-contrast black/white or chrome palettes.
- **Screen-Space Curvature Heatmaps**:
  - **Gaussian Curvature ($K = \kappa_1 \kappa_2$)**: Distinguishes synclastic dome regions ($K > 0$), anticlastic saddle/hyperbolic regions ($K < 0$), and developable flat/cylindrical surfaces ($K = 0$).
  - **Mean Curvature ($H = \frac{\kappa_1 + \kappa_2}{2}$)**: Visualizes overall surface tension and minimal surface properties.
  - Colormaps: Turbo, Cool-to-Warm, Jet, and Grayscale with numeric min/max bounds.
- **$G^0 / G^1 / G^2$ Seam Continuity Diagnostics**:
  - Automatically iterates boundary edges between adjacent surface sheets to detect positional gaps ($G^0$), dihedral angle kinks ($G^1$), and radius discontinuities ($G^2$).
- **Porcupine Curvature Combs**:
  - Renders 3D needle vectors along curve rails showing local curvature magnitude and inflection points where curvature crosses zero.

---

## 19. Multi-Body CAD Assemblies & Kinematics

Accessed via the **Assembly** tab in the Inspector:

- **MateConnectors**:
  - Local coordinate reference frames placed on vertices, edges, face centers, or cylinder axes with explicit Origin, Up, and Normal vectors.
- **7 Kinematic Joint Types**:
  - **Coincident**: Locks translational separation between planar faces.
  - **Concentric**: Aligns central cylindrical axes (pins, shafts, bushings).
  - **Revolute**: Single rotational degree of freedom (hinge, pivot).
  - **Prismatic**: Single linear sliding degree of freedom along an axis.
  - **Cylindrical**: Combined rotation and sliding along a shared cylinder axis.
  - **Gear**: Coupled rotational ratio ($N_1 : N_2$) between gear bodies.
  - **Fixed**: Rigidly locks all 6 degrees of freedom.
  - **Alignment Flips**: Toggle between `Aligned` and `AntiAligned` orientations.
- **Separating Axis Theorem (SAT) 3D Clash Detection**:
  - AABB broadphase bounding volume filtering for instant rejection of non-overlapping bodies.
  - Narrowphase exact triangle-triangle SAT intersection detecting spatial interferences.
  - Computes penetration depth and categorizes contacts into hard **Interference** clashes vs safe **Clearance** margins.
- **Exploded View Sequencer**:
  - Multi-step assembly explosion sequencing with interactive scrub slider ($0.0 \to 1.0$).
  - Procedural 3D dashed trail lines linking exploded components back to their assembled positions.

---

## 20. Technical 2D Engineering Drawings & GD&T

Accessed via the **Drafting** tab in the Inspector:

- **Orthographic Multi-View Projection Engine**:
  - Supports **First-Angle (ISO)** and **Third-Angle (ANSI)** projection standards.
  - Automatically generates 4 standard orthographic views: Front, Top, Right, and Isometric ($30^\circ/30^\circ$).
- **Analytical Vector Hidden Line Removal (HLR)**:
  - Raycasts and surface occlusion tests classify every model edge:
    - `VisibleSolid`: Unobstructed foreground edges (rendered in solid black).
    - `HiddenDashed`: Occluded internal edges (rendered in dashed lines).
    - `SilhouetteOutline`: Exterior boundary profile silhouettes.
- **ASME Y14.5 Geometric Dimensioning & Tolerancing (GD&T)**:
  - Feature Control Frames: Position, Flatness, Perpendicularity, Concentricity, and Runout.
  - Material condition symbols: Maximum Material Condition (MMC - $\textcircled{\text{M}}$), Least Material Condition (LMC - $\textcircled{\text{L}}$), and Regardless of Feature Size (RFS).
  - Parametric witness dimension lines, diametral callouts ($\varnothing$), and center marks.
- **Standard Drawing Sheet Templates & Vector Export**:
  - ISO standards: A4, A3, A2, A1, A0.
  - ANSI standards: A (Letter), B (Tabloid), C, D, E.
  - Formal title blocks with Part Name, Material, Scale, Date, and Engineering approval.
  - 1-click export to **AutoCAD DXF** and high-resolution **Vector SVG**.

---

## 21. Sheet Metal Design & CNC Flat-Pattern Unfolding

Accessed via the **Sheet Metal** tab in the Inspector:

- **DIN 6935 K-Factor Bend Physics**:
  - Accurate neutral axis mechanics accounting for sheet metal stretching and compression during press-brake air bending.
  - Calculates Bend Allowance ($\text{BA}$), Outside Setback ($\text{OSSB}$), and Bend Deduction ($\text{BD}$).
  - Standard material presets: Mild Steel ($K=0.44$), Aluminum ($K=0.40$), and Stainless Steel ($K=0.50$).
  - Built-in corner relief notches: Rectangular, Tear (slit), and Round.
- **Base Flange & Edge Flange Builders**:
  - **Base Flange**: Creates planar solid sheet metal blanks with uniform thickness.
  - **Edge Flange**: Bends selected boundary edges upward or downward at specified angles (e.g. $90^\circ$, $45^\circ$) with true concentric cylindrical bend transitions and straight flange extensions.
- **1-Click CNC Flat-Pattern Unfolding**:
  - Flattens 3D folded brackets into 2D cut patterns using neutral axis compensation.
  - Extracts outer cutting perimeter and press-brake bend centerlines with angle, direction (`BEND UP` / `BEND DOWN`), and allowance callouts.
  - **Multi-Layer AutoCAD DXF Exporter**: Generates industry-standard CAD layers (`OUTER_CUT` for laser/waterjet, `BEND_UP` and `BEND_DOWN` for press brake CNC).
  - **SVG Exporter**: Vector drawing with color-coded bend dashed lines.

---

## 22. Gestural Modeling & 8-Directional Radial Pie Menu

- **Radial Pie Menu (`Space` or Hold `Tab`)**:
  - Instant circular menu centered at the mouse cursor. Flicking the mouse in one of the 8 cardinal directions triggers modeling operators with muscle memory speed:
    - **East ($0^\circ$)**: Extrude Region (`E`)
    - **North-East ($45^\circ$)**: Direct Push-Pull (`Shift+E`)
    - **North ($90^\circ$)**: Bevel Edge / Chamfer (`Ctrl+B`)
    - **North-West ($135^\circ$)**: Loop Cut and Slide (`Ctrl+R`)
    - **West ($180^\circ$)**: Inset Face (`I`)
    - **South-West ($225^\circ$)**: Align CAD Workplane (`Alt+W`)
    - **South ($270^\circ$)**: Interactive Knife Bisect (`K`)
    - **South-East ($315^\circ$)**: Cycle Sub-element level (Vertex $\to$ Edge $\to$ Face)
- **CAD Workplane Snapping (`Alt+W`)**:
  - Instantly aligns an infinite planar construction grid to any selected face or normal vector.
  - Symmetrize geometry across the active workplane using `Alt+X`.

---

## 23. Mechanical Part Generators, Fasteners & Involute Gears

Accessed via the **Mechanical** tab in the Inspector or the Command Palette (`F3`):

### 1. ISO / ANSI Standard Fasteners
- **Standard Thread Sizes**: Covers M2, M2.5, M3, M4, M5, M6, M8, M10, M12, M16, M20, M24, and M30.
- **Fastener Styles**:
  - **ISO 4014 / DIN 931 Hex Head Bolt**: Hexagonal head with standardized width across flats ($s$), head height ($k$), and cylindrical shank of custom length ($L$).
  - **ISO 4762 / DIN 912 Socket Head Cap Screw (SHCS)**: Cylindrical head with internal hex drive socket.
  - **ISO 4032 / DIN 934 Hex Nut**: Hexagonal nut body with internal clearance bore.
  - **ISO 7089 / DIN 125 Flat Washer**: Standard plain flat washer disc ($d_1$ inner, $d_2$ outer, $s$ thickness).
  - **DIN 127 Lock Washer**: Helical split spring lock washer with continuous elevation ramp and split gap.
- **1-Click Placement**: Spawns watertight 2-manifold BMesh geometry directly into the active scene.

### 2. Involute Gear Transmission Engine
- **Kinematic Transmission Styles**:
  - **Spur Gear**: Straight teeth extruded axially along face width $b$.
  - **Helical Gear**: Continuous cylindrical tooth twist determined by the helix angle $\beta$ ($\Delta\theta = \frac{b \tan\beta}{r}$).
  - **Bevel Gear**: Conical gear teeth tapering towards a pitch cone apex for intersecting $90^\circ$ shaft drives.
- **AGMA / ISO Tooth Profile Calculations**:
  - Standard Module $m$ (mm), tooth count $Z$, and pressure angle $\alpha$ ($20.0^\circ$ standard, $14.5^\circ$).
  - Pitch diameter $d = m \cdot Z$, base radius $r_b = r \cdot \cos\alpha$.
  - Addendum $h_a = 1.0 \cdot m$, dedendum $h_f = 1.25 \cdot m$.
  - True analytical involute curve generation ($x = r(\phi)\cos\theta, y = r(\phi)\sin\theta$) preventing undercut.
  - Center shaft mounting bore with parametric diameter.

### 3. Roller Chain Sprockets (ANSI B29.1)
- **Chain Pitch Classes**: ANSI #25 ($1/4"$), ANSI #35 ($3/8"$), ANSI #40 ($1/2"$), ANSI #50 ($5/8"$), ANSI #60 ($3/4"$).
- **Parametric Tooth Geometry**:
  - Pitch Circle Diameter $PCD = \frac{p}{\sin(180^\circ / N)}$.
  - Bottom Root Diameter $BD = PCD - d_r$ with cylindrical roller seating pockets.
  - Outside Tip Diameter $OD = p \cdot (0.6 + \cot(180^\circ / N))$.
  - Optional mounting hub collar with parametric hub diameter and extension length.

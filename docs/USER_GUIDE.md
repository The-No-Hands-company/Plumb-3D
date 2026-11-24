# DCC Suite User Guide

Welcome to the **DCC Suite User Guide**! This comprehensive manual walks you through the concepts, tools, shortcuts, and workflows of this next-generation 3D modeling and rendering package.

---

## Table of Contents
1. [User Interface Layout](#1-user-interface-layout)
2. [Viewport Navigation & Camera Controls](#2-viewport-navigation--camera-controls)
3. [Selection & Transform Gizmos](#3-selection--transform-gizmos)
4. [Mesh Modeling & Interactive Modals](#4-mesh-modeling--interactive-modals)
5. [Right-Click (RMB) Context Menu](#5-right-click-rmb-context-menu)
6. [3ds Max-Style Parametric Modifier Stack](#6-3ds-max-style-parametric-modifier-stack)
7. [Digital Sculpting Mode](#7-digital-sculpting-mode)
8. [Precision Spline Curves & Pipes](#8-precision-spline-curves--pipes)
9. [UV Unwrapping & 2D UV Editor](#9-uv-unwrapping--2d-uv-editor)
10. [Visual Geometry Nodes](#10-visual-geometry-nodes)
11. [Embedded Rhai Scripting](#11-embedded-rhai-scripting)
12. [Offline Production Path Tracer (F12)](#12-offline-production-path-tracer-f12)
13. [CAD & 3D Printing Export](#13-cad--3d-printing-export)

---

## 1. User Interface Layout

The workspace is organized into clean, focused functional areas:

- **Top App Menu Bar**: Provides standard desktop menus (`File`, `Edit`, `Add`, `Mode`, `Viewport`, `Shading`, `UV`, `Render`, `Window`, `Help`) along with editor quick-launch pills (`Render F12`, `Rhai Console`, `Geo Nodes`, `UV Editor`, `Help F1`).
- **Viewport Ribbon Header**: Fast access to active Interaction Modes (`[ 📦 Object | ✏ Edit | 🎨 Sculpt | 〰 Curve ]`), Sub-Element levels (`[ ⏺ Vert | ➖ Edge | ⏹ Face ]`), Transform tools (`[ ✥ Move | ⟲ Rotate | ⤢ Scale ]`), Coordinate space (`Global / Local / Normal / View`), Shading modes, and viewport display toggles.
- **Left Tool Strip**: Vertical quick-action shelf (Plasticity / Blender T-Shelf style) for instant switching between Select, Move, Rotate, Scale, Measure, Align, Sculpt, and specialized editors.
- **Scene Outliner (Left Panel)**: Maya-style DAG scene graph displaying all nodes (Meshes, Lights, Cameras, Joints, Curves) in a hierarchical tree. Supports parenting, grouping, and visibility toggling. Collapsible via the `◀` header button.
- **Properties Inspector (Right Panel)**: Deep inspector displaying Object Info, Transform parameters, PBR Material slots, the 3ds Max Modifier Stack, Vertex Groups, Constraints, and BMesh operators. Collapsible via the `▶` header button.
- **3D Viewport**: Hardware-accelerated wgpu canvas featuring infinite ground grid, 6-Axis navigation orbit sphere, active selection outlines, and 3D transform gizmos.
- **Animation Timeline & Dope Sheet (Bottom)**: Frame scrubber, playback controls, start/end range, and keyframe markers.
- **Status Bar (Footer)**: Live statistics: FPS, active mode, coordinate space, camera projection, vertex/edge/face counts, and selection metrics.

---

## 2. Viewport Navigation & Camera Controls

- **Orbiting**: Hold `Alt + Left Mouse Button` or press `Middle Mouse Button (MMB)` and drag to tumble the viewport around the active pivot.
- **Panning**: Hold `Alt + MMB` or `Shift + MMB` and drag to pan horizontally and vertically.
- **Zooming**: Scroll the `Mouse Wheel` up/down to zoom smoothly.
- **Focusing (`F`)**: Press `F` or `Numpad .` to instantly frame the camera view onto the selected object or sub-elements.
- **Orthographic Snapping**:
  - `Numpad 1`: Front View
  - `Numpad 3`: Right View
  - `Numpad 7`: Top View
  - `Numpad 5`: Toggle Perspective ⇄ Orthographic
- **Interactive 6-Axis Compass**: Click the colored bubbles ($+X, -X, +Y, -Y, +Z, -Z$) in the top-right corner of the viewport to instantly snap the camera to that view angle, or drag the compass sphere to orbit.

---

## 3. Selection & Transform Gizmos

### Selection Modes
- **Object Mode (`Tab`)**: Click any 3D model to select it. The active object receives a bright selection wireframe and corner bounds.
- **Edit Mode (`Tab`)**: Switch to sub-element editing. Use the number keys:
  - `1`: Vertex selection level.
  - `2`: Edge selection level.
  - `3`: Face selection level.
- **Multi-Select**: Hold `Shift` while clicking to add or remove elements.
- **Box Select (`B`)**: Click and drag a marquee rectangle across the screen.
- **Select All (`A`) / Deselect All (`Alt + A`)**.
- **Select Linked (`L`)**: Hover over or select a vertex and press `L` to select all contiguous manifold geometry.
- **Edge Loop (`Alt + L`)**: Select continuous topological quad edge rings.

### Transform Gizmos
- `W` (Translate): 3D arrowhead cones for $X, Y, Z$ motion, planar squares ($XY, XZ, YZ$) for planar constrained translation, and center ring for screen-parallel motion.
- `E` (Rotate): Smooth rings around each principal axis, plus an outer trackball circle.
- `R` (Scale): 3D cubes on axis tips for axial scaling, and a center cube for uniform 3D scale.
- **Coordinate Space (`,`)**: Press `,` (Comma) to cycle between Global, Local, Normal, and View coordinates.
- **Snapping (`Shift + Tab`)**: Snaps translation to the grid or vertex positions.
- **Proportional Editing (`O`)**: Enables smooth soft-selection falloff.

---

## 4. Mesh Modeling & Interactive Modals

All modeling tools in Edit Mode operate on a **2-Manifold BMesh Half-Edge kernel** with undoable transactional commands (`Ctrl+Z` / `Ctrl+Y`).

### Extrude (`E`)
Select one or more faces (or edges) and press `E`. Drag the mouse to pull new geometry outward along surface normals. The tool dynamically constructs watertight side-wall polygons. Click `LMB` or press `Enter` to confirm; press `Esc` or `RMB` to cancel.

### Bevel (`Ctrl + B`)
Select edges or faces and press `Ctrl+B`. Drag the mouse to expand the bevel distance. **Scroll the Mouse Wheel** while dragging to increase or decrease the number of bevel profile segments.

### Inset (`I`)
Select faces and press `I`. Drag the mouse inward to create an inset boundary ring. Press `I` again during the drag to toggle between:
- **Region Inset**: Insets the outer perimeter of contiguous selected faces as a single patch.
- **Individual Inset**: Insets each selected face independently.

### Interactive Knife Tool (`K`)
Press `K` to enter Knife cut mode. Move your mouse across the mesh; the crosshair snaps to vertices and edges. Click `LMB` to place cut points. Press `Enter` or `Space` to commit the cut; the engine bisects the polygons and updates half-edge twin pointers cleanly.

### Loop Cut & Slide (`Ctrl + R`)
Press `Ctrl+R` and hover over a quad face. A cut preview ring appears perpendicular to the hovered edge. Click `LMB` to confirm the cut ring, then slide the mouse along the quad loop to position the cut. Click `LMB` again to finalize.

### Semi-Sharp Creasing (`Shift + E` & `Shift + V`)
With a Subdivision or Catmull-Clark modifier active, select edges and press `Shift+E`. Drag mouse to set edge crease weights from `0.0` (fully smooth) to `1.0` (infinitely sharp). Press `Shift+V` to crease vertex corners.

### Connect Vertices (`J`) & Rip (`V`)
Select two non-adjacent vertices on a polygonal face and press `J` to split the face with a new edge. Select vertices and press `V` to rip them apart, creating a seam.

---

## 5. Right-Click (RMB) Context Menu

Right-click anywhere in the viewport to open a context-sensitive floating menu:
- In **Object Mode**: Smooth/Flat shading, Recalculate Normals, Parenting, Join Meshes, Separate Loose Parts, Delete.
- In **Edit Mode**: Instant access to Face tools (Extrude, Inset, Bevel, Poke, Triangulate, Bisect), Edge tools (Bevel, Loop Cut, Knife, Sharp, Seams, Slide), and Vertex tools.
- In **Sculpt Mode**: Brush picker (Draw, Clay, Inflate, Smooth, Grab, etc.) with Radius and Strength sliders.
- In **Empty Space**: Quick Add primitives (Cube, Cylinder, Sphere, Plane, Cone, Torus), lights, and cameras.

---

## 6. 3ds Max-Style Parametric Modifier Stack

Modifiers apply non-destructive, parametric operations on top of base meshes in the Inspector:

- **Subdivision & Catmull-Clark**: Smooths geometry while honoring semi-sharp edge/vertex creases.
- **Decimate (Garland-Heckbert QEM)**: Reduces polygon count by computing quadratic error metrics, preserving silhouette edges.
- **Solidify**: Extrudes an inner or outer shell with uniform thickness—essential for 3D printing hollow models.
- **Array**: Replicates geometry with constant or relative offsets across multiple copies.
- **Mirror**: Non-destructively mirrors across the $X, Y,$ or $Z$ plane with automatic center-weld.
- **Boolean**: Real-time CSG Union, Difference, and Intersection with target objects.
- **Cloth Simulation**: Simulates physical fabric using Verlet integration with distance constraints, ground floor collision, and vertex group pinning masks.
- **Particle System**: Emits particles with initial velocity along surface normals, simulated under gravity, bounce restitution, and drag.
- **Deformers**: Lattice FFD, Curve Deform, Simple Deform (Bend, Twist, Taper, Stretch), Shrinkwrap, and Weighted Normal.

---

## 7. Digital Sculpting Mode

Press `Tab` and switch to **Sculpt Mode** or click the Sculpt pill in the viewport ribbon:
- **9 High-Performance Brushes**:
  - `Draw`: Pushes vertices outward along surface normals.
  - `Clay`: Builds up layered organic volume like clay strips.
  - `Inflate`: Expands vertices omnidirectionally.
  - `Smooth`: Relaxes mesh curvature and relieves surface tension.
  - `Flatten`: Projects vertices toward an average local plane.
  - `Grab`: Drags a cluster of vertices in screen space.
  - `Pinch`: Contracts vertices toward the stroke center.
  - `Crease`: Carves sharp valleys or ridges.
  - `SnakeHook`: Extrudes organic tentacles and horns from the mesh.
- **Falloffs**: Switch between Cubic, Smooth, Linear, and Sharp curves.

---

## 8. Precision Spline Curves & Pipes

Create parametric cubic Bézier curves via `Add -> Curve`:
- **Tangent Handles**: Select knot points in Curve Edit Mode to translate in/out handles.
- **Curve-to-Mesh (Sweep)**: Adjust `Bevel Radius` in Inspector to turn any 2D/3D spline curve into a watertight 3D pipe mesh.
- **Profile Shapes**: Choose between Circle, Square, Rectangle, Star, Triangle, and Hexagon cross-section profiles.

---

## 9. UV Unwrapping & 2D UV Editor

Access the UV menu or open the **2D UV Editor Window** (`🗺 UV Editor`):
- **Mark Seams (`✂ Mark Seam`)**: Select boundary edges in Edit Mode and mark them as UV cutting seams (displayed in orange).
- **Unwrap (`U`)**: Unwraps 3D mesh islands into 2D UV texture space using conformal boundary relaxation.
- **Smart UV Project**: Automatically detects sharp face angle thresholds and packs separated planar islands with configurable margins.
- **Projections**: Fast Planar, Box / Cube, Cylindrical, and Spherical UV generation.

---

## 10. Visual Geometry Nodes

Build generative models visually using Directed Acyclic Graph (DAG) nodes:
1. Open the canvas via **`🌿 Geo Nodes`** in the top menu or left toolbar.
2. Pan the infinite canvas with `MMB` and zoom with the scroll wheel.
3. Click **`+ Add Node`** to place Primitives (Cube, Sphere, Cylinder, Grid), Transforms, Math operations, Noise Displacement, or Point Scattering.
4. Drag from output sockets (right) to input sockets (left). Data types are strictly color-coded (Green = Mesh, Yellow = Vector, Blue = Float).
5. In the object Inspector, add a **🌿 Geometry Nodes** modifier to apply the visual graph non-destructively to your 3D scene object.

---

## 11. Embedded Rhai Scripting

For technical artists and pipeline automation, open the **`📜 Rhai`** console:
- Fast, 100% safe Rust scripting runtime without crashes or segmentation faults.
- Create procedural buildings, mathematical lattices, and automated scene setups.
- See the complete [Rhai Scripting API Reference](RHAI_SCRIPTING_API.md) for methods, types, and copy-pasteable script templates.

---

## 12. Offline Production Path Tracer (F12)

When ready to produce a final, photo-realistic image:
1. Press **`F12`** or select **`Render -> Render Image`**.
2. The Production Render Dialog opens and starts rendering immediately.
3. **Resolution Presets**: Choose from 360p, 720p, 1080p Full HD, or 1440p QHD.
4. **Quality / SPP**: Adjust samples per pixel from 16 spp (rapid preview) to 512 spp (grain-free presentation).
5. **Physical Camera Bokeh**: In the Scene Camera parameters, configure `Aperture Radius` and `Focal Distance` to render real optical depth of field with circular bokeh.
6. **Save Image**: Click **`💾 Save Image (PNG)`** once the render completes.

---

## 13. CAD & 3D Printing Export

Export calibrated, watertight models under **`File -> Export`**:
- **STL (.stl)**: Binary or ASCII format for slicing software (UltiMaker Cura, PrusaSlicer, Bambu Studio).
- **PLY (.ply)**: For 3D scanning, point cloud visualization, and color vertex data.
- **glTF 2.0 (.gltf)** & **Wavefront OBJ (.obj)**: For game engine integration (Unreal Engine 5, Unity, Godot).

---

## Need More Help?
Press **`F1`** inside the application at any time to open the built-in, searchable User Manual and Shortcuts Reference!

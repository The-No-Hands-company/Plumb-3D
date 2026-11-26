# Plumb 3D Beginner Modeling Tutorials

Welcome to the **Plumb 3D Beginner Tutorials**! This guide contains 7 comprehensive, hands-on tutorials designed to teach you the fundamentals of 3D modeling, sculpting, procedural design, and rendering across different styles and industries.

---

## Table of Tutorials
1. [Tutorial 1: Sci-Fi Cargo Crate (Hard-Surface Box Modeling)](#tutorial-1-sci-fi-cargo-crate-hard-surface-box-modeling)
2. [Tutorial 2: Medieval Castle Tower (Architectural Modeling)](#tutorial-2-medieval-castle-tower-architectural-modeling)
   - 🏰 **[Full Masterclass: Medieval Citadel & Fortress Architectural Scene](CITADEL_TUTORIAL.md)** (Multi-mesh fortress, lighting rig, PBR materials, and LookDev rendering)
3. [Tutorial 3: Alien Creature Bust (Organic Digital Sculpting)](#tutorial-3-alien-creature-bust-organic-digital-sculpting)
4. [Tutorial 4: Industrial Pipe & Conduit Harness (Spline Curves)](#tutorial-4-industrial-pipe--conduit-harness-spline-curves)
5. [Tutorial 5: Procedural Sci-Fi City (Visual Geometry Nodes)](#tutorial-5-procedural-sci-fi-city-visual-geometry-nodes)
6. [Tutorial 6: Studio Lighting & Bokeh Render (Monte Carlo Path Tracer)](#tutorial-6-studio-lighting--bokeh-render-monte-carlo-path-tracer)
7. [Tutorial 7: Watertight Geometric Vase for 3D Printing (CAD & Slicers)](#tutorial-7-watertight-geometric-vase-for-3d-printing-cad--slicers)

---

## Tutorial 1: Sci-Fi Cargo Crate (Hard-Surface Box Modeling)

```text
       +-----------------------+
      / |                     /|
     /  |     [ Recessed ]   / |
    +---+-------------------+  |
    |   |     |       |     |  |
    |   |     +-------+     |  |
    |   |                   |  |
    |   +-------------------+--+
    |  /                     | /
    | /      [ Crate ]       |/
    +------------------------+
```

### Overview & Learning Goals
- **Category**: Hard-Surface / Game Prop Asset.
- **Workflow**: Classical Box Modeling.
- **Key Operators**: Inset (`I`), Extrude Region (`E`), Bevel Modifier (`Ctrl+B`), and PBR Metallic Shaders.
- **Estimated Time**: 8-10 minutes.

### Step-by-Step Walkthrough

#### Step 1: Base Primitive Setup
1. Launch the DCC application or select **`File -> New Scene`**.
2. A default cube ($2.0\text{m} \times 2.0\text{m} \times 2.0\text{m}$) is automatically present at the origin.
3. Verify you are in **Object Mode** (top ribbon shows `[ 📦 Object ]`). Press **`Tab`** to switch to **Edit Mode** (`[ ✏ Edit ]`).

#### Step 2: Insetting Outer Border Plates
1. Press **`3`** on your keyboard to switch to **Face Selection Level** (or click `[ ⏹ Face ]` in the ribbon).
2. Click **Left Mouse (LMB)** on the top quad face of the cube to select it.
3. Press **`I`** to activate the **Inset** operator.
4. Move your mouse slightly inward to create a $0.15\text{m}$ border frame. Click **LMB** or press **`Enter`** to confirm.

#### Step 3: Extruding Recessed Paneling
1. With the newly inset face still selected, press **`E`** to activate the **Extrude** operator.
2. Move your mouse downwards along the surface normal by approximately $-0.2\text{m}$ into the cube.
3. Click **LMB** to confirm. The engine automatically creates watertight side-wall polygons connecting the outer frame to the recessed panel.

#### Step 4: Replicating across Side Panels
1. Orbit the viewport (`Alt + LMB` or `MMB`).
2. Select the front face of the cube.
3. Press **`I`**, drag inward to match the border frame, and confirm.
4. Press **`E`**, move inward $-0.2\text{m}$, and confirm.
5. Repeat for the remaining side faces. Your cube now resembles a heavy reinforced sci-fi storage container!

#### Step 5: Applying Non-Destructive Bevel Modifier
1. Press **`Tab`** to return to **Object Mode**.
2. In the right-hand **Inspector** panel, locate the **3ds Max Modifier Stack**.
3. Click **`+ Bevel`** in the modifier strip.
4. In the Bevel modifier controls:
   - Set **Width**: `0.04m`
   - Set **Segments**: `2`
5. The sharp geometric edges across your entire crate instantly receive smooth, light-catching chamfers.

#### Step 6: Assigning PBR Metallic Materials
1. In the Inspector, open the **Material Slots & PBR Shading** panel.
2. Set **Base Color** to dark titanium slate (`R: 0.18, G: 0.20, B: 0.24`).
3. Set **Metallic**: `0.85`.
4. Set **Roughness**: `0.28`.
5. Rotate your camera in the viewport to admire the realistic specular reflection across the beveled edges!

---

## Tutorial 2: Medieval Castle Tower (Architectural Modeling)

```text
           |---|---|---|   <- Crenellations (Battlements)
         ================= <- Corbelled Parapet
            |         |
            |  Stone  |
            |  Tower  |
            |         |
         ===============   <- Base Plinth
```

### Overview & Learning Goals
- **Category**: Environment Architecture.
- **Workflow**: Cylindrical Wall Modeling, Interactive Loop Cutting & Insetting.
- **Key Operators**: Cylinder Primitive, Inset (`I`), Extrude (`E`), Knife Bisect (`K`).
- **Estimated Time**: 10-12 minutes.

### Step-by-Step Walkthrough

#### Step 1: Base Cylinder
1. Go to **`File -> New Scene`**.
2. Click **`Add Primitive -> Cylinder`** in the top menu or ribbon.
3. In the Inspector under Mesh Properties, configure:
   - **Radius**: `2.5m`
   - **Height**: `8.0m`
   - **Segments**: `16`
4. Press **`F`** to frame the tower in your viewport.

#### Step 2: Parapet Walkway Inset
1. Press **`Tab`** to enter **Edit Mode**.
2. Press **`3`** for **Face Level**.
3. Select the circular top cap face of the cylinder.
4. Press **`I`** and drag inward by approximately $0.4\text{m}$ to create the outer stone perimeter wall. Confirm with **LMB**.
5. With the central disc face selected, press **`E`** and push downward into the tower by $-1.2\text{m}$ to create the sunken parapet walkway. Confirm with **LMB**.

#### Step 3: Outer Corbel Overhang
1. Press **`2`** for **Edge Level** (or click `[ ➖ Edge ]`).
2. Press **`Ctrl + R`** (Loop Cut) and hover over the upper third of the cylinder. Click **LMB** to place a horizontal cut ring, and slide it to $0.8\text{m}$ below the top rim. Click **LMB** to finalize.
3. Press **`3`** for **Face Level**. Hold **`Shift`** and select the ring of faces around the top rim.
4. Press **`E`** and scale outward slightly (`R`) to create the classic medieval flared corbelled parapet.

#### Step 4: Battlements & Crenels
1. Select alternating outer rim faces around the top parapet.
2. Press **`E`** to extrude them upward by $0.8\text{m}$.
3. You now have authentic medieval crenellations (protective merlons and embrasures).

#### Step 5: Granite Stone Shading
1. In the Inspector under **Material Slots**, set **Base Color** to a weathered granite tone (`R: 0.45, G: 0.44, B: 0.42`).
2. Set **Roughness** to `0.90` (matte stone) and **Metallic** to `0.0`.

---

## Tutorial 3: Alien Creature Bust (Organic Digital Sculpting)

```text
              .-""""-.
            /          \   <- Cranium / Brow Ridge
           |  (o)  (o)  |  <- Orbital Eye Sockets
           |    /\      |
            \  '--'    /   <- Jaw / Chin
             '--------'
```

### Overview & Learning Goals
- **Category**: Character Art & Organic Sculpting.
- **Workflow**: Digital sculpting brushes, dynamic falloffs, and mesh smoothing.
- **Key Brushes**: Grab, Clay, Draw, Inflate, SnakeHook, and Smooth.
- **Estimated Time**: 12-15 minutes.

### Step-by-Step Walkthrough

#### Step 1: Base Sculpting Sphere
1. Clear the scene (**`File -> New Scene`**).
2. Add a **UV Sphere** (**`Add Primitive -> Sphere`**).
3. In the Inspector under **Modifier Stack**, click **`+ Subdivide`** (level 2) to increase vertex resolution for sculpting.

#### Step 2: Entering Sculpt Mode
1. Click the **`🎨 Sculpt`** pill in the top ribbon header.
2. Your cursor transforms into a circular brush with interactive radius falloff.

#### Step 3: Establishing the Skull Silhouette (Grab Brush)
1. Select the **`Grab`** brush from the vertical left tool strip or Right-Click (RMB) context menu.
2. In the Inspector (or RMB menu), set **Brush Radius** to `1.2m` and **Brush Strength** to `0.8`.
3. Click and pull the lower front vertices downward to stretch an alien jawline.
4. Click and pull the upper rear vertices backward to elongate the cranium.

#### Step 4: Building Musculature & Brow Ridge (Clay & Draw Brushes)
1. Switch to the **`Clay`** brush.
2. Set **Radius** to `0.35m`.
3. Stroke across the forehead horizontally to build up an aggressive brow ridge.
4. Stroke vertically along the jaw and neck to add fibrous muscle tendons.

#### Step 5: Carving Eye Sockets (Inflate / Inverted)
1. Switch to the **`Inflate`** brush.
2. Reduce **Radius** to `0.25m`.
3. Set **Brush Strength** to `-0.7` (negative displacement).
4. Click and hold over both eye positions to carve deep orbital sockets into the skull.

#### Step 6: Extruding Horns & Tendrils (SnakeHook Brush)
1. Switch to the **`SnakeHook`** brush.
2. Set **Radius** to `0.30m` and **Strength** to `1.0`.
3. Click near the temple or crest of the skull and drag outward in an arc. The mesh dynamically pulls out tapering organic horns!

#### Step 7: Polishing Surface Tension (Smooth Brush)
1. Switch to the **`Smooth`** brush (or hold **`Shift`** while using any brush).
2. Lightly stroke over rough polygonal faceting to relax surface curvature into sleek, natural skin.

---

## Tutorial 4: Industrial Pipe & Conduit Harness (Spline Curves)

```text
       Knot 0         Handle
         O~~~~~~~~~~~~~~o
          \
           \   Bézier Curve Path
            \
             O====================O  <- Swept 3D Pipe
           Knot 1               Knot 2
```

### Overview & Learning Goals
- **Category**: Industrial Design & Hard-Surface CAD.
- **Workflow**: Parametric cubic Bézier curves, tangent handle shaping, and automated pipe sweep.
- **Key Features**: Curve Edit Mode, Knot Extrusion (`E`), Bevel Radius sweep, and Profile Shapes.
- **Estimated Time**: 6-8 minutes.

### Step-by-Step Walkthrough

#### Step 1: Adding a 3D Spline Curve
1. Click **`Add Curve -> 3D Spline Curve`** in the top menu.
2. A cubic Bézier curve with initial control knots appears centered in the viewport.

#### Step 2: Entering Curve Edit Mode
1. In the top ribbon header, click the **`〰 Curve`** mode pill.
2. Knot points appear as circular vertices with outward-extending tangent handles.

#### Step 3: Shaping the Curve Path
1. Click on any knot point to select it.
2. Press **`W`** to activate the 3D Translation Gizmo.
3. Drag the gizmo handles to position the knot in 3D space.
4. Click on a tangent handle endpoint and drag to adjust the curvature and bend radius of the spline path.

#### Step 4: Extruding New Sequential Knots
1. Select the last knot of the curve.
2. Press **`E`** (Extrude). A new sequential Bézier knot is generated and placed along the path direction.
3. Repeat to route your pipe through 3D space (around machinery, corners, or bulkheads).

#### Step 5: Converting Curve into a 3D Pipe Mesh
1. In the right-hand **Inspector**, locate **Curve Properties**.
2. Set **Bevel Radius** to `0.15m`.
3. Set **Bevel Resolution** to `12`.
4. The engine instantly sweeps a watertight cylindrical mesh along the exact trajectory of your curve!

#### Step 6: Customizing Profile Shapes
1. Change **Profile Shape** from `Circle` to `Square` to create ventilation ducting.
2. Change to `Hexagon` to create high-pressure hydraulic hose fittings.

---

## Tutorial 5: Procedural Sci-Fi City (Visual Geometry Nodes)

```text
  [ Grid Primitive ] ──(Mesh)──> [ Noise Displace ] ──(Mesh)──> [ Group Output ]
         │                              │
     (Transform)                    (Amplitude)
```

### Overview & Learning Goals
- **Category**: Procedural & Generative Modeling.
- **Workflow**: Visual Node Graph editing (Unreal Blueprints / Blender Geo Nodes style).
- **Key Concepts**: Socket dataflow, DAG evaluation, procedural noise displacement, and modifier binding.
- **Estimated Time**: 8-10 minutes.

### Step-by-Step Walkthrough

#### Step 1: Opening the Node Canvas
1. In the top application menu bar or toolbar, click **`🌿 Geo Nodes`** (or toggle `Window -> Geometry Nodes Canvas`).
2. An infinite, grid-based node editor window opens.
3. Pan the canvas by dragging with **Middle Mouse Button (MMB)**; zoom with the scroll wheel.

#### Step 2: Adding Generator Nodes
1. Right-click on the node canvas (or click **`+ Add Node`** in the canvas header).
2. Under **Inputs / Primitives**, add a **Grid Primitive** node.
3. Under **Geometry Ops**, add a **Noise Displace** node.
4. Under **Outputs**, ensure a **Group Output** node is placed.

#### Step 3: Connecting Sockets
1. Click and drag a connection wire from the green **Mesh** output socket of the **Grid** node.
2. Drop it onto the green **Mesh** input socket of the **Noise Displace** node.
3. Connect the green **Mesh** output socket of the **Noise Displace** node to the **Group Output** node.
4. The nodes are now wired in a Directed Acyclic Graph (DAG) pipeline.

#### Step 4: Attaching the Node Graph to a Scene Object
1. Select your target mesh in the 3D viewport.
2. In the **Properties Inspector** under **Modifier Stack**, click **`+ Geo Nodes`**.
3. The procedural node graph is now actively evaluated as a live modifier in the object's stack, non-destructively displacing geometry into procedural terrain or architectural blocks!

---

## Tutorial 6: Studio Lighting & Bokeh Render (Monte Carlo Path Tracer)

```text
                 [ Sun Light ] (Key Light)
                     \
                      \
      [ Subject ] <────+───── [ Camera with Thin-Lens Bokeh ]
     /         \
    /           \
[ Fill Light ]   [ Spot Light ] (Rim / Hair)
```

### Overview & Learning Goals
- **Category**: Lighting, Camera Optics & Production Rendering.
- **Workflow**: 3-point studio illumination, thin-lens optical depth of field, and offline path tracing.
- **Key Concepts**: PBR direct/indirect lighting, soft area shadows, and PNG image export.
- **Estimated Time**: 6-8 minutes.

### Step-by-Step Walkthrough

#### Step 1: Setting up 3-Point Studio Lights
1. Clear the scene or use your completed Sci-Fi Crate model.
2. Frame your subject in the viewport and press **`F`** to center the camera.
3. Go to **`Add -> Light`**:
   - **Key Light (Sun)**: Add a Directional Sun light from the top-left (`Intensity: 3.0`, warm white color).
   - **Fill Light (Point)**: Add a Point light from the front-right (`Intensity: 15.0`, soft cyan tint) to soften harsh shadow contrasts.
   - **Rim Light (Spot)**: Add a Spot light positioned behind the subject pointing forward (`Intensity: 30.0`, golden yellow) to create a glowing silhouette edge.

#### Step 2: Configuring Thin-Lens Camera Bokeh
1. In the **Scene Outliner**, select the scene **Camera** node.
2. In the **Inspector** under Camera Settings:
   - Set **Aperture Radius**: `0.06m`
   - Set **Focal Distance**: Adjust until your subject is in razor-sharp focus (e.g. `5.0m`).
3. The physical thin-lens optical camera will sample rays across the lens disc, producing genuine circular bokeh in out-of-focus background regions!

#### Step 3: Rendering the Production Image (F12)
1. Press **`F12`** or select **`Render -> Render Image`**.
2. The Production Render Dialog opens:
   - **Resolution**: Choose `1080p Full HD (1920x1080)`.
   - **Samples per Pixel (SPP)**: Set to `128` (or `256` for ultra-clean presentation).
   - **Max Bounces**: `4`.
3. Watch the multi-threaded Rayon path tracer accumulate soft shadows and indirect global illumination across progressive passes.
4. When finished, click **`💾 Save Image (PNG)`** to export your high-resolution render to disk!

---

## Tutorial 7: Watertight Geometric Vase for 3D Printing (CAD & Slicers)

```text
            \         /   <- Flared Rim (Hollowed)
             \_______/
             /       \    <- Convex Belly
            /         \
           +-----------+  <- Stable Base
```

### Overview & Learning Goals
- **Category**: 3D Printing & Functional CAD.
- **Workflow**: Radial profile extrusion, face deletion, non-destructive Solidify wall thickness, and STL export.
- **Key Features**: 2-manifold topological invariants, Solidify modifier, and Binary STL export.
- **Estimated Time**: 8-10 minutes.

### Step-by-Step Walkthrough

#### Step 1: Base Octagon Cylinder
1. Go to **`File -> New Scene`**.
2. Click **`Add Primitive -> Cylinder`**.
3. In Inspector, set **Segments** to `8` (an elegant octagonal profile), **Radius** to `1.2m`, and **Height** to `0.5m`.

#### Step 2: Profile Extrusions
1. Press **`Tab`** to enter **Edit Mode**.
2. Press **`3`** for **Face Level** and select the top cap face.
3. Press **`E`** (Extrude) and pull upward $1.0\text{m}$.
4. Press **`R`** (Scale) and scale outward $1.4\times$ to create the expanding belly of the vase.
5. Press **`E`** again and pull upward $1.2\text{m}$.
6. Press **`R`** and scale inward $0.7\times$ to constrict the neck of the vase.
7. Press **`E`** one final time and scale outward slightly to create the flared top rim.

#### Step 3: Opening the Vessel
1. With only the uppermost top cap face selected, press **`Delete`** (or **`X`**).
2. The top cap face is deleted, creating an open hollow vessel.

#### Step 4: Adding Uniform Wall Thickness (Solidify Modifier)
1. Press **`Tab`** to return to **Object Mode**.
2. In the Inspector under **Modifier Stack**, click **`+ Solidify`**.
3. Set **Thickness** to `0.15m`.
4. The engine automatically extrudes an inner manifold wall shell, turning the single-layer surface into a fully solid, 2-manifold watertight volume.

#### Step 5: Exporting for 3D Slicers (Cura / PrusaSlicer / Bambu Studio)
1. Go to **`File -> Export Active Mesh as STL for 3D Printing (.stl)`**.
2. The model is exported in calibrated binary STL format with guaranteed 2-manifold orientation.
3. Open the `.stl` file in your favorite slicer—no holes, non-manifold edges, or inverted normal errors!

---

## Congratulations!
You now have mastered the essential workflows of Plumb 3D:
- **Box Modeling**: Inset (`I`), Extrude (`E`), and Bevel (`Ctrl+B`).
- **Architectural Modeling**: Primitives, cuts, and battlements.
- **Digital Sculpting**: Grab, Clay, Draw, Inflate, and Smooth brushes.
- **Spline Curves**: Bézier routing, knot extrusion, and 3D pipe sweeps.
- **Procedural Nodes**: Visual DAG generation and modifier binding.
- **Production Rendering**: Photorealistic lighting, optical bokeh, and path tracing (`F12`).
- **3D Printing**: Watertight 2-manifold geometry and STL slicing export.

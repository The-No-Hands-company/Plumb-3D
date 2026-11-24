# Masterclass Tutorial: Modeling & Rendering the Medieval Citadel

This comprehensive tutorial guides you step-by-step through modeling, texturing, lighting, and rendering a complete **Medieval Citadel & Fortress** directly inside **Plumb 3D**.

```text
                  / \          <- Conical Timber Roof & Finial
                 /   \
               |---|---|---|   <- Crenellated Battlements & Parapet Walkway
              ===============  <- Corbelled Machicolation Ring
                 |       |
      ===========| Keep  |=========== <- Rampart Curtain Walls
     / Bastion   | Tower | Gatehouse \
    +------------+-------+------------+ <- Battered Talus Base Plinth
   ===================================== <- Cobblestone Courtyard Ground
```

---

## Prerequisites & Scene Setup

1. Launch **Plumb 3D**.
2. From the **Welcome Hub** (`F2`), select **General 3D** or press **Continue to Viewport**.
3. In the top workspace strip, click **`🔨 Hard-Surface`** or press **`F3`** and type `Hard-Surface`.
4. Delete the default cube: click `DefaultCube` in the Outliner and press **`Delete`** or **`X`**.

---

## Phase 1: Donjon Keep Tower (The Hero Landmark)

### 1.1 The Battered Talus Plinth Base
1. In the top menu, click **`Add Primitive -> Cylinder`**.
2. In the Operator adjustment panel (bottom-left):
   - **Radius**: `3.2 m`
   - **Height**: `2.0 m`
   - **Segments**: `24`
3. In the right **Inspector -> Transform**, set **Position Y**: `1.0 m` (sitting on the ground plane).
4. Double-click the node in the Scene Outliner and rename it: `Keep_Talus_Base`.

### 1.2 Main Tower Shaft & Non-Destructive Modifiers
1. Click **`Add Primitive -> Cylinder`**:
   - **Radius**: `2.6 m`
   - **Height**: `7.5 m`
   - **Segments**: `24`
2. In the Inspector, set **Position Y**: `5.75 m` (aligns on top of the talus base).
3. Rename the node: `Keep_Main_Tower`.
4. **Add Bevel Modifier**:
   - In the right-hand **Inspector**, switch to the **Modifiers** tab (`wrench` icon).
   - Click **`+ Bevel`**.
   - Set **Width**: `0.05 m`, **Segments**: `2`.
   - The tower edges automatically receive smooth, light-catching chamfers without altering base topology.

### 1.3 Machicolation Flared Ring (Corbels)
1. Click **`Add Primitive -> Cylinder`**:
   - **Radius**: `3.2 m`
   - **Height**: `0.8 m`
   - **Segments**: `24`
2. Set **Position Y**: `9.8 m`.
3. Rename node: `Keep_Machicolation_Ring`.

### 1.4 Crenellated Parapet & Battlements Walkway
1. Click **`Add Primitive -> Cylinder`**:
   - **Radius**: `3.25 m`
   - **Height**: `0.9 m`
   - **Segments**: `16` (16 segments allow 8 alternating merlons and 8 crenels/openings).
2. Set **Position Y**: `10.65 m`.
3. Rename node: `Keep_Crenels_Parapet`.
4. **Modeling the Walkway & Merlons**:
   - Press **`Tab`** to enter **Edit Mode**.
   - Press **`3`** for Face select mode.
   - Select the top disk face and press **`I`** (Inset). Drag inward `0.45 m` to define the defensive wall thickness. Confirm with **LMB**.
   - Press **`E`** (Extrude) and move downward `-0.7 m` to carve out the sunken parapet walkway.
   - Select every other perimeter top face (8 faces total), press **`E`**, and extrude upward `0.5 m` to create the protective stone merlons!
   - Press **`Tab`** to return to **Object Mode**.

### 1.5 Conical Timber Roof & Forged Iron Finial
1. Click **`Add Primitive -> Cone`**:
   - **Radius**: `3.4 m`
   - **Height**: `3.5 m`
   - **Segments**: `24`
2. Set **Position Y**: `12.8 m`.
3. Rename node: `Keep_Timber_Roof`.
4. Click **`Add Primitive -> Cylinder`**:
   - **Radius**: `0.04 m`
   - **Height**: `1.2 m`
   - **Segments**: `8`
5. Set **Position Y**: `15.0 m`.
6. Rename node: `Roof_Iron_Finial`.

---

## Phase 2: Fortifications & Architecture

### 2.1 Defensive Curtain Rampart Walls
1. Click **`Add Primitive -> Cube`**.
2. In the Inspector Transform:
   - **Position**: `X: -5.5, Y: 2.5, Z: 0.0`
   - **Scale**: `X: 7.0, Y: 5.0, Z: 1.4`
3. Rename node: `Curtain_Wall_West`.
4. Duplicate for the East wall: Press **`Shift + D`** (or Add Cube):
   - **Position**: `X: 5.5, Y: 2.5, Z: 0.0`
   - **Scale**: `X: 7.0, Y: 5.0, Z: 1.4`
5. Rename node: `Curtain_Wall_East`.

### 2.2 Corner Bastion Flanker Turret
1. Click **`Add Primitive -> Cylinder`**:
   - **Radius**: `1.8 m`
   - **Height**: `6.5 m`
   - **Segments**: `20`
2. Set **Position**: `X: 9.2, Y: 3.25, Z: 0.0`.
3. Rename node: `Bastion_East_Turret`.

### 2.3 Fortified Gatehouse Arch & Portcullis Grille
1. Click **`Add Primitive -> Cube`**:
   - **Position**: `X: 0.0, Y: 2.2, Z: 3.0`
   - **Scale**: `X: 3.6, Y: 4.4, Z: 1.8`
2. Rename node: `Gatehouse_Archway`.
3. Portcullis Grille:
   - Click **`Add Primitive -> Cube`**.
   - **Position**: `X: 0.0, Y: 1.8, Z: 3.8`
   - **Scale**: `X: 1.8, Y: 3.2, Z: 0.15`
   - Rename node: `Portcullis_Iron_Grille`.

### 2.4 Cobblestone Courtyard Ground Plinth
1. Click **`Add Primitive -> Cylinder`**:
   - **Radius**: `22.0 m`
   - **Height**: `0.3 m`
   - **Segments**: `48`
2. Set **Position**: `X: 0.0, Y: -0.15, Z: 0.0`.
3. Rename node: `Citadel_Courtyard_Ground`.

### 2.5 Torch Sconces (Emissive Light Geometry)
1. Click **`Add Primitive -> Cylinder`**:
   - **Radius**: `0.12 m`, **Height**: `0.35 m`, **Segments**: `12`
   - **Position**: `X: -2.1, Y: 2.8, Z: 4.0`
   - Rename node: `Torch_Sconce_Gate_L`.
2. Duplicate or Add Cylinder for the right sconce:
   - **Position**: `X: 2.1, Y: 2.8, Z: 4.0`
   - Rename node: `Torch_Sconce_Gate_R`.

---

## Phase 3: PBR Material Setup

In the Inspector, select each object and navigate to the **Shading / Material** panel:

| Material Name | Assigned Nodes | Base Color (RGB) | Metallic | Roughness | Emission | Procedural Pattern |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **WeatheredGranite** | `Keep_Talus_Base`, `Keep_Main_Tower`, `Keep_Machicolation_Ring`, `Keep_Crenels_Parapet`, `Curtain_Wall_*`, `Bastion_*`, `Gatehouse_Archway` | `0.42, 0.44, 0.48` | `0.15` | `0.82` | `0.0` | `Grid / Micro-Stone` |
| **FortressDarkOak** | `Keep_Timber_Roof` | `0.24, 0.16, 0.10` | `0.05` | `0.68` | `0.0` | `Constant` |
| **ForgedSteel** | `Roof_Iron_Finial`, `Portcullis_Iron_Grille` | `0.18, 0.19, 0.22` | `0.92` | `0.35` | `0.0` | `Constant` |
| **CourtyardCobble** | `Citadel_Courtyard_Ground` | `0.12, 0.13, 0.15` | `0.20` | `0.75` | `0.0` | `Grid` |
| **TorchFlame** | `Torch_Sconce_Gate_L`, `Torch_Sconce_Gate_R` | `1.00, 0.65, 0.18` | `0.00` | `0.00` | `6.0` (High Glow) | `Constant` |

---

## Phase 4: Atmospheric Cinematic 4-Point Lighting Rig

From the top menu, use **`Add Light`** to build the lighting mood:

1. **Directional Moon Key Light**:
   - Menu: **`Add Light -> Directional Light`**
   - Rename: `Moonlight_Key`
   - **Rotation**: `X: -37°, Y: 48°, Z: 0°` (casts long, dramatic shadows across ramparts)
   - **Color**: Pale cold sapphire (`R: 0.55, G: 0.72, B: 0.95`)
   - **Intensity**: `2.8`

2. **Left Torch Point Light**:
   - Menu: **`Add Light -> Point Light`**
   - Rename: `Torch_Light_Left`
   - **Position**: `X: -2.1, Y: 3.2, Z: 4.2`
   - **Color**: Warm fire amber (`R: 1.00, G: 0.65, B: 0.20`)
   - **Intensity**: `24.0`, **Radius**: `22.0 m`

3. **Right Torch Point Light**:
   - Rename: `Torch_Light_Right`
   - **Position**: `X: 2.1, Y: 3.2, Z: 4.2`
   - **Color**: Warm fire amber (`R: 1.00, G: 0.65, B: 0.20`)
   - **Intensity**: `24.0`, **Radius**: `22.0 m`

4. **Parapet Walkway Accent Torch**:
   - Rename: `Parapet_Torch_Accent`
   - **Position**: `X: 0.0, Y: 11.2, Z: 2.2`
   - **Color**: Golden orange (`R: 1.00, G: 0.72, B: 0.25`)
   - **Intensity**: `18.0`, **Radius**: `20.0 m`

5. **Sky Ambient Fill Light**:
   - Rename: `Sky_Ambient_Fill`
   - **Position**: `X: 0.0, Y: 16.0, Z: 0.0`
   - **Color**: Deep twilight blue (`R: 0.30, G: 0.42, B: 0.65`)
   - **Intensity**: `10.0`, **Radius**: `60.0 m`

---

## Phase 5: Camera Framing & Composition

1. Orbit the camera to a low-angle three-quarters heroic view:
   - **Target**: `(0.0, 5.0, 0.0)`
   - **Distance**: `22.0 m`
   - **Pitch**: `0.22 rad` (~12° upward angle)
   - **Yaw**: `0.65 rad` (~37° three-quarter view)
2. In the top ribbon, verify **Perspective Projection** is enabled.

---

## Phase 6: Production Render (`F12`) & LookDev Settings

Press **`F12`** or click **`Render -> Production Render`** to open the render suite:

### 1. Ray Tracing Quality
- **Resolution Preset**: `1080p (1920x1080)` or `1440p (2560x1440)`.
- **Target Samples**: `128` (or `256` for ultra-clean shadow penumbras).
- **Max Ray Depth**: `6` bounces.
- **Physical Camera Aperture**: `0.012` (subtle optical depth-of-field bokeh).

### 2. Built-in Post-Production Compositor Settings
Expand the **✨ Post-Production & LookDev** panel:
- **Enable Post-Processing**: `[X] Checked`
- **Bloom / Emissive Glow**: `[X] Checked`
  - **Bloom Intensity**: `0.42` (creates a warm halo around the torches without washing out the stone)
  - **Bloom Threshold**: `1.05` (isolates the emissive torch flames)
- **Vignette Intensity**: `0.32` (darkens the frame corners to guide viewer focus to the gate and tower)
- **Exposure (EV)**: `1.15`
- **Contrast**: `1.12` (S-curve punch for deep castle shadows)
- **Saturation**: `1.18` (enriches the torch amber and moonlight sapphire)
- **ACES Filmic Tone Mapping**: `[X] Checked` (preserves highlight details on illuminated stone edges)

### 3. Rendering & Exporting
1. Click **`Start Render (Progressive)`**.
2. Watch the multi-threaded passes converge.
3. When finished, enter your export path (e.g. `citadel_render.png`) and click **`Save Image as PNG`**.

---

## Phase 7: Professional DCC Finishing Tips

To elevate your render from raw 3D to cinematic concept art:
1. **Atmospheric Depth**: Add a subtle mist or smoke layer in front of the lower walls to separate foreground battlements from background towers.
2. **Contact Shadows**: Ensure the talus plinth intersects the courtyard ground plinth cleanly so that the moon directional light casts strong ambient occlusion contact lines.
3. **Bevel Highlights**: Keep the Bevel modifier active on the main tower; the subtle specular bevel gleam separates the cylindrical form from dark background skies.

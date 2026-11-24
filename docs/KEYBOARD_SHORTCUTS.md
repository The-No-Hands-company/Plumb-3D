# Keyboard Shortcuts & Viewport Gestures

This document provides a comprehensive index of all default keyboard shortcuts, mouse gestures, and interactive modals in the DCC Suite.

---

## 1. Viewport Camera Navigation

| Shortcut | Action | Description |
| :--- | :--- | :--- |
| `Alt + Left Mouse` | **Orbit** | Tumble / orbit the camera around the orbit center. |
| `Middle Mouse (MMB)` | **Orbit** | Alternative one-button camera orbit. |
| `Alt + Middle Mouse` | **Pan** | Pan camera laterally in the screen view plane. |
| `Shift + Middle Mouse` | **Pan** | Standard Blender-style camera panning. |
| `Mouse Scroll Wheel` | **Zoom** | Smoothly zoom camera closer to or further from target. |
| `F` | **Frame Selected** | Focus camera view directly onto selected object or sub-elements. |
| `Numpad Decimal (.)` | **Frame Selected** | Industry standard numpad focus shortcut. |
| `Numpad 1` | **Front View** | Snap camera to front orthographic view ($+Y$ looking forward). |
| `Numpad 3` | **Right View** | Snap camera to right orthographic view ($+X$). |
| `Numpad 7` | **Top View** | Snap camera to top orthographic view ($+Z$). |
| `Numpad 5` | **Toggle Ortho** | Switch between Perspective and Orthographic projection. |
| `Numpad 0` | **Active Camera** | Switch viewport to view through the active scene camera node. |
| `6-Axis Navigation Gizmo` | **Snap & Orbit** | Click red/green/blue bubbles in top-right HUD to snap to views; drag sphere to orbit. |

---

## 2. Selection & Modes

| Shortcut | Action | Description |
| :--- | :--- | :--- |
| `Left Mouse Click` | **Select** | Pick object (in Object Mode) or sub-element (in Edit Mode). |
| `Shift + Left Mouse` | **Multi-Select** | Add or remove items from active selection set. |
| `B` | **Box Select** | Marquee drag selection box to select multiple elements. |
| `A` | **Select All** | Select all visible objects or elements in active mesh. |
| `Alt + A` | **Deselect All** | Clear all active selections. |
| `L` | **Select Linked** | Select all manifold connected geometry sharing vertices. |
| `Alt + L` | **Loop Select** | Select continuous edge loops or face loops. |
| `Ctrl + I` | **Invert Selection** | Invert the currently selected elements. |
| `Tab` | **Toggle Mode** | Switch between Object Mode and Edit Mode. |
| `1` | **Vertex Level** | Set mesh selection level to Vertices (Edit Mode). |
| `2` | **Edge Level** | Set mesh selection level to Edges (Edit Mode). |
| `3` | **Face Level** | Set mesh selection level to Faces (Edit Mode). |

---

## 3. Transformation & Gizmos

| Shortcut | Action | Description |
| :--- | :--- | :--- |
| `W` | **Translate Gizmo** | Activate 3D Translation Gizmo (Move mode). |
| `E` | **Rotate Gizmo** | Activate 3D Rotation Gizmo (Rotate mode). |
| `R` | **Scale Gizmo** | Activate 3D Scaling Gizmo (Scale mode). |
| `,` (Comma) | **Cycle Orientation** | Toggle between Global, Local, Normal, and View coordinates. |
| `Shift + Tab` | **Toggle Snapping** | Enable/disable grid and vertex snapping during drag. |
| `O` | **Proportional Editing** | Toggle soft-selection falloff for nearby vertices. |

---

## 4. Interactive Modeling Modals (Edit Mode)

| Shortcut | Operator | Description & Modal Controls |
| :--- | :--- | :--- |
| `E` | **Extrude Region** | Extrudes selected faces along normal. Drag mouse to set distance. Press `Enter`/`LMB` to confirm; `Esc`/`RMB` to cancel. |
| `Ctrl + B` | **Bevel** | Bevels selected edges/faces. Drag mouse for width. **Scroll Wheel** adds/removes segments. `LMB` confirms; `Esc` cancels. |
| `Shift + Ctrl + B` | **Bevel Vertex** | Bevels selected vertices into polygonal facets. |
| `I` | **Inset Faces** | Insets faces inward. Drag mouse for inset depth. Press `I` to toggle between Region Inset and Individual Inset. |
| `Ctrl + R` | **Loop Cut** | Hover over quad to preview cut line. Click `LMB`, then drag mouse to slide cut along edges. Click `LMB` to finalize. |
| `K` | **Knife Tool** | Interactive bisect tool. Hover over geometry to snap crosshair. Click `LMB` to place cut points. Press `Enter`/`Space` to commit. |
| `Shift + E` | **Edge Crease** | Adjust semi-sharp crease weight (0.0 to 1.0) for Catmull-Clark subdivision. Drag mouse to adjust. |
| `Shift + V` | **Vertex Crease** | Adjust vertex corner sharpness for Catmull-Clark subdivision. |
| `J` | **Connect Vertices** | Split a face by connecting two non-adjacent vertices with a new edge. |
| `V` | **Rip Vertices** | Detach vertices and duplicate topology along cursor direction. |
| `Alt + M` | **Merge Vertices** | Merge selected vertices at center. |
| `Delete` / `X` | **Delete** | Delete selected vertices, edges, or faces with topological healing. |

---

## 5. Viewport Overlays & Display

| Shortcut | Action | Description |
| :--- | :--- | :--- |
| `Z` | **Shading Dropdown** | Switch between Studio MatCap, Flat, and PBR Shading. |
| `M` | **Measure Tool** | Interactive 3D calipers to measure Euclidean distance between points. |
| `Alt + Z` | **X-Ray / Wireframe** | Toggle wireframe overlay on shaded surfaces. |

---

## 6. General & Interface Windows

| Shortcut | Action | Description |
| :--- | :--- | :--- |
| `Right Click (RMB)`| **Context Menu** | Context-sensitive floating menu adapting to active mode and selection. |
| `F1` | **User Manual** | Open the interactive in-software documentation & cheat sheet. |
| `F12` | **Render Image** | Launch the multi-threaded Monte Carlo offline path tracer. |
| `Ctrl + Z` | **Undo** | Undo the previous modeling, modifier, or scene operation. |
| `Ctrl + Y` / `Ctrl + Shift + Z` | **Redo** | Redo the previously undone operation. |
| `Ctrl + S` | **Save Scene** | Save current scene graph to project format. |

---

## 7. Animation & Scene DAG Hierarchy

| Shortcut | Action | Description |
| :--- | :--- | :--- |
| `I` (Object/Pose Mode) | **Insert Keyframe** | Insert PRS (Position, Rotation, Scale) keyframe on all selected objects at current frame. |
| `Space` | **Play / Pause** | Toggle animation timeline playback (60+ FPS loop with dynamic physics). |
| `Arrow Left / Right` | **Step Frame** | Scrub timeline back or forward by 1 frame. |
| `Ctrl + G` | **Group Objects** | Create an empty group at selection centroid and parent all selected objects. |
| `P` | **Set Parent** | Parent selected child object to secondary selected parent target. |
| `Shift + P` | **Unparent** | Clear parent and restore object to root level. |
| `Ctrl + J` | **Join Meshes** | Combine secondary selected mesh into the active mesh. |
| `RMB -> Live Boolean Carve` | **CAD Carve** | Carve secondary selected cutter object out of active object in real time. |

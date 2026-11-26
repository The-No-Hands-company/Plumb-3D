# Plumb 3D Keyboard Shortcuts & Viewport Gestures

This document provides a comprehensive index of all default keyboard shortcuts, mouse gestures, and interactive modals in Plumb 3D.

---

## 1. Viewport Camera Navigation

| Shortcut | Action | Description |
| :--- | :--- | :--- |
| `Alt + Left Mouse` | **Orbit** | Tumble / orbit the camera around the active orbit center. |
| `Middle Mouse (MMB)` | **Orbit** | One-button camera orbit around selection or origin. |
| `Alt + Middle Mouse` | **Pan** | Pan camera laterally in the screen view plane. |
| `Shift + Middle Mouse` | **Pan** | Alternative camera panning gesture. |
| `Mouse Scroll Wheel` | **Zoom** | Smoothly zoom camera closer to or further from target. |
| `F` | **Frame Selected** | Focus camera view directly onto selected object or sub-elements. |
| `Numpad Decimal (.)` | **Frame Selected** | Focus camera onto active selection. |
| `Numpad 1` | **Front View** | Snap camera to front orthographic view ($+Y$ looking forward). |
| `Numpad 3` | **Right View** | Snap camera to right orthographic view ($+X$). |
| `Numpad 7` | **Top View** | Snap camera to top orthographic view ($+Z$). |
| `Numpad 5` | **Toggle Ortho** | Switch between Perspective and Orthographic projection. |
| `Numpad 0` | **Active Camera** | Switch viewport between active scene camera and free orbit. |
| `6-Axis Navigation Gizmo` | **Snap & Orbit** | Click axis bubbles in top-right HUD to snap views; drag sphere to orbit. |

---

## 2. Selection & Interaction Modes

| Shortcut | Action | Description |
| :--- | :--- | :--- |
| `Left Mouse Click` | **Select** | Pick object (in Object Mode) or sub-element (in Edit Mode). |
| `Shift + Left Mouse` | **Multi-Select** | Add or remove items from active selection set. |
| `B` | **Box Select** | Marquee drag selection box to select multiple elements. |
| `A` | **Select All** | Select all visible objects or elements in active mesh. |
| `Alt + A` | **Deselect All** | Clear all active selections. |
| `L` | **Select Linked** | Select all contiguous manifold geometry sharing vertices. |
| `Alt + L` | **Loop Select** | Select continuous topological quad edge loops. |
| `Ctrl + I` | **Invert Selection** | Invert the currently selected elements. |
| `Tab` | **Toggle Mode** | Switch between Object Mode and Edit Mode. |
| `1` | **Vertex Level** | Set mesh selection level to Vertices (Edit Mode). |
| `2` | **Edge Level** | Set mesh selection level to Edges (Edit Mode). |
| `3` | **Face Level** | Set mesh selection level to Faces (Edit Mode). |

---

## 3. Transformation & Snapping Gizmos

| Shortcut | Action | Description |
| :--- | :--- | :--- |
| `W` | **Translate Gizmo** | Activate 3D Translation Gizmo (Move mode). |
| `E` | **Rotate Gizmo** | Activate 3D Rotation Gizmo (Rotate mode). |
| `R` | **Scale Gizmo** | Activate 3D Scaling Gizmo (Scale mode). |
| `,` (Comma) | **Cycle Orientation** | Toggle between Global, Local, Normal, and View coordinates. |
| `V` | **Toggle Snapping** | Toggle precision vertex and grid magnetic snapping. |
| `Alt + W` | **Align Workplane** | Align CAD construction workplane directly to active face. |
| `Alt + X` | **Symmetrize Workplane**| Symmetrize active geometry across the CAD workplane. |
| `Alt + A` | **Align Dialog** | Open the 3ds Max / Maya style Object Alignment dialog. |

---

## 4. Interactive Modeling Modals (Edit Mode)

| Shortcut | Operator | Description & Modal Controls |
| :--- | :--- | :--- |
| `E` | **Extrude Region** | Extrudes selected faces along normal. Drag mouse for distance. `LMB`/`Enter` confirms; `Esc` cancels. |
| `Shift + E` | **Direct Push-Pull** | Face push-pull extrusion with automatic boolean union/cavity subtraction. |
| `Ctrl + B` | **Bevel** | Bevels selected edges/faces. Drag mouse for width. **Scroll Wheel** adds/removes segments. |
| `Shift + Ctrl + B` | **Bevel Vertex** | Bevels selected vertices into polygonal facets. |
| `I` | **Inset Faces** | Insets faces inward. Drag for inset depth. Press `I` to toggle Region/Individual. |
| `Ctrl + R` | **Loop Cut** | Hover over quad to preview cut line. Click `LMB`, drag to slide, click `LMB` to finalize. |
| `K` | **Knife Tool** | Interactive bisect tool. Snaps to edges/vertices. Click `LMB` to place cut points; `Enter` commits. |
| `C` | **Direct Cutter** | Interactive Box, Circle, Ngon, and Radial Compound cutting tools with live ghost mesh preview. |
| `K` / `Shift + K` | **Kitbash Hardware** | Drop industrial bolts, screws, vents, and latches with surface normal alignment. |
| `P` | **Bézier Curve Pen** | Click to place 3D anchor points; drag for tangent handles. `C` closes curve; `Enter` commits. |
| `Shift + E` | **Edge Crease** | Adjust semi-sharp crease weight (0.0 to 1.0) for Catmull-Clark subdivision. |
| `Shift + V` | **Vertex Crease** | Adjust vertex corner sharpness for Catmull-Clark subdivision. |
| `J` | **Connect Vertices** | Split a face by connecting two non-adjacent vertices with a new edge. |
| `V` | **Rip Vertices** | Detach vertices and duplicate topology along cursor direction. |
| `Alt + M` | **Merge Vertices** | Merge selected vertices at center. |
| `Delete` / `X` | **Delete** | Delete selected vertices, edges, or faces with topological healing. |

---

## 5. General, Studio Windows & Dialogs

| Shortcut | Action | Description |
| :--- | :--- | :--- |
| `F1` | **User Manual** | Open the in-software interactive documentation, tutorials & shortcuts reference. |
| `F2` | **Welcome Hub** | Open Welcome & Quick-Start Hub with project templates and tutorials. |
| `F3` / `Ctrl + K` | **Command Palette** | Omnipresent fuzzy search across all modeling tools, modifiers, and actions. |
| `F12` | **Render Image** | Launch the production Monte Carlo path tracer window. |
| `Right Click (RMB)`| **Context Menu** | Context-sensitive floating menu adapting to active mode and selection. |
| `Ctrl + Z` | **Undo** | Undo the previous modeling, modifier, or scene operation. |
| `Ctrl + Y` | **Redo** | Redo the previously undone operation. |
| `Ctrl + S` | **Save Project** | Save scene to native `.plmb` binary container format. |
| `Ctrl + O` | **Open Project** | Open an existing `.plmb` project archive. |

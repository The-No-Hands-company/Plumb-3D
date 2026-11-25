# Changelog

All notable changes to the Plumb 3D DCC Suite are documented in this file.
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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

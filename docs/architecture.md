# Plumb 3D — System Architecture & Topology

## 1. System Overview

Plumb 3D is engineered as a decoupled, multi-crate workspace in 100% Safe Pure Rust. The system strictly isolates domain algorithms from UI presentation and graphics rendering.

```
                    ┌───────────────────────────┐
                    │       crates/app          │  Application shell & UI
                    └──────┬─────────────┬──────┘
                           │             │
              ┌────────────▼──┐       ┌──▼────────────────┐
              │ graph-engine  │       │  render-viewport  │  Viewport & WGPU
              └──────┬────────┘       └──┬────────────────┘
                     │                   │
         ┌───────────▼─────────┐         │
         │     geo-kernel      │◄────────┘  2-Manifold BMesh & Math
         └───────────┬─────────┘
                     │
         ┌───────────▼─────────┐
         │     core-math       │  Vector math, Rays, Transforms
         └─────────────────────┘
```

## 2. Workspace Crates & Responsibilities

| Crate | Responsibility | Dependencies |
| :--- | :--- | :--- |
| `crates/core-math` | Primitive types (`Vec2`, `Vec3`, `Quat`, `Mat4`, `Ray`, `Aabb`). | `glam` |
| `crates/geo-kernel` | 2-Manifold Half-Edge BMesh data structure, Euler operators, BSP CSG booleans, subdivision, bevel, and file I/O (STL, OBJ, PLY, glTF). | `core-math` |
| `crates/graph-engine` | Maya-style DAG scene hierarchy, transforms, modifier evaluation stack, animation tracks, and rigid body dynamics. | `core-math`, `geo-kernel` |
| `crates/geo-nodes` | Procedural node graph with type-checked sockets (Geometry, Float, Vector, Matrix). | `core-math`, `geo-kernel` |
| `crates/render-viewport` | Hardware-accelerated viewport rendering with WGPU (forward pass, depth-stencil, grid, gizmos, wireframe). | `core-math`, `wgpu` |
| `crates/render-pathtracer` | Multi-threaded CPU Monte Carlo path tracer with SAH BVH trees, thin-lens optics, and Principled BSDF. | `core-math`, `geo-kernel`, `rayon` |
| `crates/app` | Desktop entry point, egui user interface, interaction modals, autosave, and native dialogs. | All workspace crates |

## 3. Core Architectural Guarantees

1. **Topological Invariant ($V - E + F = 2$)**: All half-edges maintain reciprocal twins. Non-manifold configurations are rejected at the kernel boundary.
2. **Zero Ambient Files**: All build, test, and package workflows are orchestrated via `scripts/`.
3. **Deterministic Memory Safety**: Zero `unsafe` blocks. Memory allocations use RAII and pre-allocated buffers.
4. **Mechanical Guardrails**: Workspace builds enforce zero warnings under `cargo clippy -- -D warnings`.

## 4. Geometric Kernel Subsystem Architecture (`crates/geo-kernel`)

The geometric kernel is organized into modular mathematical engines:

| Module | Responsibility & Core Algorithms |
| :--- | :--- |
| `mesh/` | 2-Manifold BMesh Half-Edge core (`Vertex`, `Edge`, `HalfEdge`, `Face`), Euler operators, edge flips, and quad/ngon tessellation. |
| `sketch/` | 2D parametric constraint sketcher on 3D workplanes (Points, Lines, Circles, Arcs) with geometric distance and angular constraint solver. |
| `trimmed_nurbs/` | Analytic Rational B-Spline surfaces, 2D UV trimming curve loops, and marching Surface-Surface Intersection (SSI) engine. |
| `diagnostics/` | Class-A surface inspection: screen-space Gaussian & Mean curvature heatmaps, dynamic Zebra reflection striping, $G^0/G^1/G^2$ seam analysis, and porcupine curvature combs. |
| `operators/` | Advanced B-Rep operations: variable-radius rolling-ball & chordal fillets, Coons/Gordon $N$-sided boundary patch filler, solid shell hollowing & thickening, push-pull carving, and coplanar polygon inscriber. |
| `assembly/` | Multi-body CAD kinematics: `MateConnector` coordinate frames, 7 kinematic joint types (Revolute, Prismatic, Concentric, Cylindrical, Gear, Fixed), Separating Axis Theorem (SAT) 3D interference clash detection, and exploded view sequencer. |
| `drafting/` | Technical 2D drafting: First & Third angle orthographic camera projections, analytical vector Hidden Line Removal (HLR), ASME Y14.5 GD&T feature control frames, and DXF / SVG sheet exporters. |
| `sheet_metal/` | DIN 6935 K-factor bend deduction engine, parametric base & edge flanges, 1-click flat pattern unfolding, and multi-layer CNC DXF (`OUTER_CUT`, `BEND_UP`, `BEND_DOWN`) + SVG export. |
| `mechanical/` | Parametric ANSI/ISO Fasteners (M2-M30 bolts, SHCS, nuts, washers), Involute Gears (Spur, Helical with twist, Bevel with taper), and ANSI roller chain sprockets (#25-#60). |
| `fea/` | Finite element linear elasticity solver, 4-node tetrahedral (Tet4) formulation, material models, Cauchy stress recovery, Von Mises yield criteria, and Factor of Safety (FoS). |
| `io/` | Universal CAD and DCC interop: ISO 10303-21 STEP, ANSI/USAF IGES 5.3, native glTF 2.0 / GLB, USDA, native binary `.plmb` container, OBJ, STL, and PLY. |
| `modifiers/` | Non-destructive 16-modifier stack: Bevel, Solidify, Mirror, Array, Boolean (BSP), Subdivide, Displace, Shrinkwrap, Curve, Lattice, Remesh, and Weighted Normals. |

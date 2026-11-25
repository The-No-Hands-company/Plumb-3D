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

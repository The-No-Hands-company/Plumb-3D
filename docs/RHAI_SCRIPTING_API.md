# Rhai Embedded Scripting API Reference

The DCC suite embeds **Rhai**, a 100% safe, fast, and sandboxed scripting language natively integrated with the pure-Rust geometry kernel and scene graph.

---

## 1. Overview & Safety Guarantees

Unlike embedded C/C++ runtimes (such as Lua or Python) which introduce foreign FFI risks, unstable ABI layers, and potential segmentation faults:
- **Zero Unsafe C FFI**: The entire scripting engine runs in safe Rust.
- **Infinite Loop Protection**: Operations execute with configurable maximum instruction step limits, preventing freezes.
- **Memory Caps**: String allocations and data buffers are constrained within safe thresholds.
- **First-Class Rust Integration**: Direct mapping to `core_math::Vec3`, `geo_kernel::Mesh`, and `graph_engine::SceneGraph`.

To open the interactive Script Console, click **`📜 Rhai`** in the top bar or toolbar.

---

## 2. Core Math Types & Functions

### `Vec3` (3D Vector)
Vectors represent positions, directions, colors, and offsets:

```rust
// Constructors
let v = vec3(1.0, 2.0, 3.0);
let zero = vec3(0.0, 0.0, 0.0);

// Basic Arithmetic
let sum = v + vec3(0.0, 1.0, 0.0);
let scaled = v * 2.5;

// Vector Operations
let len = length(v);
let dir = normalize(v);
let d = dot(v, vec3(0.0, 1.0, 0.0));
let n = cross(vec3(1.0, 0.0, 0.0), vec3(0.0, 1.0, 0.0));
```

---

## 3. Mesh Primitives & Creation

Generate calibrated half-edge BMesh models programmatically using the `Primitives` factory:

| Function | Parameters | Description |
| :--- | :--- | :--- |
| `Primitives::create_cube(size)` | `f32` | Creates a cube centered at origin with edge length `size`. |
| `Primitives::create_cylinder(radius, height, segments)` | `f32, f32, int` | Creates a closed cylinder with `segments` circumferential divisions. |
| `Primitives::create_uv_sphere(radius, segments, rings)` | `f32, int, int` | Creates a UV sphere with latitude rings and longitude segments. |
| `Primitives::create_plane(width, depth, seg_x, seg_z)` | `f32, f32, int, int` | Creates a planar grid subdivided into `seg_x * seg_z` quads. |
| `Primitives::create_cone(radius, height, segments)` | `f32, f32, int` | Creates a closed cone with base radius and height. |
| `Primitives::create_torus(major_r, minor_r, seg_u, seg_v)` | `f32, f32, int, int` | Creates a doughnut torus mesh with major and minor radii. |

### Example:
```rust
let my_cylinder = Primitives::create_cylinder(1.5, 4.0, 24);
let my_sphere = Primitives::create_uv_sphere(2.0, 32, 16);
```

---

## 4. Mesh Operators & Queries

Mesh objects provide procedural methods for geometric and topological transformation:

```rust
let mesh = Primitives::create_cube(2.0);

// Topological Subdivision (Midpoint Split)
mesh.subdivide(1); // Subdivides each quad into 4 quads

// Procedural Face Extrusion
mesh.extrude_all_faces(0.5); // Extrudes all faces along their surface normal

// Affine Transformations
// transform(tx, ty, tz, sx, sy, sz)
mesh.transform(0.0, 2.0, 0.0, 1.0, 1.5, 1.0);

// Geometric Queries
let v_count = mesh.vertex_count();
let f_count = mesh.face_count();
print("Mesh has " + v_count + " vertices and " + f_count + " faces.");
```

---

## 5. Scene Graph Integration

The global `scene` object allows scripts to instantiate objects, lights, and cameras into the active 3D world:

### Adding Mesh Nodes
```rust
// scene.add_mesh(name_string, mesh_object)
let cube = Primitives::create_cube(1.0);
scene.add_mesh("Building_Block", cube);
```

### Adding Light Sources
```rust
// scene.add_point_light(name, x, y, z, r, g, b, intensity)
scene.add_point_light("KeyLight", 5.0, 8.0, 5.0, 1.0, 0.95, 0.8, 25.0);

// scene.add_sun_light(name, dir_x, dir_y, dir_z, r, g, b, intensity)
scene.add_sun_light("SunLight", -0.5, -1.0, -0.3, 1.0, 1.0, 1.0, 5.0);
```

---

## 6. Practical Script Examples

### Example 1: Procedural Sci-Fi Tower Generator
```rust
// Generate stacked procedural building with extrusions
let tower = Primitives::create_cube(3.0);
tower.subdivide(1);
tower.extrude_all_faces(0.6);

scene.add_mesh("SciFiTower", tower);

// Add dynamic accent lighting
scene.add_point_light("RoofLight", 0.0, 4.5, 0.0, 0.2, 0.8, 1.0, 20.0);
```

### Example 2: Circular Pillar Colonnade (Procedural Scattering)
```rust
let num_pillars = 8;
let radius = 6.0;
let tau = 6.2831853;

for i in 0..num_pillars {
    let angle = (i.to_float() / num_pillars.to_float()) * tau;
    let px = cos(angle) * radius;
    let pz = sin(angle) * radius;

    let pillar = Primitives::create_cylinder(0.4, 4.0, 16);
    pillar.transform(px, 2.0, pz, 1.0, 1.0, 1.0);
    scene.add_mesh("Pillar_" + i, pillar);
}
```

### Example 3: Procedural Stepped Pyramid
```rust
let levels = 5;
for lvl in 0..levels {
    let size = (levels - lvl).to_float() * 1.5;
    let y_pos = lvl.to_float() * 0.5 + 0.25;

    let tier = Primitives::create_cube(size);
    tier.transform(0.0, y_pos, 0.0, 1.0, 0.25, 1.0);
    scene.add_mesh("Pyramid_Lvl_" + lvl, tier);
}
```

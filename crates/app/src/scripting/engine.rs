use core_math::Vec3;
use geo_kernel::{
    decimate_mesh, modifiers::Modifier, BooleanOp, CatmullClarkModifier, Mesh, Primitives,
    SolidifyModifier,
};
use rhai::{Engine, Scope};
use std::sync::{Arc, Mutex};

/// Output channel from a script execution back to the application.
#[derive(Debug, Default, Clone)]
pub struct ScriptExecutionOutput {
    pub meshes_to_add: Vec<(String, Mesh)>,
    pub logs: Vec<String>,
}

/// Initializes a sandboxed, pure-Rust Rhai engine with complete 3D modeling bindings.
pub fn create_scripting_engine(output: Arc<Mutex<ScriptExecutionOutput>>) -> Engine {
    let mut engine = Engine::new();

    // Sandbox safeguards: limit operations and call stack depth to prevent infinite loops/hangs
    engine.set_max_operations(500_000);
    engine.set_max_call_levels(64);

    // 1. Vector3 Type Binding
    engine.register_type_with_name::<Vec3>("Vec3");
    engine.register_fn("vec3", |x: f32, y: f32, z: f32| Vec3::new(x, y, z));
    engine.register_fn("vec3", |x: f64, y: f64, z: f64| {
        Vec3::new(x as f32, y as f32, z as f32)
    });
    engine.register_fn("vec3", |x: i64, y: i64, z: i64| {
        Vec3::new(x as f32, y as f32, z as f32)
    });
    engine.register_get_set(
        "x",
        |v: &mut Vec3| v.x as f64,
        |v: &mut Vec3, val: f64| v.x = val as f32,
    );
    engine.register_get_set(
        "y",
        |v: &mut Vec3| v.y as f64,
        |v: &mut Vec3, val: f64| v.y = val as f32,
    );
    engine.register_get_set(
        "z",
        |v: &mut Vec3| v.z as f64,
        |v: &mut Vec3, val: f64| v.z = val as f32,
    );
    engine.register_fn("+", |a: Vec3, b: Vec3| a + b);
    engine.register_fn("-", |a: Vec3, b: Vec3| a - b);
    engine.register_fn("*", |a: Vec3, scalar: f64| a * (scalar as f32));
    engine.register_fn("to_string", |v: &mut Vec3| {
        format!("Vec3({:.2}, {:.2}, {:.2})", v.x, v.y, v.z)
    });

    // 2. Mesh Type Binding
    engine.register_type_with_name::<Mesh>("Mesh");
    engine.register_fn("vertex_count", |m: &mut Mesh| m.vertex_count() as i64);
    engine.register_fn("face_count", |m: &mut Mesh| m.face_count() as i64);
    engine.register_fn("edge_count", |m: &mut Mesh| m.edge_count() as i64);
    engine.register_fn("to_string", |m: &mut Mesh| {
        format!("Mesh(V: {}, F: {})", m.vertex_count(), m.face_count())
    });

    // 3. Primitive Mesh Generators
    engine.register_fn("create_cube", |size: f64| {
        Primitives::create_cube(size as f32)
    });
    engine.register_fn("create_cube", |size: i64| {
        Primitives::create_cube(size as f32)
    });
    engine.register_fn("create_uv_sphere", |radius: f64, segs: i64, rings: i64| {
        Primitives::create_uv_sphere(radius as f32, segs.max(3) as u32, rings.max(3) as u32)
    });
    engine.register_fn("create_cylinder", |radius: f64, height: f64, segs: i64| {
        Primitives::create_cylinder(radius as f32, height as f32, segs.max(3) as u32)
    });
    engine.register_fn("create_plane", |w: f64, l: f64, sub_x: i64, sub_z: i64| {
        Primitives::create_plane(w as f32, l as f32, sub_x.max(1) as u32, sub_z.max(1) as u32)
    });
    engine.register_fn(
        "create_torus",
        |maj: f64, min: f64, maj_s: i64, min_s: i64| {
            Primitives::create_torus(
                maj as f32,
                min as f32,
                maj_s.max(3) as u32,
                min_s.max(3) as u32,
            )
        },
    );
    engine.register_fn("create_plumb_bob", Primitives::create_plumb_bob);

    // 4. Mesh Geometry Operators
    engine.register_fn("subdivide", |mesh: &mut Mesh, levels: i64| -> Mesh {
        let l = levels.clamp(1, 3) as u32;
        CatmullClarkModifier::new(l)
            .apply(mesh)
            .unwrap_or_else(|_| mesh.clone())
    });

    engine.register_fn("extrude_all_faces", |mesh: &mut Mesh, dist: f64| -> Mesh {
        let mut m = mesh.clone();
        let face_keys: Vec<_> = m.faces.keys().collect();
        for fk in face_keys {
            let _ = m.extrude_face(fk, dist as f32);
        }
        m.update_normals();
        m
    });

    engine.register_fn("solidify", |mesh: &mut Mesh, thickness: f64| -> Mesh {
        let modifier = SolidifyModifier::new(thickness as f32);
        modifier.apply(mesh).unwrap_or_else(|_| mesh.clone())
    });

    engine.register_fn("decimate", |mesh: &mut Mesh, ratio: f64| -> Mesh {
        decimate_mesh(mesh, ratio.clamp(0.05, 1.0) as f32).unwrap_or_else(|_| mesh.clone())
    });

    engine.register_fn("boolean_union", |a: &mut Mesh, b: Mesh| -> Mesh {
        a.boolean(&b, BooleanOp::Union)
            .unwrap_or_else(|_| a.clone())
    });

    engine.register_fn("boolean_difference", |a: &mut Mesh, b: Mesh| -> Mesh {
        a.boolean(&b, BooleanOp::Difference)
            .unwrap_or_else(|_| a.clone())
    });

    engine.register_fn("boolean_intersection", |a: &mut Mesh, b: Mesh| -> Mesh {
        a.boolean(&b, BooleanOp::Intersection)
            .unwrap_or_else(|_| a.clone())
    });

    // 5. Procedural Points & Scattering
    #[derive(Debug, Clone)]
    struct ScriptPoints(Vec<geo_nodes::PointInstance>);

    engine.register_type_with_name::<ScriptPoints>("Points");
    engine.register_fn("count", |pts: &mut ScriptPoints| pts.0.len() as i64);

    engine.register_fn(
        "distribute_points",
        |mesh: &mut Mesh, density: f64, seed: i64| -> ScriptPoints {
            use geo_nodes::GeoNodeOp;
            let op = geo_nodes::DistributePointsOnFacesNode;
            let inputs = vec![
                geo_nodes::SocketValue::Geometry(Some(mesh.clone())),
                geo_nodes::SocketValue::Float(density as f32),
                geo_nodes::SocketValue::Int(seed),
            ];
            if let Ok(outputs) = op.execute(&inputs) {
                if let Some(pts) = outputs.first().and_then(|v| v.as_points()) {
                    return ScriptPoints(pts.to_vec());
                }
            }
            ScriptPoints(Vec::new())
        },
    );

    engine.register_fn(
        "instance_on_points",
        |pts: &mut ScriptPoints, instance: Mesh| -> Mesh {
            use geo_nodes::GeoNodeOp;
            let op = geo_nodes::InstanceOnPointsNode;
            let inputs = vec![
                geo_nodes::SocketValue::Points(pts.0.clone()),
                geo_nodes::SocketValue::Geometry(Some(instance)),
                geo_nodes::SocketValue::Vector3(Vec3::ONE),
            ];
            if let Ok(outputs) = op.execute(&inputs) {
                if let Some(mesh) = outputs.first().and_then(|v| v.as_geometry()) {
                    return mesh.clone();
                }
            }
            Mesh::new()
        },
    );

    // 6. Scene Output Function: scene.add_mesh("Name", mesh)
    let out_clone = output.clone();
    engine.register_fn("add_mesh", move |name: &str, mesh: Mesh| {
        if let Ok(mut out) = out_clone.lock() {
            out.logs.push(format!(
                "Added mesh '{}' ({} verts, {} faces)",
                name,
                mesh.vertex_count(),
                mesh.face_count()
            ));
            out.meshes_to_add.push((name.to_string(), mesh));
        }
    });

    let out_clone2 = output;
    engine.on_print(move |msg| {
        if let Ok(mut out) = out_clone2.lock() {
            out.logs.push(msg.to_string());
        }
    });

    engine
}

/// Executes a Rhai script in a sandboxed context and collects output meshes and logs.
pub fn execute_script(script: &str) -> Result<ScriptExecutionOutput, String> {
    let output = Arc::new(Mutex::new(ScriptExecutionOutput::default()));
    let engine = create_scripting_engine(output.clone());

    let mut scope = Scope::new();
    match engine.run_with_scope(&mut scope, script) {
        Ok(()) => {
            let res = output.lock().map_err(|e| e.to_string())?.clone();
            Ok(res)
        }
        Err(err) => Err(err.to_string()),
    }
}

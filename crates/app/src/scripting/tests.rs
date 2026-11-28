#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    use crate::scripting::engine::execute_script;

    #[test]
    fn test_rhai_cube_generation() {
        let script = r#"
            let cube = create_cube(2.0);
            print(`Verts: ${cube.vertex_count()}`);
            add_mesh("ScriptCube", cube);
        "#;

        let result = execute_script(script).expect("Script execution should succeed");
        assert_eq!(result.meshes_to_add.len(), 1);
        assert_eq!(result.meshes_to_add[0].0, "ScriptCube");
        assert_eq!(result.meshes_to_add[0].1.vertex_count(), 8);
        assert_eq!(result.meshes_to_add[0].1.face_count(), 6);
    }

    #[test]
    fn test_rhai_procedural_pipeline() {
        let script = r#"
            let cube = create_cube(2.0);
            let extruded = cube.extrude_all_faces(0.5);
            let sub = extruded.subdivide(1);
            add_mesh("ProceduralObj", sub);
        "#;

        let result = execute_script(script).expect("Pipeline should succeed");
        assert_eq!(result.meshes_to_add.len(), 1);
        let mesh = &result.meshes_to_add[0].1;
        assert!(mesh.face_count() > 6);
        assert!(mesh.vertex_count() > 8);
    }

    #[test]
    fn test_rhai_infinite_loop_protection() {
        let script = r#"
            let i = 0;
            while true {
                i += 1;
            }
        "#;

        let result = execute_script(script);
        assert!(
            result.is_err(),
            "Sandbox should prevent runaway infinite loop"
        );
    }

    #[test]
    fn test_rhai_scattering_pipeline() {
        let script = r#"
            let ground = create_plane(10.0, 10.0, 4, 4);
            let rock = create_cube(0.5);
            let points = ground.distribute_points(2.0, 123);
            print(`Scattered points: ${points.count()}`);
            let field = points.instance_on_points(rock);
            add_mesh("ScatteredRocks", field);
        "#;

        let result = execute_script(script).expect("Scattering script should succeed");
        assert_eq!(result.meshes_to_add.len(), 1);
        let mesh = &result.meshes_to_add[0].1;
        assert!(mesh.vertex_count() >= 8 * 2);
    }
}

use core_math::Transform;
use geo_kernel::Mesh;
use graph_engine::{NodeData, SceneNode};

use super::UiActions;
use crate::state::AppState;

pub fn handle_io_ops(state: &mut AppState, actions: &mut UiActions) {
    // Handle Plumb Native Binary Project (.plmb) Export
    if actions.export_plmb {
        if let Some(sel_id) = state.selected_node {
            if let Some(node) = state.scene.nodes.get(sel_id) {
                if let NodeData::Mesh { base_mesh, .. } = &node.data {
                    let plmb_bytes = base_mesh.to_plmb(&node.name);
                    let file_path = format!("{}.plmb", node.name);
                    let _ = std::fs::write(&file_path, plmb_bytes);
                    println!("Exported Plumb 3D Native Binary to: {}", file_path);
                }
            }
        }
    }

    // Handle glTF 2.0 Export
    if actions.export_gltf {
        if let Some(sel_id) = state.selected_node {
            if let Some(node) = state.scene.nodes.get(sel_id) {
                if let NodeData::Mesh { base_mesh, .. } = &node.data {
                    let gltf_content = base_mesh.to_gltf(&node.name);
                    let file_path = format!("{}.gltf", node.name);
                    let _ = std::fs::write(&file_path, gltf_content);
                    println!("Exported glTF 2.0 to: {}", file_path);
                }
            }
        }
    }

    // Handle OBJ Export
    if actions.export_obj {
        if let Some(sel_id) = state.selected_node {
            if let Some(node) = state.scene.nodes.get(sel_id) {
                if let NodeData::Mesh { base_mesh, .. } = &node.data {
                    let obj_content = base_mesh.to_obj(&node.name);
                    let file_path = format!("{}.obj", node.name);
                    let _ = std::fs::write(&file_path, obj_content);
                    println!("Exported OBJ to: {}", file_path);
                }
            }
        }
    }

    // Handle STL Export (Standard 3D Printing / Stereolithography)
    if actions.export_stl {
        if let Some(sel_id) = state.selected_node {
            if let Some(node) = state.scene.nodes.get(sel_id) {
                if let NodeData::Mesh { base_mesh, .. } = &node.data {
                    let stl_bytes = base_mesh.to_stl_binary();
                    let file_path = format!("{}.stl", node.name);
                    let _ = std::fs::write(&file_path, stl_bytes);
                    println!("Exported Binary STL to: {}", file_path);
                }
            }
        }
    }

    // Handle PLY Export (Stanford Polygon File Format / Point Clouds)
    if actions.export_ply {
        if let Some(sel_id) = state.selected_node {
            if let Some(node) = state.scene.nodes.get(sel_id) {
                if let NodeData::Mesh { base_mesh, .. } = &node.data {
                    let ply_bytes = base_mesh.to_ply_binary();
                    let file_path = format!("{}.ply", node.name);
                    let _ = std::fs::write(&file_path, ply_bytes);
                    println!("Exported Binary PLY to: {}", file_path);
                }
            }
        }
    }

    // Handle OBJ Import
    if let Some(obj_src) = actions.import_obj_text.take() {
        if let Ok(imported_mesh) = Mesh::from_obj(&obj_src) {
            state.selected_vertices.clear();
            state.selected_edges.clear();
            state.selected_faces.clear();
            if let Some(first_v) = imported_mesh.vertices.keys().next() {
                state.selected_vertices.insert(first_v);
            }
            if let Some(first_e) = imported_mesh.edges.keys().next() {
                state.selected_edges.insert(first_e);
            }
            if let Some(first_f) = imported_mesh.faces.keys().next() {
                state.selected_faces.insert(first_f);
            }
            let id = state.scene.add_node(SceneNode::new_mesh(
                "ImportedMesh",
                Transform::IDENTITY,
                imported_mesh,
            ));
            state.selected_node = Some(id);
            state.object_colors.insert(id, [0.65, 0.75, 0.85]);
            state.object_materials.insert(id, [0.0, 0.5, 0.0, 0.0]);
        }
    }

    // Handle glTF Import
    if let Some(gltf_json) = actions.import_gltf_text.take() {
        if let Ok(imported_mesh) = Mesh::from_gltf(&gltf_json) {
            state.selected_vertices.clear();
            state.selected_edges.clear();
            state.selected_faces.clear();
            if let Some(first_v) = imported_mesh.vertices.keys().next() {
                state.selected_vertices.insert(first_v);
            }
            if let Some(first_e) = imported_mesh.edges.keys().next() {
                state.selected_edges.insert(first_e);
            }
            if let Some(first_f) = imported_mesh.faces.keys().next() {
                state.selected_faces.insert(first_f);
            }
            let id = state.scene.add_node(SceneNode::new_mesh(
                "ImportedGLTF",
                Transform::IDENTITY,
                imported_mesh,
            ));
            state.selected_node = Some(id);
            state.object_colors.insert(id, [0.85, 0.75, 0.65]);
            state.object_materials.insert(id, [0.0, 0.5, 0.0, 0.0]);
        }
    }

    // Handle STL Import
    if let Some(stl_data) = actions.import_stl_bytes.take() {
        if let Ok(imported_mesh) = Mesh::from_stl(&stl_data) {
            state.selected_vertices.clear();
            state.selected_edges.clear();
            state.selected_faces.clear();
            if let Some(first_v) = imported_mesh.vertices.keys().next() {
                state.selected_vertices.insert(first_v);
            }
            if let Some(first_e) = imported_mesh.edges.keys().next() {
                state.selected_edges.insert(first_e);
            }
            if let Some(first_f) = imported_mesh.faces.keys().next() {
                state.selected_faces.insert(first_f);
            }
            let id = state.scene.add_node(SceneNode::new_mesh(
                "ImportedSTL",
                Transform::IDENTITY,
                imported_mesh,
            ));
            state.selected_node = Some(id);
            state.object_colors.insert(id, [0.75, 0.85, 0.70]);
            state.object_materials.insert(id, [0.0, 0.4, 0.0, 0.0]);
        }
    }

    // Handle PLY Import
    if let Some(ply_data) = actions.import_ply_bytes.take() {
        if let Ok(imported_mesh) = Mesh::from_ply(&ply_data) {
            state.selected_vertices.clear();
            state.selected_edges.clear();
            state.selected_faces.clear();
            if let Some(first_v) = imported_mesh.vertices.keys().next() {
                state.selected_vertices.insert(first_v);
            }
            if let Some(first_e) = imported_mesh.edges.keys().next() {
                state.selected_edges.insert(first_e);
            }
            if let Some(first_f) = imported_mesh.faces.keys().next() {
                state.selected_faces.insert(first_f);
            }
            let id = state.scene.add_node(SceneNode::new_mesh(
                "ImportedPLY",
                Transform::IDENTITY,
                imported_mesh,
            ));
            state.selected_node = Some(id);
            state.object_colors.insert(id, [0.90, 0.70, 0.85]);
            state.object_materials.insert(id, [0.0, 0.5, 0.0, 0.0]);
        }
    }

    // Handle Plumb Native Binary Project (.plmb) Import
    if let Some(plmb_data) = actions.import_plmb_bytes.take() {
        if let Ok((imported_mesh, meta)) = Mesh::from_plmb(&plmb_data) {
            state.selected_vertices.clear();
            state.selected_edges.clear();
            state.selected_faces.clear();
            if let Some(first_v) = imported_mesh.vertices.keys().next() {
                state.selected_vertices.insert(first_v);
            }
            if let Some(first_e) = imported_mesh.edges.keys().next() {
                state.selected_edges.insert(first_e);
            }
            if let Some(first_f) = imported_mesh.faces.keys().next() {
                state.selected_faces.insert(first_f);
            }
            let node_name = if !meta.model_name.is_empty() {
                meta.model_name
            } else {
                "PlumbModel".to_string()
            };
            let id = state.scene.add_node(SceneNode::new_mesh(
                node_name,
                Transform::IDENTITY,
                imported_mesh,
            ));
            state.selected_node = Some(id);
            state.object_colors.insert(id, [0.85, 0.70, 0.40]);
            state.object_materials.insert(id, [0.0, 0.35, 0.0, 0.0]);
            println!("Loaded .plmb project (Version {}) with {} verts, {} faces",
                meta.version.to_string(), meta.vertex_count, meta.face_count);
        }
    }
}

use crate::state::AppState;
use core_math::Transform;
use geo_kernel::Mesh;
use graph_engine::{NodeData, SceneNode};
use std::path::PathBuf;
use std::time::{Duration, Instant, SystemTime};

/// Manages background autosaving and crash recovery sessions for Plumb 3D.
pub struct AutosaveManager {
    pub last_autosave: Instant,
    pub autosave_interval: Duration,
    pub autosave_dir: PathBuf,
    pub recovery_file: PathBuf,
    pub clean_exit_file: PathBuf,
    pub last_autosave_display: Option<String>,
}

impl Default for AutosaveManager {
    fn default() -> Self {
        let home = std::env::var("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("."));
        let autosave_dir = home.join(".plumb3d").join("autosave");
        let _ = std::fs::create_dir_all(&autosave_dir);

        let recovery_file = autosave_dir.join("recovery_session.plmb");
        let clean_exit_file = autosave_dir.join(".clean_exit");

        Self {
            last_autosave: Instant::now(),
            autosave_interval: Duration::from_secs(180), // 3 minutes
            autosave_dir,
            recovery_file,
            clean_exit_file,
            last_autosave_display: None,
        }
    }
}

impl AutosaveManager {
    /// Checks if a recovery session is available from an unexpected shutdown or crash.
    pub fn check_recovery_available(&self) -> Option<SystemTime> {
        let rec_meta = std::fs::metadata(&self.recovery_file).ok()?;
        if rec_meta.len() < 32 {
            return None;
        }

        let rec_modified = rec_meta.modified().ok()?;

        if let Ok(clean_meta) = std::fs::metadata(&self.clean_exit_file) {
            if let Ok(clean_modified) = clean_meta.modified() {
                if clean_modified >= rec_modified {
                    return None; // Clean shutdown occurred after last autosave
                }
            }
        }

        Some(rec_modified)
    }

    /// Automatically snapshot the current scene if the interval has elapsed.
    pub fn update(&mut self, state: &mut AppState) {
        if self.last_autosave.elapsed() >= self.autosave_interval {
            self.save_snapshot(state);
        }
    }

    /// Saves a recovery snapshot immediately.
    pub fn save_snapshot(&mut self, state: &mut AppState) {
        self.last_autosave = Instant::now();

        // Find primary mesh in scene to snapshot
        let mesh_to_save = if let Some(sel_id) = state.selected_node {
            state.scene.nodes.get(sel_id).and_then(|n| {
                if let NodeData::Mesh { base_mesh, .. } = &n.data {
                    Some((n.name.clone(), base_mesh.clone()))
                } else {
                    None
                }
            })
        } else {
            state.scene.nodes.values().find_map(|n| {
                if let NodeData::Mesh { base_mesh, .. } = &n.data {
                    Some((n.name.clone(), base_mesh.clone()))
                } else {
                    None
                }
            })
        };

        if let Some((name, mesh)) = mesh_to_save {
            let bytes = mesh.to_plmb(&name);
            if std::fs::write(&self.recovery_file, bytes).is_ok() {
                let time_str = format_current_time();
                self.last_autosave_display = Some(time_str.clone());
                state.set_status_message(format!("💾 Auto-saved session ({})", time_str));
            }
        }
    }

    /// Restores the scene from the recovery session file.
    pub fn restore_session(&mut self, state: &mut AppState) -> Result<(), String> {
        let bytes = std::fs::read(&self.recovery_file)
            .map_err(|e| format!("Failed to read recovery file: {}", e))?;

        let (mesh, meta) = Mesh::from_plmb(&bytes)
            .map_err(|e| format!("Failed to parse recovery .plmb: {:?}", e))?;

        state.scene.nodes.clear();
        state.gpu_mesh_cache.clear();
        state.selected_vertices.clear();
        state.selected_edges.clear();
        state.selected_faces.clear();

        let id = state.scene.add_node(SceneNode::new_mesh(
            meta.model_name,
            Transform::IDENTITY,
            mesh,
        ));
        state.selected_node = Some(id);
        state.sync_gpu_meshes();
        state.set_status_message("✨ Restored un-saved recovery session!");
        Ok(())
    }

    /// Records a clean exit timestamp so the user is not prompted next launch.
    pub fn mark_clean_exit(&self) {
        let _ = std::fs::write(&self.clean_exit_file, b"clean_exit");
    }
}

fn format_current_time() -> String {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let hours = (now / 3600) % 24;
    let mins = (now / 60) % 60;
    format!("{:02}:{:02} UTC", hours, mins)
}

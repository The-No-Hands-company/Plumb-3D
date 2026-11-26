pub mod tab_viewer;

pub use tab_viewer::*;

use egui_dock::{DockArea, DockState, NodeIndex, Style};
use crate::state::AppState;
use crate::ui::actions::UiActions;

/// Docking layout manager for customizable drag-and-drop workspace panes
#[derive(Debug, Clone)]
pub struct AppDockState {
    pub dock_state: DockState<DockTab>,
    pub is_enabled: bool,
}

impl Default for AppDockState {
    fn default() -> Self {
        let mut dock_state = DockState::new(vec![DockTab::Viewport]);

        // Split left: Outliner
        let [_left, center] = dock_state.main_surface_mut().split_left(
            NodeIndex::root(),
            0.20,
            vec![DockTab::Outliner],
        );

        // Split right from center: Inspector
        let [center_pane, _right] = dock_state.main_surface_mut().split_right(
            center,
            0.75,
            vec![DockTab::Inspector],
        );

        // Split below center pane: Timeline, Geometry Nodes, UV Editor, Console
        dock_state.main_surface_mut().split_below(
            center_pane,
            0.70,
            vec![DockTab::Timeline, DockTab::NodeEditor, DockTab::UvEditor, DockTab::ScriptConsole],
        );

        Self {
            dock_state,
            is_enabled: false,
        }
    }
}

pub fn draw_dock_area(state: &mut AppState, actions: &mut UiActions) {
    if !state.docking_state.is_enabled {
        return;
    }

    let egui_ctx = state.egui_ctx.clone();
    let style = Style::from_egui(egui_ctx.style().as_ref());

    let mut dock_state = std::mem::replace(&mut state.docking_state.dock_state, DockState::new(vec![]));
    let mut tab_viewer = PlumbTabViewer { state, actions };

    egui::CentralPanel::default()
        .frame(egui::Frame::none().fill(egui::Color32::TRANSPARENT))
        .show(&egui_ctx, |ui| {
            DockArea::new(&mut dock_state)
                .style(style)
                .show_inside(ui, &mut tab_viewer);
        });

    tab_viewer.state.docking_state.dock_state = dock_state;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dock_state_initialization() {
        let app_dock = AppDockState::default();
        assert!(!app_dock.is_enabled);
        assert_eq!(DockTab::Viewport.title(), "3D Viewport");
        assert_eq!(DockTab::Outliner.title(), "Scene Outliner");
        assert_eq!(DockTab::Inspector.title(), "Inspector");
        assert_eq!(DockTab::NodeEditor.title(), "Geometry Nodes");
    }
}

use crate::state::AppState;
use crate::ui::actions::UiActions;
use egui_dock::TabViewer;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum DockTab {
    Viewport,
    Outliner,
    History,
    Inspector,
    NodeEditor,
    UvEditor,
    Timeline,
    ScriptConsole,
}

impl DockTab {
    pub fn title(&self) -> &'static str {
        match self {
            DockTab::Viewport => "3D Viewport",
            DockTab::Outliner => "Scene Outliner",
            DockTab::History => "Action History",
            DockTab::Inspector => "Inspector",
            DockTab::NodeEditor => "Geometry Nodes",
            DockTab::UvEditor => "2D UV Editor",
            DockTab::Timeline => "Animation Timeline",
            DockTab::ScriptConsole => "Script Console",
        }
    }
}

pub struct PlumbTabViewer<'a> {
    pub state: &'a mut AppState,
    pub actions: &'a mut UiActions,
}

impl<'a> TabViewer for PlumbTabViewer<'a> {
    type Tab = DockTab;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        tab.title().into()
    }

    fn clear_background(&self, tab: &Self::Tab) -> bool {
        !matches!(tab, DockTab::Viewport)
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab {
            DockTab::Viewport => {
                ui.allocate_exact_size(ui.available_size(), egui::Sense::hover());
            }
            DockTab::Outliner => {
                crate::ui::outliner::draw_outliner_inner(self.state, self.actions, ui);
            }
            DockTab::History => {
                crate::ui::history_panel::draw_history_inner(self.state, ui);
            }
            DockTab::Inspector => {
                crate::ui::inspector::draw_inspector_inner(self.state, self.actions, ui);
            }
            DockTab::NodeEditor => {
                crate::ui::node_editor::draw_node_editor_inner(self.state, ui);
            }
            DockTab::UvEditor => {
                crate::ui::uv_editor::draw_uv_editor_inner(self.state, self.actions, ui);
            }
            DockTab::Timeline => {
                crate::ui::timeline::draw_timeline_inner(self.state, self.actions, ui);
            }
            DockTab::ScriptConsole => {
                crate::scripting::console::draw_script_console_inner(self.state, ui);
            }
        }
    }
}

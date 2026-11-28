pub mod actions;
pub mod align_dialog;
pub mod bake_dialog;
pub mod collab_dialog;
pub mod command_palette;
pub mod context_menu;
pub mod docking;
pub mod help_dialog;
pub mod help_tutorials;
pub mod history_panel;
pub mod inspector;
pub mod node_editor;
pub mod outliner;
pub mod overlays;
pub mod preferences;
pub mod print_inspector;
pub mod render_dialog;
pub mod theme;
pub mod timeline;
pub mod top_menu;
pub mod uv_editor;
pub mod viewport_hints;
pub mod welcome_hub;
pub mod workspaces;

pub use actions::*;
pub use align_dialog::*;
pub use bake_dialog::*;
pub use collab_dialog::*;
pub use command_palette::*;
pub use context_menu::*;
pub use help_dialog::*;
pub use history_panel::*;
pub use inspector::*;
pub use node_editor::*;
pub use outliner::*;
pub use overlays::*;
pub use preferences::*;
pub use print_inspector::*;
pub use render_dialog::*;
pub use theme::*;
pub use timeline::*;
pub use top_menu::*;
pub use uv_editor::*;
pub use viewport_hints::*;
pub use welcome_hub::*;
pub use workspaces::*;

use crate::scripting::console::draw_script_console;
use crate::state::AppState;

pub fn draw_ui(state: &mut AppState) {
    let mut actions = UiActions::default();

    top_menu::draw_top_menu(state, &mut actions);
    timeline::draw_status_bar(state);

    if state.docking_state.is_enabled {
        docking::draw_dock_area(state, &mut actions);
    } else {
        timeline::draw_timeline(state, &mut actions);
        outliner::draw_outliner(state, &mut actions);
        inspector::draw_inspector(state, &mut actions);
        uv_editor::draw_uv_editor(state, &mut actions);
        node_editor::draw_node_editor(state);
        draw_script_console(state);
    }

    align_dialog::draw_align_dialog(state, &mut actions);
    render_dialog::draw_render_dialog(state);
    bake_dialog::draw_bake_dialog(state);
    print_inspector::draw_print_inspector(state);
    collab_dialog::draw_collab_dialog(state);
    help_dialog::draw_help_dialog(state);
    context_menu::draw_context_menu(state, &mut actions);

    if state.preferences.show_viewport_hints {
        viewport_hints::draw_viewport_hints(state);
    }
    command_palette::draw_command_palette(state, &mut actions);
    history_panel::draw_history_window(state);
    welcome_hub::draw_welcome_hub(state, &mut actions);
    preferences::draw_preferences_dialog(state);

    actions::execute_ui_actions(state, actions);
    overlays::draw_overlays(state);
}

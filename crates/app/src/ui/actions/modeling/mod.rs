pub mod curves;
pub mod mesh_ops;
pub mod separate_join;
pub mod topology;

use crate::state::AppState;
use crate::ui::actions::UiActions;

pub fn handle_modeling(state: &mut AppState, actions: &mut UiActions) {
    topology::handle_topology_actions(state, actions);
    separate_join::handle_separate_join_actions(state, actions);
    curves::handle_curve_actions(state, actions);
    mesh_ops::handle_mesh_ops_actions(state, actions);
}

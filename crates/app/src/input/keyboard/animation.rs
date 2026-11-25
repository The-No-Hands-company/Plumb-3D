use graph_engine::InterpolationMode;
use winit::keyboard::KeyCode;

use crate::state::AppState;
use crate::types::InteractionMode;

pub fn handle_animation_key(state: &mut AppState, key_code: KeyCode) -> bool {
    match key_code {
        KeyCode::Space => {
            state.animation.is_playing = !state.animation.is_playing;
            state.window.request_redraw();
            true
        }
        KeyCode::KeyI if state.interaction_mode == InteractionMode::Object || state.interaction_mode == InteractionMode::Pose => {
            let frame = state.animation.current_frame.round();
            let targets: Vec<_> = if !state.selected_nodes.is_empty() {
                state.selected_nodes.iter().copied().collect()
            } else if let Some(sel) = state.selected_node {
                vec![sel]
            } else {
                Vec::new()
            };

            for target_id in targets {
                if let Some(node) = state.scene.nodes.get(target_id) {
                    state.animation.insert_keyframe(
                        target_id,
                        frame,
                        node.local_transform,
                        InterpolationMode::Linear,
                    );
                }
            }
            state.window.request_redraw();
            true
        }
        KeyCode::ArrowLeft => {
            if state.shift_pressed {
                state.animation.scrub_to_frame(state.animation.start_frame, &mut state.scene);
            } else {
                let prev = (state.animation.current_frame - 1.0).max(state.animation.start_frame);
                state.animation.scrub_to_frame(prev, &mut state.scene);
            }
            state.window.request_redraw();
            true
        }
        KeyCode::ArrowRight => {
            if state.shift_pressed {
                state.animation.scrub_to_frame(state.animation.end_frame, &mut state.scene);
            } else {
                let next = (state.animation.current_frame + 1.0).min(state.animation.end_frame);
                state.animation.scrub_to_frame(next, &mut state.scene);
            }
            state.window.request_redraw();
            true
        }
        KeyCode::ArrowUp => {
            // Jump to next keyframe
            let cur = state.animation.current_frame;
            let mut next_k: Option<f32> = None;
            for track in state.animation.tracks.values() {
                for f in track.keyframe_frames() {
                    if f > cur + 0.05
                        && (next_k.is_none() || f < next_k.unwrap()) {
                            next_k = Some(f);
                        }
                }
            }
            if let Some(target_f) = next_k {
                state.animation.scrub_to_frame(target_f, &mut state.scene);
                state.window.request_redraw();
                true
            } else {
                false
            }
        }
        KeyCode::ArrowDown => {
            // Jump to previous keyframe
            let cur = state.animation.current_frame;
            let mut prev_k: Option<f32> = None;
            for track in state.animation.tracks.values() {
                for f in track.keyframe_frames() {
                    if f < cur - 0.05
                        && (prev_k.is_none() || f > prev_k.unwrap()) {
                            prev_k = Some(f);
                        }
                }
            }
            if let Some(target_f) = prev_k {
                state.animation.scrub_to_frame(target_f, &mut state.scene);
                state.window.request_redraw();
                true
            } else {
                false
            }
        }
        _ => false,
    }
}

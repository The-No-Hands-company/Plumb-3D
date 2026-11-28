//! # app
//!
//! Orchestration crate combining window lifecycle management (`winit`), immediate mode GUI (`egui`),
//! interactive viewport modals, transactional undo/redo commands, Rhai scripting console, and render passes.
//!
//! ## Submodules
//! - [`commands`]: Undoable command implementations for modeling, scene hierarchy, materials, and painting.
//! - [`input`]: Mouse, keyboard, and viewport raycasting / picking event handlers.
//! - [`modals`]: Isolated state machines for interactive drag operations (Extrude, Bevel, Inset, Knife, LoopCut, Crease).
//! - [`render`]: wgpu rendering pass coordinator and egui overlay compositor.
//! - [`scripting`]: Embedded Rhai scripting runtime and live console window.
//! - [`state`]: Global [`AppState`] representing scene graph, active tool, selection, and viewport configurations.
//! - [`ui`]: Complete egui DCC interface (Top menu bar, ribbon header, outliner, inspector, UV editor, help manual, node editor).

pub mod autosave;
pub mod commands;
pub mod input;
pub mod live_pathtracer;
pub mod modals;
pub mod render;
pub mod scenes;
pub mod scripting;
pub mod state;
pub mod types;
pub mod ui;

pub use autosave::*;
pub use commands::*;
pub use input::*;
pub use live_pathtracer::*;
pub use render::*;
pub use scenes::*;
pub use scripting::*;
pub use state::*;
pub use types::*;
pub use ui::*;

use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::PhysicalKey;
use winit::window::{Window, WindowId};

pub struct App {
    state: Option<AppState>,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        Self { state: None }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_none() {
            let window_attributes = Window::default_attributes()
                .with_title("Plumb 3D — Architectural & CAD DCC Suite")
                .with_inner_size(winit::dpi::LogicalSize::new(1280, 800));

            let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
            self.state = Some(AppState::new(window));
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let state = match &mut self.state {
            Some(s) => s,
            None => return,
        };

        let response = state.egui_state.on_window_event(&state.window, &event);
        if response.consumed {
            state.window.request_redraw();
            return;
        }

        match event {
            WindowEvent::CloseRequested => {
                state.autosave.mark_clean_exit();
                event_loop.exit();
            }
            WindowEvent::Resized(physical_size) => {
                state.resize(physical_size);
                state.window.request_redraw();
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state: ElementState::Pressed,
                        physical_key: PhysicalKey::Code(key_code),
                        ..
                    },
                ..
            } => {
                input::handle_keyboard_input(state, key_code);
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                state.shift_pressed = modifiers.state().shift_key();
                state.ctrl_pressed = modifiers.state().control_key();
                state.alt_pressed = modifiers.state().alt_key();
            }
            WindowEvent::MouseInput {
                state: elem_state,
                button,
                ..
            } => {
                input::handle_mouse_input(state, button, elem_state == ElementState::Pressed);
            }
            WindowEvent::CursorMoved { position, .. } => {
                input::handle_cursor_moved(state, position);
            }
            WindowEvent::MouseWheel { delta, .. } => {
                input::handle_mouse_wheel(state, delta);
            }
            WindowEvent::RedrawRequested => {
                let mut autosave = std::mem::take(&mut state.autosave);
                autosave.update(state);
                state.autosave = autosave;
                state.render();
                state.window.request_redraw();
            }
            _ => {}
        }
    }
}

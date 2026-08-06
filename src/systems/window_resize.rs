//! Window resize handler.
//!
//! `on_window_resized` is a logic-thread observer for [`WindowResizedEvent`], which the
//! engine triggers whenever the render thread's window-size sample changes (see
//! `aberredengine::events::window`). It writes the new size into `GameConfig` (which rides
//! the `DrawableSnapshot`/`RenderGameConfig` mirror to the render thread and triggers
//! `apply_gameconfig_changes` to recreate the render target) and re-centers the camera
//! offset so the letterbox stays centred.
//!
//! Registered globally via `EngineBuilder::add_observer`, so it also applies during the
//! intro scene — a resize while the window title/logo is showing is picked up immediately,
//! with nothing left to reconcile once the editor scene is entered (the engine itself
//! inserts `Camera2DRes`/`WindowSize` in sync with `GameConfig`'s initial render size at
//! startup, so there is no separate "initial size" case to handle here).
use aberredengine::bevy_ecs::prelude::*;
use aberredengine::core::events::window::WindowResizedEvent;
use aberredengine::core::resources::camera2d::Camera2DRes;
use aberredengine::core::resources::gameconfig::GameConfig;
use log::debug;

pub fn on_window_resized(
    trigger: On<WindowResizedEvent>,
    mut config: ResMut<GameConfig>,
    mut camera: ResMut<Camera2DRes>,
) {
    let event = trigger.event();
    let (w, h) = (event.w, event.h);
    if w <= 0 || h <= 0 {
        return;
    }
    config.set_render_size(w as u32, h as u32);
    camera.0.offset.x = w as f32 * 0.5;
    camera.0.offset.y = h as f32 * 0.5;
    debug!("on_window_resized: render size set to {}x{}", w, h);
}

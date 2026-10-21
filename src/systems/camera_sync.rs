//! Per-frame camera state sync to `WorldSignals`.
//!
//! Writes camera target, zoom, rotation, offset, render size, and letterbox parameters every
//! frame so editor systems and GUI overlays can convert between render, world, and window-space
//! coordinates.
//!
//! Also mirrors `GameConfig.pixel_snap_camera` into `render_prefs::RenderPrefsMutex` (in
//! `AppState`) whenever `GameConfig` changes, so the Render Preferences modal can display it.
use crate::signals as sig;
use crate::systems::render_prefs::RenderPrefsMutex;
use aberredengine::bevy_ecs::change_detection::DetectChanges;
use aberredengine::bevy_ecs::prelude::{Res, ResMut};
use aberredengine::core::resources::app_state::AppState;
use aberredengine::core::resources::camera2d::MainCamera;
use aberredengine::core::resources::game_config::GameConfig;
use aberredengine::core::resources::render_size::RenderSize;
use aberredengine::core::resources::window_size::WindowSize;
use aberredengine::core::resources::world_signals::WorldSignals;

/// Per-frame system that writes the active camera, render target, and letterbox parameters to
/// `WorldSignals` so editor systems and GUI overlays can convert coordinates without needing
/// direct ECS access.
pub fn editor_camera_sync_system(
    camera: Res<MainCamera>,
    render_size: Res<RenderSize>,
    window: Res<WindowSize>,
    config: Res<GameConfig>,
    app_state: Res<AppState>,
    mut signals: ResMut<WorldSignals>,
) {
    let lb = window.calculate_letterbox(render_size.w as u32, render_size.h as u32);
    let lb_scale = lb.width / render_size.w as f32;

    signals.set_scalar(sig::CAM_TARGET_X, camera.0.target.x);
    signals.set_scalar(sig::CAM_TARGET_Y, camera.0.target.y);
    signals.set_scalar(sig::CAM_ZOOM, camera.0.zoom);
    signals.set_scalar(sig::CAM_ROTATION, camera.0.rotation);
    signals.set_scalar(sig::CAM_OFFSET_X, camera.0.offset.x);
    signals.set_scalar(sig::CAM_OFFSET_Y, camera.0.offset.y);
    signals.set_scalar(sig::RENDER_WIDTH, render_size.w as f32);
    signals.set_scalar(sig::RENDER_HEIGHT, render_size.h as f32);
    signals.set_scalar(sig::WIN_SCALE, lb_scale);
    signals.set_scalar(sig::WIN_OFFSET_X, lb.x);
    signals.set_scalar(sig::WIN_OFFSET_Y, lb.y);

    if config.is_changed()
        && let Some(mutex) = app_state.get::<RenderPrefsMutex>()
    {
        *mutex.lock().unwrap() = config.pixel_snap_camera;
    }
}

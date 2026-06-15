//! Render preferences observer.
//!
//! `TogglePixelSnapCameraRequested` flips `GameConfig.pixel_snap_camera`. The current value is
//! mirrored to `WorldSignals` every frame by `camera_sync::editor_camera_sync_system` so the
//! "Render Preferences" modal can display it without direct `GameConfig` access.
use aberredengine::bevy_ecs;
use aberredengine::bevy_ecs::prelude::{Event, On, ResMut};
use aberredengine::engine_app::EngineBuilder;
use aberredengine::resources::gameconfig::GameConfig;

/// Toggle `GameConfig.pixel_snap_camera`.
#[derive(Event)]
pub struct TogglePixelSnapCameraRequested;

/// AppState mirror of `GameConfig.pixel_snap_camera`, refreshed every frame by
/// `camera_sync::editor_camera_sync_system` so the Render Preferences modal can
/// display the current value without direct `GameConfig` access.
pub type RenderPrefsMutex = std::sync::Mutex<bool>;

pub fn toggle_pixel_snap_camera_observer(
    _trigger: On<TogglePixelSnapCameraRequested>,
    mut config: ResMut<GameConfig>,
) {
    config.pixel_snap_camera = !config.pixel_snap_camera;
}

pub fn register(builder: EngineBuilder) -> EngineBuilder {
    builder.add_observer(toggle_pixel_snap_camera_observer)
}

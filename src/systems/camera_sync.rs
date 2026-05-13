//! Per-frame camera state sync to `WorldSignals`.
//!
//! Writes camera target, zoom, rotation, offset, render size, and letterbox parameters every
//! frame so editor systems and GUI overlays can convert between render, world, and window-space
//! coordinates.
use crate::signals as sig;
use aberredengine::bevy_ecs::prelude::{Res, ResMut};
use aberredengine::resources::camera2d::Camera2DRes;
use aberredengine::resources::screensize::ScreenSize;
use aberredengine::resources::windowsize::WindowSize;
use aberredengine::resources::worldsignals::WorldSignals;

/// Per-frame system that writes the active camera, render target, and letterbox parameters to
/// `WorldSignals` so editor systems and GUI overlays can convert coordinates without needing
/// direct ECS access.
pub fn editor_camera_sync_system(
    camera: Res<Camera2DRes>,
    screen: Res<ScreenSize>,
    window: Res<WindowSize>,
    mut signals: ResMut<WorldSignals>,
) {
    let lb = window.calculate_letterbox(screen.w as u32, screen.h as u32);
    let lb_scale = lb.width / screen.w as f32;

    signals.set_scalar(sig::CAM_TARGET_X, camera.0.target.x);
    signals.set_scalar(sig::CAM_TARGET_Y, camera.0.target.y);
    signals.set_scalar(sig::CAM_ZOOM, camera.0.zoom);
    signals.set_scalar(sig::CAM_ROTATION, camera.0.rotation);
    signals.set_scalar(sig::CAM_OFFSET_X, camera.0.offset.x);
    signals.set_scalar(sig::CAM_OFFSET_Y, camera.0.offset.y);
    signals.set_scalar(sig::RENDER_WIDTH, screen.w as f32);
    signals.set_scalar(sig::RENDER_HEIGHT, screen.h as f32);
    signals.set_scalar(sig::WIN_SCALE, lb_scale);
    signals.set_scalar(sig::WIN_OFFSET_X, lb.x);
    signals.set_scalar(sig::WIN_OFFSET_Y, lb.y);
}

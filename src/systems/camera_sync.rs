//! Per-frame camera state sync to `WorldSignals`.
//!
//! Writes camera target, zoom, offset, and letterbox parameters every frame so the GUI
//! callback can project world-space entity positions onto the ImGui screen coordinate system
//! (used by `overlay::draw_selection_outline`).
use crate::signals as sig;
use aberredengine::bevy_ecs::prelude::{Res, ResMut};
use aberredengine::resources::camera2d::Camera2DRes;
use aberredengine::resources::screensize::ScreenSize;
use aberredengine::resources::windowsize::WindowSize;
use aberredengine::resources::worldsignals::WorldSignals;

/// Per-frame system that writes the active camera and letterbox parameters to
/// `WorldSignals` so the GUI callback can project world-space coordinates onto
/// the ImGui screen without needing direct ECS access.
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
    signals.set_scalar(sig::CAM_OFFSET_X, camera.0.offset.x);
    signals.set_scalar(sig::CAM_OFFSET_Y, camera.0.offset.y);
    signals.set_scalar(sig::WIN_SCALE, lb_scale);
    signals.set_scalar(sig::WIN_OFFSET_X, lb.x);
    signals.set_scalar(sig::WIN_OFFSET_Y, lb.y);
}

use aberredengine::bevy_ecs::prelude::{Res, ResMut};
use aberredengine::resources::camera2d::Camera2DRes;
use aberredengine::resources::screensize::ScreenSize;
use aberredengine::resources::windowsize::WindowSize;
use aberredengine::resources::worldsignals::WorldSignals;

/// Per-frame system that writes the active camera and letterbox parameters to
/// `WorldSignals` so the GUI callback can project world-space coordinates onto
/// the ImGui screen without needing direct ECS access.
///
/// Keys written every frame:
/// - `editor:cam:target_x/y` — camera target (world origin of the screen center)
/// - `editor:cam:zoom`       — current zoom level
/// - `editor:cam:offset_x/y` — camera offset (screen-space pivot, usually half the render size)
/// - `editor:win:scale`      — letterbox scale (render pixels per window pixel)
/// - `editor:win:offset_x/y` — letterbox origin in window space (pixels from top-left)
pub fn editor_camera_sync_system(
    camera: Res<Camera2DRes>,
    screen: Res<ScreenSize>,
    window: Res<WindowSize>,
    mut signals: ResMut<WorldSignals>,
) {
    let lb = window.calculate_letterbox(screen.w as u32, screen.h as u32);
    // lb.width covers `screen.w` game pixels → scale = lb.width / screen.w
    let lb_scale = lb.width / screen.w as f32;

    signals.set_scalar("editor:cam:target_x", camera.0.target.x);
    signals.set_scalar("editor:cam:target_y", camera.0.target.y);
    signals.set_scalar("editor:cam:zoom", camera.0.zoom);
    signals.set_scalar("editor:cam:offset_x", camera.0.offset.x);
    signals.set_scalar("editor:cam:offset_y", camera.0.offset.y);
    signals.set_scalar("editor:win:scale", lb_scale);
    signals.set_scalar("editor:win:offset_x", lb.x);
    signals.set_scalar("editor:win:offset_y", lb.y);
}

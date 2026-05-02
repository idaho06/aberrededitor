//! Window resize handler.
//!
//! `sync_render_size_to_window` is an observer registered via `configure_resize_schedule`.
//! It recreates the Raylib render target to match the new window size and updates `ScreenSize`,
//! `WindowSize`, and the camera offset to keep the letterbox centred.
//! `configure_resize_schedule` wires the observer into the Bevy schedule before `camera_sync`.
use aberredengine::bevy_ecs::prelude::*;
use aberredengine::resources::camera2d::Camera2DRes;
use aberredengine::resources::gameconfig::GameConfig;
use aberredengine::resources::rendertarget::RenderTarget;
use aberredengine::resources::screensize::ScreenSize;
use aberredengine::resources::windowsize::WindowSize;
use aberredengine::systems::RaylibAccess;
use aberredengine::systems::gamestate::{check_pending_state, state_is_playing};
use log::debug;

use crate::systems::camera_sync::editor_camera_sync_system;

fn sync_camera_offset(camera: &mut Camera2DRes, width: u32, height: u32) {
    camera.0.offset.x = width as f32 * 0.5;
    camera.0.offset.y = height as f32 * 0.5;
}

pub fn sync_render_size_to_window(
    mut raylib: RaylibAccess,
    mut render_target: NonSendMut<RenderTarget>,
    mut screen_size: ResMut<ScreenSize>,
    mut window_size: ResMut<WindowSize>,
    mut camera: ResMut<Camera2DRes>,
    mut config: ResMut<GameConfig>,
) {
    let (current_w, current_h) = {
        let rl = &mut *raylib.rl;
        (rl.get_screen_width(), rl.get_screen_height())
    };

    window_size.w = current_w;
    window_size.h = current_h;

    if current_w <= 0 || current_h <= 0 {
        return;
    }

    let new_width = current_w as u32;
    let new_height = current_h as u32;

    if render_target.game_width == new_width && render_target.game_height == new_height {
        sync_camera_offset(&mut camera, new_width, new_height);
        if config.render_width != new_width || config.render_height != new_height {
            config.set_render_size(new_width, new_height);
        }
        return;
    }

    let (rl, th) = (&mut *raylib.rl, &*raylib.th);
    if render_target
        .recreate(rl, th, new_width, new_height)
        .is_err()
    {
        return;
    }

    debug!(
        "sync_render_size_to_window: resized render target to {}x{}",
        new_width, new_height
    );

    screen_size.w = new_width as i32;
    screen_size.h = new_height as i32;
    sync_camera_offset(&mut camera, new_width, new_height);
    config.set_render_size(new_width, new_height);
}

pub fn configure_resize_schedule(schedule: &mut Schedule) {
    schedule.add_systems(
        sync_render_size_to_window
            .run_if(state_is_playing)
            .after(check_pending_state)
            .before(editor_camera_sync_system),
    );
}

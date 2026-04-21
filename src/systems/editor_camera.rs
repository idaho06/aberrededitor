use crate::signals as sig;
use aberredengine::bevy_ecs::prelude::{Query, Res, ResMut};
use aberredengine::components::cameratarget::CameraTarget;
use aberredengine::components::mapposition::MapPosition;
use aberredengine::resources::input::InputState;
use aberredengine::resources::worldsignals::WorldSignals;
use aberredengine::resources::worldtime::WorldTime;

pub fn editor_camera_system(
    mut world_signals: ResMut<WorldSignals>,
    mut camera_targets: Query<&mut CameraTarget>,
    mut positions: Query<&mut MapPosition>,
    world_time: Res<WorldTime>,
    input: Res<InputState>,
) {
    let Some(entity) = world_signals.get_entity(sig::EDITOR_CAMERA).copied() else {
        return;
    };

    if world_signals.has_flag(sig::IMGUI_WANTS_KEYBOARD) {
        return;
    }

    if world_signals.take_flag(sig::ACTION_VIEW_RESET_ZOOM)
        && let Ok(mut ct) = camera_targets.get_mut(entity)
    {
        ct.zoom = 1.0;
    }

    let mut dx = 0.0_f32;
    let mut dy = 0.0_f32;
    if input.maindirection_left.active || input.secondarydirection_left.active {
        dx -= 1.0;
    }
    if input.maindirection_right.active || input.secondarydirection_right.active {
        dx += 1.0;
    }
    if input.maindirection_up.active || input.secondarydirection_up.active {
        dy -= 1.0;
    }
    if input.maindirection_down.active || input.secondarydirection_down.active {
        dy += 1.0;
    }
    if dx != 0.0 || dy != 0.0 {
        let pan_speed = 300.0_f32; // pixels/sec at zoom 1.0
        let zoom = camera_targets.get(entity).map(|ct| ct.zoom).unwrap_or(1.0);
        let speed = pan_speed * world_time.delta / zoom;
        if let Ok(mut pos) = positions.get_mut(entity) {
            pos.translate(dx * speed, dy * speed);
        }
    }

    if input.scroll_y.abs() > 0.0
        && let Ok(mut ct) = camera_targets.get_mut(entity)
    {
        let factor = 1.1_f32.powf(input.scroll_y);
        ct.zoom = (ct.zoom * factor).clamp(0.1, 10.0);
    }
}

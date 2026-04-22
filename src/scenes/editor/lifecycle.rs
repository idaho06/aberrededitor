use super::state::clear_entity_editor_pending;
use crate::signals as sig;
use crate::systems::entity_selector::clear_selector_state;
use aberredengine::components::cameratarget::CameraTarget;
use aberredengine::components::mapposition::MapPosition;
use aberredengine::events::input::InputAction;
use aberredengine::raylib::camera::Camera2D;
use aberredengine::raylib::ffi::{KeyboardKey, MouseButton};
use aberredengine::resources::camera2d::Camera2DRes;
use aberredengine::resources::camerafollowconfig::FollowMode;
use aberredengine::resources::input_bindings::InputBinding;
use aberredengine::systems::GameCtx;
use log::info;

pub fn editor_enter(ctx: &mut GameCtx) {
    info!("editor_enter: entering editor scene");

    let rw = ctx.config.render_width as f32;
    let rh = ctx.config.render_height as f32;

    ctx.commands.insert_resource(Camera2DRes(Camera2D {
        offset: (rw / 2.0, rh / 2.0).into(),
        target: (0.0, 0.0).into(),
        rotation: 0.0,
        zoom: 1.0,
    }));

    let entity = ctx
        .commands
        .spawn((MapPosition::new(0.0, 0.0), CameraTarget::new(0)))
        .id();
    ctx.world_signals.set_entity(sig::EDITOR_CAMERA, entity);

    ctx.camera_follow.enabled = true;
    ctx.camera_follow.mode = FollowMode::Instant;
    ctx.camera_follow.zoom_lerp_speed = 10.0;

    // Rebind Action1 to mouse-left only so Space doesn't trigger entity picking
    ctx.input_bindings.rebind(
        InputAction::Action1,
        InputBinding::MouseButton(MouseButton::MOUSE_BUTTON_LEFT),
    );
}

pub fn editor_exit(ctx: &mut GameCtx) {
    info!("editor_exit: leaving editor scene");

    // Restore default Action1 bindings (Space + MouseLeft)
    ctx.input_bindings.rebind(
        InputAction::Action1,
        InputBinding::Keyboard(KeyboardKey::KEY_SPACE),
    );
    ctx.input_bindings.add_binding(
        InputAction::Action1,
        InputBinding::MouseButton(MouseButton::MOUSE_BUTTON_LEFT),
    );

    clear_selector_state(&mut ctx.world_signals, &mut ctx.app_state);
    clear_entity_editor_pending(&ctx.app_state);
    ctx.world_signals.clear_flag(sig::IMGUI_WANTS_MOUSE);
    ctx.world_signals.clear_flag(sig::IMGUI_WANTS_KEYBOARD);
}

//! Editor scene enter and exit handlers.
//!
//! `editor_enter` configures the camera and rebinds `Action1` to mouse-left-only (so WASD
//! panning works without the mouse button firing entity picks) and resets the shared selection
//! mode to its default. `editor_exit` restores input bindings and clears selector and pending
//! state to avoid stale state on scene re-entry.
use super::ctx::EditorCtx;
use super::{exit_placement_mode, reset_tool, state::clear_entity_editor_pending};
use crate::signals as sig;
use crate::systems::entity_selector::clear_selector_state;
use crate::systems::file_dialogs::clear_async_dialog;
use aberredengine::core::resources::input_bindings::{
    InputBinding, InputBindings, Key, MouseButton,
};
use aberredengine::prelude::{
    Camera2D, CameraFollowConfig, CameraTarget, FollowMode, GameConfig, InputAction, MainCamera,
    On, Res, ResMut, SceneEntered, SceneExited, Vec2, WorldPosition,
};
use log::info;

pub fn editor_enter(
    _: On<SceneEntered>,
    mut ctx: EditorCtx,
    config: Res<GameConfig>,
    mut bindings: ResMut<InputBindings>,
    mut follow: ResMut<CameraFollowConfig>,
) {
    info!("editor_enter: entering editor scene");

    let rw = config.render_width as f32;
    let rh = config.render_height as f32;

    ctx.commands.insert_resource(MainCamera(Camera2D::new(
        Vec2::ZERO,
        Vec2::new(rw / 2.0, rh / 2.0),
    )));

    let entity = ctx
        .commands
        .spawn((WorldPosition::new(0.0, 0.0), CameraTarget::new(0)))
        .id();
    ctx.world_signals
        .register_entity(sig::EDITOR_CAMERA, entity);

    follow.enabled = true;
    follow.mode = FollowMode::Instant;
    follow.zoom_lerp_speed = 10.0;
    reset_tool(&ctx.app_state);

    // Rebind Action1 to mouse-left only so Space doesn't trigger entity picking
    bindings.rebind(
        InputAction::Action1,
        InputBinding::MouseButton(MouseButton::MOUSE_BUTTON_LEFT),
    );
}

pub fn editor_exit(_: On<SceneExited>, mut ctx: EditorCtx, mut bindings: ResMut<InputBindings>) {
    info!("editor_exit: leaving editor scene");

    // Restore default Action1 bindings (Space + MouseLeft)
    bindings.rebind(InputAction::Action1, InputBinding::Keyboard(Key::KEY_SPACE));
    bindings.add_binding(
        InputAction::Action1,
        InputBinding::MouseButton(MouseButton::MOUSE_BUTTON_LEFT),
    );

    clear_selector_state(&mut ctx.world_signals, &mut ctx.app_state);
    clear_async_dialog(&ctx.app_state);
    clear_entity_editor_pending(&ctx.app_state);
    ctx.world_signals.remove_flag(sig::IMGUI_WANTS_MOUSE);
    ctx.world_signals.remove_flag(sig::IMGUI_WANTS_KEYBOARD);
    exit_placement_mode(&ctx.app_state);
}

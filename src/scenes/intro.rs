use aberredengine::resources::input::InputState;
use aberredengine::systems::GameCtx;
use log::info;

pub fn intro_enter(_ctx: &mut GameCtx) {
    info!("intro_enter: entering intro scene");
}

pub fn intro_update(ctx: &mut GameCtx, _dt: f32, _input: &InputState) {
    ctx.world_signals.set_string("scene", "editor");
    ctx.world_signals.set_flag("switch_scene");
}

use aberredengine::components::mapposition::MapPosition;
use aberredengine::components::sprite::Sprite;
use aberredengine::components::zindex::ZIndex;
use aberredengine::resources::input::InputState;
use aberredengine::systems::GameCtx;
use log::info;

pub fn intro_enter(ctx: &mut GameCtx) {
    info!("intro_enter: entering intro scene");
    // spawn an entity with a sprite component to display the intro image
    ctx.commands.spawn_empty().insert((
        Sprite {
            tex_key: "aberred_engine_isometric_alpha".into(),
            width: 807.0,
            height: 970.0,
            origin: (0.0, 0.0).into(),
            offset: (0.0, 0.0).into(),
            flip_h: false,
            flip_v: false,
        },
        MapPosition {
            pos: (0.0, 0.0).into(),
        },
        ZIndex(0.0),
    ));
}

pub fn intro_update(_ctx: &mut GameCtx, _dt: f32, _input: &InputState) {
    //ctx.world_signals.set_string("scene", "editor");
    //ctx.world_signals.set_flag("switch_scene");
}

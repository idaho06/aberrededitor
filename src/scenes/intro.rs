use aberredengine::components::mapposition::MapPosition;
use aberredengine::components::sprite::Sprite;
use aberredengine::components::zindex::ZIndex;
use aberredengine::raylib::camera::Camera2D;
use aberredengine::resources::camera2d::Camera2DRes;
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
            origin: (403.0, 242.0).into(),
            offset: (0.0, 0.0).into(),
            flip_h: false,
            flip_v: false,
        },
        MapPosition {
            pos: (0.0, 0.0).into(),
        },
        ZIndex(0.0),
    ));
    ctx.commands.insert_resource(Camera2DRes(Camera2D {
        offset: (
            ctx.config.render_width as f32 / 2.0,
            ctx.config.render_height as f32 / 2.0,
        )
            .into(),
        target: (0.0, 0.0).into(),
        rotation: 0.0,
        zoom: 1.0,
    }));
}

pub fn intro_update(ctx: &mut GameCtx, _dt: f32, input: &InputState) {
    // if the user presses any action button, switch to the editor scene
    if input.action_1.active
        || input.action_2.active
        || input.action_3.active
        || input.action_back.active
    {
        info!("intro_update: action button pressed, switching to editor scene");
        ctx.world_signals.set_string("scene", "editor");
        ctx.world_signals.set_flag("switch_scene");
    }
}

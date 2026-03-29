use aberredengine::bevy_ecs::prelude::Entity;
use aberredengine::components::mapposition::MapPosition;
use aberredengine::components::phase::{Phase, PhaseCallbackFns};
use aberredengine::components::signals::Signals;
use aberredengine::components::sprite::Sprite;
use aberredengine::components::tween::{Easing, Tween};
use aberredengine::components::zindex::ZIndex;
use aberredengine::raylib::camera::Camera2D;
use aberredengine::resources::camera2d::Camera2DRes;
use aberredengine::resources::input::InputState;
use aberredengine::resources::uniformvalue::UniformValue;
use aberredengine::systems::GameCtx;
use log::info;
use rustc_hash::FxHashMap;

fn shader_control_phases() -> FxHashMap<String, PhaseCallbackFns> {
    let mut phases = FxHashMap::default();
    phases.insert(
        "start".into(),
        PhaseCallbackFns {
            on_enter: None,
            on_update: Some(shader_control_start),
            on_exit: None,
        },
    );
    phases.insert(
        "medium".into(),
        PhaseCallbackFns {
            on_enter: None,
            on_update: Some(shader_control_medium),
            on_exit: None,
        },
    );
    phases.insert(
        "fadeout".into(),
        PhaseCallbackFns {
            on_enter: None,
            on_update: Some(shader_control_fadeout),
            on_exit: None,
        },
    );
    phases
}

fn shader_control_signals() -> Signals {
    let mut signals = Signals::default();
    signals.set_scalar("uglitch", 0.0);
    signals.set_scalar("fade", 0.0);
    signals.set_scalar("phase_time", 0.0);
    signals
}

pub fn intro_enter(ctx: &mut GameCtx) {
    info!("intro_enter: entering intro scene");
    let start_position = MapPosition::new(0.0, ctx.config.render_height as f32);
    let end_position = MapPosition::new(0.0, 0.0);
    let phases = shader_control_phases();

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
        start_position,
        Tween::new(start_position, end_position, 2.0).with_easing(Easing::CubicOut),
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
    ctx.post_process
        .set_shader_chain(Some(vec!["glitch".to_string(), "fade".to_string()]));
    ctx.commands
        .spawn((Phase::new("start", phases), shader_control_signals()));
}

pub fn shader_control_start(
    entity: Entity,
    ctx: &mut GameCtx,
    _input: &InputState,
    dt: f32,
) -> Option<String> {
    let mut entity_signals = ctx.signals.get_mut(entity).ok()?;
    let phase_time = entity_signals.get_scalar("phase_time").unwrap_or(0.0) + dt;
    entity_signals.set_scalar("phase_time", phase_time);
    let uglitch = entity_signals.get_scalar("uglitch").unwrap_or(0.0);
    ctx.post_process
        .set_uniform("uGlitch", UniformValue::Float(uglitch));
    if phase_time > 1.5 {
        entity_signals.set_scalar("phase_time", 0.0);
        entity_signals.set_scalar("uglitch", uglitch);
        return Some("medium".into());
    }
    None
}

pub fn shader_control_medium(
    entity: Entity,
    ctx: &mut GameCtx,
    _input: &InputState,
    dt: f32,
) -> Option<String> {
    let mut entity_signals = ctx.signals.get_mut(entity).ok()?;
    let phase_time = entity_signals.get_scalar("phase_time").unwrap_or(0.0) + dt;
    entity_signals.set_scalar("phase_time", phase_time);
    let uglitch = entity_signals.get_scalar("uglitch").unwrap_or(0.0) + dt * 0.1; // increase uglitch over time
    entity_signals.set_scalar("uglitch", uglitch);
    ctx.post_process
        .set_uniform("uGlitch", UniformValue::Float(uglitch));
    if phase_time > 1.5 {
        entity_signals.set_scalar("phase_time", 0.0);
        entity_signals.set_scalar("uglitch", uglitch);
        return Some("fadeout".into());
    }
    None
}

pub fn shader_control_fadeout(
    entity: Entity,
    ctx: &mut GameCtx,
    _input: &InputState,
    dt: f32,
) -> Option<String> {
    let mut entity_signals = ctx.signals.get_mut(entity).ok()?;
    let phase_time = entity_signals.get_scalar("phase_time").unwrap_or(0.0) + dt;
    entity_signals.set_scalar("phase_time", phase_time);
    let fade = entity_signals.get_scalar("fade").unwrap_or(0.0) + dt * 0.7; // increase fade over time
    entity_signals.set_scalar("fade", fade);
    ctx.post_process.set_uniform(
        "fadeColor",
        UniformValue::Vec4 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: fade,
        },
    );
    if fade > 1.1 {
        // switch to editor scene after fadeout
        ctx.world_signals.set_string("scene", "editor");
        ctx.world_signals.set_flag("switch_scene");
    }
    None
}

pub fn intro_update(ctx: &mut GameCtx, _dt: f32, input: &InputState) {
    // ctx.post_process
    //     .set_uniform("uGlitch", UniformValue::Float(0.0));

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

pub fn intro_exit(ctx: &mut GameCtx) {
    info!("intro_exit: clearing post-process shader chain");
    ctx.post_process.set_shader_chain(None);
}

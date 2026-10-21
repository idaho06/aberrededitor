//! Intro splash scene with glitch and fade shader animations.
//!
//! Uses a `Phase` state machine (`start` → `medium` → `fadeout`) to animate shader uniforms
//! (`uGlitch`, `fadeColor`) over time. The user can skip to the editor scene by pressing any
//! action button. The scene also transitions automatically when the `fadeout` phase completes.
//!
//! This scene demonstrates the canonical scene lifecycle pattern: enter spawns entities and
//! sets up shaders; `intro_shader_system` drives the phase machine; `intro_update` handles the
//! skip input; exit cleans up shaders and despawns assets.
use aberredengine::bevy_ecs;
use aberredengine::bevy_ecs::prelude::{Commands, Component, On, Query, Res, ResMut, With};
use aberredengine::core::components::world_position::WorldPosition;
use aberredengine::core::components::phase::Phase;
use aberredengine::core::components::sprite::Sprite;
use aberredengine::core::components::tween::{Easing, Tween};
use aberredengine::core::components::z_index::ZIndex;
use aberredengine::core::resources::input::InputAction;
use aberredengine::core::events::scene::{SceneEntered, SceneExited};
use aberredengine::core::math::Vec2;
use aberredengine::core::resources::camera2d::{Camera2D, MainCamera};
use aberredengine::core::resources::game_config::GameConfig;
use aberredengine::core::resources::input::InputState;
use aberredengine::core::resources::post_process_shader::PostProcessShader;
use aberredengine::core::resources::uniform_value::UniformValue;
use aberredengine::core::resources::next_scene::NextScene;
use log::info;

use crate::systems::map_ops::RemoveTextureRequested;

const PHASE_DURATION: f32 = 1.5;
const GLITCH_RATE: f32 = 0.1; // uglitch units per second during "medium" phase
const FADE_RATE: f32 = 0.7; // fade alpha units per second during "fadeout" phase
const FADE_THRESHOLD: f32 = 1.1; // alpha at which we switch to the editor

/// Marks the entity whose `Phase` drives the intro's post-process shader animation.
#[derive(Component)]
pub struct IntroShaderControl;

pub fn intro_enter(
    _: On<SceneEntered>,
    mut commands: Commands,
    config: Res<GameConfig>,
    mut post_process: ResMut<PostProcessShader>,
) {
    info!("intro_enter: entering intro scene");
    let start_position = WorldPosition::new(0.0, config.render_height as f32);
    let end_position = WorldPosition::new(0.0, 0.0);

    // spawn an entity with a sprite component to display the intro image
    commands.spawn_empty().insert((
        Sprite::new("aberred_engine_isometric_alpha", 807.0, 970.0)
            .with_origin(Vec2::new(403.0, 242.0)),
        start_position,
        Tween::new(start_position, end_position, 2.0).with_easing(Easing::CubicOut),
        ZIndex(0.0),
    ));
    commands.insert_resource(MainCamera(Camera2D::new(
        Vec2::ZERO,
        Vec2::new(
            config.render_width as f32 / 2.0,
            config.render_height as f32 / 2.0,
        ),
    )));
    post_process.set_shader_chain(Some(vec!["glitch".to_string(), "fade".to_string()]));
    commands.spawn((Phase::new("start"), IntroShaderControl));
}

/// Drives the `start` → `medium` → `fadeout` shader animation. The engine's phase system
/// applies `phase.next` and advances `time_in_phase` (reset on each transition) before this
/// runs, so the glitch and fade amounts are derived from it.
pub fn intro_shader_system(
    mut query: Query<&mut Phase, With<IntroShaderControl>>,
    mut post_process: ResMut<PostProcessShader>,
    mut next_scene: ResMut<NextScene>,
) {
    for mut phase in &mut query {
        let t = phase.time_in_phase;
        match phase.current.as_str() {
            "start" | "medium" => {
                let medium = phase.current == "medium";
                let uglitch = if medium { t * GLITCH_RATE } else { 0.0 };
                post_process.set_uniform("uGlitch", UniformValue::Float(uglitch));
                if t > PHASE_DURATION {
                    phase.next = Some(if medium { "fadeout" } else { "medium" }.into());
                }
            }
            "fadeout" => {
                let fade = t * FADE_RATE;
                post_process.set_uniform(
                    "fadeColor",
                    UniformValue::Vec4 {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                        w: fade,
                    },
                );
                if fade > FADE_THRESHOLD {
                    // switch to editor scene after fadeout
                    info!("intro_shader_system: fadeout complete, switching to editor scene");
                    next_scene.set("editor");
                }
            }
            _ => {}
        }
    }
}

pub fn intro_update(input: Res<InputState>, mut next_scene: ResMut<NextScene>) {
    // if the user presses any action button, switch to the editor scene
    if [
        InputAction::Action1,
        InputAction::Action2,
        InputAction::Action3,
        InputAction::Back,
    ]
    .into_iter()
    .any(|action| input.action(action).active)
    {
        info!("intro_update: action button pressed, switching to editor scene");
        next_scene.set("editor");
    }
}

pub fn intro_exit(
    _: On<SceneExited>,
    mut commands: Commands,
    mut post_process: ResMut<PostProcessShader>,
) {
    info!("intro_exit: clearing post-process shader chain");
    post_process.set_shader_chain(None);
    commands.trigger(RemoveTextureRequested {
        key: "aberred_engine_isometric_alpha".into(),
    });
}

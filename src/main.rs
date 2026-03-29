mod scenes;

use aberredengine::bevy_ecs::prelude::{Commands, ResMut};
use aberredengine::engine_app::EngineBuilder;
use aberredengine::raylib::prelude::Color;
use aberredengine::resources::gameconfig::GameConfig;
use aberredengine::resources::gamestate::{GameStates, NextGameState};
use aberredengine::resources::texturestore::TextureStore;
use aberredengine::systems::RaylibAccess;
use aberredengine::systems::scene_dispatch::{GuiCallback, SceneDescriptor};
use log::info;

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    EngineBuilder::new()
        .config("config.ini")
        // .title("Map Editor")
        .on_setup(load_assets)
        .add_scene(
            "intro",
            SceneDescriptor {
                on_enter: scenes::intro::intro_enter,
                on_update: Some(scenes::intro::intro_update),
                on_exit: None,
                gui_callback: None,
            },
        )
        .add_scene(
            "editor",
            SceneDescriptor {
                on_enter: scenes::editor::editor_enter,
                on_update: Some(scenes::editor::editor_update),
                on_exit: None,
                gui_callback: Some(scenes::editor::editor_gui as GuiCallback),
            },
        )
        .initial_scene("intro")
        .run();
}

fn load_assets(
    mut commands: Commands,
    mut config: ResMut<GameConfig>,
    mut raylib: RaylibAccess,
    mut next_state: ResMut<NextGameState>,
) {
    info!("load_assets: loading editor assets");
    config.background_color = Color::BLACK;

    let (rl, th) = (&mut *raylib.rl, &*raylib.th);

    let mut texture_store = TextureStore::new();
    let texture = rl
        .load_texture(th, "./assets/textures/aberred_engine_isometric_alpha.png")
        .expect("Failed to load texture");
    texture_store.insert("aberred_engine_isometric_alpha", texture);
    commands.insert_resource(texture_store);

    next_state.set(GameStates::Playing);
}

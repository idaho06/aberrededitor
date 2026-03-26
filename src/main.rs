mod scenes;

use aberredengine::engine_app::EngineBuilder;
use aberredengine::resources::gamestate::{GameStates, NextGameState};
use aberredengine::systems::GameCtx;
use aberredengine::systems::scene_dispatch::{GuiCallback, SceneDescriptor};
use aberredengine::bevy_ecs::prelude::ResMut;
use log::info;

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    EngineBuilder::new()
        .config("config.ini")
        // .title("Map Editor")
        .on_setup(load_assets)
        .add_scene(
            "editor",
            SceneDescriptor {
                on_enter: scenes::editor::editor_enter,
                on_update: Some(scenes::editor::editor_update),
                on_exit: None,
                gui_callback: Some(scenes::editor::editor_gui as GuiCallback),
            },
        )
        .initial_scene("editor")
        .run();
}

fn load_assets(_ctx: GameCtx, mut next_state: ResMut<NextGameState>) {
    info!("load_assets: loading editor assets");
    next_state.set(GameStates::Playing);
}

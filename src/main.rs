mod scenes;
mod systems;

use aberredengine::engine_app::EngineBuilder;
use aberredengine::systems::scene_dispatch::{GuiCallback, SceneDescriptor};

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    EngineBuilder::new()
        .config("config.ini")
        // .title("Map Editor")
        .on_setup(systems::load_assets::load_assets)
        .add_observer(systems::tilemap_load::tilemap_load_observer)
        .add_scene(
            "intro",
            SceneDescriptor {
                on_enter: scenes::intro::intro_enter,
                on_update: Some(scenes::intro::intro_update),
                on_exit: Some(scenes::intro::intro_exit),
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

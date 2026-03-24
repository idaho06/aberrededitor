use aberredengine::engine_app::EngineBuilder;
use aberredengine::resources::gamestate::{GameStates, NextGameState};
use aberredengine::resources::input::InputState;
use aberredengine::resources::worldsignals::WorldSignals;
use aberredengine::systems::GameCtx;
use aberredengine::systems::scene_dispatch::{GuiCallback, SceneDescriptor};
use bevy_ecs::prelude::ResMut;
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
                on_enter: editor_enter,
                on_update: Some(editor_update),
                on_exit: None,
                gui_callback: Some(editor_gui as GuiCallback),
            },
        )
        .initial_scene("editor")
        .run();
}

fn load_assets(_ctx: GameCtx, mut next_state: ResMut<NextGameState>) {
    info!("load_assets: loading editor assets");
    next_state.set(GameStates::Playing);
}

fn editor_enter(_ctx: &mut GameCtx) {
    info!("editor_enter: entering editor scene");
}

fn editor_update(ctx: &mut GameCtx, _dt: f32, _input: &InputState) {
    if ctx.world_signals.has_flag("gui:action:file:save") {
        ctx.world_signals.clear_flag("gui:action:file:save");
        info!("editor_update: save requested");
        // handle save
    }
}

fn editor_gui(ui: &imgui::Ui, signals: &mut WorldSignals) {
    if let Some(_mb) = ui.begin_main_menu_bar()
        && let Some(_file) = ui.begin_menu("File")
        && ui.menu_item("Save")
    {
        signals.set_flag("gui:action:file:save");
    }
}

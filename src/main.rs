//! aberrededitor — 2D map editor built on aberredengine (Bevy ECS + Raylib).
//!
//! Entry point: [`main`] wires all observers, per-frame systems, and scenes (enter/exit observers,
//! scene systems, GUI and world-draw callbacks) via
//! [`aberredengine::engine_app::EngineBuilder`]. See `docs/architecture.md` for the
//! two-layer ECS/GUI model, and `docs/patterns.md` for the recurring design patterns.
mod components;
mod editor_types;
mod scenes;
mod signals;
mod systems;

use aberredengine::prelude::EngineBuilder;

fn main() -> Result<(), aberredengine::EngineError> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let b = EngineBuilder::new()
        .config_str(include_str!("../config.ini"))
        .on_setup(systems::load_assets::load_assets)
        .add_observer(systems::debug_mirror::debug_mode_mirror_observer)
        .add_observer(systems::entity_inspector::entity_inspect_observer);

    let b = systems::map_ops::register(b);
    let b = systems::entity_selector::register(b);
    let b = systems::entity_edit::register(b);
    let b = systems::render_prefs::register(b);
    let b = b
        .add_observer(systems::window_resize::on_window_resized)
        .add_system(systems::animation_store_sync::animation_store_sync_system)
        .add_system(systems::camera_sync::editor_camera_sync_system)
        .add_system(systems::editor_camera::editor_camera_system)
        .add_system(systems::file_dialogs::poll_async_dialogs)
        .add_system(systems::group_selector::update_group_cache)
        .add_system(systems::quit::quit_handler);

    systems::tilemap_load::register(b)
        .add_system(systems::template_selector::update_template_cache)
        .add_system(scenes::editor::entity_editor_selection_change_system)
        .add_scene("intro")
        .on_scene_enter("intro", scenes::intro::intro_enter)
        .add_scene_system("intro", scenes::intro::intro_update)
        .add_scene_system("intro", scenes::intro::intro_shader_system)
        .on_scene_exit("intro", scenes::intro::intro_exit)
        .add_scene("editor")
        .on_scene_enter("editor", scenes::editor::editor_enter)
        .add_scene_system("editor", scenes::editor::editor_update)
        .on_scene_exit("editor", scenes::editor::editor_exit)
        .add_scene_gui("editor", scenes::editor::editor_gui)
        .add_scene_world_draw("editor", scenes::editor::draw_world_overlays)
        .initial_scene("intro")
        .try_run()
}

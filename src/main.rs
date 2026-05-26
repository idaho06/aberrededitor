//! aberrededitor — 2D map editor built on aberredengine (Bevy ECS + Raylib).
//!
//! Entry point: [`main`] wires all observers, per-frame systems, and scene descriptors via
//! [`aberredengine::engine_app::EngineBuilder`]. See `docs/architecture.md` for the
//! two-layer ECS/GUI model, and `docs/patterns.md` for the recurring design patterns.
mod components;
mod editor_types;
mod scenes;
mod signals;
mod systems;

use aberredengine::engine_app::EngineBuilder;
use aberredengine::systems::scene_dispatch::{GuiCallback, SceneDescriptor, WorldDrawCallback};

fn main() -> Result<(), String> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let b = EngineBuilder::new()
        .config_str(include_str!("../config.ini"))
        .on_setup(systems::load_assets::load_assets)
        .add_observer(systems::debug_mirror::debug_mode_mirror_observer)
        .add_observer(systems::entity_inspector::entity_inspect_observer);

    let b = systems::map_ops::register(b);
    let b = systems::entity_selector::register(b);
    let b = systems::entity_edit::register(b);
    let b = b
        .configure_schedule(systems::window_resize::configure_resize_schedule)
        .add_system(systems::animation_store_sync::animation_store_sync_system)
        .add_system(systems::camera_sync::editor_camera_sync_system)
        .add_system(systems::editor_camera::editor_camera_system)
        .add_system(systems::file_dialogs::poll_async_dialogs)
        .add_system(systems::group_selector::update_group_cache);

    systems::tilemap_load::register(b)
        .add_system(systems::template_selector::update_template_cache)
        .add_system(scenes::editor::entity_editor_selection_change_system)
        .add_scene(
            "intro",
            SceneDescriptor {
                on_enter: scenes::intro::intro_enter,
                on_update: Some(scenes::intro::intro_update),
                on_exit: Some(scenes::intro::intro_exit),
                gui_callback: None,
                world_draw_callback: None,
            },
        )
        .add_scene(
            "editor",
            SceneDescriptor {
                on_enter: scenes::editor::editor_enter,
                on_update: Some(scenes::editor::editor_update),
                on_exit: Some(scenes::editor::editor_exit),
                gui_callback: Some(scenes::editor::editor_gui as GuiCallback),
                world_draw_callback: Some(scenes::editor::draw_world_overlays as WorldDrawCallback),
            },
        )
        .initial_scene("intro")
        .try_run()
}

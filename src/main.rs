mod components;
mod editor_types;
mod scenes;
mod signals;
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
        .add_observer(systems::map_ops::new_map_observer)
        .add_observer(systems::map_ops::load_map_observer)
        .add_observer(systems::map_ops::save_map_observer)
        .add_observer(systems::map_ops::add_texture_observer)
        .add_observer(systems::map_ops::rename_texture_key_observer)
        .add_observer(systems::map_ops::remove_texture_observer)
        .add_observer(systems::map_ops::preview_mapdata_observer)
        .add_observer(systems::debug_mirror::debug_mode_mirror_observer)
        .add_observer(systems::entity_edit::update_map_position_observer)
        .add_observer(systems::entity_edit::update_z_index_observer)
        .add_observer(systems::entity_edit::update_group_observer)
        .add_observer(systems::entity_edit::update_rotation_observer)
        .add_observer(systems::entity_edit::update_scale_observer)
        .add_observer(systems::entity_edit::update_sprite_observer)
        .add_observer(systems::entity_edit::update_box_collider_observer)
        .add_observer(systems::entity_edit::update_animation_observer)
        .add_observer(systems::entity_edit::remove_map_position_observer)
        .add_observer(systems::entity_edit::remove_z_index_observer)
        .add_observer(systems::entity_edit::remove_group_observer)
        .add_observer(systems::entity_edit::remove_sprite_observer)
        .add_observer(systems::entity_edit::remove_box_collider_observer)
        .add_observer(systems::entity_edit::remove_rotation_observer)
        .add_observer(systems::entity_edit::remove_scale_observer)
        .add_observer(systems::entity_edit::remove_animation_observer)
        .add_observer(systems::entity_edit::remove_ttl_observer)
        .add_observer(systems::entity_edit::remove_timer_observer)
        .add_observer(systems::entity_edit::remove_phase_observer)
        .add_observer(systems::entity_edit::remove_persistent_observer)
        .add_observer(systems::entity_edit::remove_tilemap_observer)
        .add_observer(systems::entity_edit::bake_tilemap_observer)
        .add_observer(systems::entity_edit::add_component_observer)
        .add_observer(systems::entity_selector::entity_pick_observer)
        .add_observer(systems::entity_selector::select_entity_observer)
        .add_observer(systems::entity_selector::select_group_observer)
        .add_observer(systems::entity_inspector::entity_inspect_observer)
        .configure_schedule(systems::window_resize::configure_resize_schedule)
        .add_system(systems::camera_sync::editor_camera_sync_system)
        .add_system(systems::editor_camera::editor_camera_system)
        .add_system(systems::group_selector::update_group_cache)
        .add_system(systems::tilemap_load::on_tilemap_added)
        .add_system(systems::tilemap_load::tag_plain_map_entities)
        .add_system(systems::template_selector::update_template_cache)
        .add_system(scenes::editor::entity_editor_selection_change_system)
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
                on_exit: Some(scenes::editor::editor_exit),
                gui_callback: Some(scenes::editor::editor_gui as GuiCallback),
            },
        )
        .initial_scene("intro")
        .run();
}

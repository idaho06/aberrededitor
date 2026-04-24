use aberredengine::bevy_ecs;
use aberredengine::bevy_ecs::prelude::{Added, Commands, Event, On, Query, ResMut};
use aberredengine::components::group::Group;
use aberredengine::components::tilemap::TileMap;
use aberredengine::resources::mapdata::{MapData, TilemapEntry};
use aberredengine::resources::texturestore::TextureStore;
use log::info;

use crate::systems::utils::to_relative;

#[derive(Event)]
pub struct LoadTilemapRequested {
    pub path: String,
}

pub fn tilemap_load_observer(
    trigger: On<LoadTilemapRequested>,
    mut commands: Commands,
    mut map_data: ResMut<MapData>,
) {
    let dir_path = &trigger.event().path;
    let id = std::path::Path::new(dir_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("tilemap")
        .to_string();

    commands.spawn((TileMap::new(dir_path), Group::new("tilemap-roots")));

    if !map_data.tilemaps.iter().any(|e| e.key == id) {
        map_data.tilemaps.push(TilemapEntry {
            key: id.clone(),
            path: to_relative(dir_path),
        });
    }

    info!(
        "tilemap_load_observer: queued tilemap '{}' from '{}'",
        id, dir_path
    );
}

/// Per-frame system that detects freshly-spawned TileMap entities and
/// records their texture paths in TextureStore.paths for editor display
/// and map persistence. The engine's tilemap_spawn_system loads the texture
/// itself but does not populate paths — that is an editor-side concern.
pub fn track_tilemap_texture_path(
    query: Query<&TileMap, Added<TileMap>>,
    mut texture_store: ResMut<TextureStore>,
) {
    for tilemap in query.iter() {
        let stem = tilemap.path.split('/').next_back().unwrap_or(&tilemap.path);
        let tex_path = format!("{}/{}.png", tilemap.path, stem);
        texture_store
            .paths
            .insert(stem.to_owned(), to_relative(&tex_path));
    }
}

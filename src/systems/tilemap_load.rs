use aberredengine::bevy_ecs;
use aberredengine::bevy_ecs::prelude::{Added, Commands, Event, On, Query, ResMut};
use aberredengine::components::group::Group;
use aberredengine::components::tilemap::TileMap;
use aberredengine::resources::mapdata::{EntityDef, MapData};
use aberredengine::resources::texturestore::TextureStore;
use log::info;

use crate::systems::map_ops::GROUP_TILEMAP_ROOTS;
use crate::systems::utils::{tilemap_stem, to_relative};

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
    let rel = to_relative(dir_path);
    let id = tilemap_stem(&rel).to_owned();

    commands.spawn((TileMap::new(dir_path), Group::new(GROUP_TILEMAP_ROOTS)));

    if !map_data.entities.iter().any(|e| e.tilemap_path.as_deref() == Some(rel.as_str())) {
        map_data.entities.push(EntityDef {
            group: Some(GROUP_TILEMAP_ROOTS.to_string()),
            tilemap_path: Some(rel),
            ..Default::default()
        });
    }

    info!(
        "tilemap_load_observer: queued tilemap '{}' from '{}'",
        id, dir_path
    );
}

/// The engine's tilemap_spawn_system loads the texture itself but does not
/// populate TextureStore.paths — that is an editor-side concern.
pub fn track_tilemap_texture_path(
    query: Query<&TileMap, Added<TileMap>>,
    mut texture_store: ResMut<TextureStore>,
) {
    for tilemap in query.iter() {
        let stem = tilemap_stem(&tilemap.path);
        let tex_path = format!("{}/{}.png", tilemap.path, stem);
        texture_store
            .paths
            .insert(stem.to_owned(), to_relative(&tex_path));
    }
}

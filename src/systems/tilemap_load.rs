use aberredengine::bevy_ecs;
use aberredengine::bevy_ecs::hierarchy::ChildOf;
use aberredengine::bevy_ecs::prelude::{Added, Commands, Entity, Event, On, Query, ResMut, Without};
use aberredengine::components::group::Group;
use aberredengine::components::mapposition::MapPosition;
use aberredengine::components::tilemap::TileMap;
use aberredengine::resources::mapdata::{EntityDef, MapData};
use aberredengine::resources::texturestore::TextureStore;
use log::{info, warn};

use crate::components::map_entity::MapEntity;

use crate::systems::map_ops::GROUP_TILEMAP_ROOTS;
use crate::systems::utils::{tilemap_stem, tilemap_tex_path, to_relative};

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
    if dir_path.is_empty() {
        warn!("tilemap_load_observer: empty path, ignoring");
        return;
    }
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

/// Tags plain entities (no TileMap, no ChildOf) that just gained MapPosition —
/// these are baked tile entities being re-spawned from a saved map file.
/// The engine's spawn_entity has no way to insert MapEntity directly.
type PlainMapPositionQuery<'w, 's> =
    Query<'w, 's, Entity, (Added<MapPosition>, Without<TileMap>, Without<ChildOf>)>;

pub fn tag_plain_map_entities(
    query: PlainMapPositionQuery,
    mut commands: Commands,
) {
    for entity in query.iter() {
        commands.entity(entity).insert(MapEntity);
    }
}

/// Runs on Added<TileMap> — covers both the UI-trigger path and the engine's
/// load-from-file spawn path. TextureStore.paths is an editor concern; the
/// engine's tilemap_spawn_system does not populate it.
pub fn on_tilemap_added(
    query: Query<(Entity, &TileMap), Added<TileMap>>,
    mut commands: Commands,
    mut texture_store: ResMut<TextureStore>,
) {
    for (entity, tilemap) in query.iter() {
        commands.entity(entity).insert(MapEntity);
        let stem = tilemap_stem(&tilemap.path);
        texture_store
            .paths
            .insert(stem.to_owned(), tilemap_tex_path(&tilemap.path, stem));
    }
}

use aberredengine::bevy_ecs;
use aberredengine::bevy_ecs::prelude::{Commands, Entity, Event, On, Query, Res, ResMut};
use aberredengine::components::group::Group;
use aberredengine::events::spawnmap::SpawnMapRequested;
use aberredengine::resources::mapdata::{load_map, save_map, MapData};
use aberredengine::resources::tilemapstore::TilemapStore;
use log::{info, warn};

#[derive(Event)]
pub struct NewMapRequested;

#[derive(Event)]
pub struct LoadMapRequested {
    pub path: String,
}

#[derive(Event)]
pub struct SaveMapRequested {
    pub path: String,
}

pub fn new_map_observer(
    _trigger: On<NewMapRequested>,
    mut commands: Commands,
    groups: Query<(Entity, &Group)>,
    mut tilemap_store: ResMut<TilemapStore>,
) {
    clear_map_entities(&mut commands, &groups);
    tilemap_store.clear();
    commands.insert_resource(MapData::default());
    info!("new_map_observer: cleared map");
}

pub fn load_map_observer(
    trigger: On<LoadMapRequested>,
    mut commands: Commands,
    groups: Query<(Entity, &Group)>,
    mut tilemap_store: ResMut<TilemapStore>,
) {
    let path = &trigger.event().path;
    let map = match load_map(path) {
        Ok(m) => m,
        Err(e) => {
            warn!("load_map_observer: failed to load '{}': {}", path, e);
            return;
        }
    };
    clear_map_entities(&mut commands, &groups);
    tilemap_store.clear();
    commands.insert_resource(map.clone());
    commands.trigger(SpawnMapRequested { map });
    info!("load_map_observer: loaded map from '{}'", path);
}

pub fn save_map_observer(
    trigger: On<SaveMapRequested>,
    map_data: Res<MapData>,
) {
    let path = &trigger.event().path;
    if let Err(e) = save_map(path, &map_data) {
        warn!("save_map_observer: failed to save '{}': {}", path, e);
    } else {
        info!("save_map_observer: saved map to '{}'", path);
    }
}

fn clear_map_entities(commands: &mut Commands, groups: &Query<(Entity, &Group)>) {
    for (entity, group) in groups.iter() {
        if group.name() == "tiles" || group.name() == "tiles-templates" {
            commands.entity(entity).despawn();
        }
    }
}

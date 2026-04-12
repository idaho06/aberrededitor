use aberredengine::bevy_ecs;
use aberredengine::bevy_ecs::prelude::{Commands, Entity, Event, On, Query, Res, ResMut};
use aberredengine::components::group::Group;
use aberredengine::events::spawnmap::SpawnMapRequested;
use aberredengine::resources::mapdata::{load_map, save_map, MapData, TextureEntry};
use aberredengine::resources::texturestore::TextureStore;
use aberredengine::resources::tilemapstore::TilemapStore;
use aberredengine::resources::worldsignals::WorldSignals;
use aberredengine::systems::RaylibAccess;
use log::{info, warn};

use crate::systems::entity_selector::{clear_selector_state, EntitySelectorCache};

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
    mut world_signals: ResMut<WorldSignals>,
    mut selector_cache: ResMut<EntitySelectorCache>,
) {
    clear_map_entities(&mut commands, &groups);
    tilemap_store.clear();
    commands.insert_resource(MapData::default());
    clear_selector_state(&mut world_signals, &mut selector_cache);
    info!("new_map_observer: cleared map");
}

pub fn load_map_observer(
    trigger: On<LoadMapRequested>,
    mut commands: Commands,
    groups: Query<(Entity, &Group)>,
    mut tilemap_store: ResMut<TilemapStore>,
    mut world_signals: ResMut<WorldSignals>,
    mut selector_cache: ResMut<EntitySelectorCache>,
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
    clear_selector_state(&mut world_signals, &mut selector_cache);
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

// ---------------------------------------------------------------------------
// Texture store events
// ---------------------------------------------------------------------------

#[derive(Event)]
pub struct AddTextureRequested {
    pub key: String,
    pub path: String,
}

#[derive(Event)]
pub struct RenameTextureKeyRequested {
    pub old_key: String,
    pub new_key: String,
}

#[derive(Event)]
pub struct RemoveTextureRequested {
    pub key: String,
}

use crate::systems::utils::to_relative;

pub fn add_texture_observer(
    trigger: On<AddTextureRequested>,
    mut raylib: RaylibAccess,
    mut texture_store: ResMut<TextureStore>,
    mut map_data: ResMut<MapData>,
) {
    let key = trigger.event().key.clone();
    let path = trigger.event().path.clone();
    let (rl, th) = (&mut *raylib.rl, &*raylib.th);
    match rl.load_texture(th, &path) {
        Ok(texture) => {
            let rel_path = to_relative(&path);
            info!("add_texture_observer: added '{}' from '{}'", key, rel_path);
            texture_store.insert(&key, texture);
            texture_store.paths.insert(key.clone(), rel_path.clone());
            if !map_data.textures.iter().any(|e| e.key == key) {
                map_data.textures.push(TextureEntry { key: key.clone(), path: rel_path });
            }
        }
        Err(e) => {
            warn!("add_texture_observer: failed to load '{}': {}", path, e);
        }
    }
}

pub fn rename_texture_key_observer(
    trigger: On<RenameTextureKeyRequested>,
    mut texture_store: ResMut<TextureStore>,
    mut map_data: ResMut<MapData>,
) {
    let old_key = &trigger.event().old_key;
    let new_key = &trigger.event().new_key;
    if old_key == new_key {
        return;
    }
    if texture_store.map.contains_key(new_key.as_str()) {
        warn!(
            "rename_texture_key_observer: key '{}' already exists, skipping",
            new_key
        );
        return;
    }
    if let Some(texture) = texture_store.remove(old_key.as_str()) {
        texture_store.insert(new_key, texture);
        if let Some(p) = texture_store.paths.remove(old_key.as_str()) {
            texture_store.paths.insert(new_key.clone(), p);
        }
    } else {
        warn!(
            "rename_texture_key_observer: key '{}' not found in TextureStore",
            old_key
        );
    }
    for entry in map_data.textures.iter_mut() {
        if entry.key == *old_key {
            entry.key = new_key.clone();
            break;
        }
    }
    info!(
        "rename_texture_key_observer: renamed '{}' -> '{}'",
        old_key, new_key
    );
}

pub fn remove_texture_observer(
    trigger: On<RemoveTextureRequested>,
    mut texture_store: ResMut<TextureStore>,
    mut map_data: ResMut<MapData>,
) {
    let key = &trigger.event().key;
    texture_store.remove(key.as_str());
    texture_store.paths.remove(key.as_str());
    map_data.textures.retain(|e| e.key != *key);
    info!("remove_texture_observer: removed '{}'", key);
}

// ---------------------------------------------------------------------------
// Map data preview
// ---------------------------------------------------------------------------

#[derive(Event)]
pub struct PreviewMapDataRequested;

pub fn preview_mapdata_observer(
    _trigger: On<PreviewMapDataRequested>,
    map_data: Res<MapData>,
    mut world_signals: ResMut<WorldSignals>,
) {
    match serde_json::to_string_pretty(&*map_data) {
        Ok(json) => {
            world_signals.set_string("gui:mapdata_preview_json", json.as_str());
            world_signals.set_flag("gui:view:preview_mapdata_open");
        }
        Err(e) => {
            warn!("preview_mapdata_observer: serialization failed: {}", e);
        }
    }
}

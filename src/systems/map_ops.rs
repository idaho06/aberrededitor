use crate::signals as sig;
use aberredengine::bevy_ecs;
use aberredengine::bevy_ecs::prelude::{Commands, Entity, Event, On, Query, ResMut};
use aberredengine::components::group::Group;
use aberredengine::components::mapposition::MapPosition;
use aberredengine::components::rotation::Rotation;
use aberredengine::components::scale::Scale;
use aberredengine::components::tilemap::TileMap;
use aberredengine::components::zindex::ZIndex;
use aberredengine::events::spawnmap::SpawnMapRequested;
use aberredengine::resources::appstate::AppState;
use aberredengine::resources::mapdata::{MapData, TextureEntry, load_map, save_map};
use aberredengine::resources::texturestore::TextureStore;
use aberredengine::resources::worldsignals::WorldSignals;
use aberredengine::systems::RaylibAccess;
use log::{info, warn};

use crate::systems::entity_selector::clear_selector_state;
use crate::systems::utils::to_relative;

const GROUP_TILES: &str = "tiles";
const GROUP_TILES_TEMPLATES: &str = "tiles-templates";
pub const GROUP_TILEMAP_ROOTS: &str = "tilemap-roots";

// ---------------------------------------------------------------------------
// Map lifecycle events
// ---------------------------------------------------------------------------

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
    mut world_signals: ResMut<WorldSignals>,
    mut app_state: ResMut<AppState>,
) {
    reset_editor_map(
        &mut commands,
        &groups,
        &mut world_signals,
        &mut app_state,
        MapData::default(),
    );
    info!("new_map_observer: cleared map");
}

pub fn load_map_observer(
    trigger: On<LoadMapRequested>,
    mut commands: Commands,
    groups: Query<(Entity, &Group)>,
    mut world_signals: ResMut<WorldSignals>,
    mut app_state: ResMut<AppState>,
) {
    let path = &trigger.event().path;
    let map = match load_map(path) {
        Ok(m) => m,
        Err(e) => {
            warn!("load_map_observer: failed to load '{}': {}", path, e);
            return;
        }
    };
    reset_editor_map(
        &mut commands,
        &groups,
        &mut world_signals,
        &mut app_state,
        map.clone(),
    );
    commands.trigger(SpawnMapRequested { map });
    info!("load_map_observer: loaded map from '{}'", path);
}

type TilemapRootsQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static TileMap,
        Option<&'static MapPosition>,
        Option<&'static ZIndex>,
        Option<&'static Group>,
        Option<&'static Rotation>,
        Option<&'static Scale>,
    ),
>;

fn sync_tilemap_entities(map_data: &mut MapData, tilemap_roots: &TilemapRootsQuery) {
    for (tilemap, pos, z, group, rot, scale) in tilemap_roots.iter() {
        let path = to_relative(&tilemap.path);
        if let Some(def) = map_data
            .entities
            .iter_mut()
            .find(|e| e.tilemap_path.as_deref() == Some(path.as_str()))
        {
            def.position = pos.map(|p| [p.pos.x, p.pos.y]);
            def.z_index = z.map(|z| z.0);
            def.group = group.map(|g| g.0.clone());
            def.rotation_deg = rot.map(|r| r.degrees);
            def.scale = scale.map(|s| [s.scale.x, s.scale.y]);
        }
    }
}

pub fn save_map_observer(
    trigger: On<SaveMapRequested>,
    mut map_data: ResMut<MapData>,
    tilemap_roots: TilemapRootsQuery,
) {
    sync_tilemap_entities(&mut map_data, &tilemap_roots);

    let path = &trigger.event().path;
    if let Err(e) = save_map(path, &*map_data) {
        warn!("save_map_observer: failed to save '{}': {}", path, e);
    } else {
        info!("save_map_observer: saved map to '{}'", path);
    }
}

/// Clears tile entities, resets tilemap store, inserts fresh map data, and
/// clears entity selector state. Called by both new-map and load-map paths.
fn reset_editor_map(
    commands: &mut Commands,
    groups: &Query<(Entity, &Group)>,
    world_signals: &mut WorldSignals,
    app_state: &mut AppState,
    map_data: MapData,
) {
    clear_map_entities(commands, groups);
    commands.insert_resource(map_data);
    clear_selector_state(world_signals, app_state);
}

fn clear_map_entities(commands: &mut Commands, groups: &Query<(Entity, &Group)>) {
    for (entity, group) in groups.iter() {
        if group.name() == GROUP_TILES
            || group.name() == GROUP_TILES_TEMPLATES
            || group.name() == GROUP_TILEMAP_ROOTS
        {
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

pub fn add_texture_observer(
    trigger: On<AddTextureRequested>,
    mut raylib: RaylibAccess,
    mut texture_store: ResMut<TextureStore>,
    mut map_data: ResMut<MapData>,
) {
    let key = &trigger.event().key;
    let path = &trigger.event().path;
    let (rl, th) = (&mut *raylib.rl, &*raylib.th);
    match rl.load_texture(th, path) {
        Ok(texture) => {
            let rel_path = to_relative(path);
            info!("add_texture_observer: added '{}' from '{}'", key, rel_path);
            texture_store.insert(key, texture);
            texture_store.paths.insert(key.clone(), rel_path.clone());
            if !map_data.textures.iter().any(|e| e.key == *key) {
                map_data.textures.push(TextureEntry {
                    key: key.clone(),
                    path: rel_path,
                });
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
    mut map_data: ResMut<MapData>,
    tilemap_roots: TilemapRootsQuery,
    mut world_signals: ResMut<WorldSignals>,
) {
    sync_tilemap_entities(&mut map_data, &tilemap_roots);
    match serde_json::to_string_pretty(&*map_data) {
        Ok(json) => {
            world_signals.set_string(sig::MAPDATA_PREVIEW_JSON, json.as_str());
            world_signals.set_flag(sig::UI_PREVIEW_MAPDATA_OPEN);
        }
        Err(e) => {
            warn!("preview_mapdata_observer: serialization failed: {}", e);
        }
    }
}

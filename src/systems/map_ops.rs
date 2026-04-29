use crate::signals as sig;
use aberredengine::bevy_ecs;
use aberredengine::bevy_ecs::prelude::{Commands, Entity, Event, NonSendMut, On, Query, Res, ResMut, With};
use aberredengine::components::group::Group;
use aberredengine::components::mapposition::MapPosition;
use aberredengine::components::rotation::Rotation;
use aberredengine::components::scale::Scale;
use aberredengine::components::sprite::Sprite;
use aberredengine::components::tilemap::TileMap;
use aberredengine::components::tint::Tint;
use aberredengine::components::zindex::ZIndex;
use aberredengine::events::spawnmap::SpawnMapRequested;
use aberredengine::resources::appstate::AppState;
use aberredengine::resources::fontstore::FontStore;
use aberredengine::resources::mapdata::{EntityDef, FontEntry, MapData, TextureEntry, load_map, save_map};
use aberredengine::systems::mapspawn::load_font_with_mipmaps;
use aberredengine::resources::texturestore::TextureStore;
use aberredengine::resources::worldsignals::WorldSignals;
use aberredengine::systems::RaylibAccess;
use log::{info, warn};

use crate::components::map_entity::MapEntity;
use crate::systems::entity_selector::clear_selector_state;
use crate::systems::utils::{sprite_to_entry, to_relative};

pub const GROUP_TILES: &str = "tiles";
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
    map_entities: Query<Entity, With<MapEntity>>,
    mut world_signals: ResMut<WorldSignals>,
    mut app_state: ResMut<AppState>,
    mut texture_store: ResMut<TextureStore>,
    mut font_store: NonSendMut<FontStore>,
) {
    texture_store.map.clear();
    texture_store.paths.clear();
    font_store.clear();
    reset_editor_map(
        &mut commands,
        &map_entities,
        &mut world_signals,
        &mut app_state,
        MapData::default(),
    );
    info!("new_map_observer: cleared map");
}

pub fn load_map_observer(
    trigger: On<LoadMapRequested>,
    mut commands: Commands,
    map_entities: Query<Entity, With<MapEntity>>,
    mut world_signals: ResMut<WorldSignals>,
    mut app_state: ResMut<AppState>,
    mut texture_store: ResMut<TextureStore>,
    mut font_store: NonSendMut<FontStore>,
) {
    let path = &trigger.event().path;
    let map = match load_map(path) {
        Ok(m) => m,
        Err(e) => {
            warn!("load_map_observer: failed to load '{}': {}", path, e);
            return;
        }
    };
    texture_store.map.clear();
    texture_store.paths.clear();
    font_store.clear();
    reset_editor_map(
        &mut commands,
        &map_entities,
        &mut world_signals,
        &mut app_state,
        map.clone(),
    );
    for tex in &map.textures {
        commands.trigger(AddTextureRequested {
            key: tex.key.clone(),
            path: tex.path.clone(),
        });
    }
    for font in &map.fonts {
        commands.trigger(AddFontRequested {
            key: font.key.clone(),
            path: font.path.clone(),
            font_size: font.font_size,
        });
    }
    commands.trigger(SpawnMapRequested { map });
    info!("load_map_observer: loaded map from '{}'", path);
}

type MapEntitiesQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        Option<&'static TileMap>,
        Option<&'static MapPosition>,
        Option<&'static ZIndex>,
        Option<&'static Group>,
        Option<&'static Rotation>,
        Option<&'static Scale>,
        Option<&'static Sprite>,
        Option<&'static Tint>,
    ),
    With<MapEntity>,
>;

fn sync_map_entities(map_data: &mut MapData, entities: &MapEntitiesQuery, world_signals: &WorldSignals) {
    // Plain-entity defs are rebuilt from ECS state on every sync; only tilemap defs
    // are kept between syncs (matched by path).
    map_data.entities.retain(|e| e.tilemap_path.is_some());

    // Build reverse lookup once: entity → user-registered key (filter internal keys).
    let user_keys: Vec<(Entity, &str)> = world_signals
        .entities
        .iter()
        .filter(|(k, _)| sig::is_user_entity_key(k))
        .map(|(k, e)| (*e, k.as_str()))
        .collect();

    for (entity, tilemap, pos, z, group, rot, scale, sprite, tint) in entities.iter() {
        let registered_as = user_keys
            .iter()
            .find(|(e, _)| *e == entity)
            .map(|(_, k)| k.to_string());

        let tint_arr = tint.map(|t| [t.color.r, t.color.g, t.color.b, t.color.a]);

        if let Some(tilemap) = tilemap {
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
                def.registered_as = registered_as;
                def.tint = tint_arr;
            }
        } else {
            map_data.entities.push(EntityDef {
                position: pos.map(|p| [p.pos.x, p.pos.y]),
                z_index: z.map(|z| z.0),
                group: group.map(|g| g.0.clone()),
                rotation_deg: rot.map(|r| r.degrees),
                scale: scale.map(|s| [s.scale.x, s.scale.y]),
                sprite: sprite.map(sprite_to_entry),
                tilemap_path: None,
                registered_as,
                tint: tint_arr,
            });
        }
    }
}

pub fn save_map_observer(
    trigger: On<SaveMapRequested>,
    mut map_data: ResMut<MapData>,
    map_entities: MapEntitiesQuery,
    world_signals: Res<WorldSignals>,
) {
    sync_map_entities(&mut map_data, &map_entities, &world_signals);

    let path = &trigger.event().path;
    if let Err(e) = save_map(path, &map_data) {
        warn!("save_map_observer: failed to save '{}': {}", path, e);
    } else {
        info!("save_map_observer: saved map to '{}'", path);
    }
}

/// Clears tile entities, resets tilemap store, inserts fresh map data, and
/// clears entity selector state. Called by both new-map and load-map paths.
fn reset_editor_map(
    commands: &mut Commands,
    map_entities: &Query<Entity, With<MapEntity>>,
    world_signals: &mut WorldSignals,
    app_state: &mut AppState,
    map_data: MapData,
) {
    clear_map_entities(commands, map_entities);
    // Drop all user entity registrations; internal editor keys are retained.
    world_signals
        .entities
        .retain(|k, _| !sig::is_user_entity_key(k));
    commands.insert_resource(map_data);
    clear_selector_state(world_signals, app_state);
}

fn clear_map_entities(commands: &mut Commands, map_entities: &Query<Entity, With<MapEntity>>) {
    for entity in map_entities.iter() {
        commands.entity(entity).despawn();
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
    if texture_store.map.contains_key(key.as_str()) {
        return;
    }
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
// Font store events
// ---------------------------------------------------------------------------

#[derive(Event)]
pub struct AddFontRequested {
    pub key: String,
    pub path: String,
    pub font_size: f32,
}

#[derive(Event)]
pub struct RenameFontKeyRequested {
    pub old_key: String,
    pub new_key: String,
}

#[derive(Event)]
pub struct RemoveFontRequested {
    pub key: String,
}

pub fn add_font_observer(
    trigger: On<AddFontRequested>,
    mut raylib: RaylibAccess,
    mut font_store: NonSendMut<FontStore>,
    mut map_data: ResMut<MapData>,
) {
    let key = &trigger.event().key;
    let path = &trigger.event().path;
    let font_size = trigger.event().font_size;
    if font_store.meta.contains_key(key.as_str()) {
        return;
    }
    let (rl, th) = (&mut *raylib.rl, &*raylib.th);
    let font = load_font_with_mipmaps(rl, th, path, font_size as i32);
    info!("add_font_observer: added '{}' from '{}'", key, path);
    font_store.add_with_meta(key, font, path.clone(), font_size);
    if !map_data.fonts.iter().any(|e| e.key == *key) {
        map_data.fonts.push(FontEntry {
            key: key.clone(),
            path: path.clone(),
            font_size,
        });
    }
}

pub fn rename_font_key_observer(
    trigger: On<RenameFontKeyRequested>,
    mut font_store: NonSendMut<FontStore>,
    mut map_data: ResMut<MapData>,
) {
    let old_key = &trigger.event().old_key;
    let new_key = &trigger.event().new_key;
    if old_key == new_key {
        return;
    }
    if font_store.meta.contains_key(new_key.as_str()) {
        warn!("rename_font_key_observer: key '{}' already exists, skipping", new_key);
        return;
    }
    font_store.rename(old_key.as_str(), new_key.clone());
    if let Some(entry) = map_data.fonts.iter_mut().find(|e| e.key == *old_key) {
        entry.key = new_key.clone();
    }
    info!("rename_font_key_observer: '{}' -> '{}'", old_key, new_key);
}

pub fn remove_font_observer(
    trigger: On<RemoveFontRequested>,
    mut font_store: NonSendMut<FontStore>,
    mut map_data: ResMut<MapData>,
) {
    let key = &trigger.event().key;
    font_store.remove(key.as_str());
    map_data.fonts.retain(|e| e.key != *key);
    info!("remove_font_observer: removed '{}'", key);
}

// ---------------------------------------------------------------------------
// Map data preview
// ---------------------------------------------------------------------------

#[derive(Event)]
pub struct PreviewMapDataRequested;

pub fn preview_mapdata_observer(
    _trigger: On<PreviewMapDataRequested>,
    mut map_data: ResMut<MapData>,
    map_entities: MapEntitiesQuery,
    mut world_signals: ResMut<WorldSignals>,
) {
    sync_map_entities(&mut map_data, &map_entities, &world_signals);
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

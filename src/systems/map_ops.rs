//! Map lifecycle and asset store CRUD observers.
//!
//! **Map lifecycle:**
//! - [`NewMapRequested`] / [`new_map_observer`] — clears all stores and despawns all `MapEntity` entities.
//! - [`LoadMapRequested`] / [`load_map_observer`] — deserialises a `.map` JSON file and
//!   spawns entities + populates stores. Uses [`aberredengine::events::spawnmap::SpawnMapRequested`]
//!   for the actual entity instantiation.
//! - [`SaveMapRequested`] / [`save_map_observer`] — serialises all `MapEntity` components to
//!   JSON and writes the file.
//!
//! **Asset store CRUD (Texture / Font / Animation):**
//! Each store has three or four event/observer pairs: Add, Rename, Remove (and Update for
//! animations). All observers mutate the corresponding engine resource directly; the GUI reads
//! the updated state via the per-frame sync systems.
//!
//! **Group constants:**
//! - [`GROUP_TILES`] — group name for individual tile entities inside a tilemap.
//! - [`GROUP_TILEMAP_ROOTS`] — group name for tilemap root entities.
use crate::components::serialized_lua_setup::SerializedLuaSetup;
use crate::signals as sig;
use aberredengine::bevy_ecs;
use aberredengine::bevy_ecs::prelude::{
    Commands, Entity, Event, NonSendMut, On, Query, Res, ResMut, With,
};
use aberredengine::components::animation::Animation;
use aberredengine::components::group::Group;
use aberredengine::components::mapposition::MapPosition;
use aberredengine::components::rotation::Rotation;
use aberredengine::components::scale::Scale;
use aberredengine::components::sprite::Sprite;
use aberredengine::components::tilemap::TileMap;
use aberredengine::components::tint::Tint;
use aberredengine::components::zindex::ZIndex;
use aberredengine::components::dynamictext::DynamicText;
use aberredengine::events::spawnmap::SpawnMapRequested;
use aberredengine::raylib::prelude::Vector2;
use aberredengine::resources::animationstore::{AnimationResource, AnimationStore};
use aberredengine::resources::appstate::AppState;
use aberredengine::resources::fontstore::FontStore;
use aberredengine::resources::mapdata::{
    AnimationEntry, DynamicTextEntry, EntityDef, FontEntry, MapData, TextureEntry, load_map,
    save_map,
};
use aberredengine::resources::texturestore::TextureStore;
use std::sync::Arc;
use aberredengine::resources::worldsignals::WorldSignals;
use aberredengine::systems::RaylibAccess;
use aberredengine::systems::mapspawn::load_font_with_mipmaps;
use log::{info, warn};

use crate::components::map_entity::MapEntity;
use crate::systems::entity_selector::clear_selector_state;
use crate::systems::tilemap_load::PendingLuaSetupLoadMutex;
use crate::systems::utils::{sprite_to_entry, to_relative};

/// Group name assigned to individual tile entities spawned by a tilemap.
pub const GROUP_TILES: &str = "tiles";
/// Group name assigned to the root entity of a loaded tilemap.
pub const GROUP_TILEMAP_ROOTS: &str = "tilemap-roots";

// ---------------------------------------------------------------------------
// Map lifecycle events
// ---------------------------------------------------------------------------

/// Clear all stores and despawn all `MapEntity` entities, resetting to an empty map.
#[derive(Event)]
pub struct NewMapRequested;

/// Load a `.map` JSON file from `path` and populate ECS + asset stores.
#[derive(Event)]
pub struct LoadMapRequested {
    pub path: String,
}

/// Serialise all `MapEntity` components to JSON and write to `path`.
#[derive(Event)]
pub struct SaveMapRequested {
    pub path: String,
}

#[allow(clippy::too_many_arguments)]
pub fn new_map_observer(
    _trigger: On<NewMapRequested>,
    mut commands: Commands,
    map_entities: Query<Entity, With<MapEntity>>,
    mut world_signals: ResMut<WorldSignals>,
    mut app_state: ResMut<AppState>,
    mut texture_store: ResMut<TextureStore>,
    mut font_store: NonSendMut<FontStore>,
    mut anim_store: ResMut<AnimationStore>,
) {
    texture_store.map.clear();
    texture_store.paths.clear();
    font_store.clear();
    anim_store.animations.clear();
    reset_editor_map(
        &mut commands,
        &map_entities,
        &mut world_signals,
        &mut app_state,
        MapData::default(),
    );
    info!("new_map_observer: cleared map");
}

#[allow(clippy::too_many_arguments)]
pub fn load_map_observer(
    trigger: On<LoadMapRequested>,
    mut commands: Commands,
    map_entities: Query<Entity, With<MapEntity>>,
    mut world_signals: ResMut<WorldSignals>,
    mut app_state: ResMut<AppState>,
    mut texture_store: ResMut<TextureStore>,
    mut font_store: NonSendMut<FontStore>,
    mut anim_store: ResMut<AnimationStore>,
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
    anim_store.animations.clear();
    reset_editor_map(
        &mut commands,
        &map_entities,
        &mut world_signals,
        &mut app_state,
        map.clone(),
    );
    if let Some(mutex) = app_state.get::<PendingLuaSetupLoadMutex>() {
        mutex.lock().unwrap().reset_from_map(&map);
    }
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
        Option<&'static SerializedLuaSetup>,
        Option<&'static DynamicText>,
        Option<&'static Animation>,
    ),
    With<MapEntity>,
>;

fn sync_map_entities(
    map_data: &mut MapData,
    entities: &MapEntitiesQuery,
    world_signals: &WorldSignals,
) {
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

    for (entity, tilemap, pos, z, group, rot, scale, sprite, tint, lua_setup, dynamic_text, animation) in
        entities.iter()
    {
        let registered_as = user_keys
            .iter()
            .find(|(e, _)| *e == entity)
            .map(|(_, k)| k.to_string());

        let tint_arr = tint.map(|t| [t.color.r, t.color.g, t.color.b, t.color.a]);
        let animation_key = animation.map(|a| a.animation_key.clone());
        let lua_setup_callback = lua_setup.map(|l| l.callback.clone());
        let dynamic_text_entry = dynamic_text.map(|d| DynamicTextEntry {
            text: d.text.to_string(),
            font_key: d.font.to_string(),
            font_size: d.font_size,
            color: [d.color.r, d.color.g, d.color.b, d.color.a],
        });

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
                def.lua_setup = lua_setup_callback;
                def.dynamic_text = dynamic_text_entry.clone();
            }
        } else {
            map_data.entities.push(EntityDef {
                position: pos.map(|p| [p.pos.x, p.pos.y]),
                z_index: z.map(|z| z.0),
                group: group.map(|g| g.0.clone()),
                rotation_deg: rot.map(|r| r.degrees),
                scale: scale.map(|s| [s.scale.x, s.scale.y]),
                sprite: sprite.map(sprite_to_entry),
                registered_as,
                tint: tint_arr,
                animation_key,
                lua_setup: lua_setup_callback,
                dynamic_text: dynamic_text_entry,
                ..Default::default()
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
    if let Some(mutex) = app_state.get::<PendingLuaSetupLoadMutex>() {
        mutex.lock().unwrap().clear();
    }
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

/// Load the texture at `path` (relative to CWD) into `TextureStore` under `key`.
/// No-ops if `key` already exists. Also records the entry in `MapData.textures`.
#[derive(Event)]
pub struct AddTextureRequested {
    pub key: String,
    /// Relative-to-CWD path (converted via `to_relative` before triggering).
    pub path: String,
}

/// Rename a texture key in `TextureStore`, `TextureStore.paths`, and `MapData.textures`.
#[derive(Event)]
pub struct RenameTextureKeyRequested {
    pub old_key: String,
    pub new_key: String,
}

/// Remove a texture from `TextureStore` and `MapData.textures`.
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

/// Load the font at `path` with `font_size` into `FontStore` under `key`.
/// No-ops if `key` already exists. `FontStore` is `NonSend`; this observer requests `NonSendMut`.
#[derive(Event)]
pub struct AddFontRequested {
    pub key: String,
    /// Relative-to-CWD path.
    pub path: String,
    pub font_size: f32,
}

/// Rename a font key in `FontStore` and `MapData.fonts`.
#[derive(Event)]
pub struct RenameFontKeyRequested {
    pub old_key: String,
    pub new_key: String,
}

/// Remove a font from `FontStore` and `MapData.fonts`.
#[derive(Event)]
pub struct RemoveFontRequested {
    pub key: String,
}

// ---------------------------------------------------------------------------
// Animation store CRUD events
// ---------------------------------------------------------------------------

/// Add a default-valued [`AnimationResource`] under `key` to `AnimationStore`.
#[derive(Event)]
pub struct AddAnimationRequested {
    pub key: String,
}

/// Overwrite the [`AnimationResource`] stored under `key` in `AnimationStore`.
#[derive(Event)]
pub struct UpdateAnimationResourceRequested {
    pub key: String,
    pub resource: AnimationResource,
}

/// Rename an animation key in `AnimationStore` and `MapData.animations`.
#[derive(Event)]
pub struct RenameAnimationKeyRequested {
    pub old_key: String,
    pub new_key: String,
}

/// Remove an animation from `AnimationStore` and `MapData.animations`.
#[derive(Event)]
pub struct RemoveAnimationRequested {
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
    let font = match load_font_with_mipmaps(rl, th, path, font_size as i32) {
        Ok(f) => f,
        Err(e) => {
            warn!("add_font_observer: failed to load '{}': {}", path, e);
            return;
        }
    };
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
        warn!(
            "rename_font_key_observer: key '{}' already exists, skipping",
            new_key
        );
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

// ---------------------------------------------------------------------------
// Animation store CRUD observers
// ---------------------------------------------------------------------------

fn resource_to_entry(key: &str, res: &AnimationResource) -> AnimationEntry {
    AnimationEntry {
        key: key.to_owned(),
        texture_key: res.tex_key.as_ref().to_owned(),
        position: [res.position.x, res.position.y],
        horizontal_displacement: res.horizontal_displacement,
        vertical_displacement: res.vertical_displacement,
        frame_count: res.frame_count as u32,
        fps: res.fps,
        looping: res.looped,
    }
}

pub fn add_animation_observer(
    trigger: On<AddAnimationRequested>,
    mut anim_store: ResMut<AnimationStore>,
    mut map_data: ResMut<MapData>,
) {
    let key = &trigger.event().key;
    if anim_store.animations.contains_key(key.as_str()) {
        return;
    }
    let resource = AnimationResource {
        tex_key: Arc::from(""),
        position: Vector2 { x: 0.0, y: 0.0 },
        horizontal_displacement: 16.0,
        vertical_displacement: 0.0,
        frame_count: 1,
        fps: 12.0,
        looped: true,
    };
    map_data.animations.push(resource_to_entry(key, &resource));
    anim_store.insert(key.clone(), resource);
    info!("add_animation_observer: added '{}'", key);
}

pub fn update_animation_resource_observer(
    trigger: On<UpdateAnimationResourceRequested>,
    mut anim_store: ResMut<AnimationStore>,
    mut map_data: ResMut<MapData>,
) {
    let key = &trigger.event().key;
    let res = &trigger.event().resource;
    anim_store.insert(key.clone(), res.clone());
    if let Some(entry) = map_data.animations.iter_mut().find(|e| e.key == *key) {
        *entry = resource_to_entry(key, res);
    }
}

pub fn rename_animation_key_observer(
    trigger: On<RenameAnimationKeyRequested>,
    mut anim_store: ResMut<AnimationStore>,
    mut map_data: ResMut<MapData>,
) {
    let old_key = &trigger.event().old_key;
    let new_key = &trigger.event().new_key;
    if old_key == new_key || anim_store.animations.contains_key(new_key.as_str()) {
        return;
    }
    if let Some(resource) = anim_store.animations.remove(old_key.as_str()) {
        anim_store.insert(new_key.clone(), resource);
    }
    if let Some(entry) = map_data.animations.iter_mut().find(|e| e.key == *old_key) {
        entry.key = new_key.clone();
    }
    info!(
        "rename_animation_key_observer: '{}' -> '{}'",
        old_key, new_key
    );
}

pub fn remove_animation_observer(
    trigger: On<RemoveAnimationRequested>,
    mut anim_store: ResMut<AnimationStore>,
    mut map_data: ResMut<MapData>,
) {
    let key = &trigger.event().key;
    anim_store.animations.remove(key.as_str());
    map_data.animations.retain(|e| e.key != *key);
    info!("remove_animation_observer: removed '{}'", key);
}

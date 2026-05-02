//! Tilemap loading: spawns a `TileMap` entity from a folder path and tags children.
//!
//! `tilemap_load_observer` handles `LoadTilemapRequested`. It must run in an ECS observer
//! (not the GUI callback) because it needs `RaylibAccess` to load the tilemap texture.
//!
//! `tag_plain_map_entities` and `on_tilemap_added` are per-frame systems that run after
//! the tilemap is spawned to insert `MapEntity`, `SerializedLuaSetup`, and `TextureStore`
//! entries on the newly created entities.
//!
//! `PendingLuaSetupLoadState` tracks which entities still need their `SerializedLuaSetup`
//! component populated from the map file's `lua_setup` fields.
use crate::components::serialized_lua_setup::SerializedLuaSetup;
use aberredengine::bevy_ecs;
use aberredengine::bevy_ecs::hierarchy::ChildOf;
use aberredengine::bevy_ecs::prelude::{
    Added, Commands, Entity, Event, On, Query, Res, ResMut, Without,
};
use aberredengine::components::cameratarget::CameraTarget;
use aberredengine::components::group::Group;
use aberredengine::components::mapposition::MapPosition;
use aberredengine::components::tilemap::TileMap;
use aberredengine::resources::appstate::AppState;
use aberredengine::resources::mapdata::{EntityDef, MapData};
use aberredengine::resources::texturestore::TextureStore;
use log::{info, warn};
use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;

use crate::components::map_entity::MapEntity;

use crate::systems::map_ops::GROUP_TILEMAP_ROOTS;
use crate::systems::utils::{tilemap_stem, tilemap_tex_path, to_relative};

#[derive(Default)]
pub struct PendingLuaSetupLoadState {
    plain_callbacks: VecDeque<Option<String>>,
    tilemap_callbacks: HashMap<String, String>,
}

impl PendingLuaSetupLoadState {
    pub fn reset_from_map(&mut self, map: &MapData) {
        self.clear();
        self.plain_callbacks = map
            .entities
            .iter()
            .filter(|def| def.tilemap_path.is_none())
            .map(|def| def.lua_setup.clone())
            .collect();
        self.tilemap_callbacks = map
            .entities
            .iter()
            .filter_map(|def| {
                def.tilemap_path.as_ref().and_then(|path| {
                    def.lua_setup
                        .clone()
                        .map(|callback| (path.clone(), callback))
                })
            })
            .collect();
    }

    pub fn take_plain_callback(&mut self) -> Option<String> {
        self.plain_callbacks.pop_front().flatten()
    }

    pub fn take_tilemap_callback(&mut self, path: &str) -> Option<String> {
        self.tilemap_callbacks.remove(path)
    }

    pub fn clear(&mut self) {
        self.plain_callbacks.clear();
        self.tilemap_callbacks.clear();
    }
}

pub type PendingLuaSetupLoadMutex = Mutex<PendingLuaSetupLoadState>;

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

    commands.spawn((TileMap::new(rel.as_str()), Group::new(GROUP_TILEMAP_ROOTS)));

    if !map_data
        .entities
        .iter()
        .any(|e| e.tilemap_path.as_deref() == Some(rel.as_str()))
    {
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
type PlainMapPositionQuery<'w, 's> = Query<
    'w,
    's,
    Entity,
    (
        Added<MapPosition>,
        Without<TileMap>,
        Without<ChildOf>,
        Without<CameraTarget>,
    ),
>;

pub fn tag_plain_map_entities(
    query: PlainMapPositionQuery,
    mut commands: Commands,
    app_state: Res<AppState>,
) {
    let pending = app_state.get::<PendingLuaSetupLoadMutex>();
    for entity in query.iter() {
        let mut entity_commands = commands.entity(entity);
        entity_commands.insert(MapEntity);
        if let Some(mutex) = pending
            && let Some(callback) = mutex.lock().unwrap().take_plain_callback()
        {
            entity_commands.insert(SerializedLuaSetup::new(callback));
        }
    }
}

/// Runs on `Added<TileMap>` — covers both the UI-trigger path and the engine's
/// load-from-file spawn path. `TextureStore.paths` is an editor concern; the
/// engine's tilemap_spawn_system does not populate it.
pub fn on_tilemap_added(
    query: Query<(Entity, &TileMap), Added<TileMap>>,
    mut commands: Commands,
    mut texture_store: ResMut<TextureStore>,
    app_state: Res<AppState>,
) {
    let pending = app_state.get::<PendingLuaSetupLoadMutex>();
    for (entity, tilemap) in query.iter() {
        let rel_path = to_relative(&tilemap.path);
        let stem = tilemap_stem(&rel_path);
        let mut entity_commands = commands.entity(entity);
        entity_commands.insert(MapEntity);
        if let Some(mutex) = pending
            && let Some(callback) = mutex
                .lock()
                .unwrap()
                .take_tilemap_callback(rel_path.as_str())
        {
            entity_commands.insert(SerializedLuaSetup::new(callback));
        }
        texture_store
            .paths
            .insert(stem.to_owned(), tilemap_tex_path(&rel_path, stem));
    }
}

use aberredengine::bevy_ecs;
use aberredengine::bevy_ecs::prelude::{Commands, Event, On, ResMut};
use aberredengine::resources::mapdata::{MapData, TextureEntry, TilemapEntry};
use aberredengine::resources::texturestore::TextureStore;
use aberredengine::resources::tilemapstore::TilemapStore;
use aberredengine::systems::tilemap::{load_tilemap, spawn_tiles};
use aberredengine::systems::RaylibAccess;
use log::info;

#[derive(Event)]
pub struct LoadTilemapRequested {
    pub path: String,
}

pub fn tilemap_load_observer(
    trigger: On<LoadTilemapRequested>,
    mut commands: Commands,
    mut raylib: RaylibAccess,
    mut texture_store: ResMut<TextureStore>,
    mut tilemap_store: ResMut<TilemapStore>,
    mut map_data: ResMut<MapData>,
) {
    let dir_path = &trigger.event().path;
    let id = std::path::Path::new(dir_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("tilemap")
        .to_string();

    let (rl, th) = (&mut *raylib.rl, &*raylib.th);
    let (texture, tilemap) = load_tilemap(rl, th, dir_path);

    let tex_width = texture.width;
    let tex_height = texture.height;
    texture_store.insert(&id, texture);
    spawn_tiles(&mut commands, &id, tex_width, tex_height, &tilemap);
    tilemap_store.insert(&id, tilemap);

    if !map_data.textures.iter().any(|e| e.key == id) {
        map_data.textures.push(TextureEntry {
            key: id.clone(),
            path: format!("{}/{}.png", dir_path, id),
        });
    }
    if !map_data.tilemaps.iter().any(|e| e.key == id) {
        map_data.tilemaps.push(TilemapEntry {
            key: id.clone(),
            path: dir_path.clone(),
        });
    }

    info!("tilemap_load_observer: loaded tilemap '{}' from '{}'", id, dir_path);
}

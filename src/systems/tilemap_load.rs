use std::sync::Arc;

use aberredengine::bevy_ecs;
use aberredengine::bevy_ecs::prelude::{Commands, Event, On, ResMut};
use aberredengine::components::group::Group;
use aberredengine::components::mapposition::MapPosition;
use aberredengine::components::sprite::Sprite;
use aberredengine::components::zindex::ZIndex;
use aberredengine::raylib::prelude::Vector2;
use aberredengine::resources::texturestore::TextureStore;
use aberredengine::resources::tilemapstore::{Tilemap, TilemapStore};
use aberredengine::systems::RaylibAccess;
use log::{error, info};

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
) {
    let json_path = &trigger.event().path;

    let path = std::path::Path::new(&json_path);
    let id = path
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "tilemap".to_string());
    let png_path = path.with_extension("png");
    let png_path_str = png_path.display().to_string();

    // Parse JSON
    let json_string = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            error!("tilemap_load_observer: failed to read '{}': {}", json_path, e);
            return;
        }
    };
    let tilemap: Tilemap = match serde_json::from_str(&json_string) {
        Ok(t) => t,
        Err(e) => {
            error!("tilemap_load_observer: failed to parse tilemap JSON: {}", e);
            return;
        }
    };

    // Load tileset texture (must be a sibling .png with the same stem)
    let (rl, th) = (&mut *raylib.rl, &*raylib.th);
    let texture = match rl.load_texture(th, &png_path_str) {
        Ok(t) => t,
        Err(_) => {
            error!(
                "tilemap_load_observer: tileset texture not found at '{}' - skipping load",
                png_path_str
            );
            return;
        }
    };

    let tex_width = texture.width;
    texture_store.insert(&id, texture);
    tilemap_store.insert(&id, tilemap);

    // Spawn tile entities
    let tilemap = tilemap_store.get(&id).expect("just inserted");
    let tex_key: Arc<str> = Arc::from(id.as_str());
    let tile_size = tilemap.tile_size as f32;
    let tiles_per_row = ((tex_width as f32 / tile_size).floor() as u32).max(1);
    let layer_count = tilemap.layers.len() as f32;

    for (layer_index, layer) in tilemap.layers.iter().enumerate() {
        let z = -(layer_count - layer_index as f32);
        for pos in &layer.positions {
            let col = pos.id % tiles_per_row;
            let row = pos.id / tiles_per_row;
            commands.spawn((
                Group::new("tiles"),
                MapPosition::new(pos.x as f32 * tile_size, pos.y as f32 * tile_size),
                ZIndex(z),
                Sprite {
                    tex_key: tex_key.clone(),
                    width: tile_size,
                    height: tile_size,
                    offset: Vector2 {
                        x: col as f32 * tile_size,
                        y: row as f32 * tile_size,
                    },
                    origin: Vector2 { x: 0.0, y: 0.0 },
                    flip_h: false,
                    flip_v: false,
                },
            ));
        }
    }

    info!(
        "tilemap_load_observer: loaded tilemap '{}' from '{}'",
        id, json_path
    );
}

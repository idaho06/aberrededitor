use super::{BakeTilemapRequested, RemoveTileMapRequested};
use crate::systems::entity_selector::clear_selector_state;
use crate::systems::map_ops::GROUP_TILES;
use crate::systems::utils::{sprite_to_entry, tilemap_stem, tilemap_tex_path};
use aberredengine::bevy_ecs::hierarchy::{ChildOf, Children};
use aberredengine::bevy_ecs::prelude::{Commands, On, Query, ResMut};
use aberredengine::components::globaltransform2d::GlobalTransform2D;
use aberredengine::components::group::Group;
use aberredengine::components::mapposition::MapPosition;
use aberredengine::components::rotation::Rotation;
use aberredengine::components::scale::Scale;
use aberredengine::components::sprite::Sprite;
use aberredengine::components::tilemap::TileMap;
use aberredengine::components::zindex::ZIndex;
use aberredengine::resources::appstate::AppState;
use aberredengine::resources::mapdata::{EntityDef, MapData, TextureEntry};
use aberredengine::resources::texturestore::TextureStore;
use aberredengine::resources::worldsignals::WorldSignals;
use crate::components::map_entity::MapEntity;
use log::{debug, info, warn};

pub fn remove_tilemap_observer(
    trigger: On<RemoveTileMapRequested>,
    mut commands: Commands,
    tilemap_query: Query<&TileMap>,
    mut map_data: ResMut<MapData>,
    mut texture_store: ResMut<TextureStore>,
    mut world_signals: ResMut<WorldSignals>,
    mut app_state: ResMut<AppState>,
) {
    let entity = trigger.event().entity;

    if let Ok(tilemap) = tilemap_query.get(entity) {
        let tilemap_path = crate::systems::utils::to_relative(&tilemap.path);
        let stem = tilemap_stem(&tilemap.path).to_owned();
        map_data
            .entities
            .retain(|e| e.tilemap_path.as_deref() != Some(tilemap_path.as_str()));
        texture_store.paths.remove(&stem);
        debug!(
            "remove_tilemap_observer: removed tilemap '{}' (entity {})",
            stem,
            entity.to_bits()
        );
    }

    super::remove_entity_registrations(&mut world_signals, entity);
    commands.entity(entity).despawn();
    clear_selector_state(&mut world_signals, &mut app_state);
}

type TileChildQuery<'w, 's> = Query<
    'w,
    's,
    (
        Option<&'static Group>,
        Option<&'static Sprite>,
        Option<&'static ZIndex>,
        Option<&'static GlobalTransform2D>,
    ),
>;

pub fn bake_tilemap_observer(
    trigger: On<BakeTilemapRequested>,
    mut commands: Commands,
    root_query: Query<(&TileMap, Option<&Children>)>,
    child_query: TileChildQuery,
    mut map_data: ResMut<MapData>,
    mut world_signals: ResMut<WorldSignals>,
    mut app_state: ResMut<AppState>,
) {
    let root = trigger.event().entity;
    let Ok((tilemap, maybe_children)) = root_query.get(root) else {
        warn!("bake_tilemap_observer: root entity has no TileMap");
        return;
    };
    let tilemap_path = crate::systems::utils::to_relative(&tilemap.path);
    let stem = tilemap_stem(&tilemap_path);

    if let Some(children) = maybe_children {
        for &child in children.iter() {
            let Ok((group, sprite, zidx, gt)) = child_query.get(child) else {
                continue;
            };

            let is_tiles_group = group.map(|g| g.name() == GROUP_TILES).unwrap_or(false);
            if !is_tiles_group {
                super::remove_entity_registrations(&mut world_signals, child);
                commands.entity(child).despawn();
                continue;
            }

            let Some(gt) = gt else {
                warn!("bake_tilemap_observer: tile child missing GlobalTransform2D, skipping");
                continue;
            };

            map_data.entities.push(EntityDef {
                position: Some([gt.position.x, gt.position.y]),
                z_index: zidx.map(|z| z.0),
                group: group.map(|g| g.0.clone()),
                rotation_deg: Some(gt.rotation_degrees),
                scale: Some([gt.scale.x, gt.scale.y]),
                sprite: sprite.map(sprite_to_entry),
                ..Default::default()
            });

            commands
                .entity(child)
                .insert(MapEntity)
                .insert(MapPosition::new(gt.position.x, gt.position.y))
                .insert(Rotation {
                    degrees: gt.rotation_degrees,
                })
                .insert(Scale::new(gt.scale.x, gt.scale.y))
                .remove::<ChildOf>();
        }
    }

    map_data
        .entities
        .retain(|e| e.tilemap_path.as_deref() != Some(&tilemap_path));

    // Register the tilemap's texture so it's saved with the map and reloaded next time.
    if !map_data.textures.iter().any(|e| e.key == stem) {
        map_data.textures.push(TextureEntry {
            key: stem.to_string(),
            path: tilemap_tex_path(&tilemap_path, stem),
            filter: None,
        });
    }

    super::remove_entity_registrations(&mut world_signals, root);
    commands.entity(root).despawn();
    clear_selector_state(&mut world_signals, &mut app_state);
    info!("bake_tilemap_observer: baked tilemap '{}'", stem);
}

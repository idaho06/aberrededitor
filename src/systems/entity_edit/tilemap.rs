use super::{BakeTilemapRequested, RemoveTileMapRequested};
use crate::components::map_entity::MapEntity;
use crate::systems::entity_selector::clear_selector_state;
use crate::systems::map_ops::GROUP_TILES;
use crate::systems::utils::{find_texture, sprite_to_entry, tilemap_stem, tilemap_tex_path};
use aberredengine::bevy_ecs::hierarchy::{ChildOf, Children};
use aberredengine::bevy_ecs::prelude::{Commands, MessageWriter, On, Query, ResMut};
use aberredengine::core::components::globaltransform2d::GlobalTransform2D;
use aberredengine::core::components::group::Group;
use aberredengine::core::components::mapposition::MapPosition;
use aberredengine::core::components::rotation::Rotation;
use aberredengine::core::components::scale::Scale;
use aberredengine::core::components::sprite::Sprite;
use aberredengine::core::components::tilemap::TileMap;
use aberredengine::core::components::zindex::ZIndex;
use aberredengine::core::protocol::render_assets::RenderAssetCmd;
use aberredengine::core::resources::appstate::AppState;
use aberredengine::core::resources::mapdata::{EntityDef, MapData, TextureEntry};
use aberredengine::core::resources::worldsignals::WorldSignals;
use aberredengine::core::systems::tilemap::tilemap_texture_key;
use log::{debug, info, warn};

pub fn remove_tilemap_observer(
    trigger: On<RemoveTileMapRequested>,
    mut commands: Commands,
    tilemap_query: Query<&TileMap>,
    mut map_data: ResMut<MapData>,
    mut asset_cmds: MessageWriter<RenderAssetCmd>,
    mut world_signals: ResMut<WorldSignals>,
    mut app_state: ResMut<AppState>,
) {
    let entity = trigger.event().entity;

    if let Ok(tilemap) = tilemap_query.get(entity) {
        let tilemap_path = crate::systems::utils::to_relative(&tilemap.path);
        let atlas_key = tilemap_texture_key(&tilemap.path);
        map_data
            .entities
            .retain(|e| e.tilemap_path.as_deref() != Some(tilemap_path.as_str()));
        asset_cmds.write(RenderAssetCmd::RemoveTexture {
            key: atlas_key.clone(),
        });
        debug!(
            "remove_tilemap_observer: removed tilemap '{}' (entity {})",
            atlas_key,
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
    // Baked tiles keep the engine's atlas key in their sprites, so the atlas is saved
    // under that key too.
    let atlas_key = tilemap_texture_key(&tilemap.path);

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
    if find_texture(&map_data, &atlas_key).is_none() {
        map_data.textures.push(TextureEntry {
            key: atlas_key,
            path: tilemap_tex_path(&tilemap_path, stem),
            filter: None,
        });
    }

    super::remove_entity_registrations(&mut world_signals, root);
    commands.entity(root).despawn();
    clear_selector_state(&mut world_signals, &mut app_state);
    info!("bake_tilemap_observer: baked tilemap '{}'", stem);
}

#[cfg(test)]
mod tests {
    use super::*;
    use aberredengine::bevy_ecs::message::Messages;
    use aberredengine::bevy_ecs::prelude::Entity;
    use aberredengine::bevy_ecs::world::World;

    const TILEMAP_DIR: &str = "assets/tilemaps/forest";

    fn world_with_tilemap() -> (World, Entity) {
        let mut world = World::new();
        world.insert_resource(Messages::<RenderAssetCmd>::default());
        world.insert_resource(MapData::default());
        world.insert_resource(WorldSignals::default());
        world.insert_resource(AppState::default());
        let root = world.spawn(TileMap::new(TILEMAP_DIR)).id();
        // A tile as the engine's `tilemap_spawn_system` spawns it: its sprite uses the atlas key.
        world.spawn((
            ChildOf(root),
            Group::new(GROUP_TILES),
            Sprite::new(tilemap_texture_key(TILEMAP_DIR), 16.0, 16.0),
            ZIndex(0.0),
            GlobalTransform2D::default(),
        ));
        (world, root)
    }

    #[test]
    fn remove_tilemap_unloads_the_engine_atlas_key() {
        let (mut world, root) = world_with_tilemap();
        world.add_observer(remove_tilemap_observer);
        world.trigger(RemoveTileMapRequested { entity: root });
        world.flush();

        let removed: Vec<_> = world
            .resource::<Messages<RenderAssetCmd>>()
            .iter_current_update_messages()
            .filter_map(|cmd| match cmd {
                RenderAssetCmd::RemoveTexture { key } => Some(key.clone()),
                _ => None,
            })
            .collect();
        assert_eq!(removed, vec![tilemap_texture_key(TILEMAP_DIR)]);
    }

    #[test]
    fn baked_tiles_reference_a_registered_texture() {
        let (mut world, root) = world_with_tilemap();
        world.add_observer(bake_tilemap_observer);
        world.trigger(BakeTilemapRequested { entity: root });
        world.flush();

        let map_data = world.resource::<MapData>();
        let baked: Vec<_> = map_data
            .entities
            .iter()
            .filter_map(|e| e.sprite.as_ref())
            .collect();
        assert_eq!(baked.len(), 1, "expected one baked tile");
        for sprite in baked {
            assert!(
                find_texture(map_data, &sprite.texture_key).is_some(),
                "baked tile uses texture '{}', but MapData registers only {:?}",
                sprite.texture_key,
                map_data.textures.iter().map(|t| &t.key).collect::<Vec<_>>()
            );
        }
    }
}

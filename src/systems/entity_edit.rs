use super::entity_inspector::InspectEntityRequested;
use crate::editor_types::ComponentKind;
use crate::systems::entity_selector::clear_selector_state;
use crate::systems::utils::{tilemap_stem, tilemap_tex_path, sprite_to_entry};
use aberredengine::bevy_ecs;
use aberredengine::bevy_ecs::hierarchy::{ChildOf, Children};
use aberredengine::bevy_ecs::prelude::{Commands, Entity, Event, On, Query, Res, ResMut};
use aberredengine::resources::appstate::AppState;
use aberredengine::components::animation::Animation;
use aberredengine::components::boxcollider::BoxCollider;
use aberredengine::components::globaltransform2d::GlobalTransform2D;
use aberredengine::components::group::Group;
use aberredengine::components::mapposition::MapPosition;
use aberredengine::components::persistent::Persistent;
use aberredengine::components::phase::Phase;
use aberredengine::components::rotation::Rotation;
use aberredengine::components::scale::Scale;
use aberredengine::components::sprite::Sprite;
use aberredengine::components::tilemap::TileMap;
use aberredengine::components::timer::Timer;
use aberredengine::components::tint::Tint;
use aberredengine::components::ttl::Ttl;
use aberredengine::components::zindex::ZIndex;
use aberredengine::raylib::prelude::{Color, Vector2};
use aberredengine::resources::mapdata::{EntityDef, MapData, TextureEntry};
use aberredengine::resources::texturestore::TextureStore;
use aberredengine::resources::worldsignals::WorldSignals;
use log::{debug, info, warn};
use std::sync::Arc;

use crate::components::map_entity::MapEntity;
use crate::systems::map_ops::GROUP_TILES;

#[derive(Event)]
pub struct UpdateMapPositionRequested {
    pub entity: Entity,
    pub x: f32,
    pub y: f32,
}

#[derive(Event)]
pub struct UpdateZIndexRequested {
    pub entity: Entity,
    pub z_index: f32,
}

#[derive(Event)]
pub struct UpdateGroupRequested {
    pub entity: Entity,
    pub group: String,
}

#[derive(Event)]
pub struct UpdateRotationRequested {
    pub entity: Entity,
    pub rotation_deg: f32,
}

#[derive(Event)]
pub struct UpdateScaleRequested {
    pub entity: Entity,
    pub x: f32,
    pub y: f32,
}

#[derive(Event)]
pub struct UpdateSpriteRequested {
    pub entity: Entity,
    pub tex_key: String,
    pub width: f32,
    pub height: f32,
    pub offset: [f32; 2],
    pub origin: [f32; 2],
    pub flip_h: bool,
    pub flip_v: bool,
}

#[derive(Event)]
pub struct UpdateBoxColliderRequested {
    pub entity: Entity,
    pub size: [f32; 2],
    pub offset: [f32; 2],
    pub origin: [f32; 2],
}

#[derive(Event)]
pub struct UpdateAnimationRequested {
    pub entity: Entity,
    pub animation_key: String,
    pub frame_index: usize,
    pub elapsed_time: f32,
}

#[derive(Event)]
pub struct RemoveMapPositionRequested  { pub entity: Entity }
#[derive(Event)]
pub struct RemoveZIndexRequested       { pub entity: Entity }
#[derive(Event)]
pub struct RemoveGroupRequested        { pub entity: Entity }
#[derive(Event)]
pub struct RemoveSpriteRequested       { pub entity: Entity }
#[derive(Event)]
pub struct RemoveBoxColliderRequested  { pub entity: Entity }
#[derive(Event)]
pub struct RemoveRotationRequested     { pub entity: Entity }
#[derive(Event)]
pub struct RemoveScaleRequested        { pub entity: Entity }
#[derive(Event)]
pub struct RemoveAnimationRequested    { pub entity: Entity }
#[derive(Event)]
pub struct RemoveTtlRequested          { pub entity: Entity }
#[derive(Event)]
pub struct RemoveTimerRequested        { pub entity: Entity }
#[derive(Event)]
pub struct RemovePhaseRequested        { pub entity: Entity }
#[derive(Event)]
pub struct RemovePersistentRequested   { pub entity: Entity }
#[derive(Event)]
pub struct RemoveTileMapRequested      { pub entity: Entity }
#[derive(Event)]
pub struct RemoveTintRequested         { pub entity: Entity }
#[derive(Event)]
pub struct RemoveEntityRequested       { pub entity: Entity }
#[derive(Event)]
pub struct BakeTilemapRequested        { pub entity: Entity }

#[derive(Event)]
pub struct UpdateTintRequested {
    pub entity: Entity,
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[derive(Event)]
pub struct RegisterEntityRequested {
    pub entity:  Entity,
    pub key:     String,
    pub old_key: Option<String>,
}

#[derive(Event)]
pub struct UnregisterEntityRequested {
    pub entity: Entity,
    pub key:    String,
}

#[derive(Event)]
pub struct AddComponentRequested {
    pub entity: Entity,
    pub kind:   ComponentKind,
}

macro_rules! component_edit_observer {
    (
        $fn_name:ident,
        $event:ty,
        $component:ty,
        $component_name:literal,
        |$component_var:ident, $event_var:ident, $entity_var:ident| $body:block
    ) => {
        pub fn $fn_name(
            trigger: On<$event>,
            mut query: Query<&mut $component>,
            mut commands: Commands,
        ) {
            let $event_var = trigger.event();
            let $entity_var = $event_var.entity;
            let Ok(mut $component_var) = query.get_mut($entity_var) else {
                warn_missing_component(stringify!($fn_name), $entity_var, $component_name);
                return;
            };
            $body
            refresh_inspector(&mut commands, $entity_var);
        }
    };
}

component_edit_observer!(
    update_map_position_observer,
    UpdateMapPositionRequested,
    MapPosition,
    "MapPosition",
    |map_position, event, entity| {
        map_position.pos = Vector2::new(event.x, event.y);
        debug!(
            "update_map_position_observer: updated entity {} -> ({:.3}, {:.3})",
            entity.to_bits(),
            event.x,
            event.y
        );
    }
);

component_edit_observer!(
    update_z_index_observer,
    UpdateZIndexRequested,
    ZIndex,
    "ZIndex",
    |z_index, event, entity| {
        z_index.0 = event.z_index;
        debug!(
            "update_z_index_observer: updated entity {} -> {:.3}",
            entity.to_bits(),
            event.z_index
        );
    }
);

component_edit_observer!(
    update_group_observer,
    UpdateGroupRequested,
    Group,
    "Group",
    |group, event, entity| {
        group.0 = event.group.clone();
        debug!(
            "update_group_observer: updated entity {} -> '{}'",
            entity.to_bits(),
            event.group
        );
    }
);

component_edit_observer!(
    update_rotation_observer,
    UpdateRotationRequested,
    Rotation,
    "Rotation",
    |rotation, event, entity| {
        rotation.degrees = event.rotation_deg;
        debug!(
            "update_rotation_observer: updated entity {} -> {:.3} deg",
            entity.to_bits(),
            event.rotation_deg
        );
    }
);

component_edit_observer!(
    update_scale_observer,
    UpdateScaleRequested,
    Scale,
    "Scale",
    |scale, event, entity| {
        scale.scale = Vector2::new(event.x, event.y);
        debug!(
            "update_scale_observer: updated entity {} -> ({:.3}, {:.3})",
            entity.to_bits(),
            event.x,
            event.y
        );
    }
);

component_edit_observer!(
    update_sprite_observer,
    UpdateSpriteRequested,
    Sprite,
    "Sprite",
    |sprite, event, entity| {
        sprite.tex_key = Arc::from(event.tex_key.as_str());
        sprite.width = event.width;
        sprite.height = event.height;
        sprite.offset = Vector2::new(event.offset[0], event.offset[1]);
        sprite.origin = Vector2::new(event.origin[0], event.origin[1]);
        sprite.flip_h = event.flip_h;
        sprite.flip_v = event.flip_v;
        debug!(
            "update_sprite_observer: updated entity {} sprite '{}'",
            entity.to_bits(),
            event.tex_key
        );
    }
);

component_edit_observer!(
    update_box_collider_observer,
    UpdateBoxColliderRequested,
    BoxCollider,
    "BoxCollider",
    |collider, event, entity| {
        collider.size = Vector2::new(event.size[0], event.size[1]);
        collider.offset = Vector2::new(event.offset[0], event.offset[1]);
        collider.origin = Vector2::new(event.origin[0], event.origin[1]);
        debug!(
            "update_box_collider_observer: updated entity {} collider",
            entity.to_bits()
        );
    }
);

component_edit_observer!(
    update_animation_observer,
    UpdateAnimationRequested,
    Animation,
    "Animation",
    |animation, event, entity| {
        animation.animation_key = event.animation_key.clone();
        animation.frame_index = event.frame_index;
        animation.elapsed_time = event.elapsed_time;
        debug!(
            "update_animation_observer: updated entity {} animation '{}' frame {} elapsed {:.3}",
            entity.to_bits(),
            event.animation_key,
            event.frame_index,
            event.elapsed_time
        );
    }
);

macro_rules! component_remove_observer {
    ($fn_name:ident, $event:ty, $component:ty, $component_name:literal) => {
        pub fn $fn_name(trigger: On<$event>, mut commands: Commands) {
            let entity = trigger.event().entity;
            commands.entity(entity).remove::<$component>();
            debug!(
                concat!(stringify!($fn_name), ": removed ", $component_name, " from entity {}"),
                entity.to_bits()
            );
            refresh_inspector(&mut commands, entity);
        }
    };
}

component_remove_observer!(remove_map_position_observer, RemoveMapPositionRequested, MapPosition, "MapPosition");
component_remove_observer!(remove_z_index_observer,      RemoveZIndexRequested,      ZIndex,      "ZIndex");
component_remove_observer!(remove_group_observer,        RemoveGroupRequested,        Group,       "Group");
component_remove_observer!(remove_sprite_observer,       RemoveSpriteRequested,       Sprite,      "Sprite");
component_remove_observer!(remove_box_collider_observer, RemoveBoxColliderRequested,  BoxCollider, "BoxCollider");
component_remove_observer!(remove_rotation_observer,     RemoveRotationRequested,     Rotation,    "Rotation");
component_remove_observer!(remove_scale_observer,        RemoveScaleRequested,        Scale,       "Scale");
component_remove_observer!(remove_animation_observer,    RemoveAnimationRequested,    Animation,   "Animation");
component_remove_observer!(remove_ttl_observer,          RemoveTtlRequested,          Ttl,         "Ttl");
component_remove_observer!(remove_timer_observer,        RemoveTimerRequested,        Timer,       "Timer");
component_remove_observer!(remove_phase_observer,        RemovePhaseRequested,        Phase,       "Phase");
component_remove_observer!(remove_persistent_observer,   RemovePersistentRequested,   Persistent,  "Persistent");
component_remove_observer!(remove_tint_observer,         RemoveTintRequested,         Tint,        "Tint");

component_edit_observer!(
    update_tint_observer,
    UpdateTintRequested,
    Tint,
    "Tint",
    |tint, event, entity| {
        tint.color = Color::new(event.r, event.g, event.b, event.a);
        debug!(
            "update_tint_observer: updated entity {} tint -> ({}, {}, {}, {})",
            entity.to_bits(),
            event.r,
            event.g,
            event.b,
            event.a
        );
    }
);

pub fn add_component_observer(
    trigger: On<AddComponentRequested>,
    textures: Res<TextureStore>,
    mut commands: Commands,
) {
    let event = trigger.event();
    let entity = event.entity;
    match event.kind {
        ComponentKind::MapPosition => {
            commands.entity(entity).insert(MapPosition::new(0.0, 0.0));
        }
        ComponentKind::ZIndex => {
            commands.entity(entity).insert(ZIndex(0.0));
        }
        ComponentKind::Group => {
            commands.entity(entity).insert(Group::new(""));
        }
        ComponentKind::Rotation => {
            commands.entity(entity).insert(Rotation::default());
        }
        ComponentKind::Scale => {
            commands.entity(entity).insert(Scale::default());
        }
        ComponentKind::Sprite => {
            let tex_key: Arc<str> = Arc::from(
                textures.map.keys().min().map(|k| k.as_str()).unwrap_or(""),
            );
            commands.entity(entity).insert(Sprite {
                tex_key,
                width: 32.0,
                height: 32.0,
                offset: Vector2::zero(),
                origin: Vector2::zero(),
                flip_h: false,
                flip_v: false,
            });
        }
        ComponentKind::BoxCollider => {
            commands.entity(entity).insert(BoxCollider::new(32.0, 32.0));
        }
        ComponentKind::Animation => {
            commands.entity(entity).insert(Animation::new(""));
        }
        ComponentKind::Ttl => {
            commands.entity(entity).insert(Ttl::new(5.0));
        }
        ComponentKind::Persistent => {
            commands.entity(entity).insert(Persistent);
        }
        ComponentKind::Tint => {
            commands.entity(entity).insert(Tint::default());
        }
    }
    debug!(
        "add_component_observer: added {:?} to entity {}",
        event.kind,
        entity.to_bits()
    );
    refresh_inspector(&mut commands, entity);
}

fn refresh_inspector(commands: &mut Commands, entity: Entity) {
    commands.trigger(InspectEntityRequested { entity });
}

fn remove_entity_registrations(world_signals: &mut WorldSignals, entity: Entity) {
    let keys_to_remove: Vec<String> = world_signals
        .entities
        .iter()
        .filter(|(_, e)| **e == entity)
        .map(|(k, _)| k.clone())
        .collect();
    for key in &keys_to_remove {
        world_signals.remove_entity(key);
    }
    if !keys_to_remove.is_empty() {
        debug!(
            "remove_entity_registrations: removed keys [{}] for entity {}",
            keys_to_remove.join(", "),
            entity.to_bits()
        );
    }
}

fn warn_missing_component(observer: &str, entity: Entity, component: &str) {
    warn!(
        "{}: entity {} missing {}",
        observer,
        entity.to_bits(),
        component
    );
}

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
        let stem = tilemap_stem(&tilemap.path).to_owned();
        map_data.entities.retain(|e| e.tilemap_path.as_deref() != Some(&tilemap.path));
        texture_store.paths.remove(&stem);
        debug!(
            "remove_tilemap_observer: removed tilemap '{}' (entity {})",
            stem,
            entity.to_bits()
        );
    }

    remove_entity_registrations(&mut world_signals, entity);
    commands.entity(entity).despawn();
    clear_selector_state(&mut world_signals, &mut app_state);
}

pub fn remove_entity_observer(
    trigger: On<RemoveEntityRequested>,
    mut commands: Commands,
    mut world_signals: ResMut<WorldSignals>,
    mut app_state: ResMut<AppState>,
) {
    let entity = trigger.event().entity;
    remove_entity_registrations(&mut world_signals, entity);
    commands.entity(entity).despawn();
    clear_selector_state(&mut world_signals, &mut app_state);
}

type TileChildQuery<'w, 's> =
    Query<'w, 's, (Option<&'static Group>, Option<&'static Sprite>, Option<&'static ZIndex>, Option<&'static GlobalTransform2D>)>;

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
    let tilemap_path = tilemap.path.clone();
    let stem = tilemap_stem(&tilemap_path);

    if let Some(children) = maybe_children {
        for &child in children.iter() {
            let Ok((group, sprite, zidx, gt)) = child_query.get(child) else {
                continue;
            };

            let is_tiles_group = group.map(|g| g.name() == GROUP_TILES).unwrap_or(false);
            if !is_tiles_group {
                remove_entity_registrations(&mut world_signals, child);
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
                tilemap_path: None,
                registered_as: None,
                tint: None,
            });

            commands
                .entity(child)
                .insert(MapEntity)
                .insert(MapPosition::new(gt.position.x, gt.position.y))
                .insert(Rotation { degrees: gt.rotation_degrees })
                .insert(Scale::new(gt.scale.x, gt.scale.y))
                .remove::<ChildOf>();
        }
    }

    map_data
        .entities
        .retain(|e| e.tilemap_path.as_deref() != Some(&tilemap_path));

    // Register the tilemap's texture so it's saved with the map and reloaded next time.
    // Tilemap textures are normally tracked only in TextureStore, not in MapData.textures.
    if !map_data.textures.iter().any(|e| e.key == stem) {
        map_data.textures.push(TextureEntry {
            key: stem.to_string(),
            path: tilemap_tex_path(&tilemap_path, stem),
        });
    }

    remove_entity_registrations(&mut world_signals, root);
    commands.entity(root).despawn();
    clear_selector_state(&mut world_signals, &mut app_state);
    info!("bake_tilemap_observer: baked tilemap '{}'", stem);
}

pub fn register_entity_observer(
    trigger: On<RegisterEntityRequested>,
    mut world_signals: ResMut<WorldSignals>,
    mut commands: Commands,
) {
    let ev = trigger.event();
    if ev.key.is_empty() {
        return;
    }
    if let Some(ref old) = ev.old_key {
        world_signals.remove_entity(old.as_str());
    }
    world_signals.set_entity(ev.key.clone(), ev.entity);
    debug!(
        "register_entity_observer: registered entity {} as '{}'",
        ev.entity.to_bits(),
        ev.key
    );
    refresh_inspector(&mut commands, ev.entity);
}

pub fn unregister_entity_observer(
    trigger: On<UnregisterEntityRequested>,
    mut world_signals: ResMut<WorldSignals>,
    mut commands: Commands,
) {
    let ev = trigger.event();
    world_signals.remove_entity(ev.key.as_str());
    debug!("unregister_entity_observer: removed key '{}'", ev.key);
    refresh_inspector(&mut commands, ev.entity);
}

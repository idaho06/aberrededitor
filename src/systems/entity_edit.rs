use super::entity_inspector::InspectEntityRequested;
use aberredengine::bevy_ecs;
use aberredengine::bevy_ecs::prelude::{Commands, Entity, Event, On, Query};
use aberredengine::components::animation::Animation;
use aberredengine::components::boxcollider::BoxCollider;
use aberredengine::components::group::Group;
use aberredengine::components::mapposition::MapPosition;
use aberredengine::components::rotation::Rotation;
use aberredengine::components::scale::Scale;
use aberredengine::components::sprite::Sprite;
use aberredengine::components::zindex::ZIndex;
use aberredengine::raylib::prelude::Vector2;
use log::{info, warn};
use std::sync::Arc;

#[allow(dead_code)]
#[derive(Event)]
pub struct UpdateMapPositionRequested {
    pub entity: Entity,
    pub x: f32,
    pub y: f32,
}

#[allow(dead_code)]
#[derive(Event)]
pub struct UpdateZIndexRequested {
    pub entity: Entity,
    pub z_index: f32,
}

#[allow(dead_code)]
#[derive(Event)]
pub struct UpdateGroupRequested {
    pub entity: Entity,
    pub group: String,
}

#[allow(dead_code)]
#[derive(Event)]
pub struct UpdateRotationRequested {
    pub entity: Entity,
    pub rotation_deg: f32,
}

#[allow(dead_code)]
#[derive(Event)]
pub struct UpdateScaleRequested {
    pub entity: Entity,
    pub x: f32,
    pub y: f32,
}

#[allow(dead_code)]
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

#[allow(dead_code)]
#[derive(Event)]
pub struct UpdateBoxColliderRequested {
    pub entity: Entity,
    pub size: [f32; 2],
    pub offset: [f32; 2],
    pub origin: [f32; 2],
}

#[allow(dead_code)]
#[derive(Event)]
pub struct UpdateAnimationRequested {
    pub entity: Entity,
    pub animation_key: String,
    pub frame_index: usize,
    pub elapsed_time: f32,
}

pub fn update_map_position_observer(
    trigger: On<UpdateMapPositionRequested>,
    mut query: Query<&mut MapPosition>,
    mut commands: Commands,
) {
    let event = trigger.event();
    let entity = event.entity;
    let Ok(mut map_position) = query.get_mut(entity) else {
        warn_missing_component("update_map_position_observer", entity, "MapPosition");
        return;
    };
    map_position.pos = Vector2::new(event.x, event.y);
    info!(
        "update_map_position_observer: updated entity {} -> ({:.3}, {:.3})",
        entity.to_bits(),
        event.x,
        event.y
    );
    refresh_inspector(&mut commands, entity);
}

pub fn update_z_index_observer(
    trigger: On<UpdateZIndexRequested>,
    mut query: Query<&mut ZIndex>,
    mut commands: Commands,
) {
    let event = trigger.event();
    let entity = event.entity;
    let Ok(mut z_index) = query.get_mut(entity) else {
        warn_missing_component("update_z_index_observer", entity, "ZIndex");
        return;
    };
    z_index.0 = event.z_index;
    info!(
        "update_z_index_observer: updated entity {} -> {:.3}",
        entity.to_bits(),
        event.z_index
    );
    refresh_inspector(&mut commands, entity);
}

pub fn update_group_observer(
    trigger: On<UpdateGroupRequested>,
    mut query: Query<&mut Group>,
    mut commands: Commands,
) {
    let event = trigger.event();
    let entity = event.entity;
    let Ok(mut group) = query.get_mut(entity) else {
        warn_missing_component("update_group_observer", entity, "Group");
        return;
    };
    group.0 = event.group.clone();
    info!(
        "update_group_observer: updated entity {} -> '{}'",
        entity.to_bits(),
        event.group
    );
    refresh_inspector(&mut commands, entity);
}

pub fn update_rotation_observer(
    trigger: On<UpdateRotationRequested>,
    mut query: Query<&mut Rotation>,
    mut commands: Commands,
) {
    let event = trigger.event();
    let entity = event.entity;
    let Ok(mut rotation) = query.get_mut(entity) else {
        warn_missing_component("update_rotation_observer", entity, "Rotation");
        return;
    };
    rotation.degrees = event.rotation_deg;
    info!(
        "update_rotation_observer: updated entity {} -> {:.3} deg",
        entity.to_bits(),
        event.rotation_deg
    );
    refresh_inspector(&mut commands, entity);
}

pub fn update_scale_observer(
    trigger: On<UpdateScaleRequested>,
    mut query: Query<&mut Scale>,
    mut commands: Commands,
) {
    let event = trigger.event();
    let entity = event.entity;
    let Ok(mut scale) = query.get_mut(entity) else {
        warn_missing_component("update_scale_observer", entity, "Scale");
        return;
    };
    scale.scale = Vector2::new(event.x, event.y);
    info!(
        "update_scale_observer: updated entity {} -> ({:.3}, {:.3})",
        entity.to_bits(),
        event.x,
        event.y
    );
    refresh_inspector(&mut commands, entity);
}

pub fn update_sprite_observer(
    trigger: On<UpdateSpriteRequested>,
    mut query: Query<&mut Sprite>,
    mut commands: Commands,
) {
    let event = trigger.event();
    let entity = event.entity;
    let Ok(mut sprite) = query.get_mut(entity) else {
        warn_missing_component("update_sprite_observer", entity, "Sprite");
        return;
    };
    sprite.tex_key = Arc::from(event.tex_key.as_str());
    sprite.width = event.width;
    sprite.height = event.height;
    sprite.offset = Vector2::new(event.offset[0], event.offset[1]);
    sprite.origin = Vector2::new(event.origin[0], event.origin[1]);
    sprite.flip_h = event.flip_h;
    sprite.flip_v = event.flip_v;
    info!(
        "update_sprite_observer: updated entity {} sprite '{}'",
        entity.to_bits(),
        event.tex_key
    );
    refresh_inspector(&mut commands, entity);
}

pub fn update_box_collider_observer(
    trigger: On<UpdateBoxColliderRequested>,
    mut query: Query<&mut BoxCollider>,
    mut commands: Commands,
) {
    let event = trigger.event();
    let entity = event.entity;
    let Ok(mut collider) = query.get_mut(entity) else {
        warn_missing_component("update_box_collider_observer", entity, "BoxCollider");
        return;
    };
    collider.size = Vector2::new(event.size[0], event.size[1]);
    collider.offset = Vector2::new(event.offset[0], event.offset[1]);
    collider.origin = Vector2::new(event.origin[0], event.origin[1]);
    info!(
        "update_box_collider_observer: updated entity {} collider",
        entity.to_bits()
    );
    refresh_inspector(&mut commands, entity);
}

pub fn update_animation_observer(
    trigger: On<UpdateAnimationRequested>,
    mut query: Query<&mut Animation>,
    mut commands: Commands,
) {
    let event = trigger.event();
    let entity = event.entity;
    let Ok(mut animation) = query.get_mut(entity) else {
        warn_missing_component("update_animation_observer", entity, "Animation");
        return;
    };
    animation.animation_key = event.animation_key.clone();
    animation.frame_index = event.frame_index;
    animation.elapsed_time = event.elapsed_time;
    info!(
        "update_animation_observer: updated entity {} animation '{}' frame {} elapsed {:.3}",
        entity.to_bits(),
        event.animation_key,
        event.frame_index,
        event.elapsed_time
    );
    refresh_inspector(&mut commands, entity);
}

fn refresh_inspector(commands: &mut Commands, entity: Entity) {
    commands.trigger(InspectEntityRequested { entity });
}

fn warn_missing_component(observer: &str, entity: Entity, component: &str) {
    warn!(
        "{}: entity {} missing {}",
        observer,
        entity.to_bits(),
        component
    );
}

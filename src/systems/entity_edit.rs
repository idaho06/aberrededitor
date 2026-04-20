use super::entity_inspector::InspectEntityRequested;
use aberredengine::bevy_ecs;
use aberredengine::bevy_ecs::prelude::{Commands, Entity, Event, On};

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
    mut commands: Commands,
) {
    let entity = trigger.event().entity;
    commands.trigger(InspectEntityRequested { entity });
}

pub fn update_z_index_observer(trigger: On<UpdateZIndexRequested>, mut commands: Commands) {
    let entity = trigger.event().entity;
    commands.trigger(InspectEntityRequested { entity });
}

pub fn update_group_observer(trigger: On<UpdateGroupRequested>, mut commands: Commands) {
    let entity = trigger.event().entity;
    commands.trigger(InspectEntityRequested { entity });
}

pub fn update_rotation_observer(trigger: On<UpdateRotationRequested>, mut commands: Commands) {
    let entity = trigger.event().entity;
    commands.trigger(InspectEntityRequested { entity });
}

pub fn update_scale_observer(trigger: On<UpdateScaleRequested>, mut commands: Commands) {
    let entity = trigger.event().entity;
    commands.trigger(InspectEntityRequested { entity });
}

pub fn update_sprite_observer(trigger: On<UpdateSpriteRequested>, mut commands: Commands) {
    let entity = trigger.event().entity;
    commands.trigger(InspectEntityRequested { entity });
}

pub fn update_box_collider_observer(
    trigger: On<UpdateBoxColliderRequested>,
    mut commands: Commands,
) {
    let entity = trigger.event().entity;
    commands.trigger(InspectEntityRequested { entity });
}

pub fn update_animation_observer(trigger: On<UpdateAnimationRequested>, mut commands: Commands) {
    let entity = trigger.event().entity;
    commands.trigger(InspectEntityRequested { entity });
}

use super::entity_inspector::InspectEntityRequested;
use aberredengine::bevy_ecs;
use aberredengine::bevy_ecs::prelude::{Commands, Entity, Event, On, Query};
use aberredengine::components::animation::Animation;
use aberredengine::components::boxcollider::BoxCollider;
use aberredengine::components::group::Group;
use aberredengine::components::mapposition::MapPosition;
use aberredengine::components::phase::Phase;
use aberredengine::components::rotation::Rotation;
use aberredengine::components::scale::Scale;
use aberredengine::components::sprite::Sprite;
use aberredengine::components::timer::Timer;
use aberredengine::components::ttl::Ttl;
use aberredengine::components::zindex::ZIndex;
use aberredengine::raylib::prelude::Vector2;
use log::{debug, warn};
use std::sync::Arc;

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

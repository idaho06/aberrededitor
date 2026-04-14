use crate::signals as sig;
use aberredengine::bevy_ecs;
use aberredengine::bevy_ecs::prelude::{Entity, Event, On, Query, ResMut};
use aberredengine::components::boxcollider::BoxCollider;
use aberredengine::components::group::Group;
use aberredengine::components::mapposition::MapPosition;
use aberredengine::components::rotation::Rotation;
use aberredengine::components::scale::Scale;
use aberredengine::components::sprite::Sprite;
use aberredengine::components::zindex::ZIndex;
use aberredengine::resources::worldsignals::WorldSignals;

// ---------------------------------------------------------------------------
// Event
// ---------------------------------------------------------------------------

#[derive(Event)]
pub struct InspectEntityRequested {
    pub entity: Entity,
}

// ---------------------------------------------------------------------------
// Snapshot structs
// ---------------------------------------------------------------------------

#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct SpriteSnapshot {
    pub(crate) tex_key: String,
    pub(crate) width: f32,
    pub(crate) height: f32,
    pub(crate) offset: [f32; 2],
    pub(crate) origin: [f32; 2],
    pub(crate) flip_h: bool,
    pub(crate) flip_v: bool,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct ColliderSnapshot {
    pub(crate) size: [f32; 2],
    pub(crate) offset: [f32; 2],
    pub(crate) origin: [f32; 2],
}

#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct ComponentSnapshot {
    pub(crate) entity_bits: u64,
    /// WorldSignals entity keys whose value matches this entity.
    pub(crate) world_signal_keys: Vec<String>,
    pub(crate) map_position: [f32; 2],
    pub(crate) z_index: Option<f32>,
    pub(crate) group: Option<String>,
    pub(crate) sprite: Option<SpriteSnapshot>,
    pub(crate) box_collider: Option<ColliderSnapshot>,
    pub(crate) rotation_deg: Option<f32>,
    pub(crate) scale: Option<[f32; 2]>,
}

// ---------------------------------------------------------------------------
// Observer
// ---------------------------------------------------------------------------

#[allow(clippy::type_complexity)]
pub fn entity_inspect_observer(
    trigger: On<InspectEntityRequested>,
    query: Query<(
        &MapPosition,
        Option<&ZIndex>,
        Option<&Sprite>,
        Option<&BoxCollider>,
        Option<&Group>,
        Option<&Rotation>,
        Option<&Scale>,
    )>,
    mut signals: ResMut<WorldSignals>,
) {
    let entity = trigger.event().entity;
    let Ok((pos, z, sprite, collider, group, rot, scale)) = query.get(entity) else {
        return;
    };

    let world_signal_keys: Vec<String> = signals.entities
        .iter()
        .filter(|(_, e)| **e == entity)
        .map(|(k, _)| k.clone())
        .collect();

    let snapshot = ComponentSnapshot {
        entity_bits: entity.to_bits(),
        world_signal_keys,
        map_position: [pos.pos.x, pos.pos.y],
        z_index: z.map(|z| z.0),
        group: group.map(|g| g.0.clone()),
        sprite: sprite.map(|s| SpriteSnapshot {
            tex_key: s.tex_key.to_string(),
            width: s.width,
            height: s.height,
            offset: [s.offset.x, s.offset.y],
            origin: [s.origin.x, s.origin.y],
            flip_h: s.flip_h,
            flip_v: s.flip_v,
        }),
        box_collider: collider.map(|c| ColliderSnapshot {
            size: [c.size.x, c.size.y],
            offset: [c.offset.x, c.offset.y],
            origin: [c.origin.x, c.origin.y],
        }),
        rotation_deg: rot.map(|r| r.degrees),
        scale: scale.map(|s| [s.scale.x, s.scale.y]),
    };

    if let Ok(json) = serde_json::to_string(&snapshot) {
        signals.set_string(sig::EE_COMPONENT_SNAPSHOT, &json);
        signals.set_flag(sig::UI_ENTITY_EDITOR_OPEN);
    }
}

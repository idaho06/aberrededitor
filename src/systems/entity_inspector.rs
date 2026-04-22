use crate::editor_types::{
    AnimationSnapshot, ColliderSnapshot, ComponentSnapshot, PhaseSnapshot, SpriteSnapshot,
    TimerSnapshot, TtlSnapshot,
};
use crate::signals as sig;
use aberredengine::bevy_ecs;
use aberredengine::bevy_ecs::prelude::{Entity, Event, On, Query, ResMut};
use aberredengine::components::animation::Animation;
use aberredengine::components::boxcollider::BoxCollider;
use aberredengine::components::group::Group;
use aberredengine::components::mapposition::MapPosition;
use aberredengine::components::persistent::Persistent;
use aberredengine::components::phase::Phase;
use aberredengine::components::rotation::Rotation;
use aberredengine::components::scale::Scale;
use aberredengine::components::sprite::Sprite;
use aberredengine::components::timer::Timer;
use aberredengine::components::ttl::Ttl;
use aberredengine::components::zindex::ZIndex;
use aberredengine::resources::appstate::AppState;
use aberredengine::resources::worldsignals::WorldSignals;

// ---------------------------------------------------------------------------
// Event
// ---------------------------------------------------------------------------

#[derive(Event)]
pub struct InspectEntityRequested {
    pub entity: Entity,
}

// ---------------------------------------------------------------------------
// Observer
// ---------------------------------------------------------------------------

#[allow(clippy::type_complexity)]
pub fn entity_inspect_observer(
    trigger: On<InspectEntityRequested>,
    query: Query<(
        Option<&MapPosition>,
        Option<&ZIndex>,
        Option<&Sprite>,
        Option<&BoxCollider>,
        Option<&Group>,
        Option<&Rotation>,
        Option<&Scale>,
        Option<&Animation>,
        Option<&Ttl>,
        Option<&Timer>,
        Option<&Phase>,
        Option<&Persistent>,
    )>,
    mut signals: ResMut<WorldSignals>,
    mut app_state: ResMut<AppState>,
) {
    let entity = trigger.event().entity;
    let Ok((pos, z, sprite, collider, group, rot, scale, animation, ttl, timer, phase, persistent)) =
        query.get(entity)
    else {
        return;
    };

    let world_signal_keys: Vec<String> = signals
        .entities
        .iter()
        .filter(|(_, e)| **e == entity)
        .map(|(k, _)| k.clone())
        .collect();

    let snapshot = ComponentSnapshot {
        entity_bits: entity.to_bits(),
        world_signal_keys,
        map_position: pos.map(|p| [p.pos.x, p.pos.y]),
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
        animation: animation.map(|a| AnimationSnapshot {
            animation_key: a.animation_key.clone(),
            frame_index: a.frame_index,
            elapsed_time: a.elapsed_time,
        }),
        ttl: ttl.map(|t| TtlSnapshot {
            remaining: t.remaining,
        }),
        timer: timer.map(|t| TimerSnapshot {
            duration: t.duration,
            elapsed: t.elapsed,
        }),
        phase: phase.map(|p| {
            let mut phase_names: Vec<String> = p.phases.keys().cloned().collect();
            phase_names.sort();
            PhaseSnapshot {
                current: p.current.clone(),
                previous: p.previous.clone(),
                next: p.next.clone(),
                time_in_phase: p.time_in_phase,
                phase_names,
            }
        }),
        persistent: persistent.is_some(),
    };

    app_state.insert(snapshot);
    signals.set_flag(sig::UI_ENTITY_EDITOR_OPEN);
}

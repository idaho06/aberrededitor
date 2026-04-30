use crate::components::serialized_lua_setup::SerializedLuaSetup;
use crate::editor_types::{
    AnimationSnapshot, ColliderSnapshot, ComponentSnapshot, DynamicTextSnapshot, PhaseSnapshot,
    SpriteSnapshot, TimerSnapshot, TintSnapshot, TtlSnapshot,
};
use crate::signals as sig;
use aberredengine::bevy_ecs;
use aberredengine::bevy_ecs::hierarchy::ChildOf;
use aberredengine::bevy_ecs::prelude::{Entity, Event, On, Query, ResMut, With};
use aberredengine::components::animation::Animation;
use aberredengine::components::boxcollider::BoxCollider;
use aberredengine::components::dynamictext::DynamicText;
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
        (
            Option<&MapPosition>,
            Option<&ZIndex>,
            Option<&Sprite>,
            Option<&BoxCollider>,
            Option<&Group>,
            Option<&Rotation>,
            Option<&Scale>,
            Option<&Animation>,
        ),
        (
            Option<&Ttl>,
            Option<&Timer>,
            Option<&Phase>,
            Option<&Persistent>,
            Option<&TileMap>,
            Option<&ChildOf>,
            Option<&Tint>,
            Option<&SerializedLuaSetup>,
            Option<&DynamicText>,
        ),
    )>,
    tilemap_roots: Query<(), With<TileMap>>,
    mut signals: ResMut<WorldSignals>,
    mut app_state: ResMut<AppState>,
) {
    let entity = trigger.event().entity;
    let Ok((
        (pos, z, sprite, collider, group, rot, scale, animation),
        (ttl, timer, phase, persistent, tilemap, child_of, tint, lua_setup, dynamic_text),
    )) = query.get(entity)
    else {
        return;
    };

    let world_signal_keys: Vec<String> = signals
        .entities
        .iter()
        .filter(|(k, e)| **e == entity && sig::is_user_entity_key(k))
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
        tilemap_path: tilemap.map(|t| t.path.clone()),
        tilemap_parent: child_of.and_then(|c| tilemap_roots.get(c.0).ok().map(|_| c.0.to_bits())),
        tint: tint.map(|t| TintSnapshot {
            r: t.color.r,
            g: t.color.g,
            b: t.color.b,
            a: t.color.a,
        }),
        lua_setup: lua_setup.map(|l| l.callback.clone()),
        dynamic_text: dynamic_text.map(|d| DynamicTextSnapshot {
            text: d.text.to_string(),
            font_key: d.font.to_string(),
            font_size: d.font_size,
            r: d.color.r,
            g: d.color.g,
            b: d.color.b,
            a: d.color.a,
        }),
    };

    app_state.insert(snapshot);
    signals.set_entity(sig::ES_SELECTED_ENTITY, entity);
    signals.set_flag(sig::UI_ENTITY_EDITOR_OPEN);
}

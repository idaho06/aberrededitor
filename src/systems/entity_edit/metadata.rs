use super::{
    RemoveGroupRequested, RemoveParticleEmitterRequested, RemovePersistentRequested,
    RemovePhaseRequested, RemoveTimerRequested, RemoveTtlRequested, UpdateGroupRequested,
    UpdateParticleEmitterRequested,
};
use crate::editor_types::{EmitterShapeKind, TtlKind};
use aberredengine::bevy_ecs::prelude::{Commands, On, Query, Res};
use aberredengine::components::group::Group;
use aberredengine::components::particleemitter::{EmitterShape, ParticleEmitter, TtlSpec};
use aberredengine::components::persistent::Persistent;
use aberredengine::components::phase::Phase;
use aberredengine::components::timer::Timer;
use aberredengine::components::ttl::Ttl;
use aberredengine::raylib::prelude::Vector2;
use aberredengine::resources::worldsignals::WorldSignals;
use log::{debug, warn};

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

component_remove_observer!(
    remove_group_observer,
    RemoveGroupRequested,
    Group,
    "Group"
);

component_remove_observer!(remove_ttl_observer, RemoveTtlRequested, Ttl, "Ttl");
component_remove_observer!(remove_timer_observer, RemoveTimerRequested, Timer, "Timer");
component_remove_observer!(remove_phase_observer, RemovePhaseRequested, Phase, "Phase");
component_remove_observer!(
    remove_persistent_observer,
    RemovePersistentRequested,
    Persistent,
    "Persistent"
);

pub fn update_particle_emitter_observer(
    trigger: On<UpdateParticleEmitterRequested>,
    world_signals: Res<WorldSignals>,
    existing: Query<Option<&ParticleEmitter>>,
    mut commands: Commands,
) {
    let ev = trigger.event();
    let entity = ev.entity;

    let templates: Vec<_> = ev
        .template_keys
        .iter()
        .filter_map(|k| {
            let e = world_signals.get_entity(k).copied();
            if e.is_none() {
                warn!(
                    "update_particle_emitter_observer: template key '{}' not found; skipping",
                    k
                );
            }
            e
        })
        .collect();

    let shape = match ev.shape {
        EmitterShapeKind::Point => EmitterShape::Point,
        EmitterShapeKind::Rect => EmitterShape::Rect {
            width: ev.shape_rect_w,
            height: ev.shape_rect_h,
        },
    };

    let ttl = match ev.ttl_kind {
        TtlKind::None => TtlSpec::None,
        TtlKind::Fixed => TtlSpec::Fixed(ev.ttl_fixed),
        TtlKind::Range => TtlSpec::Range {
            min: ev.ttl_min,
            max: ev.ttl_max,
        },
    };

    let arc_degrees = if ev.arc_min_deg <= ev.arc_max_deg {
        (ev.arc_min_deg, ev.arc_max_deg)
    } else {
        (ev.arc_max_deg, ev.arc_min_deg)
    };
    let speed_range = if ev.speed_min <= ev.speed_max {
        (ev.speed_min, ev.speed_max)
    } else {
        (ev.speed_max, ev.speed_min)
    };

    let time_since_emit = existing
        .get(entity)
        .ok()
        .flatten()
        .map(|pe| pe.time_since_emit)
        .unwrap_or(0.0);

    commands.entity(entity).insert(ParticleEmitter {
        templates,
        shape,
        offset: Vector2 {
            x: ev.offset[0],
            y: ev.offset[1],
        },
        particles_per_emission: ev.particles_per_emission,
        emissions_per_second: ev.emissions_per_second,
        emissions_remaining: ev.emissions_remaining,
        initial_emissions_remaining: ev.emissions_remaining,
        arc_degrees,
        speed_range,
        ttl,
        time_since_emit,
    });

    debug!(
        "update_particle_emitter_observer: updated entity {} emitter",
        entity.to_bits()
    );
    super::refresh_inspector(&mut commands, entity);
}

component_remove_observer!(
    remove_particle_emitter_observer,
    RemoveParticleEmitterRequested,
    ParticleEmitter,
    "ParticleEmitter"
);

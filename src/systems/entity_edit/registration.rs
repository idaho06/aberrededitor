use super::{RegisterEntityRequested, UnregisterEntityRequested};
use aberredengine::bevy_ecs::prelude::{Commands, On, ResMut};
use aberredengine::resources::worldsignals::WorldSignals;
use log::debug;

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
    super::refresh_inspector(&mut commands, ev.entity);
}

pub fn unregister_entity_observer(
    trigger: On<UnregisterEntityRequested>,
    mut world_signals: ResMut<WorldSignals>,
    mut commands: Commands,
) {
    let ev = trigger.event();
    world_signals.remove_entity(ev.key.as_str());
    debug!("unregister_entity_observer: removed key '{}'", ev.key);
    super::refresh_inspector(&mut commands, ev.entity);
}

use aberredengine::bevy_ecs::observer::On;
use aberredengine::bevy_ecs::prelude::*;
use aberredengine::events::switchdebug::SwitchDebugEvent;
use aberredengine::resources::debugmode::DebugMode;
use aberredengine::resources::worldsignals::WorldSignals;

/// Mirrors the debug-mode toggle into `WorldSignals` so the GUI can show a
/// checked state on the "Toggle Debug Mode" menu item.
///
/// `switch_debug_observer` (engine-side) uses deferred commands, so `DebugMode`
/// is still in its **pre-toggle** state when this observer runs. We invert:
/// `Some` → about to go OFF, `None` → about to go ON.
pub fn debug_mode_mirror_observer(
    _trigger: On<SwitchDebugEvent>,
    debug_mode: Option<Res<DebugMode>>,
    mut world_signals: ResMut<WorldSignals>,
) {
    if debug_mode.is_some() {
        world_signals.clear_flag("ui:debug_active");
    } else {
        world_signals.set_flag("ui:debug_active");
    }
}

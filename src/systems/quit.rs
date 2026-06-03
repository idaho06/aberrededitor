use crate::signals as sig;
use aberredengine::bevy_ecs::prelude::{Res, ResMut};
use aberredengine::resources::gamestate::{GameStates, NextGameState};
use aberredengine::resources::worldsignals::WorldSignals;

pub fn quit_handler(
    world_signals: Res<WorldSignals>,
    mut next_state: ResMut<NextGameState>,
) {
    if world_signals.has_flag(sig::ACTION_QUIT) {
        next_state.set(GameStates::Quitting);
    }
}

use crate::signals as sig;
use aberredengine::prelude::{GameState, NextGameState, Res, ResMut, WorldSignals};

pub fn quit_handler(world_signals: Res<WorldSignals>, mut next_state: ResMut<NextGameState>) {
    if world_signals.has_flag(sig::ACTION_QUIT) {
        next_state.set(GameState::Quitting);
    }
}

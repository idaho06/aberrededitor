use super::pending_state::{PendingEditState, PendingMutex};
use crate::signals as sig;
use aberredengine::bevy_ecs;
use aberredengine::bevy_ecs::prelude::{Entity, Res, ResMut, Resource};
use aberredengine::resources::appstate::AppState;
use aberredengine::resources::worldsignals::WorldSignals;

/// Canonical owner of ECS-only transient editor state.
///
/// Fields here are never needed by the GUI callback (which only reads `WorldSignals`).
/// Storing them here instead of in `WorldSignals` keeps the signal bus as pure transport.
#[derive(Resource, Default)]
pub struct EditorState {
    /// The entity that was selected when the last inspector snapshot was built.
    /// Used to detect selection changes and clear pending edit buffers.
    pub last_selected: Option<Entity>,
}

/// Detects entity selection changes and clears pending edit buffers on change.
pub fn entity_editor_selection_change_system(
    mut editor_state: ResMut<EditorState>,
    signals: Res<WorldSignals>,
    app_state: Res<AppState>,
) {
    let current = signals.get_entity(sig::ES_SELECTED_ENTITY).copied();
    if current != editor_state.last_selected {
        clear_entity_editor_pending(&app_state);
        editor_state.last_selected = current;
    }
}

pub(super) fn clear_entity_editor_pending(app_state: &AppState) {
    if let Some(m) = app_state.get::<PendingMutex>() {
        *m.lock().unwrap() = PendingEditState::default();
    }
}

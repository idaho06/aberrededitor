use aberredengine::bevy_ecs;
use aberredengine::bevy_ecs::prelude::Resource;
use crate::signals as sig;
use aberredengine::bevy_ecs::prelude::{Entity, ResMut};
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
    mut signals: ResMut<WorldSignals>,
) {
    let current = signals.get_entity(sig::ES_SELECTED_ENTITY).copied();
    if current != editor_state.last_selected {
        clear_entity_editor_pending(&mut signals);
        editor_state.last_selected = current;
    }
}

pub(super) fn clear_entity_editor_pending(signals: &mut WorldSignals) {
    for key in [
        sig::GUI_EE_PENDING_POS_X,
        sig::GUI_EE_PENDING_POS_Y,
        sig::GUI_EE_PENDING_Z_INDEX,
        sig::GUI_EE_PENDING_ROT_DEG,
        sig::GUI_EE_PENDING_SCALE_X,
        sig::GUI_EE_PENDING_SCALE_Y,
        sig::GUI_EE_PENDING_SPRITE_WIDTH,
        sig::GUI_EE_PENDING_SPRITE_HEIGHT,
        sig::GUI_EE_PENDING_SPRITE_OFFX,
        sig::GUI_EE_PENDING_SPRITE_OFFY,
        sig::GUI_EE_PENDING_SPRITE_ORGX,
        sig::GUI_EE_PENDING_SPRITE_ORGY,
        sig::GUI_EE_PENDING_BOX_SIZE_X,
        sig::GUI_EE_PENDING_BOX_SIZE_Y,
        sig::GUI_EE_PENDING_BOX_OFFX,
        sig::GUI_EE_PENDING_BOX_OFFY,
        sig::GUI_EE_PENDING_BOX_ORGX,
        sig::GUI_EE_PENDING_BOX_ORGY,
        sig::GUI_EE_PENDING_ANIM_ELAPSED,
    ] {
        signals.clear_scalar(key);
    }

    signals.clear_integer(sig::GUI_EE_PENDING_ANIM_FRAME_INDEX);

    for key in [
        sig::GUI_EE_PENDING_GROUP,
        sig::GUI_EE_PENDING_SPRITE_TEX_KEY,
        sig::GUI_EE_PENDING_ANIM_KEY,
    ] {
        signals.remove_string(key);
    }

    for key in [
        sig::GUI_EE_PENDING_GROUP_DIRTY,
        sig::GUI_EE_PENDING_SPRITE_TEX_KEY_DIRTY,
        sig::GUI_EE_PENDING_ANIM_KEY_DIRTY,
        sig::GUI_EE_PENDING_SPRITE_FLIP_H,
        sig::GUI_EE_PENDING_SPRITE_FLIP_V,
        sig::GUI_EE_PENDING_SPRITE_FLIP_H_DIRTY,
        sig::GUI_EE_PENDING_SPRITE_FLIP_V_DIRTY,
        sig::ACTION_EE_COMMIT_POSITION,
        sig::ACTION_EE_COMMIT_Z,
        sig::ACTION_EE_COMMIT_GROUP,
        sig::ACTION_EE_COMMIT_ROTATION,
        sig::ACTION_EE_COMMIT_SCALE,
        sig::ACTION_EE_COMMIT_SPRITE,
        sig::ACTION_EE_COMMIT_COLLIDER,
        sig::ACTION_EE_COMMIT_ANIMATION,
    ] {
        signals.clear_flag(key);
    }
}

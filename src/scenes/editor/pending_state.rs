use crate::editor_types::ComponentKind;
use super::components::{
    animation::PendingAnimation,
    collider::PendingCollider,
    dynamic_text::PendingDynamicText,
    lua_setup::PendingLuaSetup,
    particle_emitter::PendingParticleEmitter,
    readonly::PendingReadonlyRemovals,
    sprite::PendingSprite,
    tint::PendingTint,
    transform::PendingTransform,
};
use std::sync::Mutex;

/// Thin aggregate of per-component pending sub-structs plus entity-level action flags.
///
/// `Option<T>` fields inside each sub-struct encode dirty state: `None` = unedited
/// (fall back to snapshot), `Some(v)` = user changed it. Commit booleans signal which
/// component group should be written to ECS this frame.
///
/// Reset with `*self = Self::default()` after each commit or on selection change.
#[derive(Default, Clone)]
pub(crate) struct PendingEditState {
    pub transform: PendingTransform,
    pub sprite: PendingSprite,
    pub collider: PendingCollider,
    pub animation: PendingAnimation,
    pub tint: PendingTint,
    pub lua_setup: PendingLuaSetup,
    pub dynamic_text: PendingDynamicText,
    pub particle_emitter: PendingParticleEmitter,
    pub readonly_removals: PendingReadonlyRemovals,
    // Entity-level actions (not component-specific):
    pub remove_entity: bool,
    pub clone_entity: bool,
    pub remove_tilemap: bool,
    pub bake_tilemap: bool,
    pub select_tilemap_parent: bool,
    pub pending_register_key: Option<String>,
    pub commit_registration: bool,
    pub remove_registration: bool,
    pub add_component: Option<ComponentKind>,
    pub add_combo_selection: usize,
}

impl PendingEditState {
    pub(crate) fn any_commit(&self) -> bool {
        self.transform.is_dirty()
            || self.sprite.is_dirty()
            || self.collider.is_dirty()
            || self.animation.is_dirty()
            || self.tint.is_dirty()
            || self.lua_setup.is_dirty()
            || self.dynamic_text.is_dirty()
            || self.particle_emitter.is_dirty()
            || self.readonly_removals.is_dirty()
            || self.remove_entity
            || self.clone_entity
            || self.remove_tilemap
            || self.bake_tilemap
            || self.select_tilemap_parent
            || self.commit_registration
            || self.remove_registration
            || self.add_component.is_some()
    }
}

/// Convenience alias used by callers that store this in AppState.
pub(crate) type PendingMutex = Mutex<PendingEditState>;

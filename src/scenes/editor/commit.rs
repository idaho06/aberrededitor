//! Pending-state commit dispatcher: converts GUI edit buffers into ECS observer events.
//!
//! `consume_entity_editor_commits` is called from `editor_update` every frame. It:
//! 1. Checks `PendingMutex` — returns immediately if `any_commit()` is false.
//! 2. Verifies a selected entity exists and its snapshot is consistent.
//! 3. For each dirty component group, delegates to `components::<name>::commit`.
//! 4. Clears the pending state via `clear_entity_editor_pending`.
use super::components;
use super::pending_state::PendingMutex;
use super::state::clear_entity_editor_pending;
use crate::editor_types::ComponentSnapshot;
use crate::signals as sig;
use crate::systems::entity_edit::{
    AddComponentRequested, BakeTilemapRequested, CloneEntityRequested, RegisterEntityRequested,
    RemoveEntityRequested, RemoveTileMapRequested, UnregisterEntityRequested,
};
use crate::systems::entity_inspector::InspectEntityRequested;
use aberredengine::bevy_ecs::prelude::Entity;
use aberredengine::resources::appstate::AppState;
use aberredengine::resources::worldsignals::WorldSignals;
use aberredengine::systems::GameCtx;

pub(super) fn consume_entity_editor_commits(ctx: &mut GameCtx) {
    // Clone pending state out of the mutex before any mutable borrow of ctx.
    let p = {
        let Some(mutex) = ctx.app_state.get::<PendingMutex>() else {
            return;
        };
        let guard = mutex.lock().unwrap();
        if !guard.any_commit() {
            return;
        }
        guard.clone()
    };

    let Some((entity, snapshot)) = selected_entity_and_snapshot(&ctx.world_signals, &ctx.app_state)
    else {
        clear_entity_editor_pending(&ctx.app_state);
        return;
    };

    if p.remove_entity {
        ctx.commands.trigger(RemoveEntityRequested { entity });
        clear_entity_editor_pending(&ctx.app_state);
        return;
    }

    if p.clone_entity {
        let x = ctx
            .world_signals
            .get_scalar(sig::CAM_TARGET_X)
            .unwrap_or(0.0);
        let y = ctx
            .world_signals
            .get_scalar(sig::CAM_TARGET_Y)
            .unwrap_or(0.0);
        ctx.commands.trigger(CloneEntityRequested { entity, x, y });
        clear_entity_editor_pending(&ctx.app_state);
        return;
    }

    components::transform::commit(ctx, entity, &snapshot, &p.transform);
    components::sprite::commit(ctx, entity, &snapshot, &p.sprite);
    components::collider::commit(ctx, entity, &snapshot, &p.collider);
    components::animation::commit(ctx, entity, &snapshot, &p.animation);
    components::readonly::commit(ctx, entity, &p.readonly_removals);
    components::tint::commit(ctx, entity, &snapshot, &p.tint);
    components::lua_setup::commit(ctx, entity, &snapshot, &p.lua_setup);
    components::dynamic_text::commit(ctx, entity, &snapshot, &p.dynamic_text);
    components::particle_emitter::commit(ctx, entity, &snapshot, &p.particle_emitter);

    if p.remove_tilemap {
        ctx.commands.trigger(RemoveTileMapRequested { entity });
    }
    if p.bake_tilemap {
        ctx.commands.trigger(BakeTilemapRequested { entity });
    }
    if p.select_tilemap_parent
        && let Some(parent_bits) = snapshot.tilemap_parent
    {
        let parent = Entity::from_bits(parent_bits);
        ctx.commands
            .trigger(InspectEntityRequested { entity: parent });
    }
    if p.commit_registration
        && let Some(ref key) = p.pending_register_key
        && !key.is_empty()
    {
        let old_key = snapshot.world_signal_keys.first().cloned();
        ctx.commands.trigger(RegisterEntityRequested {
            entity,
            key: key.clone(),
            old_key,
        });
    }
    if p.remove_registration
        && let Some(key) = snapshot.world_signal_keys.first().cloned()
    {
        ctx.commands
            .trigger(UnregisterEntityRequested { entity, key });
    }
    if let Some(kind) = p.add_component {
        ctx.commands.trigger(AddComponentRequested { entity, kind });
    }

    clear_entity_editor_pending(&ctx.app_state);
}

fn selected_entity_and_snapshot(
    signals: &WorldSignals,
    app_state: &AppState,
) -> Option<(Entity, ComponentSnapshot)> {
    let entity = signals.get_entity(sig::ES_SELECTED_ENTITY).copied()?;
    let snapshot = app_state.get::<ComponentSnapshot>()?.clone();
    (snapshot.entity_bits == entity.to_bits()).then_some((entity, snapshot))
}

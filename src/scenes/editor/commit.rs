//! Pending-state commit dispatcher: converts GUI edit buffers into ECS observer events.
//!
//! `consume_entity_editor_commits` is called from `editor_update` every frame. It:
//! 1. Checks `PendingMutex` — returns immediately if `any_commit()` is false.
//! 2. Verifies a selected entity exists and its snapshot is consistent.
//! 3. For each dirty component group (`commit_xyz = true` or `remove_xyz = true`), triggers
//!    the corresponding `Update*Requested` or `Remove*Requested` event.
//! 4. Clears the pending state via `clear_entity_editor_pending`.
//!
//! Each `consume_xyz_commit` helper merges the pending `Option<T>` fields with the snapshot
//! fallback (`p.field.unwrap_or(snap.field)`) to produce the final event payload. This
//! implements the `PendingEditState` dirty-encoding pattern — see `docs/patterns.md §4`.
use super::pending_state::{PendingEditState, PendingMutex};
use super::state::clear_entity_editor_pending;
use crate::editor_types::ComponentSnapshot;
use crate::signals as sig;
use crate::systems::entity_edit::{
    AddComponentRequested, BakeTilemapRequested, CloneEntityRequested, RegisterEntityRequested,
    RemoveAnimationRequested, RemoveBoxColliderRequested, RemoveDynamicTextRequested,
    RemoveEntityRequested, RemoveGroupRequested, RemoveLuaSetupRequested,
    RemoveMapPositionRequested, RemovePersistentRequested, RemovePhaseRequested,
    RemoveRotationRequested, RemoveScaleRequested, RemoveSpriteRequested, RemoveTileMapRequested,
    RemoveTimerRequested, RemoveTintRequested, RemoveTtlRequested, RemoveZIndexRequested,
    UnregisterEntityRequested, UpdateAnimationRequested, UpdateBoxColliderRequested,
    UpdateDynamicTextRequested, UpdateGroupRequested, UpdateLuaSetupRequested,
    UpdateMapPositionRequested, UpdateRotationRequested, UpdateScaleRequested,
    UpdateSpriteRequested, UpdateTintRequested, UpdateZIndexRequested,
};
use crate::systems::entity_inspector::InspectEntityRequested;
use aberredengine::bevy_ecs::prelude::Entity;
use aberredengine::resources::appstate::AppState;
use aberredengine::resources::worldsignals::WorldSignals;
use aberredengine::systems::GameCtx;
use log::warn;

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

    if p.remove_map_position {
        ctx.commands.trigger(RemoveMapPositionRequested { entity });
    } else if p.commit_position {
        consume_position_commit(ctx, entity, &snapshot, &p);
    }
    if p.remove_z {
        ctx.commands.trigger(RemoveZIndexRequested { entity });
    } else if p.commit_z {
        consume_z_commit(ctx, entity, &snapshot, &p);
    }
    if p.remove_group {
        ctx.commands.trigger(RemoveGroupRequested { entity });
    } else if p.commit_group {
        consume_group_commit(ctx, entity, &snapshot, &p);
    }
    if p.remove_rotation {
        ctx.commands.trigger(RemoveRotationRequested { entity });
    } else if p.commit_rotation {
        consume_rotation_commit(ctx, entity, &snapshot, &p);
    }
    if p.remove_scale {
        ctx.commands.trigger(RemoveScaleRequested { entity });
    } else if p.commit_scale {
        consume_scale_commit(ctx, entity, &snapshot, &p);
    }
    if p.remove_sprite {
        ctx.commands.trigger(RemoveSpriteRequested { entity });
    } else if p.commit_sprite {
        consume_sprite_commit(ctx, entity, &snapshot, &p);
    }
    if p.remove_collider {
        ctx.commands.trigger(RemoveBoxColliderRequested { entity });
    } else if p.commit_collider {
        consume_collider_commit(ctx, entity, &snapshot, &p);
    }
    if p.remove_animation {
        ctx.commands.trigger(RemoveAnimationRequested { entity });
    } else if p.commit_animation {
        consume_animation_commit(ctx, entity, &snapshot, &p);
    }
    if p.remove_ttl {
        ctx.commands.trigger(RemoveTtlRequested { entity });
    }
    if p.remove_timer {
        ctx.commands.trigger(RemoveTimerRequested { entity });
    }
    if p.remove_phase {
        ctx.commands.trigger(RemovePhaseRequested { entity });
    }
    if p.remove_persistent {
        ctx.commands.trigger(RemovePersistentRequested { entity });
    }
    if p.remove_tint {
        ctx.commands.trigger(RemoveTintRequested { entity });
    } else if p.commit_tint {
        consume_tint_commit(ctx, entity, &snapshot, &p);
    }
    if p.remove_lua_setup {
        ctx.commands.trigger(RemoveLuaSetupRequested { entity });
    } else if p.commit_lua_setup {
        consume_lua_setup_commit(ctx, entity, &snapshot, &p);
    }
    if p.remove_dynamic_text {
        ctx.commands.trigger(RemoveDynamicTextRequested { entity });
    } else if p.commit_dynamic_text {
        consume_dynamic_text_commit(ctx, entity, &snapshot, &p);
    }
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

fn consume_position_commit(
    ctx: &mut GameCtx,
    entity: Entity,
    snapshot: &ComponentSnapshot,
    p: &PendingEditState,
) {
    let Some([snap_x, snap_y]) = snapshot.map_position else {
        warn!(
            "consume_position_commit: snapshot missing MapPosition for entity {}",
            entity.to_bits()
        );
        return;
    };
    ctx.commands.trigger(UpdateMapPositionRequested {
        entity,
        x: p.pos_x.unwrap_or(snap_x),
        y: p.pos_y.unwrap_or(snap_y),
    });
}

fn consume_z_commit(
    ctx: &mut GameCtx,
    entity: Entity,
    snapshot: &ComponentSnapshot,
    p: &PendingEditState,
) {
    let Some(z_index) = snapshot.z_index else {
        warn!(
            "consume_z_commit: snapshot missing ZIndex for entity {}",
            entity.to_bits()
        );
        return;
    };
    ctx.commands.trigger(UpdateZIndexRequested {
        entity,
        z_index: p.z_index.unwrap_or(z_index),
    });
}

fn consume_group_commit(
    ctx: &mut GameCtx,
    entity: Entity,
    snapshot: &ComponentSnapshot,
    p: &PendingEditState,
) {
    let Some(ref group) = snapshot.group else {
        warn!(
            "consume_group_commit: snapshot missing Group for entity {}",
            entity.to_bits()
        );
        return;
    };
    ctx.commands.trigger(UpdateGroupRequested {
        entity,
        group: p.group.clone().unwrap_or_else(|| group.clone()),
    });
}

fn consume_rotation_commit(
    ctx: &mut GameCtx,
    entity: Entity,
    snapshot: &ComponentSnapshot,
    p: &PendingEditState,
) {
    let Some(rotation_deg) = snapshot.rotation_deg else {
        warn!(
            "consume_rotation_commit: snapshot missing Rotation for entity {}",
            entity.to_bits()
        );
        return;
    };
    ctx.commands.trigger(UpdateRotationRequested {
        entity,
        rotation_deg: p.rotation_deg.unwrap_or(rotation_deg),
    });
}

fn consume_scale_commit(
    ctx: &mut GameCtx,
    entity: Entity,
    snapshot: &ComponentSnapshot,
    p: &PendingEditState,
) {
    let Some([scale_x, scale_y]) = snapshot.scale else {
        warn!(
            "consume_scale_commit: snapshot missing Scale for entity {}",
            entity.to_bits()
        );
        return;
    };
    ctx.commands.trigger(UpdateScaleRequested {
        entity,
        x: p.scale_x.unwrap_or(scale_x),
        y: p.scale_y.unwrap_or(scale_y),
    });
}

fn consume_sprite_commit(
    ctx: &mut GameCtx,
    entity: Entity,
    snapshot: &ComponentSnapshot,
    p: &PendingEditState,
) {
    let Some(ref sprite) = snapshot.sprite else {
        warn!(
            "consume_sprite_commit: snapshot missing Sprite for entity {}",
            entity.to_bits()
        );
        return;
    };
    ctx.commands.trigger(UpdateSpriteRequested {
        entity,
        tex_key: p
            .sprite_tex_key
            .clone()
            .unwrap_or_else(|| sprite.tex_key.clone()),
        width: p.sprite_width.unwrap_or(sprite.width),
        height: p.sprite_height.unwrap_or(sprite.height),
        offset: [
            p.sprite_off_x.unwrap_or(sprite.offset[0]),
            p.sprite_off_y.unwrap_or(sprite.offset[1]),
        ],
        origin: [
            p.sprite_org_x.unwrap_or(sprite.origin[0]),
            p.sprite_org_y.unwrap_or(sprite.origin[1]),
        ],
        flip_h: p.sprite_flip_h.unwrap_or(sprite.flip_h),
        flip_v: p.sprite_flip_v.unwrap_or(sprite.flip_v),
    });
}

fn consume_collider_commit(
    ctx: &mut GameCtx,
    entity: Entity,
    snapshot: &ComponentSnapshot,
    p: &PendingEditState,
) {
    let Some(ref collider) = snapshot.box_collider else {
        warn!(
            "consume_collider_commit: snapshot missing BoxCollider for entity {}",
            entity.to_bits()
        );
        return;
    };
    ctx.commands.trigger(UpdateBoxColliderRequested {
        entity,
        size: [
            p.box_size_x.unwrap_or(collider.size[0]),
            p.box_size_y.unwrap_or(collider.size[1]),
        ],
        offset: [
            p.box_off_x.unwrap_or(collider.offset[0]),
            p.box_off_y.unwrap_or(collider.offset[1]),
        ],
        origin: [
            p.box_org_x.unwrap_or(collider.origin[0]),
            p.box_org_y.unwrap_or(collider.origin[1]),
        ],
    });
}

fn consume_animation_commit(
    ctx: &mut GameCtx,
    entity: Entity,
    snapshot: &ComponentSnapshot,
    p: &PendingEditState,
) {
    let Some(ref animation) = snapshot.animation else {
        warn!(
            "consume_animation_commit: snapshot missing Animation for entity {}",
            entity.to_bits()
        );
        return;
    };
    ctx.commands.trigger(UpdateAnimationRequested {
        entity,
        animation_key: p
            .anim_key
            .clone()
            .unwrap_or_else(|| animation.animation_key.clone()),
    });
}

fn selected_entity_and_snapshot(
    signals: &WorldSignals,
    app_state: &AppState,
) -> Option<(Entity, ComponentSnapshot)> {
    let entity = signals.get_entity(sig::ES_SELECTED_ENTITY).copied()?;
    let snapshot = app_state.get::<ComponentSnapshot>()?.clone();
    (snapshot.entity_bits == entity.to_bits()).then_some((entity, snapshot))
}

fn consume_tint_commit(
    ctx: &mut GameCtx,
    entity: Entity,
    snapshot: &ComponentSnapshot,
    p: &PendingEditState,
) {
    let Some(ref tint) = snapshot.tint else {
        warn!(
            "consume_tint_commit: snapshot missing Tint for entity {}",
            entity.to_bits()
        );
        return;
    };
    let [r, g, b, a] = p.tint_color.unwrap_or(tint.color_normalized());
    ctx.commands.trigger(UpdateTintRequested {
        entity,
        r: (r * 255.0).round() as u8,
        g: (g * 255.0).round() as u8,
        b: (b * 255.0).round() as u8,
        a: (a * 255.0).round() as u8,
    });
}

fn consume_dynamic_text_commit(
    ctx: &mut GameCtx,
    entity: Entity,
    snapshot: &ComponentSnapshot,
    p: &PendingEditState,
) {
    let Some(ref dt) = snapshot.dynamic_text else {
        warn!(
            "consume_dynamic_text_commit: snapshot missing DynamicText for entity {}",
            entity.to_bits()
        );
        return;
    };
    let [r, g, b, a] = p.dynamic_text_color.unwrap_or(dt.color_normalized());
    ctx.commands.trigger(UpdateDynamicTextRequested {
        entity,
        text: p
            .dynamic_text_text
            .clone()
            .unwrap_or_else(|| dt.text.clone()),
        font_key: p
            .dynamic_text_font_key
            .clone()
            .unwrap_or_else(|| dt.font_key.clone()),
        font_size: p.dynamic_text_font_size.unwrap_or(dt.font_size),
        r: (r * 255.0).round() as u8,
        g: (g * 255.0).round() as u8,
        b: (b * 255.0).round() as u8,
        a: (a * 255.0).round() as u8,
    });
}

fn consume_lua_setup_commit(
    ctx: &mut GameCtx,
    entity: Entity,
    snapshot: &ComponentSnapshot,
    p: &PendingEditState,
) {
    let Some(ref callback) = snapshot.lua_setup else {
        warn!(
            "consume_lua_setup_commit: snapshot missing LuaSetup for entity {}",
            entity.to_bits()
        );
        return;
    };
    ctx.commands.trigger(UpdateLuaSetupRequested {
        entity,
        callback: p
            .lua_setup_callback
            .clone()
            .unwrap_or_else(|| callback.clone()),
    });
}

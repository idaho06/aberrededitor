use crate::editor_types::ComponentSnapshot;
use crate::signals as sig;
use crate::systems::entity_edit::{
    UpdateAnimationRequested, UpdateBoxColliderRequested, UpdateGroupRequested,
    UpdateMapPositionRequested, UpdateRotationRequested, UpdateScaleRequested,
    UpdateSpriteRequested, UpdateZIndexRequested,
};
use aberredengine::bevy_ecs::prelude::Entity;
use aberredengine::resources::appstate::AppState;
use aberredengine::resources::worldsignals::WorldSignals;
use aberredengine::systems::GameCtx;
use log::warn;

pub(super) fn consume_entity_editor_commits(ctx: &mut GameCtx) {
    const COMMIT_FLAGS: &[&str] = &[
        sig::ACTION_EE_COMMIT_POSITION,
        sig::ACTION_EE_COMMIT_Z,
        sig::ACTION_EE_COMMIT_GROUP,
        sig::ACTION_EE_COMMIT_ROTATION,
        sig::ACTION_EE_COMMIT_SCALE,
        sig::ACTION_EE_COMMIT_SPRITE,
        sig::ACTION_EE_COMMIT_COLLIDER,
        sig::ACTION_EE_COMMIT_ANIMATION,
    ];

    if !COMMIT_FLAGS
        .iter()
        .any(|key| ctx.world_signals.has_flag(key))
    {
        return;
    }

    let Some((entity, snapshot)) = selected_entity_and_snapshot(&ctx.world_signals, &ctx.app_state) else {
        for key in COMMIT_FLAGS {
            ctx.world_signals.clear_flag(key);
        }
        return;
    };

    if ctx.world_signals.take_flag(sig::ACTION_EE_COMMIT_POSITION) {
        consume_position_commit(ctx, entity, &snapshot);
    }
    if ctx.world_signals.take_flag(sig::ACTION_EE_COMMIT_Z) {
        consume_z_commit(ctx, entity, &snapshot);
    }
    if ctx.world_signals.take_flag(sig::ACTION_EE_COMMIT_GROUP) {
        consume_group_commit(ctx, entity, &snapshot);
    }
    if ctx.world_signals.take_flag(sig::ACTION_EE_COMMIT_ROTATION) {
        consume_rotation_commit(ctx, entity, &snapshot);
    }
    if ctx.world_signals.take_flag(sig::ACTION_EE_COMMIT_SCALE) {
        consume_scale_commit(ctx, entity, &snapshot);
    }
    if ctx.world_signals.take_flag(sig::ACTION_EE_COMMIT_SPRITE) {
        consume_sprite_commit(ctx, entity, &snapshot);
    }
    if ctx.world_signals.take_flag(sig::ACTION_EE_COMMIT_COLLIDER) {
        consume_collider_commit(ctx, entity, &snapshot);
    }
    if ctx.world_signals.take_flag(sig::ACTION_EE_COMMIT_ANIMATION) {
        consume_animation_commit(ctx, entity, &snapshot);
    }
}

fn consume_position_commit(ctx: &mut GameCtx, entity: Entity, snapshot: &ComponentSnapshot) {
    ctx.commands.trigger(UpdateMapPositionRequested {
        entity,
        x: pending_scalar_or(
            &ctx.world_signals,
            sig::GUI_EE_PENDING_POS_X,
            snapshot.map_position[0],
        ),
        y: pending_scalar_or(
            &ctx.world_signals,
            sig::GUI_EE_PENDING_POS_Y,
            snapshot.map_position[1],
        ),
    });
}

fn consume_z_commit(ctx: &mut GameCtx, entity: Entity, snapshot: &ComponentSnapshot) {
    let Some(z_index) = snapshot.z_index else {
        warn!(
            "consume_z_commit: snapshot missing ZIndex for entity {}",
            entity.to_bits()
        );
        return;
    };

    ctx.commands.trigger(UpdateZIndexRequested {
        entity,
        z_index: pending_scalar_or(&ctx.world_signals, sig::GUI_EE_PENDING_Z_INDEX, z_index),
    });
}

fn consume_group_commit(ctx: &mut GameCtx, entity: Entity, snapshot: &ComponentSnapshot) {
    let Some(ref group) = snapshot.group else {
        warn!(
            "consume_group_commit: snapshot missing Group for entity {}",
            entity.to_bits()
        );
        return;
    };

    ctx.commands.trigger(UpdateGroupRequested {
        entity,
        group: pending_string_or(
            &ctx.world_signals,
            sig::GUI_EE_PENDING_GROUP,
            sig::GUI_EE_PENDING_GROUP_DIRTY,
            group,
        ),
    });
    ctx.world_signals
        .clear_flag(sig::GUI_EE_PENDING_GROUP_DIRTY);
}

fn consume_rotation_commit(ctx: &mut GameCtx, entity: Entity, snapshot: &ComponentSnapshot) {
    let Some(rotation_deg) = snapshot.rotation_deg else {
        warn!(
            "consume_rotation_commit: snapshot missing Rotation for entity {}",
            entity.to_bits()
        );
        return;
    };

    ctx.commands.trigger(UpdateRotationRequested {
        entity,
        rotation_deg: pending_scalar_or(
            &ctx.world_signals,
            sig::GUI_EE_PENDING_ROT_DEG,
            rotation_deg,
        ),
    });
}

fn consume_scale_commit(ctx: &mut GameCtx, entity: Entity, snapshot: &ComponentSnapshot) {
    let Some([scale_x, scale_y]) = snapshot.scale else {
        warn!(
            "consume_scale_commit: snapshot missing Scale for entity {}",
            entity.to_bits()
        );
        return;
    };

    ctx.commands.trigger(UpdateScaleRequested {
        entity,
        x: pending_scalar_or(&ctx.world_signals, sig::GUI_EE_PENDING_SCALE_X, scale_x),
        y: pending_scalar_or(&ctx.world_signals, sig::GUI_EE_PENDING_SCALE_Y, scale_y),
    });
}

fn consume_sprite_commit(ctx: &mut GameCtx, entity: Entity, snapshot: &ComponentSnapshot) {
    let Some(ref sprite) = snapshot.sprite else {
        warn!(
            "consume_sprite_commit: snapshot missing Sprite for entity {}",
            entity.to_bits()
        );
        return;
    };

    ctx.commands.trigger(UpdateSpriteRequested {
        entity,
        tex_key: pending_string_or(
            &ctx.world_signals,
            sig::GUI_EE_PENDING_SPRITE_TEX_KEY,
            sig::GUI_EE_PENDING_SPRITE_TEX_KEY_DIRTY,
            &sprite.tex_key,
        ),
        width: pending_scalar_or(
            &ctx.world_signals,
            sig::GUI_EE_PENDING_SPRITE_WIDTH,
            sprite.width,
        ),
        height: pending_scalar_or(
            &ctx.world_signals,
            sig::GUI_EE_PENDING_SPRITE_HEIGHT,
            sprite.height,
        ),
        offset: [
            pending_scalar_or(
                &ctx.world_signals,
                sig::GUI_EE_PENDING_SPRITE_OFFX,
                sprite.offset[0],
            ),
            pending_scalar_or(
                &ctx.world_signals,
                sig::GUI_EE_PENDING_SPRITE_OFFY,
                sprite.offset[1],
            ),
        ],
        origin: [
            pending_scalar_or(
                &ctx.world_signals,
                sig::GUI_EE_PENDING_SPRITE_ORGX,
                sprite.origin[0],
            ),
            pending_scalar_or(
                &ctx.world_signals,
                sig::GUI_EE_PENDING_SPRITE_ORGY,
                sprite.origin[1],
            ),
        ],
        flip_h: pending_bool_or(
            &ctx.world_signals,
            sig::GUI_EE_PENDING_SPRITE_FLIP_H,
            sig::GUI_EE_PENDING_SPRITE_FLIP_H_DIRTY,
            sprite.flip_h,
        ),
        flip_v: pending_bool_or(
            &ctx.world_signals,
            sig::GUI_EE_PENDING_SPRITE_FLIP_V,
            sig::GUI_EE_PENDING_SPRITE_FLIP_V_DIRTY,
            sprite.flip_v,
        ),
    });
    ctx.world_signals
        .clear_flag(sig::GUI_EE_PENDING_SPRITE_TEX_KEY_DIRTY);
    ctx.world_signals
        .clear_flag(sig::GUI_EE_PENDING_SPRITE_FLIP_H_DIRTY);
    ctx.world_signals
        .clear_flag(sig::GUI_EE_PENDING_SPRITE_FLIP_V_DIRTY);
}

fn consume_collider_commit(ctx: &mut GameCtx, entity: Entity, snapshot: &ComponentSnapshot) {
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
            pending_scalar_or(
                &ctx.world_signals,
                sig::GUI_EE_PENDING_BOX_SIZE_X,
                collider.size[0],
            ),
            pending_scalar_or(
                &ctx.world_signals,
                sig::GUI_EE_PENDING_BOX_SIZE_Y,
                collider.size[1],
            ),
        ],
        offset: [
            pending_scalar_or(
                &ctx.world_signals,
                sig::GUI_EE_PENDING_BOX_OFFX,
                collider.offset[0],
            ),
            pending_scalar_or(
                &ctx.world_signals,
                sig::GUI_EE_PENDING_BOX_OFFY,
                collider.offset[1],
            ),
        ],
        origin: [
            pending_scalar_or(
                &ctx.world_signals,
                sig::GUI_EE_PENDING_BOX_ORGX,
                collider.origin[0],
            ),
            pending_scalar_or(
                &ctx.world_signals,
                sig::GUI_EE_PENDING_BOX_ORGY,
                collider.origin[1],
            ),
        ],
    });
}

fn consume_animation_commit(ctx: &mut GameCtx, entity: Entity, snapshot: &ComponentSnapshot) {
    let Some(ref animation) = snapshot.animation else {
        warn!(
            "consume_animation_commit: snapshot missing Animation for entity {}",
            entity.to_bits()
        );
        return;
    };

    let frame_index = pending_integer_or(
        &ctx.world_signals,
        sig::GUI_EE_PENDING_ANIM_FRAME_INDEX,
        i32::try_from(animation.frame_index).unwrap_or(i32::MAX),
    )
    .max(0) as usize;

    ctx.commands.trigger(UpdateAnimationRequested {
        entity,
        animation_key: pending_string_or(
            &ctx.world_signals,
            sig::GUI_EE_PENDING_ANIM_KEY,
            sig::GUI_EE_PENDING_ANIM_KEY_DIRTY,
            &animation.animation_key,
        ),
        frame_index,
        elapsed_time: pending_scalar_or(
            &ctx.world_signals,
            sig::GUI_EE_PENDING_ANIM_ELAPSED,
            animation.elapsed_time,
        ),
    });
    ctx.world_signals
        .clear_flag(sig::GUI_EE_PENDING_ANIM_KEY_DIRTY);
}

fn selected_entity_and_snapshot(
    signals: &WorldSignals,
    app_state: &AppState,
) -> Option<(Entity, ComponentSnapshot)> {
    let entity = signals.get_entity(sig::ES_SELECTED_ENTITY).copied()?;
    let snapshot = app_state.get::<ComponentSnapshot>()?.clone();
    (snapshot.entity_bits == entity.to_bits()).then_some((entity, snapshot))
}

fn pending_scalar_or(signals: &WorldSignals, key: &str, fallback: f32) -> f32 {
    signals.get_scalar(key).unwrap_or(fallback)
}

fn pending_integer_or(signals: &WorldSignals, key: &str, fallback: i32) -> i32 {
    signals.get_integer(key).unwrap_or(fallback)
}

fn pending_string_or(signals: &WorldSignals, key: &str, dirty_key: &str, fallback: &str) -> String {
    if signals.has_flag(dirty_key) {
        signals
            .get_string(key)
            .cloned()
            .unwrap_or_else(|| fallback.to_owned())
    } else {
        fallback.to_owned()
    }
}

fn pending_bool_or(signals: &WorldSignals, key: &str, dirty_key: &str, fallback: bool) -> bool {
    if signals.has_flag(dirty_key) {
        signals.has_flag(key)
    } else {
        fallback
    }
}

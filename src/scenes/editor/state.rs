use aberredengine::bevy_ecs;
use aberredengine::bevy_ecs::prelude::Resource;
use crate::signals as sig;
use crate::systems::entity_edit::{
    UpdateAnimationRequested, UpdateBoxColliderRequested, UpdateGroupRequested,
    UpdateMapPositionRequested, UpdateRotationRequested, UpdateScaleRequested,
    UpdateSpriteRequested, UpdateZIndexRequested,
};
use crate::editor_types::ComponentSnapshot;
use aberredengine::bevy_ecs::prelude::{Entity, ResMut};
use aberredengine::imgui;
use aberredengine::resources::appstate::AppState;
use aberredengine::resources::worldsignals::WorldSignals;
use aberredengine::systems::GameCtx;
use log::warn;

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

pub(super) const BTN_W: f32 = 22.0;
pub(super) const BTN_SPACING: f32 = 4.0;
const MIN_NUMERIC_INPUT_W: f32 = 96.0;

fn split_imgui_label(label: &str) -> (&str, &str) {
    label.split_once("##").unwrap_or((label, ""))
}

fn hidden_numeric_label(label: &str) -> String {
    let (_, id_suffix) = split_imgui_label(label);
    if id_suffix.is_empty() {
        format!("##{label}")
    } else {
        format!("##{id_suffix}")
    }
}

fn numeric_input_width(ui: &imgui::Ui, visible_label: &str) -> f32 {
    let label_width = if visible_label.is_empty() {
        0.0
    } else {
        ui.calc_text_size(visible_label)[0] + BTN_SPACING
    };
    let reserved_width = BTN_W * 2.0 + BTN_SPACING * 2.0 + label_width;
    (ui.content_region_avail()[0] - reserved_width).max(MIN_NUMERIC_INPUT_W)
}

fn draw_trailing_numeric_label(ui: &imgui::Ui, visible_label: &str) {
    if visible_label.is_empty() {
        return;
    }

    ui.same_line_with_spacing(0.0, BTN_SPACING);
    ui.text(visible_label);
}

/// Renders − and + step buttons after the previously rendered widget (same line).
pub(super) fn draw_step_buttons(
    ui: &imgui::Ui,
    signals: &mut WorldSignals,
    pending_key: &str,
    value: f32,
    step: f32,
    action_key: &str,
) {
    ui.same_line_with_spacing(0.0, BTN_SPACING);
    if ui.button_with_size(format!("-##{pending_key}"), [BTN_W, 0.0]) {
        commit_scalar_signal(signals, pending_key, value - step, action_key);
    }
    ui.same_line_with_spacing(0.0, BTN_SPACING);
    if ui.button_with_size(format!("+##{pending_key}"), [BTN_W, 0.0]) {
        commit_scalar_signal(signals, pending_key, value + step, action_key);
    }
}

pub(super) fn draw_float_input(
    ui: &imgui::Ui,
    signals: &mut WorldSignals,
    label: &str,
    snapshot_value: f32,
    pending_key: &str,
    action_key: &str,
    step: f32,
) {
    let (visible_label, _) = split_imgui_label(label);
    let hidden_label = hidden_numeric_label(label);
    ui.set_next_item_width(numeric_input_width(ui, visible_label));
    let mut value = snapshot_value;
    ui.input_float(hidden_label.as_str(), &mut value)
        .enter_returns_true(true)
        .build();
    if ui.is_item_deactivated_after_edit() {
        commit_scalar_signal(signals, pending_key, value, action_key);
    }
    draw_step_buttons(ui, signals, pending_key, snapshot_value, step, action_key);
    draw_trailing_numeric_label(ui, visible_label);
}

pub(super) fn draw_drag_float_input(
    ui: &imgui::Ui,
    signals: &mut WorldSignals,
    label: &str,
    snapshot_value: f32,
    pending_key: &str,
    action_key: &str,
    step: f32,
    speed: f32,
) {
    let (visible_label, _) = split_imgui_label(label);
    let hidden_label = hidden_numeric_label(label);
    ui.set_next_item_width(numeric_input_width(ui, visible_label));
    let mut value = snapshot_value;
    imgui::Drag::new(hidden_label.as_str())
        .speed(speed)
        .display_format("%.2f")
        .build(ui, &mut value);
    if ui.is_item_deactivated_after_edit() {
        commit_scalar_signal(signals, pending_key, value, action_key);
    }
    draw_step_buttons(ui, signals, pending_key, snapshot_value, step, action_key);
    draw_trailing_numeric_label(ui, visible_label);
}

pub(super) fn draw_int_input(
    ui: &imgui::Ui,
    signals: &mut WorldSignals,
    label: &str,
    snapshot_value: i32,
    pending_key: &str,
    action_key: &str,
) {
    let (visible_label, _) = split_imgui_label(label);
    let hidden_label = hidden_numeric_label(label);
    ui.set_next_item_width(numeric_input_width(ui, visible_label));
    let mut value = snapshot_value;
    ui.input_int(hidden_label.as_str(), &mut value)
        .enter_returns_true(true)
        .build();
    if ui.is_item_deactivated_after_edit() {
        commit_integer_signal(signals, pending_key, value, action_key);
    }
    draw_trailing_numeric_label(ui, visible_label);
}

pub(super) fn draw_text_buffer_input(
    ui: &imgui::Ui,
    signals: &mut WorldSignals,
    label: &str,
    snapshot_value: &str,
    buffer_key: &str,
    dirty_key: &str,
    action_key: &str,
) {
    let mut buffer = seed_text_buffer(signals, buffer_key, dirty_key, snapshot_value);
    if ui.input_text(label, &mut buffer).build() {
        signals.set_string(buffer_key, buffer.as_str());
        signals.set_flag(dirty_key);
    }
    if ui.is_item_deactivated_after_edit() {
        signals.set_string(buffer_key, buffer.as_str());
        signals.set_flag(dirty_key);
        signals.set_flag(action_key);
    }
}

pub(super) fn seed_text_buffer(
    signals: &WorldSignals,
    buffer_key: &str,
    dirty_key: &str,
    snapshot_value: &str,
) -> String {
    if signals.has_flag(dirty_key) {
        signals
            .get_string(buffer_key)
            .cloned()
            .unwrap_or_else(|| snapshot_value.to_owned())
    } else {
        snapshot_value.to_owned()
    }
}

pub(super) fn commit_scalar_signal(
    signals: &mut WorldSignals,
    pending_key: &str,
    value: f32,
    action_key: &str,
) {
    signals.set_scalar(pending_key, value);
    signals.set_flag(action_key);
}

pub(super) fn commit_bool_flag(
    signals: &mut WorldSignals,
    pending_key: &str,
    dirty_key: &str,
    value: bool,
    action_key: &str,
) {
    if value {
        signals.set_flag(pending_key);
    } else {
        signals.clear_flag(pending_key);
    }
    signals.set_flag(dirty_key);
    signals.set_flag(action_key);
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

fn commit_integer_signal(
    signals: &mut WorldSignals,
    pending_key: &str,
    value: i32,
    action_key: &str,
) {
    signals.set_integer(pending_key, value);
    signals.set_flag(action_key);
}

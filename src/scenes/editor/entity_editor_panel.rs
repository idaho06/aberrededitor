use super::state::{
    commit_bool_flag, draw_drag_float_input, draw_float_input, draw_int_input,
    draw_text_buffer_input, seed_text_buffer,
};
use crate::editor_types::ComponentSnapshot;
use crate::signals as sig;
use aberredengine::imgui;
use aberredengine::resources::appstate::AppState;
use aberredengine::resources::texturestore::TextureStore;
use aberredengine::resources::worldsignals::WorldSignals;

pub(super) fn draw_entity_editor(
    ui: &imgui::Ui,
    signals: &mut WorldSignals,
    textures: &TextureStore,
    app_state: &AppState,
) {
    if !signals.has_flag(sig::UI_ENTITY_EDITOR_OPEN) {
        return;
    }

    let mut window_open = true;
    ui.window("Entity Inspector")
        .size([380.0, 420.0], imgui::Condition::FirstUseEver)
        .opened(&mut window_open)
        .build(|| {
            let Some(snap) = app_state.get::<ComponentSnapshot>().cloned() else {
                ui.text_disabled("No entity selected.");
                return;
            };

            ui.text(format!("Entity #{}", snap.entity_bits & 0xFFFF_FFFF));
            if snap.world_signal_keys.is_empty() {
                ui.text_disabled("  Not in WorldSignals");
            } else {
                ui.text_disabled(format!("  Keys: {}", snap.world_signal_keys.join(", ")));
            }
            ui.separator();

            ui.text("MapPosition");
            draw_drag_float_input(
                ui,
                signals,
                "x##map_position",
                snap.map_position[0],
                sig::GUI_EE_PENDING_POS_X,
                sig::ACTION_EE_COMMIT_POSITION,
                1.0,
                0.1,
            );
            draw_drag_float_input(
                ui,
                signals,
                "y##map_position",
                snap.map_position[1],
                sig::GUI_EE_PENDING_POS_Y,
                sig::ACTION_EE_COMMIT_POSITION,
                1.0,
                0.1,
            );

            if let Some(z) = snap.z_index {
                ui.separator();
                ui.text("ZIndex");
                draw_float_input(
                    ui,
                    signals,
                    "value##zindex",
                    z,
                    sig::GUI_EE_PENDING_Z_INDEX,
                    sig::ACTION_EE_COMMIT_Z,
                    1.0,
                );
            }
            if let Some(ref group) = snap.group {
                ui.separator();
                ui.text("Group");
                ui.set_next_item_width(-1.0);
                draw_text_buffer_input(
                    ui,
                    signals,
                    "name##group",
                    group,
                    sig::GUI_EE_PENDING_GROUP,
                    sig::GUI_EE_PENDING_GROUP_DIRTY,
                    sig::ACTION_EE_COMMIT_GROUP,
                );
            }
            if let Some(ref sprite) = snap.sprite {
                ui.separator();
                ui.text("Sprite");

                let sprite_tex_key = seed_text_buffer(
                    signals,
                    sig::GUI_EE_PENDING_SPRITE_TEX_KEY,
                    sig::GUI_EE_PENDING_SPRITE_TEX_KEY_DIRTY,
                    sprite.tex_key.as_str(),
                );
                let mut texture_keys: Vec<&str> =
                    textures.map.keys().map(|key| key.as_str()).collect();
                texture_keys.sort_unstable();
                if texture_keys.is_empty() {
                    ui.text_disabled("No textures loaded.");
                } else {
                    let mut current_tex = texture_keys
                        .iter()
                        .position(|key| *key == sprite_tex_key)
                        .or_else(|| {
                            texture_keys
                                .iter()
                                .position(|key| *key == sprite.tex_key.as_str())
                        })
                        .unwrap_or(0);
                    if ui.combo_simple_string("tex_key##sprite", &mut current_tex, &texture_keys) {
                        let selected = texture_keys[current_tex];
                        signals.set_string(sig::GUI_EE_PENDING_SPRITE_TEX_KEY, selected);
                        signals.set_flag(sig::GUI_EE_PENDING_SPRITE_TEX_KEY_DIRTY);
                        signals.set_flag(sig::ACTION_EE_COMMIT_SPRITE);
                    }
                }

                draw_float_input(
                    ui,
                    signals,
                    "width##sprite",
                    sprite.width,
                    sig::GUI_EE_PENDING_SPRITE_WIDTH,
                    sig::ACTION_EE_COMMIT_SPRITE,
                    1.0,
                );
                draw_float_input(
                    ui,
                    signals,
                    "height##sprite",
                    sprite.height,
                    sig::GUI_EE_PENDING_SPRITE_HEIGHT,
                    sig::ACTION_EE_COMMIT_SPRITE,
                    1.0,
                );
                draw_float_input(
                    ui,
                    signals,
                    "offset x##sprite",
                    sprite.offset[0],
                    sig::GUI_EE_PENDING_SPRITE_OFFX,
                    sig::ACTION_EE_COMMIT_SPRITE,
                    1.0,
                );
                draw_float_input(
                    ui,
                    signals,
                    "offset y##sprite",
                    sprite.offset[1],
                    sig::GUI_EE_PENDING_SPRITE_OFFY,
                    sig::ACTION_EE_COMMIT_SPRITE,
                    1.0,
                );
                draw_float_input(
                    ui,
                    signals,
                    "origin x##sprite",
                    sprite.origin[0],
                    sig::GUI_EE_PENDING_SPRITE_ORGX,
                    sig::ACTION_EE_COMMIT_SPRITE,
                    1.0,
                );
                draw_float_input(
                    ui,
                    signals,
                    "origin y##sprite",
                    sprite.origin[1],
                    sig::GUI_EE_PENDING_SPRITE_ORGY,
                    sig::ACTION_EE_COMMIT_SPRITE,
                    1.0,
                );

                let mut flip_h = sprite.flip_h;
                if ui.checkbox("flip_h##sprite", &mut flip_h) {
                    commit_bool_flag(
                        signals,
                        sig::GUI_EE_PENDING_SPRITE_FLIP_H,
                        sig::GUI_EE_PENDING_SPRITE_FLIP_H_DIRTY,
                        flip_h,
                        sig::ACTION_EE_COMMIT_SPRITE,
                    );
                }

                let mut flip_v = sprite.flip_v;
                if ui.checkbox("flip_v##sprite", &mut flip_v) {
                    commit_bool_flag(
                        signals,
                        sig::GUI_EE_PENDING_SPRITE_FLIP_V,
                        sig::GUI_EE_PENDING_SPRITE_FLIP_V_DIRTY,
                        flip_v,
                        sig::ACTION_EE_COMMIT_SPRITE,
                    );
                }
            }
            if let Some(ref collider) = snap.box_collider {
                ui.separator();
                ui.text("BoxCollider");
                draw_float_input(
                    ui,
                    signals,
                    "size x##collider",
                    collider.size[0],
                    sig::GUI_EE_PENDING_BOX_SIZE_X,
                    sig::ACTION_EE_COMMIT_COLLIDER,
                    1.0,
                );
                draw_float_input(
                    ui,
                    signals,
                    "size y##collider",
                    collider.size[1],
                    sig::GUI_EE_PENDING_BOX_SIZE_Y,
                    sig::ACTION_EE_COMMIT_COLLIDER,
                    1.0,
                );
                draw_float_input(
                    ui,
                    signals,
                    "offset x##collider",
                    collider.offset[0],
                    sig::GUI_EE_PENDING_BOX_OFFX,
                    sig::ACTION_EE_COMMIT_COLLIDER,
                    1.0,
                );
                draw_float_input(
                    ui,
                    signals,
                    "offset y##collider",
                    collider.offset[1],
                    sig::GUI_EE_PENDING_BOX_OFFY,
                    sig::ACTION_EE_COMMIT_COLLIDER,
                    1.0,
                );
                draw_float_input(
                    ui,
                    signals,
                    "origin x##collider",
                    collider.origin[0],
                    sig::GUI_EE_PENDING_BOX_ORGX,
                    sig::ACTION_EE_COMMIT_COLLIDER,
                    1.0,
                );
                draw_float_input(
                    ui,
                    signals,
                    "origin y##collider",
                    collider.origin[1],
                    sig::GUI_EE_PENDING_BOX_ORGY,
                    sig::ACTION_EE_COMMIT_COLLIDER,
                    1.0,
                );
            }
            if let Some(rotation_deg) = snap.rotation_deg {
                ui.separator();
                ui.text("Rotation");
                draw_float_input(
                    ui,
                    signals,
                    "degrees##rotation",
                    rotation_deg,
                    sig::GUI_EE_PENDING_ROT_DEG,
                    sig::ACTION_EE_COMMIT_ROTATION,
                    1.0,
                );
            }
            if let Some([scale_x, scale_y]) = snap.scale {
                ui.separator();
                ui.text("Scale");
                draw_float_input(
                    ui,
                    signals,
                    "x##scale",
                    scale_x,
                    sig::GUI_EE_PENDING_SCALE_X,
                    sig::ACTION_EE_COMMIT_SCALE,
                    1.0,
                );
                draw_float_input(
                    ui,
                    signals,
                    "y##scale",
                    scale_y,
                    sig::GUI_EE_PENDING_SCALE_Y,
                    sig::ACTION_EE_COMMIT_SCALE,
                    1.0,
                );
            }
            if let Some(ref animation) = snap.animation {
                ui.separator();
                ui.text("Animation");
                ui.set_next_item_width(-1.0);
                draw_text_buffer_input(
                    ui,
                    signals,
                    "key##animation",
                    animation.animation_key.as_str(),
                    sig::GUI_EE_PENDING_ANIM_KEY,
                    sig::GUI_EE_PENDING_ANIM_KEY_DIRTY,
                    sig::ACTION_EE_COMMIT_ANIMATION,
                );
                draw_int_input(
                    ui,
                    signals,
                    "frame_index##animation",
                    i32::try_from(animation.frame_index).unwrap_or(i32::MAX),
                    sig::GUI_EE_PENDING_ANIM_FRAME_INDEX,
                    sig::ACTION_EE_COMMIT_ANIMATION,
                );
                draw_float_input(
                    ui,
                    signals,
                    "elapsed_time##animation",
                    animation.elapsed_time,
                    sig::GUI_EE_PENDING_ANIM_ELAPSED,
                    sig::ACTION_EE_COMMIT_ANIMATION,
                    1.0,
                );
            }
            if let Some(ref ttl) = snap.ttl {
                ui.separator();
                ui.text("Ttl");
                ui.group(|| ui.text_disabled(format!("  remaining: {:.3}", ttl.remaining)));
            }
            if let Some(ref timer) = snap.timer {
                ui.separator();
                ui.text("Timer");
                ui.group(|| {
                    ui.text_disabled(format!("  duration: {:.3}", timer.duration));
                    ui.text_disabled(format!("  elapsed: {:.3}", timer.elapsed));
                });
            }
            if let Some(ref phase) = snap.phase {
                ui.separator();
                ui.text("Phase");
                ui.group(|| {
                    ui.text_disabled(format!("  current: {}", phase.current));
                    ui.text_disabled(format!(
                        "  previous: {}",
                        phase.previous.as_deref().unwrap_or("(none)")
                    ));
                    ui.text_disabled(format!(
                        "  next: {}",
                        phase.next.as_deref().unwrap_or("(none)")
                    ));
                    ui.text_disabled(format!("  time_in_phase: {:.3}", phase.time_in_phase));
                    if phase.phase_names.is_empty() {
                        ui.text_disabled("  phase_names: (none)");
                    } else {
                        ui.text_disabled(format!(
                            "  phase_names: {}",
                            phase.phase_names.join(", ")
                        ));
                    }
                });
            }
        });

    if !window_open {
        signals.take_flag(sig::UI_ENTITY_EDITOR_OPEN);
    }
}

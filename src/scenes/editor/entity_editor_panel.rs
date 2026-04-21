use super::pending_state::PendingMutex;
use super::widgets::{draw_drag_float_input, draw_float_input, draw_int_input, draw_text_buffer_input};
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

    let Some(mutex) = app_state.get::<PendingMutex>() else {
        return;
    };

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

            let mut p = mutex.lock().unwrap();

            ui.text("MapPosition");
            if let Some(v) = draw_drag_float_input(ui, "x##map_position", p.pos_x.unwrap_or(snap.map_position[0]), 1.0, 0.1) {
                p.pos_x = Some(v);
                p.commit_position = true;
            }
            if let Some(v) = draw_drag_float_input(ui, "y##map_position", p.pos_y.unwrap_or(snap.map_position[1]), 1.0, 0.1) {
                p.pos_y = Some(v);
                p.commit_position = true;
            }

            if let Some(z) = snap.z_index {
                ui.separator();
                ui.text("ZIndex");
                if let Some(v) = draw_float_input(ui, "value##zindex", p.z_index.unwrap_or(z), 1.0) {
                    p.z_index = Some(v);
                    p.commit_z = true;
                }
            }
            if let Some(ref group) = snap.group {
                ui.separator();
                ui.text("Group");
                ui.set_next_item_width(-1.0);
                let mut committed = false;
                draw_text_buffer_input(ui, "name##group", &mut p.group, &mut committed, group);
                if committed {
                    p.commit_group = true;
                }
            }
            if let Some(ref sprite) = snap.sprite {
                ui.separator();
                ui.text("Sprite");

                let tex_key_current = p.sprite_tex_key.clone().unwrap_or_else(|| sprite.tex_key.clone());
                let mut texture_keys: Vec<&str> =
                    textures.map.keys().map(|key| key.as_str()).collect();
                texture_keys.sort_unstable();
                if texture_keys.is_empty() {
                    ui.text_disabled("No textures loaded.");
                } else {
                    let mut current_tex = texture_keys
                        .iter()
                        .position(|key| *key == tex_key_current.as_str())
                        .or_else(|| texture_keys.iter().position(|key| *key == sprite.tex_key.as_str()))
                        .unwrap_or(0);
                    if ui.combo_simple_string("tex_key##sprite", &mut current_tex, &texture_keys) {
                        p.sprite_tex_key = Some(texture_keys[current_tex].to_owned());
                        p.commit_sprite = true;
                    }
                }

                if let Some(v) = draw_float_input(ui, "width##sprite", p.sprite_width.unwrap_or(sprite.width), 1.0) {
                    p.sprite_width = Some(v);
                    p.commit_sprite = true;
                }
                if let Some(v) = draw_float_input(ui, "height##sprite", p.sprite_height.unwrap_or(sprite.height), 1.0) {
                    p.sprite_height = Some(v);
                    p.commit_sprite = true;
                }
                if let Some(v) = draw_float_input(ui, "offset x##sprite", p.sprite_off_x.unwrap_or(sprite.offset[0]), 1.0) {
                    p.sprite_off_x = Some(v);
                    p.commit_sprite = true;
                }
                if let Some(v) = draw_float_input(ui, "offset y##sprite", p.sprite_off_y.unwrap_or(sprite.offset[1]), 1.0) {
                    p.sprite_off_y = Some(v);
                    p.commit_sprite = true;
                }
                if let Some(v) = draw_float_input(ui, "origin x##sprite", p.sprite_org_x.unwrap_or(sprite.origin[0]), 1.0) {
                    p.sprite_org_x = Some(v);
                    p.commit_sprite = true;
                }
                if let Some(v) = draw_float_input(ui, "origin y##sprite", p.sprite_org_y.unwrap_or(sprite.origin[1]), 1.0) {
                    p.sprite_org_y = Some(v);
                    p.commit_sprite = true;
                }

                let mut flip_h = p.sprite_flip_h.unwrap_or(sprite.flip_h);
                if ui.checkbox("flip_h##sprite", &mut flip_h) {
                    p.sprite_flip_h = Some(flip_h);
                    p.commit_sprite = true;
                }

                let mut flip_v = p.sprite_flip_v.unwrap_or(sprite.flip_v);
                if ui.checkbox("flip_v##sprite", &mut flip_v) {
                    p.sprite_flip_v = Some(flip_v);
                    p.commit_sprite = true;
                }
            }
            if let Some(ref collider) = snap.box_collider {
                ui.separator();
                ui.text("BoxCollider");
                if let Some(v) = draw_float_input(ui, "size x##collider", p.box_size_x.unwrap_or(collider.size[0]), 1.0) {
                    p.box_size_x = Some(v);
                    p.commit_collider = true;
                }
                if let Some(v) = draw_float_input(ui, "size y##collider", p.box_size_y.unwrap_or(collider.size[1]), 1.0) {
                    p.box_size_y = Some(v);
                    p.commit_collider = true;
                }
                if let Some(v) = draw_float_input(ui, "offset x##collider", p.box_off_x.unwrap_or(collider.offset[0]), 1.0) {
                    p.box_off_x = Some(v);
                    p.commit_collider = true;
                }
                if let Some(v) = draw_float_input(ui, "offset y##collider", p.box_off_y.unwrap_or(collider.offset[1]), 1.0) {
                    p.box_off_y = Some(v);
                    p.commit_collider = true;
                }
                if let Some(v) = draw_float_input(ui, "origin x##collider", p.box_org_x.unwrap_or(collider.origin[0]), 1.0) {
                    p.box_org_x = Some(v);
                    p.commit_collider = true;
                }
                if let Some(v) = draw_float_input(ui, "origin y##collider", p.box_org_y.unwrap_or(collider.origin[1]), 1.0) {
                    p.box_org_y = Some(v);
                    p.commit_collider = true;
                }
            }
            if let Some(rotation_deg) = snap.rotation_deg {
                ui.separator();
                ui.text("Rotation");
                if let Some(v) = draw_float_input(ui, "degrees##rotation", p.rotation_deg.unwrap_or(rotation_deg), 1.0) {
                    p.rotation_deg = Some(v);
                    p.commit_rotation = true;
                }
            }
            if let Some([scale_x, scale_y]) = snap.scale {
                ui.separator();
                ui.text("Scale");
                if let Some(v) = draw_float_input(ui, "x##scale", p.scale_x.unwrap_or(scale_x), 1.0) {
                    p.scale_x = Some(v);
                    p.commit_scale = true;
                }
                if let Some(v) = draw_float_input(ui, "y##scale", p.scale_y.unwrap_or(scale_y), 1.0) {
                    p.scale_y = Some(v);
                    p.commit_scale = true;
                }
            }
            if let Some(ref animation) = snap.animation {
                ui.separator();
                ui.text("Animation");
                ui.set_next_item_width(-1.0);
                let mut committed = false;
                draw_text_buffer_input(ui, "key##animation", &mut p.anim_key, &mut committed, animation.animation_key.as_str());
                if committed {
                    p.commit_animation = true;
                }
                let frame_current = p.anim_frame_index.unwrap_or_else(|| i32::try_from(animation.frame_index).unwrap_or(i32::MAX));
                if let Some(v) = draw_int_input(ui, "frame_index##animation", frame_current) {
                    p.anim_frame_index = Some(v);
                    p.commit_animation = true;
                }
                if let Some(v) = draw_float_input(ui, "elapsed_time##animation", p.anim_elapsed.unwrap_or(animation.elapsed_time), 1.0) {
                    p.anim_elapsed = Some(v);
                    p.commit_animation = true;
                }
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

use std::sync::Arc;

use aberredengine::imgui;
use aberredengine::resources::animationstore::AnimationResource;
use aberredengine::resources::appstate::AppState;
use aberredengine::resources::texturestore::TextureStore;
use aberredengine::resources::worldsignals::WorldSignals;

use crate::scenes::editor::texture_viewer_panel::open_texture_viewer;
use crate::scenes::editor::widgets::{draw_float_input, draw_int_input};
use crate::signals as sig;
use crate::systems::animation_store_sync::AnimationStoreMutex;

pub(super) fn draw_animation_store(
    ui: &imgui::Ui,
    signals: &mut WorldSignals,
    textures: &TextureStore,
    app_state: &AppState,
) -> (bool, bool) {
    if !signals.has_flag(sig::UI_ANIMATION_STORE_OPEN) {
        return (false, false);
    }

    let Some(mutex) = app_state.get::<AnimationStoreMutex>() else {
        return (false, false);
    };

    let mut open_rename_popup = false;
    let mut open_remove_popup = false;
    let mut window_open = true;

    // Collect a snapshot of keys+resources outside the window closure to avoid
    // holding the lock across imgui calls.
    let entries: Vec<(String, AnimationResource)> = {
        let cache = mutex.lock().unwrap();
        let mut v: Vec<_> = cache
            .iter()
            .map(|(k, r)| (k.clone(), r.clone()))
            .collect();
        v.sort_by(|a, b| a.0.cmp(&b.0));
        v
    };

    let mut texture_keys: Vec<String> = textures.map.keys().cloned().collect();
    texture_keys.sort_unstable();
    let tex_key_strs: Vec<&str> = texture_keys.iter().map(|s| s.as_str()).collect();

    ui.window("Animation Store")
        .size([580.0, 600.0], imgui::Condition::FirstUseEver)
        .opened(&mut window_open)
        .build(|| {
            ui.child_window("##anim_list").size([0.0, -60.0]).build(|| {
                if entries.is_empty() {
                    ui.text_disabled("No animations defined.");
                }

                for (key, resource) in &entries {
                    let _id = ui.push_id(key.as_str());

                    // --- Texture thumbnail ---
                    let tex_lookup = textures.map.get(resource.tex_key.as_ref());
                    if let Some(texture) = tex_lookup {
                        let tex = texture.as_ref();
                        let tex_id = imgui::TextureId::from(tex as *const _ as usize);
                        imgui::Image::new(tex_id, [64.0, 64.0]).build(ui);
                        if ui.is_item_clicked() {
                            open_texture_viewer(
                                signals,
                                sig::TEXTURE_VIEWER_SOURCE_ANIMATION,
                                resource.tex_key.as_ref(),
                            );
                        }
                    } else {
                        ui.dummy([64.0, 64.0]);
                        if ui.is_item_hovered() {
                            ui.tooltip_text("Texture not loaded");
                        }
                    }

                    ui.same_line();

                    // --- Editable fields ---
                    let mut changed_resource: Option<AnimationResource> = None;

                    ui.group(|| {
                        // Key + Rename / Remove
                        ui.text(key.as_str());
                        ui.same_line();
                        if ui.small_button("Rename##anim_rename") {
                            signals.set_string(sig::ANIM_RENAME_SRC, key.as_str());
                            signals.set_string(sig::ANIM_RENAME_BUF, key.as_str());
                            open_rename_popup = true;
                        }
                        ui.same_line();
                        if ui.small_button("Remove##anim_remove") {
                            signals.set_string(sig::ANIM_REMOVE_KEY, key.as_str());
                            open_remove_popup = true;
                        }

                        // tex_key combo
                        let mut current_tex = tex_key_strs
                            .iter()
                            .position(|k| *k == resource.tex_key.as_ref())
                            .unwrap_or(0);
                        ui.set_next_item_width(200.0);
                        if !tex_key_strs.is_empty()
                            && ui.combo_simple_string(
                                "tex_key##anim",
                                &mut current_tex,
                                &tex_key_strs,
                            )
                        {
                            let mut r = resource.clone();
                            r.tex_key = Arc::from(tex_key_strs[current_tex]);
                            changed_resource = Some(r);
                        } else if tex_key_strs.is_empty() {
                            ui.text_disabled("(no textures loaded)");
                        }

                        // Numeric fields
                        if let Some(v) =
                            draw_float_input(ui, "pos x##anim", resource.position.x, 1.0)
                        {
                            let mut r = changed_resource.take().unwrap_or_else(|| resource.clone());
                            r.position.x = v;
                            changed_resource = Some(r);
                        }
                        if let Some(v) =
                            draw_float_input(ui, "pos y##anim", resource.position.y, 1.0)
                        {
                            let mut r = changed_resource.take().unwrap_or_else(|| resource.clone());
                            r.position.y = v;
                            changed_resource = Some(r);
                        }
                        if let Some(v) = draw_float_input(
                            ui,
                            "h_displacement##anim",
                            resource.horizontal_displacement,
                            1.0,
                        ) {
                            let mut r = changed_resource.take().unwrap_or_else(|| resource.clone());
                            r.horizontal_displacement = v;
                            changed_resource = Some(r);
                        }
                        if let Some(v) = draw_float_input(
                            ui,
                            "v_displacement##anim",
                            resource.vertical_displacement,
                            1.0,
                        ) {
                            let mut r = changed_resource.take().unwrap_or_else(|| resource.clone());
                            r.vertical_displacement = v;
                            changed_resource = Some(r);
                        }
                        if let Some(v) = draw_int_input(
                            ui,
                            "frame_count##anim",
                            resource.frame_count as i32,
                        ) {
                            let mut r = changed_resource.take().unwrap_or_else(|| resource.clone());
                            r.frame_count = v.max(1) as usize;
                            changed_resource = Some(r);
                        }
                        if let Some(v) = draw_float_input(ui, "fps##anim", resource.fps, 1.0) {
                            let mut r = changed_resource.take().unwrap_or_else(|| resource.clone());
                            r.fps = v.max(0.0);
                            changed_resource = Some(r);
                        }

                        let mut looped = resource.looped;
                        if ui.checkbox("looped##anim", &mut looped) {
                            let mut r = changed_resource.take().unwrap_or_else(|| resource.clone());
                            r.looped = looped;
                            changed_resource = Some(r);
                        }
                    });

                    // Commit any field change: write back to cache and set action flag.
                    if let Some(updated) = changed_resource {
                        mutex.lock().unwrap().insert(key.clone(), updated);
                        signals.set_string(sig::ANIM_UPDATE_KEY, key.as_str());
                        signals.set_flag(sig::ACTION_ANIM_UPDATE);
                    }

                    ui.separator();
                }
            });

            // --- Add section ---
            ui.separator();
            ui.text("Add animation");
            ui.same_line();
            let mut add_key = signals
                .get_string(sig::ANIM_ADD_KEY_BUF)
                .cloned()
                .unwrap_or_default();
            ui.set_next_item_width(ui.content_region_avail()[0] - 50.0);
            if ui.input_text("##anim_add_key", &mut add_key).build() {
                signals.set_string(sig::ANIM_ADD_KEY_BUF, add_key.as_str());
            }
            ui.same_line();
            if ui.button("Add##anim_add") && !add_key.is_empty() {
                signals.set_flag(sig::ACTION_ANIM_ADD);
            }
        });

    if !window_open {
        signals.take_flag(sig::UI_ANIMATION_STORE_OPEN);
    }

    (open_rename_popup, open_remove_popup)
}

pub(super) fn draw_animation_modals(ui: &imgui::Ui, signals: &mut WorldSignals) {
    ui.modal_popup_config("Rename Key##animation_store")
        .always_auto_resize(true)
        .resizable(false)
        .movable(false)
        .build(|| {
            let src = signals
                .get_string(sig::ANIM_RENAME_SRC)
                .cloned()
                .unwrap_or_default();
            ui.text(format!("Old key: {src}"));
            ui.spacing();

            let mut buf = signals
                .get_string(sig::ANIM_RENAME_BUF)
                .cloned()
                .unwrap_or_default();
            if ui
                .input_text("New key##anim_rename_input", &mut buf)
                .build()
            {
                signals.set_string(sig::ANIM_RENAME_BUF, buf.as_str());
            }

            ui.spacing();
            ui.separator();
            if ui.button("OK##anim_rename_ok") {
                signals.set_flag(sig::ACTION_ANIM_RENAME);
                ui.close_current_popup();
            }
            ui.same_line();
            if ui.button("Cancel##anim_rename_cancel") {
                ui.close_current_popup();
            }
        });

    ui.modal_popup_config("Remove Animation##animation_store")
        .always_auto_resize(true)
        .resizable(false)
        .movable(false)
        .build(|| {
            let key = signals
                .get_string(sig::ANIM_REMOVE_KEY)
                .cloned()
                .unwrap_or_default();
            ui.text(format!("Remove \"{key}\"?"));
            ui.spacing();
            ui.separator();
            if ui.button("Yes##anim_remove_yes") {
                signals.set_flag(sig::ACTION_ANIM_REMOVE);
                ui.close_current_popup();
            }
            ui.same_line();
            if ui.button("No##anim_remove_no") {
                ui.close_current_popup();
            }
        });
}


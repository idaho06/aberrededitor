use super::texture_viewer_panel::open_texture_viewer;
use crate::signals as sig;
use aberredengine::imgui;
use aberredengine::resources::texturestore::TextureStore;
use aberredengine::resources::worldsignals::WorldSignals;

pub(super) fn draw_texture_editor(
    ui: &imgui::Ui,
    signals: &mut WorldSignals,
    textures: &TextureStore,
) -> (bool, bool) {
    if !signals.has_flag(sig::UI_TEXTURE_EDITOR_OPEN) {
        return (false, false);
    }

    let mut open_rename_popup = false;
    let mut open_remove_popup = false;
    let mut window_open = true;

    ui.window("Texture Store")
        .size([460.0, 520.0], imgui::Condition::FirstUseEver)
        .opened(&mut window_open)
        .build(|| {
            ui.child_window("##texture_list")
                .size([0.0, -65.0])
                .build(|| {
                    let mut sorted_keys: Vec<&String> = textures.map.keys().collect();
                    sorted_keys.sort();

                    if sorted_keys.is_empty() {
                        ui.text_disabled("No textures loaded.");
                    }

                    for key in &sorted_keys {
                        let Some(texture) = textures.map.get(key.as_str()) else {
                            continue;
                        };

                        let tex = texture.as_ref();
                        let tex_id = imgui::TextureId::from(tex as *const _ as usize);

                        imgui::Image::new(tex_id, [96.0f32, 96.0]).build(ui);
                        if ui.is_item_clicked() {
                            open_texture_viewer(
                                signals,
                                sig::TEXTURE_VIEWER_SOURCE_TEXTURE,
                                key.as_str(),
                            );
                        }
                        ui.same_line();

                        let _id = ui.push_id(key.as_str());
                        ui.group(|| {
                            ui.text(key.as_str());
                            ui.text_disabled(format!("{}×{}", tex.width, tex.height));
                            if let Some(path) = textures.paths.get(key.as_str()) {
                                let filename = std::path::Path::new(path)
                                    .file_name()
                                    .and_then(|name| name.to_str())
                                    .unwrap_or(path.as_str());
                                ui.text_disabled(filename);
                                if ui.is_item_hovered() {
                                    ui.tooltip_text(path.as_str());
                                }
                            }
                            if ui.small_button("Rename") {
                                signals.set_string(sig::TEX_RENAME_SRC, key.as_str());
                                signals.set_string(sig::TEX_RENAME_BUF, key.as_str());
                                open_rename_popup = true;
                            }
                            ui.same_line();
                            if ui.small_button("Remove") {
                                signals.set_string(sig::TEX_REMOVE_KEY, key.as_str());
                                open_remove_popup = true;
                            }
                        });
                    }
                });

            ui.separator();
            ui.text("Add texture");
            ui.text("Key:");
            ui.same_line();

            let mut add_key = signals
                .get_string(sig::TEX_ADD_KEY_BUF)
                .cloned()
                .unwrap_or_default();
            ui.set_next_item_width(ui.content_region_avail()[0] - 85.0);
            if ui.input_text("##add_key", &mut add_key).build() {
                signals.set_string(sig::TEX_ADD_KEY_BUF, add_key.as_str());
            }
            ui.same_line();
            if ui.button("Browse...##add") {
                signals.set_flag(sig::ACTION_TEXTURE_ADD_BROWSE);
            }
        });

    if !window_open {
        signals.take_flag(sig::UI_TEXTURE_EDITOR_OPEN);
    }

    (open_rename_popup, open_remove_popup)
}

pub(super) fn draw_texture_modals(ui: &imgui::Ui, signals: &mut WorldSignals) {
    ui.modal_popup_config("Rename Key##texture_editor")
        .always_auto_resize(true)
        .resizable(false)
        .movable(false)
        .build(|| {
            let src = signals
                .get_string(sig::TEX_RENAME_SRC)
                .cloned()
                .unwrap_or_default();
            ui.text(format!("Old key: {src}"));
            ui.spacing();

            let mut buf = signals
                .get_string(sig::TEX_RENAME_BUF)
                .cloned()
                .unwrap_or_default();
            if ui.input_text("New key##rename_input", &mut buf).build() {
                signals.set_string(sig::TEX_RENAME_BUF, buf.as_str());
            }

            ui.spacing();
            ui.separator();
            if ui.button("OK##rename_ok") {
                signals.set_flag(sig::ACTION_TEXTURE_RENAME);
                ui.close_current_popup();
            }
            ui.same_line();
            if ui.button("Cancel##rename_cancel") {
                ui.close_current_popup();
            }
        });

    ui.modal_popup_config("Remove Texture##texture_editor")
        .always_auto_resize(true)
        .resizable(false)
        .movable(false)
        .build(|| {
            let key = signals
                .get_string(sig::TEX_REMOVE_KEY)
                .cloned()
                .unwrap_or_default();
            ui.text(format!("Remove \"{key}\"?"));
            ui.text_disabled("This also unloads the texture from GPU memory.");
            ui.spacing();
            ui.separator();
            if ui.button("Yes##remove_yes") {
                signals.set_flag(sig::ACTION_TEXTURE_REMOVE);
                ui.close_current_popup();
            }
            ui.same_line();
            if ui.button("No##remove_no") {
                ui.close_current_popup();
            }
        });
}

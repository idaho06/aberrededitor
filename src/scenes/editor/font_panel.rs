//! Font store editor panel.
//!
//! `draw_font_editor` renders the font list window with add/rename/remove controls.
//! Returns two booleans indicating whether the rename or remove modal should be opened;
//! `editor_gui` in `update.rs` calls `ui.open_popup` based on those.
//! `draw_font_modals` renders both modal popups.
//!
//! Reads `FontStore` directly (passed as `&FontStore` in `GuiCallback`).
use super::texture_viewer_panel::open_texture_viewer;
use crate::signals as sig;
use aberredengine::imgui;
use aberredengine::resources::fontstore::FontStore;
use aberredengine::resources::worldsignals::WorldSignals;

pub(super) fn draw_font_editor(
    ui: &imgui::Ui,
    signals: &mut WorldSignals,
    fonts: &FontStore,
) -> (bool, bool) {
    if !signals.has_flag(sig::UI_FONT_STORE_OPEN) {
        return (false, false);
    }

    let mut open_rename_popup = false;
    let mut open_remove_popup = false;
    let mut window_open = true;

    ui.window("Font Store")
        .size([460.0, 520.0], imgui::Condition::FirstUseEver)
        .opened(&mut window_open)
        .build(|| {
            ui.child_window("##font_list").size([0.0, -80.0]).build(|| {
                let mut sorted_keys: Vec<&String> = fonts.meta.keys().collect();
                sorted_keys.sort();

                if sorted_keys.is_empty() {
                    ui.text_disabled("No fonts loaded.");
                }

                for key in &sorted_keys {
                    let Some(font) = fonts.get(key.as_str()) else {
                        continue;
                    };
                    let Some(meta) = fonts.meta.get(key.as_str()) else {
                        continue;
                    };

                    // rlImGui dereferences the pointer as a full ffi::Texture2D — passing .id crashes.
                    let tex_id = imgui::TextureId::from(&font.texture as *const _ as usize);
                    imgui::Image::new(tex_id, [64.0f32, 64.0]).build(ui);
                    if ui.is_item_clicked() {
                        open_texture_viewer(
                            signals,
                            sig::TEXTURE_VIEWER_SOURCE_FONT,
                            key.as_str(),
                        );
                    }
                    ui.same_line();

                    let _id = ui.push_id(key.as_str());
                    ui.group(|| {
                        ui.text(key.as_str());
                        let filename = std::path::Path::new(&meta.path)
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or(meta.path.as_str());
                        ui.text_disabled(filename);
                        if ui.is_item_hovered() {
                            ui.tooltip_text(meta.path.as_str());
                        }
                        ui.text_disabled(format!("size: {}", meta.font_size));
                        if ui.small_button("Rename") {
                            signals.set_string(sig::FONT_RENAME_SRC, key.as_str());
                            signals.set_string(sig::FONT_RENAME_BUF, key.as_str());
                            open_rename_popup = true;
                        }
                        ui.same_line();
                        if ui.small_button("Remove") {
                            signals.set_string(sig::FONT_REMOVE_KEY, key.as_str());
                            open_remove_popup = true;
                        }
                    });
                }
            });

            ui.separator();
            ui.text("Add font");
            ui.text("Key:");
            ui.same_line();

            let mut add_key = signals
                .get_string(sig::FONT_ADD_KEY_BUF)
                .cloned()
                .unwrap_or_default();
            ui.set_next_item_width(ui.content_region_avail()[0] - 85.0);
            if ui.input_text("##font_add_key", &mut add_key).build() {
                signals.set_string(sig::FONT_ADD_KEY_BUF, add_key.as_str());
            }
            ui.same_line();
            if ui.button("Browse...##font_add") {
                signals.set_flag(sig::ACTION_FONT_ADD_BROWSE);
            }

            ui.text("Size:");
            ui.same_line();
            let mut add_size = signals.get_scalar(sig::FONT_ADD_SIZE_BUF).unwrap_or(32.0);
            ui.set_next_item_width(80.0);
            if ui
                .input_float("##font_add_size", &mut add_size)
                .step(1.0)
                .build()
            {
                signals.set_scalar(sig::FONT_ADD_SIZE_BUF, add_size);
            }
        });

    if !window_open {
        signals.take_flag(sig::UI_FONT_STORE_OPEN);
    }

    (open_rename_popup, open_remove_popup)
}

pub(super) fn draw_font_modals(ui: &imgui::Ui, signals: &mut WorldSignals) {
    ui.modal_popup_config("Rename Key##font_store")
        .always_auto_resize(true)
        .resizable(false)
        .movable(false)
        .build(|| {
            let src = signals
                .get_string(sig::FONT_RENAME_SRC)
                .cloned()
                .unwrap_or_default();
            ui.text(format!("Old key: {src}"));
            ui.spacing();

            let mut buf = signals
                .get_string(sig::FONT_RENAME_BUF)
                .cloned()
                .unwrap_or_default();
            if ui
                .input_text("New key##font_rename_input", &mut buf)
                .build()
            {
                signals.set_string(sig::FONT_RENAME_BUF, buf.as_str());
            }

            ui.spacing();
            ui.separator();
            if ui.button("OK##font_rename_ok") {
                signals.set_flag(sig::ACTION_FONT_RENAME);
                ui.close_current_popup();
            }
            ui.same_line();
            if ui.button("Cancel##font_rename_cancel") {
                ui.close_current_popup();
            }
        });

    ui.modal_popup_config("Remove Font##font_store")
        .always_auto_resize(true)
        .resizable(false)
        .movable(false)
        .build(|| {
            let key = signals
                .get_string(sig::FONT_REMOVE_KEY)
                .cloned()
                .unwrap_or_default();
            ui.text(format!("Remove \"{key}\"?"));
            ui.text_disabled("This also unloads the font from memory.");
            ui.spacing();
            ui.separator();
            if ui.button("Yes##font_remove_yes") {
                signals.set_flag(sig::ACTION_FONT_REMOVE);
                ui.close_current_popup();
            }
            ui.same_line();
            if ui.button("No##font_remove_no") {
                ui.close_current_popup();
            }
        });
}

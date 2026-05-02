use crate::signals as sig;
use aberredengine::imgui;
use aberredengine::resources::worldsignals::WorldSignals;

pub(super) fn draw_menu_bar(ui: &imgui::Ui, signals: &mut WorldSignals) -> bool {
    let mut open_about = false;
    if let Some(_mb) = ui.begin_main_menu_bar() {
        if let Some(_file) = ui.begin_menu("File") {
            if ui.menu_item("New Map") {
                signals.set_flag(sig::ACTION_FILE_NEW_MAP);
            }
            if ui.menu_item("Open Map...") {
                signals.set_flag(sig::ACTION_FILE_OPEN_MAP);
            }
            ui.separator();
            if ui.menu_item("Add Tilemap...") {
                signals.set_flag(sig::ACTION_FILE_LOAD_TILEMAP);
            }
            ui.separator();
            if ui.menu_item("Save Map") {
                signals.set_flag(sig::ACTION_FILE_SAVE);
            }
            if ui.menu_item("Save Map As...") {
                signals.set_flag(sig::ACTION_FILE_SAVE_AS);
            }
        }

        if let Some(_view) = ui.begin_menu("View") {
            if ui.menu_item("Reset Zoom") {
                signals.set_flag(sig::ACTION_VIEW_RESET_ZOOM);
            }
            if ui
                .menu_item_config("Toggle Debug Mode")
                .shortcut("F11")
                .selected(signals.has_flag(sig::UI_DEBUG_ACTIVE))
                .build()
            {
                signals.set_flag(sig::ACTION_VIEW_TOGGLE_DEBUG);
            }
            ui.separator();
            if ui
                .menu_item_config("Texture Store")
                .selected(signals.has_flag(sig::UI_TEXTURE_EDITOR_OPEN))
                .build()
            {
                signals.toggle_flag(sig::UI_TEXTURE_EDITOR_OPEN);
            }
            if ui
                .menu_item_config("Font Store")
                .selected(signals.has_flag(sig::UI_FONT_STORE_OPEN))
                .build()
            {
                signals.toggle_flag(sig::UI_FONT_STORE_OPEN);
            }
            if ui
                .menu_item_config("Animation Store")
                .selected(signals.has_flag(sig::UI_ANIMATION_STORE_OPEN))
                .build()
            {
                signals.toggle_flag(sig::UI_ANIMATION_STORE_OPEN);
            }
            ui.separator();
            if ui
                .menu_item_config("Entity Selector")
                .selected(signals.has_flag(sig::UI_ENTITY_SELECTOR_OPEN))
                .build()
            {
                signals.toggle_flag(sig::UI_ENTITY_SELECTOR_OPEN);
            }
            if ui
                .menu_item_config("Groups")
                .selected(signals.has_flag(sig::UI_GROUPS_WINDOW_OPEN))
                .build()
            {
                signals.toggle_flag(sig::UI_GROUPS_WINDOW_OPEN);
            }
            if ui
                .menu_item_config("Entity Registry")
                .selected(signals.has_flag(sig::UI_ENTITY_REGISTRY_OPEN))
                .build()
            {
                signals.toggle_flag(sig::UI_ENTITY_REGISTRY_OPEN);
            }
            if ui
                .menu_item_config("Templates")
                .selected(signals.has_flag(sig::UI_TEMPLATE_BROWSER_OPEN))
                .build()
            {
                signals.toggle_flag(sig::UI_TEMPLATE_BROWSER_OPEN);
            }
            ui.separator();
            let preview_open = signals.has_flag(sig::UI_PREVIEW_MAPDATA_OPEN);
            if ui
                .menu_item_config("Preview Map Data")
                .selected(preview_open)
                .build()
            {
                if preview_open {
                    signals.take_flag(sig::UI_PREVIEW_MAPDATA_OPEN);
                } else {
                    signals.set_flag(sig::ACTION_VIEW_PREVIEW_MAPDATA);
                }
            }
        }

        if let Some(_entity) = ui.begin_menu("Entity")
            && ui.menu_item("Add")
        {
            signals.set_flag(sig::ACTION_ENTITY_ADD);
        }

        if let Some(_help) = ui.begin_menu("Help")
            && ui.menu_item("About")
        {
            open_about = true;
        }
    }
    open_about
}

pub(super) fn draw_about_modal(ui: &imgui::Ui) {
    ui.modal_popup_config("About")
        .always_auto_resize(true)
        .resizable(false)
        .movable(false)
        .build(|| {
            ui.text(format!(
                "Aberred Map Editor version {}",
                env!("CARGO_PKG_VERSION")
            ));
            ui.text("By Idaho06 from AkinoSoft!");
            ui.text("(c) 2026");
            ui.separator();
            if ui.button("OK") {
                ui.close_current_popup();
            }
        });
}

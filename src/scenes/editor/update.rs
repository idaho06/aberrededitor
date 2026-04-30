use super::commit::consume_entity_editor_commits;
use super::entity_editor_panel::{draw_entity_delete_modal, draw_entity_editor};
use super::entity_registry_panel::draw_entity_registry;
use super::entity_selector_panel::draw_entity_selector;
use super::font_panel::{draw_font_editor, draw_font_modals};
use super::groups_panel::draw_groups_window;
use super::menu::{draw_about_modal, draw_menu_bar};
use super::overlay::draw_selection_outline;
use super::template_browser_panel::draw_template_browser;
use super::texture_panel::{draw_texture_editor, draw_texture_modals};
use crate::signals as sig;
use crate::systems::entity_edit::CreateBlankEntityRequested;
use crate::systems::entity_inspector::InspectEntityRequested;
use crate::systems::entity_selector::SelectGroupRequested;
use crate::systems::entity_selector::{
    PickEntitiesAtPointRequested, SelectEntityRequested, SelectRegisteredEntityRequested,
};
use crate::systems::map_ops::{
    AddTextureRequested, LoadMapRequested, NewMapRequested, PreviewMapDataRequested,
    RemoveTextureRequested, RenameTextureKeyRequested, SaveMapRequested,
};
use crate::systems::tilemap_load::LoadTilemapRequested;
use crate::systems::utils::to_relative;
use aberredengine::events::switchdebug::SwitchDebugEvent;
use aberredengine::imgui;
use aberredengine::resources::appstate::AppState;
use aberredengine::resources::fontstore::FontStore;
use aberredengine::resources::input::InputState;
use aberredengine::resources::texturestore::TextureStore;
use aberredengine::resources::worldsignals::WorldSignals;
use aberredengine::systems::GameCtx;

pub fn editor_update(ctx: &mut GameCtx, _dt: f32, input: &InputState) {
    // Entity picking — left mouse click (Action1 rebound to mouse-only in editor_enter).
    // Suppressed when ImGui captured the mouse last frame to prevent clicks on UI widgets
    // from triggering world picks.
    if input.action_1.just_pressed && !ctx.world_signals.has_flag(sig::IMGUI_WANTS_MOUSE) {
        ctx.commands.trigger(PickEntitiesAtPointRequested {
            x: input.mouse_world_x,
            y: input.mouse_world_y,
        });
    }

    if let Some(row) = ctx.world_signals.clear_integer(sig::ES_SELECTED_ROW) {
        ctx.commands.trigger(SelectEntityRequested {
            index: row as usize,
        });
    }

    if let Some(group) = ctx
        .world_signals
        .remove_string(sig::GROUPS_SELECTED_GROUP)
        .map(|s| s.to_owned())
    {
        ctx.commands.trigger(SelectGroupRequested { group });
    }

    if let Some(key) = ctx
        .world_signals
        .remove_string(sig::ENTITY_REGISTRY_SELECTED_KEY)
        .map(|s| s.to_owned())
    {
        ctx.commands
            .trigger(SelectRegisteredEntityRequested { key });
    }

    if let Some(entity) = ctx.world_signals.remove_entity(sig::TEMPLATE_SELECT_ENTITY) {
        ctx.world_signals
            .set_entity(sig::ES_SELECTED_ENTITY, entity);
        ctx.commands.trigger(InspectEntityRequested { entity });
    }

    consume_entity_editor_commits(ctx);
    handle_entity_actions(ctx);
    handle_file_actions(ctx);
    handle_texture_actions(ctx);
    handle_font_actions(ctx);
    handle_view_actions(ctx);
}

pub fn editor_gui(
    ui: &imgui::Ui,
    signals: &mut WorldSignals,
    textures: &TextureStore,
    fonts: &FontStore,
    app_state: &AppState,
) {
    // Publish ImGui mouse-capture state so editor_update can suppress world picks next frame.
    if ui.io().want_capture_mouse {
        signals.set_flag(sig::IMGUI_WANTS_MOUSE);
    } else {
        signals.clear_flag(sig::IMGUI_WANTS_MOUSE);
    }
    if ui.io().want_capture_keyboard {
        signals.set_flag(sig::IMGUI_WANTS_KEYBOARD);
    } else {
        signals.clear_flag(sig::IMGUI_WANTS_KEYBOARD);
    }

    let open_about = draw_menu_bar(ui, signals);
    let (open_rename_popup, open_remove_popup) = draw_texture_editor(ui, signals, textures);
    let (open_font_rename, open_font_remove) = draw_font_editor(ui, signals, fonts);
    draw_map_preview(ui, signals);
    draw_groups_window(ui, signals, app_state);
    draw_entity_registry(ui, signals);
    draw_entity_selector(ui, signals, app_state);
    let open_delete_popup = draw_entity_editor(ui, signals, textures, fonts, app_state);
    draw_template_browser(ui, signals, app_state);

    if open_rename_popup {
        ui.open_popup("Rename Key##texture_editor");
    }
    if open_remove_popup {
        ui.open_popup("Remove Texture##texture_editor");
    }
    if open_font_rename {
        ui.open_popup("Rename Key##font_store");
    }
    if open_font_remove {
        ui.open_popup("Remove Font##font_store");
    }
    if open_about {
        ui.open_popup("About");
    }
    if open_delete_popup {
        ui.open_popup("Delete Entity##entity_editor");
    }

    draw_texture_modals(ui, signals);
    draw_font_modals(ui, signals);
    draw_about_modal(ui);
    draw_entity_delete_modal(ui, app_state);
    draw_selection_outline(ui, signals, app_state);
}

fn handle_file_actions(ctx: &mut GameCtx) {
    if ctx.world_signals.take_flag(sig::ACTION_FILE_NEW_MAP) {
        ctx.commands.trigger(NewMapRequested);
    }

    if ctx.world_signals.take_flag(sig::ACTION_FILE_OPEN_MAP)
        && let Some(path) = rfd::FileDialog::new()
            .add_filter("Map", &["map"])
            .pick_file()
    {
        let path = to_relative(&path.display().to_string());
        ctx.world_signals
            .set_string(sig::MAP_CURRENT_PATH, path.clone());
        ctx.commands.trigger(LoadMapRequested { path });
    }

    if ctx.world_signals.take_flag(sig::ACTION_FILE_SAVE) {
        if let Some(path) = ctx
            .world_signals
            .get_string(sig::MAP_CURRENT_PATH)
            .map(|s| s.to_owned())
        {
            ctx.commands.trigger(SaveMapRequested { path });
        } else {
            ctx.world_signals.set_flag(sig::ACTION_FILE_SAVE_AS);
        }
    }

    if ctx.world_signals.take_flag(sig::ACTION_FILE_SAVE_AS)
        && let Some(path) = rfd::FileDialog::new()
            .add_filter("Map", &["map"])
            .save_file()
    {
        let path = to_relative(&path.display().to_string());
        ctx.world_signals
            .set_string(sig::MAP_CURRENT_PATH, path.clone());
        ctx.commands.trigger(SaveMapRequested { path });
    }

    if ctx.world_signals.take_flag(sig::ACTION_FILE_LOAD_TILEMAP)
        && let Some(path) = rfd::FileDialog::new().pick_folder()
    {
        ctx.commands.trigger(LoadTilemapRequested {
            path: to_relative(&path.display().to_string()),
        });
    }
}

fn handle_entity_actions(ctx: &mut GameCtx) {
    if ctx.world_signals.take_flag(sig::ACTION_ENTITY_ADD) {
        let x = ctx
            .world_signals
            .get_scalar(sig::CAM_TARGET_X)
            .unwrap_or(0.0);
        let y = ctx
            .world_signals
            .get_scalar(sig::CAM_TARGET_Y)
            .unwrap_or(0.0);
        ctx.commands.trigger(CreateBlankEntityRequested { x, y });
    }
}

fn handle_texture_actions(ctx: &mut GameCtx) {
    if ctx.world_signals.take_flag(sig::ACTION_TEXTURE_RENAME) {
        let old_key = ctx
            .world_signals
            .get_string(sig::TEX_RENAME_SRC)
            .map(|s| s.to_owned());
        let new_key = ctx
            .world_signals
            .get_string(sig::TEX_RENAME_BUF)
            .map(|s| s.to_owned());
        if let (Some(old_key), Some(new_key)) = (old_key, new_key) {
            ctx.commands
                .trigger(RenameTextureKeyRequested { old_key, new_key });
        }
    }

    if ctx.world_signals.take_flag(sig::ACTION_TEXTURE_REMOVE)
        && let Some(key) = ctx
            .world_signals
            .get_string(sig::TEX_REMOVE_KEY)
            .map(|s| s.to_owned())
    {
        ctx.commands.trigger(RemoveTextureRequested { key });
    }

    if ctx.world_signals.take_flag(sig::ACTION_TEXTURE_ADD_BROWSE) {
        let key = ctx
            .world_signals
            .get_string(sig::TEX_ADD_KEY_BUF)
            .map(|s| s.to_owned())
            .unwrap_or_default();
        if !key.is_empty()
            && let Some(path) = rfd::FileDialog::new()
                .add_filter("Image", &["png", "jpg", "jpeg", "bmp"])
                .pick_file()
        {
            ctx.commands.trigger(AddTextureRequested {
                key,
                path: to_relative(&path.display().to_string()),
            });
        }
    }
}

fn handle_font_actions(ctx: &mut GameCtx) {
    use crate::systems::map_ops::{AddFontRequested, RemoveFontRequested, RenameFontKeyRequested};

    if ctx.world_signals.take_flag(sig::ACTION_FONT_RENAME) {
        let old_key = ctx
            .world_signals
            .get_string(sig::FONT_RENAME_SRC)
            .map(|s| s.to_owned());
        let new_key = ctx
            .world_signals
            .get_string(sig::FONT_RENAME_BUF)
            .map(|s| s.to_owned());
        if let (Some(old_key), Some(new_key)) = (old_key, new_key) {
            ctx.commands
                .trigger(RenameFontKeyRequested { old_key, new_key });
        }
    }

    if ctx.world_signals.take_flag(sig::ACTION_FONT_REMOVE)
        && let Some(key) = ctx
            .world_signals
            .get_string(sig::FONT_REMOVE_KEY)
            .map(|s| s.to_owned())
    {
        ctx.commands.trigger(RemoveFontRequested { key });
    }

    if ctx.world_signals.take_flag(sig::ACTION_FONT_ADD_BROWSE) {
        let key = ctx
            .world_signals
            .get_string(sig::FONT_ADD_KEY_BUF)
            .map(|s| s.to_owned())
            .unwrap_or_default();
        let font_size = ctx
            .world_signals
            .get_scalar(sig::FONT_ADD_SIZE_BUF)
            .unwrap_or(32.0);
        if !key.is_empty()
            && let Some(path) = rfd::FileDialog::new()
                .add_filter("Font", &["ttf", "otf"])
                .pick_file()
        {
            ctx.commands.trigger(AddFontRequested {
                key,
                path: to_relative(&path.display().to_string()),
                font_size,
            });
        }
    }
}

fn handle_view_actions(ctx: &mut GameCtx) {
    if ctx.world_signals.take_flag(sig::ACTION_VIEW_TOGGLE_DEBUG) {
        ctx.commands.trigger(SwitchDebugEvent {});
    }

    if ctx
        .world_signals
        .take_flag(sig::ACTION_VIEW_PREVIEW_MAPDATA)
    {
        ctx.commands.trigger(PreviewMapDataRequested);
    }
}

fn draw_map_preview(ui: &imgui::Ui, signals: &mut WorldSignals) {
    if !signals.has_flag(sig::UI_PREVIEW_MAPDATA_OPEN) {
        return;
    }

    let mut window_open = true;
    ui.window("Map Data Preview")
        .size([600.0, 500.0], imgui::Condition::FirstUseEver)
        .opened(&mut window_open)
        .build(|| {
            if ui.button("Refresh") {
                signals.set_flag(sig::ACTION_VIEW_PREVIEW_MAPDATA);
            }
            ui.separator();

            let mut json = signals
                .get_string(sig::MAPDATA_PREVIEW_JSON)
                .cloned()
                .unwrap_or_default();
            ui.input_text_multiline("##mapdata_json", &mut json, [-1.0, -1.0])
                .read_only(true)
                .build();
        });

    if !window_open {
        signals.take_flag(sig::UI_PREVIEW_MAPDATA_OPEN);
    }
}

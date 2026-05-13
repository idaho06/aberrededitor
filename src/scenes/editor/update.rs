//! Main editor scene update loop and ImGui GUI callback.
//!
//! Two public functions are registered in `main.rs` as the editor scene's callbacks:
//!
//! - `editor_update` — the `SceneUpdateFn`. Runs in ECS context each frame. Processes
//!   `WorldSignals` flags from the previous GUI frame (mouse picks, menu actions, selector
//!   selections, asset CRUD, and selection-mode-specific input handling before dispatching the
//!   corresponding events via `commands.trigger`.
//!
//! - `editor_gui` — the `GuiCallback`. Runs every frame after `editor_update`. Draws all
//!   ImGui panels and synchronises `IMGUI_WANTS_MOUSE/KEYBOARD` flags so `editor_update` can
//!   suppress world picks when the GUI is active.
//!
//! Action handling is split into `handle_file_actions`, `handle_entity_actions`,
//! `handle_texture_actions`, `handle_font_actions`, `handle_animation_actions`, and
//! `handle_view_actions` to keep `editor_update` readable.
use super::animation_panel::{draw_animation_modals, draw_animation_store};
use super::commit::consume_entity_editor_commits;
use super::entity_editor_panel::{draw_entity_delete_modal, draw_entity_editor};
use super::entity_registry_panel::draw_entity_registry;
use super::entity_selector_panel::draw_entity_selector;
use super::font_panel::{draw_font_editor, draw_font_modals};
use super::groups_panel::draw_groups_window;
use super::menu::{draw_about_modal, draw_menu_bar};
use super::multi_entity_selector_panel::{
    draw_multi_entity_selector, draw_multi_entity_selector_modals,
};
use super::overlay::{
    corners_aabb, draw_grid_preferences_modal, draw_multi_entity_outlines,
    draw_selection_drag_overlay, draw_selection_outline, render_to_world,
    GRID_PREFERENCES_POPUP_ID,
};
use super::template_browser_panel::draw_template_browser;
use super::texture_panel::{draw_texture_editor, draw_texture_modals};
use super::texture_viewer_panel::draw_texture_viewer;
use super::{
    SelectionDragRect, SelectionMode, current_selection_mode, finish_selection_drag,
    start_selection_drag, update_selection_drag,
};
use crate::signals as sig;
use crate::systems::animation_store_sync::AnimationStoreMutex;
use crate::systems::entity_edit::{
    AdjustMultiSelectionZRequested, CreateBlankEntityRequested, MoveMultiSelectionRequested,
};
use crate::systems::entity_inspector::InspectEntityRequested;
use crate::systems::entity_selector::SelectGroupRequested;
use crate::systems::entity_selector::{
    MultiEntitySelectionMutex, PickEntitiesAtPointRequested, PickEntitiesInRectRequested,
    SelectEntityRequested, SelectRegisteredEntityRequested,
};
use crate::systems::file_dialogs::{AsyncFileDialogRequest, request_async_dialog};
use crate::systems::map_ops::{
    AddAnimationRequested, NewMapRequested, PreviewMapDataRequested, RemoveAnimationRequested,
    RemoveTextureRequested, RenameAnimationKeyRequested, RenameTextureKeyRequested,
    SaveMapRequested, UpdateAnimationResourceRequested,
};
use aberredengine::events::switchdebug::SwitchDebugEvent;
use aberredengine::imgui;
use aberredengine::resources::appstate::AppState;
use aberredengine::resources::fontstore::FontStore;
use aberredengine::resources::input::InputState;
use aberredengine::resources::texturestore::TextureStore;
use aberredengine::resources::worldsignals::WorldSignals;
use aberredengine::systems::GameCtx;

pub fn editor_update(ctx: &mut GameCtx, _dt: f32, input: &InputState) {
    let wants_mouse = ctx.world_signals.has_flag(sig::IMGUI_WANTS_MOUSE);
    match current_selection_mode(&ctx.app_state) {
        SelectionMode::Click => {
            // Entity picking — left mouse click (Action1 rebound to mouse-only in editor_enter).
            // Suppressed when ImGui captured the mouse last frame to prevent clicks on UI widgets
            // from triggering world picks.
            if input.action_1.just_pressed && !wants_mouse {
                ctx.commands.trigger(PickEntitiesAtPointRequested {
                    x: input.mouse_world_x,
                    y: input.mouse_world_y,
                });
            }
        }
        SelectionMode::Rectangle => handle_rectangle_drag(ctx, input, wants_mouse),
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
    consume_multi_entity_commits(ctx);
    handle_entity_actions(ctx);
    handle_file_actions(ctx);
    handle_texture_actions(ctx);
    handle_font_actions(ctx);
    handle_animation_actions(ctx);
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

    let menu_actions = draw_menu_bar(ui, signals, app_state);
    let (open_rename_popup, open_remove_popup) = draw_texture_editor(ui, signals, textures);
    let (open_font_rename, open_font_remove) = draw_font_editor(ui, signals, fonts);
    let (open_anim_rename, open_anim_remove) =
        draw_animation_store(ui, signals, textures, app_state);
    draw_texture_viewer(ui, signals, textures, fonts);
    draw_map_preview(ui, signals);
    draw_groups_window(ui, signals, app_state);
    draw_entity_registry(ui, signals);
    draw_entity_selector(ui, signals, app_state);
    let (open_multi_move_popup, open_multi_z_popup) =
        draw_multi_entity_selector(ui, signals, app_state);
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
    if open_anim_rename {
        ui.open_popup("Rename Key##animation_store");
    }
    if open_anim_remove {
        ui.open_popup("Remove Animation##animation_store");
    }
    if menu_actions.open_about {
        ui.open_popup("About");
    }
    if menu_actions.open_grid_preferences {
        ui.open_popup(GRID_PREFERENCES_POPUP_ID);
    }
    if open_delete_popup {
        ui.open_popup("Delete Entity##entity_editor");
    }
    if open_multi_move_popup {
        ui.open_popup("Move Selected Entities##multi_selector");
    }
    if open_multi_z_popup {
        ui.open_popup("Adjust ZIndex##multi_selector");
    }

    draw_texture_modals(ui, signals);
    draw_font_modals(ui, signals);
    draw_animation_modals(ui, signals);
    draw_about_modal(ui);
    draw_grid_preferences_modal(ui, app_state);
    draw_multi_entity_selector_modals(ui, app_state);
    draw_entity_delete_modal(ui, app_state);
    draw_selection_outline(ui, signals, app_state);
    draw_multi_entity_outlines(ui, signals, app_state);
    draw_selection_drag_overlay(ui, signals, app_state);
}

fn consume_multi_entity_commits(ctx: &mut GameCtx) {
    let (move_request, z_request) = {
        let Some(mutex) = ctx.app_state.get::<MultiEntitySelectionMutex>() else {
            return;
        };
        let Ok(mut cache) = mutex.lock() else {
            return;
        };
        (
            cache.bulk_edit.pending_move_request.take(),
            cache.bulk_edit.pending_z_request.take(),
        )
    };

    if let Some([dx, dy]) = move_request {
        ctx.commands.trigger(MoveMultiSelectionRequested { dx, dy });
    }
    if let Some(delta) = z_request {
        ctx.commands
            .trigger(AdjustMultiSelectionZRequested { delta });
    }
}

fn handle_rectangle_drag(ctx: &mut GameCtx, input: &InputState, wants_mouse: bool) {
    let current_point = [input.mouse_x, input.mouse_y];

    if input.action_1.just_pressed && !wants_mouse {
        start_selection_drag(&ctx.app_state, current_point);
    }
    if input.action_1.active {
        update_selection_drag(&ctx.app_state, current_point);
    }
    if input.action_1.just_released
        && let Some(drag_rect) = finish_selection_drag(&ctx.app_state, current_point)
    {
        dispatch_rectangle_pick(ctx, drag_rect);
    }
}

fn dispatch_rectangle_pick(ctx: &mut GameCtx, drag_rect: SelectionDragRect) {
    let ([min_render_x, min_render_y], [max_render_x, max_render_y]) = drag_rect.normalized();
    let corners = [
        render_to_world(&ctx.world_signals, min_render_x, min_render_y),
        render_to_world(&ctx.world_signals, max_render_x, min_render_y),
        render_to_world(&ctx.world_signals, max_render_x, max_render_y),
        render_to_world(&ctx.world_signals, min_render_x, max_render_y),
    ];
    let (min_x, max_x, min_y, max_y) = corners_aabb(corners);
    ctx.commands.trigger(PickEntitiesInRectRequested {
        min_x,
        min_y,
        max_x,
        max_y,
    });
}

fn handle_file_actions(ctx: &mut GameCtx) {
    if ctx.world_signals.take_flag(sig::ACTION_FILE_NEW_MAP) {
        ctx.commands.trigger(NewMapRequested);
    }

    if ctx.world_signals.take_flag(sig::ACTION_FILE_OPEN_MAP) {
        request_async_dialog(&ctx.app_state, AsyncFileDialogRequest::OpenMap);
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

    if ctx.world_signals.take_flag(sig::ACTION_FILE_SAVE_AS) {
        request_async_dialog(&ctx.app_state, AsyncFileDialogRequest::SaveMapAs);
    }

    if ctx.world_signals.take_flag(sig::ACTION_FILE_LOAD_TILEMAP) {
        request_async_dialog(&ctx.app_state, AsyncFileDialogRequest::LoadTilemapFolder);
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
        if !key.is_empty() {
            request_async_dialog(&ctx.app_state, AsyncFileDialogRequest::AddTexture { key });
        }
    }
}

fn handle_font_actions(ctx: &mut GameCtx) {
    use crate::systems::map_ops::{RemoveFontRequested, RenameFontKeyRequested};

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
        if !key.is_empty() {
            request_async_dialog(
                &ctx.app_state,
                AsyncFileDialogRequest::AddFont { key, font_size },
            );
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

fn handle_animation_actions(ctx: &mut GameCtx) {
    if ctx.world_signals.take_flag(sig::ACTION_ANIM_ADD)
        && let Some(key) = ctx
            .world_signals
            .get_string(sig::ANIM_ADD_KEY_BUF)
            .filter(|k| !k.is_empty())
            .map(|s| s.to_owned())
    {
        ctx.commands.trigger(AddAnimationRequested { key });
    }

    if ctx.world_signals.take_flag(sig::ACTION_ANIM_RENAME) {
        let old_key = ctx
            .world_signals
            .get_string(sig::ANIM_RENAME_SRC)
            .map(|s| s.to_owned());
        let new_key = ctx
            .world_signals
            .get_string(sig::ANIM_RENAME_BUF)
            .map(|s| s.to_owned());
        if let (Some(old_key), Some(new_key)) = (old_key, new_key) {
            ctx.commands
                .trigger(RenameAnimationKeyRequested { old_key, new_key });
        }
    }

    if ctx.world_signals.take_flag(sig::ACTION_ANIM_REMOVE)
        && let Some(key) = ctx
            .world_signals
            .get_string(sig::ANIM_REMOVE_KEY)
            .map(|s| s.to_owned())
    {
        ctx.commands.trigger(RemoveAnimationRequested { key });
    }

    if ctx.world_signals.take_flag(sig::ACTION_ANIM_UPDATE)
        && let Some(key) = ctx
            .world_signals
            .get_string(sig::ANIM_UPDATE_KEY)
            .map(|s| s.to_owned())
        && let Some(mutex) = ctx.app_state.get::<AnimationStoreMutex>()
        && let Some(resource) = mutex.lock().unwrap().get(key.as_str()).cloned()
    {
        ctx.commands
            .trigger(UpdateAnimationResourceRequested { key, resource });
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

//! Named constants for all `WorldSignals` keys used in the editor.
//!
//! Centralising key strings here means typos become compile-time errors and
//! editors can navigate to a single definition instead of hunting through code.

// ---- Camera state (written by camera_sync, read by editor_gui) ----
pub const CAM_TARGET_X: &str = "editor:cam:target_x";
pub const CAM_TARGET_Y: &str = "editor:cam:target_y";
pub const CAM_ZOOM: &str = "editor:cam:zoom";
pub const CAM_OFFSET_X: &str = "editor:cam:offset_x";
pub const CAM_OFFSET_Y: &str = "editor:cam:offset_y";
pub const WIN_SCALE: &str = "editor:win:scale";
pub const WIN_OFFSET_X: &str = "editor:win:offset_x";
pub const WIN_OFFSET_Y: &str = "editor:win:offset_y";

// ---- ImGui / input state ----
pub const IMGUI_WANTS_MOUSE: &str = "imgui:wants_mouse";
pub const IMGUI_WANTS_KEYBOARD: &str = "imgui:wants_keyboard";

// ---- Debug mode ----
pub const UI_DEBUG_ACTIVE: &str = "ui:debug_active";

// ---- Editor camera entity ----
pub const EDITOR_CAMERA: &str = "editor:camera";

// ---- Map file path ----
pub const MAP_CURRENT_PATH: &str = "map:current_path";

// ---- File menu actions ----
pub const ACTION_FILE_NEW_MAP: &str = "gui:action:file:new_map";
pub const ACTION_FILE_OPEN_MAP: &str = "gui:action:file:open_map";
pub const ACTION_FILE_SAVE: &str = "gui:action:file:save";
pub const ACTION_FILE_SAVE_AS: &str = "gui:action:file:save_as";
pub const ACTION_FILE_LOAD_TILEMAP: &str = "gui:action:file:load_tilemap";

// ---- View menu actions ----
pub const ACTION_VIEW_RESET_ZOOM: &str = "gui:action:view:reset_zoom";
pub const ACTION_VIEW_TOGGLE_DEBUG: &str = "gui:action:view:toggle_debug";
pub const ACTION_VIEW_PREVIEW_MAPDATA: &str = "gui:action:view:preview_mapdata";

// ---- Entity menu actions ----
pub const ACTION_ENTITY_ADD: &str = "gui:action:entity:add";

// ---- Texture editor UI state ----
pub const UI_TEXTURE_EDITOR_OPEN: &str = "ui:texture_editor:open";
pub const TEX_ADD_KEY_BUF: &str = "gui:texture_editor:add_key_buf";
pub const TEX_RENAME_SRC: &str = "gui:texture_editor:rename_src";
pub const TEX_RENAME_BUF: &str = "gui:texture_editor:rename_buf";
pub const TEX_REMOVE_KEY: &str = "gui:texture_editor:remove_key";

// ---- Texture editor actions ----
pub const ACTION_TEXTURE_ADD_BROWSE: &str = "gui:action:texture:add_browse";
pub const ACTION_TEXTURE_RENAME: &str = "gui:action:texture:rename";
pub const ACTION_TEXTURE_REMOVE: &str = "gui:action:texture:remove";

// ---- Font store editor ----
pub const UI_FONT_STORE_OPEN: &str = "ui:font_store:open";
pub const FONT_ADD_KEY_BUF: &str = "gui:font_store:add_key_buf";
pub const FONT_ADD_SIZE_BUF: &str = "gui:font_store:add_size_buf";
pub const FONT_RENAME_SRC: &str = "gui:font_store:rename_src";
pub const FONT_RENAME_BUF: &str = "gui:font_store:rename_buf";
pub const FONT_REMOVE_KEY: &str = "gui:font_store:remove_key";
pub const ACTION_FONT_ADD_BROWSE: &str = "gui:action:font:add_browse";
pub const ACTION_FONT_RENAME: &str = "gui:action:font:rename";
pub const ACTION_FONT_REMOVE: &str = "gui:action:font:remove";

// ---- Shared texture viewer ----
pub const UI_TEXTURE_VIEWER_OPEN: &str = "ui:texture_viewer:open";
pub const TEXTURE_VIEWER_SOURCE_KIND: &str = "gui:texture_viewer:source_kind";
pub const TEXTURE_VIEWER_SOURCE_KEY: &str = "gui:texture_viewer:source_key";
pub const TEXTURE_VIEWER_SOURCE_TEXTURE: &str = "texture";
pub const TEXTURE_VIEWER_SOURCE_FONT: &str = "font";
pub const TEXTURE_VIEWER_SOURCE_ANIMATION: &str = "animation";

// ---- Animation store editor ----
pub const UI_ANIMATION_STORE_OPEN: &str = "ui:animation_store:open";
pub const ANIM_ADD_KEY_BUF: &str = "gui:animation_store:add_key_buf";
pub const ANIM_RENAME_SRC: &str = "gui:animation_store:rename_src";
pub const ANIM_RENAME_BUF: &str = "gui:animation_store:rename_buf";
pub const ANIM_REMOVE_KEY: &str = "gui:animation_store:remove_key";
pub const ANIM_UPDATE_KEY: &str = "gui:animation_store:update_key";
pub const ACTION_ANIM_ADD: &str = "gui:action:anim:add";
pub const ACTION_ANIM_RENAME: &str = "gui:action:anim:rename";
pub const ACTION_ANIM_REMOVE: &str = "gui:action:anim:remove";
pub const ACTION_ANIM_UPDATE: &str = "gui:action:anim:update";

// ---- Map data preview ----
pub const MAPDATA_PREVIEW_JSON: &str = "gui:mapdata_preview_json";
pub const UI_PREVIEW_MAPDATA_OPEN: &str = "gui:view:preview_mapdata_open";

// ---- Entity selector ----
pub const UI_ENTITY_SELECTOR_OPEN: &str = "ui:entity_selector:open";
pub const UI_MULTI_ENTITY_SELECTOR_OPEN: &str = "ui:multi_entity_selector:open";
pub const ES_SELECTED_ROW: &str = "gui:entity_selector:selected_row";
pub const ES_SELECTED_LABEL: &str = "gui:entity_selector:selected_label";
pub const ES_SELECTED_ENTITY: &str = "editor:selected_entity";

// ---- Groups window ----
pub const UI_GROUPS_WINDOW_OPEN: &str = "ui:groups_window:open";
pub const GROUPS_SELECTED_GROUP: &str = "gui:groups:selected_group";

// ---- Entity registry window ----
pub const UI_ENTITY_REGISTRY_OPEN: &str = "ui:entity_registry:open";
pub const ENTITY_REGISTRY_SELECTED_KEY: &str = "gui:entity_registry:selected_key";

// ---- Entity editor / inspector ----
pub const UI_ENTITY_EDITOR_OPEN: &str = "ui:entity_editor:open";

// ---- Template browser ----
pub const UI_TEMPLATE_BROWSER_OPEN: &str = "ui:template_browser:open";
pub const TEMPLATE_SELECT_ENTITY: &str = "gui:template_browser:select_entity";

/// All `WorldSignals.entities` keys that are used internally by the editor.
/// These are excluded from `EntityDef.registered_as` when saving a map.
pub const EDITOR_INTERNAL_ENTITY_KEYS: &[&str] =
    &[EDITOR_CAMERA, ES_SELECTED_ENTITY, TEMPLATE_SELECT_ENTITY];

pub fn is_user_entity_key(key: &str) -> bool {
    !EDITOR_INTERNAL_ENTITY_KEYS.contains(&key)
}

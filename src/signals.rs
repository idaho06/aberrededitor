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

// ---- Map data preview ----
pub const MAPDATA_PREVIEW_JSON: &str = "gui:mapdata_preview_json";
pub const UI_PREVIEW_MAPDATA_OPEN: &str = "gui:view:preview_mapdata_open";

// ---- Entity selector ----
pub const UI_ENTITY_SELECTOR_OPEN: &str = "ui:entity_selector:open";
pub const ES_PAYLOAD: &str = "gui:entity_selector:payload";
pub const ES_SELECTED_ROW: &str = "gui:entity_selector:selected_row";
pub const ES_SELECTED_LABEL: &str = "gui:entity_selector:selected_label";
pub const ES_SELECTION_CORNERS: &str = "gui:entity_selector:selection_corners";
pub const ES_SELECTED_ENTITY: &str = "editor:selected_entity";

// ---- Entity editor / inspector ----
pub const UI_ENTITY_EDITOR_OPEN: &str = "ui:entity_editor:open";
pub const EE_COMPONENT_SNAPSHOT: &str = "editor:entity_editor:component_snapshot";

// ---- Entity editor: pending buffers ----
pub const GUI_EE_PENDING_POS_X: &str = "gui:entity_editor:pending:pos_x";
pub const GUI_EE_PENDING_POS_Y: &str = "gui:entity_editor:pending:pos_y";
pub const GUI_EE_PENDING_Z_INDEX: &str = "gui:entity_editor:pending:z_index";
pub const GUI_EE_PENDING_ROT_DEG: &str = "gui:entity_editor:pending:rot_deg";
pub const GUI_EE_PENDING_SCALE_X: &str = "gui:entity_editor:pending:scale_x";
pub const GUI_EE_PENDING_SCALE_Y: &str = "gui:entity_editor:pending:scale_y";
pub const GUI_EE_PENDING_SPRITE_WIDTH: &str = "gui:entity_editor:pending:sprite_width";
pub const GUI_EE_PENDING_SPRITE_HEIGHT: &str = "gui:entity_editor:pending:sprite_height";
pub const GUI_EE_PENDING_SPRITE_OFFX: &str = "gui:entity_editor:pending:sprite_offx";
pub const GUI_EE_PENDING_SPRITE_OFFY: &str = "gui:entity_editor:pending:sprite_offy";
pub const GUI_EE_PENDING_SPRITE_ORGX: &str = "gui:entity_editor:pending:sprite_orgx";
pub const GUI_EE_PENDING_SPRITE_ORGY: &str = "gui:entity_editor:pending:sprite_orgy";
pub const GUI_EE_PENDING_BOX_SIZE_X: &str = "gui:entity_editor:pending:box_size_x";
pub const GUI_EE_PENDING_BOX_SIZE_Y: &str = "gui:entity_editor:pending:box_size_y";
pub const GUI_EE_PENDING_BOX_OFFX: &str = "gui:entity_editor:pending:box_offx";
pub const GUI_EE_PENDING_BOX_OFFY: &str = "gui:entity_editor:pending:box_offy";
pub const GUI_EE_PENDING_BOX_ORGX: &str = "gui:entity_editor:pending:box_orgx";
pub const GUI_EE_PENDING_BOX_ORGY: &str = "gui:entity_editor:pending:box_orgy";
pub const GUI_EE_PENDING_ANIM_FRAME_INDEX: &str = "gui:entity_editor:pending:anim_frame_index";
pub const GUI_EE_PENDING_ANIM_ELAPSED: &str = "gui:entity_editor:pending:anim_elapsed";

pub const GUI_EE_PENDING_GROUP: &str = "gui:entity_editor:pending:group";
pub const GUI_EE_PENDING_GROUP_DIRTY: &str = "gui:entity_editor:pending:group_dirty";
pub const GUI_EE_PENDING_SPRITE_TEX_KEY: &str = "gui:entity_editor:pending:sprite_tex_key";
pub const GUI_EE_PENDING_SPRITE_TEX_KEY_DIRTY: &str =
    "gui:entity_editor:pending:sprite_tex_key_dirty";
pub const GUI_EE_PENDING_ANIM_KEY: &str = "gui:entity_editor:pending:anim_key";
pub const GUI_EE_PENDING_ANIM_KEY_DIRTY: &str = "gui:entity_editor:pending:anim_key_dirty";

pub const GUI_EE_PENDING_SPRITE_FLIP_H: &str = "gui:entity_editor:pending:sprite_flip_h";
pub const GUI_EE_PENDING_SPRITE_FLIP_V: &str = "gui:entity_editor:pending:sprite_flip_v";

// ---- Entity editor actions ----
pub const ACTION_EE_COMMIT_POSITION: &str = "gui:action:entity_editor:commit_position";
pub const ACTION_EE_COMMIT_Z: &str = "gui:action:entity_editor:commit_z";
pub const ACTION_EE_COMMIT_GROUP: &str = "gui:action:entity_editor:commit_group";
pub const ACTION_EE_COMMIT_ROTATION: &str = "gui:action:entity_editor:commit_rotation";
pub const ACTION_EE_COMMIT_SCALE: &str = "gui:action:entity_editor:commit_scale";
pub const ACTION_EE_COMMIT_SPRITE: &str = "gui:action:entity_editor:commit_sprite";
pub const ACTION_EE_COMMIT_COLLIDER: &str = "gui:action:entity_editor:commit_collider";
pub const ACTION_EE_COMMIT_ANIMATION: &str = "gui:action:entity_editor:commit_animation";

const _: &[&str] = &[
    GUI_EE_PENDING_POS_X,
    GUI_EE_PENDING_POS_Y,
    GUI_EE_PENDING_Z_INDEX,
    GUI_EE_PENDING_ROT_DEG,
    GUI_EE_PENDING_SCALE_X,
    GUI_EE_PENDING_SCALE_Y,
    GUI_EE_PENDING_SPRITE_WIDTH,
    GUI_EE_PENDING_SPRITE_HEIGHT,
    GUI_EE_PENDING_SPRITE_OFFX,
    GUI_EE_PENDING_SPRITE_OFFY,
    GUI_EE_PENDING_SPRITE_ORGX,
    GUI_EE_PENDING_SPRITE_ORGY,
    GUI_EE_PENDING_BOX_SIZE_X,
    GUI_EE_PENDING_BOX_SIZE_Y,
    GUI_EE_PENDING_BOX_OFFX,
    GUI_EE_PENDING_BOX_OFFY,
    GUI_EE_PENDING_BOX_ORGX,
    GUI_EE_PENDING_BOX_ORGY,
    GUI_EE_PENDING_ANIM_FRAME_INDEX,
    GUI_EE_PENDING_ANIM_ELAPSED,
    GUI_EE_PENDING_GROUP,
    GUI_EE_PENDING_GROUP_DIRTY,
    GUI_EE_PENDING_SPRITE_TEX_KEY,
    GUI_EE_PENDING_SPRITE_TEX_KEY_DIRTY,
    GUI_EE_PENDING_ANIM_KEY,
    GUI_EE_PENDING_ANIM_KEY_DIRTY,
    GUI_EE_PENDING_SPRITE_FLIP_H,
    GUI_EE_PENDING_SPRITE_FLIP_V,
    ACTION_EE_COMMIT_POSITION,
    ACTION_EE_COMMIT_Z,
    ACTION_EE_COMMIT_GROUP,
    ACTION_EE_COMMIT_ROTATION,
    ACTION_EE_COMMIT_SCALE,
    ACTION_EE_COMMIT_SPRITE,
    ACTION_EE_COMMIT_COLLIDER,
    ACTION_EE_COMMIT_ANIMATION,
];

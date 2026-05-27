//! Editor scene — main map-editing interface.
//!
//! Public API consumed by `main.rs`:
//! - [`editor_enter`] / [`editor_exit`] — scene lifecycle callbacks.
//! - `editor_update` — `SceneUpdateFn`; processes signals and triggers ECS events.
//! - `editor_gui` — `GuiCallback`; draws all ImGui panels.
//! - [`EditorState`] — Bevy `Resource` with transient ECS-side editor state.
//! - [`entity_editor_selection_change_system`] — per-frame system; clears pending state on selection change.
//!
//! Internal modules (private): each GUI panel is a separate `mod` with a `draw_*` function
//! called from `editor_gui`. `commit` converts `PendingEditState` commits into ECS events.
//! `components/` holds per-component pending sub-structs, draw sections, and commit helpers.
mod animation_panel;
pub(crate) mod components;
mod commit;
mod entity_editor_panel;
mod entity_registry_panel;
mod entity_selector_panel;
mod font_panel;
mod groups_panel;
mod lifecycle;
pub(crate) mod map_properties_panel;
mod menu;
mod multi_entity_selector_panel;
mod overlay;
pub(crate) mod pending_state;
mod editor_tool;
mod state;
mod template_browser_panel;
mod texture_panel;
mod texture_viewer_panel;
mod update;
mod widgets;

pub use lifecycle::{editor_enter, editor_exit};
pub(crate) use overlay::draw_world_overlays;
pub(crate) use overlay::{OverlaySettingsMutex, OverlaySettingsState};
pub(crate) use editor_tool::{
    EditorTool, EditorToolMutex, EditorToolState, SelectionDragRect,
    current_selection_drag, current_tool, enter_placement_mode, exit_placement_mode,
    finish_selection_drag, reset_tool, set_tool, start_selection_drag,
    update_selection_drag,
};
pub use state::{EditorState, entity_editor_selection_change_system};
pub use update::{editor_gui, editor_update};

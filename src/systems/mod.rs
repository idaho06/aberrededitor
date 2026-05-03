//! ECS systems and observers for the editor.
//!
//! Each module owns a cohesive responsibility:
//! - `load_assets` — one-shot setup; inserts resources and loads shaders.
//! - `camera_sync` / `editor_camera` — camera state and pan/zoom input.
//! - `animation_store_sync` / `group_selector` / `template_selector` — per-frame AppState caches.
//! - `entity_selector` / `entity_inspector` / `entity_edit` — entity pick, inspect, mutate.
//! - `map_ops` — map file load/save and asset store CRUD.
//! - `tilemap_load` — tilemap folder loading and Lua-setup tagging.
//! - `debug_mirror` / `window_resize` — misc observers.
//! - `utils` — shared helper functions.
pub mod animation_store_sync;
pub mod camera_sync;
pub mod debug_mirror;
pub mod editor_camera;
pub mod entity_edit;
pub mod entity_inspector;
pub mod entity_selector;
pub mod file_dialogs;
pub mod group_selector;
pub mod load_assets;
pub mod map_ops;
pub mod template_selector;
pub mod tilemap_load;
pub mod utils;
pub mod window_resize;

//! Editor-specific Bevy components.
//!
//! - [`map_entity`] — `MapEntity` marker for entities owned by the current map.
//! - [`serialized_lua_setup`] — `SerializedLuaSetup` for storing Lua callback strings without the `lua` feature.
pub mod map_entity;
pub mod serialized_lua_setup;

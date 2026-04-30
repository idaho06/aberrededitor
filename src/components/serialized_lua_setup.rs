use aberredengine::bevy_ecs;
use aberredengine::bevy_ecs::prelude::Component;

/// Editor-side representation of `EntityDef.lua_setup`.
///
/// The engine's runtime `LuaSetup` component is cfg-gated behind the `lua`
/// feature, but the map format now always includes the optional field. This
/// metadata component lets the editor inspect, edit, clone, and save that
/// callback string without enabling Lua support in this binary.
#[derive(Component, Clone, Debug)]
pub struct SerializedLuaSetup {
    pub callback: String,
}

impl SerializedLuaSetup {
    pub fn new(callback: impl Into<String>) -> Self {
        Self {
            callback: callback.into(),
        }
    }
}

//! `EditorCtx` — the editor's `SystemParam` bundle for scene systems and observers.
//!
//! Holds only what `update.rs` and `commit.rs` touch. Take `GameConfig`, `InputBindings`,
//! `CameraFollowConfig` etc. as separate params where needed. A system that takes `EditorCtx`
//! must not also take `Res`/`ResMut` of `WorldSignals` or `AppState`: Bevy panics at startup
//! (B0002 conflicting access), which `cargo check` cannot catch.
use aberredengine::prelude::{AppState, Commands, ResMut, SystemParam, WorldSignals, bevy_ecs};

#[derive(SystemParam)]
pub struct EditorCtx<'w, 's> {
    pub commands: Commands<'w, 's>,
    pub world_signals: ResMut<'w, WorldSignals>,
    pub app_state: ResMut<'w, AppState>,
}

//! Per-frame sync: mirrors `AnimationStore` into `AppState` for the GUI callback.
//!
//! The GUI callback cannot query `AnimationStore` (a Bevy resource) directly because
//! it only receives `&AppState`. `animation_store_sync_system` detects changes to the store
//! and copies the full animation map into `AnimationStoreMutex`, which the animation panel
//! then reads via `app_state.get::<AnimationStoreMutex>()`.
use aberredengine::bevy_ecs::change_detection::DetectChanges;
use aberredengine::bevy_ecs::prelude::{Res, ResMut};
use aberredengine::resources::animationstore::{AnimationResource, AnimationStore};
use aberredengine::resources::appstate::AppState;
use rustc_hash::FxHashMap;

/// `AppState` key for the animation store mirror. Keyed by animation name, value is the full
/// `AnimationResource`. Populated by `animation_store_sync_system` on every change to `AnimationStore`.
pub type AnimationStoreMutex = std::sync::Mutex<FxHashMap<String, AnimationResource>>;

pub fn animation_store_sync_system(anim_store: Res<AnimationStore>, app_state: ResMut<AppState>) {
    if !anim_store.is_changed() {
        return;
    }
    if let Some(mutex) = app_state.get::<AnimationStoreMutex>() {
        *mutex.lock().unwrap() = anim_store.animations.clone();
    }
}

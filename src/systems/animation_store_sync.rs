use aberredengine::bevy_ecs::change_detection::DetectChanges;
use aberredengine::bevy_ecs::prelude::{Res, ResMut};
use aberredengine::resources::animationstore::{AnimationResource, AnimationStore};
use aberredengine::resources::appstate::AppState;
use rustc_hash::FxHashMap;

pub type AnimationStoreMutex = std::sync::Mutex<FxHashMap<String, AnimationResource>>;

pub fn animation_store_sync_system(
    anim_store: Res<AnimationStore>,
    app_state: ResMut<AppState>,
) {
    if !anim_store.is_changed() {
        return;
    }
    if let Some(mutex) = app_state.get::<AnimationStoreMutex>() {
        *mutex.lock().unwrap() = anim_store.animations.clone();
    }
}

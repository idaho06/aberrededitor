use crate::editor_types::ComponentSnapshot;
use crate::systems::animation_store_sync::AnimationStoreMutex;
use crate::systems::entity_edit::{RemoveAnimationRequested, UpdateAnimationRequested};
use aberredengine::bevy_ecs::prelude::Entity;
use aberredengine::imgui;
use aberredengine::systems::GameCtx;
use log::warn;

#[derive(Default, Clone)]
pub(crate) struct PendingAnimation {
    pub anim_key: Option<String>,
    pub commit: bool,
    pub remove: bool,
}

impl PendingAnimation {
    pub(crate) fn is_dirty(&self) -> bool {
        self.commit || self.remove
    }
}

pub(crate) fn draw_section(
    ui: &imgui::Ui,
    snap: &ComponentSnapshot,
    p: &mut PendingAnimation,
    anim_store: Option<&AnimationStoreMutex>,
) {
    let Some(ref animation) = snap.animation else {
        return;
    };
    ui.separator();
    ui.text("Animation");
    ui.same_line();
    if ui.button("Del##animation") {
        p.remove = true;
    }
    let anim_keys: Vec<String> = if let Some(mutex) = anim_store {
        let mut keys: Vec<String> = mutex.lock().unwrap().keys().cloned().collect();
        keys.sort_unstable();
        keys
    } else {
        vec![]
    };
    let key_strs: Vec<&str> = anim_keys.iter().map(|k| k.as_str()).collect();
    let current_key = p.anim_key.as_deref().unwrap_or(&animation.animation_key);
    let mut idx = key_strs.iter().position(|k| *k == current_key).unwrap_or(0);
    if key_strs.is_empty() {
        ui.text_disabled("(no animations in store)");
    } else {
        ui.set_next_item_width(-1.0);
        if ui.combo_simple_string("key##animation", &mut idx, &key_strs) {
            p.anim_key = Some(key_strs[idx].to_owned());
            p.commit = true;
        }
    }
}

pub(crate) fn commit(
    ctx: &mut GameCtx,
    entity: Entity,
    snap: &ComponentSnapshot,
    p: &PendingAnimation,
) {
    if p.remove {
        ctx.commands.trigger(RemoveAnimationRequested { entity });
    } else if p.commit {
        if let Some(ref animation) = snap.animation {
            ctx.commands.trigger(UpdateAnimationRequested {
                entity,
                animation_key: p
                    .anim_key
                    .as_ref()
                    .unwrap_or(&animation.animation_key)
                    .clone(),
            });
        } else {
            warn!(
                "consume_animation_commit: snapshot missing Animation for entity {}",
                entity.to_bits()
            );
        }
    }
}

use super::utils::entity_label;
use aberredengine::bevy_ecs::prelude::{Entity, Or, Query, Res, ResMut, Without};
use aberredengine::components::group::Group;
use aberredengine::components::mapposition::MapPosition;
use aberredengine::components::persistent::Persistent;
use aberredengine::components::zindex::ZIndex;
use aberredengine::resources::appstate::AppState;
use aberredengine::resources::worldsignals::WorldSignals;

use crate::signals as sig;

// ---------------------------------------------------------------------------
// Cache
// ---------------------------------------------------------------------------

#[derive(Default)]
pub struct TemplateSelectorCache {
    pub entities: Vec<Entity>,
    pub labels: Vec<String>,
}

pub type TemplateSelectorMutex = std::sync::Mutex<TemplateSelectorCache>;

// ---------------------------------------------------------------------------
// Per-frame system
// ---------------------------------------------------------------------------

type TemplateQuery<'w, 's> =
    Query<'w, 's, (Entity, Option<&'static Group>, Option<&'static Persistent>), Or<(Without<MapPosition>, Without<ZIndex>)>>;

pub fn update_template_cache(
    app_state: ResMut<AppState>,
    signals: Res<WorldSignals>,
    query: TemplateQuery,
) {
    if !signals.has_flag(sig::UI_TEMPLATE_BROWSER_OPEN) {
        return;
    }
    let Some(mutex) = app_state.get::<TemplateSelectorMutex>() else {
        return;
    };
    let Ok(mut cache) = mutex.lock() else {
        return;
    };
    cache.entities.clear();
    cache.labels.clear();
    for (entity, maybe_group, maybe_persistent) in query.iter() {
        let label = entity_label(entity, maybe_group, maybe_persistent);
        cache.entities.push(entity);
        cache.labels.push(label);
    }
}

//! Per-frame template entity cache for the template browser panel.
//!
//! Template entities are `MapEntity` entities that have neither `WorldPosition` nor `ZIndex` —
//! they serve as data-only archetypes for cloning into the map. `update_template_cache` scans
//! for these entities every frame when the template browser window is open.
use super::utils::entity_label;
use aberredengine::bevy_ecs::prelude::{Entity, Or, Query, Res, ResMut, Without};
use aberredengine::core::components::group::Group;
use aberredengine::core::components::world_position::WorldPosition;
use aberredengine::core::components::persistent::Persistent;
use aberredengine::core::components::z_index::ZIndex;
use aberredengine::core::resources::app_state::AppState;
use aberredengine::core::resources::world_signals::WorldSignals;

use crate::signals as sig;

// ---------------------------------------------------------------------------
// Cache
// ---------------------------------------------------------------------------

#[derive(Default)]
pub struct TemplateSelectorCache {
    pub entities: Vec<Entity>,
    pub labels: Vec<String>,
}

pub type TemplateSelectorMutex = std::sync::Arc<std::sync::Mutex<TemplateSelectorCache>>;

// ---------------------------------------------------------------------------
// Per-frame system
// ---------------------------------------------------------------------------

type TemplateQuery<'w, 's> = Query<
    'w,
    's,
    (Entity, Option<&'static Group>, Option<&'static Persistent>),
    Or<(Without<WorldPosition>, Without<ZIndex>)>,
>;

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

use crate::signals as sig;
use aberredengine::bevy_ecs::prelude::{Added, Changed, Or, Query, RemovedComponents, Res, ResMut};
use aberredengine::components::group::Group;
use aberredengine::resources::appstate::AppState;
use aberredengine::resources::worldsignals::WorldSignals;
use std::collections::HashMap;

use super::utils::display_group_name;

#[derive(Clone)]
pub struct GroupEntry {
    pub raw_name: String,
    pub count: usize,
}

pub struct GroupListCache {
    pub entries: Vec<GroupEntry>,
    dirty: bool,
}

impl Default for GroupListCache {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            dirty: true,
        }
    }
}

pub type GroupListMutex = std::sync::Mutex<GroupListCache>;

#[allow(clippy::type_complexity)]
pub fn update_group_cache(
    app_state: ResMut<AppState>,
    signals: Res<WorldSignals>,
    query: Query<&'static Group>,
    changed: Query<(), Or<(Added<Group>, Changed<Group>)>>,
    mut removed: RemovedComponents<Group>,
) {
    let has_changes = !changed.is_empty() || removed.read().count() > 0;

    let Some(mutex) = app_state.get::<GroupListMutex>() else {
        return;
    };
    let Ok(mut cache) = mutex.lock() else {
        return;
    };

    if has_changes {
        cache.dirty = true;
    }

    if !signals.has_flag(sig::UI_GROUPS_WINDOW_OPEN) || !cache.dirty {
        return;
    }

    let mut counts: HashMap<String, usize> = HashMap::new();
    for group in query.iter() {
        if let Some(v) = counts.get_mut(group.0.as_str()) {
            *v += 1;
        } else {
            counts.insert(group.0.clone(), 1);
        }
    }

    cache.entries.clear();
    cache.entries.extend(
        counts
            .into_iter()
            .map(|(raw_name, count)| GroupEntry { raw_name, count }),
    );
    cache.entries.sort_by(|a, b| {
        display_group_name(&a.raw_name)
            .cmp(display_group_name(&b.raw_name))
            .then_with(|| a.raw_name.cmp(&b.raw_name))
    });
    cache.dirty = false;
}

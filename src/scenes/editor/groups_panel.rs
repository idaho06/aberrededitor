//! Groups browser panel — lists all entity groups and their entity counts.
//!
//! Reads `GroupListMutex` from `AppState` (maintained by `update_group_cache`). Clicking a
//! group sets `GROUPS_SELECTED_GROUP` in `WorldSignals`; `editor_update` triggers
//! `SelectGroupRequested` to populate the entity selector with that group's entities.
use crate::signals as sig;
use crate::systems::group_selector::GroupListMutex;
use crate::systems::utils::display_group_name;
use aberredengine::imgui;
use aberredengine::resources::appstate::AppState;
use aberredengine::resources::worldsignals::WorldSignals;

pub(super) fn draw_groups_window(ui: &imgui::Ui, signals: &mut WorldSignals, app_state: &AppState) {
    if !signals.has_flag(sig::UI_GROUPS_WINDOW_OPEN) {
        return;
    }

    let entries = app_state.get::<GroupListMutex>().and_then(|mutex| {
        let cache = mutex.lock().ok()?;
        Some(cache.entries.clone())
    });

    let mut window_open = true;
    let mut selected_group: Option<String> = None;

    ui.window("Groups")
        .size([280.0, 360.0], imgui::Condition::FirstUseEver)
        .opened(&mut window_open)
        .build(|| {
            let Some(entries) = entries.as_ref() else {
                ui.text_disabled("Group cache not initialised.");
                return;
            };

            ui.text_disabled(format!("{} groups", entries.len()));
            ui.separator();

            if entries.is_empty() {
                ui.text_disabled("No grouped entities found.");
            } else {
                for (index, entry) in entries.iter().enumerate() {
                    let row_text =
                        format!("{} ({})", display_group_name(&entry.raw_name), entry.count);
                    let _id = ui.push_id_usize(index);
                    if ui.selectable_config(&row_text).build() {
                        selected_group = Some(entry.raw_name.clone());
                    }
                }
            }
        });

    if let Some(raw_name) = selected_group {
        signals.set_string(sig::GROUPS_SELECTED_GROUP, &raw_name);
    }
    if !window_open {
        signals.take_flag(sig::UI_GROUPS_WINDOW_OPEN);
    }
}

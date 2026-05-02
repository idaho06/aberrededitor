//! Template browser panel — pick a template entity to clone at the camera centre.
//!
//! Reads `TemplateSelectorMutex` from `AppState` (maintained by `update_template_cache`).
//! Template entities are those without `MapPosition` or `ZIndex` — they act as data-only
//! archetypes that can be cloned into the map.
//!
//! On selection, sets `TEMPLATE_SELECT_ENTITY` in `WorldSignals`; `editor_update` reads that
//! flag and triggers `InspectEntityRequested` to show the entity in the editor.
use crate::signals as sig;
use crate::systems::template_selector::TemplateSelectorMutex;
use aberredengine::bevy_ecs::prelude::Entity;
use aberredengine::imgui;
use aberredengine::resources::appstate::AppState;
use aberredengine::resources::worldsignals::WorldSignals;

pub(super) fn draw_template_browser(
    ui: &imgui::Ui,
    signals: &mut WorldSignals,
    app_state: &AppState,
) {
    if !signals.has_flag(sig::UI_TEMPLATE_BROWSER_OPEN) {
        return;
    }

    let snapshot: Option<(Vec<Entity>, Vec<String>)> =
        app_state.get::<TemplateSelectorMutex>().and_then(|mutex| {
            let cache = mutex.lock().ok()?;
            Some((cache.entities.clone(), cache.labels.clone()))
        });

    let mut window_open = true;
    let mut entity_to_select = None;

    ui.window("Template Browser")
        .size([300.0, 400.0], imgui::Condition::FirstUseEver)
        .opened(&mut window_open)
        .build(|| {
            let Some((entities, labels)) = snapshot.as_ref() else {
                ui.text_disabled("Template cache not initialised.");
                return;
            };

            ui.text_disabled(format!("{} template entities", entities.len()));
            ui.separator();

            if entities.is_empty() {
                ui.text_disabled("No non-rendered entities found.");
            } else {
                for (i, (entity, label)) in entities.iter().zip(labels.iter()).enumerate() {
                    let _id = ui.push_id_usize(i);
                    if ui.selectable_config(label.as_str()).build() {
                        entity_to_select = Some(*entity);
                    }
                }
            }
        });

    if let Some(entity) = entity_to_select {
        signals.set_entity(sig::TEMPLATE_SELECT_ENTITY, entity);
    }
    if !window_open {
        signals.take_flag(sig::UI_TEMPLATE_BROWSER_OPEN);
    }
}

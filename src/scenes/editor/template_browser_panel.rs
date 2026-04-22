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
        signals.set_flag(sig::UI_ENTITY_EDITOR_OPEN);
    }
    if !window_open {
        signals.take_flag(sig::UI_TEMPLATE_BROWSER_OPEN);
    }
}

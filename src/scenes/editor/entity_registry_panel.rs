use crate::signals as sig;
use aberredengine::imgui;
use aberredengine::resources::worldsignals::WorldSignals;

pub(super) fn draw_entity_registry(ui: &imgui::Ui, signals: &mut WorldSignals) {
    if !signals.has_flag(sig::UI_ENTITY_REGISTRY_OPEN) {
        return;
    }

    let entries: Vec<&str> = signals
        .entities
        .keys()
        .filter(|key| sig::is_user_entity_key(key))
        .map(String::as_str)
        .collect();

    let mut window_open = true;
    let mut selected_key: Option<String> = None;

    ui.window("Entity Registry")
        .size([320.0, 400.0], imgui::Condition::FirstUseEver)
        .opened(&mut window_open)
        .build(|| {
            ui.text_disabled(format!("{} keys", entries.len()));
            ui.separator();

            if entries.is_empty() {
                ui.text_disabled("No registered entities.");
            } else {
                for (index, key) in entries.iter().copied().enumerate() {
                    let _id = ui.push_id_usize(index);
                    if ui.selectable_config(key).build() {
                        selected_key = Some(key.to_owned());
                    }
                }
            }
        });

    if let Some(key) = selected_key {
        signals.set_string(sig::ENTITY_REGISTRY_SELECTED_KEY, &key);
    }
    if !window_open {
        signals.take_flag(sig::UI_ENTITY_REGISTRY_OPEN);
    }
}
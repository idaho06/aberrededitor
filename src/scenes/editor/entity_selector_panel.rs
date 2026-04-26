use crate::signals as sig;
use crate::systems::entity_selector::{RenderableSelectorMutex, SelectorSource};
use aberredengine::imgui;
use aberredengine::resources::appstate::AppState;
use aberredengine::resources::worldsignals::WorldSignals;

pub(super) fn draw_entity_selector(
    ui: &imgui::Ui,
    signals: &mut WorldSignals,
    app_state: &AppState,
) {
    if !signals.has_flag(sig::UI_ENTITY_SELECTOR_OPEN) {
        return;
    }

    let snapshot = app_state.get::<RenderableSelectorMutex>().and_then(|mutex| {
        let cache = mutex.lock().ok()?;
        Some((
            cache.source.clone(),
            cache.labels.clone(),
            cache.z_indices.clone(),
        ))
    });

    let mut window_open = true;
    let mut row_to_select: Option<i32> = None;

    ui.window("Entity Selector")
        .size([320.0, 400.0], imgui::Condition::FirstUseEver)
        .opened(&mut window_open)
        .build(|| {
            let Some((source, labels, z_indices)) = snapshot.as_ref() else {
                ui.text_disabled("Selector cache not initialised.");
                return;
            };

            let empty_msg = match source {
                SelectorSource::None => {
                    ui.text_disabled("Left-click in the scene to pick entities.");
                    None
                }
                SelectorSource::Click { x, y } => {
                    ui.text_disabled(format!("Click: ({:.1}, {:.1})", x, y));
                    ui.separator();
                    Some("No entities at click position.")
                }
                SelectorSource::Group { display_name } => {
                    ui.text_disabled(format!("Group: {}", display_name));
                    ui.text_disabled(format!("{} entities", labels.len()));
                    ui.separator();
                    Some("No entities found in this group.")
                }
                SelectorSource::Registry { key } => {
                    ui.text_disabled(format!("Registry: {}", key));
                    ui.separator();
                    Some("Registered entity is not available.")
                }
            };

            if let Some(empty_msg) = empty_msg {
                if labels.is_empty() {
                    ui.text_disabled(empty_msg);
                } else {
                    for (index, (label, &zindex)) in labels.iter().zip(z_indices.iter()).enumerate() {
                        let row_text = format!("{} (z={:.1})", label, zindex);
                        let _id = ui.push_id_usize(index);
                        if ui.selectable_config(&row_text).build() {
                            row_to_select = Some(index as i32);
                        }
                    }
                }
                ui.separator();
                if let Some(label) = signals.get_string(sig::ES_SELECTED_LABEL).cloned() {
                    ui.text(format!("Selected: {}", label));
                } else {
                    ui.text_disabled("No entity selected.");
                }
            }
        });

    if let Some(row) = row_to_select {
        signals.set_integer(sig::ES_SELECTED_ROW, row);
    }
    if !window_open {
        signals.take_flag(sig::UI_ENTITY_SELECTOR_OPEN);
    }
}

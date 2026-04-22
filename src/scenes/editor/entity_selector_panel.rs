use crate::signals as sig;
use crate::systems::entity_selector::SelectorMutex;
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

    let mut window_open = true;
    let mut row_to_select: Option<i32> = None;

    ui.window("Entity Selector")
        .size([320.0, 400.0], imgui::Condition::FirstUseEver)
        .opened(&mut window_open)
        .build(|| {
            let mutex = app_state.get::<SelectorMutex>();
            let guard = mutex.map(|m| m.lock().unwrap());
            let click_pos = guard.as_deref().and_then(|c| c.click_pos);

            match click_pos {
                None => {
                    ui.text_disabled("Left-click in the scene to pick entities.");
                }
                Some((cx, cy)) => {
                    ui.text_disabled(format!("Click: ({:.1}, {:.1})", cx, cy));
                    ui.separator();

                    let cache = guard.as_deref().unwrap();
                    if cache.hits.is_empty() {
                        ui.text_disabled("No entities at click position.");
                    } else {
                        for (index, (label, &zindex)) in
                            cache.labels.iter().zip(cache.z_indices.iter()).enumerate()
                        {
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
            }
        });

    if let Some(row) = row_to_select {
        signals.set_integer(sig::ES_SELECTED_ROW, row);
    }
    if !window_open {
        signals.take_flag(sig::UI_ENTITY_SELECTOR_OPEN);
    }
}

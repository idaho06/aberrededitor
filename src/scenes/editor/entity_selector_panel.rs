use crate::signals as sig;
use aberredengine::imgui;
use aberredengine::resources::worldsignals::WorldSignals;

pub(super) fn draw_entity_selector(ui: &imgui::Ui, signals: &mut WorldSignals) {
    if !signals.has_flag(sig::UI_ENTITY_SELECTOR_OPEN) {
        return;
    }

    let mut window_open = true;
    ui.window("Entity Selector")
        .size([320.0, 400.0], imgui::Condition::FirstUseEver)
        .opened(&mut window_open)
        .build(|| {
            let payload_str = signals.get_string(sig::ES_PAYLOAD).cloned();

            match payload_str
                .as_deref()
                .and_then(|payload| serde_json::from_str::<serde_json::Value>(payload).ok())
            {
                None => {
                    ui.text_disabled("Left-click in the scene to pick entities.");
                }
                Some(payload) => {
                    if let (Some(cx), Some(cy)) =
                        (payload["click"][0].as_f64(), payload["click"][1].as_f64())
                    {
                        ui.text_disabled(format!("Click: ({:.1}, {:.1})", cx, cy));
                    }
                    ui.separator();

                    if let Some(hits) = payload["hits"].as_array() {
                        if hits.is_empty() {
                            ui.text_disabled("No entities at click position.");
                        } else {
                            for (index, hit) in hits.iter().enumerate() {
                                let label = hit["label"].as_str().unwrap_or("?");
                                let zindex = hit["zindex"].as_f64().unwrap_or(0.0);
                                let row_text = format!("{} (z={:.1})", label, zindex);
                                let _id = ui.push_id_usize(index);
                                if ui.selectable_config(&row_text).build() {
                                    signals.set_integer(sig::ES_SELECTED_ROW, index as i32);
                                }
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

    if !window_open {
        signals.take_flag(sig::UI_ENTITY_SELECTOR_OPEN);
    }
}

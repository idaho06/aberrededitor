//! Multi-entity selector panel — shows the active rectangle multi-selection result set.
//!
//! Reads `MultiEntitySelectionMutex` from `AppState`. All rows are rendered as selected. The panel
//! also owns the bulk-action buttons and modals for relative move / z-index updates.
use crate::signals as sig;
use crate::systems::entity_selector::{MultiEntitySelectionMutex, SelectorSource};
use aberredengine::imgui;
use aberredengine::resources::appstate::AppState;
use aberredengine::resources::worldsignals::WorldSignals;

pub(super) fn draw_multi_entity_selector(
    ui: &imgui::Ui,
    signals: &mut WorldSignals,
    app_state: &AppState,
) -> (bool, bool) {
    if !signals.has_flag(sig::UI_MULTI_ENTITY_SELECTOR_OPEN) {
        return (false, false);
    }

    let snapshot = app_state
        .get::<MultiEntitySelectionMutex>()
        .and_then(|mutex| {
            let cache = mutex.lock().ok()?;
            Some((cache.source.clone(), cache.labels.clone(), cache.hits.len()))
        });

    let mut window_open = true;
    let mut open_move_popup = false;
    let mut open_z_popup = false;
    ui.window("Selected Entities")
        .size([360.0, 420.0], imgui::Condition::FirstUseEver)
        .opened(&mut window_open)
        .build(|| {
            let Some((source, labels, hit_count)) = snapshot.as_ref() else {
                ui.text_disabled("Multi-selection cache not initialised.");
                return;
            };

            match source {
                SelectorSource::Rectangle {
                    min_x,
                    min_y,
                    max_x,
                    max_y,
                } => {
                    ui.text_disabled(format!(
                        "Rectangle: ({:.1}, {:.1}) -> ({:.1}, {:.1})",
                        min_x, min_y, max_x, max_y
                    ));
                }
                _ => ui.text_disabled("Multiple selected entities"),
            }
            ui.text_disabled(format!("{} entities selected", labels.len()));
            ui.separator();

            if *hit_count > 0 {
                if ui.button("Move") {
                    if let Some(mutex) = app_state.get::<MultiEntitySelectionMutex>()
                        && let Ok(mut cache) = mutex.lock()
                    {
                        cache.bulk_edit.reset_move_buffer();
                    }
                    open_move_popup = true;
                }
                ui.same_line();
                if ui.button("Adjust Z") {
                    if let Some(mutex) = app_state.get::<MultiEntitySelectionMutex>()
                        && let Ok(mut cache) = mutex.lock()
                    {
                        cache.bulk_edit.reset_z_buffer();
                    }
                    open_z_popup = true;
                }
                ui.separator();
            }

            if labels.is_empty() {
                ui.text_disabled("No entities selected.");
            } else {
                for (index, label) in labels.iter().enumerate() {
                    let _id = ui.push_id_usize(index);
                    let _ = ui.selectable_config(label).selected(true).build();
                }
            }
        });

    if !window_open {
        signals.take_flag(sig::UI_MULTI_ENTITY_SELECTOR_OPEN);
    }
    (open_move_popup, open_z_popup)
}

pub(super) fn draw_multi_entity_selector_modals(ui: &imgui::Ui, app_state: &AppState) {
    ui.modal_popup_config("Move Selected Entities##multi_selector")
        .always_auto_resize(true)
        .resizable(false)
        .movable(false)
        .build(|| {
            let Some(mutex) = app_state.get::<MultiEntitySelectionMutex>() else {
                ui.text_disabled("Multi-selection cache not initialised.");
                return;
            };
            let Ok(mut cache) = mutex.lock() else {
                ui.text_disabled("Multi-selection cache unavailable.");
                return;
            };

            ui.text(format!("Move {} selected entities by:", cache.hits.len()));
            ui.spacing();
            ui.input_float("X##multi_move_x", &mut cache.bulk_edit.move_dx)
                .enter_returns_true(true)
                .build();
            ui.input_float("Y##multi_move_y", &mut cache.bulk_edit.move_dy)
                .enter_returns_true(true)
                .build();
            ui.spacing();
            ui.separator();
            if ui.button("Apply##multi_move_apply") {
                cache.bulk_edit.pending_move_request =
                    Some([cache.bulk_edit.move_dx, cache.bulk_edit.move_dy]);
                cache.bulk_edit.reset_move_buffer();
                ui.close_current_popup();
            }
            ui.same_line();
            if ui.button("Cancel##multi_move_cancel") {
                cache.bulk_edit.reset_move_buffer();
                ui.close_current_popup();
            }
        });

    ui.modal_popup_config("Adjust ZIndex##multi_selector")
        .always_auto_resize(true)
        .resizable(false)
        .movable(false)
        .build(|| {
            let Some(mutex) = app_state.get::<MultiEntitySelectionMutex>() else {
                ui.text_disabled("Multi-selection cache not initialised.");
                return;
            };
            let Ok(mut cache) = mutex.lock() else {
                ui.text_disabled("Multi-selection cache unavailable.");
                return;
            };

            ui.text(format!(
                "Adjust z-index of {} selected entities by:",
                cache.hits.len()
            ));
            ui.spacing();
            ui.input_float("Delta##multi_z_delta", &mut cache.bulk_edit.z_delta)
                .enter_returns_true(true)
                .build();
            ui.spacing();
            ui.separator();
            if ui.button("Apply##multi_z_apply") {
                cache.bulk_edit.pending_z_request = Some(cache.bulk_edit.z_delta);
                cache.bulk_edit.reset_z_buffer();
                ui.close_current_popup();
            }
            ui.same_line();
            if ui.button("Cancel##multi_z_cancel") {
                cache.bulk_edit.reset_z_buffer();
                ui.close_current_popup();
            }
        });
}

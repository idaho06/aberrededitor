use super::widgets;
use crate::signals as sig;
use aberredengine::imgui;
use aberredengine::resources::appstate::AppState;
use aberredengine::resources::mapdata::MapData;
use aberredengine::resources::worldsignals::WorldSignals;
use std::sync::{Arc, Mutex};

#[derive(Default)]
pub struct MapPropertiesState {
    // Snapshot values — kept in sync with MapData by observers on map load/new.
    pub name: String,
    pub description: String,
    pub author: String,
    pub version: String,
    pub background_color: Option<[u8; 3]>,
    // Pending text buffers (None = unmodified since last sync)
    pub pending_name: Option<String>,
    pub pending_description: Option<String>,
    pub pending_author: Option<String>,
    pub pending_version: Option<String>,
    // Pending background color as [f32; 3] for direct use with color_edit3
    pub pending_bg_color: Option<[f32; 3]>,
}

impl MapPropertiesState {
    pub fn reset_from_map(&mut self, map: &MapData) {
        self.name = map.name.clone();
        self.description = map.description.clone();
        self.author = map.author.clone();
        self.version = map.version.clone();
        self.background_color = map.background_color;
        self.pending_name = None;
        self.pending_description = None;
        self.pending_author = None;
        self.pending_version = None;
        self.pending_bg_color = None;
    }
}

pub type MapPropertiesMutex = Arc<Mutex<MapPropertiesState>>;

pub fn draw_map_properties_panel(
    ui: &imgui::Ui,
    signals: &mut WorldSignals,
    app_state: &AppState,
) {
    if !signals.has_flag(sig::UI_MAP_PROPERTIES_OPEN) {
        return;
    }
    let Some(mutex) = app_state.get::<MapPropertiesMutex>() else {
        return;
    };
    let Ok(mut state) = mutex.lock() else {
        return;
    };

    let mut window_open = true;
    // Clone snapshots before creating the closure so the borrow checker sees
    // them as owned locals rather than borrows through the MutexGuard.
    let snap_name = state.name.clone();
    let snap_description = state.description.clone();
    let snap_author = state.author.clone();
    let snap_version = state.version.clone();
    // Explicitly deref the MutexGuard so the borrow checker can split fields.
    let s = &mut *state;
    // The panel commits via the Apply button, not per-field, so the
    // `committed` out-param required by draw_text_buffer_input is discarded.
    let mut committed = false;

    ui.window("Map Properties")
        .size([340.0, 280.0], imgui::Condition::FirstUseEver)
        .opened(&mut window_open)
        .build(|| {
            ui.text("Name");
            ui.same_line();
            ui.set_next_item_width(-1.0);
            widgets::draw_text_buffer_input(
                ui,
                "##map_name",
                &mut s.pending_name,
                &mut committed,
                &snap_name,
            );

            ui.text("Description");
            ui.same_line();
            ui.set_next_item_width(-1.0);
            widgets::draw_text_buffer_input(
                ui,
                "##map_description",
                &mut s.pending_description,
                &mut committed,
                &snap_description,
            );

            ui.text("Author");
            ui.same_line();
            ui.set_next_item_width(-1.0);
            widgets::draw_text_buffer_input(
                ui,
                "##map_author",
                &mut s.pending_author,
                &mut committed,
                &snap_author,
            );

            ui.text("Version");
            ui.same_line();
            ui.set_next_item_width(-1.0);
            widgets::draw_text_buffer_input(
                ui,
                "##map_version",
                &mut s.pending_version,
                &mut committed,
                &snap_version,
            );

            ui.separator();

            ui.text("Background");
            ui.same_line();
            let snapshot_f32 = s.background_color.map_or(
                [0.0_f32, 0.0, 0.0],
                |[r, g, b]| [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0],
            );
            let mut color_f32 = s.pending_bg_color.unwrap_or(snapshot_f32);
            if ui.color_edit3("##bg_color", &mut color_f32) {
                s.pending_bg_color = Some(color_f32);
            }

            ui.separator();

            if ui.button("Apply") {
                signals.set_flag(sig::ACTION_MAP_PROPERTIES_APPLY);
            }
        });

    if !window_open {
        signals.take_flag(sig::UI_MAP_PROPERTIES_OPEN);
    }
}

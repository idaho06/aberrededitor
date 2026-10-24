use super::super::widgets::{draw_float_input, draw_text_buffer_input};
use crate::editor_types::ComponentSnapshot;
use crate::systems::entity_edit::{RemoveDynamicTextRequested, UpdateDynamicTextRequested};
use aberredengine::core::resources::font_key::DEFAULT_FONT_KEY;
use aberredengine::prelude::{Commands, Entity, FontStore, imgui};
use log::warn;

#[derive(Default, Clone)]
pub(crate) struct PendingDynamicText {
    pub text: Option<String>,
    pub font_key: Option<String>,
    pub font_size: Option<f32>,
    pub color: Option<[f32; 4]>,
    pub commit: bool,
    pub remove: bool,
}

impl PendingDynamicText {
    pub(crate) fn is_dirty(&self) -> bool {
        self.commit || self.remove
    }
}

pub(crate) fn draw_section(
    ui: &imgui::Ui,
    snap: &ComponentSnapshot,
    p: &mut PendingDynamicText,
    fonts: &FontStore,
) {
    let Some(ref dt) = snap.dynamic_text else {
        return;
    };
    ui.separator();
    ui.text("DynamicText");
    ui.same_line();
    if ui.button("Del##dynamictext") {
        p.remove = true;
    }

    let mut committed = false;
    draw_text_buffer_input(ui, "text##dt", &mut p.text, &mut committed, &dt.text);
    if committed {
        p.commit = true;
    }

    // "" is an alias of the built-in default font; show and commit it as "default".
    let font_key_current: &str = match p.font_key.as_deref().unwrap_or(&dt.font_key) {
        "" => DEFAULT_FONT_KEY,
        k => k,
    };
    let mut map_keys: Vec<&str> = fonts.meta.keys().map(|k| k.as_str()).collect();
    map_keys.sort_unstable();
    let font_keys: Vec<&str> = std::iter::once(DEFAULT_FONT_KEY).chain(map_keys).collect();
    let mut current_font = font_keys
        .iter()
        .position(|k| *k == font_key_current)
        .unwrap_or(0);
    if ui.combo_simple_string("font_key##dt", &mut current_font, &font_keys) {
        p.font_key = Some(font_keys[current_font].to_owned());
        p.commit = true;
    }

    if let Some(v) = draw_float_input(
        ui,
        "font_size##dt",
        p.font_size.unwrap_or(dt.font_size),
        1.0,
    ) {
        p.font_size = Some(v);
        p.commit = true;
    }

    let mut color = p.color.unwrap_or(dt.color_normalized());
    if ui.color_edit4("##dt_color", &mut color) {
        p.color = Some(color);
    }
    if ui.is_item_deactivated_after_edit() {
        p.commit = true;
    }
}

pub(crate) fn commit(
    commands: &mut Commands,
    entity: Entity,
    snap: &ComponentSnapshot,
    p: &PendingDynamicText,
) {
    if p.remove {
        commands.trigger(RemoveDynamicTextRequested { entity });
    } else if p.commit {
        if let Some(ref dt) = snap.dynamic_text {
            let [r, g, b, a] = p.color.unwrap_or(dt.color_normalized());
            commands.trigger(UpdateDynamicTextRequested {
                entity,
                text: p.text.as_ref().unwrap_or(&dt.text).clone(),
                font_key: p.font_key.as_ref().unwrap_or(&dt.font_key).clone(),
                font_size: p.font_size.unwrap_or(dt.font_size),
                r: (r * 255.0).round() as u8,
                g: (g * 255.0).round() as u8,
                b: (b * 255.0).round() as u8,
                a: (a * 255.0).round() as u8,
            });
        } else {
            warn!(
                "consume_dynamic_text_commit: snapshot missing DynamicText for entity {}",
                entity.to_bits()
            );
        }
    }
}

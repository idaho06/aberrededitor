//! Reusable ImGui input widgets for the entity editor panels.
//!
//! All helpers follow the same convention: they take a mutable reference to the value being
//! edited and return `true` when the value should be committed (input deactivated or button
//! clicked). Callers set the corresponding `PendingEditState` commit flag on `true`.
use aberredengine::imgui;

pub(super) const BTN_W: f32 = 22.0;
pub(super) const BTN_SPACING: f32 = 4.0;
const MIN_NUMERIC_INPUT_W: f32 = 96.0;

fn split_imgui_label(label: &str) -> (&str, &str) {
    label.split_once("##").unwrap_or((label, ""))
}

fn hidden_numeric_label(label: &str) -> String {
    format!("##{label}")
}

fn numeric_input_width(ui: &imgui::Ui, visible_label: &str) -> f32 {
    let label_width = if visible_label.is_empty() {
        0.0
    } else {
        ui.calc_text_size(visible_label)[0] + BTN_SPACING
    };
    let reserved_width = BTN_W * 2.0 + BTN_SPACING * 2.0 + label_width;
    (ui.content_region_avail()[0] - reserved_width).max(MIN_NUMERIC_INPUT_W)
}

fn draw_trailing_numeric_label(ui: &imgui::Ui, visible_label: &str) {
    if visible_label.is_empty() {
        return;
    }
    ui.same_line_with_spacing(0.0, BTN_SPACING);
    ui.text(visible_label);
}

/// Renders − and + step buttons. Returns `Some(new_value)` if a button was clicked.
fn draw_step_buttons(ui: &imgui::Ui, label: &str, value: f32, step: f32) -> Option<f32> {
    let mut result = None;
    ui.same_line_with_spacing(0.0, BTN_SPACING);
    if ui.button_with_size(format!("-##{label}"), [BTN_W, 0.0]) {
        result = Some(value - step);
    }
    ui.same_line_with_spacing(0.0, BTN_SPACING);
    if ui.button_with_size(format!("+##{label}"), [BTN_W, 0.0]) {
        result = Some(value + step);
    }
    result
}

/// Float input with step buttons. Returns `Some(committed_value)` on deactivation or step click.
pub(super) fn draw_float_input(
    ui: &imgui::Ui,
    label: &str,
    current: f32,
    step: f32,
) -> Option<f32> {
    let (visible_label, _) = split_imgui_label(label);
    let hidden_label = hidden_numeric_label(label);
    ui.set_next_item_width(numeric_input_width(ui, visible_label));
    let mut value = current;
    ui.input_float(hidden_label.as_str(), &mut value)
        .enter_returns_true(true)
        .build();
    let mut result = ui.is_item_deactivated_after_edit().then_some(value);
    if let Some(stepped) = draw_step_buttons(ui, label, current, step) {
        result = Some(stepped);
    }
    draw_trailing_numeric_label(ui, visible_label);
    result
}

/// Drag float input with step buttons. Returns `Some(committed_value)` on deactivation or step click.
#[allow(clippy::too_many_arguments)]
pub(super) fn draw_drag_float_input(
    ui: &imgui::Ui,
    label: &str,
    current: f32,
    step: f32,
    speed: f32,
) -> Option<f32> {
    let (visible_label, _) = split_imgui_label(label);
    let hidden_label = hidden_numeric_label(label);
    ui.set_next_item_width(numeric_input_width(ui, visible_label));
    let mut value = current;
    imgui::Drag::new(hidden_label.as_str())
        .speed(speed)
        .display_format("%.2f")
        .build(ui, &mut value);
    let mut result = ui.is_item_deactivated_after_edit().then_some(value);
    if let Some(stepped) = draw_step_buttons(ui, label, current, step) {
        result = Some(stepped);
    }
    draw_trailing_numeric_label(ui, visible_label);
    result
}

/// Integer input. Returns `Some(committed_value)` on deactivation.
pub(super) fn draw_int_input(ui: &imgui::Ui, label: &str, current: i32) -> Option<i32> {
    let (visible_label, _) = split_imgui_label(label);
    let hidden_label = hidden_numeric_label(label);
    ui.set_next_item_width(numeric_input_width(ui, visible_label));
    let mut value = current;
    ui.input_int(hidden_label.as_str(), &mut value)
        .enter_returns_true(true)
        .build();
    let result = ui.is_item_deactivated_after_edit().then_some(value);
    draw_trailing_numeric_label(ui, visible_label);
    result
}

/// Text input. Updates `pending` live on every keystroke; sets `*committed = true` on deactivation.
pub(super) fn draw_text_buffer_input(
    ui: &imgui::Ui,
    label: &str,
    pending: &mut Option<String>,
    committed: &mut bool,
    snapshot_value: &str,
) {
    let mut buffer = pending.clone().unwrap_or_else(|| snapshot_value.to_owned());
    if ui.input_text(label, &mut buffer).build() {
        *pending = Some(buffer.clone());
    }
    if ui.is_item_deactivated_after_edit() {
        *pending = Some(buffer);
        *committed = true;
    }
}

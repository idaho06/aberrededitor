use aberredengine::imgui;
use aberredengine::resources::worldsignals::WorldSignals;

pub(super) const BTN_W: f32 = 22.0;
pub(super) const BTN_SPACING: f32 = 4.0;
const MIN_NUMERIC_INPUT_W: f32 = 96.0;

fn split_imgui_label(label: &str) -> (&str, &str) {
    label.split_once("##").unwrap_or((label, ""))
}

fn hidden_numeric_label(label: &str) -> String {
    let (_, id_suffix) = split_imgui_label(label);
    if id_suffix.is_empty() {
        format!("##{label}")
    } else {
        format!("##{id_suffix}")
    }
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

/// Renders − and + step buttons after the previously rendered widget (same line).
pub(super) fn draw_step_buttons(
    ui: &imgui::Ui,
    signals: &mut WorldSignals,
    pending_key: &str,
    value: f32,
    step: f32,
    action_key: &str,
) {
    ui.same_line_with_spacing(0.0, BTN_SPACING);
    if ui.button_with_size(format!("-##{pending_key}"), [BTN_W, 0.0]) {
        commit_scalar_signal(signals, pending_key, value - step, action_key);
    }
    ui.same_line_with_spacing(0.0, BTN_SPACING);
    if ui.button_with_size(format!("+##{pending_key}"), [BTN_W, 0.0]) {
        commit_scalar_signal(signals, pending_key, value + step, action_key);
    }
}

pub(super) fn draw_float_input(
    ui: &imgui::Ui,
    signals: &mut WorldSignals,
    label: &str,
    snapshot_value: f32,
    pending_key: &str,
    action_key: &str,
    step: f32,
) {
    let (visible_label, _) = split_imgui_label(label);
    let hidden_label = hidden_numeric_label(label);
    ui.set_next_item_width(numeric_input_width(ui, visible_label));
    let mut value = snapshot_value;
    ui.input_float(hidden_label.as_str(), &mut value)
        .enter_returns_true(true)
        .build();
    if ui.is_item_deactivated_after_edit() {
        commit_scalar_signal(signals, pending_key, value, action_key);
    }
    draw_step_buttons(ui, signals, pending_key, snapshot_value, step, action_key);
    draw_trailing_numeric_label(ui, visible_label);
}

#[allow(clippy::too_many_arguments)]
pub(super) fn draw_drag_float_input(
    ui: &imgui::Ui,
    signals: &mut WorldSignals,
    label: &str,
    snapshot_value: f32,
    pending_key: &str,
    action_key: &str,
    step: f32,
    speed: f32,
) {
    let (visible_label, _) = split_imgui_label(label);
    let hidden_label = hidden_numeric_label(label);
    ui.set_next_item_width(numeric_input_width(ui, visible_label));
    let mut value = snapshot_value;
    imgui::Drag::new(hidden_label.as_str())
        .speed(speed)
        .display_format("%.2f")
        .build(ui, &mut value);
    if ui.is_item_deactivated_after_edit() {
        commit_scalar_signal(signals, pending_key, value, action_key);
    }
    draw_step_buttons(ui, signals, pending_key, snapshot_value, step, action_key);
    draw_trailing_numeric_label(ui, visible_label);
}

pub(super) fn draw_int_input(
    ui: &imgui::Ui,
    signals: &mut WorldSignals,
    label: &str,
    snapshot_value: i32,
    pending_key: &str,
    action_key: &str,
) {
    let (visible_label, _) = split_imgui_label(label);
    let hidden_label = hidden_numeric_label(label);
    ui.set_next_item_width(numeric_input_width(ui, visible_label));
    let mut value = snapshot_value;
    ui.input_int(hidden_label.as_str(), &mut value)
        .enter_returns_true(true)
        .build();
    if ui.is_item_deactivated_after_edit() {
        commit_integer_signal(signals, pending_key, value, action_key);
    }
    draw_trailing_numeric_label(ui, visible_label);
}

pub(super) fn draw_text_buffer_input(
    ui: &imgui::Ui,
    signals: &mut WorldSignals,
    label: &str,
    snapshot_value: &str,
    buffer_key: &str,
    dirty_key: &str,
    action_key: &str,
) {
    let mut buffer = seed_text_buffer(signals, buffer_key, dirty_key, snapshot_value);
    if ui.input_text(label, &mut buffer).build() {
        signals.set_string(buffer_key, buffer.as_str());
        signals.set_flag(dirty_key);
    }
    if ui.is_item_deactivated_after_edit() {
        signals.set_string(buffer_key, buffer.as_str());
        signals.set_flag(dirty_key);
        signals.set_flag(action_key);
    }
}

pub(super) fn seed_text_buffer(
    signals: &WorldSignals,
    buffer_key: &str,
    dirty_key: &str,
    snapshot_value: &str,
) -> String {
    if signals.has_flag(dirty_key) {
        signals
            .get_string(buffer_key)
            .cloned()
            .unwrap_or_else(|| snapshot_value.to_owned())
    } else {
        snapshot_value.to_owned()
    }
}

pub(super) fn commit_scalar_signal(
    signals: &mut WorldSignals,
    pending_key: &str,
    value: f32,
    action_key: &str,
) {
    signals.set_scalar(pending_key, value);
    signals.set_flag(action_key);
}

pub(super) fn commit_bool_flag(
    signals: &mut WorldSignals,
    pending_key: &str,
    dirty_key: &str,
    value: bool,
    action_key: &str,
) {
    if value {
        signals.set_flag(pending_key);
    } else {
        signals.clear_flag(pending_key);
    }
    signals.set_flag(dirty_key);
    signals.set_flag(action_key);
}

pub(super) fn commit_integer_signal(
    signals: &mut WorldSignals,
    pending_key: &str,
    value: i32,
    action_key: &str,
) {
    signals.set_integer(pending_key, value);
    signals.set_flag(action_key);
}

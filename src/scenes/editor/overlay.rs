//! In-world selection overlay rendering via ImGui draw list.
//!
//! `draw_selection_outline` reads `SelectionCorners` from `AppState` (world-space quad) and draws
//! the active entity outline. `draw_multi_entity_outlines` draws one outline per entity in a
//! multi-selection. `draw_selection_drag_overlay` draws the in-progress rectangle marquee for
//! rectangle-selection mode using render-target drag points stored in `AppState`.
use super::current_selection_drag;
use crate::editor_types::SelectionCorners;
use crate::signals as sig;
use crate::systems::entity_selector::MultiEntitySelectionMutex;
use aberredengine::imgui;
use aberredengine::resources::appstate::AppState;
use aberredengine::resources::worldsignals::WorldSignals;

pub(super) fn draw_selection_outline(ui: &imgui::Ui, signals: &WorldSignals, app_state: &AppState) {
    let Some(corners) = app_state.get::<SelectionCorners>() else {
        return;
    };
    let points = corners
        .0
        .map(|[world_x, world_y]| world_to_screen(signals, world_x, world_y));
    draw_quad_outline(&ui.get_background_draw_list(), points);
}

pub(super) fn draw_multi_entity_outlines(
    ui: &imgui::Ui,
    signals: &WorldSignals,
    app_state: &AppState,
) {
    if !signals.has_flag(sig::UI_MULTI_ENTITY_SELECTOR_OPEN) {
        return;
    }
    let Some(mutex) = app_state.get::<MultiEntitySelectionMutex>() else {
        return;
    };
    let Ok(cache) = mutex.lock() else {
        return;
    };
    let draw_list = ui.get_background_draw_list();
    for corners in cache.corner_sets.iter().flatten() {
        let points = corners.map(|[world_x, world_y]| world_to_screen(signals, world_x, world_y));
        draw_quad_outline(&draw_list, points);
    }
}

fn draw_quad_outline(draw_list: &imgui::DrawListMut<'_>, points: [[f32; 2]; 4]) {
    const COLOR: [f32; 4] = [1.0, 0.85, 0.0, 1.0];
    for i in 0..4 {
        draw_list
            .add_line(points[i], points[(i + 1) % 4], COLOR)
            .thickness(2.0)
            .build();
    }
}

pub(super) fn draw_selection_drag_overlay(
    ui: &imgui::Ui,
    signals: &WorldSignals,
    app_state: &AppState,
) {
    let Some(drag_rect) = current_selection_drag(app_state) else {
        return;
    };

    let start = render_to_screen(signals, drag_rect.start[0], drag_rect.start[1]);
    let current = render_to_screen(signals, drag_rect.current[0], drag_rect.current[1]);
    let min_x = start[0].min(current[0]);
    let min_y = start[1].min(current[1]);
    let max_x = start[0].max(current[0]);
    let max_y = start[1].max(current[1]);
    let color = [0.6_f32, 0.6, 0.6, 1.0];

    draw_dotted_line(ui, [min_x, min_y], [max_x, min_y], color);
    draw_dotted_line(ui, [max_x, min_y], [max_x, max_y], color);
    draw_dotted_line(ui, [max_x, max_y], [min_x, max_y], color);
    draw_dotted_line(ui, [min_x, max_y], [min_x, min_y], color);
}

pub(super) fn render_to_world(signals: &WorldSignals, render_x: f32, render_y: f32) -> [f32; 2] {
    let target_x = signals.get_scalar(sig::CAM_TARGET_X).unwrap_or(0.0);
    let target_y = signals.get_scalar(sig::CAM_TARGET_Y).unwrap_or(0.0);
    let zoom = signals.get_scalar(sig::CAM_ZOOM).unwrap_or(1.0);
    let offset_x = signals.get_scalar(sig::CAM_OFFSET_X).unwrap_or(0.0);
    let offset_y = signals.get_scalar(sig::CAM_OFFSET_Y).unwrap_or(0.0);

    [
        (render_x - offset_x) / zoom + target_x,
        (render_y - offset_y) / zoom + target_y,
    ]
}

fn world_to_screen(signals: &WorldSignals, world_x: f32, world_y: f32) -> [f32; 2] {
    let render = world_to_render(signals, world_x, world_y);
    render_to_screen(signals, render[0], render[1])
}

fn world_to_render(signals: &WorldSignals, world_x: f32, world_y: f32) -> [f32; 2] {
    let target_x = signals.get_scalar(sig::CAM_TARGET_X).unwrap_or(0.0);
    let target_y = signals.get_scalar(sig::CAM_TARGET_Y).unwrap_or(0.0);
    let zoom = signals.get_scalar(sig::CAM_ZOOM).unwrap_or(1.0);
    let offset_x = signals.get_scalar(sig::CAM_OFFSET_X).unwrap_or(0.0);
    let offset_y = signals.get_scalar(sig::CAM_OFFSET_Y).unwrap_or(0.0);
    [
        (world_x - target_x) * zoom + offset_x,
        (world_y - target_y) * zoom + offset_y,
    ]
}

fn render_to_screen(signals: &WorldSignals, render_x: f32, render_y: f32) -> [f32; 2] {
    let lb_scale = signals.get_scalar(sig::WIN_SCALE).unwrap_or(1.0);
    let lb_x = signals.get_scalar(sig::WIN_OFFSET_X).unwrap_or(0.0);
    let lb_y = signals.get_scalar(sig::WIN_OFFSET_Y).unwrap_or(0.0);
    [render_x * lb_scale + lb_x, render_y * lb_scale + lb_y]
}

fn draw_dotted_line(ui: &imgui::Ui, start: [f32; 2], end: [f32; 2], color: [f32; 4]) {
    const DASH_LENGTH: f32 = 6.0;
    const GAP_LENGTH: f32 = 4.0;

    let dx = end[0] - start[0];
    let dy = end[1] - start[1];
    let length = (dx * dx + dy * dy).sqrt();
    if length <= f32::EPSILON {
        return;
    }

    let step_x = dx / length;
    let step_y = dy / length;
    let draw_list = ui.get_background_draw_list();
    let mut distance = 0.0;
    while distance < length {
        let dash_end = (distance + DASH_LENGTH).min(length);
        draw_list
            .add_line(
                [start[0] + step_x * distance, start[1] + step_y * distance],
                [start[0] + step_x * dash_end, start[1] + step_y * dash_end],
                color,
            )
            .thickness(1.5)
            .build();
        distance += DASH_LENGTH + GAP_LENGTH;
    }
}

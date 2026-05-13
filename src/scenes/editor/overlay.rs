//! In-world overlay rendering and overlay settings.
//!
//! Selection outlines and the drag marquee are rendered via ImGui's background draw list.
//! This module also owns the editor-local origin-axis/grid overlay state and the grid
//! preferences modal.
use super::current_selection_drag;
use crate::editor_types::SelectionCorners;
use crate::signals as sig;
use crate::systems::entity_selector::MultiEntitySelectionMutex;
use aberredengine::imgui;
use aberredengine::resources::appstate::AppState;
use aberredengine::resources::worldsignals::WorldSignals;

pub(super) const GRID_PREFERENCES_POPUP_ID: &str = "Grid Preferences##overlay";
const ORIGIN_AXIS_COLOR: [f32; 4] = [0.7, 0.7, 0.7, 1.0];
const GRID_COLOR: [f32; 4] = [0.25, 0.25, 0.25, 1.0];
const SELECTION_OUTLINE_COLOR: [f32; 4] = [1.0, 0.85, 0.0, 1.0];
const DRAG_MARQUEE_COLOR: [f32; 4] = [0.6, 0.6, 0.6, 1.0];
const DRAG_MARQUEE_THICKNESS: f32 = 1.5;
const DRAG_MARQUEE_DASH: f32 = 6.0;
const DRAG_MARQUEE_GAP: f32 = 4.0;
const DEFAULT_GRID_WIDTH: f32 = 24.0;
const DEFAULT_GRID_HEIGHT: f32 = 24.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct GridOverlayConfig {
    pub width: f32,
    pub height: f32,
    pub offset_x: f32,
    pub offset_y: f32,
}

impl Default for GridOverlayConfig {
    fn default() -> Self {
        Self {
            width: DEFAULT_GRID_WIDTH,
            height: DEFAULT_GRID_HEIGHT,
            offset_x: 0.0,
            offset_y: 0.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct OverlaySettingsState {
    pub show_origin_axis: bool,
    pub show_grid: bool,
    pub grid: GridOverlayConfig,
    pub grid_draft: GridOverlayConfig,
}

impl Default for OverlaySettingsState {
    fn default() -> Self {
        let grid = GridOverlayConfig::default();
        Self {
            show_origin_axis: false,
            show_grid: false,
            grid,
            grid_draft: grid,
        }
    }
}

pub(crate) type OverlaySettingsMutex = std::sync::Mutex<OverlaySettingsState>;

#[derive(Clone, Copy, Debug)]
struct VisibleWorldBounds {
    min_x: f32,
    max_x: f32,
    min_y: f32,
    max_y: f32,
}

fn lock_overlay_settings(app_state: &AppState) -> std::sync::MutexGuard<'_, OverlaySettingsState> {
    app_state
        .get::<OverlaySettingsMutex>()
        .expect("OverlaySettingsMutex not in AppState")
        .lock()
        .expect("OverlaySettingsMutex poisoned")
}

pub(super) fn origin_axis_visible(app_state: &AppState) -> bool {
    lock_overlay_settings(app_state).show_origin_axis
}

pub(super) fn toggle_origin_axis(app_state: &AppState) {
    let mut state = lock_overlay_settings(app_state);
    state.show_origin_axis = !state.show_origin_axis;
}

pub(super) fn toggle_grid(app_state: &AppState) {
    let mut state = lock_overlay_settings(app_state);
    state.show_grid = !state.show_grid;
}

pub(super) fn prepare_grid_preferences(app_state: &AppState) {
    let mut state = lock_overlay_settings(app_state);
    state.grid_draft = state.grid;
}

pub(super) fn overlay_visibility(app_state: &AppState) -> (bool, bool) {
    let state = lock_overlay_settings(app_state);
    (state.show_origin_axis, state.show_grid)
}

pub(super) fn corners_aabb(corners: [[f32; 2]; 4]) -> (f32, f32, f32, f32) {
    corners.iter().fold(
        (f32::INFINITY, f32::NEG_INFINITY, f32::INFINITY, f32::NEG_INFINITY),
        |(min_x, max_x, min_y, max_y), [x, y]| {
            (min_x.min(*x), max_x.max(*x), min_y.min(*y), max_y.max(*y))
        },
    )
}

pub(super) fn draw_grid_preferences_modal(ui: &imgui::Ui, app_state: &AppState) {
    ui.modal_popup_config(GRID_PREFERENCES_POPUP_ID)
        .always_auto_resize(true)
        .resizable(false)
        .movable(false)
        .build(|| {
            let mut state = lock_overlay_settings(app_state);

            ui.text("Configure world-space grid spacing and origin offset.");
            ui.text_disabled("Width and height must be greater than 0.");
            ui.spacing();

            ui.input_float("Width##grid_width", &mut state.grid_draft.width)
                .enter_returns_true(true)
                .build();
            ui.input_float("Height##grid_height", &mut state.grid_draft.height)
                .enter_returns_true(true)
                .build();
            ui.input_float("Offset X##grid_offset_x", &mut state.grid_draft.offset_x)
                .enter_returns_true(true)
                .build();
            ui.input_float("Offset Y##grid_offset_y", &mut state.grid_draft.offset_y)
                .enter_returns_true(true)
                .build();

            let valid_spacing = state.grid_draft.width > 0.0 && state.grid_draft.height > 0.0;
            if !valid_spacing {
                ui.spacing();
                ui.text("Grid spacing must be greater than 0.");
            }

            ui.spacing();
            ui.separator();
            if ui.button("Apply##grid_preferences_apply") && valid_spacing {
                state.grid = state.grid_draft;
                ui.close_current_popup();
            }
            ui.same_line();
            if ui.button("Cancel##grid_preferences_cancel") {
                state.grid_draft = state.grid;
                ui.close_current_popup();
            }
        });
}

pub(super) fn draw_origin_axis_overlay(
    ui: &imgui::Ui,
    signals: &WorldSignals,
    app_state: &AppState,
) {
    if !origin_axis_visible(app_state) {
        return;
    }
    let Some(bounds) = visible_world_bounds(signals) else {
        return;
    };
    let draw_list = ui.get_background_draw_list();

    if let Some([start, end]) =
        clip_world_segment_to_screen(signals, [bounds.min_x, 0.0], [bounds.max_x, 0.0])
    {
        draw_line(&draw_list, start, end, ORIGIN_AXIS_COLOR, 1.0);
    }
    if let Some([start, end]) =
        clip_world_segment_to_screen(signals, [0.0, bounds.min_y], [0.0, bounds.max_y])
    {
        draw_line(&draw_list, start, end, ORIGIN_AXIS_COLOR, 1.0);
    }
}

pub(super) fn draw_grid_overlay(ui: &imgui::Ui, signals: &WorldSignals, app_state: &AppState) {
    let state = lock_overlay_settings(app_state);
    if !state.show_grid {
        return;
    }
    let grid = state.grid;
    drop(state);
    if grid.width <= 0.0 || grid.height <= 0.0 {
        return;
    }

    let Some(bounds) = visible_world_bounds(signals) else {
        return;
    };
    let draw_list = ui.get_background_draw_list();

    let mut x = aligned_grid_line_start(bounds.min_x, grid.offset_x, grid.width);
    while x <= bounds.max_x + f32::EPSILON {
        if let Some([start, end]) =
            clip_world_segment_to_screen(signals, [x, bounds.min_y], [x, bounds.max_y])
        {
            draw_line(&draw_list, start, end, GRID_COLOR, 1.0);
        }
        x += grid.width;
    }

    let mut y = aligned_grid_line_start(bounds.min_y, grid.offset_y, grid.height);
    while y <= bounds.max_y + f32::EPSILON {
        if let Some([start, end]) =
            clip_world_segment_to_screen(signals, [bounds.min_x, y], [bounds.max_x, y])
        {
            draw_line(&draw_list, start, end, GRID_COLOR, 1.0);
        }
        y += grid.height;
    }
}

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
    for i in 0..4 {
        draw_line(
            draw_list,
            points[i],
            points[(i + 1) % 4],
            SELECTION_OUTLINE_COLOR,
            2.0,
        );
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
    let ([min_rx, min_ry], [max_rx, max_ry]) = drag_rect.normalized();
    let [min_x, min_y] = render_to_screen(signals, min_rx, min_ry);
    let [max_x, max_y] = render_to_screen(signals, max_rx, max_ry);
    let draw_list = ui.get_background_draw_list();
    draw_dotted_line(&draw_list, [min_x, min_y], [max_x, min_y], DRAG_MARQUEE_COLOR, DRAG_MARQUEE_THICKNESS, DRAG_MARQUEE_DASH, DRAG_MARQUEE_GAP);
    draw_dotted_line(&draw_list, [max_x, min_y], [max_x, max_y], DRAG_MARQUEE_COLOR, DRAG_MARQUEE_THICKNESS, DRAG_MARQUEE_DASH, DRAG_MARQUEE_GAP);
    draw_dotted_line(&draw_list, [max_x, max_y], [min_x, max_y], DRAG_MARQUEE_COLOR, DRAG_MARQUEE_THICKNESS, DRAG_MARQUEE_DASH, DRAG_MARQUEE_GAP);
    draw_dotted_line(&draw_list, [min_x, max_y], [min_x, min_y], DRAG_MARQUEE_COLOR, DRAG_MARQUEE_THICKNESS, DRAG_MARQUEE_DASH, DRAG_MARQUEE_GAP);
}

struct CameraParams {
    target_x: f32,
    target_y: f32,
    zoom: f32,
    rotation_rad: f32,
    offset_x: f32,
    offset_y: f32,
}

impl CameraParams {
    fn from_signals(signals: &WorldSignals) -> Self {
        Self {
            target_x: signals.get_scalar(sig::CAM_TARGET_X).unwrap_or(0.0),
            target_y: signals.get_scalar(sig::CAM_TARGET_Y).unwrap_or(0.0),
            zoom: signals.get_scalar(sig::CAM_ZOOM).unwrap_or(1.0),
            rotation_rad: signals.get_scalar(sig::CAM_ROTATION).unwrap_or(0.0).to_radians(),
            offset_x: signals.get_scalar(sig::CAM_OFFSET_X).unwrap_or(0.0),
            offset_y: signals.get_scalar(sig::CAM_OFFSET_Y).unwrap_or(0.0),
        }
    }
}

pub(super) fn render_to_world(signals: &WorldSignals, render_x: f32, render_y: f32) -> [f32; 2] {
    let cam = CameraParams::from_signals(signals);
    let translated_x = (render_x - cam.offset_x) / cam.zoom;
    let translated_y = (render_y - cam.offset_y) / cam.zoom;
    let cos_a = cam.rotation_rad.cos();
    let sin_a = cam.rotation_rad.sin();
    [
        translated_x * cos_a - translated_y * sin_a + cam.target_x,
        translated_x * sin_a + translated_y * cos_a + cam.target_y,
    ]
}

fn world_to_screen(signals: &WorldSignals, world_x: f32, world_y: f32) -> [f32; 2] {
    let render = world_to_render(signals, world_x, world_y);
    render_to_screen(signals, render[0], render[1])
}

fn world_to_render(signals: &WorldSignals, world_x: f32, world_y: f32) -> [f32; 2] {
    let cam = CameraParams::from_signals(signals);
    let rotation_rad = -cam.rotation_rad;
    let dx = world_x - cam.target_x;
    let dy = world_y - cam.target_y;
    let cos_a = rotation_rad.cos();
    let sin_a = rotation_rad.sin();
    [
        (dx * cos_a - dy * sin_a) * cam.zoom + cam.offset_x,
        (dx * sin_a + dy * cos_a) * cam.zoom + cam.offset_y,
    ]
}

fn render_to_screen(signals: &WorldSignals, render_x: f32, render_y: f32) -> [f32; 2] {
    let lb_scale = signals.get_scalar(sig::WIN_SCALE).unwrap_or(1.0);
    let lb_x = signals.get_scalar(sig::WIN_OFFSET_X).unwrap_or(0.0);
    let lb_y = signals.get_scalar(sig::WIN_OFFSET_Y).unwrap_or(0.0);
    [render_x * lb_scale + lb_x, render_y * lb_scale + lb_y]
}

fn render_size(signals: &WorldSignals) -> Option<[f32; 2]> {
    let width = signals.get_scalar(sig::RENDER_WIDTH)?;
    let height = signals.get_scalar(sig::RENDER_HEIGHT)?;
    (width > 0.0 && height > 0.0).then_some([width, height])
}

fn visible_world_bounds(signals: &WorldSignals) -> Option<VisibleWorldBounds> {
    let [render_width, render_height] = render_size(signals)?;
    let corners = [
        render_to_world(signals, 0.0, 0.0),
        render_to_world(signals, render_width, 0.0),
        render_to_world(signals, render_width, render_height),
        render_to_world(signals, 0.0, render_height),
    ];
    let (min_x, max_x, min_y, max_y) = corners_aabb(corners);
    Some(VisibleWorldBounds { min_x, max_x, min_y, max_y })
}

fn clip_world_segment_to_screen(
    signals: &WorldSignals,
    world_start: [f32; 2],
    world_end: [f32; 2],
) -> Option<[[f32; 2]; 2]> {
    let [render_width, render_height] = render_size(signals)?;
    let render_start = world_to_render(signals, world_start[0], world_start[1]);
    let render_end = world_to_render(signals, world_end[0], world_end[1]);
    let [clipped_start, clipped_end] = clip_segment_to_rect(
        render_start,
        render_end,
        0.0,
        0.0,
        render_width,
        render_height,
    )?;

    Some([
        render_to_screen(signals, clipped_start[0], clipped_start[1]),
        render_to_screen(signals, clipped_end[0], clipped_end[1]),
    ])
}

fn clip_segment_to_rect(
    start: [f32; 2],
    end: [f32; 2],
    min_x: f32,
    min_y: f32,
    max_x: f32,
    max_y: f32,
) -> Option<[[f32; 2]; 2]> {
    let dx = end[0] - start[0];
    let dy = end[1] - start[1];
    let mut t0 = 0.0_f32;
    let mut t1 = 1.0_f32;

    for (p, q) in [
        (-dx, start[0] - min_x),
        (dx, max_x - start[0]),
        (-dy, start[1] - min_y),
        (dy, max_y - start[1]),
    ] {
        if p.abs() <= f32::EPSILON {
            if q < 0.0 {
                return None;
            }
            continue;
        }

        let r = q / p;
        if p < 0.0 {
            if r > t1 {
                return None;
            }
            if r > t0 {
                t0 = r;
            }
        } else {
            if r < t0 {
                return None;
            }
            if r < t1 {
                t1 = r;
            }
        }
    }

    Some([
        [start[0] + dx * t0, start[1] + dy * t0],
        [start[0] + dx * t1, start[1] + dy * t1],
    ])
}

fn aligned_grid_line_start(min_value: f32, offset: f32, spacing: f32) -> f32 {
    offset + ((min_value - offset) / spacing).floor() * spacing
}

fn draw_line(
    draw_list: &imgui::DrawListMut<'_>,
    start: [f32; 2],
    end: [f32; 2],
    color: [f32; 4],
    thickness: f32,
) {
    draw_list
        .add_line(start, end, color)
        .thickness(thickness)
        .build();
}

fn draw_dotted_line(
    draw_list: &imgui::DrawListMut<'_>,
    start: [f32; 2],
    end: [f32; 2],
    color: [f32; 4],
    thickness: f32,
    dash_length: f32,
    gap_length: f32,
) {
    let dx = end[0] - start[0];
    let dy = end[1] - start[1];
    let length = (dx * dx + dy * dy).sqrt();
    if length <= f32::EPSILON {
        return;
    }

    let step_x = dx / length;
    let step_y = dy / length;
    let mut distance = 0.0;
    while distance < length {
        let dash_end = (distance + dash_length).min(length);
        draw_line(
            draw_list,
            [start[0] + step_x * distance, start[1] + step_y * distance],
            [start[0] + step_x * dash_end, start[1] + step_y * dash_end],
            color,
            thickness,
        );
        distance += dash_length + gap_length;
    }
}

//! In-world overlay rendering and overlay settings.
//!
//! Grid and origin-axis overlays are drawn via Raylib world-space (`draw_world_overlays`).
//! Selection outlines and the drag marquee use ImGui's background draw list.
//! This module also owns the overlay state (`OverlaySettingsMutex`) and the grid preferences modal.
use super::current_selection_drag;
use crate::editor_types::SelectionCorners;
use crate::signals as sig;
use crate::systems::entity_selector::MultiEntitySelectionMutex;
use aberredengine::imgui;
use aberredengine::resources::appstate::AppState;
use aberredengine::resources::screensize::ScreenSize;
use aberredengine::resources::worldsignals::WorldSignals;
use aberredengine::systems::scene_dispatch::WorldDraw;
use aberredengine::raylib::prelude::{Camera2D, Color, Vector2};

pub(super) const GRID_PREFERENCES_POPUP_ID: &str = "Grid Preferences##overlay";
const ORIGIN_AXIS_COLOR: Color = Color { r: 178, g: 178, b: 178, a: 255 };
const GRID_COLOR: Color = Color { r: 64, g: 64, b: 64, a: 255 };
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

fn lock_overlay_settings(app_state: &AppState) -> std::sync::MutexGuard<'_, OverlaySettingsState> {
    app_state
        .get::<OverlaySettingsMutex>()
        .expect("OverlaySettingsMutex not in AppState")
        .lock()
        .expect("OverlaySettingsMutex poisoned")
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

pub(crate) fn draw_world_overlays(
    d: &mut dyn WorldDraw,
    camera: &Camera2D,
    screen: &ScreenSize,
    app_state: &AppState,
    _signals: &WorldSignals,
) {
    let state = lock_overlay_settings(app_state);
    let show_axis = state.show_origin_axis;
    let show_grid = state.show_grid;
    let grid = state.grid;
    drop(state);

    let grid_active = show_grid && grid.width > 0.0 && grid.height > 0.0;
    if !show_axis && !grid_active {
        return;
    }

    let bounds = world_bounds_from_camera(camera, screen);
    if grid_active {
        draw_grid_lines(d, bounds, grid);
    }
    if show_axis {
        draw_axis_lines(d, bounds);
    }
}

fn world_bounds_from_camera(camera: &Camera2D, screen: &ScreenSize) -> (f32, f32, f32, f32) {
    let w = screen.w as f32;
    let h = screen.h as f32;
    let rotation_rad = camera.rotation.to_radians();
    let cos_a = rotation_rad.cos();
    let sin_a = rotation_rad.sin();
    let to_world = |rx: f32, ry: f32| -> [f32; 2] {
        let tx = (rx - camera.offset.x) / camera.zoom;
        let ty = (ry - camera.offset.y) / camera.zoom;
        [
            tx * cos_a - ty * sin_a + camera.target.x,
            tx * sin_a + ty * cos_a + camera.target.y,
        ]
    };
    let corners = [
        to_world(0.0, 0.0),
        to_world(w, 0.0),
        to_world(w, h),
        to_world(0.0, h),
    ];
    corners_aabb(corners)
}

fn draw_axis_lines(
    d: &mut dyn WorldDraw,
    (min_x, max_x, min_y, max_y): (f32, f32, f32, f32),
) {
    let color = ORIGIN_AXIS_COLOR;
    d.draw_line_v(Vector2::new(min_x, 0.0), Vector2::new(max_x, 0.0), color);
    d.draw_line_v(Vector2::new(0.0, min_y), Vector2::new(0.0, max_y), color);
}

fn draw_grid_lines(
    d: &mut dyn WorldDraw,
    (min_x, max_x, min_y, max_y): (f32, f32, f32, f32),
    grid: GridOverlayConfig,
) {
    // Guard against runaway loops if the user sets a very small grid spacing.
    const MAX_LINES: f32 = 2000.0;
    if (max_x - min_x) / grid.width > MAX_LINES || (max_y - min_y) / grid.height > MAX_LINES {
        return;
    }

    let color = GRID_COLOR;
    let mut x = aligned_grid_line_start(min_x, grid.offset_x, grid.width);
    while x <= max_x + grid.width * 0.5 {
        d.draw_line_v(Vector2::new(x, min_y), Vector2::new(x, max_y), color);
        x += grid.width;
    }

    let mut y = aligned_grid_line_start(min_y, grid.offset_y, grid.height);
    while y <= max_y + grid.height * 0.5 {
        d.draw_line_v(Vector2::new(min_x, y), Vector2::new(max_x, y), color);
        y += grid.height;
    }
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
    let side = |a, b| draw_dotted_line(&draw_list, a, b, DRAG_MARQUEE_COLOR, DRAG_MARQUEE_THICKNESS, DRAG_MARQUEE_DASH, DRAG_MARQUEE_GAP);
    side([min_x, min_y], [max_x, min_y]);
    side([max_x, min_y], [max_x, max_y]);
    side([max_x, max_y], [min_x, max_y]);
    side([min_x, max_y], [min_x, min_y]);
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

fn render_to_screen(signals: &WorldSignals, render_x: f32, render_y: f32) -> [f32; 2] {
    let lb_scale = signals.get_scalar(sig::WIN_SCALE).unwrap_or(1.0);
    let lb_x = signals.get_scalar(sig::WIN_OFFSET_X).unwrap_or(0.0);
    let lb_y = signals.get_scalar(sig::WIN_OFFSET_Y).unwrap_or(0.0);
    [render_x * lb_scale + lb_x, render_y * lb_scale + lb_y]
}

fn world_to_screen(signals: &WorldSignals, world_x: f32, world_y: f32) -> [f32; 2] {
    let cam = CameraParams::from_signals(signals);
    let rotation_rad = -cam.rotation_rad;
    let dx = world_x - cam.target_x;
    let dy = world_y - cam.target_y;
    let cos_a = rotation_rad.cos();
    let sin_a = rotation_rad.sin();
    let render_x = (dx * cos_a - dy * sin_a) * cam.zoom + cam.offset_x;
    let render_y = (dx * sin_a + dy * cos_a) * cam.zoom + cam.offset_y;
    render_to_screen(signals, render_x, render_y)
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

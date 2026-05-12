//! Shared selection-mode and rectangle-drag state for editor GUI and update logic.
//!
//! The current mode lives in `AppState` rather than a Bevy resource because both the GUI
//! callback and the scene update path need direct access to it.
use aberredengine::resources::appstate::AppState;

/// Active entity-selection interaction mode.
#[derive(Clone, Copy, Default, Eq, PartialEq)]
pub enum SelectionMode {
    #[default]
    Click,
    Rectangle,
}

/// Current rectangle-drag endpoints in render-target space.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SelectionDragRect {
    pub start: [f32; 2],
    pub current: [f32; 2],
}

impl SelectionDragRect {
    pub fn normalized(self) -> ([f32; 2], [f32; 2]) {
        (
            [
                self.start[0].min(self.current[0]),
                self.start[1].min(self.current[1]),
            ],
            [
                self.start[0].max(self.current[0]),
                self.start[1].max(self.current[1]),
            ],
        )
    }
}

/// Shared selection settings stored in `AppState`.
#[derive(Default)]
pub struct SelectionModeState {
    pub mode: SelectionMode,
    pub drag_rect: Option<SelectionDragRect>,
}

/// `AppState` key for editor selection mode.
pub type SelectionModeMutex = std::sync::Mutex<SelectionModeState>;

fn lock_mode_state(app_state: &AppState) -> std::sync::MutexGuard<'_, SelectionModeState> {
    app_state
        .get::<SelectionModeMutex>()
        .expect("SelectionModeMutex not in AppState")
        .lock()
        .expect("SelectionModeMutex poisoned")
}

pub fn current_selection_mode(app_state: &AppState) -> SelectionMode {
    lock_mode_state(app_state).mode
}

pub fn set_selection_mode(app_state: &AppState, mode: SelectionMode) {
    let mut state = lock_mode_state(app_state);
    if state.mode != mode {
        state.drag_rect = None;
    }
    state.mode = mode;
}

pub fn current_selection_drag(app_state: &AppState) -> Option<SelectionDragRect> {
    lock_mode_state(app_state).drag_rect
}

pub fn start_selection_drag(app_state: &AppState, point: [f32; 2]) {
    lock_mode_state(app_state).drag_rect = Some(SelectionDragRect {
        start: point,
        current: point,
    });
}

pub fn update_selection_drag(app_state: &AppState, point: [f32; 2]) {
    if let Some(drag_rect) = lock_mode_state(app_state).drag_rect.as_mut() {
        drag_rect.current = point;
    }
}

pub fn finish_selection_drag(app_state: &AppState, point: [f32; 2]) -> Option<SelectionDragRect> {
    let mut state = lock_mode_state(app_state);
    if let Some(drag_rect) = state.drag_rect.as_mut() {
        drag_rect.current = point;
    }
    state.drag_rect.take()
}

pub fn reset_selection_mode(app_state: &AppState) {
    *lock_mode_state(app_state) = SelectionModeState::default();
}

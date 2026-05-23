//! Active editor tool state, rectangle-drag tracking, and tool-transition helpers.
//!
//! The active tool lives in `AppState` rather than a Bevy resource because both the GUI
//! callback and the scene update path need direct access to it.
use aberredengine::raylib::ffi::{MouseCursor, SetMouseCursor};
use aberredengine::resources::appstate::AppState;

/// Active editor tool (selection modes and entity-placement modes).
#[derive(Clone, Copy, Default, Eq, PartialEq, Debug)]
pub enum EditorTool {
    #[default]
    Click,
    Rectangle,
    AddEntity,
    AddCollider,
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

/// Active tool state and current drag rectangle, stored in `AppState`.
#[derive(Default)]
pub struct EditorToolState {
    pub mode: EditorTool,
    pub drag_rect: Option<SelectionDragRect>,
}

/// `AppState` key for active editor tool state.
pub type EditorToolMutex = std::sync::Mutex<EditorToolState>;

fn lock_mode_state(app_state: &AppState) -> std::sync::MutexGuard<'_, EditorToolState> {
    app_state
        .get::<EditorToolMutex>()
        .expect("EditorToolMutex not in AppState")
        .lock()
        .expect("EditorToolMutex poisoned")
}

pub fn current_tool(app_state: &AppState) -> EditorTool {
    lock_mode_state(app_state).mode
}

pub fn set_tool(app_state: &AppState, mode: EditorTool) {
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

pub fn reset_tool(app_state: &AppState) {
    *lock_mode_state(app_state) = EditorToolState::default();
    unsafe { SetMouseCursor(MouseCursor::MOUSE_CURSOR_DEFAULT as i32); }
}

/// Enter a placement mode (AddEntity / AddCollider): sets the mode and switches the cursor to a crosshair.
pub fn enter_placement_mode(app_state: &AppState, mode: EditorTool) {
    debug_assert!(
        matches!(mode, EditorTool::AddEntity | EditorTool::AddCollider),
        "enter_placement_mode called with non-placement tool {mode:?}"
    );
    set_tool(app_state, mode);
    unsafe { SetMouseCursor(MouseCursor::MOUSE_CURSOR_CROSSHAIR as i32); }
}

/// Exit any placement mode: resets to Click and restores the default cursor.
pub fn exit_placement_mode(app_state: &AppState) {
    set_tool(app_state, EditorTool::Click);
    unsafe { SetMouseCursor(MouseCursor::MOUSE_CURSOR_DEFAULT as i32); }
}

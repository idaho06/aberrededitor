//! Async native file-dialog bridge.
//!
//! `editor_update` runs every frame and must not block on native file dialogs. This module
//! keeps a single dialog in flight, awaits the `rfd::AsyncFileDialog` future on a worker
//! thread, and re-emits the completion into the existing ECS event flow from a per-frame
//! system.
//!
//! Architecture:
//! - `editor_update` consumes a GUI-originated signal and calls [`request_async_dialog`].
//! - [`request_async_dialog`] creates one `rfd::AsyncFileDialog` future for the requested
//!   action and stores a receiver in [`AsyncFileDialogState`] inside `AppState`.
//! - A worker thread awaits that future with a small `futures` executor and sends the result
//!   back through a `crossbeam-channel` receiver.
//! - [`poll_async_dialogs`] runs every frame, drains any completed result, normalizes paths via
//!   `to_relative()`, updates `WorldSignals` when necessary, and triggers the same domain
//!   events that the old blocking flow used.
//!
//! The bridge is intentionally narrow: it handles only native file picker orchestration.
//! Map loading, saving, texture/font insertion, and tilemap loading still happen in their
//! existing observers. This keeps file dialogs as an orchestration concern instead of mixing
//! them into asset or map logic.
//!
//! Operational rules:
//! - Only one native dialog may be in flight at a time.
//! - Canceling a dialog yields `None` and is treated as a no-op.
//! - Clearing the bridge on scene exit drops the receiver so a late completion is ignored
//!   instead of mutating editor state after the scene is gone.
use crate::signals as sig;
use crate::systems::map_ops::{
    AddFontRequested, AddTextureRequested, LoadMapRequested, SaveMapRequested,
};
use crate::systems::tilemap_load::LoadTilemapRequested;
use crate::systems::utils::to_relative;
use aberredengine::bevy_ecs::prelude::{Commands, Res, ResMut};
use aberredengine::resources::appstate::AppState;
use aberredengine::resources::worldsignals::WorldSignals;
use crossbeam_channel::{self, Receiver, TryRecvError};
use log::{debug, warn};
use std::future::Future;
use std::pin::Pin;
use std::sync::Mutex;

/// User-intent dialog requests emitted from the editor scene.
///
/// These variants represent the dialog operation itself, not the downstream ECS mutation.
/// The completion path converts them back into existing domain events such as
/// `LoadMapRequested` or `AddTextureRequested`.
#[derive(Debug)]
pub enum AsyncFileDialogRequest {
    /// Pick a `.map` file to load and update `MAP_CURRENT_PATH` on success.
    OpenMap,
    /// Pick a `.map` output path and update `MAP_CURRENT_PATH` on success.
    SaveMapAs,
    /// Pick a folder to load as a tilemap root.
    LoadTilemapFolder,
    /// Pick an image path for a texture key that was already collected from GUI state.
    AddTexture { key: String },
    /// Pick a font path for a font key and size that were already collected from GUI state.
    AddFont { key: String, font_size: f32 },
}

/// Normalized dialog completions consumed by [`poll_async_dialogs`].
///
/// These still carry absolute paths when they leave the worker thread. Path normalization is
/// deferred to the main ECS world so the relative-path invariant stays enforced in one place.
#[derive(Debug)]
pub enum AsyncFileDialogResult {
    /// Successful map-open selection.
    OpenMap { path: String },
    /// Successful save-as selection.
    SaveMapAs { path: String },
    /// Successful tilemap folder selection.
    LoadTilemapFolder { path: String },
    /// Successful texture file selection.
    AddTexture { key: String, path: String },
    /// Successful font file selection.
    AddFont {
        key: String,
        path: String,
        font_size: f32,
    },
}

/// Shared `AppState` cache for dialog orchestration.
///
/// The bridge stores at most one receiver because the editor allows only one native dialog to
/// be open at a time. `None` means there is no dialog currently in flight.
#[derive(Default)]
pub struct AsyncFileDialogState {
    receiver: Option<Receiver<Option<AsyncFileDialogResult>>>,
}

/// `AppState` handle for the async dialog bridge.
pub type AsyncFileDialogMutex = Mutex<AsyncFileDialogState>;

type DialogTask = Pin<Box<dyn Future<Output = Option<AsyncFileDialogResult>> + Send>>;

/// Start one async native dialog if none is already in flight.
///
/// Silently ignored when the bridge state is missing or another dialog is still pending.
/// The dialog future is created before leaving the caller thread; the worker thread only
/// awaits it and forwards the result back to the ECS world.
pub fn request_async_dialog(app_state: &AppState, request: AsyncFileDialogRequest) {
    let Some(mutex) = app_state.get::<AsyncFileDialogMutex>() else {
        warn!("request_async_dialog: missing AsyncFileDialogMutex");
        return;
    };

    let mut state = mutex.lock().unwrap();
    if state.receiver.is_some() {
        debug!("request_async_dialog: native dialog already in flight");
        return;
    }

    let task = build_dialog_task(request);
    let (sender, receiver) = crossbeam_channel::unbounded();
    std::thread::spawn(move || {
        let result = futures::executor::block_on(task);
        if sender.send(result).is_err() {
            debug!("request_async_dialog: receiver dropped before dialog completion");
        }
    });
    state.receiver = Some(receiver);
}

/// Drop any tracked dialog receiver so late completions are discarded after scene exit.
pub fn clear_async_dialog(app_state: &AppState) {
    let Some(mutex) = app_state.get::<AsyncFileDialogMutex>() else {
        return;
    };
    let mut state = mutex.lock().unwrap();
    state.receiver = None;
}

/// Drain any completed dialog and translate it back into the existing ECS event flow.
///
/// This system is registered in `main.rs` and runs every frame. It is intentionally cheap when
/// idle: a missing mutex, no receiver, or an empty channel all return immediately.
///
/// Successful completions are converted to relative paths here, then forwarded as the same
/// events the editor used before the async migration.
pub fn poll_async_dialogs(
    mut commands: Commands,
    mut world_signals: ResMut<WorldSignals>,
    app_state: Res<AppState>,
) {
    let Some(result) = drain_completed(&app_state) else {
        return;
    };

    match result {
        AsyncFileDialogResult::OpenMap { path } => {
            let path = to_relative(&path);
            world_signals.set_string(sig::MAP_CURRENT_PATH, path.clone());
            commands.trigger(LoadMapRequested { path });
        }
        AsyncFileDialogResult::SaveMapAs { path } => {
            let path = to_relative(&path);
            world_signals.set_string(sig::MAP_CURRENT_PATH, path.clone());
            commands.trigger(SaveMapRequested { path });
        }
        AsyncFileDialogResult::LoadTilemapFolder { path } => {
            commands.trigger(LoadTilemapRequested {
                path: to_relative(&path),
            });
        }
        AsyncFileDialogResult::AddTexture { key, path } => {
            commands.trigger(AddTextureRequested {
                key,
                path: to_relative(&path),
            });
        }
        AsyncFileDialogResult::AddFont {
            key,
            path,
            font_size,
        } => {
            commands.trigger(AddFontRequested {
                key,
                path: to_relative(&path),
                font_size,
            });
        }
    }
}

fn drain_completed(app_state: &AppState) -> Option<AsyncFileDialogResult> {
    let mutex = app_state.get::<AsyncFileDialogMutex>()?;
    let mut state = mutex.lock().unwrap();
    let receiver = state.receiver.as_ref()?;

    match receiver.try_recv() {
        Ok(result) => {
            state.receiver = None;
            result
        }
        Err(TryRecvError::Empty) => None,
        Err(TryRecvError::Disconnected) => {
            warn!("poll_async_dialogs: dialog worker disconnected");
            state.receiver = None;
            None
        }
    }
}

/// Build the concrete `rfd::AsyncFileDialog` future for one request.
///
/// This function owns dialog construction; [`poll_async_dialogs`] owns ECS mutation and path
/// normalization.
fn build_dialog_task(request: AsyncFileDialogRequest) -> DialogTask {
    let path_of = |h: rfd::FileHandle| h.path().to_string_lossy().into_owned();
    match request {
        AsyncFileDialogRequest::OpenMap => {
            let dialog = rfd::AsyncFileDialog::new()
                .add_filter("Map", &["map"])
                .pick_file();
            Box::pin(async move {
                dialog.await.map(|file| AsyncFileDialogResult::OpenMap {
                    path: path_of(file),
                })
            })
        }
        AsyncFileDialogRequest::SaveMapAs => {
            let dialog = rfd::AsyncFileDialog::new()
                .add_filter("Map", &["map"])
                .save_file();
            Box::pin(async move {
                dialog.await.map(|file| AsyncFileDialogResult::SaveMapAs {
                    path: path_of(file),
                })
            })
        }
        AsyncFileDialogRequest::LoadTilemapFolder => {
            let dialog = rfd::AsyncFileDialog::new().pick_folder();
            Box::pin(async move {
                dialog
                    .await
                    .map(|file| AsyncFileDialogResult::LoadTilemapFolder {
                        path: path_of(file),
                    })
            })
        }
        AsyncFileDialogRequest::AddTexture { key } => {
            let dialog = rfd::AsyncFileDialog::new()
                .add_filter("Image", &["png", "jpg", "jpeg", "bmp"])
                .pick_file();
            Box::pin(async move {
                dialog.await.map(|file| AsyncFileDialogResult::AddTexture {
                    key,
                    path: path_of(file),
                })
            })
        }
        AsyncFileDialogRequest::AddFont { key, font_size } => {
            let dialog = rfd::AsyncFileDialog::new()
                .add_filter("Font", &["ttf", "otf"])
                .pick_file();
            Box::pin(async move {
                dialog.await.map(|file| AsyncFileDialogResult::AddFont {
                    key,
                    path: path_of(file),
                    font_size,
                })
            })
        }
    }
}

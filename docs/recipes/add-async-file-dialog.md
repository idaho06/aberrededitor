# Recipe: Add a new async file dialog

How to add a new native file or directory picker without blocking the editor loop.

The full path is:

```text
menu/panel click
  -> signal flag
  -> editor_update()
  -> request_async_dialog(...)
  -> poll_async_dialogs()
  -> commands.trigger(MyEvent { ... })
  -> observer does the real work
```

## 1. Decide whether you actually need a dialog

Use the async dialog bridge only when the user must pick a file or folder from the OS.

If the action only needs a key, toggle, or number, keep it as a normal signal -> event ->
observer flow and skip the dialog machinery.

## 2. Collect non-path data first

Gather everything that is not the picked path inside `editor_update()` before you request the
dialog. Typical examples are texture keys, font sizes, or the currently selected entity.

```rust
let key = ctx
    .world_signals
    .get_string(sig::FONT_ADD_KEY_BUF)
    .map(|s| s.to_owned())
    .unwrap_or_default();
let font_size = ctx
    .world_signals
    .get_scalar(sig::FONT_ADD_SIZE_BUF)
    .unwrap_or(32.0);
```

Do not depend on reading that UI state again after the dialog completes. The user may have
changed it while the dialog was open.

## 3. Add a request variant

In `src/systems/file_dialogs.rs`, add a variant to `AsyncFileDialogRequest` that carries the
metadata you need.

```rust
pub enum AsyncFileDialogRequest {
    AddSound { key: String, volume: f32 },
}
```

Add a matching completion variant to `AsyncFileDialogResult`.

```rust
pub enum AsyncFileDialogResult {
    AddSound { key: String, volume: f32, path: String },
}
```

## 4. Build the dialog future

Extend `build_dialog_task()` to construct the correct `rfd::AsyncFileDialog` future and package
its result back into your new result variant. `build_dialog_task` returns
`Pin<Box<dyn Future<Output = Option<AsyncFileDialogResult>> + Send>>` — `None` means the user
cancelled.

```rust
AsyncFileDialogRequest::AddSound { key, volume } => {
    let dialog = rfd::AsyncFileDialog::new()
        .add_filter("Audio", &["wav", "ogg"])
        .pick_file();
    Box::pin(async move {
        dialog.await.map(|file| AsyncFileDialogResult::AddSound {
            key,
            volume,
            path: file.path().to_string_lossy().into_owned(),
        })
    })
}
```

## 5. Request the dialog from editor_update()

In the appropriate `handle_*_actions()` function, enqueue the dialog request instead of opening
`rfd::FileDialog` inline.

```rust
if ctx.world_signals.take_flag(sig::ACTION_SOUND_ADD_BROWSE) && !key.is_empty() {
    request_async_dialog(
        &ctx.app_state,
        AsyncFileDialogRequest::AddSound { key, volume },
    );
}
```

`request_async_dialog` returns `()`. If another dialog is already in flight it silently ignores
the call (logs a debug message). There is currently no feedback to the user that a dialog was
suppressed — if you need that, check `AsyncFileDialogState.receiver.is_some()` before calling.

## 6. Re-emit the existing domain event on completion

Extend `poll_async_dialogs()` to translate your completion back into the event that already owns
the real mutation.

```rust
AsyncFileDialogResult::AddSound { key, volume, path } => {
    commands.trigger(AddSoundRequested {
        key,
        volume,
        path: to_relative(&path),
    });
}
```

This is the place to enforce the relative-path invariant. Do not store absolute paths in
`MapData`, `TextureStore`, `FontStore`, or any other persistent state.

## 7. Keep loading and saving in observers

The async dialog bridge should not load textures, mutate stores, or write files directly. Keep
the actual work in a normal observer that is registered from `main.rs`.

```rust
.add_observer(systems::my_module::add_sound_observer)
```

That preserves the existing architecture and avoids mixing dialog orchestration with resource
mutation.

## 8. Register and persist the bridge state if needed

If you are reusing the existing bridge, nothing new is needed here. If you refactor it or move
it to another module, remember the two wiring points:

- Insert `AsyncFileDialogMutex` in `load_assets()`.
- Register `poll_async_dialogs()` as a per-frame system in `main.rs`.

Without both, requests will either fail immediately or never complete.

## Verification

- `cargo check` passes
- Trigger the action and confirm the native dialog opens
- Confirm the editor remains responsive while the dialog is open
- Confirm cancel is a no-op
- Confirm the downstream observer still performs the same mutation as before
- Confirm stored paths are relative to CWD
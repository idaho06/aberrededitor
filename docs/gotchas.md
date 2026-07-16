# Known Gotchas

Constraints that are not obvious from the code and will burn you if you don't know them.
This file is the documentation counterpart to the "Critical constraints" section in `CLAUDE.md`.

---

## 1. Lua feature gate

`Cargo.toml` sets `default-features = false` on `aberredengine`. The `lua` feature is off.

**Do NOT import from:**

- `aberredengine::resources::lua_runtime`
- `aberredengine::systems::lua_commands`

Both are `#[cfg(feature = "lua")]` and produce non-obvious `E0432` compile errors. The editor
stores Lua callback strings as `SerializedLuaSetup` (a plain `String` component) without
enabling the Lua runtime.

---

## 2. Path relativity invariant

All paths stored in `TextureStore.paths`, `FontStore.meta`, `MapData.textures`,
`MapData.fonts`, `MapData.tilemaps`, and any other persistent store **must be relative to CWD**.
Never absolute.

`rfd` (the file dialog library) returns absolute paths. Always convert immediately:

```rust
let path = to_relative(&path);
commands.trigger(LoadMapRequested { path });
```

`to_relative()` is in `src/systems/utils.rs`. Violating this means maps saved on one machine
won't load on another, and paths won't match between `TextureStore` lookups and `MapData`.

With the async dialog bridge in `src/systems/file_dialogs.rs`, this conversion happens in
`poll_async_dialogs()` right before the downstream event is triggered. Do not normalize paths in
the GUI callback or while constructing the dialog request; the request should carry only user
intent and any non-path metadata it needs.

---

## 3. Logic-thread code can never touch render-owned stores

The engine runs three ECS worlds on three threads (render/logic/audio). `TextureStore`,
`FontStore`, `ShaderStore`, `RenderTarget`, and `RaylibAccess` live **only** on the render thread.
Any logic-side system or observer that requests `Res<TextureStore>`, `NonSend<FontStore>`,
`NonSendMut<ShaderStore>`, or `RaylibAccess` **panics at schedule-init** — not at compile time.
`cargo check` cannot catch this; always do a real `cargo run` after touching asset-loading code.

The fix is to queue a `RenderAssetCmd` via `MessageWriter<RenderAssetCmd>` instead and use
`MapData` for logic-side existence/default-key lookups — see "Why RenderAssetCmd for asset
loading" in `docs/architecture.md` for the full pipeline and variant list, and
`src/systems/map_ops.rs`/`src/systems/tilemap_load.rs`/`src/systems/load_assets.rs` for working
examples.

The **GUI callback** is the one place that still gets the *real*, current `TextureStore`/
`FontStore` directly (it runs on the render thread) — no `RenderAssetCmd` indirection needed
there, only for logic-side code.

---

## 4. Manual resource and AppState insertion

The engine does **not** pre-insert the editor-owned startup resources created in
`load_assets` (`src/systems/load_assets.rs`). Today that includes `MapData` and `EditorState`.
(`TextureStore`/`FontStore`/`ShaderStore` are render-only now — the editor never inserts them;
the intro shaders/texture are loaded by queuing `RenderAssetCmd` instead.)

Similarly, the editor's caches are **not** Bevy resources. They live in `AppState` as
`Arc<Mutex<T>>` values, for example `RenderableSelectorMutex`, `MultiEntitySelectionMutex`,
`AsyncFileDialogMutex`, `GroupListMutex`, `AnimationStoreMutex`, `TemplateSelectorMutex`,
`PendingLuaSetupLoadMutex`, `EditorToolMutex`, `OverlaySettingsMutex`, `PendingMutex`,
`MapPropertiesMutex`, and `RenderPrefsMutex`. The `Arc` wrapper is required because
`AppState::insert` needs `T: Clone` (the render thread's `AppState` is a clone taken every
frame) — the `Arc` clones cheaply while sharing the same `Mutex`-guarded data with the logic-side
original.

If you add a new store or cache and forget to insert it, the first `app_state.get::<T>()` call
will panic with a confusing message. Always insert in `load_assets`.

---

## 5. bevy_ecs derive macro workaround

Do **not** add `bevy_ecs` as a direct dependency in `Cargo.toml`. The engine re-exports it
and must be the single source of truth for the ECS version.

To use `#[derive(Event)]`, `#[derive(Component)]`, etc., add this at the top of the file:

```rust
use aberredengine::bevy_ecs;
```

This brings the `bevy_ecs` identifier into scope so the derive macros can find it. Without
this line you get cryptic macro resolution errors.

Also note that defining the event type is only half of the wiring. If nothing ever observes the
event, it will fail silently. Register the observer with `.add_observer(...)` in `main.rs` or via
the subsystem `register(builder)` helper that `main.rs` calls.

---

## 6. ImGui texture pointer (segfault risk)

When passing a texture to ImGui for rendering (`ui.image`, `draw_list.add_image`, etc.),
pass a pointer to the **full `ffi::Texture2D` struct** — never the raw `.id` field.

The rlImGui C backend dereferences the pointer as a `Texture2D` struct. Passing `.id as usize`
gives it address 1 or 2, which it then dereferences → immediate segfault.

```rust
// CORRECT
let tex_ptr = texture as *const ffi::Texture2D as usize;
ui.image(ImTextureID::new(tex_ptr), [w, h]);

// WRONG — crashes
let tex_ptr = texture.id as usize;  // DO NOT DO THIS
```

See `src/scenes/editor/texture_panel.rs` for the correct pattern.

---

## 7. FontStore/TextureStore/ShaderStore are render-thread-only

These are Raylib resources and only exist on the render thread (the main thread, which owns the
window). Logic-side systems and observers cannot take them as any kind of `SystemParam`
(`Res`, `NonSend`, etc.) — see gotcha #3 for the `RenderAssetCmd` pattern to use instead. Only the
GUI callback (which runs on the render thread) receives them directly, as plain `&TextureStore`/
`&FontStore` references — no `NonSend` wrapper needed there, since it isn't a Bevy system param.

The engine auto-inserts these stores render-side. The editor never inserts them.

---

## 8. EngineBuilder callback styles

Two callback styles exist and **must not be mixed**:

| Registration method | Signature | Example |
| --- | --- | --- |
| `on_setup`, `add_observer`, `add_system` | Bevy system — params **by value** | `fn f(ctx: GameCtx)` |
| Scene `on_enter/update/exit` | Plain fn pointer — params **by ref** | `fn f(ctx: &mut GameCtx)` |
| `gui_callback` | Fixed signature | `fn f(&Ui, &SignalSnapshot, &mut SignalIntents, &TextureStore, &FontStore, &AppState)` |

Using a by-value system function where a by-ref scene callback is expected (or vice versa) is
the most common compile error when adding new scenes or setup functions.

---

## 9. Async dialogs are orchestration, not loading

Do not put `rfd::AsyncFileDialog` directly into GUI code or use it to replace an observer.

The correct split is:

1. GUI or menu sets a signal.
2. `editor_update()` calls `request_async_dialog(...)`.
3. `poll_async_dialogs()` converts the completed selection into the existing domain event.
4. The observer for that domain event performs the real load/save/mutation.

This matters because `GameCtx` still does not expose `RaylibAccess`, `FontStore`/`TextureStore`
are still render-thread-only (see gotcha #3), and the relative-path invariant still has to be
enforced in one place.

Also note that the bridge currently allows only one in-flight dialog. A second request while a
dialog is already open is ignored. If you need queueing or visible UI feedback, build that on
top of the bridge rather than opening multiple native dialogs at once. `poll_async_dialogs()`
must stay registered as a per-frame system in `main.rs`; without it, completions will never be
drained back into ECS.

# Recipe: Add a new menu action

How to add a new menu item that triggers ECS work. The full path is:
menu item → signal flag → `editor_update` → `commands.trigger(Event)` → observer.

## 1. Add a signal constant

In `src/signals.rs`:

```rust
/// Set by the View menu when the user clicks "My Action".
pub const ACTION_MY_FEATURE: &str = "gui:action:my_feature";
```

Use the `gui:action:...` prefix for user-initiated actions.

## 2. Add the menu item

In `src/scenes/editor/menu.rs`, inside `draw_menu_bar`, add to the appropriate menu:

```rust
// Inside ui.menu("View") { ... }
ui.separator();
if ui.menu_item("My Feature") {
    signals.set_flag(sig::ACTION_MY_FEATURE);
}
```

`menu_item` returns `true` when the item is clicked. Setting a flag is the standard way to
communicate the click back to `editor_update`.

If the action is a toggle (like Debug Mode), use:
```rust
if ui.menu_item_config("My Toggle").selected(signals.has_flag(sig::MY_TOGGLE_STATE)).build() {
    signals.set_flag(sig::ACTION_MY_TOGGLE);
}
```

## 3. Decide: GUI-only or ECS mutation?

**GUI-only** (opens a window, resets a view setting, etc.): just set a flag in step 2 that
the relevant panel reads — no `editor_update` or observer needed. Example: the Store windows
are opened this way via `UI_*_OPEN` flags.

**ECS mutation** (modifies entities, files, stores): continue with steps 4–6.

## 4. Define an event

In the appropriate systems file (e.g., `src/systems/map_ops.rs` for map-level actions, or a
new file for a new domain):

```rust
#[derive(Event)]
pub struct MyFeatureRequested {
    // carry any parameters the observer needs
}
```

## 5. Write the observer

```rust
pub fn my_feature_observer(
    _trigger: On<MyFeatureRequested>,
    mut map_data: ResMut<MapData>,
    mut world_signals: ResMut<WorldSignals>,
    // ... other params as needed
) {
    // Do the ECS work
}
```

## 6. Consume the flag in editor_update

In `src/scenes/editor/update.rs`, in the appropriate `handle_*_actions` function (or a new one):

```rust
fn handle_my_actions(ctx: &mut GameCtx) {
    if ctx.world_signals.take_flag(sig::ACTION_MY_FEATURE) {
        ctx.commands.trigger(MyFeatureRequested { ... });
    }
}
```

Call `handle_my_actions(ctx)` from `editor_update`.

**Note:** If the action requires a file dialog, start it from `editor_update()` but do not open
`rfd::FileDialog` inline anymore. Use the async dialog bridge so the editor loop stays responsive:

```rust
if ctx.world_signals.take_flag(sig::ACTION_MY_FEATURE) {
    request_async_dialog(&ctx.app_state, AsyncFileDialogRequest::OpenMap);
}
```

If you need a new dialog shape, add a new request/result variant in `src/systems/file_dialogs.rs`
and map its completion back to your domain event in `poll_async_dialogs()`. Keep asset loading,
map writes, and other real mutations in observers.

## 7. Register the observer in main.rs

In `src/main.rs`:

```rust
.add_observer(systems::my_module::my_feature_observer)
```

## Verification

- `cargo check` passes
- Run the editor, click the menu item — the observer fires (check log output)
- The expected ECS/map state change happens
- If a file dialog was involved, test the cancel path (no crash, no stale state)
- If a file dialog was involved, confirm the editor stays responsive while the native dialog is open

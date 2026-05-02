# Recipe: Add a new editor panel

How to add a new ImGui window panel to the editor. This recipe covers the simplest case: a panel
that reads from `AppState` and writes back via `WorldSignals`.

## 1. Add a signal constant for the open/close flag

In `src/signals.rs`:

```rust
/// Flag: my panel window is open.
pub const UI_MY_PANEL_OPEN: &str = "ui:my_panel:open";
```

If your panel also has action signals (buttons that trigger ECS work), add those too:
```rust
pub const ACTION_MY_PANEL_DO_THING: &str = "gui:action:my_panel:do_thing";
```

Follow the naming conventions already established:
- `ui:...` — window open/close state
- `gui:action:...` — user actions (buttons that trigger ECS events)
- `gui:<domain>:...` — buffer/intermediate state

## 2. Create the panel module

Create `src/scenes/editor/my_panel.rs`:

```rust
//! My panel — shows [...] and lets the user [...].

use crate::signals as sig;
use aberredengine::imgui;
use aberredengine::resources::appstate::AppState;
use aberredengine::resources::worldsignals::WorldSignals;

pub fn draw_my_panel(ui: &imgui::Ui, signals: &mut WorldSignals, app_state: &AppState) {
    if !signals.has_flag(sig::UI_MY_PANEL_OPEN) {
        return;
    }

    let mut window_open = true;
    ui.window("My Panel")
        .size([400.0, 300.0], imgui::Condition::FirstUseEver)
        .opened(&mut window_open)
        .build(|| {
            // Draw panel contents here.
            // Read from app_state caches; write to signals.

            if ui.button("Do Thing") {
                signals.set_flag(sig::ACTION_MY_PANEL_DO_THING);
            }
        });

    // Close the panel if the user clicked X on the window title bar.
    if !window_open {
        signals.clear_flag(sig::UI_MY_PANEL_OPEN);
    }
}
```

If your panel needs modal popups, follow the same pattern as `draw_texture_editor` + 
`draw_texture_modals` in `texture_panel.rs` — return `bool` flags from the panel function
and open the popups in `editor_gui` after calling the panel.

## 3. Declare the module

In `src/scenes/editor/mod.rs`, add:

```rust
mod my_panel;
```

## 4. Call the panel from editor_gui

In `src/scenes/editor/update.rs`, import and call your panel:

```rust
use super::my_panel::draw_my_panel;

pub fn editor_gui(ui: &imgui::Ui, signals: &mut WorldSignals, textures: &TextureStore, fonts: &FontStore, app_state: &AppState) {
    // ... existing panel calls ...
    draw_my_panel(ui, signals, app_state);
    // ...
}
```

## 5. Add a menu item to open the panel

In `src/scenes/editor/menu.rs`, inside `draw_menu_bar`, under the View menu or whichever is
appropriate:

```rust
if ui.menu_item("My Panel") {
    signals.set_flag(sig::UI_MY_PANEL_OPEN);
}
```

## 6. Handle the action signal in editor_update (if needed)

If your panel triggers ECS work via signals, handle it in `src/scenes/editor/update.rs` inside
`handle_view_actions` (or a new `handle_my_panel_actions` function):

```rust
fn handle_my_panel_actions(ctx: &mut GameCtx) {
    if ctx.world_signals.take_flag(sig::ACTION_MY_PANEL_DO_THING) {
        ctx.commands.trigger(MyPanelDoThingRequested { ... });
    }
}
```

Call `handle_my_panel_actions(ctx)` from `editor_update`.

## 7. If the panel needs a live data cache

If the panel needs to show data from ECS (entity lists, resource contents, etc.) that can't be
queried in the GUI callback, use the AppState mutex cache pattern from `docs/patterns.md`.

Short version:
1. Define `MyPanelCache` and `type MyPanelCacheMutex = Mutex<MyPanelCache>;`
2. Insert it in `load_assets()`: `app_state.insert(MyPanelCacheMutex::new(...))`
3. Write a sync system that populates it
4. Register the system in `main.rs`
5. In `draw_my_panel`, read via `app_state.get::<MyPanelCacheMutex>().unwrap().lock().unwrap()`

## Verification

- `cargo check` passes
- Run the editor, open View menu → "My Panel" appears and opens the window
- Close button (X on title bar) hides the window
- If actions trigger ECS work, verify the effect appears in the editor

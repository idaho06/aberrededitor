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
let path = to_relative(&path.display().to_string());
// then store/trigger with `path`
```

`to_relative()` is in `src/systems/utils.rs`. Violating this means maps saved on one machine
won't load on another, and paths won't match between `TextureStore` lookups and `MapData`.

---

## 3. RaylibHandle is not in GameCtx

`GameCtx` and `SceneUpdateFn` callbacks do **not** expose `RaylibHandle`. Runtime asset
loading (textures, tilemaps, fonts) requires Raylib at load time and must go through a Bevy ECS
observer registered via `.add_observer()`.

Pattern for loading an asset on demand:
1. GUI or `editor_update` triggers `MyLoadRequested { path }`.
2. Observer function takes `mut raylib: RaylibAccess` — this is only available in observers.
3. Observer loads the asset and inserts it into the appropriate store.

See `src/systems/tilemap_load.rs` and `src/systems/map_ops.rs` for working examples.

---

## 4. Manual resource insertion (MapData, TilemapStore)

The engine does **not** pre-insert `TilemapStore` or `MapData`. Both must be inserted as Bevy
resources in `load_assets` (`src/systems/load_assets.rs`).

Similarly, `EntitySelectorCache` is NOT a Bevy resource — it lives in `AppState` as
`SelectorMutex` (inserted via `app_state.insert(...)` in `load_assets`).

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

## 7. FontStore is NonSend (main-thread only)

`FontStore` is a Raylib resource and must only be accessed on the main thread. In ECS systems
and observers that need it, use `NonSend<FontStore>` (read) or `NonSendMut<FontStore>` (write).
Using `Res<FontStore>` will cause a panic at runtime.

The engine auto-inserts `FontStore`. Do NOT insert it again in `load_assets`.

---

## 8. EngineBuilder callback styles

Two callback styles exist and **must not be mixed**:

| Registration method | Signature | Example |
|---|---|---|
| `on_setup`, `add_observer`, `add_system` | Bevy system — params **by value** | `fn f(ctx: GameCtx)` |
| Scene `on_enter/update/exit` | Plain fn pointer — params **by ref** | `fn f(ctx: &mut GameCtx)` |
| `gui_callback` | Fixed signature | `fn f(&Ui, &mut WorldSignals, &TextureStore, &FontStore, &AppState)` |

Using a by-value system function where a by-ref scene callback is expected (or vice versa) is
the most common compile error when adding new scenes or setup functions.

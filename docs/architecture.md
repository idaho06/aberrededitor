# Architecture

aberrededitor is a 2D map editor built on aberredengine (Bevy ECS + Raylib). The engine runs
**three ECS worlds on three threads** (render, logic, audio); understanding what crosses that
boundary, and how, is the key to working with this codebase.

## The three-thread model

```text
┌─────────────────────────────────────────────────┐
│  Logic thread  (owns all game state and sim)    │
│                                                 │
│  Resources: WorldSignals, AppState,             │
│             MapData, AnimationStore,            │
│             TextureDimsStore, FontMetricsStore  │
│             (async mirrors of render-side dims) │
│                                                 │
│  Entities: MapEntity + components               │
│                                                 │
│  Observers: fire once on Event                  │
│  Systems: run every frame                       │
│                                                 │
│  Can NEVER take RaylibAccess, NonSend<FontStore>,│
│  NonSend<ShaderStore>, or Res<TextureStore> —    │
│  those live on the render thread and panic at    │
│  schedule-init if requested here.                │
└─────────────────┬───────────────────────────────┘
                  │  DrawableSnapshot (triple buffer): AppState clone,
                  │  GameConfig, SignalSnapshot, camera, …
                  │  RenderAssetCmd (MessageWriter): queued asset loads
                  ▼
┌─────────────────────────────────────────────────┐
│  Render thread  (main): owns the window,        │
│  TextureStore, FontStore, ShaderStore,          │
│  RenderTarget, ImGui                            │
│                                                 │
│  fn editor_gui(                                 │
│    ui: &imgui::Ui,                              │
│    signals: &SignalSnapshot,     ← 1-tick-stale │
│    intents: &mut SignalIntents,  ← queued write │
│    textures: &TextureStore,      ← real, current│
│    fonts: &FontStore,            ← real, current│
│    app_state: &AppState,         ← cloned snap  │
│  )                                              │
└─────────────────┬───────────────────────────────┘
                  │  LogicMsg (channel): ScreenSize, TextureLoaded,
                  │  FontLoaded, TextureRemoved, FontRemoved,
                  │  SignalIntents, Shutdown
                  ▼
            (back to logic thread's mirrored resources)
```

The GUI callback runs on the render thread every frame but cannot query the logic-side ECS
directly. It reads a one-tick-stale `SignalSnapshot`/cloned `AppState`, and queues writes into
`SignalIntents` — applied at the start of the next logic tick. `TextureStore`/`FontStore` are the
*real*, current render-side stores (not snapshots), since GUI code runs on the same thread that
owns them.

## Why SignalSnapshot/SignalIntents as the bridge

`WorldSignals` (logic-side, live) is a typed key-value bus (`scalars`, `integers`, `strings`,
`flags`, `entities`). Its per-frame mirror on the render side is `SignalSnapshot` — a plain-field
read-only snapshot (`signals.flags.contains(k)`, `signals.scalars.get(k)`, no getter methods).
GUI writes go through `SignalIntents` (`intents.set_flag(k)`, `intents.set_scalar(k, v)`, …),
applied to `WorldSignals` at the start of the next logic tick — a one-tick latency accepted
project-wide. Every communication from GUI to ECS goes through this pair:

- GUI sets a flag via `intents` → `editor_update()` reads it from `WorldSignals` next tick →
  triggers an `Event`
- ECS system writes a scalar into `WorldSignals` → rides the next `DrawableSnapshot` → GUI reads
  it from `SignalSnapshot` to project world coordinates to screen

Signal key constants live in `src/signals.rs`. **Never write raw string literals for signal keys.**
Always use the constants via `use crate::signals as sig; ... sig::MY_KEY`.

## Why AppState `Arc<Mutex<T>>` caches

`AppState::insert` requires `T: Clone` — the render thread's `AppState` is a **clone** taken every
frame as part of the `DrawableSnapshot`. Every editor cache type is therefore wrapped
`Arc<Mutex<T>>`: the `Arc` satisfies `Clone` and the clone shares the *same* underlying data with
the logic-side original, guarded by the `Mutex`. GUI and logic genuinely contend on these now —
keep lock scopes short (never hold a lock across a whole panel draw), and never lock two of these
mutexes in different orders across call sites.

ECS data that the GUI needs to display — entity selector hit list, animation store contents, group
list, entity component snapshot — cannot be queried live from the render thread. Instead, dedicated
per-frame systems mirror this data into `Arc<Mutex<T>>` values stored inside `AppState`:

| Mutex type | Populated by | Consumed by |
| --- | --- | --- |
| `RenderableSelectorMutex` | selection observers in `entity_selector.rs` | `draw_entity_selector` |
| `MultiEntitySelectionMutex` | rectangle/group selection observers in `entity_selector.rs` | `draw_multi_entity_selector` |
| `AnimationStoreMutex` | `animation_store_sync_system` | `draw_animation_store` |
| `GroupListMutex` | `update_group_cache` | `draw_groups_window` |
| `TemplateSelectorMutex` | `update_template_cache` | `draw_template_browser` |
| `PendingLuaSetupLoadMutex` | tilemap/Lua setup loading systems | tilemap/Lua setup workflow |
| `PendingMutex` | entity editor panels | `consume_entity_editor_commits` |
| `OverlaySettingsMutex` | overlay toggle helpers and modal UI | `draw_world_overlays` |
| `EditorToolMutex` | `editor_update()` tool handlers | click/drag tool flow in `editor_update()` |
| `MapPropertiesMutex` | map metadata observers and panel helpers | `draw_map_properties_panel` |
| `RenderPrefsMutex` | `camera_sync_system` and render-preferences modal | overlay and render-preference UI |
| `ComponentSnapshot` in AppState | `entity_inspect_observer` | `draw_entity_editor` |
| `AsyncFileDialogMutex` | `request_async_dialog()` + worker thread | `poll_async_dialogs` |

The Mutex provides interior mutability: the ECS system acquires the write lock, the GUI callback
acquires the read lock. Both sides see up-to-date data without sharing mutable refs.
Not every `AppState` entry is a live ECS mirror; some are control-plane caches that bridge GUI
state (`PendingMutex`, `EditorToolMutex`, `AsyncFileDialogMutex`) across frames.

## Why RenderAssetCmd for asset loading

Logic-thread code (observers, systems) can never touch `TextureStore`/`FontStore`/`ShaderStore` —
those resources live on the render thread and taking them (`Res<TextureStore>`,
`NonSendMut<FontStore>`, `RaylibAccess`, …) from a logic-side system panics at schedule-init, not at
compile time (`cargo check` cannot catch this class of bug — always check with a real run after
touching asset-loading code). Instead, logic code queues a `RenderAssetCmd`
(`aberredengine::events::render_assets::RenderAssetCmd`) via `MessageWriter<RenderAssetCmd>`:
`Texture`, `Font`, `Shader`, `TilemapTexture`, `TextureFromMemory`, `ShaderFromMemory`,
`RemoveTexture`, `RemoveFont`. The render thread loads the asset asynchronously and replies via a
`LogicMsg` (`TextureLoaded`/`FontLoaded`/`TextureRemoved`/`FontRemoved`), which the engine mirrors
into logic-side `Res<TextureDimsStore>`/`Res<FontMetricsStore>` — read these with the "tolerate not
loaded yet" pattern (`let Some(v) = store.get(key) else { return/continue }`), never assume the
load already landed.

`MapData` (not the render-side `TextureStore`/`FontStore`) is the logic-side source of truth for
which texture/font keys exist — use it for existence checks, default-key lookups, and dedup, since
the real stores aren't queryable from logic code. GUI-side panels (`texture_panel.rs`,
`font_panel.rs`, …) still read the *real* `TextureStore`/`FontStore` directly, since they run in
`gui_callback` on the render thread.

## Async file dialog bridge

Native file dialogs used to open directly inside `editor_update()`. That kept them out of the
GUI callback, but it still stalled the frame loop while the OS dialog was open. The editor now
routes file and directory pickers through `src/systems/file_dialogs.rs`.

The full path is:

```text
menu/panel click
   -> WorldSignals flag
   -> editor_update()
   -> request_async_dialog(...)
   -> AsyncFileDialog future + worker thread
   -> AppState receiver
   -> poll_async_dialogs() system
   -> commands.trigger(existing domain event)
   -> existing observer does the real work
```

This keeps the architecture consistent with the rest of the editor:

- GUI still only emits signals.
- `editor_update()` still owns action routing.
- The bridge module owns dialog orchestration only.
- Existing map and asset observers still own loading, saving, and store mutation.

`AsyncFileDialogState` lives in `AppState` as a mutex cache just like the selector, group list,
and animation store mirrors. The difference is that this cache stores control-flow state
(`Receiver<...>`) rather than view-model data.

The bridge currently enforces one in-flight native dialog at a time. If a second request arrives
while one dialog is open, `request_async_dialog()` silently ignores it (logs a debug message and
returns). That keeps reentrancy simple and avoids multiple overlapping OS dialogs.

## Selection system and tool flow

Selection is owned by the editor scene update loop, not the GUI callback. `editor_update()` reads
the current `EditorTool` from `EditorToolMutex` and routes input through one of four tool modes:

- `Click` — trigger `PickEntitiesAtPointRequested` at the current world-space mouse position.
- `Rectangle` — maintain a `SelectionDragRect` in `EditorToolState`, then trigger
   `PickEntitiesInRectRequested` when the drag ends.
- `AddEntity` — place a new entity at the clicked world position.
- `AddCollider` — drag out a world-space collider rectangle.

ImGui focus is part of the control path. `editor_gui()` publishes `IMGUI_WANTS_MOUSE` and
`IMGUI_WANTS_KEYBOARD` through `SignalIntents`; `editor_update()` reads those flags on the next
tick and suppresses world picks or placement cancellation when the GUI owned the input.

The selection observers in `src/systems/entity_selector.rs` normalize all selection sources into
two `AppState` caches:

- `RenderableSelectorMutex` stores the single-pick hit list shown in the entity selector panel.
- `MultiEntitySelectionMutex` stores rectangle/group results plus `MultiEntityBulkEditState`
   buffers for bulk move and Z-index adjustments.

`SelectorSource` records whether the current result set came from a click, rectangle, group, or
registry-key selection. The topmost hit from a click pick is auto-selected and triggers
`InspectEntityRequested`; rectangle and group picks can route into the dedicated multi-selection
panel instead.

## Entity editor lifecycle

The entity editor is a two-stage read/edit pipeline built around a snapshot plus pending buffers:

```text
selection request
    -> InspectEntityRequested
    -> entity_inspect_observer
    -> ComponentSnapshot stored in AppState
    -> entity editor panels draw from snapshot
    -> user edits PendingEditState sub-structs
    -> consume_entity_editor_commits()
    -> component commit helpers trigger Update* / Remove* events
    -> entity_edit observers mutate ECS
    -> InspectEntityRequested re-fired
    -> fresh ComponentSnapshot next frame
```

`ComponentSnapshot` is the GUI-facing read model. It is rebuilt by
`entity_inspect_observer` in response to `InspectEntityRequested` and cloned into `AppState`, so
the render-thread GUI can draw entity details without querying ECS directly.

`PendingEditState` is the write model. Each editable component owns a dedicated pending sub-struct
inside `src/scenes/editor/pending_state.rs`; fields are `Option<T>` so `None` means "unchanged"
and `Some(value)` means "apply this on commit". `consume_entity_editor_commits()` clones that
aggregate out of `PendingMutex`, resolves the selected entity plus its current snapshot, delegates
to `components::<name>::commit(...)`, and then clears the pending buffers.

Selection changes are handled separately by `entity_editor_selection_change_system()`. That system
stores the previous entity in the `EditorState` resource and resets `PendingEditState` whenever
`ES_SELECTED_ENTITY` changes, preventing stale buffered values from leaking across selections.

The refresh step is deliberate: entity mutation observers re-trigger `InspectEntityRequested`
after applying changes so the inspector snapshot remains authoritative. The GUI never patches the
snapshot in place.

## Observer dispatch vs per-frame systems

Use **observers** (`#[derive(Event)]` + `.add_observer()`) for one-shot mutations triggered by a
specific action — load map, update a component, register an entity. Observers fire exactly once
per triggering event.

Use **per-frame systems** (`.add_system()`) for continuous background work — syncing caches,
tracking camera state, maintaining the group list. Systems run every frame regardless.

Mixing the two is the most common mistake. If you register a mutation that should happen on every
frame, use a system. If it should happen once in response to a user action, use an observer.

## Scene lifecycle

```text
main()
  └─ EngineBuilder
       ├── on_setup: load_assets        (one-shot setup)
       ├── add_observer(...)            (all observers, registered once)
       ├── add_system(...)              (all per-frame systems)
       ├── add_scene("intro", ...)      → splash with glitch/fade shaders
       └── add_scene("editor", ...)     → main editing interface
```

The intro scene transitions to the editor via:

```rust
ctx.world_signals.set_string("scene", "editor".to_string());
ctx.world_signals.set_flag("switch_scene");
```

On scene transition, all non-`Persistent` entities are despawned by the engine. Editor state
persists because it lives in ECS resources (`WorldSignals`, `AppState`, `MapData`, stores), not in
ephemeral entities.

Scene callbacks use `&mut GameCtx` (by-ref), whereas observers and systems receive params by value.
Do not mix the two patterns — `GameCtx` is a `SystemParam` bundle that only works when borrowed in
a scene context.

## Map data round-trip

**Load:** `.map` JSON file → `load_map_observer` → inserts ECS components + populates `MapData`,
`AnimationStore` (logic-owned) and queues `RenderAssetCmd::Texture`/`Font` per asset (render-owned
stores are populated asynchronously as the commands are processed).

**Edit:** GUI panels write into the per-component sub-structs inside `PendingEditState` →
`consume_entity_editor_commits` clones the aggregate and delegates to
`components::<name>::commit(...)` in `src/scenes/editor/components/` → those commit helpers
trigger `Update*Requested` / `Remove*Requested` events → observers update ECS components and
re-trigger `InspectEntityRequested` → `entity_inspect_observer` rebuilds `ComponentSnapshot` in
`AppState` → GUI shows updated state next frame.

**Save:** `save_map_observer` queries all `MapEntity` components → serializes to `EntityDef` list
→ writes `.map` JSON.

The `MapEntity` marker component (`src/components/map_entity.rs`) scopes all queries to
user-placed entities and excludes internal editor entities (editor camera, selector overlays, etc).

## GuiCallback constraints

The `GuiCallback` signature is fixed by the engine (`fn(&Ui, &SignalSnapshot, &mut SignalIntents,
&TextureStore, &FontStore, &AppState)`). One gotcha:

**Texture pointer safety (segfault risk).** When rendering a `Texture2D` in ImGui, pass a pointer
to the full `ffi::Texture2D` struct — not the raw `.id` field. The rlImGui C backend dereferences
the pointer as a struct. Passing `.id as usize` dereferences address 1 or 2 and crashes
immediately. See `texture_panel.rs` and `font_panel.rs` for the correct pattern. This still holds
under the three-thread engine: `gui_callback` receives the real render-side `TextureStore`, so the
pattern is unchanged — only the import path moved to `resources::render::texturestore`.

## Initialization sequence

1. `main()` — configures `EngineBuilder`, registers all observers and systems, declares scenes
2. `load_assets()` (`on_setup`, logic thread) — queues the intro shaders/texture via
   `RenderAssetCmd` (`ShaderFromMemory`/`TextureFromMemory`); inserts logic-owned ECS resources
   (`MapData`, `EditorState`) plus all `AppState` `Arc<Mutex<_>>` caches
   (`RenderableSelectorMutex`, `MultiEntitySelectionMutex`, `AsyncFileDialogMutex`,
   `GroupListMutex`, `AnimationStoreMutex`, `TemplateSelectorMutex`, `PendingLuaSetupLoadMutex`,
   `EditorToolMutex`, `OverlaySettingsMutex`, `PendingMutex`, `MapPropertiesMutex`,
   `RenderPrefsMutex`). `RenderPrefsMutex` is seeded from `GameConfig.pixel_snap_camera` so the
   GUI preference state starts in sync with the logic-side camera configuration.
3. Intro scene runs, transitions to editor on input or timeout
4. `editor_enter()` — configures camera and input bindings
5. Every frame: all systems run (including `poll_async_dialogs()`) → `editor_update()` processes
   signals → render thread's `editor_gui()` draws ImGui from the latest `DrawableSnapshot`
6. Window resizes: render thread samples the window size every frame → mirrored into logic-side
   `WindowSize` → on change, the engine triggers `WindowResizedEvent`, handled by
   `systems::window_resize::on_window_resized` (`GameConfig::set_render_size` + camera offset) →
   rides the next `DrawableSnapshot` back to the render thread, which recreates the render target

# Architecture

aberrededitor is a 2D map editor built on aberredengine (Bevy ECS + Raylib). Understanding the
two-layer separation between ECS and GUI is the key to working with this codebase.

## The two-layer model

```
┌─────────────────────────────────────────────────┐
│  Bevy ECS  (owns all state and simulation)      │
│                                                 │
│  Resources: WorldSignals, AppState,             │
│             TextureStore, FontStore,            │
│             AnimationStore, MapData             │
│                                                 │
│  Entities: MapEntity + components               │
│                                                 │
│  Observers: fire once on Event                  │
│  Systems: run every frame                       │
└─────────────────┬───────────────────────────────┘
                  │  WorldSignals (mutable)
                  │  AppState (read-only in GUI)
                  │  TextureStore / FontStore (read-only in GUI)
                  ▼
┌─────────────────────────────────────────────────┐
│  ImGui GUI callback  (read+signal overlay)      │
│                                                 │
│  fn editor_gui(                                 │
│    ui: &imgui::Ui,                              │
│    signals: &mut WorldSignals,   ← only mut     │
│    textures: &TextureStore,      ← read-only    │
│    fonts: &FontStore,            ← read-only    │
│    app_state: &AppState,         ← read-only    │
│  )                                              │
└─────────────────────────────────────────────────┘
```

The GUI callback runs every frame but cannot query ECS directly. It reads from pre-mirrored
caches in `AppState` and writes back via `WorldSignals` flags. ECS systems then observe those
flags next frame and mutate state.

## Why WorldSignals as the bridge

`WorldSignals` is a typed key-value bus (`scalars`, `integers`, `strings`, `flags`, `entities`).
It is the only mutable parameter in the `GuiCallback` signature. Every communication from GUI to
ECS goes through it:

- GUI sets a flag → `editor_update()` reads it next frame → triggers an `Event`
- ECS system writes a scalar → GUI reads it to project world coordinates to screen

Signal key constants live in `src/signals.rs`. **Never write raw string literals for signal keys.**
Always use the constants via `use crate::signals as sig; ... sig::MY_KEY`.

## Why AppState mutex caches

`AppState` arrives in the GUI callback as `&AppState` (immutable). ECS data that the GUI needs to
display — entity selector hit list, animation store contents, group list, entity component snapshot
— cannot be queried live. Instead, dedicated per-frame systems mirror this data into `Mutex<T>`
values stored inside `AppState`:

| Mutex type | Populated by | Consumed by |
|---|---|---|
| `RenderableSelectorMutex` | `entity_pick_observer` | `draw_entity_selector` |
| `AnimationStoreMutex` | `animation_store_sync_system` | `draw_animation_store` |
| `GroupListMutex` | `update_group_cache` | `draw_groups_window` |
| `TemplateSelectorMutex` | `update_template_cache` | `draw_template_browser` |
| `PendingMutex` | GUI panels | `consume_entity_editor_commits` |
| `ComponentSnapshot` in AppState | `entity_inspect_observer` | `draw_entity_editor` |

The Mutex provides interior mutability: the ECS system acquires the write lock, the GUI callback
acquires the read lock. Both sides see up-to-date data without sharing mutable refs.

## Observer dispatch vs per-frame systems

Use **observers** (`#[derive(Event)]` + `.add_observer()`) for one-shot mutations triggered by a
specific action — load map, update a component, register an entity. Observers fire exactly once
per triggering event.

Use **per-frame systems** (`.add_system()`) for continuous background work — syncing caches,
tracking camera state, maintaining the group list. Systems run every frame regardless.

Mixing the two is the most common mistake. If you register a mutation that should happen on every
frame, use a system. If it should happen once in response to a user action, use an observer.

## Scene lifecycle

```
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

**Load:** `.map` JSON file → `load_map_observer` → inserts ECS components + populates
`TextureStore`, `FontStore`, `AnimationStore`, `MapData`.

**Edit:** GUI panels modify `PendingEditState` → `consume_entity_editor_commits` triggers
`Update*Requested` / `Remove*Requested` events → observers update ECS components and re-trigger
`InspectEntityRequested` → `entity_inspect_observer` rebuilds `ComponentSnapshot` in `AppState`
→ GUI shows updated state next frame.

**Save:** `save_map_observer` queries all `MapEntity` components → serializes to `EntityDef` list
→ writes `.map` JSON.

The `MapEntity` marker component (`src/components/map_entity.rs`) scopes all queries to
user-placed entities and excludes internal editor entities (editor camera, selector overlays, etc).

## GuiCallback constraints

The `GuiCallback` signature is fixed by the engine. There are two gotchas:

**Texture pointer safety (segfault risk).** When rendering a `Texture2D` in ImGui, pass a pointer
to the full `ffi::Texture2D` struct — not the raw `.id` field. The rlImGui C backend dereferences
the pointer as a struct. Passing `.id as usize` dereferences address 1 or 2 and crashes
immediately. See `texture_panel.rs` and `font_panel.rs` for the correct pattern.

**FontStore is NonSend.** `FontStore` is main-thread-only. In ECS systems that mutate it, request
`NonSendMut<FontStore>`. In observers, use `NonSendMut<FontStore>`. Do not call
`commands.insert_resource(FontStore::new())` — the engine auto-inserts it.

## Initialization sequence

1. `main()` — configures `EngineBuilder`, registers all observers and systems, declares scenes
2. `load_assets()` (`on_setup`) — loads shaders and the intro logo texture; inserts all
   `AppState` mutex caches and ECS resources that the engine doesn't auto-insert (`MapData`,
   `TilemapStore`, the various `Mutex<T>` caches)
3. Intro scene runs, transitions to editor on input or timeout
4. `editor_enter()` — configures camera and input bindings
5. Every frame: all systems run → `editor_update()` processes signals → `editor_gui()` draws ImGui
